use crate::widgets::{ChatPane, PlanPane, TaskPane, ToolLogPane, BrowserPane};
use crate::ui::FeverUI;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use fever_core::{Plan, TaskStatus, Todo};
use std::io;
use std::time::Duration;

pub struct TuiConfig {
    pub tick_rate: Duration,
    pub auto_scroll: bool,
}

impl Default for TuiConfig {
    fn default() -> Self {
        Self {
            tick_rate: Duration::from_millis(250),
            auto_scroll: true,
        }
    }
}

pub struct FeverTui {
    ui: FeverUI,
    config: TuiConfig,
    should_quit: bool,
}

impl FeverTui {
    pub fn new(config: TuiConfig) -> Self {
        let ui = FeverUI::new();
        Self {
            ui,
            config,
            should_quit: false,
        }
    }

    pub fn run(&mut self) -> io::Result<()> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

        let backend = ratatui::backend::CrosstermBackend::new(stdout);
        let mut terminal = ratatui::Terminal::new(backend)?;

        let result = self.run_app(&mut terminal);

        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;

        result
    }

    fn run_app<B: ratatui::backend::Backend>(
        &mut self,
        terminal: &mut ratatui::Terminal<B>,
    ) -> io::Result<()> {
        let mut last_tick = std::time::Instant::now();

        loop {
            terminal.draw(|f| self.ui.render(f))?;

            let timeout = self
                .config
                .tick_rate
                .saturating_sub(last_tick.elapsed());

            if crossterm::event::poll(timeout)? {
                if let Event::Key(key) = event::read()? {
                    self.handle_key(key);
                }
            }

            if last_tick.elapsed() >= self.config.tick_rate {
                self.on_tick();
                last_tick = std::time::Instant::now();
            }

            if self.should_quit {
                return Ok(());
            }
        }
    }

    fn handle_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => self.should_quit = true,
            KeyCode::Char('1') => self.ui.set_focus(crate::ui::Focus::Chat),
            KeyCode::Char('2') => self.ui.set_focus(crate::ui::Focus::Plan),
            KeyCode::Char('3') => self.ui.set_focus(crate::ui::Focus::Tasks),
            KeyCode::Char('4') => self.ui.set_focus(crate::ui::Focus::ToolLog),
            KeyCode::Char('5') => self.ui.set_focus(crate::ui::Focus::Browser),
            KeyCode::Down => self.ui.scroll_down(),
            KeyCode::Up => self.ui.scroll_up(),
            KeyCode::Enter => {
                let input = self.ui.chat.get_input_buffer();
                if !input.is_empty() {
                    self.ui.chat.add_message("user".to_string(), input.to_string());
                    self.ui.chat.clear_input_buffer();
                }
            }
            KeyCode::Backspace => {
                self.ui.chat.backspace();
            }
            KeyCode::Char(c) => {
                self.ui.chat.type_char(c);
            }
            _ => {}
        }
    }

    fn on_tick(&mut self) {
    }

    pub fn add_message(&mut self, role: String, content: String) {
        self.ui.chat.add_message(role, content);
    }

    pub fn set_plan(&mut self, plan: Plan) {
        self.ui.plan.set_plan(plan);
    }

    pub fn add_todo(&mut self, todo: Todo) {
        self.ui.tasks.add_todo(todo);
    }

    pub fn update_task_status(&mut self, task_id: &str, status: TaskStatus) {
        self.ui.tasks.update_status(task_id, status);
    }

    pub fn log_tool(&mut self, tool_name: String, args: String) {
        self.ui.tool_log.log(tool_name, args);
    }

    pub fn set_browser_url(&mut self, url: String) {
        self.ui.browser.set_url(url);
    }

    pub fn set_status(&mut self, status: String) {
        self.ui.set_status(status);
    }
}
