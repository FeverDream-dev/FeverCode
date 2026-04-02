use std::sync::Arc;

use futures::StreamExt;

use fever_agent::AgentConfig;
use fever_core::{
    Agent, AgentContext, ExecutionContext, Message as CoreMessage, PermissionGuard, ToolCall,
    ToolRegistry,
};
use fever_providers::ProviderClient;
use fever_providers::models::{ChatMessage, ChatRequest};
use fever_tui::AgentHandle;
use fever_tui::event::Message;

pub struct FeverAgentHandle {
    pub agent: Arc<fever_agent::FeverAgent>,
    pub provider: Arc<ProviderClient>,
    default_model: String,
}

impl FeverAgentHandle {
    pub fn new(
        provider: Arc<ProviderClient>,
        tools: Arc<ToolRegistry>,
        config: AgentConfig,
        guard: Arc<std::sync::RwLock<PermissionGuard>>,
    ) -> Self {
        let default_model = config.default_model.clone();
        let mut agent = fever_agent::FeverAgent::new(provider.clone(), config);
        agent = agent.with_tools(tools);
        agent = agent.with_permissions(guard);
        Self {
            agent: Arc::new(agent),
            provider,
            default_model,
        }
    }

    pub fn default_model(&self) -> &str {
        &self.default_model
    }
}

impl AgentHandle for FeverAgentHandle {
    fn submit(&self, content: String, tx: tokio::sync::mpsc::Sender<Message>) {
        let agent = Arc::clone(&self.agent);
        let provider = Arc::clone(&self.provider);
        tokio::spawn(async move {
            let initial_messages = [CoreMessage::user(content)];
            let context = AgentContext::new("tui-session".to_string());

            let result =
                run_streaming_loop(&agent, &provider, &initial_messages, &context, &tx).await;

            if let Err(err_msg) = result {
                for ch in err_msg.chars() {
                    if tx
                        .send(Message::StreamChunk {
                            content: ch.to_string(),
                        })
                        .await
                        .is_err()
                    {
                        break;
                    }
                }
                let _ = tx.send(Message::StreamEnd).await;
            }
        });
    }
}

/// Streaming agent loop that uses `chat_stream` for real-time output.
/// On tool calls: collects the response, executes tools, then makes a follow-up `chat` call.
/// Streams text chunks to the TUI as they arrive.
async fn run_streaming_loop(
    agent: &fever_agent::FeverAgent,
    provider: &ProviderClient,
    initial_messages: &[CoreMessage],
    context: &AgentContext,
    tx: &tokio::sync::mpsc::Sender<Message>,
) -> Result<(), String> {
    let mut history: Vec<CoreMessage> = initial_messages.to_vec();
    let max_iterations = 20;
    let request_timeout = std::time::Duration::from_secs(120);

    // Build the initial request with system prompt from the agent's role
    let role = agent.get_current_role();
    let system_content = format!("{}\n\nContext: {}", role.system_prompt, context.metadata,);

    for iteration in 0..max_iterations {
        let mut chat_messages = vec![ChatMessage {
            role: "system".to_string(),
            content: system_content.clone(),
            tool_calls: None,
            tool_call_id: None,
        }];

        for msg in &history {
            chat_messages.push(ChatMessage {
                role: msg.role.clone(),
                content: msg.content.clone(),
                tool_calls: None,
                tool_call_id: None,
            });
        }

        let tools = match &agent.tools {
            Some(t) => t,
            None => return Err("No tools registered".to_string()),
        };
        let tool_definitions: Vec<_> = tools
            .schemas()
            .into_iter()
            .map(|s| fever_providers::ToolDefinition {
                name: s.name,
                description: s.description,
                parameters: s.parameters,
            })
            .collect();

        let request = ChatRequest {
            model: agent.default_model().to_string(),
            messages: chat_messages,
            tools: Some(tool_definitions.clone()),
            temperature: Some(0.7),
            max_tokens: Some(4096),
            stream: true,
        };

        // Stream the response — SSE chunks arrive in real-time from the provider
        let stream_result = tokio::time::timeout(request_timeout, provider.chat_stream(&request))
            .await
            .map_err(|_| {
                format!(
                    "LLM request timed out after {}s on iteration {}",
                    request_timeout.as_secs(),
                    iteration + 1
                )
            })?
            .map_err(|e| format!("LLM stream error on iteration {}: {}", iteration + 1, e))?;

        let mut stream = stream_result;
        let mut response_content = String::new();
        let mut finish_reason = None;

        while let Some(chunk_result) = stream.next().await {
            match chunk_result {
                Ok(chunk) => {
                    if let Some(content) = &chunk.content {
                        let _ = tx
                            .send(Message::StreamChunk {
                                content: content.clone(),
                            })
                            .await;
                        response_content.push_str(content);
                    }
                    if let Some(reason) = &chunk.finish_reason {
                        finish_reason = Some(reason.clone());
                    }
                }
                Err(e) => {
                    return Err(format!("Stream error: {}", e));
                }
            }
        }

        // Check if the response has tool calls (non-streaming tool calls come in the final chunk)
        // For now, after streaming, we need to check if the model wants to use tools.
        // We do this by making a non-streaming call with the accumulated content to detect tool calls.
        let has_tool_calls = finish_reason.as_deref() == Some("tool_calls")
            || finish_reason.as_deref() == Some("function_call");

        if has_tool_calls {
            let mut retry_messages = vec![ChatMessage {
                role: "system".to_string(),
                content: system_content.clone(),
                tool_calls: None,
                tool_call_id: None,
            }];
            for msg in &history {
                retry_messages.push(ChatMessage {
                    role: msg.role.clone(),
                    content: msg.content.clone(),
                    tool_calls: None,
                    tool_call_id: None,
                });
            }
            retry_messages.push(ChatMessage {
                role: "assistant".to_string(),
                content: response_content.clone(),
                tool_calls: None,
                tool_call_id: None,
            });

            let tool_request = ChatRequest {
                model: agent.default_model().to_string(),
                messages: retry_messages,
                tools: Some(tool_definitions.clone()),
                temperature: Some(0.0),
                max_tokens: Some(4096),
                stream: false,
            };

            let tool_response = tokio::time::timeout(request_timeout, provider.chat(&tool_request))
                .await
                .map_err(|_| {
                    format!(
                        "Tool-call request timed out after {}s on iteration {}",
                        request_timeout.as_secs(),
                        iteration + 1
                    )
                })?
                .map_err(|e| {
                    format!("Tool-call LLM error on iteration {}: {}", iteration + 1, e)
                })?;

            let choice = tool_response
                .choices
                .first()
                .ok_or_else(|| "No response from provider".to_string())?;

            let tool_calls: Vec<ToolCall> = choice
                .message
                .tool_calls
                .as_ref()
                .map(|tc| {
                    tc.iter()
                        .map(|c| ToolCall {
                            id: c.id.clone(),
                            name: c.name.clone(),
                            arguments: c.arguments.clone(),
                        })
                        .collect()
                })
                .unwrap_or_default();

            if tool_calls.is_empty() {
                history.push(CoreMessage {
                    role: "assistant".to_string(),
                    content: response_content,
                });
                let _ = tx.send(Message::StreamEnd).await;
                return Ok(());
            }

            for call in &tool_calls {
                let args_str = serde_json::to_string(&call.arguments).unwrap_or_default();
                let _ = tx
                    .send(Message::ToolCallStarted {
                        tool: call.name.clone(),
                        args: args_str,
                    })
                    .await;
            }

            history.push(CoreMessage {
                role: "assistant".to_string(),
                content: response_content,
            });

            let execution_context = ExecutionContext::new(
                "loop_plan".to_string(),
                format!("iteration-{}", iteration + 1),
            );

            let tool_results = agent
                .call_tools(&tool_calls, &execution_context)
                .await
                .map_err(|e| format!("Tool execution error: {}", e))?;

            for (call, result) in tool_calls.iter().zip(tool_results.iter()) {
                let is_success =
                    matches!(result.result, fever_core::ToolResultData::Success { .. });
                let result_str = match &result.result {
                    fever_core::ToolResultData::Success { output } => {
                        let s = serde_json::to_string(output).unwrap_or_default();
                        if s.len() > 500 {
                            format!("{}...", &s[..500])
                        } else {
                            s
                        }
                    }
                    fever_core::ToolResultData::Error { message } => message.clone(),
                };

                if is_success {
                    let _ = tx
                        .send(Message::ToolCallCompleted {
                            tool: call.name.clone(),
                            result: format!("{} ({:.0}ms)", result_str, result.duration_ms),
                        })
                        .await;
                } else {
                    let _ = tx
                        .send(Message::ToolCallFailed {
                            tool: call.name.clone(),
                            error: result_str,
                        })
                        .await;
                }
            }

            for result in &tool_results {
                let json = serde_json::to_string(result).unwrap_or_else(|_| "{}".to_string());
                history.push(CoreMessage {
                    role: "tool".to_string(),
                    content: json,
                });
            }
        } else {
            history.push(CoreMessage {
                role: "assistant".to_string(),
                content: response_content,
            });
            let _ = tx.send(Message::StreamEnd).await;
            return Ok(());
        }
    }

    Err(format!(
        "Agent loop reached maximum iterations ({})",
        max_iterations
    ))
}
