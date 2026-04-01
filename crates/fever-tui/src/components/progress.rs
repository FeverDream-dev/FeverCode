use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::Span;
use ratatui::widgets::Paragraph;

use crate::theme::Theme;

const SPINNER_FRAMES: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

pub struct Spinner {
    pub active: bool,
    pub frame: usize,
    pub label: String,
    pub elapsed_secs: u64,
}

impl Spinner {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            active: false,
            frame: 0,
            label: label.into(),
            elapsed_secs: 0,
        }
    }

    pub fn start(&mut self) {
        self.active = true;
        self.frame = 0;
        self.elapsed_secs = 0;
    }

    pub fn stop(&mut self) {
        self.active = false;
    }

    pub fn tick(&mut self) {
        if self.active {
            self.frame = (self.frame + 1) % SPINNER_FRAMES.len();
        }
    }

    pub fn render(&self, f: &mut Frame, area: Rect, theme: &Theme) {
        if area.height == 0 || area.width == 0 {
            return;
        }

        let spinner_char = if self.active {
            SPINNER_FRAMES[self.frame]
        } else {
            " "
        };

        let elapsed = if self.elapsed_secs > 0 {
            format!(" ({}s)", self.elapsed_secs)
        } else {
            String::new()
        };

        let text = format!("{} {}{}", spinner_char, self.label, elapsed);
        let style = Style::default().fg(theme.accent());
        let para = Paragraph::new(Span::styled(text, style));
        f.render_widget(para, area);
    }
}
