/// Represents the current lifecycle state of the Telegram integration agent.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AgentState {
    Idle,
    Running,
    Paused,
    Stopped,
}
