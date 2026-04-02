use crate::event::Message;
use tokio::sync::mpsc;

/// Trait for sending user input to an agent and receiving streamed output.
///
/// Implementations run the agent loop asynchronously and send `Message` variants
/// back through the provided channel so the TUI Elm loop can process them.
///
/// The implementation must:
/// - Stream text chunks as `Message::StreamChunk`
/// - Report tool calls as `Message::ToolCallStarted` / `Message::ToolCallCompleted`
/// - Signal completion with `Message::StreamEnd`
/// - On error, include error info in the last streamed chunk before `StreamEnd`
pub trait AgentHandle: Send + Sync {
    fn submit(&self, content: String, tx: mpsc::Sender<Message>);
}
