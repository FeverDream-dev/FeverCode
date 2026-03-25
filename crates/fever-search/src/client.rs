use crate::cache::SearchCache;
use crate::error::{SearchError, SearchResult};
use crate::parser::{DuckDuckGoParser, SearchParser, SearxngParser};
use crate::result::SearchResults;
use reqwest::Client;
use std::sync::Arc;

#[derive(Clone)]
pub struct SearchConfig {
    pub max_results: usize,
    pub timeout_secs: u64,
    pub cache_enabled: bool,
    pub cache_ttl_hours: u64,
    pub searxng_url: Option<String>,
}

impl Default for SearchConfig {
    fn default() -> Self {
        Self {
            max_results: 10,
            timeout_secs: 30,
            cache_enabled: true,
            cache_ttl_hours: 24,
            searxng_url: None,
        }
    }
}

pub struct SearchClient {
    http: Client,
    config: SearchConfig,
    cache: Arc<SearchCache>,
    ddg_parser: DuckDuckGoParser,
    searxng_parser: Option<SearxngParser>,
}

impl SearchClient {
    pub fn new(config: SearchConfig, cache: Arc<SearchCache>) -> Self {
        let timeout = std::time::Duration::from_secs(config.timeout_secs);
        let http = Client::builder()
            .timeout(timeout)
            .user_agent("Mozilla/5.0 (X11; Linux x86_64) FeverCode/1.0")
            .build()
            .unwrap();

        let searxng_parser = config
            .searxng_url
            .as_ref()
            .map(|url| SearxngParser::new(url.clone()));

        Self {
            http,
            config,
            cache,
            ddg_parser: DuckDuckGoParser::new(),
            searxng_parser,
        }
    }

    pub async fn search(&self, query: &str) -> SearchResult<SearchResults> {
        if self.config.cache_enabled {
            if let Some(cached) = self.cache.get(query).await {
                let mut results = cached;
                results.cached = true;
                return Ok(results);
            }
        }

        let results = self.perform_search(query).await?;

        if self.config.cache_enabled {
            let _ = self.cache.set(query, &results).await;
        }

        Ok(results)
    }

    async fn perform_search(&self, query: &str) -> SearchResult<SearchResults> {
        let encoded = urlencoding::encode(query);
        let html = self.fetch_html(&encoded).await?;

        let results = self.parse_html(&html, query)?;

        let mut limited = results;
        limited.items.truncate(self.config.max_results);
        limited.total = limited.items.len();

        Ok(limited)
    }

    async fn fetch_html(&self, query: &str) -> SearchResult<String> {
        let url = "https://duckduckgo.com/html/";
        let resp = self
            .http
            .get(url)
            .query(&[("q", query)])
            .send()
            .await
            .map_err(|e| SearchError::Http(e.to_string()))?;

        if !resp.status().is_success() {
            return Err(SearchError::Http(format!("Status: {}", resp.status())));
        }

        let html = resp
            .text()
            .await
            .map_err(|e| SearchError::Http(e.to_string()))?;

        Ok(html)
    }

    fn parse_html(&self, html: &str, query: &str) -> SearchResult<SearchResults> {
        if let Some(searxng) = &self.searxng_parser {
            match searxng.parse_results(html, query) {
                Ok(results) => return Ok(results),
                Err(SearchError::NoResults) => {}
                Err(e) => return Err(e),
            }
        }

        self.ddg_parser.parse_results(html, query)
    }
}
