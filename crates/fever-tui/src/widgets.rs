use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
};

#[derive(Clone)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

pub struct ChatPane {
    messages: Vec<ChatMessage>,
    scroll: usize,
    input_buffer: String,
    input_mode: bool,
}

impl ChatPane {
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
            scroll: 0,
            input_buffer: String::new(),
            input_mode: false,
        }
    }

    pub fn add_message(&mut self, role: String, content: String) {
        self.messages.push(ChatMessage {
            role,
            content,
            timestamp: chrono::Utc::now(),
        });
    }

    pub fn toggle_input_mode(&mut self) {
        self.input_mode = !self.input_mode;
    }

    pub fn is_in_input_mode(&self) -> bool {
        self.input_mode
    }

    pub fn handle_char(&mut self, c: char) -> Option<String> {
        if self.input_mode {
            if c == '\n' || c == '\r' {
                if !self.input_buffer.is_empty() {
                    let content = self.input_buffer.clone();
                    self.input_buffer.clear();
                    self.add_message("user".to_string(), content.clone());
                    return Some(content);
                }
            } else if c == '\u{0078}' {
                self.input_buffer.pop();
            } else {
                self.input_buffer.push(c);
            }
        }
        None
    }

    pub fn get_input_buffer(&self) -> &str {
        &self.input_buffer
    }

    pub fn clear_input_buffer(&mut self) {
        self.input_buffer.clear();
    }

    pub fn backspace(&mut self) {
        self.input_buffer.pop();
    }

    pub fn type_char(&mut self, c: char) {
        self.input_buffer.push(c);
    }

    pub fn scroll_down(&mut self) {
        if self.scroll < self.messages.len().saturating_sub(1) {
            self.scroll += 1;
        }
    }

    pub fn scroll_up(&mut self) {
        if self.scroll > 0 {
            self.scroll -= 1;
        }
    }

    pub fn render(&mut self, f: &mut Frame, area: Rect, focused: bool) {
        let title = if focused { "Messages ●" } else { "Messages" };

        let block = Block::default()
            .borders(Borders::ALL)
            .title(title)
            .title_style(if focused {
                Style::default().add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            });

        let inner = block.inner(area);
        f.render_widget(block, area);

        let mut lines: Vec<Line> = Vec::new();

        for msg in self.messages.iter().skip(self.scroll) {
            let role_color = match msg.role.as_str() {
                "user" => Color::Cyan,
                "assistant" => Color::Green,
                "system" => Color::Yellow,
                _ => Color::Gray,
            };

            lines.push(Line::from(Span::styled(
                format!("[{}] ", msg.role),
                Style::default().fg(role_color).add_modifier(Modifier::BOLD),
            )));

            for line in msg.content.lines() {
                lines.push(Line::from(line.to_string()));
            }
            lines.push(Line::from(""));
        }

        let input_area = Rect {
            x: inner.x,
            y: inner.y + inner.height.saturating_sub(3),
            width: inner.width,
            height: 3,
        };

        let input_block = Block::default()
            .borders(Borders::ALL)
            .title("Input (type message, press Enter)")
            .border_style(Style::default().fg(Color::Yellow));

        let input_inner = input_block.inner(input_area);
        f.render_widget(input_block, input_area);

        let input_text =
            Paragraph::new(self.input_buffer.as_str()).style(Style::default().fg(Color::White));
        f.render_widget(input_text, input_inner);

        let messages_area = Rect {
            x: inner.x,
            y: inner.y,
            width: inner.width,
            height: inner.height.saturating_sub(3),
        };

        if !messages_area.is_empty() {
            return;
        }

        let items: Vec<ListItem> = self
            .messages
            .iter()
            .skip(self.scroll)
            .map(|msg| {
                let role_color = match msg.role.as_str() {
                    "user" => Color::Cyan,
                    "assistant" => Color::Green,
                    "system" => Color::Yellow,
                    _ => Color::Gray,
                };

                let content: Vec<Line> = vec![
                    Line::from(Span::styled(
                        format!("[{}] ", msg.role),
                        Style::default().fg(role_color).add_modifier(Modifier::BOLD),
                    )),
                    Line::from(msg.content.clone()),
                    Line::from(""),
                ];

                ListItem::new(Text::from(content))
            })
            .collect();

        let list = List::new(items);
        f.render_widget(list, messages_area);
    }
}

impl Default for ChatPane {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(dead_code)]
pub struct PlanPane {
    plan: Option<fever_core::Plan>,
    expanded_tasks: std::collections::HashSet<String>,
    scroll: usize,
}

impl PlanPane {
    pub fn new() -> Self {
        Self {
            plan: None,
            expanded_tasks: std::collections::HashSet::new(),
            scroll: 0,
        }
    }

    pub fn set_plan(&mut self, plan: fever_core::Plan) {
        self.plan = Some(plan);
    }

    pub fn scroll_down(&mut self) {
        if let Some(plan) = &self.plan {
            if self.scroll < plan.tasks.len().saturating_sub(1) {
                self.scroll += 1;
            }
        }
    }

    pub fn scroll_up(&mut self) {
        if self.scroll > 0 {
            self.scroll -= 1;
        }
    }

    pub fn render(&mut self, f: &mut Frame, area: Rect, focused: bool) {
        let title = if focused { "Plan ●" } else { "Plan" };

        let block = Block::default()
            .borders(Borders::ALL)
            .title(title)
            .title_style(if focused {
                Style::default().add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            });

        let inner = block.inner(area);
        f.render_widget(block, area);

        let content = if let Some(plan) = &self.plan {
            let lines: Vec<Line> = plan
                .tasks
                .iter()
                .skip(self.scroll)
                .map(|task| {
                    let status_color = match task.status {
                        fever_core::TaskStatus::Queued => Color::Gray,
                        fever_core::TaskStatus::Running => Color::Yellow,
                        fever_core::TaskStatus::Completed => Color::Green,
                        fever_core::TaskStatus::Failed => Color::Red,
                        fever_core::TaskStatus::Blocked => Color::DarkGray,
                    };

                    Line::from(vec![
                        Span::styled(
                            match task.status {
                                fever_core::TaskStatus::Queued => "[ ]",
                                fever_core::TaskStatus::Running => "[*]",
                                fever_core::TaskStatus::Completed => "[✓]",
                                fever_core::TaskStatus::Failed => "[✗]",
                                fever_core::TaskStatus::Blocked => "[-]",
                            },
                            Style::default().fg(status_color),
                        ),
                        Span::raw(" "),
                        Span::styled(&task.title, Style::default()),
                    ])
                })
                .collect();

            Text::from(lines)
        } else {
            Text::from("No active plan")
        };

        let paragraph = Paragraph::new(content).wrap(Wrap { trim: false });
        f.render_widget(paragraph, inner);
    }
}

impl Default for PlanPane {
    fn default() -> Self {
        Self::new()
    }
}

pub struct TaskPane {
    todos: Vec<fever_core::Todo>,
    scroll: usize,
}

impl TaskPane {
    pub fn new() -> Self {
        Self {
            todos: Vec::new(),
            scroll: 0,
        }
    }

    pub fn add_todo(&mut self, todo: fever_core::Todo) {
        self.todos.push(todo);
    }

    pub fn update_status(&mut self, task_id: &str, status: fever_core::TaskStatus) {
        if let Some(todo) = self.todos.iter_mut().find(|t| t.id == task_id) {
            todo.status = status;
        }
    }

    pub fn scroll_down(&mut self) {
        if self.scroll < self.todos.len().saturating_sub(1) {
            self.scroll += 1;
        }
    }

    pub fn scroll_up(&mut self) {
        if self.scroll > 0 {
            self.scroll -= 1;
        }
    }

    pub fn render(&mut self, f: &mut Frame, area: Rect, focused: bool) {
        let title = if focused { "Tasks ●" } else { "Tasks" };

        let block = Block::default()
            .borders(Borders::ALL)
            .title(title)
            .title_style(if focused {
                Style::default().add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            });

        let inner = block.inner(area);
        f.render_widget(block, area);

        let items: Vec<ListItem> = self
            .todos
            .iter()
            .skip(self.scroll)
            .map(|todo| {
                let status_color = match todo.status {
                    fever_core::TaskStatus::Queued => Color::Gray,
                    fever_core::TaskStatus::Running => Color::Yellow,
                    fever_core::TaskStatus::Completed => Color::Green,
                    fever_core::TaskStatus::Failed => Color::Red,
                    fever_core::TaskStatus::Blocked => Color::DarkGray,
                };

                ListItem::new(Line::from(vec![
                    Span::styled(
                        match todo.status {
                            fever_core::TaskStatus::Queued => "[ ]",
                            fever_core::TaskStatus::Running => "[*]",
                            fever_core::TaskStatus::Completed => "[✓]",
                            fever_core::TaskStatus::Failed => "[✗]",
                            fever_core::TaskStatus::Blocked => "[-]",
                        },
                        Style::default().fg(status_color),
                    ),
                    Span::raw(" "),
                    Span::styled(&todo.content, Style::default()),
                ]))
            })
            .collect();

        let list = List::new(items);
        f.render_widget(list, inner);
    }
}

impl Default for TaskPane {
    fn default() -> Self {
        Self::new()
    }
}

pub struct ToolLogPane {
    logs: Vec<(String, String, chrono::DateTime<chrono::Utc>)>,
    scroll: usize,
}

impl ToolLogPane {
    pub fn new() -> Self {
        Self {
            logs: Vec::new(),
            scroll: 0,
        }
    }

    pub fn log(&mut self, tool_name: String, args: String) {
        self.logs.push((tool_name, args, chrono::Utc::now()));
    }

    pub fn scroll_down(&mut self) {
        if self.scroll < self.logs.len().saturating_sub(1) {
            self.scroll += 1;
        }
    }

    pub fn scroll_up(&mut self) {
        if self.scroll > 0 {
            self.scroll -= 1;
        }
    }

    pub fn render(&mut self, f: &mut Frame, area: Rect, focused: bool) {
        let title = if focused { "Tool Log ●" } else { "Tool Log" };

        let block = Block::default()
            .borders(Borders::ALL)
            .title(title)
            .title_style(if focused {
                Style::default().add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            });

        let inner = block.inner(area);
        f.render_widget(block, area);

        let lines: Vec<Line> = self
            .logs
            .iter()
            .skip(self.scroll)
            .map(|(tool, args, _)| {
                Line::from(vec![
                    Span::styled(tool.clone(), Style::default().fg(Color::Magenta)),
                    Span::raw(": "),
                    Span::raw(args.clone()),
                ])
            })
            .collect();

        let paragraph = Paragraph::new(Text::from(lines)).wrap(Wrap { trim: false });
        f.render_widget(paragraph, inner);
    }
}

impl Default for ToolLogPane {
    fn default() -> Self {
        Self::new()
    }
}

pub struct BrowserPane {
    url: String,
    title: String,
    scroll: usize,
}

impl BrowserPane {
    pub fn new() -> Self {
        Self {
            url: "about:blank".to_string(),
            title: "".to_string(),
            scroll: 0,
        }
    }

    pub fn set_url(&mut self, url: String) {
        self.url = url;
    }

    pub fn set_title(&mut self, title: String) {
        self.title = title;
    }

    pub fn scroll_down(&mut self) {
        self.scroll += 1;
    }

    pub fn scroll_up(&mut self) {
        if self.scroll > 0 {
            self.scroll -= 1;
        }
    }

    pub fn render(&mut self, f: &mut Frame, area: Rect, focused: bool) {
        let title = if focused { "Browser ●" } else { "Browser" };

        let block = Block::default()
            .borders(Borders::ALL)
            .title(title)
            .title_style(if focused {
                Style::default().add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            });

        let inner = block.inner(area);
        f.render_widget(block, area);

        let content = vec![
            Line::from(vec![
                Span::styled("URL: ", Style::default().fg(Color::Cyan)),
                Span::raw(self.url.clone()),
            ]),
            Line::from(vec![
                Span::styled("Title: ", Style::default().fg(Color::Cyan)),
                Span::raw(self.title.clone()),
            ]),
            Line::from(""),
            Line::from("Browser panel - connect Chrome MCP for full functionality"),
        ];

        let paragraph = Paragraph::new(Text::from(content)).wrap(Wrap { trim: false });
        f.render_widget(paragraph, inner);
    }
}

impl Default for BrowserPane {
    fn default() -> Self {
        Self::new()
    }
}
