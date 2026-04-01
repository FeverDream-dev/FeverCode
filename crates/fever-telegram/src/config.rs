use std::env;

#[derive(Debug, Clone, PartialEq)]
pub struct TelegramConfig {
    pub bot_token: String,
    pub chat_id: Option<String>,
    pub notify_interval_secs: u64,
    pub loop_mode: bool,
}

impl TelegramConfig {
    pub fn from_env() -> Option<Self> {
        let token = match env::var("TELEGRAM_BOT_TOKEN") {
            Ok(t) if !t.trim().is_empty() => t,
            _ => return None,
        };

        let chat_id = env::var("TELEGRAM_CHAT_ID")
            .ok()
            .filter(|s| !s.trim().is_empty());

        let notify_interval_secs = env::var("TELEGRAM_NOTIFY_INTERVAL")
            .ok()
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(5);

        let loop_mode = env::var("TELEGRAM_LOOP_MODE")
            .ok()
            .and_then(|s| {
                let v = s.to_ascii_lowercase();
                if v == "1" || v == "true" || v == "yes" {
                    Some(true)
                } else if v == "0" || v == "false" || v == "no" {
                    Some(false)
                } else {
                    None
                }
            })
            .unwrap_or(true);

        Some(TelegramConfig {
            bot_token: token,
            chat_id,
            notify_interval_secs,
            loop_mode,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn from_env_minimal() {
        unsafe {
            std::env::set_var("TELEGRAM_BOT_TOKEN", "tok");
        }
        unsafe {
            std::env::remove_var("TELEGRAM_CHAT_ID");
        }
        let cfg = TelegramConfig::from_env();
        assert!(cfg.is_some());
    }

    #[test]
    fn from_env_with_chat() {
        unsafe {
            std::env::set_var("TELEGRAM_BOT_TOKEN", "tok");
        }
        unsafe {
            std::env::set_var("TELEGRAM_CHAT_ID", "123");
        }
        let cfg = TelegramConfig::from_env();
        assert!(cfg.is_some());
        assert_eq!(cfg.unwrap().chat_id, Some("123".to_string()));
        unsafe {
            std::env::remove_var("TELEGRAM_CHAT_ID");
        }
        unsafe {
            std::env::remove_var("TELEGRAM_BOT_TOKEN");
        }
    }
}
