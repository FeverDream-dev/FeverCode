use fever_telegram::TelegramConfig;
use std::env;

#[test]
fn test_from_env_config_present() {
    let _lock = fever_telegram::config::test_env_lock();
    unsafe {
        env::set_var("TELEGRAM_BOT_TOKEN", "test-token");
    }
    unsafe {
        env::set_var("TELEGRAM_CHAT_ID", "12345");
    }
    unsafe {
        env::set_var("TELEGRAM_NOTIFY_INTERVAL", "10");
    }
    unsafe {
        env::set_var("TELEGRAM_LOOP_MODE", "false");
    }

    let cfg = TelegramConfig::from_env().expect("config should be created");
    assert_eq!(cfg.bot_token, "test-token");
    assert_eq!(cfg.chat_id.as_deref(), Some("12345"));
    assert_eq!(cfg.notify_interval_secs, 10);
    assert!(!cfg.loop_mode);
    // cleanup
    unsafe {
        env::remove_var("TELEGRAM_BOT_TOKEN");
    }
    unsafe {
        env::remove_var("TELEGRAM_CHAT_ID");
    }
    unsafe {
        env::remove_var("TELEGRAM_NOTIFY_INTERVAL");
    }
    unsafe {
        env::remove_var("TELEGRAM_LOOP_MODE");
    }
}

#[test]
fn test_from_env_config_missing_token() {
    let _lock = fever_telegram::config::test_env_lock();
    unsafe {
        env::remove_var("TELEGRAM_BOT_TOKEN");
    }
    unsafe {
        env::remove_var("TELEGRAM_CHAT_ID");
    }
    let cfg = TelegramConfig::from_env();
    assert!(cfg.is_none());
}
