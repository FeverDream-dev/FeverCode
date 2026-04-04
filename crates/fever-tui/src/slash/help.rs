use crate::util::glyphs;

pub fn help_text() -> String {
    format!(
        r#"
Fever Commands
{}

  /help, /?          Show this help
  /model <name>      Switch model
  /role <name>       Switch role
  /provider <name>   Switch provider
  /theme <name>      Switch theme (no arg = list)
  /new               Start new session
  /doctor            Run health diagnostics
  /save              Save current session
  /clear             Clear conversation
  /settings          Open settings
  /status            Show status
  /version           Show version
  /quit, /q          Quit Fever
  /mcp [name]       Manage MCP servers
  /preprompt [mode] Manage pre-prompt
  /tokens           Show token usage
  /cost             Show estimated cost
  /context          Show context window usage
  /time             Show request timing
  /tools            List available tools
"#,
        glyphs::SECTION_LINE
    )
}
