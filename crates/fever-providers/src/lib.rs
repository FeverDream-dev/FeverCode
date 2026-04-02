pub mod adapter;
pub mod adapters;
pub mod client;
pub mod error;
pub mod models;

pub use adapter::{ProviderAdapter, ProviderCapabilities};
pub use client::ProviderClient;
pub use error::{ProviderError, ProviderResult};
pub use models::{
    ChatMessage, ChatRequest, ChatResponse, ModelCapability, ModelInfo, ToolDefinition,
};
