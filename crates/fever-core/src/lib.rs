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
mod instructions;
mod memory;
mod permission;
mod retry;
mod task;
mod telemetry;
mod tool;
mod understand;

pub use agent::{Agent, AgentContext, AgentResponse, Message};
pub use error::{Error, Result, TaskStatus, ToolCall, ToolResult, ToolResultData};
pub use event::{Event, EventBus};
pub use execution::{ExecutionContext, ExecutionEngine, ExecutionEvent};
pub use instructions::discover_instructions;
pub use memory::{MemoryStore, StoredMessage};
pub use permission::{
    CommandRisk, PermissionGuard, PermissionMode, PermissionScope, PermissionVerdict,
    apply_mode, classify_command_risk, guard_for_mode, normalize_and_validate_path, redact_secrets,
};
pub use retry::{BackoffType, RetryPolicy, retry_with_policy};
pub use task::{Plan, Task, Todo};
pub use telemetry::{JsonlSink, MemorySink, Telemetry, TelemetryEvent, TelemetryEventType, TelemetrySink};
pub use tool::{Tool, ToolRegistry, ToolSchema, generate_call_id};
pub use understand::{BuildSystem, LanguageInfo, ProjectSummary, ProjectUnderstanding};
