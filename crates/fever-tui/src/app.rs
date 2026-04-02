use std::collections::VecDeque;
use std::io::{self, IsTerminal};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

use crossterm::{
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyModifiers,
        MouseButton, MouseEvent, MouseEventKind,
    },
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::Frame;
use ratatui::backend::CrosstermBackend;
use tokio::sync::mpsc;

use crate::agent_bridge::AgentHandle;
use crate::animation::AnimationState;
use crate::components::message::{MessageBubble, MessageRole};
use crate::components::tool_card::ToolCard;
use crate::event::{Command, Message, Screen};
use crate::render::render_frame;
use crate::slash::SlashCommand;
use crate::theme::Theme;

// ─────────────────────────────────────────────────────────────────────
// Known providers and models
// ─────────────────────────────────────────────────────────────────────

pub const KNOWN_PROVIDERS: &[&str] = &[
    "openai",
    "anthropic",
    "gemini",
    "groq",
    "deepseek",
    "mistral",
    "together",
    "openrouter",
    "fireworks",
    "perplexity",
    "ollama",
];

pub fn known_models_for_provider(provider: &str) -> Vec<&'static str> {
    match provider {
        "openai" => vec![
            "gpt-4o",
            "gpt-4o-mini",
            "gpt-4-turbo",
            "gpt-4",
            "gpt-3.5-turbo",
            "o1",
            "o1-mini",
            "o3-mini",
        ],
        "anthropic" => vec![
            "claude-sonnet-4-20250514",
            "claude-3-5-sonnet-20241022",
            "claude-3-5-haiku-20241022",
            "claude-3-opus-20240229",
        ],
        "gemini" => vec!["gemini-2.0-flash", "gemini-1.5-pro", "gemini-1.5-flash"],
        "groq" => vec![
            "llama-3.3-70b-versatile",
            "llama-3.1-8b-instant",
            "mixtral-8x7b-32768",
        ],
        "deepseek" => vec!["deepseek-chat", "deepseek-coder", "deepseek-reasoner"],
        "mistral" => vec![
            "mistral-large-latest",
            "mistral-medium-latest",
            "mistral-small-latest",
            "codestral-latest",
        ],
        "together" => vec![
            "meta-llama/Llama-3-70b-chat-hf",
            "mistralai/Mixtral-8x7B-Instruct-v0.1",
            "togethercomputer/RedPajama-INCITE-7B-Chat",
        ],
        "openrouter" => vec![
            "openai/gpt-4o",
            "anthropic/claude-sonnet-4-20250514",
            "google/gemini-2.0-flash",
            "meta-llama/llama-3.3-70b-instruct",
        ],
        "fireworks" => vec![
            "accounts/fireworks/models/llama-v3p1-70b-instruct",
            "accounts/fireworks/models/mixtral-8x7b-instruct",
        ],
        "perplexity" => vec!["sonar-pro", "sonar"],
        "ollama" => vec![
            "llama3.3",
            "codellama",
            "mistral",
            "mixtral",
            "deepseek-coder",
            "phi3",
            "gemma2",
        ],
        _ => vec!["default"],
    }
}

// ─────────────────────────────────────────────────────────────────────
// AppState — single source of truth for the entire TUI
// ─────────────────────────────────────────────────────────────────────

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
    pub loading: bool,
    pub streaming_buffer: String,
    pub input_buffer: String,
    pub messages: Vec<MessageBubble>,
    pub tool_calls: Vec<ToolCard>,
    pub scroll_offset: u16,

    // Command palette (Ctrl+K)
    pub show_command_palette: bool,
    pub palette_query: String,
    pub palette_selection: usize,

    // Help overlay (?)
    pub show_help: bool,

    // Input history (up/down recall)
    pub input_history: VecDeque<String>,
    pub history_index: Option<usize>,

    // Sidebar (Ctrl+B)
    pub show_sidebar: bool,
    pub sidebar_selection: usize,

    // Settings screen
    pub settings_tab: usize,
    pub settings_provider_cursor: usize,
    pub settings_model_cursor: usize,
    pub settings_theme_cursor: usize,
    pub settings_behavior_cursor: usize,
    pub auto_scroll: bool,
    pub show_thinking: bool,
    pub temperature: f32,
    pub max_tokens: u32,

    // Agent bridge (optional — set by CLI when provider is configured)
    pub agent: Option<Arc<dyn AgentHandle>>,
    pub cancel_token: Option<tokio_util::sync::CancellationToken>,

    // Onboarding screen
    pub onboarding_step: usize,
    pub onboarding_selection: usize,

    // Session persistence
    pub session_id: String,

    pub notification: Option<String>,
    pub notification_tick: usize,

    pub terminal_size: (u16, u16),
}

impl AppState {
    /// Create a new AppState with auto-detected theme and sensible defaults.
    pub fn new() -> Self {
        let workspace = std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("~"))
            .to_string_lossy()
            .to_string();

        let mut state = Self {
            screen: Screen::Home,
            should_quit: false,
            theme: Theme::detect(),
            animations: AnimationState::new(),
            last_tick: Instant::now(),
            tick_rate: Duration::from_millis(100),
            provider_name: "none".to_string(),
            model_name: "none".to_string(),
            workspace,
            streaming: false,
            loading: false,
            streaming_buffer: String::new(),
            input_buffer: String::new(),
            messages: Vec::new(),
            tool_calls: Vec::new(),
            scroll_offset: 0,
            show_command_palette: false,
            palette_query: String::new(),
            palette_selection: 0,
            show_help: false,
            input_history: VecDeque::new(),
            history_index: None,
            show_sidebar: false,
            sidebar_selection: 0,
            settings_tab: 0,
            settings_provider_cursor: 0,
            settings_model_cursor: 0,
            settings_theme_cursor: 0,
            settings_behavior_cursor: 0,
            auto_scroll: true,
            show_thinking: true,
            temperature: 0.7,
            max_tokens: 4096,
            onboarding_step: 0,
            onboarding_selection: 0,
            agent: None,
            cancel_token: None,
            session_id: format!("session-{}", chrono::Local::now().format("%Y%m%d-%H%M%S")),
            notification: None,
            notification_tick: 0,
            terminal_size: (80, 24),
        };
        state.load_config();
        state
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
                        if let Some(temp) = value
                            .get("defaults")
                            .and_then(|d| d.get("temperature"))
                            .and_then(|v| v.as_float())
                        {
                            self.temperature = temp as f32;
                        }
                        if let Some(max_tok) = value
                            .get("defaults")
                            .and_then(|d| d.get("max_tokens"))
                            .and_then(|v| v.as_integer())
                        {
                            self.max_tokens = max_tok as u32;
                        }
                        if let Some(auto) = value
                            .get("ui")
                            .and_then(|u| u.get("auto_scroll"))
                            .and_then(|v| v.as_bool())
                        {
                            self.auto_scroll = auto;
                        }
                        if let Some(show) = value
                            .get("ui")
                            .and_then(|u| u.get("show_thinking"))
                            .and_then(|v| v.as_bool())
                        {
                            self.show_thinking = show;
                        }
                        if let Some(theme_name) = value
                            .get("ui")
                            .and_then(|u| u.get("theme"))
                            .and_then(|v| v.as_str())
                        {
                            if let Some(t) = Theme::find_by_name(theme_name) {
                                self.theme = t;
                            }
                        }
                    }
                }
            }
        }
    }

    pub fn save_config(&self) {
        if let Some(config_dir) = dirs::config_dir() {
            let config_dir = config_dir.join("fevercode");
            let _ = std::fs::create_dir_all(&config_dir);
            let config_path = config_dir.join("config.toml");

            let mut defaults = toml::value::Table::new();
            defaults.insert(
                "provider".to_string(),
                toml::Value::String(self.provider_name.clone()),
            );
            defaults.insert(
                "model".to_string(),
                toml::Value::String(self.model_name.clone()),
            );
            defaults.insert(
                "temperature".to_string(),
                toml::Value::Float(self.temperature as f64),
            );
            defaults.insert(
                "max_tokens".to_string(),
                toml::Value::Integer(self.max_tokens as i64),
            );

            let mut ui = toml::value::Table::new();
            ui.insert(
                "auto_scroll".to_string(),
                toml::Value::Boolean(self.auto_scroll),
            );
            ui.insert(
                "show_thinking".to_string(),
                toml::Value::Boolean(self.show_thinking),
            );
            ui.insert(
                "theme".to_string(),
                toml::Value::String(self.theme.name.to_string()),
            );

            let mut root = toml::value::Table::new();
            root.insert("defaults".to_string(), toml::Value::Table(defaults));
            root.insert("ui".to_string(), toml::Value::Table(ui));

            let doc = toml::Value::Table(root);
            if let Ok(toml_str) = toml::to_string_pretty(&doc) {
                if let Err(e) = std::fs::write(&config_path, toml_str) {
                    tracing::warn!(path = %config_path.display(), error = %e, "Failed to write config");
                } else {
                    tracing::info!(path = %config_path.display(), "Config saved");
                }
            }
        }
    }

    fn notify(&mut self, text: &str) {
        self.notification = Some(text.to_string());
        self.notification_tick = 0;
    }

    // ── Elm update ──────────────────────────────────────────────────

    /// Process a message and return side-effect commands.
    pub fn update(&mut self, msg: Message) -> Vec<Command> {
        match msg {
            Message::Key(key) => self.handle_key(key),
            Message::Tick => {
                let delta = self.last_tick.elapsed();
                self.animations.tick(delta);
                self.last_tick = Instant::now();
                if self.notification.is_some() {
                    self.notification_tick += 1;
                    if self.notification_tick > 30 {
                        self.notification = None;
                        self.notification_tick = 0;
                    }
                }
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
                self.scroll_offset = 0;
                self.loading = true;

                vec![Command::SendMessage {
                    content: self
                        .messages
                        .last()
                        .map(|m| m.content.clone())
                        .unwrap_or_default(),
                }]
            }
            Message::StreamChunk { content } => {
                if !self.streaming {
                    self.streaming = true;
                    self.loading = false;
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
                self.loading = false;
                if let Some(last) = self.messages.last_mut() {
                    last.finish_stream();
                }
                self.streaming_buffer.clear();
                vec![]
            }
            Message::StreamError { message } => {
                self.streaming = false;
                self.loading = false;
                self.messages.push(MessageBubble::new(
                    MessageRole::System,
                    format!("Error: {}", message),
                ));
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
            Message::Mouse(event) => self.handle_mouse_click(event),
            Message::Quit => {
                self.should_quit = true;
                vec![]
            }
        }
    }

    // ── Key dispatch ────────────────────────────────────────────────

    fn handle_key(&mut self, key: KeyEvent) -> Vec<Command> {
        if self.show_help {
            if key.code == KeyCode::Esc || key.code == KeyCode::Char('?') {
                self.show_help = false;
            }
            return vec![];
        }

        if key.modifiers.contains(KeyModifiers::CONTROL) {
            match key.code {
                KeyCode::Char('c') => {
                    if self.streaming || self.loading {
                        if let Some(token) = &self.cancel_token {
                            token.cancel();
                        }
                        self.streaming = false;
                        self.loading = false;
                        self.messages.push(MessageBubble::new(
                            MessageRole::System,
                            "Cancelled.".to_string(),
                        ));
                    } else {
                        self.should_quit = true;
                    }
                    return vec![];
                }
                KeyCode::Char('k') => {
                    self.show_command_palette = !self.show_command_palette;
                    if self.show_command_palette {
                        self.palette_query.clear();
                        self.palette_selection = 0;
                    }
                    return vec![];
                }
                KeyCode::Char('b') => {
                    self.show_sidebar = !self.show_sidebar;
                    return vec![];
                }
                _ => {}
            }
        }

        if key.code == KeyCode::Char('?') && !self.show_command_palette {
            self.show_help = true;
            return vec![];
        }

        // Command palette intercepts all input when open
        if self.show_command_palette {
            return self.handle_palette_key(key);
        }

        // Screen-specific dispatch
        match &self.screen {
            Screen::Home => self.handle_home_key(key),
            Screen::Chat => self.handle_chat_key(key),
            Screen::Settings => self.handle_settings_key(key),
            Screen::Onboarding { .. } => self.handle_onboarding_key(key),
        }
    }

    // ── Command palette handler ─────────────────────────────────────

    fn handle_palette_key(&mut self, key: KeyEvent) -> Vec<Command> {
        match key.code {
            KeyCode::Esc => {
                self.show_command_palette = false;
            }
            KeyCode::Enter => {
                let commands = self.palette_commands();
                if let Some(cmd) = commands.get(self.palette_selection).cloned() {
                    self.show_command_palette = false;
                    return self.handle_slash_command(cmd);
                }
                self.show_command_palette = false;
            }
            KeyCode::Up => {
                if self.palette_selection > 0 {
                    self.palette_selection -= 1;
                }
            }
            KeyCode::Down => {
                let max = self.palette_commands().len().saturating_sub(1);
                if self.palette_selection < max {
                    self.palette_selection += 1;
                }
            }
            KeyCode::Backspace => {
                self.palette_query.pop();
                self.palette_selection = 0;
            }
            KeyCode::Char(c) => {
                self.palette_query.push(c);
                self.palette_selection = 0;
            }
            _ => {}
        }
        vec![]
    }

    /// Returns all slash commands matching the palette query.
    pub fn palette_commands(&self) -> Vec<SlashCommand> {
        let all = vec![
            SlashCommand::Help,
            SlashCommand::Clear,
            SlashCommand::Save,
            SlashCommand::Settings,
            SlashCommand::Status,
            SlashCommand::Version,
            SlashCommand::Model(String::new()),
            SlashCommand::Role(String::new()),
            SlashCommand::Provider(String::new()),
            SlashCommand::Quit,
            SlashCommand::Theme(String::new()),
            SlashCommand::New,
            SlashCommand::Doctor,
            SlashCommand::Session(String::new()),
        ];
        if self.palette_query.is_empty() {
            return all;
        }
        let q = self.palette_query.to_lowercase();
        all.into_iter()
            .filter(|cmd| cmd.name().contains(&q))
            .collect()
    }

    // ── Screen key handlers ─────────────────────────────────────────

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
            KeyCode::Enter => {
                if !self.input_buffer.is_empty()
                    && self.input_history.back() != Some(&self.input_buffer)
                {
                    self.input_history.push_back(self.input_buffer.clone());
                    if self.input_history.len() > 100 {
                        self.input_history.pop_front();
                    }
                }
                self.history_index = None;
                self.update(Message::InputSubmitted)
            }
            KeyCode::Char(c) => {
                self.input_buffer.push(c);
                vec![]
            }
            KeyCode::Backspace => {
                self.input_buffer.pop();
                vec![]
            }
            KeyCode::Up => {
                if let Some(idx) = self.history_index {
                    if idx > 0 {
                        self.history_index = Some(idx - 1);
                    }
                } else if !self.input_history.is_empty() {
                    self.history_index = Some(self.input_history.len() - 1);
                }
                if let Some(idx) = self.history_index {
                    if let Some(entry) = self.input_history.get(idx) {
                        self.input_buffer = entry.clone();
                    }
                }
                vec![]
            }
            KeyCode::Down => {
                if let Some(idx) = self.history_index {
                    if idx + 1 < self.input_history.len() {
                        self.history_index = Some(idx + 1);
                        if let Some(entry) = self.input_history.get(idx + 1) {
                            self.input_buffer = entry.clone();
                        }
                    } else {
                        self.history_index = None;
                        self.input_buffer.clear();
                    }
                }
                vec![]
            }
            KeyCode::Esc => {
                self.screen = Screen::Home;
                vec![]
            }
            KeyCode::PageUp => {
                self.scroll_offset = self.scroll_offset.saturating_add(10);
                vec![]
            }
            KeyCode::PageDown => {
                self.scroll_offset = self.scroll_offset.saturating_sub(10);
                vec![]
            }
            KeyCode::Home => {
                self.scroll_offset = u16::MAX;
                vec![]
            }
            KeyCode::End => {
                self.scroll_offset = 0;
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
                if self.settings_tab == 1 {
                    let models = known_models_for_provider(&self.provider_name);
                    let idx = models
                        .iter()
                        .position(|m| *m == self.model_name)
                        .unwrap_or(0);
                    self.settings_model_cursor = idx;
                }
            }
            KeyCode::BackTab => {
                self.settings_tab = (self.settings_tab + 3) % 4;
                if self.settings_tab == 1 {
                    let models = known_models_for_provider(&self.provider_name);
                    let idx = models
                        .iter()
                        .position(|m| *m == self.model_name)
                        .unwrap_or(0);
                    self.settings_model_cursor = idx;
                }
            }
            KeyCode::Up if self.settings_tab == 0 => {
                let total = KNOWN_PROVIDERS.len();
                if self.settings_provider_cursor > 0 {
                    self.settings_provider_cursor -= 1;
                } else {
                    self.settings_provider_cursor = total.saturating_sub(1);
                }
            }
            KeyCode::Down if self.settings_tab == 0 => {
                let total = KNOWN_PROVIDERS.len();
                if self.settings_provider_cursor < total.saturating_sub(1) {
                    self.settings_provider_cursor += 1;
                } else {
                    self.settings_provider_cursor = 0;
                }
            }
            KeyCode::Enter if self.settings_tab == 0 => {
                if let Some(provider) = KNOWN_PROVIDERS.get(self.settings_provider_cursor) {
                    self.provider_name = provider.to_string();
                    let models = known_models_for_provider(provider);
                    if !models.is_empty() {
                        self.model_name = models[0].to_string();
                    }
                    self.settings_model_cursor = 0;
                    self.save_config();
                    self.notify(&format!("Provider: {} / {}", provider, self.model_name));
                }
            }
            KeyCode::Up if self.settings_tab == 1 => {
                let models = known_models_for_provider(&self.provider_name);
                let total = models.len();
                if total > 0 {
                    if self.settings_model_cursor > 0 {
                        self.settings_model_cursor -= 1;
                    } else {
                        self.settings_model_cursor = total.saturating_sub(1);
                    }
                }
            }
            KeyCode::Down if self.settings_tab == 1 => {
                let models = known_models_for_provider(&self.provider_name);
                let total = models.len();
                if total > 0 {
                    if self.settings_model_cursor < total.saturating_sub(1) {
                        self.settings_model_cursor += 1;
                    } else {
                        self.settings_model_cursor = 0;
                    }
                }
            }
            KeyCode::Enter if self.settings_tab == 1 => {
                let models = known_models_for_provider(&self.provider_name);
                if let Some(model) = models.get(self.settings_model_cursor) {
                    self.model_name = model.to_string();
                    self.save_config();
                    self.notify(&format!("Model: {}", model));
                }
            }
            KeyCode::Up if self.settings_tab == 2 => {
                if self.settings_behavior_cursor > 0 {
                    self.settings_behavior_cursor -= 1;
                } else {
                    self.settings_behavior_cursor = 3;
                }
            }
            KeyCode::Down if self.settings_tab == 2 => {
                if self.settings_behavior_cursor < 3 {
                    self.settings_behavior_cursor += 1;
                } else {
                    self.settings_behavior_cursor = 0;
                }
            }
            KeyCode::Enter if self.settings_tab == 2 => match self.settings_behavior_cursor {
                0 => {
                    self.auto_scroll = !self.auto_scroll;
                    self.save_config();
                    self.notify(&format!(
                        "Auto-scroll: {}",
                        if self.auto_scroll { "on" } else { "off" }
                    ));
                }
                1 => {
                    self.show_thinking = !self.show_thinking;
                    self.save_config();
                    self.notify(&format!(
                        "Show thinking: {}",
                        if self.show_thinking { "on" } else { "off" }
                    ));
                }
                2 => {
                    self.temperature = if self.temperature >= 1.5 {
                        0.0
                    } else {
                        self.temperature + 0.1
                    };
                    self.save_config();
                    self.notify(&format!("Temperature: {:.1}", self.temperature));
                }
                3 => {
                    self.max_tokens = match self.max_tokens {
                        256 => 512,
                        512 => 1024,
                        1024 => 2048,
                        2048 => 4096,
                        4096 => 8192,
                        8192 => 16384,
                        _ => 256,
                    };
                    self.save_config();
                    self.notify(&format!("Max tokens: {}", self.max_tokens));
                }
                _ => {}
            },
            KeyCode::Up if self.settings_tab == 3 => {
                let total = Theme::list_all().len();
                if self.settings_theme_cursor > 0 {
                    self.settings_theme_cursor -= 1;
                } else {
                    self.settings_theme_cursor = total.saturating_sub(1);
                }
            }
            KeyCode::Down if self.settings_tab == 3 => {
                let total = Theme::list_all().len();
                if self.settings_theme_cursor < total.saturating_sub(1) {
                    self.settings_theme_cursor += 1;
                } else {
                    self.settings_theme_cursor = 0;
                }
            }
            KeyCode::Enter if self.settings_tab == 3 => {
                let all = Theme::list_all();
                if let Some(selected) = all.get(self.settings_theme_cursor) {
                    self.theme =
                        Theme::find_by_name(selected.name).unwrap_or_else(|| self.theme.clone());
                    self.save_config();
                    self.notify(&format!("Theme: {}", selected.name));
                }
            }
            _ => {}
        }
        vec![]
    }

    fn handle_mouse_click(&mut self, event: MouseEvent) -> Vec<Command> {
        let (term_w, term_h) = self.terminal_size;
        if event.row >= term_h.saturating_sub(1) {
            return vec![];
        }

        match self.screen {
            Screen::Home => {
                self.screen = Screen::Chat;
                self.input_buffer.clear();
            }
            Screen::Settings => {
                let inner_y = event.row.saturating_sub(1);
                let inner_x = event.column.saturating_sub(1);

                if inner_y == 0 {
                    let tab_width = term_w / 4;
                    let clicked_tab = (inner_x / tab_width.max(1)).min(3) as usize;
                    self.settings_tab = clicked_tab;
                    if self.settings_tab == 1 {
                        let models = known_models_for_provider(&self.provider_name);
                        let idx = models
                            .iter()
                            .position(|m| *m == self.model_name)
                            .unwrap_or(0);
                        self.settings_model_cursor = idx;
                    }
                } else if self.settings_tab == 0 && inner_y >= 3 {
                    let provider_index = inner_y.saturating_sub(3) as usize;
                    if provider_index < KNOWN_PROVIDERS.len() {
                        self.settings_provider_cursor = provider_index;
                        if let Some(provider) = KNOWN_PROVIDERS.get(provider_index) {
                            self.provider_name = provider.to_string();
                            let models = known_models_for_provider(provider);
                            if !models.is_empty() {
                                self.model_name = models[0].to_string();
                            }
                            self.settings_model_cursor = 0;
                            self.save_config();
                            self.notify(&format!("Provider: {} / {}", provider, self.model_name));
                        }
                    }
                } else if self.settings_tab == 1 && inner_y >= 3 {
                    let models = known_models_for_provider(&self.provider_name);
                    let model_index = inner_y.saturating_sub(3) as usize;
                    if model_index < models.len() {
                        self.settings_model_cursor = model_index;
                        if let Some(model) = models.get(model_index) {
                            self.model_name = model.to_string();
                            self.save_config();
                            self.notify(&format!("Model: {}", model));
                        }
                    }
                } else if self.settings_tab == 2 && inner_y >= 3 {
                    let item_index = inner_y.saturating_sub(3) as usize;
                    if item_index < 4 {
                        self.settings_behavior_cursor = item_index;
                        match item_index {
                            0 => {
                                self.auto_scroll = !self.auto_scroll;
                                self.notify(&format!(
                                    "Auto-scroll: {}",
                                    if self.auto_scroll { "on" } else { "off" }
                                ));
                            }
                            1 => {
                                self.show_thinking = !self.show_thinking;
                                self.notify(&format!(
                                    "Show thinking: {}",
                                    if self.show_thinking { "on" } else { "off" }
                                ));
                            }
                            2 => {
                                self.temperature = if self.temperature >= 1.5 {
                                    0.0
                                } else {
                                    self.temperature + 0.1
                                };
                                self.notify(&format!("Temperature: {:.1}", self.temperature));
                            }
                            3 => {
                                self.max_tokens = match self.max_tokens {
                                    256 => 512,
                                    512 => 1024,
                                    1024 => 2048,
                                    2048 => 4096,
                                    4096 => 8192,
                                    8192 => 16384,
                                    _ => 256,
                                };
                                self.notify(&format!("Max tokens: {}", self.max_tokens));
                            }
                            _ => {}
                        }
                        self.save_config();
                    }
                } else if self.settings_tab == 3 && inner_y >= 3 {
                    let theme_index = inner_y.saturating_sub(3) as usize;
                    let total = Theme::list_all().len();
                    if theme_index < total {
                        self.settings_theme_cursor = theme_index;
                        if let Some(selected) = Theme::list_all().get(theme_index) {
                            self.theme = Theme::find_by_name(selected.name)
                                .unwrap_or_else(|| self.theme.clone());
                            self.save_config();
                            self.notify(&format!("Theme: {}", selected.name));
                        }
                    }
                }
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

    // ── Slash commands ──────────────────────────────────────────────

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
                    self.save_config();
                    self.messages.push(MessageBubble::new(
                        MessageRole::System,
                        format!("Model switched to: {}", name),
                    ));
                } else {
                    let info = format!(
                        "Current: {}/{}\n\
                         \n\
                         Usage: /model <name>\n\
                         \n\
                         Common models:\n\
                         gpt-4o, gpt-4o-mini, gpt-4-turbo\n\
                         claude-sonnet-4-20250514, claude-3-5-sonnet\n\
                         gemini-2.0-flash, gemini-1.5-pro\n\
                         deepseek-chat, deepseek-coder\n\
                         llama-3.3-70b, mixtral-8x7b",
                        self.provider_name, self.model_name
                    );
                    self.messages
                        .push(MessageBubble::new(MessageRole::System, info));
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
                    self.save_config();
                    self.messages.push(MessageBubble::new(
                        MessageRole::System,
                        format!("Provider switched to: {}", name),
                    ));
                } else {
                    let info = format!(
                        "Current provider: {}\n\
                         Current model: {}/{}\n\
                         \n\
                         Usage: /provider <name>\n\
                         \n\
                         Available: openai, anthropic, gemini, groq,\n\
                         deepseek, mistral, together, openrouter,\n\
                         fireworks, perplexity, ollama",
                        self.provider_name, self.provider_name, self.model_name
                    );
                    self.messages
                        .push(MessageBubble::new(MessageRole::System, info));
                }
            }
            SlashCommand::Save => {
                self.save_session();
                self.messages.push(MessageBubble::new(
                    MessageRole::System,
                    format!("Session saved: {}", self.session_id),
                ));
            }
            SlashCommand::Theme(name) => {
                if !name.is_empty() {
                    if let Some(t) = crate::theme::Theme::find_by_name(&name) {
                        self.theme = t;
                        self.save_config();
                        self.messages.push(MessageBubble::new(
                            MessageRole::System,
                            format!("Theme changed to: {}", name),
                        ));
                    } else {
                        let available: Vec<_> = crate::theme::Theme::list_all()
                            .iter()
                            .map(|t| t.name.to_string())
                            .collect();
                        let info = format!(
                            "Current: {}

Available themes:
  {}",
                            self.theme.name,
                            available.join(
                                "
  "
                            )
                        );
                        self.messages
                            .push(MessageBubble::new(MessageRole::System, info));
                    }
                } else {
                    let available: Vec<_> = crate::theme::Theme::list_all()
                        .iter()
                        .map(|t| t.name.to_string())
                        .collect();
                    let info = format!(
                        "Current: {}

Available themes:
  {}",
                        self.theme.name,
                        available.join(
                            "
  "
                        )
                    );
                    self.messages
                        .push(MessageBubble::new(MessageRole::System, info));
                }
            }
            SlashCommand::New => {
                if !self.messages.is_empty() {
                    self.save_session();
                }
                self.messages.clear();
                self.tool_calls.clear();
                self.streaming = false;
                self.streaming_buffer.clear();
                self.session_id =
                    format!("session-{}", chrono::Local::now().format("%Y%m%d-%H%M%S"));
                self.messages.push(MessageBubble::new(
                    MessageRole::System,
                    format!("New session: {}", self.session_id),
                ));
            }
            SlashCommand::Doctor => {
                let glyph_tier = format!("{:?}", crate::util::glyphs::detect_tier());
                let checks: Vec<(&str, (String, bool))> = vec![
                    ("Terminal", {
                        let check = std::io::stdout().is_terminal();
                        (
                            if check { "interactive" } else { "piped" }.to_string(),
                            check,
                        )
                    }),
                    ("Theme", (self.theme.name.to_string(), true)),
                    ("Glyphs", (glyph_tier, true)),
                    (
                        "Provider",
                        (
                            self.provider_name.clone(),
                            !self.provider_name.is_empty() && self.provider_name != "none",
                        ),
                    ),
                    (
                        "Model",
                        (
                            self.model_name.clone(),
                            !self.model_name.is_empty() && self.model_name != "none",
                        ),
                    ),
                    ("Messages", (self.messages.len().to_string(), true)),
                ];
                let mut lines: Vec<String> = Vec::new();
                lines.push("Fever Doctor".to_string());
                for (label, (value, ok)) in &checks {
                    let icon = if *ok { "✓" } else { "✗" };
                    lines.push(format!("  {} {} - {}", icon, label, value));
                }
                let status = if checks.iter().all(|(_, (_, ok))| *ok) {
                    "All checks passed."
                } else {
                    "Some checks failed. Run `fever doctor` for details."
                };
                lines.push(String::new());
                lines.push(status.to_string());
                self.messages.push(MessageBubble::new(
                    MessageRole::System,
                    lines.join(
                        "
",
                    ),
                ));
            }
            SlashCommand::Session(action) => {
                let sessions_dir = dirs::data_dir()
                    .map(|d| d.join("fevercode").join("sessions"))
                    .unwrap_or_else(|| std::path::PathBuf::from(".fevercode/sessions"));

                if action == "list" || action.is_empty() {
                    if !sessions_dir.exists() {
                        self.messages.push(MessageBubble::new(
                            MessageRole::System,
                            "No sessions found.".to_string(),
                        ));
                    } else {
                        let mut sessions: Vec<_> = std::fs::read_dir(&sessions_dir)
                            .ok()
                            .map(|entries| {
                                entries
                                    .filter_map(|e| e.ok())
                                    .filter_map(|e| {
                                        let name = e.file_name().to_string_lossy().to_string();
                                        if name.ends_with(".json") {
                                            let stem = name.trim_end_matches(".json").to_string();
                                            let meta = e.metadata().ok()?;
                                            let modified = meta.modified().ok()?;
                                            let time =
                                                chrono::DateTime::<chrono::Local>::from(modified);
                                            Some(format!(
                                                "  {}  {}",
                                                stem,
                                                time.format("%Y-%m-%d %H:%M")
                                            ))
                                        } else {
                                            None
                                        }
                                    })
                                    .collect()
                            })
                            .unwrap_or_default();
                        sessions.sort();
                        if sessions.is_empty() {
                            self.messages.push(MessageBubble::new(
                                MessageRole::System,
                                "No sessions found.".to_string(),
                            ));
                        } else {
                            let mut lines = vec![format!("Sessions ({}):", sessions.len())];
                            lines.extend(sessions);
                            self.messages
                                .push(MessageBubble::new(MessageRole::System, lines.join("\n")));
                        }
                    }
                } else if action == "clear" {
                    if sessions_dir.exists() {
                        let count = std::fs::read_dir(&sessions_dir)
                            .ok()
                            .map(|entries| {
                                entries
                                    .filter_map(|e| e.ok())
                                    .filter(|e| {
                                        e.path()
                                            .extension()
                                            .map(|ext| ext == "json")
                                            .unwrap_or(false)
                                    })
                                    .count()
                            })
                            .unwrap_or(0);
                        let _ = std::fs::remove_dir_all(&sessions_dir);
                        let _ = std::fs::create_dir_all(&sessions_dir);
                        self.messages.push(MessageBubble::new(
                            MessageRole::System,
                            format!("Cleared {} session(s).", count),
                        ));
                    } else {
                        self.messages.push(MessageBubble::new(
                            MessageRole::System,
                            "No sessions to clear.".to_string(),
                        ));
                    }
                } else {
                    self.messages.push(MessageBubble::new(
                        MessageRole::System,
                        "Usage: /session [list|clear]".to_string(),
                    ));
                }
            }
        }
        vec![]
    }

    // ── Session persistence ───────────────────────────────────────

    pub fn save_session(&self) {
        if self.messages.is_empty() {
            return;
        }

        let sessions_dir = dirs::data_dir()
            .map(|d| d.join("fevercode").join("sessions"))
            .unwrap_or_else(|| std::path::PathBuf::from(".fevercode/sessions"));

        let _ = std::fs::create_dir_all(&sessions_dir);

        let session_data = serde_json::json!({
            "id": self.session_id,
            "provider": self.provider_name,
            "model": self.model_name,
            "workspace": self.workspace,
            "messages": self.messages.iter().map(|m| {
                serde_json::json!({
                    "role": format!("{:?}", m.role).to_lowercase(),
                    "content": m.content,
                })
            }).collect::<Vec<_>>(),
            "saved_at": chrono::Local::now().to_rfc3339(),
        });

        let path = sessions_dir.join(format!("{}.json", self.session_id));
        if let Ok(json) = serde_json::to_string_pretty(&session_data) {
            let _ = std::fs::write(&path, json);
            tracing::info!(session = %self.session_id, "Saved session");
        }
    }

    // ── Rendering ──────────────────────────────────────────────────

    /// Dispatch to the current screen's render function.
    pub fn render_screen(&mut self, f: &mut Frame, area: ratatui::layout::Rect) {
        match &self.screen {
            Screen::Home => crate::screens::home::render(f, area, self),
            Screen::Chat => crate::screens::chat::render(f, area, self),
            Screen::Settings => crate::screens::settings::render(f, area, self),
            Screen::Onboarding { .. } => crate::screens::onboarding::render(f, area, self),
        }
    }

    // ── Main event loop (async) ─────────────────────────────────────

    pub async fn run(&mut self) -> anyhow::Result<()> {
        // Install panic hook to restore terminal on crash
        let original_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info| {
            let _ = disable_raw_mode();
            let _ = execute!(io::stderr(), LeaveAlternateScreen, DisableMouseCapture);
            original_hook(info);
        }));

        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

        let backend = CrosstermBackend::new(stdout);
        let mut terminal = ratatui::Terminal::new(backend)?;

        let result = self.run_loop_async(&mut terminal).await;

        let _ = disable_raw_mode();
        let _ = execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        );
        let _ = terminal.show_cursor();

        result.map_err(Into::into)
    }

    /// Async event loop using tokio::select! for concurrent event handling.
    async fn run_loop_async<B: ratatui::backend::Backend>(
        &mut self,
        terminal: &mut ratatui::Terminal<B>,
    ) -> io::Result<()> {
        let (ui_tx, mut ui_rx) = mpsc::channel::<Message>(256);

        // Bridge blocking crossterm events into the async channel
        let event_tx = ui_tx.clone();
        tokio::task::spawn_blocking(move || {
            loop {
                if event::poll(Duration::from_millis(50)).ok() != Some(true) {
                    continue;
                }
                match event::read() {
                    Ok(Event::Key(key)) => {
                        if event_tx.blocking_send(Message::Key(key)).is_err() {
                            break;
                        }
                    }
                    Ok(Event::Mouse(MouseEvent {
                        kind: MouseEventKind::ScrollUp,
                        ..
                    })) => {
                        if event_tx
                            .blocking_send(Message::Key(KeyEvent::new(
                                KeyCode::PageUp,
                                KeyModifiers::NONE,
                            )))
                            .is_err()
                        {
                            break;
                        }
                    }
                    Ok(Event::Mouse(MouseEvent {
                        kind: MouseEventKind::ScrollDown,
                        ..
                    })) => {
                        if event_tx
                            .blocking_send(Message::Key(KeyEvent::new(
                                KeyCode::PageDown,
                                KeyModifiers::NONE,
                            )))
                            .is_err()
                        {
                            break;
                        }
                    }
                    Ok(Event::Mouse(
                        me @ MouseEvent {
                            kind: MouseEventKind::Down(MouseButton::Left),
                            ..
                        },
                    )) => {
                        if event_tx.blocking_send(Message::Mouse(me)).is_err() {
                            break;
                        }
                    }
                    _ => {}
                }
            }
        });

        // Tick timer for animations
        let tick_tx = ui_tx.clone();
        let tick_rate = self.tick_rate;
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tick_rate);
            loop {
                interval.tick().await;
                if tick_tx.send(Message::Tick).await.is_err() {
                    break;
                }
            }
        });

        self.last_tick = Instant::now();

        loop {
            // Render current state
            terminal.draw(|f| {
                self.terminal_size = (f.area().width, f.area().height);
                render_frame(f, self);
            })?;

            // Wait for next message
            if let Some(msg) = ui_rx.recv().await {
                let cmds = self.update(msg);
                self.execute_commands(cmds, &ui_tx);

                // Drain any buffered messages before next render
                while let Ok(msg) = ui_rx.try_recv() {
                    let cmds = self.update(msg);
                    self.execute_commands(cmds, &ui_tx);
                }
            }

            if self.should_quit {
                self.save_session();
                return Ok(());
            }
        }
    }

    /// Handle side-effect commands (spawn streaming tasks, etc.)
    fn execute_commands(&mut self, cmds: Vec<Command>, tx: &mpsc::Sender<Message>) {
        for cmd in cmds {
            match cmd {
                Command::SendMessage { content } => {
                    self.streaming = true;
                    let tx = tx.clone();

                    let cancel_token = tokio_util::sync::CancellationToken::new();
                    let cancel_clone = cancel_token.clone();
                    self.cancel_token = Some(cancel_token);

                    if let Some(agent) = self.agent.clone() {
                        let content = content.clone();
                        tokio::spawn(async move {
                            agent.submit(content, tx);
                        });
                    } else {
                        tokio::spawn(async move {
                            let first_line = content.lines().next().unwrap_or(&content);
                            let response = format!(
                                "◈ Received: \"{}\"\n\n\
                                 I am Fever, your cold sacred coding assistant.\n\
                                 No provider configured — set one in ~/.config/fevercode/config.toml\n\
                                 or use /provider and /model commands.",
                                if first_line.len() > 60 {
                                    format!("{}...", &first_line[..60])
                                } else {
                                    first_line.to_string()
                                }
                            );
                            for ch in response.chars() {
                                if cancel_clone.is_cancelled()
                                    || tx
                                        .send(Message::StreamChunk {
                                            content: ch.to_string(),
                                        })
                                        .await
                                        .is_err()
                                {
                                    break;
                                }
                                tokio::time::sleep(Duration::from_millis(12)).await;
                            }
                            tx.send(Message::StreamEnd).await.ok();
                        });
                    }
                }
                Command::DetectProviders => {
                    // TODO: Probe for configured providers
                }
                Command::Noop => {}
            }
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
