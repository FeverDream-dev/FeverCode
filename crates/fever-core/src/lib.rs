//! Fever Code - The foundational orchestration engine for Fever Code
//!
//! This crate provides:
//! - Task graph and execution pipeline
//! - Event bus and messaging
//! - Memory/context management
//! - Retry and failure handling
//! - Run journal and persistence
//! - Tool router and dispatch

mod agent;
mod error;
mod event;
mod execution;
mod memory;
mod permission;
mod retry;
mod task;
mod tool;

pub use agent::{Agent, AgentContext, AgentResponse, Message};
pub use error::{Error, Result, TaskStatus, ToolCall, ToolResult, ToolResultData};
pub use event::{Event, EventBus};
pub use execution::{ExecutionContext, ExecutionEngine, ExecutionEvent};
pub use memory::{MemoryStore, StoredMessage};
pub use permission::{
    classify_command_risk, redact_secrets, normalize_and_validate_path,
    CommandRisk, PermissionGuard, PermissionScope, PermissionVerdict,
};
pub use retry::{retry_with_policy, BackoffType, RetryPolicy};
pub use task::{Plan, Task, Todo};
pub use tool::{generate_call_id, Tool, ToolRegistry, ToolSchema};
