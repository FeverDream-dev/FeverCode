use crate::error::{Result, ToolCall, ToolResult};
use crate::execution::ExecutionContext;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[async_trait]
pub trait Agent: Send + Sync {
    async fn chat(&self, messages: &[Message], context: &AgentContext) -> Result<AgentResponse>;

    async fn call_tools(
        &self,
        calls: &[ToolCall],
        context: &ExecutionContext,
    ) -> Result<Vec<ToolResult>>;

    fn name(&self) -> &str;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}

impl Message {
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: "user".to_string(),
            content: content.into(),
        }
    }

    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: "assistant".to_string(),
            content: content.into(),
        }
    }

    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: "system".to_string(),
            content: content.into(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct AgentResponse {
    pub content: String,
    pub tool_calls: Vec<ToolCall>,
    pub finish_reason: Option<String>,
}

#[derive(Debug, Clone)]
pub struct AgentContext {
    pub session_id: String,
    pub plan_id: Option<String>,
    pub current_role: String,
    pub metadata: Value,
}

impl AgentContext {
    pub fn new(session_id: String) -> Self {
        Self {
            session_id,
            plan_id: None,
            current_role: "default".to_string(),
            metadata: Value::Object(Default::default()),
        }
    }
}
