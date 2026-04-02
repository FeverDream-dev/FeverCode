use std::collections::VecDeque;
use std::io;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

use crossterm::{
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyModifiers,
        MouseEvent, MouseEventKind,
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

    // Agent bridge (optional — set by CLI when provider is configured)
    pub agent: Option<Arc<dyn AgentHandle>>,
    pub cancel_token: Option<tokio_util::sync::CancellationToken>,

    // Onboarding screen
    pub onboarding_step: usize,
    pub onboarding_selection: usize,

    // Session persistence
    pub session_id: String,
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
            onboarding_step: 0,
            onboarding_selection: 0,
            agent: None,
            cancel_token: None,
            session_id: format!("session-{}", chrono::Local::now().format("%Y%m%d-%H%M%S")),
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
                    }
                }
            }
        }
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
            terminal.draw(|f| render_frame(f, self))?;

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
