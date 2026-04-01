//! FeverCode Telegram integration library
//!
//! This crate provides a lightweight abstraction over Telegram interactions
//! used by Fever Code. It includes configuration, event formatting, a
//! simplified service, rate limiting, and error handling.
pub mod config;
pub mod service;
pub mod event;
pub mod command;
pub mod api;
pub mod error;
pub mod rate_limiter;
pub mod state;

// Convenience re-exports for tests and external usage.
pub use config::TelegramConfig;
pub use service::TelegramService;
pub use event::TelegramEvent;
pub use command::BotCommand;
pub use rate_limiter::RateLimiter;
pub use state::AgentState;
pub use api::TelegramApi;
pub use error::TelegramError;
