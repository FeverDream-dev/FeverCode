pub mod cache;
pub mod client;
pub mod error;
pub mod parser;
pub mod result;

pub use cache::SearchCache;
pub use client::{SearchClient, SearchConfig};
pub use error::{SearchError, SearchResult};
pub use parser::{DuckDuckGoParser, SearchParser, SearxngParser};
pub use result::{SearchResultItem, SearchResults};
