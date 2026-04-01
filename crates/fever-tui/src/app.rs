use std::io;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use crossterm::{
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyModifiers,
    },
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::Frame;
use ratatui::backend::CrosstermBackend;

use crate::animation::AnimationState;
use crate::components::message::{MessageBubble, MessageRole};
use crate::components::tool_card::ToolCard;
use crate::event::{Command, Message, Screen};
use crate::render::render_frame;
use crate::slash::SlashCommand;
use crate::theme::Theme;

/// Central application state — single source of truth for the entire TUI.
pub struct AppState {
    // Navigation
    pub screen: Screen,
    pub should_quit: bool,

    // Theme & animations
    pub theme: Theme,
    pub animations: AnimationState,
    pub last_tick: Instant,
    pub tick_rate: Duration,

    // Provider info (read from config)
    pub provider_name: String,
    pub model_name: String,
    pub workspace: String,

    // Chat screen state
    pub streaming: bool,
    pub streaming_buffer: String,
    pub input_buffer: String,
    pub messages: Vec<MessageBubble>,
    pub tool_calls: Vec<ToolCard>,

    // Settings screen
    pub settings_tab: usize,

    // Onboarding screen
    pub onboarding_step: usize,
    pub onboarding_selection: usize,
}

impl AppState {
    /// Create a new AppState with auto-detected theme and sensible defaults.
    pub fn new() -> Self {
        let workspace = std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("~"))
            .to_string_lossy()
            .to_string();

        Self {
            screen: Screen::Home,
            should_quit: false,
            theme: Theme::detect(),
            animations: AnimationState::new(),
            last_tick: Instant::now(),
            tick_rate: Duration::from_millis(250),
            provider_name: "none".to_string(),
            model_name: "none".to_string(),
            workspace,
            streaming: false,
            streaming_buffer: String::new(),
            input_buffer: String::new(),
            messages: Vec::new(),
            tool_calls: Vec::new(),
            settings_tab: 0,
            onboarding_step: 0,
            onboarding_selection: 0,
        }
    }

    /// Try to load provider/model from fever-config.
    pub fn load_config(&mut self) {
        if let Some(config_dir) = dirs::config_dir() {
            let config_path = config_dir.join("fevercode").join("config.toml");
            if config_path.exists() {
                if let Ok(contents) = std::fs::read_to_string(&config_path) {
                    if let Ok(value) = contents.parse::<toml::Value>() {
                        if let Some(provider) = value
                            .get("defaults")
                            .and_then(|d| d.get("provider"))
                            .and_then(|v| v.as_str())
                        {
                            self.provider_name = provider.to_string();
                        }
                        if let Some(model) = value
                            .get("defaults")
                            .and_then(|d| d.get("model"))
                            .and_then(|v| v.as_str())
                        {
                            self.model_name = model.to_string();
                        }
                    }
                }
            }
        }
    }

    // ── Elm update ──────────────────────────────────────────────────────

    /// Process a message and return side-effect commands.
    pub fn update(&mut self, msg: Message) -> Vec<Command> {
        match msg {
            Message::Key(key) => self.handle_key(key),
            Message::Tick => {
                let delta = self.last_tick.elapsed();
                self.animations.tick(delta);
                self.last_tick = Instant::now();
                vec![]
            }
            Message::Navigate(screen) => {
                self.screen = screen;
                vec![]
            }
            Message::InputChanged(text) => {
                self.input_buffer = text;
                vec![]
            }
            Message::InputSubmitted => {
                let content = self.input_buffer.clone();
                if content.is_empty() {
                    return vec![];
                }

                if content.starts_with('/') {
                    if let Some(cmd) = SlashCommand::parse(&content) {
                        self.input_buffer.clear();
                        return self.handle_slash_command(cmd);
                    }
                }

                self.messages
                    .push(MessageBubble::new(MessageRole::User, content));
                self.input_buffer.clear();

                vec![Command::SendMessage {
                    content: self.messages.last().unwrap().content.clone(),
                }]
            }
            Message::StreamChunk { content } => {
                if !self.streaming {
                    self.streaming = true;
                    self.streaming_buffer.clear();
                    self.messages
                        .push(MessageBubble::new(MessageRole::Assistant, String::new()));
                }
                if let Some(last) = self.messages.last_mut() {
                    last.append(&content);
                }
                self.streaming_buffer.push_str(&content);
                vec![]
            }
            Message::StreamEnd => {
                self.streaming = false;
                if let Some(last) = self.messages.last_mut() {
                    last.finish_stream();
                }
                self.streaming_buffer.clear();
                vec![]
            }
            Message::ToolCallStarted { tool, args } => {
                self.tool_calls.push(ToolCard::new(tool, args));
                vec![]
            }
            Message::ToolCallCompleted { tool, result } => {
                for card in &mut self.tool_calls {
                    if card.tool_name == tool && card.is_running() {
                        card.complete(result);
                        break;
                    }
                }
                vec![]
            }
            Message::ToolCallFailed { tool, error } => {
                for card in &mut self.tool_calls {
                    if card.tool_name == tool && card.is_running() {
                        card.fail(error);
                        break;
                    }
                }
                vec![]
            }
            Message::SlashCommand(cmd) => self.handle_slash_command(cmd),
            Message::Quit => {
                self.should_quit = true;
                vec![]
            }
        }
    }

    // ── Key dispatch ────────────────────────────────────────────────────

    fn handle_key(&mut self, key: KeyEvent) -> Vec<Command> {
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            self.should_quit = true;
            return vec![];
        }

        match &self.screen {
            Screen::Home => self.handle_home_key(key),
            Screen::Chat => self.handle_chat_key(key),
            Screen::Settings => self.handle_settings_key(key),
            Screen::Onboarding { .. } => self.handle_onboarding_key(key),
        }
    }

    fn handle_home_key(&mut self, key: KeyEvent) -> Vec<Command> {
        match key.code {
            KeyCode::Enter => {
                self.screen = Screen::Chat;
            }
            KeyCode::Char('s') | KeyCode::Char('S') => {
                self.screen = Screen::Settings;
            }
            KeyCode::Char('/') => {
                self.screen = Screen::Chat;
                self.input_buffer = "/".to_string();
            }
            KeyCode::Char('q') | KeyCode::Esc => {
                self.should_quit = true;
            }
            _ => {}
        }
        vec![]
    }

    fn handle_chat_key(&mut self, key: KeyEvent) -> Vec<Command> {
        match key.code {
            KeyCode::Enter => self.update(Message::InputSubmitted),
            KeyCode::Char(c) => {
                self.input_buffer.push(c);
                vec![]
            }
            KeyCode::Backspace => {
                self.input_buffer.pop();
                vec![]
            }
            KeyCode::Esc => {
                self.screen = Screen::Home;
                vec![]
            }
            _ => vec![],
        }
    }

    fn handle_settings_key(&mut self, key: KeyEvent) -> Vec<Command> {
        match key.code {
            KeyCode::Esc => {
                self.screen = Screen::Home;
            }
            KeyCode::Tab => {
                self.settings_tab = (self.settings_tab + 1) % 4;
            }
            KeyCode::BackTab => {
                self.settings_tab = (self.settings_tab + 3) % 4;
            }
            _ => {}
        }
        vec![]
    }

    fn handle_onboarding_key(&mut self, key: KeyEvent) -> Vec<Command> {
        match key.code {
            KeyCode::Esc => {
                self.screen = Screen::Home;
            }
            KeyCode::Up => {
                if self.onboarding_selection > 0 {
                    self.onboarding_selection -= 1;
                }
            }
            KeyCode::Down => {
                if self.onboarding_selection < 5 {
                    self.onboarding_selection += 1;
                }
            }
            KeyCode::Enter => {
                self.screen = Screen::Home;
            }
            _ => {}
        }
        vec![]
    }

    // ── Slash commands ─────────────────────────────────────────────────

    fn handle_slash_command(&mut self, cmd: SlashCommand) -> Vec<Command> {
        match cmd {
            SlashCommand::Help => {
                let help = crate::slash::help::help_text();
                self.messages
                    .push(MessageBubble::new(MessageRole::System, help));
            }
            SlashCommand::Clear => {
                self.messages.clear();
                self.tool_calls.clear();
                self.streaming = false;
                self.streaming_buffer.clear();
            }
            SlashCommand::Settings => {
                self.screen = Screen::Settings;
            }
            SlashCommand::Quit => {
                self.should_quit = true;
            }
            SlashCommand::Version => {
                let ver = env!("CARGO_PKG_VERSION");
                self.messages.push(MessageBubble::new(
                    MessageRole::System,
                    format!("Fever v{}", ver),
                ));
            }
            SlashCommand::Status => {
                let status = format!(
                    "Provider: {} | Model: {} | Workspace: {} | Messages: {}",
                    self.provider_name,
                    self.model_name,
                    self.workspace,
                    self.messages.len()
                );
                self.messages
                    .push(MessageBubble::new(MessageRole::System, status));
            }
            SlashCommand::Model(name) => {
                if !name.is_empty() {
                    self.model_name = name.clone();
                    self.messages.push(MessageBubble::new(
                        MessageRole::System,
                        format!("Model switched to: {}", name),
                    ));
                } else {
                    self.messages.push(MessageBubble::new(
                        MessageRole::System,
                        format!("Current model: {}", self.model_name),
                    ));
                }
            }
            SlashCommand::Role(name) => {
                self.messages.push(MessageBubble::new(
                    MessageRole::System,
                    if name.is_empty() {
                        "Usage: /role <name>".to_string()
                    } else {
                        format!("Role set to: {}", name)
                    },
                ));
            }
            SlashCommand::Provider(name) => {
                if !name.is_empty() {
                    self.provider_name = name.clone();
                    self.messages.push(MessageBubble::new(
                        MessageRole::System,
                        format!("Provider switched to: {}", name),
                    ));
                } else {
                    self.messages.push(MessageBubble::new(
                        MessageRole::System,
                        format!("Current provider: {}", self.provider_name),
                    ));
                }
            }
        }
        vec![]
    }

    // ── Rendering ──────────────────────────────────────────────────────

    /// Dispatch to the current screen's render function.
    pub fn render_screen(&mut self, f: &mut Frame, area: ratatui::layout::Rect) {
        match &self.screen {
            Screen::Home => crate::screens::home::render(f, area, self),
            Screen::Chat => crate::screens::chat::render(f, area, self),
            Screen::Settings => crate::screens::settings::render(f, area, self),
            Screen::Onboarding { .. } => crate::screens::onboarding::render(f, area, self),
        }
    }

    // ── Main event loop ────────────────────────────────────────────────

    /// Run the TUI application. Sets up terminal, runs event loop, restores terminal.
    pub fn run(&mut self) -> anyhow::Result<()> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

        let backend = CrosstermBackend::new(stdout);
        let mut terminal = ratatui::Terminal::new(backend)?;

        let result = self.run_loop(&mut terminal);

        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;

        result.map_err(Into::into)
    }

    fn run_loop<B: ratatui::backend::Backend>(
        &mut self,
        terminal: &mut ratatui::Terminal<B>,
    ) -> io::Result<()> {
        self.last_tick = Instant::now();

        loop {
            terminal.draw(|f| render_frame(f, self))?;

            let timeout = self.tick_rate.saturating_sub(self.last_tick.elapsed());

            if event::poll(timeout)? {
                if let Event::Key(key) = event::read()? {
                    self.update(Message::Key(key));
                }
            }

            if self.last_tick.elapsed() >= self.tick_rate {
                self.update(Message::Tick);
            }

            if self.should_quit {
                return Ok(());
            }
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
