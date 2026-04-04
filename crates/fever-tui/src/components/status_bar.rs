use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

use crate::theme::Theme;
use crate::util::glyphs;

pub struct StatusBar {
    pub provider: String,
    pub model: String,
    pub theme_name: String,
    pub workspace: String,
    pub token_count: usize,
    pub streaming: bool,
    pub message_count: usize,
    // Telemetry fields
    pub input_tokens: usize,
    pub output_tokens: usize,
    pub total_tokens: usize,
    pub estimated_cost: f64,
    pub request_elapsed: Option<std::time::Duration>,
    pub show_tokens: bool,
    pub show_cost: bool,
    pub show_elapsed: bool,
}

impl StatusBar {
    pub fn new() -> Self {
        Self {
            provider: "none".to_string(),
            model: "none".to_string(),
            theme_name: "none".to_string(),
            workspace: "~".to_string(),
            token_count: 0,
            streaming: false,
            message_count: 0,
            input_tokens: 0,
            output_tokens: 0,
            total_tokens: 0,
            estimated_cost: 0.0,
            request_elapsed: None,
            show_tokens: false,
            show_cost: false,
            show_elapsed: false,
        }
    }

    pub fn render(&self, f: &mut Frame, area: Rect, theme: &Theme) {
        if area.height == 0 || area.width == 0 {
            return;
        }

        let streaming_indicator = if self.streaming { " streaming..." } else { "" };

        // Telemetry spans (conditionally shown)
        let tokens_span = if self.show_tokens && self.total_tokens > 0 {
            Span::styled(
                format!("{} tok ", self.total_tokens),
                Style::default().fg(theme.fg_dimmed()),
            )
        } else {
            Span::styled(String::new(), Style::default())
        };

        let cost_span = if self.show_cost && self.estimated_cost > 0.0 {
            Span::styled(
                format!("${:.4} ", self.estimated_cost),
                Style::default().fg(theme.fg_dimmed()),
            )
        } else {
            Span::styled(String::new(), Style::default())
        };

        let elapsed_span = if self.show_elapsed {
            match self.request_elapsed {
                Some(d) => Span::styled(
                    format!("{:.1}s ", d.as_secs_f64()),
                    Style::default().fg(theme.fg_dimmed()),
                ),
                None => Span::styled(String::new(), Style::default()),
            }
        } else {
            Span::styled(String::new(), Style::default())
        };

        let line = Line::from(vec![
            Span::styled(
                format!(" {} ", glyphs::MARK),
                Style::default().fg(theme.accent()),
            ),
            Span::styled(
                format!("{} ", self.provider),
                Style::default().fg(theme.fg()),
            ),
            Span::styled(
                format!("{} ", self.model),
                Style::default().fg(theme.fg_dimmed()),
            ),
            Span::styled(
                format!("{} ", self.theme_name),
                Style::default().fg(theme.accent()),
            ),
            Span::styled(" ", Style::default()),
            Span::styled(
                format!("{} msg ", self.message_count),
                Style::default().fg(theme.fg_dimmed()),
            ),
            tokens_span,
            cost_span,
            elapsed_span,
            Span::styled(
                format!("{} ", self.workspace),
                Style::default().fg(theme.fg_dimmed()),
            ),
            Span::styled(
                streaming_indicator.to_string(),
                Style::default().fg(theme.warning()),
            ),
            Span::styled(" ? help", Style::default().fg(theme.fg_dimmed())),
        ]);

        let paragraph =
            Paragraph::new(line).style(Style::default().bg(theme.bg_secondary()).fg(theme.fg()));
        f.render_widget(paragraph, area);
    }
}

impl Default for StatusBar {
    fn default() -> Self {
        Self::new()
    }
}
