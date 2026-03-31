use rusqlite;
use serde::{Deserialize, Serialize};
use std::fmt;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Provider error: {0}")]
    Provider(String),

    #[error("Tool execution error: {0}")]
    ToolExecution(String),

    #[error("Task execution error: {0}")]
    TaskExecution(String),

    #[error("Memory error: {0}")]
    Memory(String),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("HTTP error: {0}")]
    Http(String),

    #[error("Rate limit exceeded: {provider}")]
    RateLimit { provider: String },

    #[error("Provider unavailable: {provider}")]
    ProviderUnavailable { provider: String },

    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    #[error("Retry exhausted: {0}")]
    RetryExhausted(String),

    #[error("Agent error: {0}")]
    Agent(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl From<rusqlite::Error> for Error {
    fn from(err: rusqlite::Error) -> Self {
        Error::Memory(err.to_string())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub call_id: String,
    pub result: ToolResultData,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ToolResultData {
    Success { output: serde_json::Value },
    Error { message: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TaskStatus {
    Queued,
    Running,
    Completed,
    Failed,
    Blocked,
}

impl fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TaskStatus::Queued => write!(f, "queued"),
            TaskStatus::Running => write!(f, "running"),
            TaskStatus::Completed => write!(f, "completed"),
            TaskStatus::Failed => write!(f, "failed"),
            TaskStatus::Blocked => write!(f, "blocked"),
        }
    }
}
