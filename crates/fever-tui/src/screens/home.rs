use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::AppState;
use crate::util::{glyphs, text};

pub fn render(f: &mut Frame, area: Rect, state: &mut AppState) {
    if area.width < 20 || area.height < 10 {
        return;
    }

    let theme = &state.theme;
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme.style_border(false))
        .style(Style::default().bg(theme.bg()));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let w = inner.width as usize;
    let mut lines: Vec<Line> = Vec::new();

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        text::center_text(glyphs::MARK, w),
        Style::default().fg(theme.accent()),
    )));
    lines.push(Line::from(Span::styled(
        text::center_text("F E V E R", w),
        Style::default().fg(theme.fg()).add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(Span::styled(
        text::center_text("cold sacred code", w),
        Style::default().fg(theme.fg_dimmed()),
    )));
    lines.push(Line::from(""));

    lines.push(Line::from(vec![
        Span::styled("  Provider: ", Style::default().fg(theme.fg_dimmed())),
        Span::styled(
            format!("{} ", state.provider_name),
            Style::default().fg(theme.fg()),
        ),
        Span::styled("\u{00b7} ", Style::default().fg(theme.fg_dimmed())),
        Span::styled(
            state.model_name.clone(),
            Style::default().fg(theme.fg_secondary()),
        ),
    ]));

    lines.push(Line::from(vec![
        Span::styled("  Workspace: ", Style::default().fg(theme.fg_dimmed())),
        Span::styled(state.workspace.clone(), Style::default().fg(theme.fg())),
    ]));

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        format!("  {}", text::stepped_divider(w.saturating_sub(4))),
        Style::default().fg(theme.fg_dimmed()),
    )));
    lines.push(Line::from(""));

    lines.push(Line::from(vec![
        Span::styled("  [Enter] ", Style::default().fg(theme.accent())),
        Span::styled("chat    ", Style::default().fg(theme.fg())),
        Span::styled("[/] ", Style::default().fg(theme.accent())),
        Span::styled("commands    ", Style::default().fg(theme.fg())),
        Span::styled("[S] ", Style::default().fg(theme.accent())),
        Span::styled("settings", Style::default().fg(theme.fg())),
    ]));

    let paragraph = Paragraph::new(lines).style(Style::default().bg(theme.bg()));
    f.render_widget(paragraph, inner);
}
