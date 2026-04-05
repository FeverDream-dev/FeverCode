use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

use crate::app::AppState;
use crate::app::{KNOWN_PROVIDERS, known_models_for_provider};
use crate::theme::Theme;
use crate::util::glyphs;

const TABS: &[&str] = &[
    "Providers",
    "Models",
    "Behavior",
    "Theme",
    "MCP",
    "PrePrompt",
    "Telemetry",
    "Advanced",
];

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
        glyphs::SECTION_LINE.repeat(4),
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

            for (i, provider) in KNOWN_PROVIDERS.iter().enumerate() {
                let is_active = *provider == state.provider_name;
                let is_cursor = i == state.settings_provider_cursor;

                let marker = if is_cursor && is_active {
                    glyphs::ACTIVE
                } else if is_cursor {
                    "\u{25b6}"
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

                let model_hint = if is_active {
                    format!("  {}", state.model_name)
                } else {
                    String::new()
                };

                lines.push(Line::from(Span::styled(
                    format!("  {} {}{}", marker, provider, model_hint),
                    style,
                )));
            }

            if state.settings_provider_cursor >= KNOWN_PROVIDERS.len() {
                state.settings_provider_cursor = KNOWN_PROVIDERS.len().saturating_sub(1);
            }
        }
        1 => {
            lines.push(Line::from(Span::styled(
                "  Model Selection",
                Style::default().fg(theme.fg()).add_modifier(Modifier::BOLD),
            )));
            lines.push(Line::from(Span::styled(
                format!("  Provider: {}", state.provider_name),
                Style::default().fg(theme.fg_dimmed()),
            )));
            lines.push(Line::from(""));

            let models = known_models_for_provider(&state.provider_name);
            for (i, model) in models.iter().enumerate() {
                let is_active = *model == state.model_name;
                let is_cursor = i == state.settings_model_cursor;

                let marker = if is_cursor && is_active {
                    glyphs::ACTIVE
                } else if is_cursor {
                    "\u{25b6}"
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
                    format!("  {} {}", marker, model),
                    style,
                )));
            }

            if state.settings_model_cursor >= models.len() {
                state.settings_model_cursor = models.len().saturating_sub(1);
            }
        }
        2 => {
            lines.push(Line::from(Span::styled(
                "  Behavior",
                Style::default().fg(theme.fg()).add_modifier(Modifier::BOLD),
            )));
            lines.push(Line::from(""));

            let items = [
                format!(
                    "  {} Auto-scroll  {}",
                    if state.settings_behavior_cursor == 0 {
                        glyphs::ACTIVE
                    } else {
                        " "
                    },
                    if state.auto_scroll { "on" } else { "off" }
                ),
                format!(
                    "  {} Show thinking  {}",
                    if state.settings_behavior_cursor == 1 {
                        glyphs::ACTIVE
                    } else {
                        " "
                    },
                    if state.show_thinking { "on" } else { "off" }
                ),
                format!(
                    "  {} Temperature  {:.1}",
                    if state.settings_behavior_cursor == 2 {
                        glyphs::ACTIVE
                    } else {
                        " "
                    },
                    state.temperature
                ),
                format!(
                    "  {} Max tokens  {}",
                    if state.settings_behavior_cursor == 3 {
                        glyphs::ACTIVE
                    } else {
                        " "
                    },
                    state.max_tokens
                ),
            ];

            for (i, item) in items.iter().enumerate() {
                let style = if i == state.settings_behavior_cursor {
                    Style::default().fg(theme.accent())
                } else {
                    Style::default().fg(theme.fg())
                };
                lines.push(Line::from(Span::styled(item.clone(), style)));
            }
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
                    "\u{25b6}"
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
        4 => {
            lines.push(Line::from(Span::styled(
                "  MCP Servers",
                Style::default().fg(theme.fg()).add_modifier(Modifier::BOLD),
            )));
            lines.push(Line::from(""));
            for (i, server) in state.mcp_servers.iter().enumerate() {
                let is_cursor = i == state.settings_mcp_cursor;
                let marker = if is_cursor { "\u{25b6}" } else { " " };
                let status = if server.enabled && server.connected {
                    "connected"
                } else if server.enabled {
                    "enabled"
                } else {
                    "disabled"
                };
                let style = if is_cursor {
                    Style::default().fg(theme.accent())
                } else {
                    Style::default().fg(theme.fg_dimmed())
                };
                lines.push(Line::from(Span::styled(
                    format!("  {} {} [{}]", marker, server.name, status),
                    style,
                )));
            }
            if state.settings_mcp_cursor >= state.mcp_servers.len() {
                state.settings_mcp_cursor = state.mcp_servers.len().saturating_sub(1);
            }
        }
        5 => {
            lines.push(Line::from(Span::styled(
                "  Pre-Prompt",
                Style::default().fg(theme.fg()).add_modifier(Modifier::BOLD),
            )));
            lines.push(Line::from(""));
            let items = [
                format!(
                    "  {} Enabled  {}",
                    if state.settings_preprompt_cursor == 0 {
                        glyphs::ACTIVE
                    } else {
                        " "
                    },
                    if state.preprompt_enabled { "on" } else { "off" }
                ),
                format!(
                    "  {} Mode  {}",
                    if state.settings_preprompt_cursor == 1 {
                        glyphs::ACTIVE
                    } else {
                        " "
                    },
                    state.preprompt_mode
                ),
            ];
            for (i, item) in items.iter().enumerate() {
                let style = if i == state.settings_preprompt_cursor {
                    Style::default().fg(theme.accent())
                } else {
                    Style::default().fg(theme.fg())
                };
                lines.push(Line::from(Span::styled(item.clone(), style)));
            }
        }
        6 => {
            lines.push(Line::from(Span::styled(
                "  Telemetry",
                Style::default().fg(theme.fg()).add_modifier(Modifier::BOLD),
            )));
            lines.push(Line::from(""));
            let items = [
                format!(
                    "  {} Show tokens  {}",
                    if state.settings_telemetry_cursor == 0 {
                        glyphs::ACTIVE
                    } else {
                        " "
                    },
                    if state.show_tokens_in_status {
                        "on"
                    } else {
                        "off"
                    }
                ),
                format!(
                    "  {} Show cost  {}",
                    if state.settings_telemetry_cursor == 1 {
                        glyphs::ACTIVE
                    } else {
                        " "
                    },
                    if state.show_cost_in_status {
                        "on"
                    } else {
                        "off"
                    }
                ),
                format!(
                    "  {} Show elapsed  {}",
                    if state.settings_telemetry_cursor == 2 {
                        glyphs::ACTIVE
                    } else {
                        " "
                    },
                    if state.show_elapsed_in_status {
                        "on"
                    } else {
                        "off"
                    }
                ),
            ];
            for (i, item) in items.iter().enumerate() {
                let style = if i == state.settings_telemetry_cursor {
                    Style::default().fg(theme.accent())
                } else {
                    Style::default().fg(theme.fg())
                };
                lines.push(Line::from(Span::styled(item.clone(), style)));
            }
        }
        7 => {
            lines.push(Line::from(Span::styled(
                "  Advanced",
                Style::default().fg(theme.fg()).add_modifier(Modifier::BOLD),
            )));
            lines.push(Line::from(""));
            let items = [
                format!(
                    "  {} Timeout  {}s",
                    if state.settings_advanced_cursor == 0 {
                        glyphs::ACTIVE
                    } else {
                        " "
                    },
                    state.timeout_secs
                ),
                format!(
                    "  {} Verbosity  {}",
                    if state.settings_advanced_cursor == 1 {
                        glyphs::ACTIVE
                    } else {
                        " "
                    },
                    state.verbosity
                ),
                format!(
                    "  {} Glyph mode  {}",
                    if state.settings_advanced_cursor == 2 {
                        glyphs::ACTIVE
                    } else {
                        " "
                    },
                    state.glyph_mode
                ),
                format!(
                    "  {} Mouse  {}",
                    if state.settings_advanced_cursor == 3 {
                        glyphs::ACTIVE
                    } else {
                        " "
                    },
                    if state.mouse_enabled { "on" } else { "off" }
                ),
            ];
            for (i, item) in items.iter().enumerate() {
                let style = if i == state.settings_advanced_cursor {
                    Style::default().fg(theme.accent())
                } else {
                    Style::default().fg(theme.fg())
                };
                lines.push(Line::from(Span::styled(item.clone(), style)));
            }
        }
        _ => {}
    }

    lines.push(Line::from(""));
    if (0..=7).contains(&state.settings_tab) {
        lines.push(Line::from(vec![
            Span::styled("  [Esc] ", Style::default().fg(theme.accent())),
            Span::styled("back    ", Style::default().fg(theme.fg())),
            Span::styled("[\u{2191}\u{2193}] ", Style::default().fg(theme.accent())),
            Span::styled("navigate    ", Style::default().fg(theme.fg())),
            Span::styled("[Enter] ", Style::default().fg(theme.accent())),
            Span::styled("toggle", Style::default().fg(theme.fg())),
        ]));
    }

    let paragraph = Paragraph::new(lines).style(Style::default().bg(theme.bg()).fg(theme.fg()));
    f.render_widget(paragraph, inner);
}
