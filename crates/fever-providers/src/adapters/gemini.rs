use crate::adapter::{ProviderAdapter, ProviderCapabilities};
use crate::error::{ProviderError, ProviderResult};
use crate::models::{
    ChatMessage, ChatRequest, ChatResponse, ChatChoice, ModelCapability, ModelInfo, StreamChunk, ToolCall, Usage,
};
use async_trait::async_trait;
use futures::Stream;
use reqwest::Client;
use serde::Deserialize;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::debug;

// Gemini OpenAI-compatible response types
#[derive(Deserialize)]
struct OpenAiCompatResponse {
    id: Option<String>,
    choices: Vec<OpenAiCompatChoice>,
    usage: Option<OpenAiCompatUsage>,
}

#[derive(Deserialize)]
struct OpenAiCompatChoice {
    index: Option<u32>,
    message: OpenAiCompatMessage,
    finish_reason: Option<String>,
}

#[derive(Deserialize)]
struct OpenAiCompatMessage {
    role: Option<String>,
    content: Option<String>,
    tool_calls: Option<Vec<OpenAiCompatToolCall>>,
}

#[derive(Deserialize)]
struct OpenAiCompatToolCall {
    id: String,
    function: OpenAiCompatFunction,
}

#[derive(Deserialize)]
struct OpenAiCompatFunction {
    name: String,
    arguments: String,
}

#[derive(Deserialize)]
struct OpenAiCompatUsage {
    prompt_tokens: Option<u64>,
    completion_tokens: Option<u64>,
    total_tokens: Option<u64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GeminiConfig {
    pub api_key: String,
    pub base_url: String,
    #[serde(default)]
    pub default_model: Option<String>,
}

impl Default for GeminiConfig {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            base_url: "https://generativelanguage.googleapis.com/v1beta/openai".to_string(),
            default_model: Some("gemini-2.5-flash-preview-05-20".to_string()),
        }
    }
}

pub struct GeminiAdapter {
    provider_name: String,
    config: GeminiConfig,
    client: Client,
    cached_models: Arc<RwLock<Vec<ModelInfo>>>,
}

impl GeminiAdapter {
    pub fn gemini(api_key: String) -> Self {
        Self::custom("gemini", api_key.clone(), GeminiConfig {
            api_key,
            base_url: "https://generativelanguage.googleapis.com/v1beta/openai".to_string(),
            ..Default::default()
        })
    }

    pub fn custom(name: &str, api_key: String, base_config: GeminiConfig) -> Self {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(120))
            .build()
            .unwrap_or_default();
        Self {
            provider_name: name.to_string(),
            config: GeminiConfig {
                api_key: api_key,
                base_url: base_config.base_url,
                default_model: base_config.default_model,
            },
            client,
            cached_models: Arc::new(RwLock::new(Vec::new())),
        }
    }

    fn auth_header(&self) -> String {
        format!("Bearer {}", self.config.api_key)
    }

    fn build_request_body(&self, request: &ChatRequest) -> serde_json::Value {
        // Convert internal ChatRequest into the OpenAI-compatible body Gemini expects
        let messages: Vec<serde_json::Value> = request.messages.iter().map(|m| {
            let mut msg = serde_json::json!({
                "role": m.role,
                "content": m.content,
            });
            if let Some(tc) = &m.tool_calls {
                msg["tool_calls"] = serde_json::to_value(tc).unwrap();
            }
            if let Some(id) = &m.tool_call_id {
                msg["tool_call_id"] = serde_json::json!(id.as_str());
            }
            msg
        }).collect();

        let mut body = serde_json::json!({
            "model": request.model,
            "messages": messages,
        });

        if let Some(temp) = request.temperature {
            body["temperature"] = serde_json::json!(temp);
        }
        if let Some(tokens) = request.max_tokens {
            body["max_tokens"] = serde_json::json!(tokens);
        }
        if let Some(tools) = &request.tools {
            body["tools"] = serde_json::to_value(
                tools.iter().map(|t| {
                    serde_json::json!({
                        "type": "function",
                        "function": {
                            "name": t.name,
                            "description": t.description,
                            "parameters": t.parameters,
                        }
                    })
                }).collect::<Vec<_>>()
            ).unwrap();
        }

        body
    }

    pub async fn fetch_models(&self) -> ProviderResult<Vec<ModelInfo>> {
        let url = format!("{}/models", self.config.base_url.trim_end_matches('/'));
        debug!("Fetching models from {}", url);

        let resp = self.client
            .get(&url)
            .header("Authorization", &self.auth_header())
            .send()
            .await
            .map_err(|e| ProviderError::Http(e.to_string()))?;

        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(ProviderError::Api {
                code: status.as_u16().to_string(),
                message: body,
            });
        }

        #[derive(Deserialize)]
        struct ModelsResponse {
            data: Option<Vec<ModelEntry>>,
        }
        #[derive(Deserialize)]
        struct ModelEntry {
            id: String,
            context_length: Option<u64>,
        }

        let body: ModelsResponse = resp
            .json()
            .await
            .map_err(|e| ProviderError::Parse(e.to_string()))?;

        let provider_name = self.provider_name.clone();
        let models: Vec<ModelInfo> = body.data.unwrap_or_default().into_iter().map(|m| ModelInfo {
            id: format!("{}/{}", provider_name, m.id),
            name: m.id,
            provider: provider_name.clone(),
            capabilities: vec![ModelCapability::Chat, ModelCapability::Streaming],
            context_length: m.context_length,
            max_output_tokens: None,
        }).collect();

        *self.cached_models.write().await = models.clone();
        Ok(models)
    }
}

#[async_trait]
impl ProviderAdapter for GeminiAdapter {
    fn name(&self) -> &str {
        &self.provider_name
    }

    fn capabilities(&self) -> ProviderCapabilities {
        ProviderCapabilities {
            supports_chat: true,
            supports_tools: true,
            supports_streaming: true,
            supports_images: true,
            supports_function_calling: true,
            max_context_length: None,
            // Gemini models follow the same capability tags as OpenAI
            supported_capabilities: vec![
                ModelCapability::Chat,
                ModelCapability::Tools,
                ModelCapability::Streaming,
                ModelCapability::Vision,
                ModelCapability::FunctionCalling,
            ],
        }
    }

    async fn chat(&self, request: &ChatRequest) -> ProviderResult<ChatResponse> {
        let url = format!(
            "{}/chat/completions",
            self.config.base_url.trim_end_matches('/')
        );
        let body = self.build_request_body(request);

        let resp = self.client
            .post(&url)
            .header("Authorization", &self.auth_header())
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| ProviderError::Http(e.to_string()))?;

        let status = resp.status();
        if status.as_u16() == 401 {
            return Err(ProviderError::Auth("Invalid API key".to_string()));
        }
        if status.as_u16() == 429 {
            return Err(ProviderError::RateLimit { provider: self.provider_name.clone() });
        }
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(ProviderError::Api {
                code: status.as_u16().to_string(),
                message: body,
            });
        }

        let raw: OpenAiCompatResponse = resp
            .json()
            .await
            .map_err(|e| ProviderError::Parse(e.to_string()))?;

        let choices = raw.choices.into_iter().map(|c| {
            let tool_calls = c.message.tool_calls.map(|tc| {
                tc.into_iter().map(|t| ToolCall {
                    id: t.id,
                    name: t.function.name,
                    arguments: serde_json::from_str(&t.function.arguments).unwrap_or(serde_json::Value::Null),
                }).collect()
            });

            ChatChoice {
                index: c.index.unwrap_or(0),
                message: ChatMessage {
                    role: c.message.role.unwrap_or_else(|| "assistant".to_string()),
                    content: c.message.content.unwrap_or_default(),
                    tool_calls,
                    // no dedicated tool_call_id in Gemini-compatible responses currently
                    tool_call_id: None,
                },
                finish_reason: c.finish_reason.unwrap_or_else(|| "stop".to_string()),
            }
        }).collect();

        Ok(ChatResponse {
            id: raw.id.unwrap_or_default(),
            choices,
            usage: raw.usage.map(|u| Usage {
                prompt_tokens: u.prompt_tokens.unwrap_or(0),
                completion_tokens: u.completion_tokens.unwrap_or(0),
                total_tokens: u.total_tokens.unwrap_or(0),
            }),
        })
    }

    async fn chat_stream(&self, _request: &ChatRequest) -> ProviderResult<Box<dyn Stream<Item = ProviderResult<StreamChunk>> + Send + Unpin>> {
        Err(ProviderError::RequestFailed("Streaming not yet implemented".to_string()))
    }

    fn list_models(&self) -> Vec<String> {
        let provider = self.provider_name.clone();
        vec![
            format!("{}/gemini-2.5-pro-preview-06-05", provider),
            format!("{}/gemini-2.5-flash-preview-05-20", provider),
            format!("{}/gemini-2.0-flash", provider),
            format!("{}/gemini-2.0-flash-lite", provider),
        ]
    }

    fn get_model_info(&self, model_id: &str) -> Option<ModelInfo> {
        if let Ok(models) = self.cached_models.try_read() {
            for m in models.iter() {
                if m.id == model_id {
                    return Some(ModelInfo {
                        id: m.id.clone(),
                        name: m.name.clone(),
                        provider: m.provider.clone(),
                        capabilities: m.capabilities.clone(),
                        context_length: m.context_length,
                        max_output_tokens: m.max_output_tokens,
                    });
                }
            }
        }
        None
    }

    async fn validate_config(&self) -> ProviderResult<()> {
        if self.config.api_key.trim().is_empty() {
            return Err(ProviderError::Config("API key is empty".to_string()));
        }
        Ok(())
    }

    fn is_configured(&self) -> bool {
        !self.config.api_key.trim().is_empty()
    }
}
