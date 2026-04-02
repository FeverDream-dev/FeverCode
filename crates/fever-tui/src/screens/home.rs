use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::AppState;
use crate::util::{glyphs, text};

/// Sacred geometry divider: `─── ◆ ───...─── ◆ ───`
///
/// Places diamond markers (ACCENT) at the 1/4 and 3/4 positions.
fn sacred_divider(width: usize) -> String {
    if width < 10 {
        return glyphs::DIVIDER.repeat(width);
    }
    let quarter = width / 4;
    let left = quarter;
    let right = quarter;
    let mid = width.saturating_sub(left + right + 2);

    format!(
        "{}{}{}{}{}",
        glyphs::DIVIDER.repeat(left),
        glyphs::ACCENT,
        glyphs::DIVIDER.repeat(mid),
        glyphs::ACCENT,
        glyphs::DIVIDER.repeat(right),
    )
}

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
    let narrow = w < 40;
    let mut lines: Vec<Line> = Vec::new();

    if !narrow {
        lines.push(Line::from(""));
    }
    lines.push(Line::from(Span::styled(
        text::center_text(glyphs::EYE_OF_HORUS, w),
        Style::default().fg(theme.accent()),
    )));
    if !narrow {
        lines.push(Line::from(""));
    }

    lines.push(Line::from(Span::styled(
        text::center_text("F E V E R   C O D E", w),
        Style::default().fg(theme.fg()).add_modifier(Modifier::BOLD),
    )));
    if !narrow {
        lines.push(Line::from(""));
    }

    lines.push(Line::from(Span::styled(
        text::center_text("code like fever, ship like dream", w),
        Style::default()
            .fg(theme.fg_dimmed())
            .add_modifier(Modifier::ITALIC),
    )));
    lines.push(Line::from(""));

    if !narrow {
        let div_w = w.saturating_sub(4);
        lines.push(Line::from(Span::styled(
            format!("  {}", sacred_divider(div_w)),
            Style::default().fg(theme.accent()),
        )));
        lines.push(Line::from(""));
    }

    let val_w = w.saturating_sub(14);

    lines.push(Line::from(vec![
        Span::styled("  Provider: ", Style::default().fg(theme.fg_dimmed())),
        Span::styled(
            text::truncate_str(&format!("{} ", state.provider_name), val_w),
            Style::default().fg(theme.fg()),
        ),
        Span::styled("\u{00b7} ", Style::default().fg(theme.fg_dimmed())),
        Span::styled(
            text::truncate_str(&state.model_name, val_w),
            Style::default().fg(theme.fg_secondary()),
        ),
    ]));

    lines.push(Line::from(vec![
        Span::styled("  Workspace: ", Style::default().fg(theme.fg_dimmed())),
        Span::styled(
            text::truncate_str(&state.workspace, val_w),
            Style::default().fg(theme.fg()),
        ),
    ]));

    if !narrow {
        lines.push(Line::from(vec![
            Span::styled("  Theme: ", Style::default().fg(theme.fg_dimmed())),
            Span::styled(theme.name, Style::default().fg(theme.fg_secondary())),
        ]));
    }

    lines.push(Line::from(""));

    if !narrow {
        let div_w = w.saturating_sub(4);
        lines.push(Line::from(Span::styled(
            format!("  {}", sacred_divider(div_w)),
            Style::default().fg(theme.accent()),
        )));
        lines.push(Line::from(""));
    }

    lines.push(Line::from(vec![
        Span::styled("  [Enter] ", Style::default().fg(theme.accent())),
        Span::styled("chat    ", Style::default().fg(theme.fg())),
        Span::styled("[/] ", Style::default().fg(theme.accent())),
        Span::styled("commands    ", Style::default().fg(theme.fg())),
        Span::styled("[S] ", Style::default().fg(theme.accent())),
        Span::styled("settings", Style::default().fg(theme.fg())),
    ]));

    if !narrow {
        lines.push(Line::from(""));
    }

    let paragraph = Paragraph::new(lines).style(Style::default().bg(theme.bg()));
    f.render_widget(paragraph, inner);
}
