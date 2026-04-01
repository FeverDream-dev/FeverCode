use crate::{config::TelegramConfig, rate_limiter::RateLimiter, state as state_mod, event::TelegramEvent, command::BotCommand, error::TelegramError};
use tracing::{debug, info};
use reqwest;

pub struct TelegramService {
    config: TelegramConfig,
    _client: reqwest::Client,
    rate_limiter: RateLimiter,
    state: state_mod::AgentState,
    session_log: Vec<String>,
    _modified_files: Vec<String>,
}

impl TelegramService {
    pub fn new(config: TelegramConfig) -> Self {
        let client = reqwest::Client::builder().build().expect("http client");
        let rate_limiter = RateLimiter::new(std::time::Duration::from_secs(config.notify_interval_secs));
        Self {
            config,
            _client: client,
            rate_limiter,
            state: state_mod::AgentState::Idle,
            session_log: Vec::new(),
            _modified_files: Vec::new(),
        }
    }

    pub async fn start(&mut self) -> Result<(), TelegramError> {
        // Registration flow: if chat_id is missing, print instruction and rely on external registration.
        if let Some(ref cid) = self.config.chat_id {
            info!("Telegram service started with chat_id={}", cid);
        } else {
            info!("Telegram service started without chat_id; will store events locally until registered");
        }
        self.state = state_mod::AgentState::Running;
        Ok(())
    }

    pub async fn send_event(&mut self, event: TelegramEvent) -> Result<(), TelegramError> {
        let msg = event.format_message();
        debug!(%msg, "Preparing to send Telegram event");
        // If no chat configured, just log locally.
        if self.config.chat_id.is_none() {
            info!("Telegram chat_id not configured; caching event locally: {}", msg);
            self.session_log.push(msg);
            return Ok(());
        }
        let _sent = self.rate_limiter.try_send(msg.clone());
        if _sent {
            debug!("Telegram rate limiter allowed immediate send for message: {}", msg);
        } else {
            debug!("Telegram rate limiter queued message: {}", msg);
        }
        // In a real implementation we'd call TelegramApi here; for tests we keep it non-blocking.
        self.session_log.push(msg);
        Ok(())
    }

    pub async fn send_urgent(&mut self, msg: &str) -> Result<(), TelegramError> {
        let _ = self.rate_limiter.force_send(msg.to_string());
        self.session_log.push(msg.to_string());
        Ok(())
    }

    pub async fn poll_commands(&mut self) -> Vec<BotCommand> {
        // Long-polling for updates would happen here; return empty for testability.
        Vec::new()
    }

    pub async fn handle_command(&mut self, cmd: BotCommand) -> Result<(), TelegramError> {
        self.session_log.push(cmd.response_text());
        Ok(())
    }

    pub async fn stop(&mut self) -> Result<(), TelegramError> {
        info!("Telegram service stopping");
        self.state = state_mod::AgentState::Stopped;
        Ok(())
    }
}
