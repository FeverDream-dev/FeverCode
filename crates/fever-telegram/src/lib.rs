//! FeverCode Telegram integration library
//!
//! This crate provides a lightweight abstraction over Telegram interactions
//! used by Fever Code. It includes configuration, event formatting, a
//! simplified service, rate limiting, and error handling.
pub mod api;
pub mod client;
pub mod command;
pub mod config;
pub mod error;
pub mod event;
pub mod rate_limiter;
pub mod reconnect;
pub mod service;
pub mod state;

// Convenience re-exports for tests and external usage.
pub use api::TelegramApi;
pub use command::BotCommand;
pub use config::TelegramConfig;
pub use error::TelegramError;
pub use event::TelegramEvent;
pub use rate_limiter::RateLimiter;
pub use service::TelegramService;
pub use state::AgentState;
