use chrono::{DateTime, Local};
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::widgets::Paragraph;

use crate::theme::Theme;
use crate::util::text::wrap_text;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MessageRole {
    User,
    Assistant,
    System,
}

pub struct MessageBubble {
    pub role: MessageRole,
    pub content: String,
    pub timestamp: DateTime<Local>,
    pub streaming: bool,
}

impl MessageBubble {
    pub fn new(role: MessageRole, content: String) -> Self {
        Self {
            role,
            content,
            timestamp: Local::now(),
            streaming: false,
        }
    }

    pub fn append(&mut self, text: &str) {
        self.content.push_str(text);
    }

    pub fn finish_stream(&mut self) {
        self.streaming = false;
    }

    pub fn content_height(&self, width: u16) -> u16 {
        let header = match self.role {
            MessageRole::User => "you",
            MessageRole::Assistant => "◈ fever",
            MessageRole::System => "system",
        };
        let text = format!("{}: {}", header, self.content);
        let lines = wrap_text(&text, width as usize);
        lines.len() as u16
    }

    pub fn render(&mut self, f: &mut Frame, area: Rect, theme: &Theme) {
        if area.height == 0 || area.width == 0 {
            return;
        }

        let header = match self.role {
            MessageRole::User => "you",
            MessageRole::Assistant => "◈ fever",
            MessageRole::System => "system",
        };
        let mut text = format!("{}: {}", header, self.content);
        if self.streaming {
            text.push('▌');
        }
        let wrapped = wrap_text(&text, area.width as usize);
        let display = wrapped.join("\n");

        // Simple styling per role
        let style = match self.role {
            MessageRole::User => theme.style_accent_bold(),
            MessageRole::Assistant => theme.style_fg(),
            MessageRole::System => theme.style_warning(),
        };

        let paragraph = Paragraph::new(display).style(style);
        f.render_widget(paragraph, area);
    }
}

// Implement the shared Component trait for this module using the parent path.
impl super::Component for MessageBubble {
    fn render(&mut self, f: &mut Frame, area: Rect, theme: &Theme) {
        self.render(f, area, theme);
    }
}
