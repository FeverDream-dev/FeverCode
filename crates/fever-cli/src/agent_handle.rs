use std::sync::Arc;

use fever_agent::{AgentConfig, FeverAgent};
use fever_core::{AgentContext, Message as CoreMessage, ToolRegistry};
use fever_providers::ProviderClient;
use fever_tui::AgentHandle;
use fever_tui::event::Message;

pub struct FeverAgentHandle {
    agent: Arc<FeverAgent>,
    default_model: String,
}

impl FeverAgentHandle {
    pub fn new(
        provider: Arc<ProviderClient>,
        tools: Arc<ToolRegistry>,
        config: AgentConfig,
    ) -> Self {
        let default_model = config.default_model.clone();
        let mut agent = FeverAgent::new(provider, config);
        agent = agent.with_tools(tools);
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
            let messages = [CoreMessage::user(content)];
            let context = AgentContext::new("tui-session".to_string());

            match agent.run_loop(&messages, &context).await {
                Ok(result) => {
                    let text = result.final_response.content;
                    for ch in text.chars() {
                        if tx
                            .send(Message::StreamChunk {
                                content: ch.to_string(),
                            })
                            .await
                            .is_err()
                        {
                            break;
                        }
                        tokio::time::sleep(std::time::Duration::from_millis(8)).await;
                    }

                    for event in &result.events {
                        if let fever_agent::LoopEvent::ToolExecuted {
                            tool_name,
                            call_id: _,
                            duration_ms: _,
                            success,
                        } = event
                        {
                            let _ = tx
                                .send(Message::ToolCallStarted {
                                    tool: tool_name.clone(),
                                    args: String::new(),
                                })
                                .await;
                            if *success {
                                let _ = tx
                                    .send(Message::ToolCallCompleted {
                                        tool: tool_name.clone(),
                                        result: "ok".to_string(),
                                    })
                                    .await;
                            } else {
                                let _ = tx
                                    .send(Message::ToolCallFailed {
                                        tool: tool_name.clone(),
                                        error: "failed".to_string(),
                                    })
                                    .await;
                            }
                        }
                    }

                    let _ = tx.send(Message::StreamEnd).await;
                }
                Err(e) => {
                    let err_msg = format!("Agent error: {}", e);
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
