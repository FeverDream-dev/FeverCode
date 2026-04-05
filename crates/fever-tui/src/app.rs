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
use fever_core::PermissionMode;
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
    "mock",
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
        "mock" => vec!["mock/default"],
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

    

#[derive(Debug, Clone)]
pub struct McpServerEntry {
    pub name: String,
    pub enabled: bool,
    pub connected: bool,
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

    // Home screen navigation state
    pub home_selection: usize,          // currently selected action (0-indexed)
    pub home_action_count: usize,       // total number of actions (computed on render)
    pub git_branch: Option<String>,     // detected git branch name
    pub is_git_repo: bool,              // whether workspace is a git repo
    pub has_provider: bool,               // provider configured (true) or not (false)

    // Permission mode (read/write/full) — wired from fever-core
    pub permission_mode: PermissionMode,

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

    // Panels (Ctrl+T for tools, Ctrl+D for diff)
    pub show_tool_panel: bool,
    pub show_diff_panel: bool,
    pub panel_width: u16,
    pub diff_content: Vec<String>,

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

    // Slash menu interactive typeahead
    pub slash_menu_visible: bool,
    pub slash_menu_selection: usize,

    // Token/cost/context telemetry
    pub input_tokens: usize,
    pub output_tokens: usize,
    pub total_tokens: usize,
    pub estimated_cost: f64,
    pub context_limit: usize,
    pub request_start: Option<Instant>,
    pub request_elapsed: Option<Duration>,
    pub show_tokens_in_status: bool,
    pub show_cost_in_status: bool,
    pub show_elapsed_in_status: bool,

    // MCP management
    pub mcp_servers: Vec<McpServerEntry>,
    pub mcp_enabled: bool,

    // Pre-prompt controls
    pub preprompt_enabled: bool,
    pub preprompt_mode: String,

    // Settings tabs 4-7 cursors
    pub settings_mcp_cursor: usize,
    pub settings_preprompt_cursor: usize,
    pub settings_telemetry_cursor: usize,
    pub settings_advanced_cursor: usize,

    // Advanced settings
    pub timeout_secs: u32,
    pub verbosity: u8,
    pub glyph_mode: String,
    pub mouse_enabled: bool,
    pub is_mock_mode: bool,

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
            permission_mode: PermissionMode::default(),
            // Home screen defaults; will be adjusted below
            home_selection: 0,
            home_action_count: 7, // 7 quick actions defined in home screen
            git_branch: None,
            is_git_repo: false,
            has_provider: false,
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
            show_tool_panel: false,
            show_diff_panel: false,
            panel_width: 30,
            diff_content: Vec::new(),
            settings_tab: 0,
            settings_provider_cursor: 0,
            settings_model_cursor: 0,
            settings_theme_cursor: 0,
            settings_behavior_cursor: 0,
            auto_scroll: true,
            show_thinking: true,
            temperature: 0.7,
            max_tokens: 4096,

            slash_menu_visible: false,
            slash_menu_selection: 0,
            input_tokens: 0,
            output_tokens: 0,
            total_tokens: 0,
            estimated_cost: 0.0,
            context_limit: 128_000,
            request_start: None,
            request_elapsed: None,
            show_tokens_in_status: false,
            show_cost_in_status: false,
            show_elapsed_in_status: false,
            mcp_servers: vec![
                McpServerEntry {
                    name: "filesystem".to_string(),
                    enabled: false,
                    connected: false,
                },
                McpServerEntry {
                    name: "github".to_string(),
                    enabled: false,
                    connected: false,
                },
                McpServerEntry {
                    name: "browser".to_string(),
                    enabled: false,
                    connected: false,
                },
            ],
            mcp_enabled: false,
            preprompt_enabled: true,
            preprompt_mode: "default".to_string(),
            settings_mcp_cursor: 0,
            settings_preprompt_cursor: 0,
            settings_telemetry_cursor: 0,
            settings_advanced_cursor: 0,
            timeout_secs: 120,
            verbosity: 0,
            glyph_mode: "auto".to_string(),
            mouse_enabled: true,
            is_mock_mode: false,
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
        // After loading config, recompute provider-based state
        state.has_provider = state.provider_name != "none";
        // Detect git repo state for the initial workspace
        let ws_path = std::path::Path::new(&state.workspace);
        if ws_path.exists() {
            let git_path = ws_path.join(".git");
            state.is_git_repo = git_path.exists();
            if state.is_git_repo {
                if let Ok(output) = std::process::Command::new("git")
                    .args(["rev-parse", "--abbrev-ref", "HEAD"])
                    .output()
                {
                    if output.status.success() {
                        let branch = String::from_utf8_lossy(&output.stdout)
                            .trim()
                            .to_string();
                        state.git_branch = Some(branch);
                    }
                }
            }
        }
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
                        // Telemetry/MCP/preprompt/advanced loading
                        if let Some(show_tok) = value
                            .get("telemetry")
                            .and_then(|t| t.get("show_tokens"))
                            .and_then(|v| v.as_bool())
                        {
                            self.show_tokens_in_status = show_tok;
                        }
                        if let Some(show_cost) = value
                            .get("telemetry")
                            .and_then(|t| t.get("show_cost"))
                            .and_then(|v| v.as_bool())
                        {
                            self.show_cost_in_status = show_cost;
                        }
                        if let Some(show_elapsed) = value
                            .get("telemetry")
                            .and_then(|t| t.get("show_elapsed"))
                            .and_then(|v| v.as_bool())
                        {
                            self.show_elapsed_in_status = show_elapsed;
                        }
                        if let Some(mcp_on) = value
                            .get("mcp")
                            .and_then(|m| m.get("enabled"))
                            .and_then(|v| v.as_bool())
                        {
                            self.mcp_enabled = mcp_on;
                        }
                        if let Some(pp_on) = value
                            .get("preprompt")
                            .and_then(|p| p.get("enabled"))
                            .and_then(|v| v.as_bool())
                        {
                            self.preprompt_enabled = pp_on;
                        }
                        if let Some(pp_mode) = value
                            .get("preprompt")
                            .and_then(|p| p.get("mode"))
                            .and_then(|v| v.as_str())
                        {
                            self.preprompt_mode = pp_mode.to_string();
                        }
                        if let Some(timeout) = value
                            .get("advanced")
                            .and_then(|a| a.get("timeout_secs"))
                            .and_then(|v| v.as_integer())
                        {
                            self.timeout_secs = timeout as u32;
                        }
                        if let Some(verb) = value
                            .get("advanced")
                            .and_then(|a| a.get("verbosity"))
                            .and_then(|v| v.as_integer())
                        {
                            self.verbosity = verb as u8;
                        }
                        if let Some(glyph) = value
                            .get("advanced")
                            .and_then(|a| a.get("glyph_mode"))
                            .and_then(|v| v.as_str())
                        {
                            self.glyph_mode = glyph.to_string();
                        }
                        if let Some(mouse) = value
                            .get("advanced")
                            .and_then(|a| a.get("mouse_enabled"))
                            .and_then(|v| v.as_bool())
                        {
                            self.mouse_enabled = mouse;
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

            // Telemetry/MCP/preprompt/advanced configuration sections
            let mut telemetry = toml::value::Table::new();
            telemetry.insert(
                "show_tokens".to_string(),
                toml::Value::Boolean(self.show_tokens_in_status),
            );
            telemetry.insert(
                "show_cost".to_string(),
                toml::Value::Boolean(self.show_cost_in_status),
            );
            telemetry.insert(
                "show_elapsed".to_string(),
                toml::Value::Boolean(self.show_elapsed_in_status),
            );

            let mut mcp_table = toml::value::Table::new();
            mcp_table.insert(
                "enabled".to_string(),
                toml::Value::Boolean(self.mcp_enabled),
            );

            let mut preprompt_table = toml::value::Table::new();
            preprompt_table.insert(
                "enabled".to_string(),
                toml::Value::Boolean(self.preprompt_enabled),
            );
            preprompt_table.insert(
                "mode".to_string(),
                toml::Value::String(self.preprompt_mode.clone()),
            );

            let mut advanced = toml::value::Table::new();
            advanced.insert(
                "timeout_secs".to_string(),
                toml::Value::Integer(self.timeout_secs as i64),
            );
            advanced.insert(
                "verbosity".to_string(),
                toml::Value::Integer(self.verbosity as i64),
            );
            advanced.insert(
                "glyph_mode".to_string(),
                toml::Value::String(self.glyph_mode.clone()),
            );
            advanced.insert(
                "mouse_enabled".to_string(),
                toml::Value::Boolean(self.mouse_enabled),
            );

            let mut root = toml::value::Table::new();
            root.insert("defaults".to_string(), toml::Value::Table(defaults));
            root.insert("ui".to_string(), toml::Value::Table(ui));

            // Attach extra sections
            root.insert("telemetry".to_string(), toml::Value::Table(telemetry));
            root.insert("mcp".to_string(), toml::Value::Table(mcp_table));
            root.insert("preprompt".to_string(), toml::Value::Table(preprompt_table));
            root.insert("advanced".to_string(), toml::Value::Table(advanced));
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
                if self.request_start.is_none() {
                    self.request_start = Some(Instant::now());
                }
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
                if let Some(start) = self.request_start.take() {
                    self.request_elapsed = Some(start.elapsed());
                }
                let total_chars: usize = self.messages.iter().map(|m| m.content.len()).sum();
                self.total_tokens = total_chars / 4;
                self.output_tokens = self.streaming_buffer.len() / 4;
                let input_cost = (self.input_tokens as f64 / 1_000_000.0) * 5.0;
                let output_cost = (self.output_tokens as f64 / 1_000_000.0) * 15.0;
                self.estimated_cost += input_cost + output_cost;
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
                KeyCode::Char('t') => {
                    self.show_tool_panel = !self.show_tool_panel;
                    return vec![];
                }
                KeyCode::Char('d') => {
                    self.show_diff_panel = !self.show_diff_panel;
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
                let specs = self.palette_commands();
                if let Some(spec) = specs.get(self.palette_selection) {
                    // Build a SlashCommand from the selected spec's name
                    let full_cmd = format!("/{0}", spec.name);
                    if let Some(cmd) = SlashCommand::parse(&full_cmd) {
                        self.show_command_palette = false;
                        self.palette_query.clear();
                        self.palette_selection = 0;
                        return self.handle_slash_command(cmd);
                    }
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

    fn execute_home_action(&mut self, index: usize) -> Vec<Command> {
        match index {
            0 => {
                self.screen = Screen::Chat;
                self.input_buffer.clear();
                vec![]
            }
            1 => {
                if let Some(cmd) = SlashCommand::parse("/session list") {
                    return self.handle_slash_command(cmd);
                }
                vec![]
            }
            2 => {
                self.screen = Screen::Settings;
                vec![]
            }
            3 => {
                if let Some(cmd) = SlashCommand::parse("/doctor") {
                    return self.handle_slash_command(cmd);
                }
                vec![]
            }
            4 => {
                self.show_command_palette = true;
                vec![]
            }
            5 => {
                if let Some(cmd) = SlashCommand::parse("/help") {
                    return self.handle_slash_command(cmd);
                }
                vec![]
            }
            6 => {
                self.screen = Screen::Settings;
                vec![]
            }
            _ => vec![],
        }
    }

    /// Returns all slash commands matching the palette query.
    pub fn palette_commands(&self) -> Vec<&'static crate::slash::commands::SlashCommandSpec> {
        // Use the new SlashCommandSpec registry with fuzzy search
        SlashCommand::find_specs(&self.palette_query)
    }

    // ── Screen key handlers ─────────────────────────────────────────

    fn handle_home_key(&mut self, key: KeyEvent) -> Vec<Command> {
        // Ensure action count reflects current UI (7 quick actions)
        self.home_action_count = 7usize;

        match key.code {
            // Navigation wrap
            KeyCode::Up => {
                if self.home_action_count > 0 {
                    self.home_selection = (self.home_selection + self.home_action_count - 1)
                        % self.home_action_count;
                }
            }
            KeyCode::Down => {
                if self.home_action_count > 0 {
                    self.home_selection = (self.home_selection + 1) % self.home_action_count;
                }
            }
            // Direct selection via number keys 1-7
            KeyCode::Char('1') => {
                self.home_selection = 0;
                return self.execute_home_action(self.home_selection);
            }
            KeyCode::Char('2') => {
                self.home_selection = 1;
                return self.execute_home_action(self.home_selection);
            }
            KeyCode::Char('3') => {
                self.home_selection = 2;
                return self.execute_home_action(self.home_selection);
            }
            KeyCode::Char('4') => {
                self.home_selection = 3;
                return self.execute_home_action(self.home_selection);
            }
            KeyCode::Char('5') => {
                self.home_selection = 4;
                return self.execute_home_action(self.home_selection);
            }
            KeyCode::Char('6') => {
                self.home_selection = 5;
                return self.execute_home_action(self.home_selection);
            }
            KeyCode::Char('7') => {
                self.home_selection = 6;
                return self.execute_home_action(self.home_selection);
            }
            // Activation / Enter
            KeyCode::Enter => {
                return self.execute_home_action(self.home_selection);
            }
            // Quick access shortcuts
            KeyCode::Esc | KeyCode::Char('q') => {
                self.should_quit = true;
            }
            KeyCode::Char('s') | KeyCode::Char('S') => {
                self.screen = Screen::Settings;
            }
            KeyCode::Char('/') => {
                self.screen = Screen::Chat;
                self.input_buffer = "/".to_string();
            }
            _ => {}
        }
        vec![]
    }

    

    fn handle_chat_key(&mut self, key: KeyEvent) -> Vec<Command> {
        // Slash menu typeahead navigation and interception
        let has_slash = self.input_buffer.starts_with('/') && !self.input_buffer.contains(' ');
        if has_slash {
            let query = self.input_buffer.trim_start_matches('/').to_lowercase();
            let matches: Vec<_> = SlashCommand::find_specs(&query)
                .iter()
                .map(|spec| (spec.name, spec.summary))
                .collect();
            self.slash_menu_visible = !matches.is_empty();
            if self.slash_menu_selection >= matches.len() {
                self.slash_menu_selection = 0;
            }
        } else {
            self.slash_menu_visible = false;
            self.slash_menu_selection = 0;
        }

        // Slash menu navigation intercept
        if self.slash_menu_visible {
            match key.code {
                KeyCode::Down => {
                    let query = self.input_buffer.trim_start_matches('/');
                    let matches: Vec<_> = SlashCommand::find_specs(query)
                        .iter()
                        .map(|s| s.name)
                        .collect();
                    if self.slash_menu_selection < matches.len().saturating_sub(1) {
                        self.slash_menu_selection += 1;
                    }
                    return vec![];
                }
                KeyCode::Up => {
                    if self.slash_menu_selection > 0 {
                        self.slash_menu_selection -= 1;
                    }
                    return vec![];
                }
                KeyCode::Enter => {
                    let query = self.input_buffer.trim_start_matches('/');
                    let matches: Vec<_> = SlashCommand::find_specs(query)
                        .iter()
                        .map(|s| s.name)
                        .collect();
                    if let Some(&name) = matches.get(self.slash_menu_selection) {
                        let full_cmd = format!("/{0}", name);
                        if let Some(cmd) = SlashCommand::parse(&full_cmd) {
                            self.slash_menu_visible = false;
                            self.slash_menu_selection = 0;
                            self.input_buffer.clear();
                            self.history_index = None;
                            return self.handle_slash_command(cmd);
                        }
                    }
                    self.slash_menu_visible = false;
                    self.slash_menu_selection = 0;
                    return vec![];
                }
                KeyCode::Esc => {
                    self.slash_menu_visible = false;
                    self.slash_menu_selection = 0;
                    self.input_buffer.clear();
                    return vec![];
                }
                KeyCode::Tab => {
                    let query = self.input_buffer.trim_start_matches('/').to_lowercase();
                    let specs = SlashCommand::find_specs(&query);
                    if let Some(name) = specs.get(self.slash_menu_selection).map(|s| s.name) {
                        self.input_buffer = format!("/{0} ", name);
                        self.slash_menu_visible = false;
                        self.slash_menu_selection = 0;
                        self.history_index = None;
                    }
                    return vec![];
                }
                _ => {}
            }
        }

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
                self.settings_tab = (self.settings_tab + 1) % 8;
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
                self.settings_tab = (self.settings_tab + 7) % 8;
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
            // Tab 4: MCP
            KeyCode::Up if self.settings_tab == 4 => {
                if self.settings_mcp_cursor > 0 {
                    self.settings_mcp_cursor -= 1;
                } else {
                    self.settings_mcp_cursor = self.mcp_servers.len().saturating_sub(1);
                }
            }
            KeyCode::Down if self.settings_tab == 4 => {
                if self.settings_mcp_cursor < self.mcp_servers.len().saturating_sub(1) {
                    self.settings_mcp_cursor += 1;
                } else {
                    self.settings_mcp_cursor = 0;
                }
            }
            KeyCode::Enter if self.settings_tab == 4 => {
                let cursor = self.settings_mcp_cursor;
                if let Some(server) = self.mcp_servers.get_mut(cursor) {
                    server.enabled = !server.enabled;
                    let name = server.name.clone();
                    let status = if server.enabled {
                        "enabled"
                    } else {
                        "disabled"
                    };
                    self.notify(&format!("MCP '{}' {}", name, status));
                }
            }
            // Tab 5: PrePrompt
            KeyCode::Up if self.settings_tab == 5 => {
                if self.settings_preprompt_cursor > 0 {
                    self.settings_preprompt_cursor -= 1;
                } else {
                    self.settings_preprompt_cursor = 2;
                }
            }
            KeyCode::Down if self.settings_tab == 5 => {
                if self.settings_preprompt_cursor < 2 {
                    self.settings_preprompt_cursor += 1;
                } else {
                    self.settings_preprompt_cursor = 0;
                }
            }
            KeyCode::Enter if self.settings_tab == 5 => match self.settings_preprompt_cursor {
                0 => {
                    self.preprompt_enabled = !self.preprompt_enabled;
                    self.notify(&format!(
                        "Pre-prompt: {}",
                        if self.preprompt_enabled { "on" } else { "off" }
                    ));
                }
                1 => {
                    let modes = ["default", "concise", "detailed"];
                    let idx = modes
                        .iter()
                        .position(|&m| m == self.preprompt_mode)
                        .unwrap_or(0);
                    self.preprompt_mode = modes[(idx + 1) % modes.len()].to_string();
                    self.notify(&format!("Pre-prompt mode: {}", self.preprompt_mode));
                }
                _ => {}
            },
            // Tab 6: Telemetry
            KeyCode::Up if self.settings_tab == 6 => {
                if self.settings_telemetry_cursor > 0 {
                    self.settings_telemetry_cursor -= 1;
                } else {
                    self.settings_telemetry_cursor = 2;
                }
            }
            KeyCode::Down if self.settings_tab == 6 => {
                if self.settings_telemetry_cursor < 2 {
                    self.settings_telemetry_cursor += 1;
                } else {
                    self.settings_telemetry_cursor = 0;
                }
            }
            KeyCode::Enter if self.settings_tab == 6 => match self.settings_telemetry_cursor {
                0 => {
                    self.show_tokens_in_status = !self.show_tokens_in_status;
                    self.save_config();
                    self.notify(&format!(
                        "Show tokens: {}",
                        if self.show_tokens_in_status {
                            "on"
                        } else {
                            "off"
                        }
                    ));
                }
                1 => {
                    self.show_cost_in_status = !self.show_cost_in_status;
                    self.save_config();
                    self.notify(&format!(
                        "Show cost: {}",
                        if self.show_cost_in_status {
                            "on"
                        } else {
                            "off"
                        }
                    ));
                }
                2 => {
                    self.show_elapsed_in_status = !self.show_elapsed_in_status;
                    self.save_config();
                    self.notify(&format!(
                        "Show elapsed: {}",
                        if self.show_elapsed_in_status {
                            "on"
                        } else {
                            "off"
                        }
                    ));
                }
                _ => {}
            },
            // Tab 7: Advanced
            KeyCode::Up if self.settings_tab == 7 => {
                if self.settings_advanced_cursor > 0 {
                    self.settings_advanced_cursor -= 1;
                } else {
                    self.settings_advanced_cursor = 3;
                }
            }
            KeyCode::Down if self.settings_tab == 7 => {
                if self.settings_advanced_cursor < 3 {
                    self.settings_advanced_cursor += 1;
                } else {
                    self.settings_advanced_cursor = 0;
                }
            }
            KeyCode::Enter if self.settings_tab == 7 => match self.settings_advanced_cursor {
                0 => {
                    self.timeout_secs = match self.timeout_secs {
                        30 => 60,
                        60 => 120,
                        120 => 300,
                        300 => 600,
                        _ => 30,
                    };
                    self.save_config();
                    self.notify(&format!("Timeout: {}s", self.timeout_secs));
                }
                1 => {
                    self.verbosity = (self.verbosity + 1) % 4;
                    self.save_config();
                    self.notify(&format!("Verbosity: {}", self.verbosity));
                }
                2 => {
                    let modes = ["auto", "unicode", "ascii"];
                    let idx = modes
                        .iter()
                        .position(|&m| m == self.glyph_mode)
                        .unwrap_or(0);
                    self.glyph_mode = modes[(idx + 1) % modes.len()].to_string();
                    self.save_config();
                    self.notify(&format!("Glyph mode: {}", self.glyph_mode));
                }
                3 => {
                    self.mouse_enabled = !self.mouse_enabled;
                    self.save_config();
                    self.notify(&format!(
                        "Mouse: {}",
                        if self.mouse_enabled { "on" } else { "off" }
                    ));
                }
                _ => {}
            },
            _ => {}
        }
        vec![]
    }

    fn handle_mouse_click(&mut self, event: MouseEvent) -> Vec<Command> {
        let (term_w, term_h) = self.terminal_size;
        if event.row >= term_h.saturating_sub(1) {
            return vec![];
        }

        
        let slash_popup_height: i32 = {
            let hints = SlashCommand::find_specs(&self.input_buffer[1..]);
            let hint_rows = hints.len().min(7);
            if hint_rows > 0 { (hint_rows as i32) + 1 } else { 0 }
        };
        let slash_popup_top_y = (term_h as i32) - 3 - slash_popup_height;

        let mut dismissed = false;
        if self.slash_menu_visible {
            let inside_slash = if slash_popup_height > 0 {
                let y = slash_popup_top_y;
                let end_y = slash_popup_top_y + slash_popup_height - 1;
                let row = event.row as i32;
                row >= y && row <= end_y
            } else {
                false
            };
            if !inside_slash {
                self.slash_menu_visible = false;
                dismissed = true;
            }
        }

        
        let palette_width = 50.min(term_w.saturating_sub(4));
        let palette_height = 14.min(term_h.saturating_sub(4));
        let palette_x = (term_w.saturating_sub(palette_width)) / 2;
        let palette_y = term_h / 4;
        if self.show_command_palette {
            let inside_palette = {
                let within_y = (event.row as i32) - palette_y as i32;
                let within_x = (event.column as i32) - palette_x as i32;
                within_y >= 0 && within_y < palette_height as i32 && within_x >= 0 && within_x < palette_width as i32
            };
            if !inside_palette {
                self.show_command_palette = false;
                dismissed = true;
            }
        }
        if dismissed {
            return vec![];
        }

        match self.screen {
            Screen::Home => {
                
                if self.slash_menu_visible {
                    if slash_popup_height > 0 {
                        let y = slash_popup_top_y;
                        let end_y = slash_popup_top_y + slash_popup_height - 1;
                        let r = event.row as i32;
                        if r >= y && r <= end_y {
                                
                            let relative = r - y;
                            if relative >= 1 {
                                let idx = (relative - 1) as usize;
                                let specs = SlashCommand::find_specs(&self.input_buffer[1..]);
                                let max = specs.len().min(7);
                                if idx < max {
                                    let spec = specs[idx];
                                    self.input_buffer = format!("/{0}", spec.name);
                                    self.slash_menu_visible = false;
                                    return vec![];
                                }
                            }
                        }
                    }
                }
                let actions = self.home_action_count.min(7);
                if actions > 0 {
                    let start_y: i32 = 2;
                    if (event.row as i32) >= start_y {
                        let per_action = if (term_h as i32 - start_y) > 0 {
                            (term_h as i32 - start_y) / actions as i32
                        } else {
                            1
                        };
                        let idx = ((event.row as i32 - start_y) / per_action) as usize;
                        if idx < actions {
                            let _ = self.execute_home_action(idx);
                        }
                    }
                }
            }
            Screen::Settings => {
                let inner_y = event.row.saturating_sub(1);
                let inner_x = event.column.saturating_sub(1);

                if inner_y == 0 {
                    let tab_width = term_w / 8;
                    let clicked_tab = (inner_x / tab_width.max(1)).min(7) as usize;
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
                if self.permission_mode == PermissionMode::ReadOnly {
                    self.notify("Blocked: read-only mode");
                } else {
                    self.messages.clear();
                    self.tool_calls.clear();
                    self.streaming = false;
                    self.streaming_buffer.clear();
                }
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
            SlashCommand::Mock => {
                // Toggle mock mode for local testing
                self.is_mock_mode = !self.is_mock_mode;
                self.notify(if self.is_mock_mode {
                    "Mock mode enabled"
                } else {
                    "Mock mode disabled"
                });
            }
            SlashCommand::Model(opt) => {
                if let Some(name) = opt {
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
            SlashCommand::Role(opt) => {
                let msg = if let Some(name) = opt {
                    if !name.is_empty() { format!("Role set to: {}", name) } else { "Usage: /role <name>".to_string() }
                } else {
                    "Usage: /role <name>".to_string()
                };
                self.messages.push(MessageBubble::new(MessageRole::System, msg));
            }
            SlashCommand::Provider(opt) => {
                if let Some(name) = opt {
                    if !name.is_empty() {
                        self.provider_name = name.clone();
                        self.save_config();
                        self.messages.push(MessageBubble::new(
                            MessageRole::System,
                            format!("Provider switched to: {}", name),
                        ));
                    } else {
                        let info = format!(
                            "Current provider: {}\nCurrent model: {}/{}\n\nUsage: /provider <name>\n\nAvailable: openai, anthropic, gemini, groq,\ndeepseek, mistral, together, openrouter,\nfireworks, perplexity, ollama",
                            self.provider_name, self.provider_name, self.model_name
                        );
                        self.messages
                            .push(MessageBubble::new(MessageRole::System, info));
                    }
                } else {
                    let info = format!(
                        "Current provider: {}\nCurrent model: {}/{}\n\nUsage: /provider <name>\n\nAvailable: openai, anthropic, gemini, groq,\ndeepseek, mistral, together, openrouter,\nfireworks, perplexity, ollama",
                        self.provider_name, self.provider_name, self.model_name
                    );
                    self.messages
                        .push(MessageBubble::new(MessageRole::System, info));
                }
            }
            SlashCommand::Permissions(opt) => {
                // Cycle through permission modes: ReadOnly -> WorkspaceWrite -> DangerFullAccess -> ReadOnly
                self.permission_mode = match self.permission_mode {
                    PermissionMode::ReadOnly => PermissionMode::WorkspaceWrite,
                    PermissionMode::WorkspaceWrite => PermissionMode::DangerFullAccess,
                    PermissionMode::DangerFullAccess => PermissionMode::ReadOnly,
                };
                self.notify(&format!("Permission mode: {}", self.permission_mode.label()));
                if let Some(p) = opt {
                    if !p.is_empty() {
                        self.messages.push(MessageBubble::new(
                            MessageRole::System,
                            format!("Permissions: {}", p),
                        ));
                    }
                }
            }
            SlashCommand::ReadOnly => {
                self.permission_mode = PermissionMode::ReadOnly;
                self.notify(&format!("Permission mode: {}", self.permission_mode.label()));
            }
            SlashCommand::Diff => {
                self.show_diff_panel = !self.show_diff_panel;
                if self.show_diff_panel {
                    let output = std::process::Command::new("git")
                        .args(["diff", "--stat"])
                        .output();
                    match output {
                        Ok(out) if out.status.success() => {
                            let text = String::from_utf8_lossy(&out.stdout).to_string();
                            self.diff_content = if text.trim().is_empty() {
                                vec!["No changes detected.".to_string()]
                            } else {
                                text.lines().map(|l| l.to_string()).collect()
                            };
                            self.notify(&format!(
                                "Diff: {} files changed",
                                self.diff_content.len().saturating_sub(1)
                            ));
                        }
                        Ok(out) => {
                            self.diff_content = vec![
                                format!("git diff failed: {}", String::from_utf8_lossy(&out.stderr))
                            ];
                        }
                        Err(e) => {
                            self.diff_content = vec![format!("git not available: {e}")];
                        }
                    }
                }
            }
            SlashCommand::Git(opt) => {
                if let Some(cmd) = opt {
                    if !cmd.is_empty() {
                        self.messages.push(MessageBubble::new(
                            MessageRole::System,
                            format!("Git command: {}", cmd),
                        ));
                    } else {
                        self.messages.push(MessageBubble::new(
                            MessageRole::System,
                            "Usage: /git <command>".to_string(),
                        ));
                    }
                } else {
                    self.messages.push(MessageBubble::new(
                        MessageRole::System,
                        "Usage: /git <command>".to_string(),
                    ));
                }
            }
            SlashCommand::Save => {
                self.save_session();
                self.messages.push(MessageBubble::new(
                    MessageRole::System,
                    format!("Session saved: {}", self.session_id),
                ));
            }
            SlashCommand::Theme(opt) => {
                if let Some(name) = opt {
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
                                "Current: {}\n\nAvailable themes:\n  {}",
                                self.theme.name,
                                available.join("\n  ")
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
                            "Current: {}\n\nAvailable themes:\n  {}",
                            self.theme.name,
                            available.join("\n  ")
                        );
                        self.messages
                            .push(MessageBubble::new(MessageRole::System, info));
                    }
                }
            }
            SlashCommand::New => {
                if self.permission_mode == PermissionMode::ReadOnly {
                    self.notify("Blocked: read-only mode");
                } else {
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
            SlashCommand::Session(opt) => {
                let sessions_dir = dirs::data_dir()
                    .map(|d| d.join("fevercode").join("sessions"))
                    .unwrap_or_else(|| std::path::PathBuf::from(".fevercode/sessions"));

                let action = opt.as_deref().unwrap_or("");
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
            SlashCommand::Mcp(opt) => {
                let sub = opt.as_deref().unwrap_or("");
                if sub == "list" || sub.is_empty() {
                    let mut lines = vec!["MCP Servers:".to_string()];
                    for server in &self.mcp_servers {
                        let status = if server.enabled && server.connected {
                            "connected"
                        } else if server.enabled {
                            "enabled"
                        } else {
                            "disabled"
                        };
                        lines.push(format!("  {} [{}]", server.name, status));
                    }
                    lines.push(String::new());
                    lines.push(format!(
                        "MCP enabled: {}",
                        if self.mcp_enabled { "yes" } else { "no" }
                    ));
                    self.messages
                        .push(MessageBubble::new(MessageRole::System, lines.join("\n")));
                } else {
                    let found = self.mcp_servers.iter_mut().find(|s| s.name == sub);
                    if let Some(server) = found {
                        server.enabled = !server.enabled;
                        self.messages.push(MessageBubble::new(
                            MessageRole::System,
                            format!(
                                "MCP '{}' {}",
                                server.name,
                                if server.enabled {
                                    "enabled"
                                } else {
                                    "disabled"
                                }
                            ),
                        ));
                    } else {
                        self.messages.push(MessageBubble::new(
                            MessageRole::System,
                            format!(
                                "Unknown MCP server: '{}'. Use /mcp list to see available.",
                                sub
                            ),
                        ));
                    }
                }
            }
            SlashCommand::Preprompt(opt) => {
                let sub = opt.as_deref().unwrap_or("");
                if sub == "on" {
                    self.preprompt_enabled = true;
                    self.messages.push(MessageBubble::new(
                        MessageRole::System,
                        "Pre-prompt enabled.".to_string(),
                    ));
                } else if sub == "off" {
                    self.preprompt_enabled = false;
                    self.messages.push(MessageBubble::new(
                        MessageRole::System,
                        "Pre-prompt disabled.".to_string(),
                    ));
                } else if !sub.is_empty() {
                    self.preprompt_mode = sub.to_string();
                    self.messages.push(MessageBubble::new(
                        MessageRole::System,
                        format!("Pre-prompt mode: {}", sub),
                    ));
                } else {
                    self.messages.push(MessageBubble::new(
                        MessageRole::System,
                        format!(
                            "Pre-prompt: {} (mode: {})\nUsage: /preprompt [on|off|<mode>]",
                            if self.preprompt_enabled {
                                "enabled"
                            } else {
                                "disabled"
                            },
                            self.preprompt_mode
                        ),
                    ));
                }
            }
            SlashCommand::Tokens => {
                self.messages.push(MessageBubble::new(
                    MessageRole::System,
                    format!(
                        "Tokens — Input: {} | Output: {} | Total: {}",
                        self.input_tokens, self.output_tokens, self.total_tokens
                    ),
                ));
            }
            SlashCommand::Cost => {
                self.messages.push(MessageBubble::new(
                    MessageRole::System,
                    format!(
                        "Estimated cost: ${:.4} ({} total tokens)",
                        self.estimated_cost, self.total_tokens
                    ),
                ));
            }
            SlashCommand::Context => {
                let pct = if self.context_limit > 0 {
                    (self.total_tokens as f64 / self.context_limit as f64) * 100.0
                } else {
                    0.0
                };
                self.messages.push(MessageBubble::new(
                    MessageRole::System,
                    format!(
                        "Context: {} / {} ({:.1}%)",
                        self.total_tokens, self.context_limit, pct
                    ),
                ));
            }
            SlashCommand::Time => {
                let info = match self.request_elapsed {
                    Some(d) => format!("Last request: {:.2}s", d.as_secs_f64()),
                    None => "No request timing data yet.".to_string(),
                };
                self.messages
                    .push(MessageBubble::new(MessageRole::System, info));
            }
            SlashCommand::Tools => {
                self.show_tool_panel = !self.show_tool_panel;
            }
            SlashCommand::Unknown(name) => {
                self.messages.push(MessageBubble::new(
                    MessageRole::System,
                    format!("Unknown slash command: {}", name),
                ));
            }
            SlashCommand::Export(_path) => {
                self.save_session();
                self.messages.push(MessageBubble::new(
                    MessageRole::System,
                    format!("Session exported: {}", self.session_id),
                ));
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
