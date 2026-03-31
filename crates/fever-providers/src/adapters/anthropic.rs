use crate::adapter::{ProviderAdapter, ProviderCapabilities};
use crate::error::{ProviderError, ProviderResult};
use crate::models::{ChatRequest, ChatResponse, ChatChoice, ChatMessage, ModelCapability, ModelInfo, StreamChunk, Usage};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::debug;

// Anthropic-specific configuration
#[derive(Debug, Clone, Deserialize)]
pub struct AnthropicConfig {
    pub api_key: String,
    pub base_url: String,
    #[serde(default)]
    pub default_model: Option<String>,
}

impl Default for AnthropicConfig {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            base_url: "https://api.anthropic.com".to_string(),
            default_model: Some("claude-sonnet-4-20250514".to_string()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct AnthropicAdapter {
    provider_name: String,
    config: AnthropicConfig,
    client: Client,
}

// Internal request/response shapes for Anthropic's Messages API
#[derive(Serialize)]
struct AnthropicRequestBody {
    model: String,
    max_tokens: u32,
    messages: Vec<AnthropicMessage>,
    system: Option<String>,
    temperature: f32,
}

#[derive(Serialize)]
struct AnthropicMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct AnthropicResponse {
    id: Option<String>,
    content: Vec<AnthropicContent>,
    #[allow(dead_code)]
    model: Option<String>,
    stop_reason: Option<String>,
    usage: Option<AnthropicUsage>,
}
#[derive(Deserialize)]
struct AnthropicContent {
    text: Option<String>,
    #[serde(rename = "type")]
    #[allow(dead_code)]
    content_type: Option<String>,
}
#[derive(Deserialize)]
struct AnthropicUsage {
    input_tokens: Option<u64>,
    output_tokens: Option<u64>,
}

impl AnthropicAdapter {
    pub fn claude(api_key: &str) -> Self {
        Self::custom("Anthropic Claude", api_key, "https://api.anthropic.com")
    }

    pub fn custom(name: &str, api_key: &str, base_url: &str) -> Self {
        let cfg = AnthropicConfig {
            api_key: api_key.to_string(),
            base_url: base_url.trim_end_matches('/').to_string(),
            default_model: None,
        };
        AnthropicAdapter {
            provider_name: name.to_string(),
            config: cfg,
            client: Client::new(),
        }
    }

    // Choose effective model: request model if non-empty, otherwise default_model or hard-coded default
    fn effective_model(&self, request_model: &str) -> String {
        if !request_model.is_empty() {
            request_model.to_string()
        } else {
            self.config
                .default_model
                .clone()
                .unwrap_or_else(|| "claude-sonnet-4-20250514".to_string())
        }
    }
}

#[async_trait]
impl ProviderAdapter for AnthropicAdapter {
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
            supported_capabilities: vec![ModelCapability::Chat, ModelCapability::Tools, ModelCapability::Streaming, ModelCapability::FunctionCalling],
        }
    }

    async fn chat(&self, request: &ChatRequest) -> ProviderResult<ChatResponse> {
        // Build Anthropic request payload
        let model = self.effective_model(&request.model);
        // Extract system message (first system) and non-system messages for Anthropic API
        let mut system_prompt: Option<String> = None;
        let mut messages: Vec<AnthropicMessage> = Vec::new();
        for m in &request.messages {
            if m.role == "system" {
                if system_prompt.is_none() {
                    system_prompt = Some(m.content.clone());
                }
            } else {
                messages.push(AnthropicMessage {
                    role: m.role.clone(),
                    content: m.content.clone(),
                });
            }
        }

        let max_tokens = request
            .max_tokens
            .unwrap_or(4096);
        let temperature = request.temperature.unwrap_or(0.7);

        let body = AnthropicRequestBody {
            model: model,
            max_tokens,
            messages,
            system: system_prompt,
            temperature,
        };

        // Endpoint and headers
        let url = format!("{}/v1/messages", self.config.base_url.trim_end_matches('/'));
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            "x-api-key",
            reqwest::header::HeaderValue::from_str(&self.config.api_key).unwrap(),
        );
        headers.insert(
            "anthropic-version",
            reqwest::header::HeaderValue::from_static("2023-06-01"),
        );
        headers.insert(
            "Content-Type",
            reqwest::header::HeaderValue::from_static("application/json"),
        );

        // Send request
        let resp = self
            .client
            .post(&url)
            .headers(headers)
            .json(&body)
            .send()
            .await
            .map_err(|e| {
                debug!(%e, "Anthropic request failed");
                ProviderError::Api {
                    code: "http_request_error".to_string(),
                    message: e.to_string(),
                }
            })?;

        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        if !status.is_success() {
            if status.as_u16() == 401 {
                return Err(ProviderError::Auth("Unauthorized: invalid API key".to_string()));
            } else if status.as_u16() == 429 {
                return Err(ProviderError::RateLimit {
                    provider: self.name().to_string(),
                });
            } else if status.as_u16() == 400 {
                return Err(ProviderError::InvalidRequest("Bad request to Anthropic API".to_string()));
            } else {
                return Err(ProviderError::Api {
                    code: status.as_u16().to_string(),
                    message: text,
                });
            }
        }

        let anthro: AnthropicResponse = serde_json::from_str(&text)
            .map_err(|e| ProviderError::Parse(format!("Failed to parse Anthropic response: {}", e)))?;

        // Map response to Fever's ChatResponse
        let mut content_text = String::new();
        if let Some(first) = anthro.content.get(0) {
            if let Some(t) = &first.text {
                content_text = t.clone();
            }
        }

        let _system_fields = anthro.system(); // unused - kept for future use
        let choice = ChatChoice {
            index: 0,
            message: ChatMessage {
                role: "assistant".to_string(),
                content: content_text,
                tool_calls: None,
                tool_call_id: None,
            },
            finish_reason: anthro.stop_reason.unwrap_or_default(),
        };

        let usage = anthro.usage.map(|u| Usage {
            prompt_tokens: u.input_tokens.unwrap_or(0),
            completion_tokens: u.output_tokens.unwrap_or(0),
            total_tokens: u.input_tokens.unwrap_or(0) + u.output_tokens.unwrap_or(0),
        });

        Ok(ChatResponse {
            id: anthro.id.unwrap_or_else(|| "anthropic-msg".to_string()),
            choices: vec![choice],
            usage,
        })
    }

    async fn chat_stream(
        &self,
        _request: &ChatRequest,
    ) -> ProviderResult<Box<dyn futures::Stream<Item = ProviderResult<StreamChunk>> + Send + Unpin>> {
        // Streaming not implemented yet
        Err(ProviderError::InvalidRequest("Streaming not yet implemented".to_string()))
    }

    fn list_models(&self) -> Vec<String> {
        vec![
            "claude-sonnet-4-20250514".to_string(),
            "claude-opus-4-20250514".to_string(),
            "claude-3-5-sonnet-20241022".to_string(),
            "claude-3-5-haiku-20241022".to_string(),
        ]
    }

    fn get_model_info(&self, model_id: &str) -> Option<ModelInfo> {
        let models = self.list_models();
        if models.contains(&model_id.to_string()) {
            Some(ModelInfo {
                id: model_id.to_string(),
                name: model_id.to_string(),
                provider: self.name().to_string(),
                capabilities: vec![ModelCapability::Chat, ModelCapability::Tools, ModelCapability::Streaming, ModelCapability::FunctionCalling],
                context_length: None,
                max_output_tokens: None,
            })
        } else {
            None
        }
    }

    async fn validate_config(&self) -> ProviderResult<()> {
        if self.config.api_key.trim().is_empty() {
            Err(ProviderError::Config("api_key is empty".to_string()))
        } else {
            Ok(())
        }
    }

    fn is_configured(&self) -> bool {
        !self.config.api_key.trim().is_empty()
    }
}

// Helpers: expose Anthropic response subset for mapping or avoid unused if not needed
impl AnthropicResponse {
    // For potential future enhancements; currently not used directly.
    fn system(&self) -> Option<String> {
        // Anthropic response does not include a separate system field; kept for API symmetry if needed
        None
    }
}
