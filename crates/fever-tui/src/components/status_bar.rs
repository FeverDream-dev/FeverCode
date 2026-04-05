use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::widgets::Wrap;
use ratatui::Frame;

use crate::theme::Theme;

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
    pub git_branch: Option<String>,
    pub permission_mode: String,
    pub is_mock_mode: bool,
    pub session_id: String,
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
            git_branch: None,
            permission_mode: "full".to_string(),
            is_mock_mode: false,
            session_id: String::new(),
        }
    }

    pub fn render(&self, f: &mut Frame, area: Rect, theme: &Theme) {
        if area.height == 0 || area.width == 0 {
            return;
        }

        // Build a segmented status bar left-to-right with optional right-aligned tips
        let narrow = area.width < 60;

        let mut spans: Vec<Span> = Vec::new();

        // Segment 1: provider connectivity dot + provider/model
        let dot_style = if self.is_mock_mode {
            Style::default().fg(ratatui::style::Color::Yellow)
        } else {
            Style::default().fg(ratatui::style::Color::Green)
        };
        spans.push(Span::styled("●", dot_style));
        spans.push(Span::raw(" "));
        spans.push(Span::raw(format!("{}/{} ", self.provider, self.model)));

        // Segment 2: separator
        spans.push(Span::styled("│", Style::default().fg(theme.fg_dimmed())));
        spans.push(Span::raw(" "));

        // Segment 3: abbreviated session ID
        let abbr = if self.session_id.len() > 16 {
            self.session_id.as_str()[0..16].to_string()
        } else {
            self.session_id.clone()
        };
        spans.push(Span::styled(
            "session:",
            Style::default().fg(theme.fg_dimmed()),
        ));
        spans.push(Span::raw(format!(" {} ", abbr)));

        if !narrow {
            // Segment 4: separator and Segment 5: git branch or workspace basename
            spans.push(Span::styled("│", Style::default().fg(theme.fg_dimmed())));
            spans.push(Span::raw(" "));
            let branch_or_base = if let Some(ref br) = self.git_branch {
                br.clone()
            } else {
                std::path::Path::new(&self.workspace)
                    .file_name()
                    .and_then(|os| os.to_str())
                    .unwrap_or(&self.workspace)
                    .to_string()
            };
            spans.push(Span::styled(
                branch_or_base,
                Style::default().fg(theme.fg()),
            ));
            spans.push(Span::raw(" "));
            // Segment 6: separator
            spans.push(Span::styled("│", Style::default().fg(theme.fg_dimmed())));
            spans.push(Span::raw(" "));
            // Segment 7: permission badge
            let badge_style = match self.permission_mode.as_str() {
                "full" => Style::default().fg(ratatui::style::Color::Green),
                "write" => Style::default().fg(theme.accent()),
                "read" => Style::default().fg(ratatui::style::Color::Yellow),
                _ => Style::default(),
            };
            spans.push(Span::styled(
                format!("[{}]", self.permission_mode),
                badge_style,
            ));
            spans.push(Span::raw(" "));
        }

        // Segment 8: right-aligned telemetry and help
        let mut right_text = String::new();
        if self.streaming {
            right_text.push_str("streaming");
        }
        if self.show_tokens && self.total_tokens > 0 {
            if !right_text.is_empty() {
                right_text.push(' ');
            }
            right_text.push_str(&format!("{} tok", self.total_tokens));
        }
        if self.show_cost && self.estimated_cost > 0.0 {
            if !right_text.is_empty() {
                right_text.push(' ');
            }
            right_text.push_str(&format!("${:.4}", self.estimated_cost));
        }
        if self.show_elapsed {
            if let Some(d) = self.request_elapsed {
                if !right_text.is_empty() {
                    right_text.push(' ');
                }
                right_text.push_str(&format!("{:.1}s", d.as_secs_f64()));
            }
        }
        let right_with_help = if right_text.is_empty() {
            String::from("  ? help")
        } else {
            format!("{}  ? help", right_text)
        };
        let right_len = right_with_help.len();
        let pad_len = if area.width as usize > right_len {
            area.width as usize - right_len
        } else {
            0
        };
        spans.push(Span::styled(" ".repeat(pad_len), Style::default()));
        spans.push(Span::styled(
            right_with_help,
            Style::default().fg(theme.fg_dimmed()),
        ));

        let line = Line::from(spans);
        let paragraph = Paragraph::new(line)
            .wrap(Wrap { trim: false })
            .style(Style::default().bg(theme.bg_secondary()).fg(theme.fg()));
        f.render_widget(paragraph, area);
    }
}

impl Default for StatusBar {
    fn default() -> Self {
        Self::new()
    }
}
