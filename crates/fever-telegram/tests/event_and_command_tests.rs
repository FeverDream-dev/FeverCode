use fever_telegram::command::BotCommand;
use fever_telegram::event::TelegramEvent;

#[test]
fn test_event_format_messages() {
    let e = TelegramEvent::AgentStarted {
        task: "setup".to_string(),
    };
    assert_eq!(e.format_message(), "🚀 Agent started: setup");

    let e2 = TelegramEvent::Thinking {
        summary: "processing".to_string(),
    };
    assert_eq!(e2.format_message(), "💭 Thinking: processing");

    let e3 = TelegramEvent::FileRead {
        path: "/tmp/file.txt".to_string(),
    };
    assert_eq!(e3.format_message(), "📄 Read: /tmp/file.txt");
}

#[test]
fn test_command_parse() {
    assert_eq!(
        BotCommand::parse_command("/status"),
        Some(BotCommand::Status)
    );
    assert_eq!(BotCommand::parse_command("/pause"), Some(BotCommand::Pause));
    assert_eq!(BotCommand::parse_command("unknown"), None);
}
