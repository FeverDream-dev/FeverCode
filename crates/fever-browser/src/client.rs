use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageSnapshot {
    pub url: String,
    pub title: String,
    pub elements: Vec<PageElement>,
    pub console_messages: Vec<ConsoleMessage>,
    pub network_requests: Vec<NetworkRequest>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageElement {
    pub uid: String,
    pub tag: String,
    pub text: String,
    pub attributes: Vec<(String, String)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsoleMessage {
    pub level: String,
    pub text: String,
    pub source: Option<String>,
    pub line: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkRequest {
    pub url: String,
    pub method: String,
    pub status: Option<u16>,
    pub content_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Screenshot {
    pub data: String,
    pub format: String,
}

pub struct BrowserClient {
    enabled: bool,
}

impl BrowserClient {
    pub fn new() -> Self {
        Self { enabled: true }
    }

    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
}

impl Default for BrowserClient {
    fn default() -> Self {
        Self::new()
    }
}
