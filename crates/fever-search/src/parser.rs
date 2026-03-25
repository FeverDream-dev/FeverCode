use crate::error::{SearchError, SearchResult};
use crate::result::{SearchResultItem, SearchResults};
use scraper::{Html, Selector};

pub trait SearchParser: Send + Sync {
    fn name(&self) -> &str;

    fn parse_results(&self, html: &str, query: &str) -> SearchResult<SearchResults>;
}

pub struct DuckDuckGoParser;

impl DuckDuckGoParser {
    pub fn new() -> Self {
        Self
    }
}

impl SearchParser for DuckDuckGoParser {
    fn name(&self) -> &str {
        "duckduckgo"
    }

    fn parse_results(&self, html: &str, query: &str) -> SearchResult<SearchResults> {
        let document = Html::parse_document(html);

        let result_selector = Selector::parse(".result").unwrap();
        let title_selector = Selector::parse(".result__title a").unwrap();
        let snippet_selector = Selector::parse(".result__snippet").unwrap();
        let url_selector = Selector::parse(".result__url").unwrap();

        let mut items = Vec::new();

        for result in document.select(&result_selector) {
            let title = result
                .select(&title_selector)
                .next()
                .and_then(|el| el.text().next())
                .unwrap_or("")
                .to_string();

            let snippet = result
                .select(&snippet_selector)
                .next()
                .and_then(|el| el.text().next())
                .unwrap_or("")
                .to_string();

            let url = result
                .select(&url_selector)
                .next()
                .and_then(|el| el.value().attr("href"))
                .unwrap_or("")
                .to_string();

            if !title.is_empty() && !url.is_empty() {
                items.push(SearchResultItem::new(title, url, snippet));
            }
        }

        if items.is_empty() {
            return Err(SearchError::NoResults);
        }

        let total = items.len();
        Ok(SearchResults {
            query: query.to_string(),
            items,
            total,
            engine: self.name().to_string(),
            cached: false,
            timestamp: chrono::Utc::now(),
        })
    }
}

impl Default for DuckDuckGoParser {
    fn default() -> Self {
        Self::new()
    }
}

pub struct SearxngParser {
    pub base_url: String,
}

impl SearxngParser {
    pub fn new(base_url: String) -> Self {
        Self { base_url }
    }
}

impl SearchParser for SearxngParser {
    fn name(&self) -> &str {
        "searxng"
    }

    fn parse_results(&self, html: &str, query: &str) -> SearchResult<SearchResults> {
        let document = Html::parse_document(html);

        let result_selector = Selector::parse(".result").unwrap();
        let title_selector = Selector::parse("h3 a").unwrap();
        let snippet_selector = Selector::parse(".content").unwrap();
        let url_selector = Selector::parse("h3 a").unwrap();

        let mut items = Vec::new();

        for result in document.select(&result_selector) {
            let title = result
                .select(&title_selector)
                .next()
                .and_then(|el| el.text().next())
                .unwrap_or("")
                .to_string();

            let snippet = result
                .select(&snippet_selector)
                .next()
                .and_then(|el| el.text().next())
                .unwrap_or("")
                .to_string();

            let url = result
                .select(&url_selector)
                .next()
                .and_then(|el| el.value().attr("href"))
                .unwrap_or("")
                .to_string();

            if !title.is_empty() && !url.is_empty() {
                items.push(SearchResultItem::new(title, url, snippet));
            }
        }

        if items.is_empty() {
            return Err(SearchError::NoResults);
        }

        let total = items.len();
        Ok(SearchResults {
            query: query.to_string(),
            items,
            total,
            engine: self.name().to_string(),
            cached: false,
            timestamp: chrono::Utc::now(),
        })
    }
}
