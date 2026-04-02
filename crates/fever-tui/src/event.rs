use crate::slash::SlashCommand;
use crossterm::event::{KeyEvent, MouseEvent};

#[derive(Debug, Clone)]
pub enum Screen {
    Home,
    Chat,
    Settings,
    Onboarding { step: usize },
}

#[derive(Debug, Clone)]
pub enum Message {
    Key(KeyEvent),
    Mouse(MouseEvent),
    Tick,
    StreamChunk { content: String },
    StreamEnd,
    StreamError { message: String },
    ToolCallStarted { tool: String, args: String },
    ToolCallCompleted { tool: String, result: String },
    ToolCallFailed { tool: String, error: String },
    Navigate(Screen),
    InputChanged(String),
    InputSubmitted,
    SlashCommand(SlashCommand),
    Quit,
}

#[derive(Debug, Clone)]
pub enum Command {
    SendMessage { content: String },
    DetectProviders,
    Noop,
}
