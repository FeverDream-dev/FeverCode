/// Bot commands supported by the Telegram integration.
#[derive(Debug, Clone, PartialEq)]
pub enum BotCommand {
    Status,
    Pause,
    Resume,
    Stop,
    Summary,
    Files,
    Log,
    Help,
}

impl BotCommand {
    /// Parse a command text into a BotCommand, if recognized.
    pub fn parse_command(text: &str) -> Option<Self> {
        match text.trim().to_ascii_lowercase().as_str() {
            "/status" => Some(BotCommand::Status),
            "/pause" => Some(BotCommand::Pause),
            "/resume" => Some(BotCommand::Resume),
            "/stop" => Some(BotCommand::Stop),
            "/summary" => Some(BotCommand::Summary),
            "/files" => Some(BotCommand::Files),
            "/log" => Some(BotCommand::Log),
            "/help" => Some(BotCommand::Help),
            _ => None,
        }
    }

    /// Produce a short response text corresponding to the command.
    pub fn response_text(&self) -> String {
        match self {
            BotCommand::Status => "Status: running".to_string(),
            BotCommand::Pause => "Paused".to_string(),
            BotCommand::Resume => "Resumed".to_string(),
            BotCommand::Stop => "Stopped".to_string(),
            BotCommand::Summary => "Summary: ...".to_string(),
            BotCommand::Files => "Files: ...".to_string(),
            BotCommand::Log => "Log: ...".to_string(),
            BotCommand::Help => "Help: These commands are supported".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_command_all() {
        let map = vec![
            ("/status", BotCommand::Status),
            ("/pause", BotCommand::Pause),
            ("/resume", BotCommand::Resume),
            ("/stop", BotCommand::Stop),
            ("/summary", BotCommand::Summary),
            ("/files", BotCommand::Files),
            ("/log", BotCommand::Log),
            ("/help", BotCommand::Help),
        ];
        for (text, expected) in map {
            assert_eq!(BotCommand::parse_command(text), Some(expected));
        }
    }

    #[test]
    fn test_response_text_variants() {
        // Ensure each variant returns a non-empty response string
        let all = [
            BotCommand::Status,
            BotCommand::Pause,
            BotCommand::Resume,
            BotCommand::Stop,
            BotCommand::Summary,
            BotCommand::Files,
            BotCommand::Log,
            BotCommand::Help,
        ];
        for cmd in &all {
            let s = cmd.response_text();
            assert!(!s.is_empty());
        }
    }
}
