use crate::client::{RealTelegramClient, TelegramClient};
use crate::{
    command::BotCommand, config::TelegramConfig, error::TelegramError, event::TelegramEvent,
    rate_limiter::RateLimiter, state as state_mod,
};
use serde_json::Value;
use std::time::Duration;
use tracing::{debug, info, warn};

pub struct TelegramService {
    config: TelegramConfig,
    telegram_client: Box<dyn TelegramClient>,
    rate_limiter: RateLimiter,
    state: state_mod::AgentState,
    session_log: Vec<String>,
    _modified_files: Vec<String>,
    update_offset: Option<i64>,
}

impl TelegramService {
    pub fn new(config: TelegramConfig) -> Self {
        let secs = config.notify_interval_secs;
        let client = RealTelegramClient::new(config.bot_token.clone());
        Self {
            config,
            telegram_client: Box::new(client),
            rate_limiter: RateLimiter::new(Duration::from_secs(secs)),
            state: state_mod::AgentState::Idle,
            session_log: Vec::new(),
            _modified_files: Vec::new(),
            update_offset: None,
        }
    }

    pub fn with_client(config: TelegramConfig, client: Box<dyn TelegramClient>) -> Self {
        let secs = config.notify_interval_secs;
        Self {
            config,
            telegram_client: client,
            rate_limiter: RateLimiter::new(Duration::from_secs(secs)),
            state: state_mod::AgentState::Idle,
            session_log: Vec::new(),
            _modified_files: Vec::new(),
            update_offset: None,
        }
    }

    pub async fn start(&mut self) -> Result<(), TelegramError> {
        if self.config.chat_id.is_none() {
            info!("Telegram chat_id not linked; attempting auto-link");
            self.auto_link().await?;
        } else {
            info!(
                "Telegram service started with chat_id={}",
                self.config.chat_id.as_deref().unwrap_or("unknown")
            );
        }
        self.state = state_mod::AgentState::Running;
        Ok(())
    }

    pub async fn send_event(&mut self, event: TelegramEvent) -> Result<(), TelegramError> {
        let msg = event.format_message();
        debug!(%msg, "Preparing to send Telegram event");
        if self.config.chat_id.is_none() {
            info!(
                "Telegram chat_id not configured; caching event locally: {}",
                msg
            );
            self.session_log.push(msg);
            return Ok(());
        }
        let chat_id = self.config.chat_id.clone().unwrap();
        let _sent = self.rate_limiter.try_send(msg.clone());
        if _sent {
            debug!("Telegram rate limiter allowed immediate send");
        } else {
            debug!("Telegram rate limiter queued message");
        }
        let mut attempt: usize = 0;
        loop {
            match self
                .telegram_client
                .send_message(chat_id.clone(), msg.clone())
                .await
            {
                Ok(()) => break,
                Err(e) => {
                    attempt += 1;
                    if attempt >= 5 {
                        return Err(e);
                    }
                    let backoff: u64 = 1 << attempt;
                    warn!(
                        "Telegram send failed (attempt {}): {:?}. backing off {}s",
                        attempt, e, backoff
                    );
                    tokio::time::sleep(Duration::from_secs(backoff)).await;
                }
            }
        }
        self.session_log.push(msg);
        Ok(())
    }

    pub async fn send_urgent(&mut self, msg: &str) -> Result<(), TelegramError> {
        let _ = self.rate_limiter.force_send(msg.to_string());
        self.session_log.push(msg.to_string());
        Ok(())
    }

    pub async fn poll_commands(&mut self) -> Vec<BotCommand> {
        match self.telegram_client.get_updates(self.update_offset).await {
            Ok(updates) => {
                if let Some(last) = updates.last() {
                    if let Some(id) = last.get("update_id").and_then(|v| v.as_i64()) {
                        self.update_offset = Some(id + 1);
                    }
                }
                Self::parse_updates(updates)
            }
            Err(e) => {
                warn!("Failed to fetch updates: {:?}", e);
                Vec::new()
            }
        }
    }

    pub fn parse_updates(updates: Vec<Value>) -> Vec<BotCommand> {
        let mut out = Vec::new();
        for u in updates {
            if let Some(m) = u.get("message") {
                if let Some(text) = m.get("text").and_then(|t| t.as_str()) {
                    if let Some(cmd) = BotCommand::parse_command(text) {
                        out.push(cmd);
                    }
                }
            }
        }
        out
    }

    pub async fn auto_link(&mut self) -> Result<(), TelegramError> {
        loop {
            match self.telegram_client.get_updates(self.update_offset).await {
                Ok(updates) => {
                    for up in &updates {
                        if let Some(m) = up.get("message") {
                            if let Some(chat) = m.get("chat") {
                                if let Some(cid) = chat.get("id").and_then(|v| v.as_i64()) {
                                    self.config.chat_id = Some(cid.to_string());
                                    let _ = self
                                        .telegram_client
                                        .send_message(cid.to_string(), "Linked!".to_string())
                                        .await;
                                    info!("Linked to chat_id={}", cid);
                                    if let Some(id) = up.get("update_id").and_then(|v| v.as_i64()) {
                                        self.update_offset = Some(id + 1);
                                    }
                                    return Ok(());
                                }
                            }
                        }
                    }
                    if let Some(id) = updates
                        .last()
                        .and_then(|u| u.get("update_id").and_then(|v| v.as_i64()))
                    {
                        self.update_offset = Some(id + 1);
                    }
                    tokio::time::sleep(Duration::from_millis(200)).await;
                }
                Err(e) => {
                    warn!("Auto-link poll failed: {:?}", e);
                    tokio::time::sleep(Duration::from_secs(1)).await;
                }
            }
        }
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
