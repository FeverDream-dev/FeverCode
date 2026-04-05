use crate::adapter::{ProviderAdapter, ProviderCapabilities};
use crate::error::{ProviderError, ProviderResult};
use crate::models::{
    ChatChoice, ChatMessage, ChatRequest, ChatResponse, ModelCapability, ModelInfo, StreamChunk,
    ToolCall, Usage,
};
use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;
use tracing::debug;

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

#[derive(Deserialize)]
struct AnthropicResponse {
    id: Option<String>,
    content: Vec<AnthropicContent>,
    #[allow(dead_code)]
    model: Option<String>,
    stop_reason: Option<String>,
    usage: Option<AnthropicUsage>,
}

#[derive(Deserialize, Debug)]
struct AnthropicContent {
    #[serde(rename = "type")]
    content_type: Option<String>,
    text: Option<String>,
    id: Option<String>,
    name: Option<String>,
    input: Option<serde_json::Value>,
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
        AnthropicAdapter {
            provider_name: name.to_string(),
            config: AnthropicConfig {
                api_key: api_key.to_string(),
                base_url: base_url.trim_end_matches('/').to_string(),
                default_model: None,
            },
            client: Client::new(),
        }
    }

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

    fn build_messages(&self, request: &ChatRequest) -> (Option<String>, Vec<serde_json::Value>) {
        let mut system_prompt: Option<String> = None;
        let mut messages: Vec<serde_json::Value> = Vec::new();

        for m in &request.messages {
            if m.role == "system" {
                if system_prompt.is_none() {
                    system_prompt = Some(m.content.clone());
                }
            } else if m.role == "tool" {
                let mut block = serde_json::json!({
                    "type": "tool_result",
                    "content": m.content,
                });
                if let Some(ref id) = m.tool_call_id {
                    block["tool_use_id"] = serde_json::json!(id);
                }
                messages.push(serde_json::json!({
                    "role": "user",
                    "content": vec![block],
                }));
            } else if let Some(ref tool_calls) = m.tool_calls {
                let mut blocks: Vec<serde_json::Value> = Vec::new();
                if !m.content.is_empty() {
                    blocks.push(serde_json::json!({
                        "type": "text",
                        "text": m.content,
                    }));
                }
                for tc in tool_calls {
                    blocks.push(serde_json::json!({
                        "type": "tool_use",
                        "id": tc.id,
                        "name": tc.name,
                        "input": tc.arguments,
                    }));
                }
                messages.push(serde_json::json!({
                    "role": m.role,
                    "content": blocks,
                }));
            } else {
                messages.push(serde_json::json!({
                    "role": m.role,
                    "content": m.content,
                }));
            }
        }

        (system_prompt, messages)
    }

    fn parse_tool_calls(content: &[AnthropicContent]) -> Option<Vec<ToolCall>> {
        let tool_calls: Vec<ToolCall> = content
            .iter()
            .filter(|c| c.content_type.as_deref() == Some("tool_use"))
            .filter_map(|c| {
                let id = c.id.clone()?;
                let name = c.name.clone()?;
                let arguments = c.input.clone().unwrap_or(serde_json::Value::Null);
                Some(ToolCall {
                    id,
                    name,
                    arguments,
                })
            })
            .collect();

        if tool_calls.is_empty() {
            None
        } else {
            Some(tool_calls)
        }
    }

    fn headers(&self) -> reqwest::header::HeaderMap {
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
        headers
    }

    fn handle_response_status(
        &self,
        status: reqwest::StatusCode,
        body: String,
    ) -> ProviderResult<()> {
        if status.as_u16() == 401 {
            return Err(ProviderError::Auth(
                "Unauthorized: invalid API key".to_string(),
            ));
        }
        if status.as_u16() == 429 {
            return Err(ProviderError::RateLimit {
                provider: self.name().to_string(),
            });
        }
        if status.as_u16() == 400 {
            return Err(ProviderError::InvalidRequest(
                "Bad request to Anthropic API".to_string(),
            ));
        }
        if !status.is_success() {
            return Err(ProviderError::Api {
                code: status.as_u16().to_string(),
                message: body,
            });
        }
        Ok(())
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
            supported_capabilities: vec![
                ModelCapability::Chat,
                ModelCapability::Tools,
                ModelCapability::Streaming,
                ModelCapability::FunctionCalling,
            ],
        }
    }

    async fn chat(&self, request: &ChatRequest) -> ProviderResult<ChatResponse> {
        let model = self.effective_model(&request.model);
        let (system_prompt, messages) = self.build_messages(request);

        let max_tokens = request.max_tokens.unwrap_or(4096);
        let temperature = request.temperature.unwrap_or(0.7);

        let mut body = serde_json::json!({
            "model": model,
            "max_tokens": max_tokens,
            "messages": messages,
            "temperature": temperature,
        });

        if let Some(sys) = system_prompt {
            body["system"] = serde_json::json!(sys);
        }

        if let Some(ref tools) = request.tools {
            let anthropic_tools: Vec<serde_json::Value> = tools
                .iter()
                .map(|t| {
                    serde_json::json!({
                        "name": t.name,
                        "description": t.description,
                        "input_schema": t.parameters,
                    })
                })
                .collect();
            body["tools"] = serde_json::json!(anthropic_tools);
        }

        let url = format!("{}/v1/messages", self.config.base_url.trim_end_matches('/'));

        let resp = self
            .client
            .post(&url)
            .headers(self.headers())
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
        self.handle_response_status(status, text.clone())?;

        let anthro: AnthropicResponse = serde_json::from_str(&text).map_err(|e| {
            ProviderError::Parse(format!("Failed to parse Anthropic response: {}", e))
        })?;

        let content_text: String = anthro
            .content
            .iter()
            .filter(|c| c.content_type.as_deref() == Some("text"))
            .filter_map(|c| c.text.as_ref())
            .cloned()
            .collect::<Vec<_>>()
            .join("");

        let tool_calls = Self::parse_tool_calls(&anthro.content);

        let finish_reason = match anthro.stop_reason.as_deref() {
            Some("tool_use") => "tool_calls".to_string(),
            Some(reason) => reason.to_string(),
            None => "stop".to_string(),
        };

        let choice = ChatChoice {
            index: 0,
            message: ChatMessage {
                role: "assistant".to_string(),
                content: content_text,
                tool_calls,
                tool_call_id: None,
            },
            finish_reason,
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
        request: &ChatRequest,
    ) -> ProviderResult<Box<dyn futures::Stream<Item = ProviderResult<StreamChunk>> + Send + Unpin>>
    {
        let model = self.effective_model(&request.model);
        let (system_prompt, messages) = self.build_messages(request);

        let max_tokens = request.max_tokens.unwrap_or(4096);
        let temperature = request.temperature.unwrap_or(0.7);

        let mut body = serde_json::json!({
            "model": model,
            "max_tokens": max_tokens,
            "messages": messages,
            "temperature": temperature,
            "stream": true,
        });

        if let Some(sys) = system_prompt {
            body["system"] = serde_json::json!(sys);
        }

        if let Some(ref tools) = request.tools {
            let anthropic_tools: Vec<serde_json::Value> = tools
                .iter()
                .map(|t| {
                    serde_json::json!({
                        "name": t.name,
                        "description": t.description,
                        "input_schema": t.parameters,
                    })
                })
                .collect();
            body["tools"] = serde_json::json!(anthropic_tools);
        }

        let url = format!("{}/v1/messages", self.config.base_url.trim_end_matches('/'));

        let resp = self
            .client
            .post(&url)
            .headers(self.headers())
            .json(&body)
            .send()
            .await
            .map_err(|e| ProviderError::Http(e.to_string()))?;

        let status = resp.status();
        if !status.is_success() {
            let text = resp.text().await.unwrap_or_default();
            return Err(self.handle_response_status(status, text).err().unwrap());
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

                    let mut event_type = String::new();
                    let mut data = String::new();

                    for line in event_text.lines() {
                        if let Some(et) = line.strip_prefix("event: ") {
                            event_type = et.to_string();
                        }
                        if let Some(d) = line.strip_prefix("data: ") {
                            data = d.to_string();
                        }
                    }

                    if event_type == "message_stop" {
                        done = true;
                        let _ = tx
                            .send(Ok(StreamChunk {
                                id: None,
                                delta: None,
                                content: None,
                                finish_reason: Some("stop".to_string()),
                                tool_calls: None,
                            }))
                            .await;
                        break;
                    }

                    if event_type == "content_block_delta" && !data.is_empty() {
                        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&data) {
                            let delta_type = json
                                .pointer("/delta/type")
                                .and_then(|v| v.as_str())
                                .unwrap_or("");
                            if delta_type == "text_delta" {
                                let content = json
                                    .pointer("/delta/text")
                                    .and_then(|v| v.as_str())
                                    .map(String::from);
                                if let Some(text) = content {
                                    let _ = tx
                                        .send(Ok(StreamChunk {
                                            id: None,
                                            delta: None,
                                            content: Some(text),
                                            finish_reason: None,
                                            tool_calls: None,
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
                capabilities: vec![
                    ModelCapability::Chat,
                    ModelCapability::Tools,
                    ModelCapability::Streaming,
                    ModelCapability::FunctionCalling,
                ],
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
