use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResults {
    pub query: String,
    pub items: Vec<SearchResultItem>,
    pub total: usize,
    pub engine: String,
    pub cached: bool,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResultItem {
    pub title: String,
    pub url: String,
    pub snippet: String,
    pub engine_specific: Option<serde_json::Value>,
}

impl SearchResultItem {
    pub fn new(title: String, url: String, snippet: String) -> Self {
        Self {
            title,
            url,
            snippet,
            engine_specific: None,
        }
    }
}
