use std::fmt;

use crossterm::event::KeyEvent;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;
use tui_textarea::TextArea;

use crate::theme::Theme;

// The input mode indicator
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InputMode {
    Chat,
    Command,
    Search,
}

// Multi-line input bar component
pub struct InputBar {
    textarea: TextArea<'static>,
    mode: InputMode,
    focused: bool,
    // kept in sync with textarea for easier access
    content: String,
}

impl InputBar {
    pub fn new() -> Self {
        Self {
            textarea: TextArea::default(),
            mode: InputMode::Chat,
            focused: false,
            content: String::new(),
        }
    }

    // Return current text input
    pub fn input(&self) -> &str {
        &self.content
    }

    pub fn set_mode(&mut self, mode: InputMode) {
        self.mode = mode;
    }

    pub fn clear(&mut self) {
        self.textarea = TextArea::default();
        self.content.clear();
    }

    // Consume keys into the textarea. Returns true if the key was consumed.
    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        let consumed = self.textarea.input(key);
        if consumed {
            // Synchronize content by reading current lines from textarea
            let lines = self.textarea.lines();
            let mut joined = String::new();
            for (i, line) in lines.iter().enumerate() {
                if i > 0 {
                    joined.push('\n');
                }
                joined.push_str(line);
            }
            self.content = joined;
        }
        consumed
    }

    pub fn render(&mut self, f: &mut Frame, area: Rect, theme: &Theme) {
        // Guard against zero-sized area
        if area.height == 0 || area.width == 0 {
            return;
        }

        let border_style = theme.style_border(self.focused);
        let outer_block = Block::default()
            .borders(Borders::ALL)
            .border_style(border_style);
        f.render_widget(outer_block, area);

        // Split horizontally: left indicator and right textarea
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(6), Constraint::Min(0)].as_ref())
            .split(area);

        // Mode indicator
        let mode_label = match self.mode {
            InputMode::Chat => "[>]",
            InputMode::Command => "[/]",
            InputMode::Search => "[?]",
        };
        let mode_style = if self.focused {
            theme.style_accent()
        } else {
            theme.style_dimmed()
        };

        let mode_widget = Paragraph::new(mode_label).style(mode_style);
        f.render_widget(mode_widget, chunks[0]);

        // Text area on the right
        // TextArea renders itself as a ratatui widget
        f.render_widget(&self.textarea, chunks[1]);
        // Note: content is updated in handle_key
    }
}

impl Default for InputBar {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for InputBar {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("InputBar")
            .field("mode", &self.mode)
            .field("focused", &self.focused)
            .field("text_length", &self.content.len())
            .finish()
    }
}

// Implement the shared Component trait for this module using the parent path.
impl super::Component for InputBar {
    fn render(&mut self, f: &mut Frame, area: Rect, theme: &Theme) {
        self.render(f, area, theme);
    }
}
