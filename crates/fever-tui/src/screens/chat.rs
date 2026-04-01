use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};

use crate::app::AppState;
use crate::components::message::MessageRole;
use crate::util::text;

pub fn render(f: &mut Frame, area: Rect, state: &mut AppState) {
    if area.height < 5 {
        return;
    }

    let theme = &state.theme;

    let chunks = Layout::vertical([Constraint::Min(1), Constraint::Length(3)]).split(area);

    let msg_width = chunks[0].width.saturating_sub(4) as usize;
    let mut lines: Vec<Line> = Vec::new();

    for msg in &state.messages {
        lines.push(Line::from(""));
        let (label, label_style) = match msg.role {
            MessageRole::User => (
                "[you]",
                Style::default()
                    .fg(theme.accent())
                    .add_modifier(Modifier::BOLD),
            ),
            MessageRole::Assistant => ("\u{25c8} fever", Style::default().fg(theme.fg_dimmed())),
            MessageRole::System => ("[system]", Style::default().fg(theme.warning())),
        };
        lines.push(Line::from(Span::styled(label, label_style)));

        let wrapped = text::wrap_text(&msg.content, msg_width);
        for line in wrapped {
            lines.push(Line::from(format!("  {}", line)));
        }
    }

    if state.streaming {
        lines.push(Line::from(Span::styled(
            "  ...\u{258c}",
            Style::default().fg(theme.accent()),
        )));
    }

    let messages_block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme.style_border(true))
        .title(Span::styled(" Chat ", theme.style_title(true)));

    let messages_para = Paragraph::new(lines)
        .block(messages_block)
        .wrap(Wrap { trim: false })
        .style(Style::default().fg(theme.fg()).bg(theme.bg()));
    f.render_widget(messages_para, chunks[0]);

    let input_block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme.style_border(true));
    let display = if state.input_buffer.is_empty() {
        "Type a message...".to_string()
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
    f.render_widget(input, chunks[1]);
}
