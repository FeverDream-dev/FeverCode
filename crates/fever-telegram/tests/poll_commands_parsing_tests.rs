use fever_telegram::{BotCommand, TelegramService};
use serde_json::json;

#[test]
fn test_parse_updates_from_updates() {
    // Build a couple of simulated Telegram updates with commands
    let updates = vec![
        json!({"update_id": 1, "message": {"text": "/status"}}),
        json!({"update_id": 2, "message": {"text": "/help"}}),
    ];
    // Use the public helper to parse updates
    let cmds = TelegramService::parse_updates(updates);
    assert_eq!(cmds, vec![BotCommand::Status, BotCommand::Help]);
}
