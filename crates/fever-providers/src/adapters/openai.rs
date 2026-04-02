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
use serde_json::{Value, json};
use std::sync::Arc;
use tokio::sync::RwLock;

// Internal OpenAI-compatible response types
#[derive(Deserialize)]
struct OpenAiResponse {
    id: Option<String>,
    choices: Vec<OpenAiChoice>,
    usage: Option<OpenAiUsage>,
}

#[derive(Deserialize)]
struct OpenAiChoice {
    index: Option<u32>,
    message: OpenAiMessage,
    finish_reason: Option<String>,
}

#[derive(Deserialize)]
struct OpenAiMessage {
    role: Option<String>,
    content: Option<String>,
    tool_calls: Option<Vec<OpenAiToolCall>>,
}

#[derive(Deserialize)]
struct OpenAiToolCall {
    id: String,
    function: OpenAiFunction,
}

#[derive(Deserialize)]
struct OpenAiFunction {
    name: String,
    arguments: String,
}

#[derive(Deserialize)]
struct OpenAiUsage {
    prompt_tokens: Option<u64>,
    completion_tokens: Option<u64>,
    total_tokens: Option<u64>,
}

// Public config struct
#[derive(Debug, Clone, Deserialize)]
pub struct OpenAiConfig {
    pub api_key: String,
    pub base_url: String,
    #[serde(default)]
    pub default_model: Option<String>,
    #[serde(default)]
    pub organization: Option<String>,
}

impl Default for OpenAiConfig {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            base_url: "https://api.openai.com/v1".to_string(),
            default_model: None,
            organization: None,
        }
    }
}

// Public adapter struct
pub struct OpenAiAdapter {
    provider_name: String,
    config: OpenAiConfig,
    client: Client,
    cached_models: Arc<RwLock<Vec<ModelInfo>>>,
}

impl OpenAiAdapter {
    fn new(provider_name: String, config: OpenAiConfig) -> Self {
        Self {
            provider_name,
            config,
            client: Client::new(),
            cached_models: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub fn openai(api_key: impl Into<String>) -> Self {
        Self::new(
            "openai".to_string(),
            OpenAiConfig {
                api_key: api_key.into(),
                base_url: "https://api.openai.com/v1".to_string(),
                default_model: None,
                organization: None,
            },
        )
    }

    pub fn openrouter(api_key: impl Into<String>) -> Self {
        Self::new(
            "openrouter".to_string(),
            OpenAiConfig {
                api_key: api_key.into(),
                base_url: "https://openrouter.ai/api/v1".to_string(),
                default_model: None,
                organization: None,
            },
        )
    }

    pub fn together(api_key: impl Into<String>) -> Self {
        Self::new(
            "together".to_string(),
            OpenAiConfig {
                api_key: api_key.into(),
                base_url: "https://api.together.xyz/v1".to_string(),
                default_model: None,
                organization: None,
            },
        )
    }

    pub fn groq(api_key: impl Into<String>) -> Self {
        Self::new(
            "groq".to_string(),
            OpenAiConfig {
                api_key: api_key.into(),
                base_url: "https://api.groq.com/openai/v1".to_string(),
                default_model: None,
                organization: None,
            },
        )
    }

    pub fn fireworks(api_key: impl Into<String>) -> Self {
        Self::new(
            "fireworks".to_string(),
            OpenAiConfig {
                api_key: api_key.into(),
                base_url: "https://api.fireworks.ai/inference/v1".to_string(),
                default_model: None,
                organization: None,
            },
        )
    }

    pub fn mistral(api_key: impl Into<String>) -> Self {
        Self::new(
            "mistral".to_string(),
            OpenAiConfig {
                api_key: api_key.into(),
                base_url: "https://api.mistral.ai/v1".to_string(),
                default_model: None,
                organization: None,
            },
        )
    }

    pub fn deepseek(api_key: impl Into<String>) -> Self {
        Self::new(
            "deepseek".to_string(),
            OpenAiConfig {
                api_key: api_key.into(),
                base_url: "https://api.deepseek.com/v1".to_string(),
                default_model: None,
                organization: None,
            },
        )
    }

    pub fn minimax(api_key: impl Into<String>) -> Self {
        Self::new(
            "minimax".to_string(),
            OpenAiConfig {
                api_key: api_key.into(),
                base_url: "https://api.minimax.chat/v1".to_string(),
                default_model: None,
                organization: None,
            },
        )
    }

    pub fn perplexity(api_key: impl Into<String>) -> Self {
        Self::new(
            "perplexity".to_string(),
            OpenAiConfig {
                api_key: api_key.into(),
                base_url: "https://api.perplexity.ai".to_string(),
                default_model: None,
                organization: None,
            },
        )
    }

    pub fn custom(name: String, api_key: impl Into<String>, base_url: impl Into<String>) -> Self {
        Self::new(
            name,
            OpenAiConfig {
                api_key: api_key.into(),
                base_url: base_url.into(),
                default_model: None,
                organization: None,
            },
        )
    }

    pub async fn fetch_models(&self) -> ProviderResult<()> {
        let url = format!("{}/models", self.config.base_url);
        let resp = self
            .client
            .get(&url)
            .bearer_auth(&self.config.api_key)
            .send()
            .await
            .map_err(|e| ProviderError::Http(e.to_string()))?;

        let status = resp.status();
        if status == reqwest::StatusCode::UNAUTHORIZED {
            return Err(ProviderError::Auth("Invalid API key".to_string()));
        }
        if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
            return Err(ProviderError::RateLimit {
                provider: self.provider_name.clone(),
            });
        }
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(ProviderError::Api {
                code: status.as_u16().to_string(),
                message: body,
            });
        }

        let data: Value = resp
            .json()
            .await
            .map_err(|e| ProviderError::Parse(e.to_string()))?;
        let items = data.get("data").and_then(|d| d.as_array());

        let mut models: Vec<ModelInfo> = Vec::new();
        if let Some(items) = items {
            for m in items {
                if let Some(id) = m.get("id").and_then(|v| v.as_str()) {
                    let context_length = m.get("context_length").and_then(|v| v.as_u64());
                    let model_id = format!("{}/{}", self.provider_name, id);
                    models.push(ModelInfo {
                        id: model_id,
                        name: id.to_string(),
                        provider: self.provider_name.clone(),
                        capabilities: vec![ModelCapability::Chat, ModelCapability::Streaming],
                        context_length,
                        max_output_tokens: None,
                    });
                }
            }
        }
        let mut w = self.cached_models.write().await;
        *w = models;
        Ok(())
    }

    fn build_request_body(request: &ChatRequest) -> Value {
        let mut body = json!({
            "model": request.model,
            "messages": request.messages.iter().map(|m| {
                json!({
                    "role": m.role,
                    "content": m.content,
                })
            }).collect::<Vec<Value>>(),
        });

        if let Some(temp) = request.temperature {
            body["temperature"] = json!(temp);
        }
        if let Some(max) = request.max_tokens {
            body["max_tokens"] = json!(max);
        }
        if let Some(tools) = &request.tools {
            let tools_json: Vec<Value> = tools
                .iter()
                .map(|t| {
                    json!({
                        "type": "function",
                        "function": {
                            "name": t.name,
                            "description": t.description,
                            "parameters": t.parameters,
                        }
                    })
                })
                .collect();
            body["tools"] = json!(tools_json);
        }
        body
    }
}

#[async_trait]
impl ProviderAdapter for OpenAiAdapter {
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
        let url = format!("{}/chat/completions", self.config.base_url);
        let body = Self::build_request_body(request);

        let resp = self
            .client
            .post(&url)
            .bearer_auth(&self.config.api_key)
            .json(&body)
            .send()
            .await
            .map_err(|e| ProviderError::Http(e.to_string()))?;

        let status = resp.status();
        if status == reqwest::StatusCode::UNAUTHORIZED {
            return Err(ProviderError::Auth("Invalid API key".to_string()));
        }
        if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
            return Err(ProviderError::RateLimit {
                provider: self.provider_name.clone(),
            });
        }
        if !status.is_success() {
            let text = resp.text().await.unwrap_or_default();
            return Err(ProviderError::Api {
                code: status.as_u16().to_string(),
                message: text,
            });
        }

        let parsed: OpenAiResponse = resp
            .json()
            .await
            .map_err(|e| ProviderError::Parse(e.to_string()))?;

        let id = parsed.id.unwrap_or_else(|| "chatcmpl-default".to_string());

        let choices: Vec<ChatChoice> = parsed
            .choices
            .into_iter()
            .map(|c| {
                let tool_calls = c.message.tool_calls.map(|tc| {
                    tc.into_iter()
                        .map(|t| crate::models::ToolCall {
                            id: t.id,
                            name: t.function.name,
                            arguments: serde_json::from_str(&t.function.arguments)
                                .unwrap_or(serde_json::Value::Null),
                        })
                        .collect()
                });

                ChatChoice {
                    index: c.index.unwrap_or(0),
                    message: ChatMessage {
                        role: c.message.role.unwrap_or_else(|| "assistant".to_string()),
                        content: c.message.content.unwrap_or_default(),
                        tool_calls,
                        tool_call_id: None,
                    },
                    finish_reason: c.finish_reason.unwrap_or_else(|| "stop".to_string()),
                }
            })
            .collect();

        let usage = parsed.usage.map(|u| Usage {
            prompt_tokens: u.prompt_tokens.unwrap_or(0),
            completion_tokens: u.completion_tokens.unwrap_or(0),
            total_tokens: u.total_tokens.unwrap_or(0),
        });

        Ok(ChatResponse { id, choices, usage })
    }

    async fn chat_stream(
        &self,
        request: &ChatRequest,
    ) -> ProviderResult<Box<dyn Stream<Item = ProviderResult<StreamChunk>> + Send + Unpin>> {
        let url = format!("{}/chat/completions", self.config.base_url);
        let mut body = Self::build_request_body(request);
        body["stream"] = json!(true);

        let resp = self
            .client
            .post(&url)
            .bearer_auth(&self.config.api_key)
            .json(&body)
            .send()
            .await
            .map_err(|e| ProviderError::Http(e.to_string()))?;

        let status = resp.status();
        if status == reqwest::StatusCode::UNAUTHORIZED {
            return Err(ProviderError::Auth("Invalid API key".to_string()));
        }
        if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
            return Err(ProviderError::RateLimit {
                provider: self.provider_name.clone(),
            });
        }
        if !status.is_success() {
            let text = resp.text().await.unwrap_or_default();
            return Err(ProviderError::Api {
                code: status.as_u16().to_string(),
                message: text,
            });
        }

        let (tx, rx) = tokio::sync::mpsc::channel::<ProviderResult<StreamChunk>>(32);

        tokio::spawn(async move {
            let mut buffer = String::new();
            let mut done = false;
            let mut resp = resp;

            while !done {
                while let Some(pos) = buffer.find("\n\n") {
                    let event_text = buffer[..pos].to_string();
                    buffer = buffer[pos + 2..].to_string();

                    for line in event_text.lines() {
                        if let Some(data) = line.strip_prefix("data: ") {
                            if data.trim() == "[DONE]" {
                                done = true;
                                let _ = tx
                                    .send(Ok(StreamChunk {
                                        id: None,
                                        delta: None,
                                        content: None,
                                        finish_reason: Some("stop".to_string()),
                                    }))
                                    .await;
                                break;
                            }

                            if let Ok(json) = serde_json::from_str::<serde_json::Value>(data) {
                                let id = json.get("id").and_then(|v| v.as_str()).map(String::from);
                                let content = json
                                    .pointer("/choices/0/delta/content")
                                    .and_then(|v| v.as_str())
                                    .map(String::from);
                                let finish_reason = json
                                    .pointer("/choices/0/finish_reason")
                                    .and_then(|v| v.as_str())
                                    .map(String::from);

                                if content.is_some() || finish_reason.is_some() {
                                    let _ = tx
                                        .send(Ok(StreamChunk {
                                            id,
                                            delta: None,
                                            content,
                                            finish_reason,
                                        }))
                                        .await;
                                }
                            }
                        }
                    }
                }

                if done {
                    break;
                }

                match resp.chunk().await {
                    Ok(Some(bytes)) => {
                        buffer.push_str(&String::from_utf8_lossy(&bytes));
                    }
                    Ok(None) => break,
                    Err(e) => {
                        let _ = tx.send(Err(ProviderError::Http(e.to_string()))).await;
                        break;
                    }
                }
            }
        });

        let stream = tokio_stream::wrappers::ReceiverStream::new(rx);
        Ok(Box::new(stream))
    }

    fn list_models(&self) -> Vec<String> {
        if let Ok(models) = self.cached_models.try_read() {
            if !models.is_empty() {
                return models.iter().map(|m| m.id.clone()).collect();
            }
        }
        vec![format!(
            "{}/{}",
            self.provider_name,
            self.config.default_model.as_deref().unwrap_or("default")
        )]
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
            return Err(ProviderError::Config("API key is required".to_string()));
        }
        Ok(())
    }

    fn is_configured(&self) -> bool {
        !self.config.api_key.trim().is_empty()
    }
}
