use crate::error::{Result, TaskStatus};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub title: String,
    pub description: String,
    pub status: TaskStatus,
    pub dependencies: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub metadata: serde_json::Value,
}

impl Task {
    pub fn new(title: String, description: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            title,
            description,
            status: TaskStatus::Queued,
            dependencies: Vec::new(),
            created_at: Utc::now(),
            started_at: None,
            completed_at: None,
            metadata: serde_json::json!({}),
        }
    }

    pub fn with_dependencies(mut self, deps: Vec<String>) -> Self {
        self.dependencies = deps;
        self
    }

    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = metadata;
        self
    }

    pub fn can_start(&self, completed_tasks: &std::collections::HashSet<String>) -> bool {
        if self.status != TaskStatus::Queued {
            return false;
        }

        self.dependencies
            .iter()
            .all(|dep| completed_tasks.contains(dep))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plan {
    pub id: String,
    pub title: String,
    pub tasks: Vec<Task>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Plan {
    pub fn new(title: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            title,
            tasks: Vec::new(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    pub fn add_task(&mut self, task: Task) {
        self.updated_at = Utc::now();
        self.tasks.push(task);
    }

    pub fn get_task(&self, task_id: &str) -> Option<&Task> {
        self.tasks.iter().find(|t| t.id == task_id)
    }

    pub fn update_task_status(&mut self, task_id: &str, status: TaskStatus) -> Result<()> {
        let task = self
            .tasks
            .iter_mut()
            .find(|t| t.id == task_id)
            .ok_or_else(|| crate::error::Error::TaskExecution(format!("Task {} not found", task_id)))?;

        task.status = status.clone();

        match status {
            TaskStatus::Running => {
                if task.started_at.is_none() {
                    task.started_at = Some(Utc::now());
                }
            }
            TaskStatus::Completed | TaskStatus::Failed => {
                if task.completed_at.is_none() {
                    task.completed_at = Some(Utc::now());
                }
            }
            _ => {}
        }

        self.updated_at = Utc::now();
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Todo {
    pub id: String,
    pub content: String,
    pub status: TaskStatus,
    pub priority: String,
    pub created_at: DateTime<Utc>,
}

impl Todo {
    pub fn new(content: String, priority: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            content,
            status: TaskStatus::Queued,
            priority,
            created_at: Utc::now(),
        }
    }
}
