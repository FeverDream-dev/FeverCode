use fever_core::*;
use serde_json::{Value, json};
use std::collections::HashSet;

struct MockTool {
    tool_name: String,
    tool_desc: String,
}

impl MockTool {
    fn new(name: &str, desc: &str) -> Self {
        Self {
            tool_name: name.to_string(),
            tool_desc: desc.to_string(),
        }
    }
}

#[async_trait::async_trait]
impl Tool for MockTool {
    fn name(&self) -> &str {
        &self.tool_name
    }

    fn description(&self) -> &str {
        &self.tool_desc
    }

    async fn execute(&self, args: Value, _context: &ExecutionContext) -> Result<Value> {
        Ok(args)
    }

    fn schema(&self) -> ToolSchema {
        ToolSchema {
            name: self.tool_name.clone(),
            description: self.tool_desc.clone(),
            parameters: json!({"type": "object"}),
        }
    }
}

// ── Tool / ToolRegistry ──

#[test]
fn test_tool_registry_register_and_get() {
    let mut registry = ToolRegistry::new();
    let tool = MockTool::new("echo", "echoes args");
    registry.register(Box::new(tool)).unwrap();

    let retrieved = registry.get("echo");
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().name(), "echo");
}

#[test]
fn test_tool_registry_list() {
    let mut registry = ToolRegistry::new();
    registry
        .register(Box::new(MockTool::new("alpha", "first")))
        .unwrap();
    registry
        .register(Box::new(MockTool::new("beta", "second")))
        .unwrap();

    let names = registry.list();
    assert_eq!(names.len(), 2);
    assert!(names.contains(&"alpha".to_string()));
    assert!(names.contains(&"beta".to_string()));
}

#[test]
fn test_tool_registry_get_missing_returns_none() {
    let registry = ToolRegistry::new();
    assert!(registry.get("nonexistent").is_none());
}

#[tokio::test]
async fn test_tool_registry_execute_call_success() {
    let mut registry = ToolRegistry::new();
    registry
        .register(Box::new(MockTool::new("echo", "echoes args")))
        .unwrap();

    let call = ToolCall {
        id: "call_1".to_string(),
        name: "echo".to_string(),
        arguments: json!({"key": "value"}),
    };
    let ctx = ExecutionContext::new("plan1".to_string(), "task1".to_string());

    let result = registry.execute_call(&call, &ctx).await.unwrap();
    assert_eq!(result.call_id, "call_1");
    match &result.result {
        ToolResultData::Success { output } => assert_eq!(output, &json!({"key": "value"})),
        _ => panic!("Expected Success"),
    }
}

#[tokio::test]
async fn test_tool_registry_execute_call_missing_tool() {
    let registry = ToolRegistry::new();
    let call = ToolCall {
        id: "call_x".to_string(),
        name: "nonexistent".to_string(),
        arguments: json!({}),
    };
    let ctx = ExecutionContext::new("p".to_string(), "t".to_string());

    let result = registry.execute_call(&call, &ctx).await;
    assert!(result.is_err());
}

#[test]
fn test_tool_registry_schemas() {
    let mut registry = ToolRegistry::new();
    registry
        .register(Box::new(MockTool::new("test_tool", "a test tool")))
        .unwrap();

    let schemas = registry.schemas();
    assert_eq!(schemas.len(), 1);
    assert_eq!(schemas[0].name, "test_tool");
}

#[test]
fn test_generate_call_id_format() {
    let id = generate_call_id();
    assert!(id.starts_with("call_"));
    assert!(id.len() > 5);
}

#[test]
fn test_generate_call_id_unique() {
    let id1 = generate_call_id();
    let id2 = generate_call_id();
    assert_ne!(id1, id2);
}

// ── Task / Plan / Todo ──

#[test]
fn test_task_new_default_state() {
    let task = Task::new("Test task".to_string(), "Do something".to_string());
    assert_eq!(task.title, "Test task");
    assert_eq!(task.description, "Do something");
    assert_eq!(task.status, TaskStatus::Queued);
    assert!(task.dependencies.is_empty());
    assert!(uuid::Uuid::parse_str(&task.id).is_ok());
}

#[test]
fn test_task_with_dependencies() {
    let task = Task::new("T".to_string(), "D".to_string())
        .with_dependencies(vec!["dep1".to_string(), "dep2".to_string()]);
    assert_eq!(task.dependencies.len(), 2);
    assert!(task.dependencies.contains(&"dep1".to_string()));
}

#[test]
fn test_task_can_start_no_deps() {
    let task = Task::new("T".to_string(), "D".to_string());
    let completed = HashSet::new();
    assert!(task.can_start(&completed));
}

#[test]
fn test_task_can_start_with_unmet_deps() {
    let task =
        Task::new("T".to_string(), "D".to_string()).with_dependencies(vec!["missing".to_string()]);
    let completed = HashSet::new();
    assert!(!task.can_start(&completed));
}

#[test]
fn test_task_can_start_with_met_deps() {
    let task =
        Task::new("T".to_string(), "D".to_string()).with_dependencies(vec!["dep1".to_string()]);
    let mut completed = HashSet::new();
    completed.insert("dep1".to_string());
    assert!(task.can_start(&completed));
}

#[test]
fn test_task_can_start_not_queued() {
    let task = Task::new("T".to_string(), "D".to_string());
    let task_id = task.id.clone();
    let mut plan = Plan::new("P".to_string());
    plan.add_task(task);
    plan.update_task_status(&task_id, TaskStatus::Running)
        .unwrap();
    assert!(!plan.tasks[0].can_start(&HashSet::new()));
}

#[test]
fn test_plan_new() {
    let plan = Plan::new("My Plan".to_string());
    assert_eq!(plan.title, "My Plan");
    assert!(plan.tasks.is_empty());
    assert!(uuid::Uuid::parse_str(&plan.id).is_ok());
}

#[test]
fn test_plan_add_task() {
    let mut plan = Plan::new("P".to_string());
    let task = Task::new("T1".to_string(), "D1".to_string());
    let updated_before = plan.updated_at;
    plan.add_task(task);
    assert_eq!(plan.tasks.len(), 1);
    assert!(plan.updated_at >= updated_before);
}

#[test]
fn test_plan_get_task() {
    let mut plan = Plan::new("P".to_string());
    let task = Task::new("Find me".to_string(), "desc".to_string());
    let task_id = task.id.clone();
    plan.add_task(task);

    assert!(plan.get_task(&task_id).is_some());
    assert_eq!(plan.get_task(&task_id).unwrap().title, "Find me");
    assert!(plan.get_task("nonexistent").is_none());
}

#[test]
fn test_plan_update_task_status_running() {
    let mut plan = Plan::new("P".to_string());
    let task = Task::new("T".to_string(), "D".to_string());
    let task_id = task.id.clone();
    plan.add_task(task);

    plan.update_task_status(&task_id, TaskStatus::Running)
        .unwrap();
    assert_eq!(plan.tasks[0].status, TaskStatus::Running);
    assert!(plan.tasks[0].started_at.is_some());
}

#[test]
fn test_plan_update_task_status_completed() {
    let mut plan = Plan::new("P".to_string());
    let task = Task::new("T".to_string(), "D".to_string());
    let task_id = task.id.clone();
    plan.add_task(task);

    plan.update_task_status(&task_id, TaskStatus::Completed)
        .unwrap();
    assert_eq!(plan.tasks[0].status, TaskStatus::Completed);
    assert!(plan.tasks[0].completed_at.is_some());
}

#[test]
fn test_plan_update_task_status_not_found() {
    let mut plan = Plan::new("P".to_string());
    let result = plan.update_task_status("nonexistent", TaskStatus::Running);
    assert!(result.is_err());
}

#[test]
fn test_todo_new() {
    let todo = Todo::new("Fix bug".to_string(), "high".to_string());
    assert_eq!(todo.content, "Fix bug");
    assert_eq!(todo.priority, "high");
    assert_eq!(todo.status, TaskStatus::Queued);
}

// ── Event / EventBus ──

#[tokio::test]
async fn test_event_bus_subscribe_and_publish() {
    let mut bus = EventBus::new();
    let mut rx = bus.subscribe();

    bus.publish(Event::Message {
        content: "hello".to_string(),
        role: "user".to_string(),
    })
    .await;

    let received = rx.recv().await.unwrap();
    assert!(matches!(received, Event::Message { content, .. } if content == "hello"));
}

#[tokio::test]
async fn test_event_bus_multiple_subscribers() {
    let mut bus = EventBus::new();
    let mut rx1 = bus.subscribe();
    let mut rx2 = bus.subscribe();

    bus.publish(Event::PlanCreated {
        plan_id: "p1".to_string(),
        title: "Plan".to_string(),
    })
    .await;

    let e1 = rx1.recv().await.unwrap();
    let e2 = rx2.recv().await.unwrap();
    assert!(matches!(e1, Event::PlanCreated { .. }));
    assert!(matches!(e2, Event::PlanCreated { .. }));
}

#[tokio::test]
async fn test_event_bus_dead_subscriber_cleanup() {
    let mut bus = EventBus::new();
    {
        let _rx = bus.subscribe();
    }

    bus.publish(Event::StatusChanged {
        status: "ok".to_string(),
    })
    .await;

    assert_eq!(bus.subscribe().len(), 0);
}

// ── ExecutionContext / ExecutionEngine ──

#[tokio::test]
async fn test_execution_context_set_get_variable() {
    let ctx = ExecutionContext::new("plan1".to_string(), "task1".to_string());
    ctx.set_variable("key".to_string(), json!("value")).await;
    let val = ctx.get_variable("key").await;
    assert_eq!(val, Some(json!("value")));
}

#[tokio::test]
async fn test_execution_context_get_missing_variable() {
    let ctx = ExecutionContext::new("plan1".to_string(), "task1".to_string());
    assert!(ctx.get_variable("nonexistent").await.is_none());
}

#[test]
fn test_execution_engine_new() {
    let registry = ToolRegistry::new();
    let _engine = ExecutionEngine::new(registry);
}

// ── Agent types ──

#[test]
fn test_message_user() {
    let msg = Message::user("hello");
    assert_eq!(msg.role, "user");
    assert_eq!(msg.content, "hello");
}

#[test]
fn test_message_assistant() {
    let msg = Message::assistant("response");
    assert_eq!(msg.role, "assistant");
    assert_eq!(msg.content, "response");
}

#[test]
fn test_message_system() {
    let msg = Message::system("instructions");
    assert_eq!(msg.role, "system");
    assert_eq!(msg.content, "instructions");
}

#[test]
fn test_agent_context_new() {
    let ctx = AgentContext::new("sess-123".to_string());
    assert_eq!(ctx.session_id, "sess-123");
    assert!(ctx.plan_id.is_none());
    assert_eq!(ctx.current_role, "default");
}

// ── Error / TaskStatus ──

#[test]
fn test_error_display() {
    let err = Error::Config("bad config".to_string());
    let msg = err.to_string();
    assert!(msg.contains("Configuration error"));
    assert!(msg.contains("bad config"));
}

#[test]
fn test_error_provider_display() {
    let err = Error::Provider("timeout".to_string());
    assert!(err.to_string().contains("Provider error"));
}

#[test]
fn test_task_status_display() {
    assert_eq!(TaskStatus::Queued.to_string(), "queued");
    assert_eq!(TaskStatus::Running.to_string(), "running");
    assert_eq!(TaskStatus::Completed.to_string(), "completed");
    assert_eq!(TaskStatus::Failed.to_string(), "failed");
    assert_eq!(TaskStatus::Blocked.to_string(), "blocked");
}

#[test]
fn test_tool_call_serialization_roundtrip() {
    let call = ToolCall {
        id: "call_abc".to_string(),
        name: "bash".to_string(),
        arguments: json!({"command": "ls"}),
    };
    let serialized = serde_json::to_string(&call).unwrap();
    let deserialized: ToolCall = serde_json::from_str(&serialized).unwrap();
    assert_eq!(deserialized.id, call.id);
    assert_eq!(deserialized.name, call.name);
    assert_eq!(deserialized.arguments, call.arguments);
}

#[test]
fn test_tool_result_data_serialization() {
    let success = ToolResultData::Success {
        output: json!({"files": ["a.rs"]}),
    };
    let serialized = serde_json::to_string(&success).unwrap();
    assert!(serialized.contains("Success"));

    let error = ToolResultData::Error {
        message: "file not found".to_string(),
    };
    let serialized = serde_json::to_string(&error).unwrap();
    assert!(serialized.contains("Error"));
}
