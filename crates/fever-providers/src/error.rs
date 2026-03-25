use thiserror::Error;

pub type ProviderResult<T> = std::result::Result<T, ProviderError>;

#[derive(Debug, Error)]
pub enum ProviderError {
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Authentication failed: {0}")]
    Auth(String),

    #[error("Rate limit exceeded: {provider}")]
    RateLimit { provider: String },

    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    #[error("Model not found: {0}")]
    ModelNotFound(String),

    #[error("Provider unavailable: {0}")]
    Unavailable(String),

    #[error("Tool calling not supported: {0}")]
    ToolCallingNotSupported(String),

    #[error("HTTP error: {0}")]
    Http(String),

    #[error("API error: {code} - {message}")]
    Api { code: String, message: String },

    #[error("Timeout: {0}")]
    Timeout(String),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("Unknown error: {0}")]
    Unknown(String),
}
