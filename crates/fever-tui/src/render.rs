use crate::app::AppState;
use crate::components::status_bar::StatusBar;
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Clear, Paragraph},
};

pub fn render_frame(f: &mut Frame, state: &mut AppState) {
    let size = f.area();
    state.terminal_size = (size.width, size.height);

    let chunks = Layout::vertical([Constraint::Min(1), Constraint::Length(1)]).split(size);

    f.render_widget(
        Block::default().style(Style::default().bg(state.theme.bg())),
        size,
    );

    state.render_screen(f, chunks[0]);

    let mut sb = StatusBar::new();
    sb.provider = state.provider_name.clone();
    sb.model = state.model_name.clone();
    sb.workspace = state.workspace.clone();
    sb.streaming = state.streaming;
    sb.message_count = state.messages.len();
    sb.render(f, chunks[1], &state.theme);

    if state.show_command_palette {
        render_command_palette(f, size, state);
    }

    if state.show_help {
        render_help_overlay(f, size, state);
    }
}

fn render_command_palette(f: &mut Frame, area: Rect, state: &AppState) {
    let theme = &state.theme;

    let palette_width = 50.min(area.width.saturating_sub(4));
    let palette_height = 14.min(area.height.saturating_sub(4));
    let x = area.width.saturating_sub(palette_width) / 2;
    let y = area.height / 4;

    let palette_area = Rect {
        x,
        y,
        width: palette_width,
        height: palette_height,
    };

    f.render_widget(Clear, palette_area);

    let block = Block::default()
        .borders(ratatui::widgets::Borders::ALL)
        .border_style(theme.style_accent())
        .title(Span::styled(
            " ◈ Commands (Ctrl+K) ",
            theme.style_accent_bold(),
        ))
        .style(Style::default().bg(theme.bg_panel()));

    let inner = block.inner(palette_area);
    f.render_widget(block, palette_area);

    let query_display = if state.palette_query.is_empty() {
        "Type to search...".to_string()
    } else {
        state.palette_query.clone()
    };
    let query_style = if state.palette_query.is_empty() {
        theme.style_dimmed()
    } else {
        theme.style_accent()
    };

    let search = Paragraph::new(format!("> {}", query_display)).style(query_style);
    f.render_widget(search, Rect::new(inner.x, inner.y, inner.width, 1));

    let separator = Paragraph::new(Line::from(Span::styled(
        "\u{2500}".repeat(inner.width as usize),
        Style::default().fg(theme.fg_dimmed()),
    )));
    f.render_widget(separator, Rect::new(inner.x, inner.y + 1, inner.width, 1));

    let commands = state.palette_commands();
    let list_start_y = inner.y + 2;
    let max_visible = (inner.height.saturating_sub(2)) as usize;

    for (i, cmd) in commands.iter().take(max_visible).enumerate() {
        let cmd_area = Rect::new(inner.x, list_start_y + i as u16, inner.width, 1);

        let is_selected = i == state.palette_selection;
        let style = if is_selected {
            Style::default()
                .fg(theme.bg())
                .bg(theme.accent())
                .add_modifier(Modifier::BOLD)
        } else {
            theme.style_fg()
        };

        let line = Line::from(vec![
            Span::styled(format!("  /{}", cmd.name()), style),
            Span::styled(
                format!(
                    "  {}",
                    "\u{2500}".repeat(12usize.saturating_sub(cmd.name().len()))
                ),
                if is_selected {
                    Style::default().fg(theme.bg()).bg(theme.accent())
                } else {
                    Style::default().fg(theme.fg_dimmed())
                },
            ),
            Span::styled(format!("  {}", cmd.description()), style),
        ]);
        f.render_widget(Paragraph::new(line), cmd_area);
    }
}

fn render_help_overlay(f: &mut Frame, area: Rect, state: &AppState) {
    let theme = &state.theme;

    let overlay_width = 44.min(area.width.saturating_sub(4));
    let overlay_height = 20.min(area.height.saturating_sub(4));
    let x = (area.width.saturating_sub(overlay_width)) / 2;
    let y = (area.height.saturating_sub(overlay_height)) / 2;

    let overlay_area = Rect {
        x,
        y,
        width: overlay_width,
        height: overlay_height,
    };

    f.render_widget(Clear, overlay_area);

    let block = Block::default()
        .borders(ratatui::widgets::Borders::ALL)
        .border_style(theme.style_accent())
        .title(Span::styled(
            " Keyboard Shortcuts ",
            theme.style_accent_bold(),
        ))
        .style(Style::default().bg(theme.bg_panel()));

    let inner = block.inner(overlay_area);
    f.render_widget(block, overlay_area);

    let shortcuts = vec![
        ("Enter", "Send message"),
        ("Up / Down", "Recall input history"),
        ("PgUp / PgDn", "Scroll messages"),
        ("Home / End", "Jump to top / bottom"),
        ("Mouse wheel", "Scroll messages"),
        ("Esc", "Go back / Close"),
        ("?", "Toggle this help"),
        ("Ctrl+K", "Command palette"),
        ("Ctrl+B", "Toggle sidebar"),
        ("Ctrl+C", "Cancel / Quit"),
        ("/", "Start slash command"),
    ];

    for (i, (key, desc)) in shortcuts.iter().enumerate() {
        let row_y = inner.y + i as u16;
        if row_y >= inner.y + inner.height {
            break;
        }

        let line = Line::from(vec![
            Span::styled(
                format!("  {:<14}", key),
                Style::default()
                    .fg(theme.accent())
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(*desc, theme.style_fg()),
        ]);
        f.render_widget(
            Paragraph::new(line),
            Rect::new(inner.x, row_y, inner.width, 1),
        );
    }
}
