use ratatui::layout::Rect;
use ratatui::style::Modifier;
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::theme::Theme;
use crate::util::text::center_text;

// Static and animated logo renderer
pub struct Logo;

impl Logo {
    pub fn new() -> Self {
        Self
    }

    pub fn render_small(&self, f: &mut Frame, area: Rect, theme: &Theme) {
        let text = "◈ FEVER";
        let centered = center_text(text, area.width as usize);
        let style = theme.style_accent_bold();
        let p = Paragraph::new(centered).style(style);
        f.render_widget(p, area);
    }

    pub fn render_full(&self, f: &mut Frame, area: Rect, theme: &Theme) {
        let lines = vec![
            "                    ◈",
            "                  FEVER",
            "              cold sacred code",
        ];
        let mut display = String::new();
        for line in lines {
            display.push_str(&center_text(line, area.width as usize));
            display.push('\n');
        }
        // Remove final newline
        if display.ends_with('\n') {
            display.pop();
        }
        let p = Paragraph::new(display).style(theme.style_fg().add_modifier(Modifier::BOLD));
        f.render_widget(p, area);
    }

    pub fn render(&self, f: &mut Frame, area: Rect, theme: &Theme) {
        // Always render full logo in this simplified implementation
        self.render_full(f, area, theme);
    }
}

impl Default for Logo {
    fn default() -> Self {
        Self::new()
    }
}

// Implement the shared Component trait for this module using the parent path.
impl super::Component for Logo {
    fn render(&mut self, f: &mut Frame, area: Rect, theme: &Theme) {
        // Delegate to full render for now
        self.render_full(f, area, theme);
    }
}
