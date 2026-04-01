use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

use crate::app::AppState;

const TABS: &[&str] = &["Providers", "Models", "Behavior", "Theme"];

pub fn render(f: &mut Frame, area: Rect, state: &mut AppState) {
    if area.width < 20 || area.height < 10 {
        return;
    }

    let theme = &state.theme;

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme.style_border(false))
        .title(Span::styled(" Settings ", theme.style_title(true)));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let mut lines: Vec<Line> = Vec::new();

    let tab_strs: Vec<Span> = TABS
        .iter()
        .enumerate()
        .map(|(i, name)| {
            if i == state.settings_tab {
                Span::styled(
                    format!(" {} ", name),
                    Style::default()
                        .fg(theme.accent())
                        .add_modifier(Modifier::BOLD),
                )
            } else {
                Span::styled(
                    format!(" {} ", name),
                    Style::default().fg(theme.fg_dimmed()),
                )
            }
        })
        .collect();
    lines.push(Line::from(tab_strs));
    lines.push(Line::from(Span::styled(
        "\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}",
        Style::default().fg(theme.accent()),
    )));
    lines.push(Line::from(""));

    match state.settings_tab {
        0 => {
            lines.push(Line::from(Span::styled(
                "  Configured Providers",
                Style::default().fg(theme.fg()).add_modifier(Modifier::BOLD),
            )));
            lines.push(Line::from(""));
            lines.push(Line::from(format!("  Provider: {}", state.provider_name)));
            lines.push(Line::from(format!("  Model: {}", state.model_name)));
        }
        1 => {
            lines.push(Line::from(Span::styled(
                "  Model Selection",
                Style::default().fg(theme.fg()).add_modifier(Modifier::BOLD),
            )));
            lines.push(Line::from(""));
            lines.push(Line::from(format!("  Current: {}", state.model_name)));
        }
        2 => {
            lines.push(Line::from(Span::styled(
                "  Behavior",
                Style::default().fg(theme.fg()).add_modifier(Modifier::BOLD),
            )));
            lines.push(Line::from(""));
            lines.push(Line::from("  (coming soon)"));
        }
        3 => {
            lines.push(Line::from(Span::styled(
                "  Theme",
                Style::default().fg(theme.fg()).add_modifier(Modifier::BOLD),
            )));
            lines.push(Line::from(""));
            lines.push(Line::from(format!("  Active: {}", theme.name)));
        }
        _ => {}
    }

    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("  [Esc] ", Style::default().fg(theme.accent())),
        Span::styled("back    ", Style::default().fg(theme.fg())),
        Span::styled("[Tab] ", Style::default().fg(theme.accent())),
        Span::styled("next section", Style::default().fg(theme.fg())),
    ]));

    let paragraph = Paragraph::new(lines).style(Style::default().bg(theme.bg()).fg(theme.fg()));
    f.render_widget(paragraph, inner);
}
