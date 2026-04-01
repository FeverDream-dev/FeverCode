use crate::theme::Theme;
use crate::util::glyphs::{ACTIVE, CHECK, CROSS, MARK};
use ratatui::layout::Rect;
use ratatui::widgets::Paragraph;
use ratatui::Frame;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolStatus {
    Running,
    Completed,
    Failed,
}

pub struct ToolCard {
    pub tool_name: String,
    pub args_summary: String,
    pub status: ToolStatus,
    pub result_preview: Option<String>,
    pub expanded: bool,
}

impl ToolCard {
    pub fn new(tool_name: String, args: String) -> Self {
        let mut summary = args;
        if summary.len() > 60 {
            summary.truncate(60);
            summary.push_str("...");
        }
        Self {
            tool_name,
            args_summary: summary,
            status: ToolStatus::Running,
            result_preview: None,
            expanded: false,
        }
    }

    pub fn complete(&mut self, result: String) {
        self.status = ToolStatus::Completed;
        self.result_preview = Some(result);
    }

    pub fn fail(&mut self, error: String) {
        self.status = ToolStatus::Failed;
        self.result_preview = Some(error);
    }

    pub fn is_running(&self) -> bool {
        self.status == ToolStatus::Running
    }

    pub fn toggle_expand(&mut self) {
        self.expanded = !self.expanded;
    }

    pub fn render(&mut self, f: &mut Frame, area: Rect, theme: &Theme) {
        if area.height == 0 || area.width == 0 {
            return;
        }

        let (symbol, color_style) = match self.status {
            ToolStatus::Running => (ACTIVE, theme.style_accent()),
            ToolStatus::Completed => (CHECK, theme.style_success()),
            ToolStatus::Failed => (CROSS, theme.style_error()),
        };

        let mut header = format!("{} Tool: {} - {}", MARK, self.tool_name, self.args_summary);
        if self.expanded {
            header.push_str(" (expanded)");
        }
        let display = if header.is_empty() {
            symbol.to_string()
        } else {
            format!("{} {}", symbol, header)
        };

        let mut text = display;
        if self.expanded {
            if let Some(ref r) = self.result_preview {
                text.push('\n');
                text.push_str(r);
            }
        }

        let paragraph = Paragraph::new(text).style(color_style);
        f.render_widget(paragraph, area);
    }
}
