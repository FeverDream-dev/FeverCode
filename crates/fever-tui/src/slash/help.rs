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
"#,
        glyphs::SECTION_LINE
    )
}
