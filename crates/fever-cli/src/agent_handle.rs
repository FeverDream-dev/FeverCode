use std::sync::Arc;

use fever_agent::AgentConfig;
use fever_core::{Agent, AgentContext, ExecutionContext, Message as CoreMessage, PermissionGuard, ToolRegistry};
use fever_providers::ProviderClient;
use fever_tui::AgentHandle;
use fever_tui::event::Message;

pub struct FeverAgentHandle {
    pub agent: Arc<fever_agent::FeverAgent>,
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
        let mut agent = fever_agent::FeverAgent::new(provider, config);
        agent = agent.with_tools(tools);
        agent = agent.with_permissions(guard);
        Self {
            agent: Arc::new(agent),
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
        tokio::spawn(async move {
            let initial_messages = [CoreMessage::user(content)];
            let context = AgentContext::new("tui-session".to_string());

            let result = run_loop_with_events(&agent, &initial_messages, &context, &tx).await;

            match result {
                Ok(final_content) => {
                    for ch in final_content.chars() {
                        if tx
                            .send(Message::StreamChunk {
                                content: ch.to_string(),
                            })
                            .await
                            .is_err()
                        {
                            return;
                        }
                        tokio::time::sleep(std::time::Duration::from_millis(8)).await;
                    }
                    let _ = tx.send(Message::StreamEnd).await;
                }
                Err(err_msg) => {
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
            }
        });
    }
}

/// Run the agent loop with real-time event streaming to the TUI.
/// Returns the final text response on success, or an error message on failure.
async fn run_loop_with_events(
    agent: &fever_agent::FeverAgent,
    initial_messages: &[CoreMessage],
    context: &AgentContext,
    tx: &tokio::sync::mpsc::Sender<Message>,
) -> Result<String, String> {
    type Msg = CoreMessage;
    let mut history: Vec<Msg> = initial_messages.to_vec();
    let max_iterations = 20;
    let request_timeout = std::time::Duration::from_secs(120);

    for iteration in 0..max_iterations {
        let response = tokio::time::timeout(request_timeout, agent.chat(&history, context))
            .await
            .map_err(|_| format!("LLM request timed out after {}s on iteration {}", request_timeout.as_secs(), iteration + 1))?
            .map_err(|e| format!("LLM error on iteration {}: {}", iteration + 1, e))?;

        if response.tool_calls.is_empty() || response.finish_reason.as_deref() == Some("stop") {
            history.push(Msg {
                role: "assistant".to_string(),
                content: response.content.clone(),
            });
            return Ok(response.content);
        }

        for call in &response.tool_calls {
            let args_str = serde_json::to_string(&call.arguments).unwrap_or_default();
            let _ = tx
                .send(Message::ToolCallStarted {
                    tool: call.name.clone(),
                    args: args_str,
                })
                .await;
        }

        history.push(Msg {
            role: "assistant".to_string(),
            content: response.content.clone(),
        });

        let execution_context = ExecutionContext::new(
            "loop_plan".to_string(),
            format!("iteration-{}", iteration + 1),
        );

        let tool_results = agent
            .call_tools(&response.tool_calls, &execution_context)
            .await
            .map_err(|e| format!("Tool execution error: {}", e))?;

        for (call, result) in response.tool_calls.iter().zip(tool_results.iter()) {
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
            history.push(Msg {
                role: "tool".to_string(),
                content: json,
            });
        }
    }

    Err(format!(
        "Agent loop reached maximum iterations ({})",
        max_iterations
    ))
}
