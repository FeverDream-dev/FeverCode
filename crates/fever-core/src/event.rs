use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Event {
    TaskQueued { task_id: String, title: String },
    TaskStarted { task_id: String, title: String },
    TaskProgress { task_id: String, progress: f32 },
    TaskCompleted { task_id: String, title: String },
    TaskFailed { task_id: String, title: String, error: String },
    ToolCalled { tool_name: String, args: serde_json::Value },
    ToolResult { tool_name: String, result: serde_json::Value },
    Message { content: String, role: String },
    PlanCreated { plan_id: String, title: String },
    PlanUpdated { plan_id: String },
    StatusChanged { status: String },
}

pub struct EventBus {
    subscribers: Vec<tokio::sync::mpsc::UnboundedSender<Event>>,
}

impl EventBus {
    pub fn new() -> Self {
        Self {
            subscribers: Vec::new(),
        }
    }

    pub fn subscribe(&mut self) -> tokio::sync::mpsc::UnboundedReceiver<Event> {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        self.subscribers.push(tx);
        rx
    }

    pub async fn publish(&mut self, event: Event) {
        let mut dead_subscribers = Vec::new();

        for (i, subscriber) in self.subscribers.iter().enumerate() {
            if subscriber.send(event.clone()).is_err() {
                dead_subscribers.push(i);
            }
        }

        for i in dead_subscribers.into_iter().rev() {
            self.subscribers.remove(i);
        }
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}
