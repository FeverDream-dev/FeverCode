use crate::adapter::{ProviderAdapter, ProviderCapabilities};
use crate::error::{ProviderError, ProviderResult};
use crate::models::{
    ChatChoice, ChatMessage, ChatRequest, ChatResponse, ModelCapability, ModelInfo, StreamChunk,
    Usage,
};
use async_trait::async_trait;
use futures::Stream;
use reqwest::Client;
use serde::Deserialize;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::debug;

/// Configuration for the Ollama adapter.
#[derive(Debug, Clone, Deserialize)]
pub struct OllamaConfig {
    pub base_url: String,
    #[serde(default)]
    pub default_model: Option<String>,
}

impl Default for OllamaConfig {
    fn default() -> Self {
        Self {
            base_url: "http://localhost:11434".to_string(),
            default_model: Some("llama3.2".to_string()),
        }
    }
}

/// Adapter for Ollama local/remote instances using OpenAI-compatible API.
pub struct OllamaAdapter {
    provider_name: String,
    config: OllamaConfig,
    client: Client,
    cached_models: Arc<RwLock<Vec<ModelInfo>>>,
}

impl OllamaAdapter {
    /// Create an adapter for a local Ollama instance (localhost:11434).
    pub fn local() -> Self {
        Self::with_url("http://localhost:11434".to_string())
    }

    /// Create an adapter with a custom base URL.
    pub fn with_url(base_url: String) -> Self {
        let config = OllamaConfig {
            base_url,
            default_model: Some("llama3.2".to_string()),
        };
        Self::new(
            "ollama".to_string(),
            config,
            Client::new(),
            Arc::new(RwLock::new(Vec::new())),
        )
    }

    /// Create a named adapter with a custom base URL.
    pub fn custom(name: String, base_url: String) -> Self {
        let config = OllamaConfig {
            base_url,
            default_model: Some("llama3.2".to_string()),
        };
        Self::new(
            name,
            config,
            Client::new(),
            Arc::new(RwLock::new(Vec::new())),
        )
    }

    fn new(
        provider_name: String,
        config: OllamaConfig,
        client: Client,
        cached_models: Arc<RwLock<Vec<ModelInfo>>>,
    ) -> Self {
        Self {
            provider_name,
            config,
            client,
            cached_models,
        }
    }

    /// Fetch available models from Ollama and cache them.
    #[allow(dead_code)]
    async fn fetch_models(&self) -> Result<Vec<ModelInfo>, ProviderError> {
        let url = format!("{}/api/tags", self.config.base_url.trim_end_matches('/'));
        debug!("Fetching Ollama models from: {}", url);

        let response = self.client.get(&url).send().await.map_err(|e| {
            if e.is_connect() {
                ProviderError::Unavailable(format!("Failed to connect to Ollama: {}", e))
            } else {
                ProviderError::RequestFailed(format!("Request to Ollama failed: {}", e))
            }
        })?;

        if !response.status().is_success() {
            return Err(ProviderError::Http(format!(
                "Ollama returned status {}",
                response.status()
            )));
        }

        let tags: TagsResponse = response
            .json()
            .await
            .map_err(|e| ProviderError::Parse(format!("Failed to parse Ollama response: {}", e)))?;

        let models: Vec<ModelInfo> = tags
            .models
            .into_iter()
            .map(|m| {
                let id = format!("ollama/{}", m.name);
                ModelInfo {
                    id: id.clone(),
                    name: m.name,
                    provider: self.provider_name.clone(),
                    capabilities: vec![ModelCapability::Chat, ModelCapability::Tools],
                    context_length: None,
                    max_output_tokens: None,
                }
            })
            .collect();

        if let Ok(mut cache) = self.cached_models.try_write() {
            *cache = models.clone();
        }

        Ok(models)
    }
}

#[async_trait]
impl ProviderAdapter for OllamaAdapter {
    fn name(&self) -> &str {
        &self.provider_name
    }

    fn capabilities(&self) -> ProviderCapabilities {
        ProviderCapabilities {
            supports_chat: true,
            supports_tools: true,
            supports_streaming: true,
            supports_images: false,
            supports_function_calling: true,
            max_context_length: None,
            supported_capabilities: vec![
                ModelCapability::Chat,
                ModelCapability::Tools,
                ModelCapability::Streaming,
                ModelCapability::FunctionCalling,
            ],
        }
    }

    async fn chat(&self, request: &ChatRequest) -> ProviderResult<ChatResponse> {
        let url = format!(
            "{}/v1/chat/completions",
            self.config.base_url.trim_end_matches('/')
        );
        debug!("Sending chat request to Ollama: {}", url);

        // Build messages array
        let messages: Vec<serde_json::Value> = request
            .messages
            .iter()
            .map(|m| {
                let mut msg = serde_json::json!({
                    "role": m.role,
                    "content": m.content,
                });
                // Include tool_calls if present
                if let Some(ref tool_calls) = m.tool_calls {
                    msg["tool_calls"] =
                        serde_json::to_value(tool_calls).unwrap_or(serde_json::json!(null));
                }
                msg
            })
            .collect();

        // Build the request payload
        let mut payload = serde_json::json!({
            "model": request.model,
            "messages": messages,
        });

        if let Some(temp) = request.temperature {
            payload["temperature"] = serde_json::json!(temp);
        }

        if let Some(max_tokens) = request.max_tokens {
            payload["max_tokens"] = serde_json::json!(max_tokens);
        }

        // Add tools if provided
        if let Some(ref tools) = request.tools {
            let tools_json: Vec<serde_json::Value> = tools
                .iter()
                .map(|t| {
                    serde_json::json!({
                        "type": "function",
                        "function": {
                            "name": t.name,
                            "description": t.description,
                            "parameters": t.parameters,
                        }
                    })
                })
                .collect();
            payload["tools"] = serde_json::json!(tools_json);
        }

        // Send request (NO Authorization header - Ollama doesn't use auth)
        let response = self
            .client
            .post(&url)
            .json(&payload)
            .send()
            .await
            .map_err(|e| {
                if e.is_connect() {
                    ProviderError::Unavailable(format!("Failed to connect to Ollama: {}", e))
                } else if e.is_timeout() {
                    ProviderError::Timeout(format!("Ollama request timed out: {}", e))
                } else {
                    ProviderError::RequestFailed(format!("Ollama request failed: {}", e))
                }
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| "unknown error".to_string());
            return Err(ProviderError::Http(format!(
                "Ollama returned status {}: {}",
                status, body
            )));
        }

        // Parse the OpenAI-compatible response
        let ollama_response: OllamaChatResponse = response
            .json()
            .await
            .map_err(|e| ProviderError::Parse(format!("Failed to parse Ollama response: {}", e)))?;

        // Convert to our ChatResponse type
        let choices: Vec<ChatChoice> = ollama_response
            .choices
            .into_iter()
            .map(|c| {
                let index = c.index.unwrap_or(0);
                let content = c.message.content.unwrap_or_default();
                let role = c.message.role.unwrap_or_else(|| "assistant".to_string());
                let finish_reason = c.finish_reason.unwrap_or_else(|| "stop".to_string());

                // Convert tool_calls if present
                let tool_calls = c.message.tool_calls.map(|tc| {
                    tc.into_iter()
                        .map(|t| crate::models::ToolCall {
                            id: t.id.unwrap_or_default(),
                            name: t.function.name,
                            arguments: serde_json::from_str(&t.function.arguments)
                                .unwrap_or(serde_json::Value::Null),
                        })
                        .collect()
                });

                ChatChoice {
                    index,
                    message: ChatMessage {
                        role,
                        content,
                        tool_calls,
                        tool_call_id: None,
                    },
                    finish_reason,
                }
            })
            .collect();

        let usage = ollama_response.usage.map(|u| Usage {
            prompt_tokens: u.prompt_tokens.unwrap_or(0),
            completion_tokens: u.completion_tokens.unwrap_or(0),
            total_tokens: u.total_tokens.unwrap_or(0),
        });

        Ok(ChatResponse {
            id: ollama_response.id.unwrap_or_default(),
            choices,
            usage,
        })
    }

    async fn chat_stream(
        &self,
        _request: &ChatRequest,
    ) -> ProviderResult<Box<dyn Stream<Item = ProviderResult<StreamChunk>> + Send + Unpin>> {
        Err(ProviderError::InvalidRequest(
            "Streaming not yet implemented for Ollama".to_string(),
        ))
    }

    fn list_models(&self) -> Vec<String> {
        // Try cached models first
        if let Ok(cache) = self.cached_models.try_read() {
            if !cache.is_empty() {
                return cache.iter().map(|m| m.id.clone()).collect();
            }
        }

        // Fallback to default model from config
        if let Some(default) = &self.config.default_model {
            vec![format!("ollama/{}", default)]
        } else {
            Vec::new()
        }
    }

    fn get_model_info(&self, model_id: &str) -> Option<ModelInfo> {
        // Look up in cache
        if let Ok(cache) = self.cached_models.try_read() {
            for model in cache.iter() {
                if model.id == model_id {
                    return Some(model.clone());
                }
            }
        }

        // Fallback: synthesize a minimal entry
        Some(ModelInfo {
            id: model_id.to_string(),
            name: model_id.to_string(),
            provider: self.provider_name.clone(),
            capabilities: vec![ModelCapability::Chat],
            context_length: None,
            max_output_tokens: None,
        })
    }

    async fn validate_config(&self) -> ProviderResult<()> {
        if self.config.base_url.trim().is_empty() {
            return Err(ProviderError::Config(
                "base_url cannot be empty".to_string(),
            ));
        }
        Ok(())
    }

    fn is_configured(&self) -> bool {
        !self.config.base_url.trim().is_empty()
    }
}

// ============================================================================
// Internal types for Ollama API responses
// ============================================================================

/// Response from GET /api/tags
#[allow(dead_code)]
#[derive(Deserialize, Debug)]
struct TagsResponse {
    models: Vec<TagsModel>,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
struct TagsModel {
    name: String,
}

/// Response from POST /v1/chat/completions (OpenAI-compatible)
#[derive(Deserialize, Debug)]
struct OllamaChatResponse {
    id: Option<String>,
    choices: Vec<OllamaChoice>,
    usage: Option<OllamaUsage>,
}

#[derive(Deserialize, Debug)]
struct OllamaChoice {
    index: Option<u32>,
    message: OllamaMessage,
    finish_reason: Option<String>,
}

#[derive(Deserialize, Debug)]
struct OllamaMessage {
    role: Option<String>,
    content: Option<String>,
    tool_calls: Option<Vec<OllamaToolCall>>,
}

#[derive(Deserialize, Debug)]
struct OllamaToolCall {
    id: Option<String>,
    function: OllamaFunction,
}

#[derive(Deserialize, Debug)]
struct OllamaFunction {
    name: String,
    arguments: String,
}

#[derive(Deserialize, Debug)]
struct OllamaUsage {
    prompt_tokens: Option<u64>,
    completion_tokens: Option<u64>,
    total_tokens: Option<u64>,
}
