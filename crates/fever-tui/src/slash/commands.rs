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
            _ => None,
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
}
