#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SlashCommand {
    Help,
    Model(String),
    Clear,
    Settings,
    Quit,
    Version,
    Status,
    Role(String),
    Provider(String),
    Save,
    Theme(String),
    New,
    Doctor,
}

impl SlashCommand {
    pub fn parse(input: &str) -> Option<Self> {
        let input = input.trim();
        if !input.starts_with('/') {
            return None;
        }
        let input = &input[1..];
        let parts: Vec<&str> = input.splitn(2, ' ').collect();
        if parts.is_empty() {
            return None;
        }
        match parts[0] {
            "help" | "?" => Some(Self::Help),
            "model" => Some(Self::Model(parts.get(1).unwrap_or(&"").to_string())),
            "clear" => Some(Self::Clear),
            "settings" | "config" => Some(Self::Settings),
            "quit" | "exit" | "q" => Some(Self::Quit),
            "version" | "v" => Some(Self::Version),
            "status" => Some(Self::Status),
            "role" => Some(Self::Role(parts.get(1).unwrap_or(&"").to_string())),
            "provider" => Some(Self::Provider(parts.get(1).unwrap_or(&"").to_string())),
            "save" => Some(Self::Save),
            "theme" => Some(Self::Theme(parts.get(1).unwrap_or(&"").to_string())),
            "new" => Some(Self::New),
            "doctor" => Some(Self::Doctor),
            _ => None,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::Help => "help",
            Self::Model(_) => "model",
            Self::Clear => "clear",
            Self::Settings => "settings",
            Self::Quit => "quit",
            Self::Version => "version",
            Self::Status => "status",
            Self::Role(_) => "role",
            Self::Provider(_) => "provider",
            Self::Save => "save",
            Self::Theme(_) => "theme",
            Self::New => "new",
            Self::Doctor => "doctor",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Self::Help => "Show available commands",
            Self::Model(_) => "Switch or view current model",
            Self::Clear => "Clear chat history",
            Self::Settings => "Open settings screen",
            Self::Quit => "Quit Fever",
            Self::Version => "Show version",
            Self::Status => "Show provider/model status",
            Self::Role(_) => "Set or view current role",
            Self::Provider(_) => "Switch or view provider",
            Self::Save => "Save current session",
            Self::Theme(_) => "Switch or list themes",
            Self::New => "Start new session",
            Self::Doctor => "Run diagnostics",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_help() {
        assert_eq!(SlashCommand::parse("/help"), Some(SlashCommand::Help));
        assert_eq!(SlashCommand::parse("/?"), Some(SlashCommand::Help));
    }

    #[test]
    fn test_parse_model() {
        assert_eq!(
            SlashCommand::parse("/model gpt-4o"),
            Some(SlashCommand::Model("gpt-4o".to_string()))
        );
    }

    #[test]
    fn test_parse_clear() {
        assert_eq!(SlashCommand::parse("/clear"), Some(SlashCommand::Clear));
    }

    #[test]
    fn test_parse_unknown() {
        assert_eq!(SlashCommand::parse("/xyz"), None);
    }

    #[test]
    fn test_parse_no_slash() {
        assert_eq!(SlashCommand::parse("hello"), None);
    }

    #[test]
    fn test_name_and_description() {
        assert_eq!(SlashCommand::Help.name(), "help");
        assert_eq!(SlashCommand::Model(String::new()).name(), "model");
        assert_eq!(SlashCommand::Clear.description(), "Clear chat history");
    }
}
