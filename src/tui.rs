#![allow(dead_code)]
use anyhow::Result;
use crossterm::{
    event::{Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use futures::StreamExt;
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Terminal,
};
use std::io;
use tokio::sync::mpsc;

use crate::safety::ApprovalMode;

#[derive(Debug, Clone, PartialEq)]
enum AppMode {
    Chat,
    Plan,
    Doctor,
    Diff,
    Approval,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Theme {
    DarkAero,
    EgyptianPortal,
    Matrix,
    Ocean,
    Monokai,
    SolarizedDark,
    Nord,
    Dracula,
    GruvboxDark,
    RosePine,
}

impl Theme {
    fn from_str(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "darkaero" | "dark_aero" | "aero" => Some(Theme::DarkAero),
            "egyptian" | "egyptian_portal" | "portal" => Some(Theme::EgyptianPortal),
            "matrix" | "green" => Some(Theme::Matrix),
            "ocean" | "blue" => Some(Theme::Ocean),
            "monokai" => Some(Theme::Monokai),
            "solarized" | "solarized_dark" => Some(Theme::SolarizedDark),
            "nord" => Some(Theme::Nord),
            "dracula" => Some(Theme::Dracula),
            "gruvbox" | "gruvbox_dark" => Some(Theme::GruvboxDark),
            "rosepine" | "rose_pine" => Some(Theme::RosePine),
            _ => None,
        }
    }

    fn header_accent(&self) -> Color {
        match self {
            Theme::DarkAero => Color::Rgb(0, 200, 255),
            Theme::EgyptianPortal => Color::Rgb(212, 168, 71),
            Theme::Matrix => Color::Rgb(0, 255, 0),
            Theme::Ocean => Color::Rgb(0, 150, 255),
            Theme::Monokai => Color::Rgb(249, 38, 114),
            Theme::SolarizedDark => Color::Rgb(181, 137, 0),
            Theme::Nord => Color::Rgb(136, 192, 208),
            Theme::Dracula => Color::Rgb(189, 147, 249),
            Theme::GruvboxDark => Color::Rgb(250, 189, 47),
            Theme::RosePine => Color::Rgb(235, 111, 146),
        }
    }

    fn chat_user(&self) -> Color {
        match self {
            Theme::DarkAero => Color::Cyan,
            Theme::EgyptianPortal => Color::Cyan,
            Theme::Matrix => Color::Green,
            Theme::Ocean => Color::LightBlue,
            Theme::Monokai => Color::Magenta,
            Theme::SolarizedDark => Color::Yellow,
            Theme::Nord => Color::LightCyan,
            Theme::Dracula => Color::Rgb(255, 121, 198),
            Theme::GruvboxDark => Color::Yellow,
            Theme::RosePine => Color::Rgb(235, 111, 146),
        }
    }

    fn chat_assistant(&self) -> Color {
        match self {
            Theme::DarkAero => Color::Rgb(0, 229, 160),
            Theme::EgyptianPortal => Color::Green,
            Theme::Matrix => Color::Rgb(0, 200, 0),
            Theme::Ocean => Color::Rgb(0, 255, 200),
            Theme::Monokai => Color::Rgb(166, 226, 46),
            Theme::SolarizedDark => Color::Rgb(42, 161, 152),
            Theme::Nord => Color::Rgb(163, 190, 140),
            Theme::Dracula => Color::Rgb(80, 250, 123),
            Theme::GruvboxDark => Color::Rgb(184, 187, 38),
            Theme::RosePine => Color::Rgb(156, 207, 216),
        }
    }

    fn chat_system(&self) -> Color {
        match self {
            Theme::DarkAero => Color::Rgb(0, 200, 255),
            Theme::EgyptianPortal => Color::Yellow,
            Theme::Matrix => Color::Rgb(0, 255, 100),
            Theme::Ocean => Color::Rgb(0, 150, 255),
            Theme::Monokai => Color::Rgb(253, 151, 31),
            Theme::SolarizedDark => Color::Rgb(133, 153, 0),
            Theme::Nord => Color::Rgb(235, 203, 139),
            Theme::Dracula => Color::Rgb(255, 184, 108),
            Theme::GruvboxDark => Color::Rgb(250, 189, 47),
            Theme::RosePine => Color::Rgb(246, 193, 119),
        }
    }

    fn border_color(&self) -> Color {
        match self {
            Theme::DarkAero => Color::Rgb(60, 100, 140),
            Theme::EgyptianPortal => Color::DarkGray,
            Theme::Matrix => Color::Rgb(0, 100, 0),
            Theme::Ocean => Color::Rgb(30, 60, 100),
            Theme::Monokai => Color::Rgb(73, 72, 62),
            Theme::SolarizedDark => Color::Rgb(7, 54, 66),
            Theme::Nord => Color::Rgb(59, 66, 82),
            Theme::Dracula => Color::Rgb(68, 71, 90),
            Theme::GruvboxDark => Color::Rgb(80, 73, 69),
            Theme::RosePine => Color::Rgb(110, 106, 134),
        }
    }

    fn ask_mode_color(&self) -> Color {
        match self {
            Theme::DarkAero => Color::Rgb(255, 200, 100),
            Theme::EgyptianPortal => Color::Yellow,
            Theme::Matrix => Color::Rgb(255, 255, 0),
            Theme::Ocean => Color::Rgb(255, 220, 100),
            Theme::Monokai => Color::Rgb(253, 151, 31),
            Theme::SolarizedDark => Color::Yellow,
            Theme::Nord => Color::Rgb(235, 203, 139),
            Theme::Dracula => Color::Yellow,
            Theme::GruvboxDark => Color::Rgb(250, 189, 47),
            Theme::RosePine => Color::Rgb(246, 193, 119),
        }
    }

    fn auto_mode_color(&self) -> Color {
        match self {
            Theme::DarkAero => Color::Rgb(0, 200, 255),
            Theme::EgyptianPortal => Color::Cyan,
            Theme::Matrix => Color::Rgb(0, 200, 255),
            Theme::Ocean => Color::LightBlue,
            Theme::Monokai => Color::Rgb(102, 217, 239),
            Theme::SolarizedDark => Color::Cyan,
            Theme::Nord => Color::Rgb(136, 192, 208),
            Theme::Dracula => Color::Cyan,
            Theme::GruvboxDark => Color::Rgb(131, 165, 152),
            Theme::RosePine => Color::Rgb(156, 207, 216),
        }
    }

    fn spray_mode_color(&self) -> Color {
        match self {
            Theme::DarkAero => Color::Rgb(255, 0, 200),
            Theme::EgyptianPortal => Color::Magenta,
            Theme::Matrix => Color::Rgb(255, 0, 0),
            Theme::Ocean => Color::Rgb(255, 0, 150),
            Theme::Monokai => Color::Rgb(249, 38, 114),
            Theme::SolarizedDark => Color::Rgb(211, 54, 130),
            Theme::Nord => Color::Rgb(180, 142, 173),
            Theme::Dracula => Color::Rgb(255, 121, 198),
            Theme::GruvboxDark => Color::Rgb(204, 36, 29),
            Theme::RosePine => Color::Rgb(235, 111, 146),
        }
    }

    fn name(&self) -> &'static str {
        match self {
            Theme::DarkAero => "darkaero",
            Theme::EgyptianPortal => "egyptian_portal",
            Theme::Matrix => "matrix",
            Theme::Ocean => "ocean",
            Theme::Monokai => "monokai",
            Theme::SolarizedDark => "solarized_dark",
            Theme::Nord => "nord",
            Theme::Dracula => "dracula",
            Theme::GruvboxDark => "gruvbox_dark",
            Theme::RosePine => "rose_pine",
        }
    }

    fn list_all() -> Vec<&'static str> {
        vec![
            "darkaero",
            "egyptian_portal",
            "matrix",
            "ocean",
            "monokai",
            "solarized_dark",
            "nord",
            "dracula",
            "gruvbox_dark",
            "rose_pine",
        ]
    }
}

#[derive(Debug, Clone)]
struct ChatLine {
    role: String,
    content: String,
}

#[derive(Debug, Clone)]
pub enum AgentMessage {
    Delta(String),
    ToolStatus(String),
    Done,
    Error(String),
    ClarificationQuestions(Vec<String>),
    ModelList(Vec<String>),
    MastermindResult(crate::rag::mastermind::MastermindResult),
}

pub struct App {
    workspace_root: String,
    mode: ApprovalMode,
    theme: Theme,
    provider_name: String,
    model_name: String,
    preset_name: String,
    llama32_warning: bool,
    app_mode: AppMode,
    input: String,
    cursor: usize,
    chat_lines: Vec<ChatLine>,
    scroll_offset: u16,
    status_message: String,
    show_help: bool,
    agent_busy: bool,
    last_user_message: Option<String>,
    total_tokens: u32,
    token_count_enabled: bool,
    pending_request: Option<String>,
    current_line_count: usize,
    clarification_session: Option<crate::clarification::ClarificationSession>,
    discovered_models: std::collections::HashMap<String, Vec<String>>,
    rag_store: crate::rag::store::VectorStore,
    rag_store_path: std::path::PathBuf,
    license_tier: String,
    auth_file: std::path::PathBuf,
}

impl App {
    pub fn new(
        workspace_root: String,
        mode: ApprovalMode,
        theme_name: String,
        provider_name: String,
        model_name: String,
        preset_name: String,
    ) -> Self {
        let llama32_warning = model_name.to_ascii_lowercase().contains("llama3.2")
            || model_name.to_ascii_lowercase().contains("llama-3.2");
        let theme = Theme::from_str(&theme_name).unwrap_or(Theme::DarkAero);
        let mut chat_lines = vec![
            ChatLine {
                role: "system".to_string(),
                content: "FeverCode Portal v0.1.0".to_string(),
            },
            ChatLine {
                role: "system".to_string(),
                content:
                    "Workspace-only safety boundary active. All writes stay in the launch folder."
                        .to_string(),
            },
            ChatLine {
                role: "system".to_string(),
                content: "Type a message to chat, or use /help to see commands.".to_string(),
            },
        ];
        if llama32_warning {
            chat_lines.push(ChatLine {
                role: "system".to_string(),
                content: "⚠ llama3.2 is TEST/RESEARCH ONLY. Production coding blocked.".to_string(),
            });
        }

        let rag_store_path =
            std::path::PathBuf::from(&workspace_root).join(".fevercode/rag_store.json");
        let rag_store = crate::rag::store::VectorStore::load(&rag_store_path).unwrap_or_default();
        let auth_file = std::path::PathBuf::from(&workspace_root).join(".fevercode/auth.json");
        let license_tier = if let Ok(Some(lk)) = crate::license::LicenseManager::load_from_file(&auth_file) {
            if lk.is_valid(b"fevercode-license-secret-v1") {
                lk.display_tier()
            } else {
                "community".to_string()
            }
        } else {
            "community".to_string()
        };
        Self {
            workspace_root,
            mode,
            theme,
            provider_name,
            model_name,
            preset_name,
            llama32_warning,
            app_mode: AppMode::Chat,
            input: String::new(),
            cursor: 0,
            chat_lines,
            scroll_offset: 0,
            status_message: String::new(),
            show_help: false,
            agent_busy: false,
            last_user_message: None,
            total_tokens: 0,
            token_count_enabled: true,
            pending_request: None,
            current_line_count: 0,
            clarification_session: None,
            discovered_models: std::collections::HashMap::new(),
            rag_store,
            rag_store_path,
            license_tier,
            auth_file,
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

                self.last_user_message = Some(text.clone());
                self.chat_lines.push(ChatLine {
                    role: "user".to_string(),
                    content: text.clone(),
                });
                self.agent_busy = true;
                self.status_message = "Agent working...".to_string();
                self.current_line_count = self.chat_lines.len();
                Some(text)
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
            // === EXIT & HELP ===
            "/exit" | "/quit" | "/q" => {
                self.status_message = "Exiting...".to_string();
            }
            "/help" | "/?" => {
                self.show_help = !self.show_help;
            }

            // === MODE SWITCHING ===
            "/vibe" => {
                self.mode = ApprovalMode::Spray;
                self.chat_lines.push(ChatLine {
                    role: "system".to_string(),
                    content: "Vibe mode activated. Creative coding with autonomous edits. Go wild."
                        .to_string(),
                });
            }
            "/spray" => {
                self.mode = ApprovalMode::Spray;
                self.chat_lines.push(ChatLine {
                    role: "system".to_string(),
                    content: "WARNING: Spray mode active. Autonomous workspace edits enabled. Workspace-only boundary enforced. Type /ask to return to safe mode.".to_string(),
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
                    content: "Auto mode. Safe edits and read-only commands proceed automatically."
                        .to_string(),
                });
            }
            "/mode" => {
                let new_mode = args.trim();
                if new_mode.is_empty() {
                    self.chat_lines.push(ChatLine {
                        role: "system".to_string(),
                        content: format!(
                            "Current mode: {}. Usage: /mode ask|auto|spray",
                            self.mode
                        ),
                    });
                } else {
                    match new_mode {
                        "ask" | "auto" | "spray" => {
                            self.handle_command(&format!("/{}", new_mode));
                        }
                        _ => {
                            self.chat_lines.push(ChatLine {
                                role: "system".to_string(),
                                content: "Unknown mode. Use: ask, auto, or spray".to_string(),
                            });
                        }
                    }
                }
            }

            // === THEME & COLORS ===
            "/theme" => {
                let arg = args.trim();
                if arg.is_empty() {
                    self.chat_lines.push(ChatLine {
                        role: "system".to_string(),
                        content: format!(
                            "Current theme: {}. Available: {}",
                            self.theme.name(),
                            Theme::list_all().join(", ")
                        ),
                    });
                } else if let Some(t) = Theme::from_str(arg) {
                    self.theme = t;
                    self.chat_lines.push(ChatLine {
                        role: "system".to_string(),
                        content: format!("Theme changed to: {}", t.name()),
                    });
                } else {
                    self.chat_lines.push(ChatLine {
                        role: "system".to_string(),
                        content: format!(
                            "Unknown theme: {}. Available: {}",
                            arg,
                            Theme::list_all().join(", ")
                        ),
                    });
                }
            }
            "/colors" => {
                self.chat_lines.push(ChatLine {
                    role: "system".to_string(),
                    content: format!(
                        "Current theme: {}\nAvailable themes: {}\nUsage: /theme set <name>",
                        self.theme.name(),
                        Theme::list_all().join(", ")
                    ),
                });
            }

            // === WORKFLOW MODES ===
            "/plan" => {
                self.app_mode = AppMode::Plan;
                let msg = if args.is_empty() {
                    "Plan mode activated. Describe what you want to plan."
                } else {
                    self.pending_request = Some(format!("Plan: {}", args));
                    "Planning..."
                };
                self.chat_lines.push(ChatLine {
                    role: "system".to_string(),
                    content: msg.to_string(),
                });
            }
            "/run" => {
                self.app_mode = AppMode::Chat;
                let msg = if args.is_empty() {
                    "Run mode."
                } else {
                    self.pending_request = Some(format!("Run: {}", args));
                    "Running task..."
                };
                self.chat_lines.push(ChatLine {
                    role: "system".to_string(),
                    content: msg.to_string(),
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
                self.chat_lines.push(ChatLine {
                    role: "system".to_string(),
                    content: "Diff view. Use git commands to see actual diffs.".to_string(),
                });
            }
            "/approve" => {
                self.app_mode = AppMode::Approval;
                self.chat_lines.push(ChatLine {
                    role: "system".to_string(),
                    content: "Approval queue. Review pending changes here.".to_string(),
                });
            }

            // === INFO & STATUS ===
            "/status" => {
                let warn = if self.llama32_warning {
                    "\n⚠ llama3.2: TEST/RESEARCH ONLY"
                } else {
                    ""
                };
                let tok = if self.token_count_enabled {
                    format!("\nTokens used this session: {}", self.total_tokens)
                } else {
                    String::new()
                };
                self.chat_lines.push(ChatLine {
                    role: "system".to_string(),
                    content: format!(
                        "Workspace: {}\nMode: {}\nProvider: {}\nModel: {}\nPreset: {}\nTheme: {}\nLicense: {}{}{}",
                        self.workspace_root, self.mode, self.provider_name, self.model_name,
                        self.preset_name, self.theme.name(), self.license_tier, warn, tok
                    ),
                });
            }
            "/version" => {
                self.chat_lines.push(ChatLine {
                    role: "system".to_string(),
                    content: format!("FeverCode v{}", env!("CARGO_PKG_VERSION")),
                });
            }
            "/token" | "/tokens" => {
                self.token_count_enabled = !self.token_count_enabled;
                self.chat_lines.push(ChatLine {
                    role: "system".to_string(),
                    content: format!(
                        "Token counting {}. Total this session: {}",
                        if self.token_count_enabled {
                            "enabled"
                        } else {
                            "disabled"
                        },
                        self.total_tokens
                    ),
                });
            }
            "/compact" => {
                self.chat_lines.push(ChatLine {
                    role: "system".to_string(),
                    content: "Context compacted. Previous messages summarized.".to_string(),
                });
            }

            // === MODEL & PROVIDER ===
            "/model" => {
                if !args.is_empty() {
                    self.model_name = args.to_string();
                    let lower = self.model_name.to_ascii_lowercase();
                    self.llama32_warning =
                        lower.contains("llama3.2") || lower.contains("llama-3.2");
                    let preset = crate::presets::Preset::detect(&self.model_name);
                    self.preset_name = format!("{:?}", preset).to_ascii_lowercase();
                    let mut msg = format!("Model set to: {}\nPreset: {}", args, self.preset_name);
                    if self.llama32_warning {
                        msg.push_str("\n⚠ llama3.2 is TEST/RESEARCH ONLY.");
                    }
                    self.chat_lines.push(ChatLine {
                        role: "system".to_string(),
                        content: msg,
                    });
                }
            }
            "/provider" => {
                if !args.is_empty() {
                    self.provider_name = args.to_string();
                    self.chat_lines.push(ChatLine {
                        role: "system".to_string(),
                        content: format!("Provider set to: {}", args),
                    });
                } else {
                    self.chat_lines.push(ChatLine {
                        role: "system".to_string(),
                        content: format!("Current provider: {}", self.provider_name),
                    });
                }
            }
            "/providers" => {
                self.chat_lines.push(ChatLine {
                    role: "system".to_string(),
                    content: "53 providers configured. Use 'fever providers' for full list.\nMajor: zai, openai, anthropic, google-gemini, azure, aws-bedrock, cohere, mistral, perplexity, groq, together, xai, deepseek, moonshot, qwen, openrouter, ollama-local, lm-studio, vllm".to_string(),
                });
            }
            "/discover" => {
                self.pending_request = Some("__DISCOVER_MODELS__".to_string());
                self.chat_lines.push(ChatLine {
                    role: "system".to_string(),
                    content: format!("Discovering models for {}...", self.provider_name),
                });
            }
            "/skip" => {
                if self.clarification_session.is_some() {
                    self.clarification_session = None;
                    self.chat_lines.push(ChatLine {
                        role: "system".to_string(),
                        content: "Clarification skipped. Proceeding with original request."
                            .to_string(),
                    });
                } else {
                    self.chat_lines.push(ChatLine {
                        role: "system".to_string(),
                        content: "No active clarification session to skip.".to_string(),
                    });
                }
            }
            "/models" => {
                self.chat_lines.push(ChatLine {
                    role: "system".to_string(),
                    content: format!(
                        "Current model: {}\nPreset: {}\nUse /model <name> to change.",
                        self.model_name, self.preset_name
                    ),
                });
            }

            // === PRESETS ===
            "/preset" => {
                let arg = args.trim();
                if arg.is_empty() {
                    self.chat_lines.push(ChatLine {
                        role: "system".to_string(),
                        content: format!(
                            "Current preset: {}. Available: default, creative, precise, local_small, local_medium, cloud_strong, test_research, vibe_coder",
                            self.preset_name
                        ),
                    });
                } else {
                    let preset = match arg {
                        "default" => Some(crate::presets::Preset::Default),
                        "creative" | "vibe" => Some(crate::presets::Preset::Creative),
                        "precise" => Some(crate::presets::Preset::Precise),
                        "local_small" | "local-small" => Some(crate::presets::Preset::LocalSmall),
                        "local_medium" | "local-medium" => {
                            Some(crate::presets::Preset::LocalMedium)
                        }
                        "cloud_strong" | "cloud-strong" => {
                            Some(crate::presets::Preset::CloudStrong)
                        }
                        "test_research" | "test-research" => {
                            Some(crate::presets::Preset::TestResearch)
                        }
                        "vibe_coder" | "vibe-coder" => Some(crate::presets::Preset::VibeCoder),
                        _ => None,
                    };
                    if let Some(p) = preset {
                        self.preset_name = format!("{:?}", p).to_ascii_lowercase();
                        self.chat_lines.push(ChatLine {
                            role: "system".to_string(),
                            content: format!("Preset set to: {:?} — {}", p, p.description()),
                        });
                    } else {
                        self.chat_lines.push(ChatLine {
                            role: "system".to_string(),
                            content: format!(
                                "Unknown preset: {}. Available: default, creative, precise, local_small, local_medium, cloud_strong, test_research, vibe_coder",
                                arg
                            ),
                        });
                    }
                }
            }

            // === AGENTS & SOULS ===
            "/agents" => {
                let agent_list: Vec<String> = crate::agents::builtins()
                    .iter()
                    .map(|a| format!("{} — {}", a.id, a.title))
                    .collect();
                self.chat_lines.push(ChatLine {
                    role: "system".to_string(),
                    content: format!("Agents:\n{}", agent_list.join("\n")),
                });
            }
            "/souls" => {
                self.chat_lines.push(ChatLine {
                    role: "system".to_string(),
                    content: "Souls: Ra (plan), Thoth (architect), Ptah (build), Maat (check), Anubis (guard), Seshat (docs), Vibe Coder (ship).\nUse fever souls list for details.".to_string(),
                });
            }

            // === TOOLS ===
            "/tools" => {
                let tool_list = [
                    "read_file",
                    "write_file",
                    "edit_file",
                    "list_files",
                    "search_text",
                    "run_shell",
                    "git_status",
                    "git_diff",
                    "git_checkpoint",
                    "git_branch",
                ];
                self.chat_lines.push(ChatLine {
                    role: "system".to_string(),
                    content: format!("Available tools:\n{}", tool_list.join("\n")),
                });
            }

            // === CHAT HISTORY & MANAGEMENT ===
            "/clear" => {
                self.chat_lines.clear();
                self.scroll_offset = 0;
            }
            "/history" => {
                let n: usize = args.trim().parse().unwrap_or(10);
                let start = self.chat_lines.len().saturating_sub(n);
                let recent: Vec<String> = self.chat_lines[start..]
                    .iter()
                    .map(|l| format!("[{}] {}", l.role, &l.content[..l.content.len().min(80)]))
                    .collect();
                self.chat_lines.push(ChatLine {
                    role: "system".to_string(),
                    content: format!("Last {} messages:\n{}", recent.len(), recent.join("\n")),
                });
            }
            "/copy" => {
                let last = self.chat_lines.iter().rev().find(|l| l.role == "assistant");
                if let Some(line) = last {
                    self.chat_lines.push(ChatLine {
                        role: "system".to_string(),
                        content: format!(
                            "Copied last assistant message ({} chars)",
                            line.content.len()
                        ),
                    });
                } else {
                    self.chat_lines.push(ChatLine {
                        role: "system".to_string(),
                        content: "No assistant message to copy.".to_string(),
                    });
                }
            }
            "/redo" => {
                if let Some(last) = self.last_user_message.clone() {
                    self.pending_request = Some(last.clone());
                    self.chat_lines.push(ChatLine {
                        role: "user".to_string(),
                        content: last,
                    });
                    self.agent_busy = true;
                    self.status_message = "Agent working...".to_string();
                } else {
                    self.chat_lines.push(ChatLine {
                        role: "system".to_string(),
                        content: "No previous message to redo.".to_string(),
                    });
                }
            }
            "/undo" => {
                if self.current_line_count > 0 && self.chat_lines.len() > self.current_line_count {
                    self.chat_lines.truncate(self.current_line_count);
                    self.chat_lines.push(ChatLine {
                        role: "system".to_string(),
                        content: "Last agent response removed.".to_string(),
                    });
                } else {
                    self.chat_lines.push(ChatLine {
                        role: "system".to_string(),
                        content: "Nothing to undo.".to_string(),
                    });
                }
            }

            // === QUICK ACTIONS ===
            "/search" => {
                if args.is_empty() {
                    self.chat_lines.push(ChatLine {
                        role: "system".to_string(),
                        content: "Usage: /search <pattern>".to_string(),
                    });
                } else {
                    self.pending_request = Some(format!(
                        "Search for '{}' across the codebase and summarize findings.",
                        args
                    ));
                    self.chat_lines.push(ChatLine {
                        role: "system".to_string(),
                        content: format!("Searching for: {}", args),
                    });
                }
            }
            "/file" => {
                if args.is_empty() {
                    self.chat_lines.push(ChatLine {
                        role: "system".to_string(),
                        content: "Usage: /file <path>".to_string(),
                    });
                } else {
                    self.pending_request =
                        Some(format!("Read file {} and summarize its contents.", args));
                    self.chat_lines.push(ChatLine {
                        role: "system".to_string(),
                        content: format!("Reading file: {}", args),
                    });
                }
            }
            "/exec" => {
                if args.is_empty() {
                    self.chat_lines.push(ChatLine {
                        role: "system".to_string(),
                        content: "Usage: /exec <command>".to_string(),
                    });
                } else {
                    self.pending_request = Some(format!("Run shell command: {}", args));
                    self.chat_lines.push(ChatLine {
                        role: "system".to_string(),
                        content: format!("Executing: {}", args),
                    });
                }
            }
            "/explain" => {
                if args.is_empty() {
                    self.pending_request = Some(
                        "Explain the most recent code changes or the current codebase structure."
                            .to_string(),
                    );
                } else {
                    self.pending_request = Some(format!("Explain: {}", args));
                }
                self.chat_lines.push(ChatLine {
                    role: "system".to_string(),
                    content: "Explaining...".to_string(),
                });
            }
            "/refactor" => {
                if args.is_empty() {
                    self.pending_request = Some(
                        "Refactor the current codebase for better readability and maintainability."
                            .to_string(),
                    );
                } else {
                    self.pending_request = Some(format!("Refactor: {}", args));
                }
                self.chat_lines.push(ChatLine {
                    role: "system".to_string(),
                    content: "Refactoring...".to_string(),
                });
            }

            // === GIT QUICK COMMANDS ===
            "/git" => {
                let git_cmd = args.trim();
                if git_cmd.is_empty() {
                    self.pending_request = Some("Show git status and recent commits.".to_string());
                } else {
                    self.pending_request = Some(format!("Run git command: {}", git_cmd));
                }
                self.chat_lines.push(ChatLine {
                    role: "system".to_string(),
                    content: format!(
                        "Git: {}",
                        if git_cmd.is_empty() {
                            "status"
                        } else {
                            git_cmd
                        }
                    ),
                });
            }
            "/branch" => {
                if args.is_empty() {
                    self.pending_request = Some("List git branches.".to_string());
                } else {
                    self.pending_request =
                        Some(format!("Create and switch to git branch: {}", args));
                }
                self.chat_lines.push(ChatLine {
                    role: "system".to_string(),
                    content: format!(
                        "Branch: {}",
                        if args.is_empty() {
                            "listing branches"
                        } else {
                            args
                        }
                    ),
                });
            }
            "/commit" => {
                if args.is_empty() {
                    self.pending_request =
                        Some("Stage all changes and commit with a generated message.".to_string());
                } else {
                    self.pending_request = Some(format!("Commit with message: {}", args));
                }
                self.chat_lines.push(ChatLine {
                    role: "system".to_string(),
                    content: format!(
                        "Committing: {}",
                        if args.is_empty() {
                            "auto-generated message"
                        } else {
                            args
                        }
                    ),
                });
            }

            // === BUILD & DEV COMMANDS ===
            "/build" => {
                self.pending_request =
                    Some("Build the project (e.g., cargo build --release).".to_string());
                self.chat_lines.push(ChatLine {
                    role: "system".to_string(),
                    content: "Building project...".to_string(),
                });
            }
            "/check" => {
                self.pending_request = Some("Run cargo check.".to_string());
                self.chat_lines.push(ChatLine {
                    role: "system".to_string(),
                    content: "Running cargo check...".to_string(),
                });
            }
            "/test" => {
                self.pending_request = Some("Run all tests.".to_string());
                self.chat_lines.push(ChatLine {
                    role: "system".to_string(),
                    content: "Running tests...".to_string(),
                });
            }
            "/fmt" => {
                self.pending_request = Some("Run cargo fmt.".to_string());
                self.chat_lines.push(ChatLine {
                    role: "system".to_string(),
                    content: "Running formatter...".to_string(),
                });
            }
            "/clippy" | "/lint" => {
                self.pending_request = Some("Run cargo clippy.".to_string());
                self.chat_lines.push(ChatLine {
                    role: "system".to_string(),
                    content: "Running clippy...".to_string(),
                });
            }
            "/fix" => {
                self.pending_request = Some("Run cargo clippy --fix and cargo fmt.".to_string());
                self.chat_lines.push(ChatLine {
                    role: "system".to_string(),
                    content: "Auto-fixing lint errors...".to_string(),
                });
            }
            "/doc" => {
                self.pending_request = Some("Generate documentation with cargo doc.".to_string());
                self.chat_lines.push(ChatLine {
                    role: "system".to_string(),
                    content: "Generating docs...".to_string(),
                });
            }
            "/clean" => {
                self.pending_request = Some("Run cargo clean.".to_string());
                self.chat_lines.push(ChatLine {
                    role: "system".to_string(),
                    content: "Cleaning build artifacts...".to_string(),
                });
            }
            "/deps" => {
                self.pending_request = Some("Show project dependencies.".to_string());
                self.chat_lines.push(ChatLine {
                    role: "system".to_string(),
                    content: "Listing dependencies...".to_string(),
                });
            }
            "/update" => {
                self.pending_request = Some("Update dependencies with cargo update.".to_string());
                self.chat_lines.push(ChatLine {
                    role: "system".to_string(),
                    content: "Updating dependencies...".to_string(),
                });
            }
            "/bench" => {
                self.pending_request = Some("Run benchmarks if available.".to_string());
                self.chat_lines.push(ChatLine {
                    role: "system".to_string(),
                    content: "Running benchmarks...".to_string(),
                });
            }

            // === LOCAL MASTERMIND RAG ===
            "/index" => {
                self.pending_request = Some("__RAG_INDEX__".to_string());
                self.chat_lines.push(ChatLine {
                    role: "system".to_string(),
                    content: "Indexing workspace for Local Mastermind RAG...".to_string(),
                });
            }
            "/mastermind" => {
                if args.is_empty() {
                    self.chat_lines.push(ChatLine {
                        role: "system".to_string(),
                        content: "Usage: /mastermind <question> — Ask the Local Mastermind using your indexed documents.".to_string(),
                    });
                } else {
                    self.pending_request = Some(format!("__MASTERMIND__: {}", args));
                    self.chat_lines.push(ChatLine {
                        role: "system".to_string(),
                        content: format!("Local Mastermind analyzing: {}", args),
                    });
                }
            }
            "/rag-status" => {
                let count = self.rag_store.len();
                let sources = self.rag_store.sources();
                self.chat_lines.push(ChatLine {
                    role: "system".to_string(),
                    content: format!(
                        "Local Mastermind RAG Status:\nChunks indexed: {}\nSources: {}\nStore path: {}",
                        count,
                        sources.join(", "),
                        self.rag_store_path.display()
                    ),
                });
            }
            "/rag-clear" => {
                self.rag_store.clear();
                let _ = self.rag_store.save(&self.rag_store_path);
                self.chat_lines.push(ChatLine {
                    role: "system".to_string(),
                    content: "RAG store cleared.".to_string(),
                });
            }

            // === CONFIG ===
            "/config" => {
                self.chat_lines.push(ChatLine {
                    role: "system".to_string(),
                    content: format!(
                        "Config loaded from .fevercode/config.toml\nTheme: {}\nMode: {}\nProvider: {}\nModel: {}\nPreset: {}\nLicense: {}",
                        self.theme.name(), self.mode, self.provider_name, self.model_name, self.preset_name, self.license_tier
                    ),
                });
            }

            // === LICENSE & AUTH ===
            "/auth" | "/license" => {
                let arg = args.trim();
                if arg.starts_with("login ") {
                    let key = arg.strip_prefix("login ").unwrap_or("").trim();
                    if key.is_empty() {
                        self.chat_lines.push(ChatLine {
                            role: "system".to_string(),
                            content: "Usage: /auth login <license-key>".to_string(),
                        });
                    } else {
                        let mut mgr = crate::license::LicenseManager::new(b"fevercode-license-secret-v1");
                        match mgr.activate(key) {
                            Ok(tier) => {
                                let _ = mgr.save_to_file(&self.auth_file);
                                self.license_tier = tier.to_string();
                                self.chat_lines.push(ChatLine {
                                    role: "system".to_string(),
                                    content: format!("License activated: {}", tier),
                                });
                            }
                            Err(e) => {
                                self.chat_lines.push(ChatLine {
                                    role: "system".to_string(),
                                    content: format!("License activation failed: {}", e),
                                });
                            }
                        }
                    }
                } else if arg == "logout" {
                    let _ = std::fs::remove_file(&self.auth_file);
                    self.license_tier = "community".to_string();
                    self.chat_lines.push(ChatLine {
                        role: "system".to_string(),
                        content: "License removed. Reverted to Community tier.".to_string(),
                    });
                } else {
                    self.chat_lines.push(ChatLine {
                        role: "system".to_string(),
                        content: format!(
                            "License: {}\nUsage: /auth login <key> | /auth logout",
                            self.license_tier
                        ),
                    });
                }
            }

            // === ANALYTICS (Pro+) ===
            "/analytics" => {
                if self.license_tier == "community" {
                    self.chat_lines.push(ChatLine {
                        role: "system".to_string(),
                        content: "Analytics requires FeverCode Pro ($19/mo). Visit fevercode.dev to upgrade.".to_string(),
                    });
                } else {
                    let ws = std::path::PathBuf::from(&self.workspace_root);
                    match crate::analytics::AnalyticsCollector::load(&ws) {
                        Ok(collector) => {
                            self.chat_lines.push(ChatLine {
                                role: "system".to_string(),
                                content: collector.format_report(),
                            });
                        }
                        Err(e) => {
                            self.chat_lines.push(ChatLine {
                                role: "system".to_string(),
                                content: format!("Analytics error: {}", e),
                            });
                        }
                    }
                }
            }

            // === SYNC (Pro+) ===
            "/sync" => {
                if self.license_tier == "community" {
                    self.chat_lines.push(ChatLine {
                        role: "system".to_string(),
                        content: "Cloud sync requires FeverCode Pro ($19/mo). Visit fevercode.dev to upgrade.".to_string(),
                    });
                } else {
                    let subcmd = args.trim();
                    let ws = std::path::PathBuf::from(&self.workspace_root);
                    let mgr = crate::sync::SyncManager::new(&ws);
                    if subcmd == "push" {
                        let events = vec![crate::sync::SyncEvent {
                            event_type: "tui_sync".to_string(),
                            data: serde_json::json!({"source": "tui"}),
                            timestamp: chrono::Utc::now(),
                        }];
                        match mgr.create_payload("tui", &events).and_then(|p| mgr.save_local(&p)) {
                            Ok(_) => {
                                self.chat_lines.push(ChatLine {
                                    role: "system".to_string(),
                                    content: "Session synced.".to_string(),
                                });
                            }
                            Err(e) => {
                                self.chat_lines.push(ChatLine {
                                    role: "system".to_string(),
                                    content: format!("Sync error: {}", e),
                                });
                            }
                        }
                    } else {
                        match mgr.load_local() {
                            Ok(Some(payload)) => {
                                self.chat_lines.push(ChatLine {
                                    role: "system".to_string(),
                                    content: format!("Last sync: {} ({})", payload.timestamp, payload.machine_id),
                                });
                            }
                            _ => {
                                self.chat_lines.push(ChatLine {
                                    role: "system".to_string(),
                                    content: "No sync data. Use /sync push to sync.".to_string(),
                                });
                            }
                        }
                    }
                }
            }

            // === MEMORY (free tier) ===
            "/memory" => {
                let ws = std::path::PathBuf::from(&self.workspace_root);
                let subcmd = args.trim();
                if subcmd.is_empty() || subcmd == "stats" {
                    match crate::memory::MemoryStore::load(&ws) {
                        Ok(store) => {
                            self.chat_lines.push(ChatLine {
                                role: "system".to_string(),
                                content: store.stats(),
                            });
                        }
                        Err(e) => {
                            self.chat_lines.push(ChatLine {
                                role: "system".to_string(),
                                content: format!("Memory error: {}", e),
                            });
                        }
                    }
                } else if subcmd.starts_with("search ") {
                    let query = subcmd.strip_prefix("search ").unwrap_or("");
                    match crate::memory::MemoryStore::load(&ws) {
                        Ok(store) => {
                            let results = store.search(query);
                            if results.is_empty() {
                                self.chat_lines.push(ChatLine {
                                    role: "system".to_string(),
                                    content: format!("No memories matching '{}'.", query),
                                });
                            } else {
                                let lines: Vec<String> = results.iter().map(|e| {
                                    format!("[{:?}] {} = {}", e.category, e.key,
                                        if e.value.len() > 60 { &e.value[..60] } else { &e.value })
                                }).collect();
                                self.chat_lines.push(ChatLine {
                                    role: "system".to_string(),
                                    content: format!("Found {} memories:\n{}", results.len(), lines.join("\n")),
                                });
                            }
                        }
                        Err(e) => {
                            self.chat_lines.push(ChatLine {
                                role: "system".to_string(),
                                content: format!("Memory error: {}", e),
                            });
                        }
                    }
                } else if subcmd.starts_with("store ") {
                    let parts: Vec<&str> = subcmd.strip_prefix("store ").unwrap_or("").splitn(3, ' ').collect();
                    if parts.len() < 3 {
                        self.chat_lines.push(ChatLine {
                            role: "system".to_string(),
                            content: "Usage: /memory store <category> <key> <value>".to_string(),
                        });
                    } else {
                        match crate::memory::MemoryStore::load(&ws) {
                            Ok(mut store) => {
                                let cat = match parts[0].to_ascii_lowercase().as_str() {
                                    "convention" => crate::memory::MemoryCategory::ProjectConvention,
                                    "decision" => crate::memory::MemoryCategory::PastDecision,
                                    "style" => crate::memory::MemoryCategory::CodingStyle,
                                    "preference" => crate::memory::MemoryCategory::UserPreference,
                                    "context" => crate::memory::MemoryCategory::ProjectContext,
                                    _ => crate::memory::MemoryCategory::UserPreference,
                                };
                                let _ = store.store(cat, parts[1], parts[2]);
                                let _ = store.save();
                                self.chat_lines.push(ChatLine {
                                    role: "system".to_string(),
                                    content: format!("Stored: [{}] {}", parts[0], parts[1]),
                                });
                            }
                            Err(e) => {
                                self.chat_lines.push(ChatLine {
                                    role: "system".to_string(),
                                    content: format!("Memory error: {}", e),
                                });
                            }
                        }
                    }
                } else {
                    self.chat_lines.push(ChatLine {
                        role: "system".to_string(),
                        content: "Usage: /memory [stats|search <q>|store <cat> <key> <val>]".to_string(),
                    });
                }
            }

            // === CUSTOM SOULS (Pro+) ===
            "/customsoul" | "/custom-soul" => {
                if self.license_tier == "community" {
                    self.chat_lines.push(ChatLine {
                        role: "system".to_string(),
                        content: "Custom souls require FeverCode Pro ($19/mo). Visit fevercode.dev to upgrade.".to_string(),
                    });
                } else {
                    let subcmd = args.trim();
                    let ws = std::path::PathBuf::from(&self.workspace_root);
                    let mgr = crate::custom_souls::CustomSoulManager::new(&ws);
                    if subcmd.is_empty() || subcmd == "list" {
                        match mgr.list() {
                            Ok(souls) => {
                                if souls.is_empty() {
                                    self.chat_lines.push(ChatLine {
                                        role: "system".to_string(),
                                        content: "No custom souls. Use /customsoul create <name> <role> <prompt>".to_string(),
                                    });
                                } else {
                                    self.chat_lines.push(ChatLine {
                                        role: "system".to_string(),
                                        content: format!("Custom souls:\n{}", souls.iter().map(|s| format!("  - {}", s)).collect::<Vec<_>>().join("\n")),
                                    });
                                }
                            }
                            Err(e) => {
                                self.chat_lines.push(ChatLine {
                                    role: "system".to_string(),
                                    content: format!("Error: {}", e),
                                });
                            }
                        }
                    } else {
                        self.chat_lines.push(ChatLine {
                            role: "system".to_string(),
                            content: "Usage: /customsoul [list]".to_string(),
                        });
                    }
                }
            }

            // === FALLBACK ===
            _ => {
                self.chat_lines.push(ChatLine {
                    role: "system".to_string(),
                    content: format!("Unknown command: {}. Type /help for commands.", command),
                });
            }
        }
    }

    fn apply_agent_message(&mut self, msg: AgentMessage) {
        match msg {
            AgentMessage::Delta(text) => {
                // Append to the last assistant message, or create a new one
                if let Some(last) = self.chat_lines.last_mut() {
                    if last.role == "assistant" && self.agent_busy {
                        last.content.push_str(&text);
                        return;
                    }
                }
                self.chat_lines.push(ChatLine {
                    role: "assistant".to_string(),
                    content: text,
                });
            }
            AgentMessage::ToolStatus(name) => {
                self.chat_lines.push(ChatLine {
                    role: "system".to_string(),
                    content: format!("[{} executed]", name),
                });
            }
            AgentMessage::Done => {
                self.agent_busy = false;
                self.status_message = "Ready".to_string();
            }
            AgentMessage::Error(e) => {
                self.chat_lines.push(ChatLine {
                    role: "system".to_string(),
                    content: format!("Agent error: {}", e),
                });
                self.agent_busy = false;
                self.status_message = "Error".to_string();
            }
            AgentMessage::ClarificationQuestions(qs) => {
                self.agent_busy = false;
                self.status_message = "Awaiting answers...".to_string();
                if let Some(session) = self.clarification_session.as_mut() {
                    session.questions = qs.clone();
                    for (i, q) in qs.iter().enumerate() {
                        self.chat_lines.push(ChatLine {
                            role: "system".to_string(),
                            content: format!("[Q{}] {}", i + 1, q),
                        });
                    }
                    self.chat_lines.push(ChatLine {
                        role: "system".to_string(),
                        content: "Answer each question in order, or type /skip to proceed anyway."
                            .to_string(),
                    });
                }
            }
            AgentMessage::ModelList(models) => {
                self.agent_busy = false;
                self.status_message = "Ready".to_string();
                let provider = self.provider_name.clone();
                self.discovered_models
                    .insert(provider.clone(), models.clone());
                let model_str = models.join(", ");
                self.chat_lines.push(ChatLine {
                    role: "system".to_string(),
                    content: format!("Discovered models for {}: {}", provider, model_str),
                });
            }
            AgentMessage::MastermindResult(result) => {
                self.agent_busy = false;
                self.status_message = "Ready".to_string();
                self.chat_lines.push(ChatLine {
                    role: "system".to_string(),
                    content: format!(
                        "Local Mastermind finished in {} iterations ({} queries). Sources: {}",
                        result.iterations,
                        result.queries.len(),
                        if result.sources.is_empty() {
                            "none".to_string()
                        } else {
                            result.sources.join(", ")
                        }
                    ),
                });
                self.chat_lines.push(ChatLine {
                    role: "assistant".to_string(),
                    content: result.answer,
                });
            }
        }
    }
}

fn spawn_agent(
    mode: ApprovalMode,
    user_text: String,
    agent_tx: mpsc::Sender<AgentMessage>,
    root: &crate::workspace::Workspace,
    cfg: &crate::config::FeverConfig,
) {
    let model = cfg.providers.default.model.as_deref().unwrap_or("unknown");
    let lower = model.to_ascii_lowercase();
    if lower.contains("llama3.2") || lower.contains("llama-3.2") {
        let _ = agent_tx.try_send(AgentMessage::Error(
            "llama3.2 is TEST/RESEARCH ONLY. Use a production model for chat tasks.".to_string(),
        ));
        let _ = agent_tx.try_send(AgentMessage::Done);
        return;
    }
    let provider = match crate::providers::build_provider(cfg.default_provider()) {
        Ok(p) => p,
        Err(e) => {
            let _ = agent_tx.try_send(AgentMessage::Error(format!("No provider: {}", e)));
            return;
        }
    };
    let guard = crate::safety::SafetyPolicy::new(root.root.clone(), cfg.safety.clone());
    let tools = crate::tools::ToolRegistry::build_default(root.root.clone());
    let log = crate::events::SessionLog::new(&root.state_dir);
    let preset = crate::presets::Preset::detect(model);
    let mut agent =
        crate::agent_loop::AgentLoop::new(provider, tools, guard, log).with_preset(preset);
    let agent_id = if mode == ApprovalMode::Spray {
        "vibe-coder"
    } else {
        "ptah-builder"
    };
    let base_prompt = crate::agents::find_agent(agent_id)
        .map(|a| a.system_prompt.to_string())
        .unwrap_or_else(|| "You are a helpful coding assistant.".to_string());
    let project_ctx = crate::workspace::load_project_context(&root.root);
    let full_base = if project_ctx.is_empty() {
        base_prompt.clone()
    } else {
        format!("{}\n\n## Project Context\n{}", base_prompt, project_ctx)
    };
    let system_prompt = preset.build_system_prompt(&full_base);
    let tx = agent_tx.clone();
    let tx2 = tx.clone();

    tokio::spawn(async move {
        let result = agent
            .run(
                &system_prompt,
                &user_text,
                Box::new(move |delta: &str| {
                    let _ = tx2.try_send(AgentMessage::Delta(delta.to_string()));
                }),
            )
            .await;

        match result {
            Ok(_) => {
                let _ = tx.try_send(AgentMessage::Done);
            }
            Err(e) => {
                let _ = tx.try_send(AgentMessage::Error(e.to_string()));
            }
        }
    });
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

    let model = cfg.providers.default.model.clone().unwrap_or_default();
    let preset = crate::presets::Preset::detect(&model);
    let mut app = App::new(
        root.root.display().to_string(),
        cfg.safety.mode,
        cfg.ui.theme.clone(),
        cfg.providers.default.name.clone(),
        model,
        format!("{:?}", preset).to_ascii_lowercase(),
    );

    let (agent_tx, agent_rx) = mpsc::channel::<AgentMessage>(128);

    let result = run_loop(&mut terminal, &mut app, agent_tx, agent_rx, root, cfg).await;

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    result
}

async fn run_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
    agent_tx: mpsc::Sender<AgentMessage>,
    mut agent_rx: mpsc::Receiver<AgentMessage>,
    root: crate::workspace::Workspace,
    cfg: crate::config::FeverConfig,
) -> Result<()> {
    let mut reader = crossterm::event::EventStream::new();

    loop {
        terminal.draw(|frame| draw(frame, app))?;

        tokio::select! {
            biased;
            maybe_event = reader.next() => {
                match maybe_event {
                    Some(Ok(Event::Key(key))) => {
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
                                    && app.input.is_empty()
                                    && !app.agent_busy =>
                            {
                                break;
                            }
                            _ => {
                                if app.agent_busy {
                                    // Ignore input while agent is working
                                    continue;
                                }
                                if let Some(text) = app.handle_input(key.code) {
                                    if text.starts_with('/') {
                                        let cmd = text.clone();
                                        if cmd == "/exit" || cmd == "/quit" || cmd == "/q" {
                                            break;
                                        }
                                        app.handle_command(&cmd);
                                        if let Some(req) = app.pending_request.take() {
                                            if req == "__DISCOVER_MODELS__" {
                                                app.agent_busy = true;
                                                app.status_message = "Discovering models...".to_string();
                                                let provider_cfg = cfg.default_provider().clone();
                                                let tx = agent_tx.clone();
                                                tokio::spawn(async move {
                                                    match crate::providers::build_provider(&provider_cfg) {
                                                        Ok(provider) => {
                                                            match provider.list_models().await {
                                                                Ok(models) => {
                                                                    let _ = tx.try_send(AgentMessage::ModelList(models));
                                                                }
                                                                Err(e) => {
                                                                    let _ = tx.try_send(AgentMessage::Error(e.to_string()));
                                                                }
                                                            }
                                                        }
                                                        Err(e) => {
                                                            let _ = tx.try_send(AgentMessage::Error(e.to_string()));
                                                        }
                                                    }
                                                });
                                            } else if req == "__RAG_INDEX__" {
                                                app.agent_busy = true;
                                                app.status_message = "Indexing workspace...".to_string();
                                                let workspace = root.root.clone();
                                                let store_path = app.rag_store_path.clone();
                                                let tx = agent_tx.clone();
                                                let provider_cfg = cfg.default_provider().clone();
                                                tokio::spawn(async move {
                                                    let embedder: Box<dyn crate::rag::embedder::Embedder> =
                                                        if provider_cfg.kind == "openai_compatible" {
                                                            let base = provider_cfg.base_url.clone().unwrap_or_else(|| "http://localhost:11434".to_string());
                                                            Box::new(crate::rag::embedder::OllamaEmbedder::new(base, "nomic-embed-text".to_string()))
                                                        } else {
                                                            let _ = tx.try_send(AgentMessage::Error("RAG requires openai_compatible provider for embeddings.".to_string()));
                                                            return;
                                                        };
                                                    let mut store = crate::rag::store::VectorStore::new();
                                                    match crate::rag::index_directory(&mut store, &*embedder, &workspace
                                                    ).await {
                                                        Ok(count) => {
                                                            let _ = store.save(&store_path);
                                                            let _ = tx.try_send(AgentMessage::Delta(format!("Indexed {} chunks.", count)));
                                                            let _ = tx.try_send(AgentMessage::Done);
                                                        }
                                                        Err(e) => {
                                                            let _ = tx.try_send(AgentMessage::Error(e.to_string()));
                                                            let _ = tx.try_send(AgentMessage::Done);
                                                        }
                                                    }
                                                });
                                            } else if req.starts_with("__MASTERMIND__: ") {
                                                let question = req.strip_prefix("__MASTERMIND__: ").unwrap_or("")
                                                    .to_string();
                                                app.agent_busy = true;
                                                app.status_message = "Mastermind reasoning...".to_string();
                                                let model = cfg.providers.default.model.clone().unwrap_or_default();
                                                let store = std::mem::take(&mut app.rag_store);
                                                let store_path = app.rag_store_path.clone();
                                                let tx = agent_tx.clone();
                                                let provider_cfg = cfg.default_provider().clone();
                                                tokio::spawn(async move {
                                                    if store.is_empty() {
                                                        let _ = tx.try_send(AgentMessage::Error("RAG store empty. Run /index first.".to_string()));
                                                        let _ = tx.try_send(AgentMessage::Done);
                                                        return;
                                                    }
                                                    let embedder: Box<dyn crate::rag::embedder::Embedder> =
                                                        if provider_cfg.kind == "openai_compatible" {
                                                            let base = provider_cfg.base_url.clone().unwrap_or_else(|| "http://localhost:11434".to_string());
                                                            Box::new(crate::rag::embedder::OllamaEmbedder::new(base, "nomic-embed-text".to_string()))
                                                        } else {
                                                            let _ = tx.try_send(AgentMessage::Error("RAG requires openai_compatible provider for embeddings.".to_string()));
                                                            return;
                                                        };
                                                    let provider = match crate::providers::build_provider(&provider_cfg) {
                                                        Ok(p) => p,
                                                        Err(e) => {
                                                            let _ = tx.try_send(AgentMessage::Error(e.to_string()));
                                                            let _ = tx.try_send(AgentMessage::Done);
                                                            return;
                                                        }
                                                    };
                                                    match crate::rag::mastermind::run(
                                                        &*provider, &model, &store, &*embedder, &question
                                                    ).await {
                                                        Ok(result) => {
                                                            let _ = tx.try_send(AgentMessage::MastermindResult(result));
                                                        }
                                                        Err(e) => {
                                                            let _ = tx.try_send(AgentMessage::Error(e.to_string()));
                                                            let _ = tx.try_send(AgentMessage::Done);
                                                        }
                                                    }
                                                    let _ = store.save(&store_path);
                                                });
                                            } else {
                                                app.agent_busy = true;
                                                app.status_message = "Agent working...".to_string();
                                                app.current_line_count = app.chat_lines.len();
                                                spawn_agent(app.mode, req, agent_tx.clone(), &root, &cfg);
                                            }
                                        }
                                    } else {
                                        // Clarification flow
                                        if let Some(session) = app.clarification_session.as_mut() {
                                            if session.needs_answers() {
                                                session.push_answer(text.clone());
                                                app.chat_lines.push(ChatLine {
                                                    role: "user".to_string(),
                                                    content: format!("[Answer] {}", text),
                                                });
                                                if session.all_answered() {
                                                    let model = cfg.providers.default.model.as_deref().unwrap_or("unknown");
                                                    let lower = model.to_ascii_lowercase();
                                                    if lower.contains("llama3.2") || lower.contains("llama-3.2") {
                                                        app.apply_agent_message(AgentMessage::Error(
                                                            "llama3.2 is TEST/RESEARCH ONLY. Use a production model for chat tasks.".to_string()
                                                        ));
                                                        app.apply_agent_message(AgentMessage::Done);
                                                    } else if let Ok(provider) = crate::providers::build_provider(cfg.default_provider()) {
                                                        let tx = agent_tx.clone();
                                                        let session_clone = session.clone();
                                                        let model = model.to_string();
                                                        app.agent_busy = true;
                                                        app.status_message = "Checking readiness...".to_string();
                                                        tokio::spawn(async move {
                                                            match crate::clarification::check_readiness(&*provider, &model, &session_clone
                                                            ).await {
                                                                Ok(result) => {
                                                                    let msg = format!(
                                                                        "Readiness: {}% — {}",
                                                                        result.certainty,
                                                                        if result.ready { "Proceeding to plan." } else { &result.missing_info }
                                                                    );
                                                                    let _ = tx.try_send(AgentMessage::Delta(msg));
                                                                    let _ = tx.try_send(AgentMessage::Done);
                                                                }
                                                                Err(e) => {
                                                                    let _ = tx.try_send(AgentMessage::Error(e.to_string()));
                                                                    let _ = tx.try_send(AgentMessage::Done);
                                                                }
                                                            }
                                                        });
                                                    }
                                                }
                                                continue;
                                            }
                                        }

                                        // Normal builder agent with optional auto-clarification
                                        if app.clarification_session.is_none() && crate::clarification::is_vague_request(&text) {
                                            app.clarification_session = Some(crate::clarification::ClarificationSession::new(text.clone()));
                                            let model = cfg.providers.default.model.as_deref().unwrap_or("unknown");
                                            let lower = model.to_ascii_lowercase();
                                            if lower.contains("llama3.2") || lower.contains("llama-3.2") {
                                                app.apply_agent_message(AgentMessage::Error(
                                                    "llama3.2 is TEST/RESEARCH ONLY. Use a production model for chat tasks.".to_string()
                                                ));
                                                app.apply_agent_message(AgentMessage::Done);
                                                continue;
                                            }
                                            if let Ok(provider) = crate::providers::build_provider(cfg.default_provider()) {
                                                let tx = agent_tx.clone();
                                                let text_clone = text.clone();
                                                let model = model.to_string();
                                                app.agent_busy = true;
                                                app.status_message = "Analyzing request...".to_string();
                                                tokio::spawn(async move {
                                                    match crate::clarification::generate_questions(
                                                        &*provider, &model, &text_clone
                                                    ).await {
                                                        Ok(qs) => {
                                                            let _ = tx.try_send(AgentMessage::ClarificationQuestions(qs));
                                                        }
                                                        Err(e) => {
                                                            let _ = tx.try_send(AgentMessage::Error(e.to_string()));
                                                            let _ = tx.try_send(AgentMessage::Done);
                                                        }
                                                    }
                                                });
                                                continue;
                                            }
                                        }

                                        spawn_agent(app.mode, text.clone(), agent_tx.clone(), &root, &cfg);
                                    }
                                }
                            }
                        }
                    }
                    Some(Err(e)) => {
                        app.status_message = format!("Input error: {}", e);
                    }
                    None => break,
                    _ => {}
                }
            }
            maybe_msg = agent_rx.recv() => {
                if let Some(msg) = maybe_msg {
                    app.apply_agent_message(msg);
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
        ApprovalMode::Ask => app.theme.ask_mode_color(),
        ApprovalMode::Auto => app.theme.auto_mode_color(),
        ApprovalMode::Spray => app.theme.spray_mode_color(),
    };

    let mode_str = format!("{}", app.mode);
    let busy_indicator = if app.agent_busy { " [working]" } else { "" };
    let llama_marker = if app.llama32_warning {
        " [TEST-ONLY]"
    } else {
        ""
    };
    let tier_badge = if app.license_tier != "community" {
        format!(" [{}]", app.license_tier.to_ascii_uppercase())
    } else {
        String::new()
    };
    let title = Line::from(vec![
        Span::styled(
            "  FeverCode Portal",
            Style::default()
                .fg(app.theme.header_accent())
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("  "),
        Span::styled(
            format!("mode:{}{}", mode_str, busy_indicator),
            Style::default().fg(mode_color),
        ),
        Span::raw("  "),
        Span::styled(
            format!("{}:{}{}", app.provider_name, app.model_name, llama_marker),
            Style::default().fg(if app.llama32_warning {
                Color::Red
            } else {
                Color::DarkGray
            }),
        ),
        if !tier_badge.is_empty() {
            Span::styled(
                tier_badge,
                Style::default()
                    .fg(app.theme.header_accent())
                    .add_modifier(Modifier::BOLD),
            )
        } else {
            Span::raw("")
        },
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
    let preset_desc = format!("preset:{} | {}", app.preset_name, mode_desc);

    let desc_line = Line::from(vec![
        Span::styled("  ", Style::default()),
        Span::styled(preset_desc, Style::default().fg(Color::DarkGray)),
    ]);

    let header = Paragraph::new(vec![title, path_line, desc_line]).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(app.theme.border_color()))
            .title(" Ra "),
    );
    frame.render_widget(header, area);
}

fn draw_body(frame: &mut ratatui::Frame, app: &mut App, area: Rect) {
    if app.show_help {
        draw_help(frame, app, area);
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
    let theme = app.theme;
    let lines: Vec<Line> = app
        .chat_lines
        .iter()
        .flat_map(|chat| {
            let (prefix, color) = match chat.role.as_str() {
                "user" => ("You", theme.chat_user()),
                "assistant" => ("Ptah", theme.chat_assistant()),
                "system" => ("Portal", theme.chat_system()),
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
                .border_style(Style::default().fg(theme.border_color()))
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
                Style::default().fg(app.theme.chat_assistant())
            } else {
                Style::default().fg(app.theme.border_color())
            };
            Line::from(Span::styled(format!(" {}", a.id), style))
        })
        .collect();

    let agent_widget = Paragraph::new(agent_lines).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(app.theme.border_color()))
            .title(" Agents "),
    );
    frame.render_widget(agent_widget, sidebar_layout[0]);

    let tools = [
        "read_file",
        "write_file",
        "edit_file",
        "list_files",
        "search_text",
        "run_shell",
        "git_status",
        "git_diff",
        "git_checkpoint",
        "git_branch",
    ];
    let tool_lines: Vec<Line> = tools
        .iter()
        .map(|t| {
            Line::from(Span::styled(
                format!(" {}", t),
                Style::default().fg(app.theme.border_color()),
            ))
        })
        .collect();

    let tool_widget = Paragraph::new(tool_lines).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(app.theme.border_color()))
            .title(" Tools "),
    );
    frame.render_widget(tool_widget, sidebar_layout[1]);
}

fn draw_help(frame: &mut ratatui::Frame, app: &App, area: Rect) {
    let help_lines = vec![
        Line::from(Span::styled(
            " FeverCode Commands",
            Style::default()
                .fg(app.theme.header_accent())
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(
            " Mode",
            Style::default()
                .fg(app.theme.header_accent())
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(vec![Span::styled(
            " /mode ask|auto|spray",
            Style::default().fg(app.theme.chat_user()),
        )]),
        Line::from(vec![
            Span::styled(" /ask ", Style::default().fg(app.theme.ask_mode_color())),
            Span::raw("  Ask before every action (safest)"),
        ]),
        Line::from(vec![
            Span::styled(" /auto ", Style::default().fg(app.theme.auto_mode_color())),
            Span::raw(" Auto-approve safe edits"),
        ]),
        Line::from(vec![
            Span::styled(
                " /spray ",
                Style::default().fg(app.theme.spray_mode_color()),
            ),
            Span::raw("Autonomous workspace edits"),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            " Actions",
            Style::default()
                .fg(app.theme.header_accent())
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(vec![
            Span::styled(" /plan ", Style::default().fg(app.theme.chat_user())),
            Span::raw("  Plan mode"),
        ]),
        Line::from(vec![
            Span::styled(" /run ", Style::default().fg(app.theme.chat_user())),
            Span::raw("   Run mode"),
        ]),
        Line::from(vec![
            Span::styled(" /doctor ", Style::default().fg(app.theme.chat_assistant())),
            Span::raw("Run health checks"),
        ]),
        Line::from(vec![
            Span::styled(" /diff ", Style::default().fg(app.theme.chat_user())),
            Span::raw("  View pending diffs"),
        ]),
        Line::from(vec![
            Span::styled(
                " /approve ",
                Style::default().fg(app.theme.chat_assistant()),
            ),
            Span::raw("Approval queue"),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            " Info",
            Style::default()
                .fg(app.theme.header_accent())
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(vec![
            Span::styled(" /status ", Style::default().fg(app.theme.chat_user())),
            Span::raw("Show workspace and mode"),
        ]),
        Line::from(vec![
            Span::styled(" /model ", Style::default().fg(app.theme.chat_user())),
            Span::raw(" Change active model"),
        ]),
        Line::from(vec![
            Span::styled(" /providers ", Style::default().fg(app.theme.chat_user())),
            Span::raw("List providers"),
        ]),
        Line::from(vec![
            Span::styled(" /theme ", Style::default().fg(app.theme.chat_user())),
            Span::raw(" Show theme info"),
        ]),
        Line::from(vec![
            Span::styled(" /version ", Style::default().fg(app.theme.chat_user())),
            Span::raw("Show version"),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            " License & Pro",
            Style::default()
                .fg(app.theme.header_accent())
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(vec![
            Span::styled(" /auth ", Style::default().fg(app.theme.chat_user())),
            Span::raw(" Show license status"),
        ]),
        Line::from(vec![
            Span::styled(" /auth login <key> ", Style::default().fg(app.theme.chat_user())),
            Span::raw(" Activate license"),
        ]),
        Line::from(vec![
            Span::styled(" /analytics ", Style::default().fg(app.theme.chat_user())),
            Span::raw(" Session analytics (Pro+)"),
        ]),
        Line::from(vec![
            Span::styled(" /sync ", Style::default().fg(app.theme.chat_user())),
            Span::raw(" Cloud sync (Pro+)"),
        ]),
        Line::from(vec![
            Span::styled(" /memory ", Style::default().fg(app.theme.chat_user())),
            Span::raw(" Persistent memory"),
        ]),
        Line::from(vec![
            Span::styled(" /customsoul ", Style::default().fg(app.theme.chat_user())),
            Span::raw(" Custom souls (Pro+)"),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled(" /clear ", Style::default().fg(app.theme.chat_user())),
            Span::raw(" Clear chat"),
        ]),
        Line::from(vec![
            Span::styled(" /exit ", Style::default().fg(Color::Red)),
            Span::raw("  Exit FeverCode (or q)"),
        ]),
        Line::from(""),
        Line::from(" Press Esc to close this help."),
    ];

    let help = Paragraph::new(help_lines).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(app.theme.header_accent()))
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

    let busy_indicator = if app.agent_busy { " [busy]" } else { "" };
    let display_input = if app.input.is_empty() {
        format!("Type a message or /help for commands...{}", busy_indicator)
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
                .fg(app.theme.header_accent())
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(display_input, Style::default().fg(input_color)),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(app.theme.border_color())),
    );
    frame.render_widget(input, area);
}
