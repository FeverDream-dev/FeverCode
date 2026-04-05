pub mod adapter;
pub mod adapters;
pub use adapters::mock::MockProvider;
pub mod client;
pub mod error;
pub mod models;
pub mod registry;

pub use adapter::{ProviderAdapter, ProviderCapabilities};
pub use client::ProviderClient;
pub use error::{ProviderError, ProviderResult};
pub use models::{
    ChatMessage, ChatRequest, ChatResponse, ModelCapability, ModelInfo, ToolDefinition,
};
pub use registry::{ProviderProfile, ProviderRegistry, AdapterType, ProviderTier};
