/// Errors produced by the fever-telegram integration.
#[derive(Debug, Clone, PartialEq)]
pub enum TelegramError {
    Network(String),
    Api { code: i32, message: String },
    NotConfigured,
    RateLimited,
    ChatIdNotRegistered,
}

impl std::fmt::Display for TelegramError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TelegramError::Network(s) => write!(f, "Network error: {}", s),
            TelegramError::Api { code, message } => {
                write!(f, "Telegram API error {}: {}", code, message)
            }
            TelegramError::NotConfigured => write!(f, "Telegram not configured"),
            TelegramError::RateLimited => write!(f, "Rate limited"),
            TelegramError::ChatIdNotRegistered => write!(f, "Chat id not registered"),
        }
    }
}

impl std::error::Error for TelegramError {}
