use std::sync::atomic::{AtomicUsize, Ordering};

use fever_core::{Agent, AgentContext, AgentResponse, Message, ToolCall, ToolResult};
use fever_agent::{LoopDriver, LoopConfig};

struct MockAgent {
    responses: Vec<AgentResponse>,
    index: AtomicUsize,
}

impl MockAgent {
    fn new(responses: Vec<AgentResponse>) -> Self {
        Self { responses, index: AtomicUsize::new(0) }
    }
}

#[async_trait::async_trait]
impl Agent for MockAgent {
    async fn chat(&self, _messages: &[Message], _context: &AgentContext) -> fever_core::Result<AgentResponse> {
        let i = self.index.fetch_add(1, Ordering::SeqCst);
        Ok(self.responses.get(i).cloned().unwrap_or(AgentResponse {
            content: String::new(),
            tool_calls: Vec::new(),
            finish_reason: Some("stop".to_string()),
        }))
    }

    async fn call_tools(&self, _calls: &[ToolCall], _context: &fever_core::ExecutionContext) -> fever_core::Result<Vec<ToolResult>> {
        // In tests that use run_loop without actual tools, return empty results
        Ok(Vec::new())
    }

    fn name(&self) -> &str { "MockAgent" }
}

#[tokio::test]
async fn loop_terminates_immediately_without_tool_calls() {
    // Arrange: one response with no tool_calls and finish_reason=stop
    let resp = AgentResponse { content: "final".to_string(), tool_calls: vec![], finish_reason: Some("stop".to_string()) };
    let mock = MockAgent::new(vec![resp]);
    let mut driver = LoopDriver::new(&mock, LoopConfig::default());

    let history = vec![Message { role: "user".to_string(), content: "start".to_string() }];
    let ctx = fever_core::AgentContext::new("sess1".to_string());

    // Act
    let result = driver.run(&history, &ctx).await.unwrap();

    // Assert
    assert_eq!(result.iterations, 1);
    assert_eq!(result.total_tool_calls, 0);
    assert_eq!(result.final_response.content, "final");
}

#[tokio::test]
async fn loop_terminates_after_one_tool_round() {
    // First response includes one tool_call, second response ends loop with stop
    let tcall = ToolCall { id: "c1".to_string(), name: "echo".to_string(), arguments: serde_json::json!({}) };
    let resp1 = AgentResponse { content: "round1".to_string(), tool_calls: vec![tcall], finish_reason: Some("in_progress".to_string()) };
    let resp2 = AgentResponse { content: "final".to_string(), tool_calls: Vec::new(), finish_reason: Some("stop".to_string()) };
    let mock = MockAgent::new(vec![resp1, resp2]);
    let mut driver = LoopDriver::new(&mock, LoopConfig { max_iterations: 5, max_tokens_budget: None });

    let history = vec![Message { role: "user".to_string(), content: "start".to_string() }];
    let ctx = fever_core::AgentContext::new("sess2".to_string());

    // Act
    let result = driver.run(&history, &ctx).await.unwrap();

    // Assert: should have at least two iterations: after first chat, tool results would be appended and next chat terminates
    assert_eq!(result.iterations, 2);
    // We didn't actually execute a real tool here, so total_tool_calls should be 1 (from first response)
    assert_eq!(result.total_tool_calls, 1);
}

#[tokio::test]
async fn max_iterations_respected() {
    // No special behavior; ensure loop stops after max_iterations reached
    let resp = AgentResponse { content: "x".to_string(), tool_calls: vec![ToolCall { id: "c1".to_string(), name: "echo".to_string(), arguments: serde_json::json!({}) }], finish_reason: Some("continue".to_string()) };
    let mock = MockAgent::new(vec![resp.clone(), resp.clone()]);
    // Set max_iterations to 2 to force early stop
    let mut driver = LoopDriver::new(&mock, LoopConfig { max_iterations: 2, max_tokens_budget: None });

    let history = vec![Message { role: "user".to_string(), content: "start".to_string() }];
    let ctx = fever_core::AgentContext::new("sess3".to_string());

    let res = driver.run(&history, &ctx).await.unwrap();
    assert_eq!(res.iterations, 2);
}
