use anyhow::Result;
use async_stream::stream;
use futures::Stream;
use std::pin::Pin;

use super::{ChatRequest, Provider, ProviderEvent, ProviderUsage};

pub struct ExternalCliProvider {
    name: String,
    command: String,
}

impl ExternalCliProvider {
    pub fn new(name: String, command: String) -> Self {
        Self { name, command }
    }
}

impl Provider for ExternalCliProvider {
    fn name(&self) -> &str {
        &self.name
    }

    fn kind(&self) -> &str {
        "external_cli"
    }

    fn chat_stream(
        &self,
        request: ChatRequest,
    ) -> Pin<Box<dyn Stream<Item = Result<ProviderEvent>> + Send>> {
        let command = self.command.clone();
        let prompt = request
            .messages
            .iter()
            .map(|m| format!("{}: {}", m.role_str(), m.content))
            .collect::<Vec<_>>()
            .join("\n\n");

        let s = stream! {
            let result = tokio::process::Command::new("sh")
                .arg("-c")
                .arg(format!("echo '{}' | {}", prompt.replace('\'', "'\\''"), command))
                .output()
                .await;

            match result {
                Ok(output) => {
                    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                    yield Ok(ProviderEvent::Delta(stdout));
                    yield Ok(ProviderEvent::Done(ProviderUsage::default()));
                }
                Err(e) => {
                    yield Err(anyhow::anyhow!("CLI provider failed: {}", e));
                }
            }
        };

        Box::pin(s)
    }
}

impl super::ChatMessage {
    pub fn role_str(&self) -> &str {
        match self.role {
            super::MessageRole::System => "system",
            super::MessageRole::User => "user",
            super::MessageRole::Assistant => "assistant",
            super::MessageRole::Tool => "tool",
        }
    }
}
