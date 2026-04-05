use crate::slash::commands::{CommandCategory, SLASH_COMMAND_SPECS};

// Dynamic help text generated from the centralized SLASH_COMMAND_SPECS table.
pub fn help_text() -> String {
    use std::collections::BTreeMap;

    struct Entry {
        name: &'static str,
        summary: &'static str,
        aliases: &'static [&'static str],
        arg: Option<&'static str>,
        category: CommandCategory,
    }

    let mut by_cat: BTreeMap<&'static str, Vec<Entry>> = BTreeMap::new();
    for spec in SLASH_COMMAND_SPECS {
        let entry = Entry {
            name: spec.name,
            summary: spec.summary,
            aliases: spec.aliases,
            arg: spec.argument_hint,
            category: spec.category.clone(),
        };
        let header = format!("{:?}", entry.category);
        by_cat
            .entry(Box::leak(header.into_boxed_str()))
            .or_default()
            .push(entry);
    }

    let mut out = String::new();
    out.push_str("Fever Commands\n\n");
    for (cat, items) in by_cat.iter() {
        out.push_str(&format!("{} commands:\n", cat));
        for it in items {
            let alias = if it.aliases.is_empty() {
                String::new()
            } else {
                format!(" (aliases: {})", it.aliases.join(", "))
            };
            let arg = it.arg.map(|a| format!(" {}", a)).unwrap_or_default();
            out.push_str(&format!(
                "  /{name}{arg}{aliases} - {summary}\n",
                name = it.name,
                arg = arg,
                aliases = alias,
                summary = it.summary
            ));
        }
        out.push('\n');
    }
    out.push_str("  /help, /?          Show this help\n");
    out.push_str("  /model <name>       Switch model\n");
    out.push_str("  /role <name>        Switch role\n");
    out.push_str("  /provider <name>    Switch provider\n");
    out.push_str("  /theme <name>       Switch theme (no arg = list)\n");
    out.push_str("  /new                Start new session\n");
    out.push_str("  /doctor             Run health diagnostics\n");
    out.push_str("  /save               Save current session\n");
    out.push_str("  /clear              Clear conversation\n");
    out.push_str("  /settings           Open settings\n");
    out.push_str("  /status             Show status\n");
    out.push_str("  /version            Show version\n");
    out.push_str("  /quit, /q           Quit Fever\n");
    out.push_str("  /mcp [name]        Manage MCP servers\n");
    out.push_str("  /preprompt [mode]   Manage pre-prompt\n");
    out.push_str("  /tokens            Show token usage\n");
    out.push_str("  /cost              Show estimated cost\n");
    out.push_str("  /context           Show context window usage\n");
    out.push_str("  /time              Show request timing\n");
    out.push_str("  /tools             List available tools\n");
    out
}
