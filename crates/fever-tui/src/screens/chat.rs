use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};

use crate::app::AppState;
use crate::components::message::MessageRole;
use crate::components::tool_card::ToolStatus;
use crate::slash::SlashCommand;
use crate::util::{glyphs, text};

fn matching_commands(input: &str) -> Vec<(&'static str, &'static str)> {
    if !input.starts_with('/') {
        return Vec::new();
    }
    let query = input.trim_start_matches('/').to_lowercase();
    if query.is_empty() {
        return SlashCommand::all_descriptions().to_vec();
    }
    SlashCommand::all_descriptions()
        .iter()
        .filter(|(name, _)| name.starts_with(&query))
        .copied()
        .collect()
}

pub fn render(f: &mut Frame, area: Rect, state: &mut AppState) {
    if area.height < 5 {
        return;
    }

    let theme = &state.theme;

    let has_autocomplete = state.input_buffer.starts_with('/')
        && !state.input_buffer.contains(' ')
        && state.input_buffer.len() > 1;

    let autocomplete_hints = if has_autocomplete {
        matching_commands(&state.input_buffer)
    } else {
        Vec::new()
    };

    let hint_rows = autocomplete_hints.len().min(5) as u16;
    let hint_height = if hint_rows > 0 { hint_rows + 1 } else { 0 };

    let chunks = Layout::vertical([
        Constraint::Min(1),
        Constraint::Length(hint_height),
        Constraint::Length(3),
    ])
    .split(area);

    let msg_width = chunks[0].width.saturating_sub(4) as usize;
    let mut lines: Vec<Line> = Vec::new();

    if state.messages.is_empty() && !state.streaming && state.tool_calls.is_empty() {
        let empty_lines = vec![
            Line::from(""),
            Line::from(Span::styled(
                "  No messages yet.",
                Style::default().fg(theme.fg_dimmed()),
            )),
            Line::from(Span::styled(
                "  Type a message below or press ? for help.",
                Style::default().fg(theme.fg_dimmed()),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "  Quick start:",
                Style::default()
                    .fg(theme.accent())
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(Span::styled(
                "  1. Set an API key: export OPENAI_API_KEY=sk-...",
                Style::default().fg(theme.fg_dimmed()),
            )),
            Line::from(Span::styled(
                "  2. Or use config: fever config --validate",
                Style::default().fg(theme.fg_dimmed()),
            )),
            Line::from(Span::styled(
                "  3. Then chat: /help for slash commands",
                Style::default().fg(theme.fg_dimmed()),
            )),
        ];
        lines.extend(empty_lines);
    } else {
        for msg in &state.messages {
            lines.push(Line::from(""));

            let is_error = msg.role == MessageRole::System
                && (msg.content.starts_with("Error:") || msg.content.starts_with("error:"));

            let (label, label_style): (String, Style) = match msg.role {
                MessageRole::User => (
                    "[you]".to_string(),
                    Style::default()
                        .fg(theme.accent())
                        .add_modifier(Modifier::BOLD),
                ),
                MessageRole::Assistant => (
                    format!("{} fever", glyphs::MARK),
                    Style::default().fg(theme.fg_dimmed()),
                ),
                MessageRole::System => {
                    if is_error {
                        (
                            "[error]".to_string(),
                            Style::default()
                                .fg(theme.error())
                                .add_modifier(Modifier::BOLD),
                        )
                    } else {
                        ("[system]".to_string(), Style::default().fg(theme.warning()))
                    }
                }
            };

            let ts = msg.timestamp.format("%H:%M").to_string();
            lines.push(Line::from(vec![
                Span::styled(format!("  {} ", label), label_style),
                Span::styled(ts, Style::default().fg(theme.fg_dimmed())),
            ]));

            let content_style = if is_error {
                Style::default().fg(theme.error())
            } else {
                Style::default().fg(theme.fg())
            };

            let wrapped = text::wrap_text(&msg.content, msg_width);
            for line in wrapped {
                lines.push(Line::from(Span::styled(
                    format!("  {}", line),
                    content_style,
                )));
            }
        }

        if !state.tool_calls.is_empty() {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                format!("  {} Tools", glyphs::SECTION_LINE),
                Style::default()
                    .fg(theme.fg_dimmed())
                    .add_modifier(Modifier::BOLD),
            )));

            for tc in &state.tool_calls {
                let (icon, color) = match tc.status {
                    ToolStatus::Running => (glyphs::ACTIVE, theme.accent()),
                    ToolStatus::Completed => (glyphs::CHECK, theme.success()),
                    ToolStatus::Failed => (glyphs::CROSS, theme.error()),
                };

                let tool_label = format!("  {} {} ", icon, tc.tool_name);
                let args_display = if tc.args_summary.len() > 50 {
                    format!("{}...", &tc.args_summary[..50])
                } else {
                    tc.args_summary.clone()
                };

                lines.push(Line::from(vec![
                    Span::styled(
                        tool_label,
                        Style::default().fg(color).add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(args_display, Style::default().fg(theme.fg_dimmed())),
                ]));

                if let Some(ref result) = tc.result_preview {
                    if tc.status != ToolStatus::Running {
                        let preview = if result.len() > 80 {
                            format!("{}...", &result[..80])
                        } else {
                            result.clone()
                        };
                        for pl in preview.lines().take(2) {
                            lines.push(Line::from(Span::styled(
                                format!("      {}", pl),
                                Style::default().fg(theme.fg_dimmed()),
                            )));
                        }
                    }
                }
            }
        }
    }

    if state.streaming {
        lines.push(Line::from(Span::styled(
            format!("  {} Streaming...", glyphs::CURSOR),
            Style::default().fg(theme.accent()),
        )));
    } else if state.loading {
        lines.push(Line::from(Span::styled(
            format!("  {} Thinking...", glyphs::THINKING),
            Style::default().fg(theme.accent()),
        )));
    }

    let title = format!(" Chat ({} messages) ", state.messages.len());
    let messages_block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme.style_border(true))
        .title(Span::styled(title, theme.style_title(true)));

    let messages_para = Paragraph::new(lines)
        .block(messages_block)
        .wrap(Wrap { trim: false })
        .scroll((state.scroll_offset, 0))
        .style(Style::default().fg(theme.fg()).bg(theme.bg()));
    f.render_widget(messages_para, chunks[0]);

    if hint_height > 0 {
        let mut hint_lines: Vec<Line> = Vec::new();
        for (name, desc) in autocomplete_hints.iter().take(5) {
            hint_lines.push(Line::from(vec![
                Span::styled(
                    format!(" /{}", name),
                    Style::default()
                        .fg(theme.accent())
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!("  {}", desc),
                    Style::default().fg(theme.fg_dimmed()),
                ),
            ]));
        }
        let hint_block = Block::default()
            .border_style(Style::default().fg(theme.border_color()))
            .style(Style::default().bg(theme.bg_secondary()));
        let hint_para = Paragraph::new(hint_lines)
            .block(hint_block)
            .style(Style::default().bg(theme.bg_secondary()));
        f.render_widget(hint_para, chunks[1]);
    }

    let input_title = if state.input_buffer.starts_with('/') {
        let cmd = state.input_buffer.split_whitespace().next().unwrap_or("");
        format!(" Command: {} ", cmd)
    } else {
        " Input ".to_string()
    };

    let input_block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme.style_border(true))
        .title(Span::styled(
            input_title,
            Style::default().fg(theme.accent()),
        ));
    let display = if state.input_buffer.is_empty() {
        "Type a message... (/? for help)".to_string()
    } else {
        state.input_buffer.clone()
    };
    let input_style = if state.input_buffer.is_empty() {
        Style::default()
            .fg(theme.fg_dimmed())
            .bg(theme.bg_secondary())
    } else {
        Style::default().fg(theme.fg()).bg(theme.bg_secondary())
    };
    let input = Paragraph::new(format!("> {}", display))
        .block(input_block)
        .style(input_style);
    f.render_widget(input, chunks[2]);
}
