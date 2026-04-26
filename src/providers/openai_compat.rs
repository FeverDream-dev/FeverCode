use anyhow::Result;
use async_stream::stream;
use futures::Stream;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::pin::Pin;

use super::{ChatRequest, Provider, ProviderEvent, ProviderUsage};

pub struct OpenAiCompatProvider {
    name: String,
    base_url: String,
    api_key: Option<String>,
    model: String,
    client: Client,
}

#[derive(Serialize)]
struct ChatCompletionRequest {
    model: String,
    messages: Vec<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    stream: bool,
}

#[derive(Deserialize)]
struct StreamChunk {
    choices: Vec<StreamChoice>,
    usage: Option<UsageResponse>,
}

#[derive(Deserialize)]
struct StreamChoice {
    delta: DeltaResponse,
    finish_reason: Option<String>,
}

#[derive(Deserialize)]
struct DeltaResponse {
    content: Option<String>,
}

#[derive(Deserialize, Debug)]
struct UsageResponse {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

#[derive(Deserialize, Debug)]
struct ChatCompletionResponse {
    choices: Vec<NonStreamChoice>,
    usage: Option<UsageResponse>,
}

#[derive(Deserialize, Debug)]
struct NonStreamChoice {
    message: Option<NonStreamMessage>,
}

#[derive(Deserialize, Debug)]
struct NonStreamMessage {
    content: Option<String>,
}

impl OpenAiCompatProvider {
    pub fn new(name: String, base_url: String, api_key: Option<String>, model: String) -> Self {
        Self {
            name,
            base_url,
            api_key,
            model,
            client: Client::new(),
        }
    }

    fn build_request_body(&self, chat: &ChatRequest, is_stream: bool) -> ChatCompletionRequest {
        let messages: Vec<serde_json::Value> = chat
            .messages
            .iter()
            .map(|m| serde_json::to_value(m).unwrap_or_default())
            .collect();

        let tools: Option<Vec<serde_json::Value>> = chat.tools.as_ref().map(|t| {
            t.iter()
                .map(|tool| serde_json::to_value(tool).unwrap_or_default())
                .collect()
        });

        ChatCompletionRequest {
            model: chat.model.clone().unwrap_or_else(|| self.model.clone()),
            messages,
            tools,
            temperature: chat.temperature,
            max_tokens: chat.max_tokens,
            stream: is_stream,
        }
    }

    fn url(&self) -> String {
        let base = self.base_url.trim_end_matches('/');
        format!("{}/chat/completions", base)
    }
}

impl Provider for OpenAiCompatProvider {
    fn name(&self) -> &str {
        &self.name
    }

    fn kind(&self) -> &str {
        "openai_compatible"
    }

    fn chat_stream(
        &self,
        request: ChatRequest,
    ) -> Pin<Box<dyn Stream<Item = Result<ProviderEvent>> + Send>> {
        let url = self.url();
        let body = self.build_request_body(&request, true);
        let client = self.client.clone();
        let api_key = self.api_key.clone();

        let s = stream! {
            let mut req = client.post(&url);
            if let Some(key) = &api_key {
                req = req.bearer_auth(key);
            }
            let resp = req.json(&body).send().await;

            match resp {
                Ok(response) => {
                    if !response.status().is_success() {
                        let status = response.status();
                        let text = response.text().await.unwrap_or_default();
                        yield Err(anyhow::anyhow!("provider error {}: {}", status, text));
                        return;
                    }

                    let byte_stream = response.bytes_stream();
                    let mut buffer = String::new();
                    let mut usage = ProviderUsage::default();

                    use futures::StreamExt;
                    let mut byte_stream = Box::pin(byte_stream);

                    while let Some(chunk) = byte_stream.next().await {
                        match chunk {
                            Ok(bytes) => {
                                buffer.push_str(&String::from_utf8_lossy(&bytes));
                                let mut lines_to_process = Vec::new();
                                let remaining;
                                {
                                    let parts: Vec<&str> = buffer.split('\n').collect();
                                    if parts.len() > 1 {
                                        for part in &parts[..parts.len() - 1] {
                                            lines_to_process.push(part.to_string());
                                        }
                                        remaining = parts.last().unwrap_or(&"").to_string();
                                    } else {
                                        remaining = buffer.clone();
                                    }
                                }
                                buffer = remaining;

                                for line in lines_to_process {
                                    let line = line.trim();
                                    if line.is_empty() || line == "data: [DONE]" {
                                        continue;
                                    }
                                    if let Some(data) = line.strip_prefix("data: ") {
                                        if let Ok(chunk) = serde_json::from_str::<StreamChunk>(data) {
                                            for choice in chunk.choices {
                                                if let Some(content) = choice.delta.content {
                                                    if !content.is_empty() {
                                                        yield Ok(ProviderEvent::Delta(content));
                                                    }
                                                }
                                            }
                                            if let Some(u) = chunk.usage {
                                                usage = ProviderUsage {
                                                    prompt_tokens: u.prompt_tokens,
                                                    completion_tokens: u.completion_tokens,
                                                    total_tokens: u.total_tokens,
                                                };
                                            }
                                        }
                                    }
                                }
                            }
                            Err(e) => {
                                yield Err(anyhow::anyhow!("stream error: {}", e));
                                return;
                            }
                        }
                    }

                    yield Ok(ProviderEvent::Done(usage));
                }
                Err(e) => {
                    yield Err(anyhow::anyhow!("request failed: {}", e));
                }
            }
        };

        Box::pin(s)
    }
}

pub async fn chat_once(provider: &OpenAiCompatProvider, request: ChatRequest) -> Result<String> {
    let url = provider.url();
    let body = provider.build_request_body(&request, false);
    let mut req = provider.client.post(&url);
    if let Some(key) = &provider.api_key {
        req = req.bearer_auth(key);
    }
    let resp = req.json(&body).send().await?;
    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        anyhow::bail!("provider error {}: {}", status, text);
    }
    let data: ChatCompletionResponse = resp.json().await?;
    let content = data
        .choices
        .first()
        .and_then(|c| c.message.as_ref())
        .and_then(|m| m.content.clone())
        .unwrap_or_default();
    Ok(content)
}
