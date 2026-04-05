use async_trait::async_trait;
use futures::Stream;
use std::pin::Pin;
use std::task::{Context, Poll};

use crate::adapter::{ProviderAdapter, ProviderCapabilities};
use crate::error::ProviderResult;
use crate::models::*;

pub struct MockProvider {
    /// If true, streaming returns chunks. If false, only chat() works.
    supports_streaming: bool,
    /// Optional scripted response override.
    response_override: Option<String>,
    /// Simulated latency in ms per chunk (streaming only).
    chunk_delay_ms: u64,
}

impl MockProvider {
    pub fn new() -> Self {
        Self {
            supports_streaming: true,
            response_override: None,
            chunk_delay_ms: 5,
        }
    }

    pub fn with_response(mut self, response: impl Into<String>) -> Self {
        self.response_override = Some(response.into());
        self
    }

    pub fn no_streaming(mut self) -> Self {
        self.supports_streaming = false;
        self
    }
}

impl Default for MockProvider {
    fn default() -> Self {
        Self::new()
    }
}

fn generate_response(request: &ChatRequest) -> String {
    let last_user_msg = request.messages.iter().rev().find(|m| m.role == "user");
    let input = last_user_msg.map(|m| m.content.as_str()).unwrap_or("");
    if input.contains("hello") || input.contains("hi") || input.contains("hey") {
        return "Hello! I'm Fever Code, demo mode.".to_string();
    }
    if input.contains("git status") || input.contains("git diff") {
        return "[Mock] git status: clean".to_string();
    }
    if input.contains("list files") || input.contains("ls") || input.contains("directory") {
        return "[Mock] files: Cargo.toml, README.md".to_string();
    }
    if input.contains("help") || input.contains("?") {
        return "Mock provider help".to_string();
    }
    format!(
        "I received your message (demo mode): {}",
        if input.len() > 100 {
            format!("{}...", &input[..100])
        } else {
            input.to_string()
        }
    )
}

#[async_trait]
impl ProviderAdapter for MockProvider {
    fn name(&self) -> &str {
        "mock"
    }

    fn capabilities(&self) -> ProviderCapabilities {
        ProviderCapabilities {
            supports_chat: true,
            supports_tools: true,
            supports_streaming: self.supports_streaming,
            supports_images: false,
            supports_function_calling: false,
            max_context_length: Some(128_000),
            supported_capabilities: vec![
                ModelCapability::Chat,
                ModelCapability::Tools,
                ModelCapability::Streaming,
            ],
        }
    }

    async fn chat(&self, request: &ChatRequest) -> ProviderResult<ChatResponse> {
        let content = self
            .response_override
            .clone()
            .unwrap_or_else(|| generate_response(request));

        Ok(ChatResponse {
            id: format!(
                "mock-{}",
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis()
            ),
            choices: vec![ChatChoice {
                index: 0,
                message: ChatMessage {
                    role: "assistant".to_string(),
                    content,
                    tool_calls: None,
                    tool_call_id: None,
                },
                finish_reason: "stop".to_string(),
            }],
            usage: Some(Usage {
                prompt_tokens: request
                    .messages
                    .iter()
                    .map(|m| m.content.len() as u64 / 4)
                    .sum(),
                completion_tokens: 0,
                total_tokens: 0,
            }),
        })
    }

    async fn chat_stream(
        &self,
        request: &ChatRequest,
    ) -> ProviderResult<Box<dyn Stream<Item = ProviderResult<StreamChunk>> + Send + Unpin>> {
        let content = self
            .response_override
            .clone()
            .unwrap_or_else(|| generate_response(request));

        let id = format!(
            "mock-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis()
        );

        Ok(Box::new(MockStream {
            content,
            id,
            delay_ms: self.chunk_delay_ms,
            position: 0,
            finished: false,
        }))
    }

    fn list_models(&self) -> Vec<String> {
        vec![
            "mock/default".to_string(),
            "mock/fast".to_string(),
            "mock/reasoning".to_string(),
        ]
    }

    fn get_model_info(&self, model_id: &str) -> Option<ModelInfo> {
        Some(ModelInfo {
            id: model_id.to_string(),
            name: model_id.to_string(),
            provider: "mock".to_string(),
            capabilities: vec![
                ModelCapability::Chat,
                ModelCapability::Tools,
                ModelCapability::Streaming,
            ],
            context_length: Some(128_000),
            max_output_tokens: Some(4096),
        })
    }

    async fn validate_config(&self) -> ProviderResult<()> {
        Ok(())
    }

    fn is_configured(&self) -> bool {
        true
    }
}

struct MockStream {
    content: String,
    id: String,
    #[allow(dead_code)]
    delay_ms: u64,
    position: usize,
    finished: bool,
}

impl Stream for MockStream {
    type Item = ProviderResult<StreamChunk>;

    fn poll_next(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();

        if this.finished {
            return Poll::Ready(None);
        }

        // Yield one word (or remaining chars) at a time
        let remaining = &this.content[this.position..];
        let next_break = remaining
            .find(' ')
            .map(|i| i + 1)
            .unwrap_or(remaining.len());
        let chunk_len = next_break.min(remaining.len());

        let chunk = remaining[..chunk_len].to_string();
        this.position += chunk_len;

        let is_finished = this.position >= this.content.len();
        let finish_reason = if is_finished {
            Some("stop".to_string())
        } else {
            None
        };

        if is_finished {
            this.finished = true;
        }

        // Return a chunk deterministically
        Poll::Ready(Some(Ok(StreamChunk {
            id: Some(this.id.clone()),
            delta: Some(chunk.clone()),
            content: Some(chunk),
            finish_reason,
            tool_calls: None,
        })))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_chat_basic() {
        let mock = MockProvider::new();
        let request = ChatRequest {
            model: "mock/default".to_string(),
            messages: vec![ChatMessage {
                role: "user".to_string(),
                content: "hello".to_string(),
                tool_calls: None,
                tool_call_id: None,
            }],
            ..Default::default()
        };

        let response = mock.chat(&request).await.unwrap();
        assert_eq!(response.choices.len(), 1);
        assert_eq!(response.choices[0].message.role, "assistant");
        assert!(response.choices[0].message.content.contains("demo mode"));
        assert_eq!(response.choices[0].finish_reason, "stop");
    }
}
