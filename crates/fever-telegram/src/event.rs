/// Telegram events emitted by FeverCode to describe agent activity.
#[derive(Debug, Clone, PartialEq)]
pub enum TelegramEvent {
    AgentStarted {
        task: String,
    },
    Thinking {
        summary: String,
    },
    FileRead {
        path: String,
    },
    FileModified {
        path: String,
        diff_summary: String,
    },
    CommandRun {
        command: String,
        output_preview: String,
    },
    ErrorFound {
        message: String,
    },
    ErrorResolved {
        description: String,
    },
    TaskComplete {
        summary: String,
        files_changed: Vec<String>,
    },
    AgentIdle,
}

impl TelegramEvent {
    /// Formats the TelegramEvent into a human-readable message suitable for Telegram.
    pub fn format_message(&self) -> String {
        match self {
            TelegramEvent::AgentStarted { task } => format!("🚀 Agent started: {}", task),
            TelegramEvent::Thinking { summary } => format!("💭 Thinking: {}", summary),
            TelegramEvent::FileRead { path } => format!("📄 Read: {}", path),
            TelegramEvent::FileModified { path, diff_summary } => {
                format!("📝 Modified: {} — {}", path, diff_summary)
            }
            TelegramEvent::CommandRun {
                command,
                output_preview,
            } => {
                format!("⚙️ Command: {} — {}", command, output_preview)
            }
            TelegramEvent::ErrorFound { message } => format!("⚠️ Error: {}", message),
            TelegramEvent::ErrorResolved { description } => {
                format!("✅ Resolved: {}", description)
            }
            TelegramEvent::TaskComplete {
                summary,
                files_changed,
            } => {
                format!("🎉 Complete: {} ({} files)", summary, files_changed.len())
            }
            TelegramEvent::AgentIdle => "😴 Idle".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_message_all_variants_nonempty() {
        let variants = vec![
            TelegramEvent::AgentStarted {
                task: "build".to_string(),
            },
            TelegramEvent::Thinking {
                summary: "thinking".to_string(),
            },
            TelegramEvent::FileRead {
                path: "/path/file.txt".to_string(),
            },
            TelegramEvent::FileModified {
                path: "/path/file.txt".to_string(),
                diff_summary: "diff".to_string(),
            },
            TelegramEvent::CommandRun {
                command: "ls".to_string(),
                output_preview: "output".to_string(),
            },
            TelegramEvent::ErrorFound {
                message: "err".to_string(),
            },
            TelegramEvent::ErrorResolved {
                description: "fixed".to_string(),
            },
            TelegramEvent::TaskComplete {
                summary: "done".to_string(),
                files_changed: vec!["a.rs".to_string()],
            },
            TelegramEvent::AgentIdle,
        ];

        for ev in variants {
            let s = ev.format_message();
            assert!(!s.is_empty());
        }
    }
}
