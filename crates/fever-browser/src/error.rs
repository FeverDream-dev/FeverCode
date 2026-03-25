use thiserror::Error;

pub type BrowserResult<T> = std::result::Result<T, BrowserError>;

#[derive(Debug, Error)]
pub enum BrowserError {
    #[error("Connection error: {0}")]
    Connection(String),

    #[error("Page not found: {0}")]
    PageNotFound(String),

    #[error("Navigation error: {0}")]
    Navigation(String),

    #[error("Script execution error: {0}")]
    Script(String),

    #[error("Element not found: {0}")]
    ElementNotFound(String),

    #[error("Timeout: {0}")]
    Timeout(String),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("Not connected to browser")]
    NotConnected,

    #[error("Invalid response: {0}")]
    InvalidResponse(String),
}
