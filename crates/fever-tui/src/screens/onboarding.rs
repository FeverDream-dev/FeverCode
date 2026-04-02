use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

use crate::app::AppState;
use crate::util::{glyphs, text};

const PROVIDERS: &[&str] = &[
    "OpenAI (GPT-4o, o1, o3)",
    "Anthropic (Claude)",
    "Google (Gemini)",
    "Ollama (local models)",
    "OpenRouter (multi-provider)",
    "Skip \u{2014} configure later",
];

pub fn render(f: &mut Frame, area: Rect, state: &mut AppState) {
    if area.width < 30 || area.height < 15 {
        return;
    }

    let theme = &state.theme;
    let w = area.width as usize;

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme.style_border(false));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let mut lines: Vec<Line> = Vec::new();

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        text::center_text(&format!("{}  F E V E R", glyphs::MARK), w),
        Style::default().fg(theme.accent()),
    )));
    lines.push(Line::from(Span::styled(
        text::center_text("code like fever, ship like dream", w),
        Style::default().fg(theme.fg_dimmed()),
    )));
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        text::center_text("Welcome. Configure your first provider.", w),
        Style::default().fg(theme.fg()),
    )));
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        format!("  {}", text::stepped_divider(w.saturating_sub(4))),
        Style::default().fg(theme.fg_dimmed()),
    )));
    lines.push(Line::from(""));

    for (i, provider) in PROVIDERS.iter().enumerate() {
        let selected = i == state.onboarding_selection;
        let marker = if selected { glyphs::ACTIVE } else { " " };
        let style = if selected {
            Style::default()
                .fg(theme.accent())
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(theme.fg())
        };
        lines.push(Line::from(Span::styled(
            format!("  {} [{}] {}", marker, i + 1, provider),
            style,
        )));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("  [Enter] ", Style::default().fg(theme.accent())),
        Span::styled("confirm    ", Style::default().fg(theme.fg())),
        Span::styled("[Esc] ", Style::default().fg(theme.accent())),
        Span::styled("back", Style::default().fg(theme.fg())),
    ]));

    let paragraph = Paragraph::new(lines).style(Style::default().bg(theme.bg()));
    f.render_widget(paragraph, inner);
}
