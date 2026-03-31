use fever_core::{AgentResponse, Message};

// Configuration for the iterative loop driver
#[derive(Clone, Debug)]
pub struct LoopConfig {
    pub max_iterations: usize,          // maximum number of iterations
    pub max_tokens_budget: Option<u64>, // optional token budget (not enforced yet)
}

impl LoopConfig {
    pub fn new(max_iterations: usize) -> Self {
        Self {
            max_iterations,
            max_tokens_budget: None,
        }
    }
}

impl Default for LoopConfig {
    fn default() -> Self {
        Self {
            max_iterations: 20,
            max_tokens_budget: None,
        }
    }
}

// Loop events for observability
#[derive(Clone, Debug)]
pub enum LoopEvent {
    IterationStarted {
        iteration: usize,
    },
    LlmResponseReceived {
        content: String,
        tool_call_count: usize,
    },
    ToolExecuted {
        tool_name: String,
        call_id: String,
        duration_ms: u64,
        success: bool,
    },
    ToolResultsAppended {
        count: usize,
    },
    LoopCompleted {
        iterations: usize,
        total_tool_calls: usize,
    },
    LoopFailed {
        iteration: usize,
        error: String,
    },
    MaxIterationsReached {
        iterations: usize,
    },
}

// Final result of running the loop
#[derive(Clone, Debug)]
pub struct LoopResult {
    pub final_response: AgentResponse,
    pub iterations: usize,
    pub total_tool_calls: usize,
    pub message_history: Vec<Message>,
    pub events: Vec<LoopEvent>,
}

// Core loop driver struct (borrows the Agent for flexibility with any implementation)
pub struct LoopDriver<'a> {
    agent: &'a dyn fever_core::Agent,
    pub config: LoopConfig,
}

impl<'a> LoopDriver<'a> {
    pub fn new(agent: &'a dyn fever_core::Agent, config: LoopConfig) -> Self {
        Self { agent, config }
    }

    pub async fn run(
        &'a mut self,
        initial_messages: &[fever_core::Message],
        context: &fever_core::AgentContext,
    ) -> fever_core::Result<LoopResult> {
        // Local type alias to keep patch self-contained in tests
        type Msg = fever_core::Message;
        // History of messages used for context during the loop
        let mut history: Vec<Msg> = initial_messages.to_vec();
        let mut events: Vec<LoopEvent> = Vec::new();

        let mut iterations: usize = 0;
        let mut total_tool_calls: usize = 0;
        let mut last_response: Option<fever_core::AgentResponse> = None;

        for idx in 0..self.config.max_iterations {
            iterations = idx + 1;
            events.push(LoopEvent::IterationStarted {
                iteration: iterations,
            });

            // 2. Call LLM with history
            let response = self.agent.chat(&history, context).await?;
            total_tool_calls += response.tool_calls.len();
            last_response = Some(response.clone());
            events.push(LoopEvent::LlmResponseReceived {
                content: response.content.clone(),
                tool_call_count: response.tool_calls.len(),
            });

            // 3. Termination condition
            if response.tool_calls.is_empty() || response.finish_reason.as_deref() == Some("stop") {
                // Append assistant content to history before finishing
                history.push(Msg {
                    role: "assistant".to_string(),
                    content: response.content.clone(),
                });
                // Build final result
                let final_response = AgentResponse {
                    content: response.content,
                    tool_calls: response.tool_calls,
                    finish_reason: response.finish_reason.clone(),
                };
                let res = LoopResult {
                    final_response,
                    iterations,
                    total_tool_calls,
                    message_history: history,
                    events,
                };
                return Ok(res);
            }

            // 4. There are tool_calls to execute
            history.push(Msg {
                role: "assistant".to_string(),
                content: response.content.clone(),
            });
            let execution_context = fever_core::ExecutionContext::new(
                "loop_plan".to_string(),
                format!("iteration-{}", iterations),
            );
            let tool_results = self
                .agent
                .call_tools(&response.tool_calls, &execution_context)
                .await?;

            // Emit ToolExecuted events
            for (call, res) in response.tool_calls.iter().zip(tool_results.iter()) {
                let success = matches!(res.result, fever_core::ToolResultData::Success { .. });
                events.push(LoopEvent::ToolExecuted {
                    tool_name: call.name.clone(),
                    call_id: res.call_id.clone(),
                    duration_ms: res.duration_ms,
                    success,
                });
            }

            // 5. Append tool results as messages to history
            for res in tool_results.iter() {
                let json = serde_json::to_string(res).unwrap_or_else(|_| "{}".to_string());
                history.push(Msg {
                    role: "tool".to_string(),
                    content: json,
                });
            }
            events.push(LoopEvent::ToolResultsAppended {
                count: tool_results.len(),
            });
            // Continue to next iteration
        }

        // 6. Max iterations reached
        events.push(LoopEvent::MaxIterationsReached { iterations });
        let final_response = last_response.unwrap_or_else(|| AgentResponse {
            content: String::new(),
            tool_calls: Vec::new(),
            finish_reason: Some("max_iterations_reached".to_string()),
        });
        let res = LoopResult {
            final_response,
            iterations,
            total_tool_calls,
            message_history: history,
            events,
        };
        Ok(res)
    }
}
