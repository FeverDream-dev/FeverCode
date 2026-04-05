use std::sync::Arc;

use futures::StreamExt;

use fever_agent::AgentConfig;
use fever_core::{
    Agent, AgentContext, ExecutionContext, Message as CoreMessage, PermissionGuard, ToolCall,
    ToolRegistry,
};
use fever_providers::ProviderClient;
use fever_providers::models::ChatMessage;
use fever_tui::AgentHandle;
use fever_tui::event::Message;

pub struct FeverAgentHandle {
    pub agent: Arc<fever_agent::FeverAgent>,
    pub provider: Arc<ProviderClient>,
    current_model: Arc<std::sync::RwLock<String>>,
}

impl FeverAgentHandle {
    pub fn new(
        provider: Arc<ProviderClient>,
        tools: Arc<ToolRegistry>,
        config: AgentConfig,
        guard: Arc<std::sync::RwLock<PermissionGuard>>,
    ) -> Self {
        let default_model = config.default_model.clone();
        let current_model = Arc::new(std::sync::RwLock::new(default_model.clone()));
        let mut agent = fever_agent::FeverAgent::new(provider.clone(), config);
        agent = agent.with_tools(tools);
        agent = agent.with_permissions(guard);
        Self {
            agent: Arc::new(agent),
            provider,
            current_model,
        }
    }

    pub fn default_model(&self) -> String {
        self.current_model
            .read()
            .map(|m| m.clone())
            .unwrap_or_default()
    }
}

impl AgentHandle for FeverAgentHandle {
    fn submit(&self, content: String, tx: tokio::sync::mpsc::Sender<Message>) {
        let agent = Arc::clone(&self.agent);
        let provider = Arc::clone(&self.provider);
        let model = self.default_model();
        tokio::spawn(async move {
            let initial_messages = [CoreMessage::user(content)];
            let context = AgentContext::new("tui-session".to_string());

            let result =
                run_streaming_loop(&agent, &provider, &initial_messages, &context, &tx, &model)
                    .await;

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

    fn switch_model(&self, model: String) -> bool {
        if let Ok(mut current) = self.current_model.write() {
            *current = model;
            true
        } else {
            false
        }
    }
}

async fn run_streaming_loop(
    agent: &fever_agent::FeverAgent,
    provider: &ProviderClient,
    initial_messages: &[CoreMessage],
    context: &AgentContext,
    tx: &tokio::sync::mpsc::Sender<Message>,
    model_override: &str,
) -> Result<(), String> {
    let mut history: Vec<CoreMessage> = initial_messages.to_vec();
    let max_iterations = 20;
    let request_timeout = std::time::Duration::from_secs(120);

    for iteration in 0..max_iterations {
        let request = agent.prepare_request(&history, context).await;
        let mut stream_request = request.clone();
        stream_request.stream = true;
        stream_request.model = model_override.to_string();

        let stream_result =
            tokio::time::timeout(request_timeout, provider.chat_stream(&stream_request))
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

        let has_tool_calls = finish_reason.as_deref() == Some("tool_calls")
            || finish_reason.as_deref() == Some("function_call");

        if has_tool_calls {
            let mut detect_messages = vec![ChatMessage {
                role: "system".to_string(),
                content: request
                    .messages
                    .first()
                    .map(|m| m.content.clone())
                    .unwrap_or_default(),
                tool_calls: None,
                tool_call_id: None,
            }];
            for msg in &history {
                detect_messages.push(ChatMessage {
                    role: msg.role.clone(),
                    content: msg.content.clone(),
                    tool_calls: None,
                    tool_call_id: None,
                });
            }
            detect_messages.push(ChatMessage {
                role: "assistant".to_string(),
                content: response_content.clone(),
                tool_calls: None,
                tool_call_id: None,
            });

            let mut detect_request = request;
            detect_request.messages = detect_messages;
            detect_request.stream = false;
            detect_request.temperature = Some(0.0);

            let tool_response =
                tokio::time::timeout(request_timeout, provider.chat(&detect_request))
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
                "tui-loop".to_string(),
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
