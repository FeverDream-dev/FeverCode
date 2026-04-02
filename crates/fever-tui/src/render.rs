use crate::app::AppState;
use crate::components::status_bar::StatusBar;
use crate::util::glyphs;
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

pub fn render_frame(f: &mut Frame, state: &mut AppState) {
    let size = f.area();
    state.terminal_size = (size.width, size.height);

    f.render_widget(
        Block::default().style(Style::default().bg(state.theme.bg())),
        size,
    );

    if state.show_sidebar {
        let sidebar_width = 28.min(size.width.saturating_sub(40));
        let chunks =
            Layout::horizontal([Constraint::Length(sidebar_width), Constraint::Min(1)]).split(size);

        render_sidebar(f, chunks[0], state);

        let inner_chunks =
            Layout::vertical([Constraint::Min(1), Constraint::Length(1)]).split(chunks[1]);

        state.render_screen(f, inner_chunks[0]);

        let mut sb = StatusBar::new();
        sb.provider = state.provider_name.clone();
        sb.model = state.model_name.clone();
        sb.theme_name = state.theme.name.to_string();
        sb.workspace = state.workspace.clone();
        sb.streaming = state.streaming;
        sb.message_count = state.messages.len();
        sb.render(f, inner_chunks[1], &state.theme);
    } else {
        let chunks = Layout::vertical([Constraint::Min(1), Constraint::Length(1)]).split(size);

        state.render_screen(f, chunks[0]);

        let mut sb = StatusBar::new();
        sb.provider = state.provider_name.clone();
        sb.model = state.model_name.clone();
        sb.theme_name = state.theme.name.to_string();
        sb.workspace = state.workspace.clone();
        sb.streaming = state.streaming;
        sb.message_count = state.messages.len();
        sb.render(f, chunks[1], &state.theme);
    }

    if state.show_command_palette {
        render_command_palette(f, size, state);
    }

    if state.show_help {
        render_help_overlay(f, size, state);
    }

    if let Some(ref text) = state.notification {
        render_notification(f, size, state, text);
    }
}

fn render_notification(f: &mut Frame, area: Rect, state: &AppState, text: &str) {
    let theme = &state.theme;
    let notif_width = (text.len() as u16 + 4).min(area.width.saturating_sub(4));
    let x = area.width.saturating_sub(notif_width) / 2;
    let y = area.height.saturating_sub(3);

    let notif_area = Rect {
        x,
        y,
        width: notif_width,
        height: 1,
    };

    let fade = if state.notification_tick > 20 {
        Style::default().fg(theme.accent()).bg(theme.bg())
    } else {
        Style::default()
            .fg(theme.bg())
            .bg(theme.accent())
            .add_modifier(Modifier::BOLD)
    };

    let line = Line::from(Span::styled(format!(" {} ", text), fade));
    let para = Paragraph::new(line);
    f.render_widget(para, notif_area);
}

fn render_sidebar(f: &mut Frame, area: Rect, state: &AppState) {
    let theme = &state.theme;

    let block = Block::default()
        .borders(Borders::RIGHT)
        .border_style(theme.style_border(false))
        .title(Span::styled(
            " Navigation ",
            Style::default().fg(theme.fg_dimmed()),
        ))
        .style(Style::default().bg(theme.bg_panel()));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let mut lines: Vec<Line> = Vec::new();

    lines.push(Line::from(Span::styled(
        format!(" {} FEVER", glyphs::MARK),
        theme.style_accent_bold(),
    )));
    lines.push(Line::from(""));

    let nav_items: Vec<(&str, &str, bool)> = vec![
        (
            "Home",
            "H",
            matches!(state.screen, crate::event::Screen::Home),
        ),
        (
            "Chat",
            "C",
            matches!(state.screen, crate::event::Screen::Chat),
        ),
        (
            "Settings",
            "S",
            matches!(state.screen, crate::event::Screen::Settings),
        ),
    ];

    for (name, shortcut, active) in &nav_items {
        let style = if *active {
            Style::default()
                .fg(theme.accent())
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(theme.fg())
        };
        lines.push(Line::from(Span::styled(
            format!(" [{}] {}", shortcut, name),
            style,
        )));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        glyphs::DIVIDER.repeat(inner.width as usize),
        Style::default().fg(theme.fg_dimmed()),
    )));
    lines.push(Line::from(""));

    lines.push(Line::from(Span::styled(
        " Provider",
        Style::default()
            .fg(theme.fg_dimmed())
            .add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(Span::styled(
        format!("  {} ", state.provider_name),
        Style::default().fg(theme.fg()),
    )));
    lines.push(Line::from(Span::styled(
        format!("  {} ", state.model_name),
        Style::default().fg(theme.fg_secondary()),
    )));
    lines.push(Line::from(""));

    lines.push(Line::from(Span::styled(
        " Theme",
        Style::default()
            .fg(theme.fg_dimmed())
            .add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(Span::styled(
        format!("  {} ", state.theme.name),
        Style::default().fg(theme.fg_secondary()),
    )));
    lines.push(Line::from(""));

    lines.push(Line::from(Span::styled(
        glyphs::DIVIDER.repeat(inner.width as usize),
        Style::default().fg(theme.fg_dimmed()),
    )));
    lines.push(Line::from(""));

    lines.push(Line::from(Span::styled(
        " Session",
        Style::default()
            .fg(theme.fg_dimmed())
            .add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(Span::styled(
        format!("  {} msg", state.messages.len()),
        Style::default().fg(theme.fg()),
    )));

    if state.streaming {
        lines.push(Line::from(Span::styled(
            format!("  {} active", glyphs::ACTIVE),
            Style::default().fg(theme.accent()),
        )));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        glyphs::DIVIDER.repeat(inner.width as usize),
        Style::default().fg(theme.fg_dimmed()),
    )));
    lines.push(Line::from(""));

    let quick_cmds = vec![
        ("/model", "Switch model"),
        ("/provider", "Switch provider"),
        ("/theme", "Switch theme"),
        ("/new", "New session"),
        ("/clear", "Clear chat"),
        ("/save", "Save session"),
        ("/session", "List sessions"),
        ("/doctor", "Diagnostics"),
        ("/help", "All commands"),
    ];

    for (cmd, desc) in &quick_cmds {
        lines.push(Line::from(vec![
            Span::styled(format!(" {} ", cmd), theme.style_accent()),
            Span::styled(*desc, Style::default().fg(theme.fg_dimmed())),
        ]));
    }

    let sidebar = Paragraph::new(lines)
        .wrap(Wrap { trim: false })
        .style(Style::default().bg(theme.bg_panel()));
    f.render_widget(sidebar, inner);
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
        .borders(Borders::ALL)
        .border_style(theme.style_accent())
        .title(Span::styled(
            " Commands (Ctrl+K) ",
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
        glyphs::DIVIDER.repeat(inner.width as usize),
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
                    glyphs::DIVIDER.repeat(12usize.saturating_sub(cmd.name().len()))
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
    let overlay_height = 22.min(area.height.saturating_sub(4));
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
        .borders(Borders::ALL)
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
        ("Mouse click", "Interact with UI"),
        ("Esc", "Go back / Close"),
        ("?", "Toggle this help"),
        ("Ctrl+K", "Command palette"),
        ("Ctrl+B", "Toggle sidebar"),
        ("Ctrl+C", "Cancel / Quit"),
        ("/", "Start slash command"),
        ("S (home)", "Open settings"),
        ("Tab (settings)", "Next settings tab"),
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
