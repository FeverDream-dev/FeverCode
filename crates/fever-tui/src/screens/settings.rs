use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::AppState;
use crate::theme::Theme;
use crate::util::glyphs;

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

            let all_themes = Theme::list_all();
            let total = all_themes.len();
            for (i, t) in all_themes.iter().enumerate() {
                let is_active = t.name == theme.name;
                let is_cursor = i == state.settings_theme_cursor;

                let marker = if is_cursor && is_active {
                    glyphs::ACTIVE
                } else if is_cursor {
                    "▶"
                } else if is_active {
                    glyphs::ACTIVE
                } else {
                    glyphs::INACTIVE
                };

                let style = if is_active {
                    Style::default()
                        .fg(theme.accent())
                        .add_modifier(Modifier::BOLD)
                } else if is_cursor {
                    Style::default().fg(theme.fg())
                } else {
                    Style::default().fg(theme.fg_dimmed())
                };

                lines.push(Line::from(Span::styled(
                    format!("  {} {}", marker, t.name),
                    style,
                )));
            }
            if state.settings_theme_cursor >= total {
                state.settings_theme_cursor = total.saturating_sub(1);
            }
        }
        _ => {}
    }

    lines.push(Line::from(""));
    if state.settings_tab == 3 {
        lines.push(Line::from(vec![
            Span::styled("  [Esc] ", Style::default().fg(theme.accent())),
            Span::styled("back    ", Style::default().fg(theme.fg())),
            Span::styled("[↑↓] ", Style::default().fg(theme.accent())),
            Span::styled("navigate    ", Style::default().fg(theme.fg())),
            Span::styled("[Enter] ", Style::default().fg(theme.accent())),
            Span::styled("apply", Style::default().fg(theme.fg())),
        ]));
    } else {
        lines.push(Line::from(vec![
            Span::styled("  [Esc] ", Style::default().fg(theme.accent())),
            Span::styled("back    ", Style::default().fg(theme.fg())),
            Span::styled("[Tab] ", Style::default().fg(theme.accent())),
            Span::styled("next section", Style::default().fg(theme.fg())),
        ]));
    }

    let paragraph = Paragraph::new(lines).style(Style::default().bg(theme.bg()).fg(theme.fg()));
    f.render_widget(paragraph, inner);
}
