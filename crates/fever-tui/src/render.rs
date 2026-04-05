use crate::app::AppState;
use crate::components::status_bar::StatusBar;
use crate::util::glyphs;
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
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

        sb.git_branch = state.git_branch.clone();
        sb.permission_mode = state.permission_mode.label().to_string();
        sb.is_mock_mode = state.is_mock_mode;
        sb.session_id = state.session_id.clone();
        sb.streaming = state.streaming;
        sb.message_count = state.messages.len();
        sb.input_tokens = state.input_tokens;
        sb.output_tokens = state.output_tokens;
        sb.total_tokens = state.total_tokens;
        sb.estimated_cost = state.estimated_cost;
        sb.request_elapsed = state.request_elapsed;
        sb.show_tokens = state.show_tokens_in_status;
        sb.show_cost = state.show_cost_in_status;
        sb.show_elapsed = state.show_elapsed_in_status;
        sb.render(f, inner_chunks[1], &state.theme);
    } else {
        let chunks = Layout::vertical([Constraint::Min(1), Constraint::Length(1)]).split(size);

        state.render_screen(f, chunks[0]);

        let mut sb = StatusBar::new();
        sb.provider = state.provider_name.clone();
        sb.model = state.model_name.clone();
        sb.theme_name = state.theme.name.to_string();
        sb.workspace = state.workspace.clone();

        sb.git_branch = state.git_branch.clone();
        sb.permission_mode = state.permission_mode.label().to_string();
        sb.is_mock_mode = state.is_mock_mode;
        sb.session_id = state.session_id.clone();
        sb.streaming = state.streaming;
        sb.message_count = state.messages.len();
        sb.input_tokens = state.input_tokens;
        sb.output_tokens = state.output_tokens;
        sb.total_tokens = state.total_tokens;
        sb.estimated_cost = state.estimated_cost;
        sb.request_elapsed = state.request_elapsed;
        sb.show_tokens = state.show_tokens_in_status;
        sb.show_cost = state.show_cost_in_status;
        sb.show_elapsed = state.show_elapsed_in_status;
        sb.render(f, chunks[1], &state.theme);
    }

    if state.show_command_palette {
        render_command_palette(f, size, state);
    }

    if state.show_help {
        render_help_overlay(f, size, state);
    }

    if state.show_tool_panel {
        render_tool_panel(f, size, state);
    }

    if state.show_diff_panel {
        render_diff_panel(f, size, state);
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

    if state.show_tokens_in_status || state.show_cost_in_status || state.show_elapsed_in_status {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            glyphs::DIVIDER.repeat(inner.width as usize),
            Style::default().fg(theme.fg_dimmed()),
        )));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            " Telemetry",
            Style::default()
                .fg(theme.fg_dimmed())
                .add_modifier(Modifier::BOLD),
        )));
        if state.show_tokens_in_status {
            lines.push(Line::from(Span::styled(
                format!("  {} tok", state.total_tokens),
                Style::default().fg(theme.fg()),
            )));
        }
        if state.show_cost_in_status {
            lines.push(Line::from(Span::styled(
                format!("  ${:.4}", state.estimated_cost),
                Style::default().fg(theme.fg()),
            )));
        }
        if state.show_elapsed_in_status {
            if let Some(d) = state.request_elapsed {
                lines.push(Line::from(Span::styled(
                    format!("  {:.1}s", d.as_secs_f64()),
                    Style::default().fg(theme.fg()),
                )));
            }
        }
    }

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

    for (i, spec) in commands.iter().take(max_visible).enumerate() {
        let spec = *spec;
        // spec is &SlashCommandSpec
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

        // Build a single line with:
        // [  /name] [● if requires_provider] [category badge] [summary] [hint if any]
        let mut parts: Vec<Span> = Vec::new();
        parts.push(Span::styled(format!("  /{}", spec.name), style));

        if spec.requires_provider {
            parts.push(Span::styled(" ●", Style::default().fg(theme.accent())));
        }

        // Category badge
        let cat_badge = format!(" [{}]", format!("{:?}", spec.category).to_lowercase());
        parts.push(Span::styled(cat_badge, theme.style_fg()));

        // Summary (description)
        parts.push(Span::styled(format!("  {}", spec.summary), style));

        // Argument hint (dimmed)
        if let Some(hint) = spec.argument_hint {
            parts.push(Span::styled(format!("  {}", hint), theme.style_dimmed()));
        }

        let line = Line::from(parts);
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
        ("Ctrl+T", "Toggle tool panel"),
        ("Ctrl+D", "Toggle diff panel"),
        ("Ctrl+C", "Cancel / Quit"),
        ("/", "Start slash command"),
        ("S (home)", "Open settings"),
        ("Tab (settings)", "Next settings tab"),
        ("Up/Down (slash)", "Navigate slash menu"),
        ("Enter (slash)", "Execute slash command"),
        ("/tokens", "Show token usage"),
        ("/cost", "Show estimated cost"),
        ("/mcp", "Manage MCP servers"),
        ("/status", "Show full status"),
        ("/time", "Show request timing"),
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

fn render_tool_panel(f: &mut Frame, area: Rect, state: &AppState) {
    let theme = &state.theme;
    let panel_w = state.panel_width.min(area.width.saturating_sub(20));
    let panel_h = area.height.saturating_sub(2);
    let panel_x = area.width.saturating_sub(panel_w);
    let panel_y = 0;

    let panel_area = Rect::new(panel_x, panel_y, panel_w, panel_h);

    let block = Block::default()
        .borders(Borders::LEFT)
        .border_style(theme.style_border(false))
        .title(Span::styled(" Tools ", theme.style_accent_bold()))
        .style(Style::default().bg(theme.bg_panel()));

    let inner = block.inner(panel_area);
    f.render_widget(block, panel_area);

    let tools = [
        ("shell", "Execute shell commands"),
        ("read_file", "Read file contents"),
        ("write_file", "Write to files"),
        ("list_directory", "List directory contents"),
        ("grep", "Search file contents"),
        ("git_status", "Show git status"),
        ("git_diff", "Show git diff"),
        ("git_log", "Show git log"),
    ];

    let mut lines: Vec<Line> = Vec::new();
    for (name, desc) in &tools {
        lines.push(Line::from(vec![
            Span::styled(format!(" {} ", name), theme.style_accent()),
            Span::styled(*desc, Style::default().fg(theme.fg_dimmed())),
        ]));
    }

    if state.tool_calls.is_empty() {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            glyphs::DIVIDER.repeat(inner.width as usize),
            Style::default().fg(theme.fg_dimmed()),
        )));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            " No tool activity yet",
            Style::default().fg(theme.fg_dimmed()),
        )));
    } else {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            glyphs::DIVIDER.repeat(inner.width as usize),
            Style::default().fg(theme.fg_dimmed()),
        )));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            format!(" Activity ({})", state.tool_calls.len()),
            theme.style_accent_bold(),
        )));
        for tc in &state.tool_calls {
            let icon = match tc.status {
                crate::components::tool_card::ToolStatus::Running => glyphs::ACTIVE,
                crate::components::tool_card::ToolStatus::Completed => glyphs::CHECK,
                crate::components::tool_card::ToolStatus::Failed => glyphs::CROSS,
            };
            let label = if tc.tool_name.len() > 18 {
                format!("{}..", &tc.tool_name[..18])
            } else {
                tc.tool_name.clone()
            };
            lines.push(Line::from(vec![
                Span::styled(format!(" {} ", icon), theme.style_fg()),
                Span::styled(label, Style::default().fg(theme.fg())),
            ]));
        }
    }

    let panel = Paragraph::new(lines)
        .wrap(Wrap { trim: false })
        .style(Style::default().bg(theme.bg_panel()));
    f.render_widget(panel, inner);
}

fn render_diff_panel(f: &mut Frame, area: Rect, state: &AppState) {
    let theme = &state.theme;
    let panel_w = state.panel_width.min(area.width.saturating_sub(20));
    let panel_h = area.height.saturating_sub(2);
    let panel_x = area.width.saturating_sub(panel_w);
    let panel_y = 0;

    let panel_area = Rect::new(panel_x, panel_y, panel_w, panel_h);

    let block = Block::default()
        .borders(Borders::LEFT)
        .border_style(theme.style_border(false))
        .title(Span::styled(" Diff ", theme.style_accent_bold()))
        .style(Style::default().bg(theme.bg_panel()));

    let inner = block.inner(panel_area);
    f.render_widget(block, panel_area);

    let mut lines: Vec<Line> = Vec::new();

    if state.diff_content.is_empty() {
        lines.push(Line::from(Span::styled(
            " Loading...",
            Style::default().fg(theme.fg_dimmed()),
        )));
    } else {
        for line in &state.diff_content {
            if line.starts_with(" ") {
                lines.push(Line::from(Span::styled(
                    line.clone(),
                    Style::default().fg(theme.fg()),
                )));
            } else if line.contains("failed") || line.contains("not available") {
                lines.push(Line::from(Span::styled(line.clone(), theme.style_error())));
            } else {
                lines.push(Line::from(Span::styled(line.clone(), theme.style_accent())));
            }
        }
    }

    let panel = Paragraph::new(lines)
        .wrap(Wrap { trim: false })
        .style(Style::default().bg(theme.bg_panel()));
    f.render_widget(panel, inner);
}
