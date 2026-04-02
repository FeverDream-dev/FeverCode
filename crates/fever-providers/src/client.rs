use crate::adapter::ProviderAdapter;
use crate::error::{ProviderError, ProviderResult};
use crate::models::{ChatRequest, ChatResponse, StreamChunk};
use futures::Stream;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::time::Duration;

fn rand_delay() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0)
}

pub struct ProviderClient {
    providers: HashMap<String, Arc<dyn ProviderAdapter>>,
    default_provider: Option<String>,
}

impl ProviderClient {
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
            default_provider: None,
        }
    }

    pub fn register(&mut self, provider: Arc<dyn ProviderAdapter>, is_default: bool) {
        let name = provider.name().to_string();
        self.providers.insert(name.clone(), provider);
        if is_default {
            self.default_provider = Some(name);
        }
    }

    pub fn get_provider(&self, name: &str) -> Option<&Arc<dyn ProviderAdapter>> {
        self.providers.get(name)
    }

    pub async fn chat(&self, request: &ChatRequest) -> ProviderResult<ChatResponse> {
        let provider_name = self.resolve_provider(&request.model)?;
        let provider = self
            .get_provider(&provider_name)
            .ok_or_else(|| ProviderError::ModelNotFound(provider_name.clone()))?;

        let max_retries = 3u32;
        let mut last_err = None;

        for attempt in 0..max_retries {
            match provider.chat(request).await {
                Ok(response) => return Ok(response),
                Err(e) if Self::is_retryable(&e) && attempt + 1 < max_retries => {
                    tracing::warn!(
                        provider = %provider_name,
                        attempt = attempt + 1,
                        error = %e,
                        "Retrying LLM request"
                    );
                    tokio::time::sleep(Self::backoff_delay(attempt)).await;
                    last_err = Some(e);
                }
                Err(e) => return Err(e),
            }
        }

        Err(last_err.unwrap_or_else(|| {
            ProviderError::Unknown("Retry loop exhausted with no error".to_string())
        }))
    }

    pub async fn chat_stream(
        &self,
        request: &ChatRequest,
    ) -> ProviderResult<Box<dyn Stream<Item = ProviderResult<StreamChunk>> + Send + Unpin>> {
        let provider_name = self.resolve_provider(&request.model)?;
        let provider = self
            .get_provider(&provider_name)
            .ok_or_else(|| ProviderError::ModelNotFound(provider_name.clone()))?;

        let max_retries = 3u32;
        let mut last_err = None;

        for attempt in 0..max_retries {
            match provider.chat_stream(request).await {
                Ok(stream) => return Ok(stream),
                Err(e) if Self::is_retryable(&e) && attempt + 1 < max_retries => {
                    tracing::warn!(
                        provider = %provider_name,
                        attempt = attempt + 1,
                        error = %e,
                        "Retrying LLM stream request"
                    );
                    tokio::time::sleep(Self::backoff_delay(attempt)).await;
                    last_err = Some(e);
                }
                Err(e) => return Err(e),
            }
        }

        Err(last_err.unwrap_or_else(|| {
            ProviderError::Unknown("Retry loop exhausted with no error".to_string())
        }))
    }

    pub fn list_providers(&self) -> Vec<String> {
        self.providers.keys().cloned().collect()
    }

    pub fn list_models(&self) -> Vec<String> {
        self.providers
            .values()
            .flat_map(|p| p.list_models())
            .collect()
    }

    pub fn set_default_provider(&mut self, name: String) -> ProviderResult<()> {
        if !self.providers.contains_key(&name) {
            return Err(ProviderError::ModelNotFound(name));
        }
        self.default_provider = Some(name);
        Ok(())
    }

    pub fn get_default_provider(&self) -> Option<&str> {
        self.default_provider.as_deref()
    }

    fn is_retryable(err: &ProviderError) -> bool {
        matches!(
            err,
            ProviderError::RateLimit { .. }
                | ProviderError::Timeout(_)
                | ProviderError::Unavailable(_)
                | ProviderError::Http(_)
        )
    }

    fn backoff_delay(attempt: u32) -> Duration {
        let base_ms = 1_000u64 * (1 << attempt);
        let jitter_ms = (rand_delay() % 400).saturating_sub(200);
        Duration::from_millis(base_ms.saturating_add(jitter_ms))
    }

    fn resolve_provider(&self, model: &str) -> ProviderResult<String> {
        if let Some(provider_name) = model.split('/').next() {
            if self.providers.contains_key(provider_name) {
                return Ok(provider_name.to_string());
            }
        }

        if let Some(default) = &self.default_provider {
            return Ok(default.clone());
        }

        Err(ProviderError::InvalidRequest(
            "No provider specified and no default set".to_string(),
        ))
    }
}

impl Default for ProviderClient {
    fn default() -> Self {
        Self::new()
    }
}

pub struct StreamingResponse {
    pub stream: Box<dyn Stream<Item = ProviderResult<StreamChunk>> + Send + Unpin>,
}
