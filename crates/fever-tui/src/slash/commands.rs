use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;

/// Centralized, richly described slash command specs.
pub struct SlashCommandSpec {
    pub name: &'static str,
    pub aliases: &'static [&'static str],
    pub summary: &'static str,
    pub argument_hint: Option<&'static str>,
    pub category: CommandCategory,
    pub requires_provider: bool,
    pub safe_in_mock: bool,
    pub destructive: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CommandCategory {
    Session,
    Config,
    Model,
    Permissions,
    Workspace,
    Tools,
    Diagnostics,
    Help,
    Appearance,
    Agent,
    Export,
}

/// Comprehensive registry of all slash commands.
pub const SLASH_COMMAND_SPECS: &[SlashCommandSpec] = &[
    // SESSION
    SlashCommandSpec {
        name: "help",
        aliases: &[],
        summary: "Show available commands",
        argument_hint: None,
        category: CommandCategory::Session,
        requires_provider: false,
        safe_in_mock: true,
        destructive: false,
    },
    SlashCommandSpec {
        name: "new",
        aliases: &[],
        summary: "Start a new session",
        argument_hint: None,
        category: CommandCategory::Session,
        requires_provider: false,
        safe_in_mock: true,
        destructive: false,
    },
    SlashCommandSpec {
        name: "save",
        aliases: &[],
        summary: "Save current session",
        argument_hint: None,
        category: CommandCategory::Session,
        requires_provider: false,
        safe_in_mock: true,
        destructive: false,
    },
    SlashCommandSpec {
        name: "session",
        aliases: &["list", "ls"],
        summary: "List or manage sessions",
        argument_hint: Some("[list|clear|resume <id>]"),
        category: CommandCategory::Session,
        requires_provider: false,
        safe_in_mock: true,
        destructive: false,
    },
    SlashCommandSpec {
        name: "clear",
        aliases: &[],
        summary: "Clear conversation history",
        argument_hint: None,
        category: CommandCategory::Session,
        requires_provider: false,
        safe_in_mock: true,
        destructive: false,
    },
    SlashCommandSpec {
        name: "compact",
        aliases: &[],
        summary: "Compact conversation context",
        argument_hint: None,
        category: CommandCategory::Session,
        requires_provider: false,
        safe_in_mock: true,
        destructive: false,
    },
    SlashCommandSpec {
        name: "quit",
        aliases: &["exit", "q"],
        summary: "Quit Fever Code",
        argument_hint: None,
        category: CommandCategory::Session,
        requires_provider: false,
        safe_in_mock: true,
        destructive: true,
    },
    SlashCommandSpec {
        name: "export",
        aliases: &[],
        summary: "Export session to file",
        argument_hint: Some("[<path>...]"),
        category: CommandCategory::Export,
        requires_provider: false,
        safe_in_mock: true,
        destructive: false,
    },
    // CONFIG
    SlashCommandSpec {
        name: "settings",
        aliases: &["config"],
        summary: "Open settings screen",
        argument_hint: None,
        category: CommandCategory::Config,
        requires_provider: false,
        safe_in_mock: true,
        destructive: false,
    },
    SlashCommandSpec {
        name: "doctor",
        aliases: &[],
        summary: "Run system diagnostics",
        argument_hint: None,
        category: CommandCategory::Diagnostics,
        requires_provider: false,
        safe_in_mock: true,
        destructive: false,
    },
    SlashCommandSpec {
        name: "workspace",
        aliases: &[],
        summary: "Show workspace info",
        argument_hint: None,
        category: CommandCategory::Workspace,
        requires_provider: false,
        safe_in_mock: true,
        destructive: false,
    },
    SlashCommandSpec {
        name: "keybindings",
        aliases: &[],
        summary: "Show keyboard shortcuts",
        argument_hint: None,
        category: CommandCategory::Appearance,
        requires_provider: false,
        safe_in_mock: true,
        destructive: false,
    },
    // MODEL
    SlashCommandSpec {
        name: "model",
        aliases: &[],
        summary: "Switch or view model",
        argument_hint: Some("<name>"),
        category: CommandCategory::Model,
        requires_provider: true,
        safe_in_mock: true,
        destructive: false,
    },
    SlashCommandSpec {
        name: "provider",
        aliases: &[],
        summary: "Switch or view provider",
        argument_hint: Some("<name>"),
        category: CommandCategory::Model,
        requires_provider: false,
        safe_in_mock: true,
        destructive: false,
    },
    SlashCommandSpec {
        name: "role",
        aliases: &[],
        summary: "Set agent role",
        argument_hint: Some("<name>"),
        category: CommandCategory::Model,
        requires_provider: false,
        safe_in_mock: true,
        destructive: false,
    },
    // PERMISSIONS
    SlashCommandSpec {
        name: "permissions",
        aliases: &["perms"],
        summary: "View or change permission mode",
        argument_hint: Some("[read|write|full]"),
        category: CommandCategory::Permissions,
        requires_provider: false,
        safe_in_mock: true,
        destructive: false,
    },
    SlashCommandSpec {
        name: "readonly",
        aliases: &[],
        summary: "Switch to read-only mode",
        argument_hint: None,
        category: CommandCategory::Permissions,
        requires_provider: false,
        safe_in_mock: true,
        destructive: false,
    },
    // WORKSPACE
    SlashCommandSpec {
        name: "diff",
        aliases: &[],
        summary: "Show git diff",
        argument_hint: None,
        category: CommandCategory::Workspace,
        requires_provider: false,
        safe_in_mock: true,
        destructive: false,
    },
    SlashCommandSpec {
        name: "git",
        aliases: &[],
        summary: "Run git command",
        argument_hint: Some("<command>"),
        category: CommandCategory::Workspace,
        requires_provider: false,
        safe_in_mock: true,
        destructive: true,
    },
    // TOOLS
    SlashCommandSpec {
        name: "tools",
        aliases: &[],
        summary: "List available tools",
        argument_hint: None,
        category: CommandCategory::Tools,
        requires_provider: false,
        safe_in_mock: true,
        destructive: false,
    },
    SlashCommandSpec {
        name: "mcp",
        aliases: &[],
        summary: "Manage MCP servers",
        argument_hint: Some("[list|<name>]"),
        category: CommandCategory::Tools,
        requires_provider: false,
        safe_in_mock: true,
        destructive: false,
    },
    // DIAGNOSTICS
    SlashCommandSpec {
        name: "status",
        aliases: &[],
        summary: "Show provider/model status",
        argument_hint: None,
        category: CommandCategory::Diagnostics,
        requires_provider: false,
        safe_in_mock: true,
        destructive: false,
    },
    SlashCommandSpec {
        name: "mock",
        aliases: &["demo"],
        summary: "Toggle or enable mock mode",
        argument_hint: None,
        category: CommandCategory::Config,
        requires_provider: false,
        safe_in_mock: true,
        destructive: false,
    },
    SlashCommandSpec {
        name: "version",
        aliases: &["v"],
        summary: "Show version",
        argument_hint: None,
        category: CommandCategory::Diagnostics,
        requires_provider: false,
        safe_in_mock: true,
        destructive: false,
    },
    SlashCommandSpec {
        name: "tokens",
        aliases: &[],
        summary: "Show token usage",
        argument_hint: None,
        category: CommandCategory::Diagnostics,
        requires_provider: false,
        safe_in_mock: true,
        destructive: false,
    },
    SlashCommandSpec {
        name: "cost",
        aliases: &[],
        summary: "Show estimated cost",
        argument_hint: None,
        category: CommandCategory::Diagnostics,
        requires_provider: false,
        safe_in_mock: true,
        destructive: false,
    },
    SlashCommandSpec {
        name: "context",
        aliases: &[],
        summary: "Show context window usage",
        argument_hint: None,
        category: CommandCategory::Diagnostics,
        requires_provider: false,
        safe_in_mock: true,
        destructive: false,
    },
    SlashCommandSpec {
        name: "time",
        aliases: &[],
        summary: "Show request timing",
        argument_hint: None,
        category: CommandCategory::Diagnostics,
        requires_provider: false,
        safe_in_mock: true,
        destructive: false,
    },
    SlashCommandSpec {
        name: "logs",
        aliases: &[],
        summary: "Show recent log entries",
        argument_hint: None,
        category: CommandCategory::Diagnostics,
        requires_provider: false,
        safe_in_mock: true,
        destructive: false,
    },
    // APPEARANCE & AGENT
    SlashCommandSpec {
        name: "theme",
        aliases: &[],
        summary: "Switch theme",
        argument_hint: Some("<name>"),
        category: CommandCategory::Appearance,
        requires_provider: false,
        safe_in_mock: true,
        destructive: false,
    },
    SlashCommandSpec {
        name: "preprompt",
        aliases: &[],
        summary: "Manage pre-prompt/system behavior",
        argument_hint: Some("[on|off|<mode>]"),
        category: CommandCategory::Agent,
        requires_provider: false,
        safe_in_mock: true,
        destructive: false,
    },
    // (No placeholder entries here)
];

/// New centralized SlashCommand enum with per-variant parsed arguments.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SlashCommand {
    Help,
    Model(Option<String>),
    Clear,
    Settings,
    Quit,
    Version,
    Status,
    Mock,
    Role(Option<String>),
    Provider(Option<String>),
    Save,
    Theme(Option<String>),
    New,
    Doctor,
    Permissions(Option<String>),
    ReadOnly,
    Session(Option<String>),
    Mcp(Option<String>),
    Preprompt(Option<String>),
    Tokens,
    Cost,
    Context,
    Time,
    Tools,
    Diff,
    Git(Option<String>),
    Export(Option<String>),
    Unknown(String),
}

impl SlashCommand {
    pub fn all_specs() -> &'static [SlashCommandSpec] {
        SLASH_COMMAND_SPECS
    }

    pub fn find_by_name(name: &str) -> Option<&'static SlashCommandSpec> {
        let name = name.to_lowercase();
        SLASH_COMMAND_SPECS.iter().find(|spec| {
            spec.name.eq_ignore_ascii_case(&name)
                || spec.aliases.iter().any(|a| a.eq_ignore_ascii_case(&name))
        })
    }

    pub fn category_specs(category: &CommandCategory) -> Vec<&'static SlashCommandSpec> {
        let mut v: Vec<_> = SLASH_COMMAND_SPECS
            .iter()
            .filter(|s| &s.category == category)
            .collect();
        v.sort_by(|a, b| a.name.cmp(b.name));
        v
    }

    pub fn find_specs(query: &str) -> Vec<&'static SlashCommandSpec> {
        let q = query.trim();
        if q.is_empty() {
            // Sort by category then name for deterministic results
            let mut all: Vec<_> = SLASH_COMMAND_SPECS.iter().collect();
            all.sort_by(|a, b| {
                let ca = &a.category as *const _ as usize;
                let cb = &b.category as *const _ as usize;
                if ca != cb {
                    ca.cmp(&cb)
                } else {
                    a.name.cmp(b.name)
                }
            });
            return all;
        }
        // category:name form
        if let Some(pos) = q.find(':') {
            let (cat, name) = q.split_at(pos);
            let name = &name[1..];
            let cat_norm = cat.to_lowercase();
            SLASH_COMMAND_SPECS
                .iter()
                .filter(|s| match (cat_norm.as_str(), &s.category) {
                    ("session", CommandCategory::Session)
                    | ("config", CommandCategory::Config)
                    | ("model", CommandCategory::Model)
                    | ("permissions", CommandCategory::Permissions)
                    | ("workspace", CommandCategory::Workspace)
                    | ("tools", CommandCategory::Tools)
                    | ("diagnostics", CommandCategory::Diagnostics)
                    | ("appearance", CommandCategory::Appearance)
                    | ("agent", CommandCategory::Agent)
                    | ("export", CommandCategory::Export) => {
                        s.name.eq_ignore_ascii_case(name)
                            || s.aliases.iter().any(|a| a.eq_ignore_ascii_case(name))
                    }
                    _ => false,
                })
                .collect()
        } else {
            // Fuzzy match against name + aliases + summary
            let mut matches: Vec<(&SlashCommandSpec, i64)> = SLASH_COMMAND_SPECS
                .iter()
                .filter_map(|spec| {
                    let candidate =
                        format!("{} {}{}", spec.name, spec.aliases.join(" "), spec.summary);
                    let m = SkimMatcherV2::default();
                    m.fuzzy_match(&candidate.to_lowercase(), &q.to_lowercase())
                        .map(|score| (spec, score))
                })
                .collect();
            matches.sort_by(|a, b| b.1.cmp(&a.1));
            matches.into_iter().map(|(s, _)| s).collect()
        }
    }

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
        let cmd = parts[0].to_lowercase();
        let arg = parts.get(1).copied();
        match cmd.as_str() {
            "help" | "?" => Some(SlashCommand::Help),
            "model" => Some(SlashCommand::Model(arg.map(|s| s.to_string()))),
            "clear" => Some(SlashCommand::Clear),
            "settings" | "config" => Some(SlashCommand::Settings),
            "quit" | "exit" | "q" => Some(SlashCommand::Quit),
            "version" | "v" => Some(SlashCommand::Version),
            "status" => Some(SlashCommand::Status),
            "mock" => Some(SlashCommand::Mock),
            "role" => Some(SlashCommand::Role(arg.map(|s| s.to_string()))),
            "provider" => Some(SlashCommand::Provider(arg.map(|s| s.to_string()))),
            "save" => Some(SlashCommand::Save),
            "theme" => Some(SlashCommand::Theme(arg.map(|s| s.to_string()))),
            "new" => Some(SlashCommand::New),
            "doctor" => Some(SlashCommand::Doctor),
            "session" => Some(SlashCommand::Session(arg.map(|s| s.to_string()))),
            "mcp" => Some(SlashCommand::Mcp(arg.map(|s| s.to_string()))),
            "preprompt" => Some(SlashCommand::Preprompt(arg.map(|s| s.to_string()))),
            "tokens" => Some(SlashCommand::Tokens),
            "cost" => Some(SlashCommand::Cost),
            "context" => Some(SlashCommand::Context),
            "time" => Some(SlashCommand::Time),
            "tools" => Some(SlashCommand::Tools),
            "permissions" | "perms" => Some(SlashCommand::Permissions(arg.map(|s| s.to_string()))),
            "readonly" => Some(SlashCommand::ReadOnly),
            "diff" => Some(SlashCommand::Diff),
            "git" => Some(SlashCommand::Git(arg.map(|s| s.to_string()))),
            "export" => Some(SlashCommand::Export(arg.map(|s| s.to_string()))),
            _ => Some(SlashCommand::Unknown(input.to_string())),
        }
    }

    pub fn name(&self) -> &str {
        match self {
            SlashCommand::Help => "help",
            SlashCommand::Model(_) => "model",
            SlashCommand::Clear => "clear",
            SlashCommand::Settings => "settings",
            SlashCommand::Quit => "quit",
            SlashCommand::Version => "version",
            SlashCommand::Status => "status",
            SlashCommand::Mock => "mock",
            SlashCommand::Role(_) => "role",
            SlashCommand::Provider(_) => "provider",
            SlashCommand::Save => "save",
            SlashCommand::Theme(_) => "theme",
            SlashCommand::Permissions(_) => "permissions",
            SlashCommand::ReadOnly => "readonly",
            SlashCommand::New => "new",
            SlashCommand::Doctor => "doctor",
            SlashCommand::Session(_) => "session",
            SlashCommand::Mcp(_) => "mcp",
            SlashCommand::Preprompt(_) => "preprompt",
            SlashCommand::Tokens => "tokens",
            SlashCommand::Cost => "cost",
            SlashCommand::Context => "context",
            SlashCommand::Time => "time",
            SlashCommand::Tools => "tools",
            SlashCommand::Diff => "diff",
            SlashCommand::Git(_) => "git",
            SlashCommand::Export(_) => "export",
            SlashCommand::Unknown(n) => n,
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            SlashCommand::Help => "Show available commands",
            SlashCommand::Model(_) => "Switch or view current model",
            SlashCommand::Clear => "Clear chat history",
            SlashCommand::Settings => "Open settings screen",
            SlashCommand::Quit => "Quit Fever Code",
            SlashCommand::Version => "Show version",
            SlashCommand::Status => "Show provider/model status",
            SlashCommand::Mock => "Toggle mock mode for local testing",
            SlashCommand::Role(_) => "Set or view current role",
            SlashCommand::Provider(_) => "Switch or view provider",
            SlashCommand::Save => "Save current session",
            SlashCommand::Theme(_) => "Switch or list themes",
            SlashCommand::Permissions(_) => "View or change permission mode",
            SlashCommand::ReadOnly => "Switch to read-only mode",
            SlashCommand::New => "Start new session",
            SlashCommand::Doctor => "Run diagnostics",
            SlashCommand::Session(_) => "List or manage sessions",
            SlashCommand::Mcp(_) => "Manage MCP servers",
            SlashCommand::Preprompt(_) => "Manage pre-prompt/system behavior",
            SlashCommand::Tokens => "Show token usage",
            SlashCommand::Cost => "Show estimated cost",
            SlashCommand::Context => "Show context window usage",
            SlashCommand::Time => "Show request timing",
            SlashCommand::Tools => "List available tools",
            SlashCommand::Diff => "Show git diff",
            SlashCommand::Git(_) => "Run git command",
            SlashCommand::Export(_) => "Export session to file",
            SlashCommand::Unknown(_) => "Unknown slash command",
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
            Some(SlashCommand::Model(Some("gpt-4o".to_string())))
        );
        assert_eq!(
            SlashCommand::parse("/model"),
            Some(SlashCommand::Model(None))
        );
    }

    #[test]
    fn test_parse_clear() {
        assert_eq!(SlashCommand::parse("/clear"), Some(SlashCommand::Clear));
    }

    #[test]
    fn test_parse_unknown() {
        assert_eq!(
            SlashCommand::parse("/xyz"),
            Some(SlashCommand::Unknown("xyz".to_string()))
        );
    }

    #[test]
    fn test_parse_no_slash() {
        assert_eq!(SlashCommand::parse("hello"), None);
    }

    #[test]
    fn test_name_and_description() {
        assert_eq!(SlashCommand::Help.name(), "help");
        assert_eq!(SlashCommand::Model(Some(String::new())).name(), "model");
        assert_eq!(SlashCommand::Clear.description(), "Clear chat history");
    }

    #[test]
    fn test_parse_all_known_commands() {
        // Basic known commands parse to the expected variants
        assert_eq!(SlashCommand::parse("/help"), Some(SlashCommand::Help));
        assert_eq!(SlashCommand::parse("/new"), Some(SlashCommand::New));
        assert_eq!(SlashCommand::parse("/save"), Some(SlashCommand::Save));
        assert_eq!(SlashCommand::parse("/status"), Some(SlashCommand::Status));
        assert_eq!(
            SlashCommand::parse("/export"),
            Some(SlashCommand::Export(None))
        );
        assert_eq!(SlashCommand::parse("/diff"), Some(SlashCommand::Diff));
        assert_eq!(
            SlashCommand::parse("/git status"),
            Some(SlashCommand::Git(Some("status".to_string())))
        );
        assert_eq!(
            SlashCommand::parse("/permissions read"),
            Some(SlashCommand::Permissions(Some("read".to_string())))
        );
        assert_eq!(
            SlashCommand::parse("/perms write"),
            Some(SlashCommand::Permissions(Some("write".to_string())))
        );
    }

    #[test]
    fn test_parse_aliases() {
        // Aliases
        assert_eq!(SlashCommand::parse("/exit"), Some(SlashCommand::Quit));
        assert_eq!(SlashCommand::parse("/config"), Some(SlashCommand::Settings));
        assert_eq!(SlashCommand::parse("/v"), Some(SlashCommand::Version));
    }

    #[test]
    fn test_find_specs_empty_query() {
        let specs = SlashCommand::find_specs("");
        assert!(!specs.is_empty());
        assert_eq!(specs.len(), SLASH_COMMAND_SPECS.len());
    }

    #[test]
    fn test_find_specs_fuzzy() {
        let results = SlashCommand::find_specs("model");
        assert!(!results.is_empty());
        assert_eq!(results[0].name, "model");
    }

    #[test]
    fn test_find_specs_category() {
        let cs = SlashCommand::category_specs(&CommandCategory::Workspace);
        // Should include diff and git
        let names: Vec<_> = cs.iter().map(|s| s.name).collect();
        assert!(names.contains(&"diff"));
        assert!(names.contains(&"git"));
    }

    #[test]
    fn test_find_by_name() {
        let s = SlashCommand::find_by_name("model");
        assert!(s.is_some());
        assert_eq!(s.unwrap().name, "model");
    }

    #[test]
    fn test_all_specs_have_unique_names() {
        let mut seen = std::collections::HashSet::<&'static str>::new();
        for spec in SLASH_COMMAND_SPECS {
            if seen.contains(spec.name) {
                panic!("Duplicate command spec name: {}", spec.name);
            }
            seen.insert(spec.name);
        }
    }
}
