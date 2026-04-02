use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

use crate::app::AppState;
use crate::util::{glyphs, text};

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
        glyphs::ornament(),
        glyphs::DIVIDER.repeat(mid),
        glyphs::ornament(),
        glyphs::DIVIDER.repeat(right),
    )
}

fn load_recent_sessions() -> Vec<(String, String)> {
    let sessions_dir = dirs::data_dir()
        .map(|d| d.join("fevercode").join("sessions"))
        .unwrap_or_else(|| std::path::PathBuf::from(".fevercode/sessions"));

    if !sessions_dir.exists() {
        return Vec::new();
    }

    let mut sessions: Vec<_> = std::fs::read_dir(&sessions_dir)
        .ok()
        .map(|entries| {
            entries
                .filter_map(|e| e.ok())
                .filter_map(|e| {
                    let name = e.file_name().to_string_lossy().to_string();
                    if name.ends_with(".json") {
                        let stem = name.trim_end_matches(".json").to_string();
                        let meta = e.metadata().ok()?;
                        let modified = meta.modified().ok()?;
                        let time = chrono::DateTime::<chrono::Local>::from(modified);
                        Some((stem, time.format("%Y-%m-%d %H:%M").to_string()))
                    } else {
                        None
                    }
                })
                .collect()
        })
        .unwrap_or_default();

    sessions.sort_by(|a, b| b.1.cmp(&a.1));
    sessions.truncate(5);
    sessions
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
        text::center_text(glyphs::logo_glyph(), w),
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
            Span::styled("  |  Session: ", Style::default().fg(theme.fg_dimmed())),
            Span::styled(&state.session_id, Style::default().fg(theme.fg())),
        ]));
    }

    lines.push(Line::from(""));

    let recent = load_recent_sessions();
    if !recent.is_empty() {
        if !narrow {
            let div_w = w.saturating_sub(4);
            lines.push(Line::from(Span::styled(
                format!("  {}", sacred_divider(div_w)),
                Style::default().fg(theme.accent()),
            )));
        }
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "  Recent Sessions",
            Style::default()
                .fg(theme.fg_dimmed())
                .add_modifier(Modifier::BOLD),
        )));

        for (id, time) in &recent {
            lines.push(Line::from(vec![
                Span::styled("    ", Style::default()),
                Span::styled(
                    text::truncate_str(id, val_w.saturating_sub(20)),
                    Style::default().fg(theme.fg()),
                ),
                Span::styled(
                    format!("  {}", time),
                    Style::default().fg(theme.fg_dimmed()),
                ),
            ]));
        }
        lines.push(Line::from(""));
    }

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

    lines.push(Line::from(vec![
        Span::styled("  [Ctrl+K] ", Style::default().fg(theme.accent())),
        Span::styled("palette    ", Style::default().fg(theme.fg())),
        Span::styled("[Ctrl+B] ", Style::default().fg(theme.accent())),
        Span::styled("sidebar    ", Style::default().fg(theme.fg())),
        Span::styled("[?] ", Style::default().fg(theme.accent())),
        Span::styled("help", Style::default().fg(theme.fg())),
    ]));

    if !narrow {
        lines.push(Line::from(""));
    }

    let paragraph = Paragraph::new(lines).style(Style::default().bg(theme.bg()));
    f.render_widget(paragraph, inner);
}
