use crate::error::{ProviderError, ProviderResult};
use crate::models::{ChatMessage, ChatRequest, ChatResponse, ModelCapability, StreamChunk};
use async_trait::async_trait;
use futures::Stream;
use serde_json::Value;

#[derive(Debug, Clone)]
pub struct ProviderCapabilities {
    pub supports_chat: bool,
    pub supports_tools: bool,
    pub supports_streaming: bool,
    pub supports_images: bool,
    pub supports_function_calling: bool,
    pub max_context_length: Option<u32>,
    pub supported_capabilities: Vec<ModelCapability>,
}

#[async_trait]
pub trait ProviderAdapter: Send + Sync {
    fn name(&self) -> &str;

    fn capabilities(&self) -> &ProviderCapabilities;

    async fn chat(&self, request: &ChatRequest) -> ProviderResult<ChatResponse>;

    async fn chat_stream(
        &self,
        _request: &ChatRequest,
    ) -> ProviderResult<Box<dyn Stream<Item = ProviderResult<StreamChunk>> + Send + Unpin>> {
        Err(ProviderError::ToolCallingNotSupported(format!(
            "Streaming not supported by {}",
            self.name()
        )))
    }

    fn list_models(&self) -> Vec<String>;

    fn get_model_info(&self, model_id: &str) -> Option<crate::models::ModelInfo>;

    async fn validate_config(&self) -> ProviderResult<()>;

    fn is_configured(&self) -> bool;
}
