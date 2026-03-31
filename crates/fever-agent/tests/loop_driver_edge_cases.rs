use fever_agent::{LoopConfig, LoopDriver, LoopEvent};
use fever_core::{
    Agent, AgentContext, AgentResponse, ExecutionContext, Message, ToolCall, ToolResult,
    ToolResultData,
};
use std::sync::atomic::{AtomicUsize, Ordering};

/// Configurable mock agent that returns pre-configured responses in sequence
struct MockAgent {
    responses: Vec<AgentResponse>,
    call_index: AtomicUsize,
}

impl MockAgent {
    fn new(responses: Vec<AgentResponse>) -> Self {
        Self {
            responses,
            call_index: AtomicUsize::new(0),
        }
    }
}

#[async_trait::async_trait]
impl Agent for MockAgent {
    async fn chat(
        &self,
        _messages: &[Message],
        _context: &AgentContext,
    ) -> fever_core::Result<AgentResponse> {
        let idx = self.call_index.fetch_add(1, Ordering::SeqCst);
        if idx < self.responses.len() {
            Ok(self.responses[idx].clone())
        } else {
            Ok(AgentResponse {
                content: "done".to_string(),
                tool_calls: vec![],
                finish_reason: Some("stop".to_string()),
            })
        }
    }

    async fn call_tools(
        &self,
        _calls: &[ToolCall],
        _context: &ExecutionContext,
    ) -> fever_core::Result<Vec<ToolResult>> {
        // Echo back successful results for each call
        Ok(_calls
            .iter()
            .map(|c| ToolResult {
                call_id: c.id.clone(),
                result: ToolResultData::Success {
                    output: serde_json::json!({"status": "ok"}),
                },
                duration_ms: 1,
            })
            .collect())
    }

    fn name(&self) -> &str {
        "MockAgent"
    }
}

fn stop_response(content: &str) -> AgentResponse {
    AgentResponse {
        content: content.to_string(),
        tool_calls: vec![],
        finish_reason: Some("stop".to_string()),
    }
}

fn tool_call_response(tool_name: &str, content: &str) -> AgentResponse {
    AgentResponse {
        content: content.to_string(),
        tool_calls: vec![ToolCall {
            id: "call_123".to_string(),
            name: tool_name.to_string(),
            arguments: serde_json::json!({"input": "test"}),
        }],
        finish_reason: Some("tool_calls".to_string()),
    }
}

fn make_context() -> AgentContext {
    AgentContext::new("test-session".to_string())
}

#[tokio::test]
async fn test_loop_zero_max_iterations() {
    let agent = MockAgent::new(vec![stop_response("hello")]);
    let mut driver = LoopDriver::new(&agent, LoopConfig::new(0));
    let ctx = make_context();
    let messages = vec![Message::user("hi")];

    let result = driver.run(&messages, &ctx).await.unwrap();
    assert_eq!(result.iterations, 0);
}

#[tokio::test]
async fn test_loop_single_tool_call_then_stop() {
    let agent = MockAgent::new(vec![
        tool_call_response("read_file", "I'll read the file"),
        stop_response("Here's the content"),
    ]);
    let mut driver = LoopDriver::new(&agent, LoopConfig::new(10));
    let ctx = make_context();
    let messages = vec![Message::user("read main.rs")];

    let result = driver.run(&messages, &ctx).await.unwrap();
    assert_eq!(result.iterations, 2);
    assert_eq!(result.total_tool_calls, 1);
    assert_eq!(result.final_response.content, "Here's the content");
}

#[tokio::test]
async fn test_loop_multiple_tool_calls_per_iteration() {
    let agent = MockAgent::new(vec![
        AgentResponse {
            content: "I'll do multiple things".to_string(),
            tool_calls: vec![
                ToolCall {
                    id: "call_1".to_string(),
                    name: "read_file".to_string(),
                    arguments: serde_json::json!({"path": "a.rs"}),
                },
                ToolCall {
                    id: "call_2".to_string(),
                    name: "read_file".to_string(),
                    arguments: serde_json::json!({"path": "b.rs"}),
                },
                ToolCall {
                    id: "call_3".to_string(),
                    name: "grep".to_string(),
                    arguments: serde_json::json!({"pattern": "fn "}),
                },
            ],
            finish_reason: Some("tool_calls".to_string()),
        },
        stop_response("Done reading 3 files"),
    ]);
    let mut driver = LoopDriver::new(&agent, LoopConfig::new(10));
    let ctx = make_context();
    let messages = vec![Message::user("analyze code")];

    let result = driver.run(&messages, &ctx).await.unwrap();
    assert_eq!(result.iterations, 2);
    assert_eq!(result.total_tool_calls, 3);
}

#[tokio::test]
async fn test_loop_finish_reason_stop() {
    let agent = MockAgent::new(vec![stop_response("immediate stop")]);
    let mut driver = LoopDriver::new(&agent, LoopConfig::new(10));
    let ctx = make_context();
    let messages = vec![Message::user("hello")];

    let result = driver.run(&messages, &ctx).await.unwrap();
    assert_eq!(result.iterations, 1);
    assert_eq!(result.total_tool_calls, 0);
    assert_eq!(result.final_response.finish_reason.as_deref(), Some("stop"));
}

#[tokio::test]
async fn test_loop_finish_reason_tool_calls_empty() {
    // Empty tool_calls vec should also terminate the loop
    let agent = MockAgent::new(vec![AgentResponse {
        content: "no tools needed".to_string(),
        tool_calls: vec![],
        finish_reason: Some("stop".to_string()),
    }]);
    let mut driver = LoopDriver::new(&agent, LoopConfig::new(10));
    let ctx = make_context();
    let messages = vec![Message::user("hello")];

    let result = driver.run(&messages, &ctx).await.unwrap();
    assert_eq!(result.iterations, 1);
    assert_eq!(result.total_tool_calls, 0);
}

#[tokio::test]
async fn test_loop_max_iterations_reached() {
    // Agent always returns tool calls — loop should hit max_iterations
    let agent = MockAgent::new(vec![
        tool_call_response("read", "reading..."),
        tool_call_response("read", "still reading..."),
        tool_call_response("read", "more reading..."),
        tool_call_response("read", "even more..."),
    ]);
    let mut driver = LoopDriver::new(&agent, LoopConfig::new(3));
    let ctx = make_context();
    let messages = vec![Message::user("keep going")];

    let result = driver.run(&messages, &ctx).await.unwrap();
    assert_eq!(result.iterations, 3);
    assert_eq!(result.total_tool_calls, 3);
    // Should have MaxIterationsReached event
    let has_max_event = result
        .events
        .iter()
        .any(|e| matches!(e, LoopEvent::MaxIterationsReached { .. }));
    assert!(has_max_event, "Expected MaxIterationsReached event");
}

#[tokio::test]
async fn test_loop_events_iteration_started() {
    let agent = MockAgent::new(vec![stop_response("done")]);
    let mut driver = LoopDriver::new(&agent, LoopConfig::new(5));
    let ctx = make_context();
    let messages = vec![Message::user("hi")];

    let result = driver.run(&messages, &ctx).await.unwrap();

    let iteration_events: Vec<_> = result
        .events
        .iter()
        .filter(|e| matches!(e, LoopEvent::IterationStarted { .. }))
        .collect();

    assert_eq!(iteration_events.len(), 1);
    if let LoopEvent::IterationStarted { iteration } = iteration_events[0] {
        assert_eq!(*iteration, 1);
    }
}

#[tokio::test]
async fn test_loop_events_tool_executed() {
    let agent = MockAgent::new(vec![
        tool_call_response("bash", "running command"),
        stop_response("command output"),
    ]);
    let mut driver = LoopDriver::new(&agent, LoopConfig::new(10));
    let ctx = make_context();
    let messages = vec![Message::user("run ls")];

    let result = driver.run(&messages, &ctx).await.unwrap();

    let tool_events: Vec<_> = result
        .events
        .iter()
        .filter(|e| matches!(e, LoopEvent::ToolExecuted { .. }))
        .collect();

    assert_eq!(tool_events.len(), 1);
    if let LoopEvent::ToolExecuted {
        tool_name, success, ..
    } = &tool_events[0]
    {
        assert_eq!(tool_name, "bash");
        assert!(success);
    }
}

#[tokio::test]
async fn test_loop_result_message_history_grows() {
    let agent = MockAgent::new(vec![
        tool_call_response("read", "reading"),
        stop_response("done"),
    ]);
    let mut driver = LoopDriver::new(&agent, LoopConfig::new(10));
    let ctx = make_context();
    let messages = vec![Message::user("read file")];

    let result = driver.run(&messages, &ctx).await.unwrap();

    // Initial: 1 user message
    // After iter 1: +1 assistant + 1 tool result = 3
    // After iter 2: +1 assistant = 4
    assert!(result.message_history.len() > messages.len());
    assert!(result.message_history.len() >= 4);
}

#[tokio::test]
async fn test_loop_with_empty_initial_messages() {
    let agent = MockAgent::new(vec![stop_response("no input needed")]);
    let mut driver = LoopDriver::new(&agent, LoopConfig::new(5));
    let ctx = make_context();
    let messages: Vec<Message> = vec![];

    let result = driver.run(&messages, &ctx).await.unwrap();
    assert_eq!(result.iterations, 1);
    assert_eq!(result.final_response.content, "no input needed");
}
