use crate::widgets::{ChatPane, PlanPane, TaskPane, ToolLogPane, BrowserPane};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Paragraph, Wrap},
    Frame,
};

#[derive(PartialEq)]
pub enum Focus {
    Chat,
    Plan,
    Tasks,
    ToolLog,
    Browser,
}

pub struct FeverUI {
    pub chat: ChatPane,
    pub plan: PlanPane,
    pub tasks: TaskPane,
    pub tool_log: ToolLogPane,
    pub browser: BrowserPane,
    focus: Focus,
    status: String,
}

impl FeverUI {
    pub fn new() -> Self {
        Self {
            chat: ChatPane::new(),
            plan: PlanPane::new(),
            tasks: TaskPane::new(),
            tool_log: ToolLogPane::new(),
            browser: BrowserPane::new(),
            focus: Focus::Chat,
            status: "Ready".to_string(),
        }
    }

    pub fn set_focus(&mut self, focus: Focus) {
        self.focus = focus;
    }

    pub fn scroll_down(&mut self) {
        match self.focus {
            Focus::Chat => self.chat.scroll_down(),
            Focus::Plan => self.plan.scroll_down(),
            Focus::Tasks => self.tasks.scroll_down(),
            Focus::ToolLog => self.tool_log.scroll_down(),
            Focus::Browser => self.browser.scroll_down(),
        }
    }

    pub fn scroll_up(&mut self) {
        match self.focus {
            Focus::Chat => self.chat.scroll_up(),
            Focus::Plan => self.plan.scroll_up(),
            Focus::Tasks => self.tasks.scroll_up(),
            Focus::ToolLog => self.tool_log.scroll_up(),
            Focus::Browser => self.browser.scroll_up(),
        }
    }

    pub fn set_status(&mut self, status: String) {
        self.status = status;
    }

    pub fn render(&mut self, f: &mut Frame) {
        let size = f.area();

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(0),
                Constraint::Length(3),
            ])
            .split(size);

        let main_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(60),
                Constraint::Percentage(40),
            ])
            .split(chunks[0]);

        let left_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(50),
                Constraint::Percentage(50),
            ])
            .split(main_chunks[0]);

        let right_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(50),
                Constraint::Percentage(50),
            ])
            .split(main_chunks[1]);

        self.chat.render(f, left_chunks[0], self.focus == Focus::Chat);
        self.plan.render(f, left_chunks[1], self.focus == Focus::Plan);
        self.tasks.render(f, right_chunks[0], self.focus == Focus::Tasks);
        self.tool_log.render(f, right_chunks[1], self.focus == Focus::ToolLog);

        let status_bar = Paragraph::new(self.status.as_str())
            .style(Style::default().bg(Color::DarkGray).fg(Color::White))
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true });

        f.render_widget(status_bar, chunks[1]);
    }
}

impl Default for FeverUI {
    fn default() -> Self {
        Self::new()
    }
}
