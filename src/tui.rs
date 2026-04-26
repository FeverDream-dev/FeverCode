use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Terminal,
};
use std::{io, time::Duration};

use crate::safety::ApprovalMode;

#[derive(Debug, Clone, PartialEq)]
enum AppMode {
    Chat,
    Plan,
    Doctor,
    Diff,
    Approval,
}

#[derive(Debug, Clone)]
struct ChatLine {
    role: String,
    content: String,
}

pub struct App {
    workspace_root: String,
    mode: ApprovalMode,
    theme: String,
    provider_name: String,
    model_name: String,
    app_mode: AppMode,
    input: String,
    cursor: usize,
    chat_lines: Vec<ChatLine>,
    scroll_offset: u16,
    status_message: String,
    show_help: bool,
}

impl App {
    pub fn new(
        workspace_root: String,
        mode: ApprovalMode,
        theme: String,
        provider_name: String,
        model_name: String,
    ) -> Self {
        let chat_lines = vec![ChatLine {
            role: "system".to_string(),
            content: "FeverCode portal initialized. Type a message or /help for commands."
                .to_string(),
        }];

        Self {
            workspace_root,
            mode,
            theme,
            provider_name,
            model_name,
            app_mode: AppMode::Chat,
            input: String::new(),
            cursor: 0,
            chat_lines,
            scroll_offset: 0,
            status_message: String::new(),
            show_help: false,
        }
    }

    fn handle_input(&mut self, key: KeyCode) -> Option<String> {
        match key {
            KeyCode::Enter => {
                let text = self.input.trim().to_string();
                if text.is_empty() {
                    return None;
                }
                self.input.clear();
                self.cursor = 0;

                if text.starts_with('/') {
                    return Some(text);
                }

                self.chat_lines.push(ChatLine {
                    role: "user".to_string(),
                    content: text.clone(),
                });
                self.chat_lines.push(ChatLine {
                    role: "assistant".to_string(),
                    content: format!("Processing: {}", text),
                });
                None
            }
            KeyCode::Char(c) => {
                self.input.insert(self.cursor, c);
                self.cursor += 1;
                None
            }
            KeyCode::Backspace => {
                if self.cursor > 0 {
                    self.cursor -= 1;
                    self.input.remove(self.cursor);
                }
                None
            }
            KeyCode::Delete => {
                if self.cursor < self.input.len() {
                    self.input.remove(self.cursor);
                }
                None
            }
            KeyCode::Left => {
                if self.cursor > 0 {
                    self.cursor -= 1;
                }
                None
            }
            KeyCode::Right => {
                if self.cursor < self.input.len() {
                    self.cursor += 1;
                }
                None
            }
            KeyCode::Home => {
                self.cursor = 0;
                None
            }
            KeyCode::End => {
                self.cursor = self.input.len();
                None
            }
            KeyCode::Up => {
                if self.scroll_offset > 0 {
                    self.scroll_offset -= 1;
                }
                None
            }
            KeyCode::Down => {
                self.scroll_offset += 1;
                None
            }
            KeyCode::PageUp => {
                self.scroll_offset = self.scroll_offset.saturating_sub(10);
                None
            }
            KeyCode::PageDown => {
                self.scroll_offset += 10;
                None
            }
            _ => None,
        }
    }

    fn handle_command(&mut self, cmd: &str) {
        let parts: Vec<&str> = cmd.splitn(2, ' ').collect();
        let command = parts[0];
        let args = parts.get(1).copied().unwrap_or("");

        match command {
            "/exit" | "/quit" | "/q" => {
                self.status_message = "Exiting...".to_string();
            }
            "/help" | "/?" => {
                self.show_help = !self.show_help;
            }
            "/plan" => {
                self.app_mode = AppMode::Plan;
                self.chat_lines.push(ChatLine {
                    role: "system".to_string(),
                    content: "Plan mode activated. Describe what you want to plan.".to_string(),
                });
            }
            "/run" => {
                self.app_mode = AppMode::Chat;
                self.chat_lines.push(ChatLine {
                    role: "system".to_string(),
                    content: "Run mode.".to_string(),
                });
            }
            "/spray" => {
                self.mode = ApprovalMode::Spray;
                self.chat_lines.push(ChatLine {
                    role: "system".to_string(),
                    content: "Spray mode activated. Autonomous workspace edits enabled."
                        .to_string(),
                });
            }
            "/ask" => {
                self.mode = ApprovalMode::Ask;
                self.chat_lines.push(ChatLine {
                    role: "system".to_string(),
                    content: "Ask mode restored. All actions require approval.".to_string(),
                });
            }
            "/auto" => {
                self.mode = ApprovalMode::Auto;
                self.chat_lines.push(ChatLine {
                    role: "system".to_string(),
                    content: "Auto mode. Safe edits proceed automatically.".to_string(),
                });
            }
            "/doctor" => {
                self.app_mode = AppMode::Doctor;
                self.chat_lines.push(ChatLine {
                    role: "system".to_string(),
                    content: format!("Doctor check for: {}", self.workspace_root),
                });
            }
            "/diff" => {
                self.app_mode = AppMode::Diff;
            }
            "/approve" => {
                self.app_mode = AppMode::Approval;
            }
            "/clear" => {
                self.chat_lines.clear();
                self.scroll_offset = 0;
            }
            "/status" => {
                self.chat_lines.push(ChatLine {
                    role: "system".to_string(),
                    content: format!(
                        "Workspace: {}\nMode: {}\nProvider: {}\nModel: {}",
                        self.workspace_root, self.mode, self.provider_name, self.model_name
                    ),
                });
            }
            "/model" => {
                if !args.is_empty() {
                    self.model_name = args.to_string();
                    self.chat_lines.push(ChatLine {
                        role: "system".to_string(),
                        content: format!("Model set to: {}", args),
                    });
                }
            }
            _ => {
                self.chat_lines.push(ChatLine {
                    role: "system".to_string(),
                    content: format!("Unknown command: {}. Type /help for commands.", command),
                });
            }
        }
    }
}

pub async fn launch(
    root: crate::workspace::Workspace,
    cfg: crate::config::FeverConfig,
) -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(
        root.root.display().to_string(),
        cfg.safety.mode,
        cfg.ui.theme.clone(),
        cfg.providers.default.name.clone(),
        cfg.providers.default.model.clone().unwrap_or_default(),
    );

    let result = run_loop(&mut terminal, &mut app);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    result
}

fn run_loop(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, app: &mut App) -> Result<()> {
    loop {
        terminal.draw(|frame| draw(frame, app))?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }

                match key.code {
                    KeyCode::Esc => {
                        if app.show_help {
                            app.show_help = false;
                        } else {
                            break;
                        }
                    }
                    KeyCode::Char('q')
                        if key.modifiers.contains(crossterm::event::KeyModifiers::NONE)
                            && app.input.is_empty() =>
                    {
                        break;
                    }
                    _ => {
                        if let Some(cmd) = app.handle_input(key.code) {
                            if cmd == "/exit" || cmd == "/quit" || cmd == "/q" {
                                break;
                            }
                            app.handle_command(&cmd);
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

fn draw(frame: &mut ratatui::Frame, app: &mut App) {
    let area = frame.area();

    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4),
            Constraint::Min(8),
            Constraint::Length(3),
        ])
        .split(area);

    draw_header(frame, app, main_layout[0]);
    draw_body(frame, app, main_layout[1]);
    draw_input(frame, app, main_layout[2]);
}

fn draw_header(frame: &mut ratatui::Frame, app: &App, area: Rect) {
    let mode_color = match app.mode {
        ApprovalMode::Ask => Color::Yellow,
        ApprovalMode::Auto => Color::Cyan,
        ApprovalMode::Spray => Color::Magenta,
    };

    let mode_str = format!("{}", app.mode);
    let title = Line::from(vec![
        Span::styled(
            "  FeverCode Portal",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("  "),
        Span::styled(
            format!("mode:{}", mode_str),
            Style::default().fg(mode_color),
        ),
        Span::raw("  "),
        Span::styled(
            format!("{}:{}", app.provider_name, app.model_name),
            Style::default().fg(Color::DarkGray),
        ),
    ]);

    let path_line = Line::from(vec![
        Span::styled("  ", Style::default()),
        Span::styled(&app.workspace_root, Style::default().fg(Color::DarkGray)),
    ]);

    let mode_desc = match app.mode {
        ApprovalMode::Ask => "All actions require approval",
        ApprovalMode::Auto => "Safe edits auto-approved",
        ApprovalMode::Spray => "Autonomous workspace edits",
    };

    let desc_line = Line::from(vec![
        Span::styled("  ", Style::default()),
        Span::styled(mode_desc, Style::default().fg(Color::DarkGray)),
    ]);

    let header = Paragraph::new(vec![title, path_line, desc_line]).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray))
            .title(" Ra "),
    );
    frame.render_widget(header, area);
}

fn draw_body(frame: &mut ratatui::Frame, app: &mut App, area: Rect) {
    if app.show_help {
        draw_help(frame, area);
        return;
    }

    let body_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(20), Constraint::Length(28)])
        .split(area);

    draw_chat(frame, app, body_layout[0]);
    draw_sidebar(frame, app, body_layout[1]);
}

fn draw_chat(frame: &mut ratatui::Frame, app: &mut App, area: Rect) {
    let lines: Vec<Line> = app
        .chat_lines
        .iter()
        .flat_map(|chat| {
            let (prefix, color) = match chat.role.as_str() {
                "user" => ("You", Color::Cyan),
                "assistant" => ("Ptah", Color::Green),
                "system" => ("Portal", Color::Yellow),
                _ => ("?", Color::White),
            };

            let content_lines: Vec<&str> = chat.content.lines().collect();
            content_lines
                .into_iter()
                .enumerate()
                .map(move |(i, line)| {
                    if i == 0 {
                        Line::from(vec![
                            Span::styled(
                                format!("{} ", prefix),
                                Style::default().fg(color).add_modifier(Modifier::BOLD),
                            ),
                            Span::styled(line.to_string(), Style::default()),
                        ])
                    } else {
                        Line::from(format!("   {}", line))
                    }
                })
                .collect::<Vec<_>>()
        })
        .collect();

    let total_lines = lines.len() as u16;
    let visible_height = area.height.saturating_sub(2);
    let max_scroll = total_lines.saturating_sub(visible_height);
    if app.scroll_offset > max_scroll {
        app.scroll_offset = max_scroll;
    }

    let chat = Paragraph::new(lines)
        .wrap(Wrap { trim: false })
        .scroll((app.scroll_offset, 0))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray))
                .title(" Portal "),
        );
    frame.render_widget(chat, area);
}

fn draw_sidebar(frame: &mut ratatui::Frame, app: &App, area: Rect) {
    let sidebar_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(12), Constraint::Min(5)])
        .split(area);

    let agents = crate::agents::builtins();
    let agent_lines: Vec<Line> = agents
        .iter()
        .map(|a| {
            let enabled =
                app.mode != ApprovalMode::Ask || a.id == "ra-planner" || a.id == "anubis-guardian";
            let style = if enabled {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(Color::DarkGray)
            };
            Line::from(Span::styled(format!(" {}", a.id), style))
        })
        .collect();

    let agent_widget = Paragraph::new(agent_lines).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray))
            .title(" Agents "),
    );
    frame.render_widget(agent_widget, sidebar_layout[0]);

    let tools = [
        "read_file",
        "write_file",
        "list_files",
        "search_text",
        "run_shell",
        "git_status",
        "git_diff",
        "git_checkpoint",
    ];
    let tool_lines: Vec<Line> = tools
        .iter()
        .map(|t| {
            Line::from(Span::styled(
                format!(" {}", t),
                Style::default().fg(Color::DarkGray),
            ))
        })
        .collect();

    let tool_widget = Paragraph::new(tool_lines).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray))
            .title(" Tools "),
    );
    frame.render_widget(tool_widget, sidebar_layout[1]);
}

fn draw_help(frame: &mut ratatui::Frame, area: Rect) {
    let help_lines = vec![
        Line::from(Span::styled(
            " FeverCode Commands",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled(" /plan ", Style::default().fg(Color::Cyan)),
            Span::raw("  Switch to plan mode"),
        ]),
        Line::from(vec![
            Span::styled(" /run ", Style::default().fg(Color::Cyan)),
            Span::raw("   Switch to run mode"),
        ]),
        Line::from(vec![
            Span::styled(" /spray ", Style::default().fg(Color::Magenta)),
            Span::raw(" Enable spray mode (autonomous)"),
        ]),
        Line::from(vec![
            Span::styled(" /ask ", Style::default().fg(Color::Yellow)),
            Span::raw("   Switch to ask mode (safe)"),
        ]),
        Line::from(vec![
            Span::styled(" /auto ", Style::default().fg(Color::Cyan)),
            Span::raw("  Switch to auto mode"),
        ]),
        Line::from(vec![
            Span::styled(" /doctor ", Style::default().fg(Color::Green)),
            Span::raw("Run health checks"),
        ]),
        Line::from(vec![
            Span::styled(" /diff ", Style::default().fg(Color::Cyan)),
            Span::raw("  View pending diffs"),
        ]),
        Line::from(vec![
            Span::styled(" /approve ", Style::default().fg(Color::Green)),
            Span::raw("Approval queue"),
        ]),
        Line::from(vec![
            Span::styled(" /status ", Style::default().fg(Color::Cyan)),
            Span::raw("Show current status"),
        ]),
        Line::from(vec![
            Span::styled(" /model ", Style::default().fg(Color::Cyan)),
            Span::raw(" Change model"),
        ]),
        Line::from(vec![
            Span::styled(" /clear ", Style::default().fg(Color::Cyan)),
            Span::raw(" Clear chat"),
        ]),
        Line::from(vec![
            Span::styled(" /exit ", Style::default().fg(Color::Red)),
            Span::raw("  Exit FeverCode (or q)"),
        ]),
        Line::from(""),
        Line::from(" Type a message to send to the AI assistant."),
        Line::from(" Press Esc to close this help."),
    ];

    let help = Paragraph::new(help_lines).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow))
            .title(" Help "),
    );
    frame.render_widget(help, area);
}

fn draw_input(frame: &mut ratatui::Frame, app: &App, area: Rect) {
    let mode_indicator = match app.mode {
        ApprovalMode::Ask => "?",
        ApprovalMode::Auto => ">",
        ApprovalMode::Spray => "!",
    };

    let display_input = if app.input.is_empty() {
        "Type a message or /help for commands...".to_string()
    } else {
        let mut s = app.input.clone();
        s.insert(app.cursor, '|');
        s
    };

    let input_color = if app.input.is_empty() {
        Color::DarkGray
    } else {
        Color::White
    };

    let input = Paragraph::new(Line::from(vec![
        Span::styled(
            format!(" {} ", mode_indicator),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(display_input, Style::default().fg(input_color)),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray)),
    );
    frame.render_widget(input, area);
}
