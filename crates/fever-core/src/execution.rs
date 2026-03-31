use crate::error::Result;
use crate::tool::ToolRegistry;
use crate::Plan;
use serde_json::Value;
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};

pub struct ExecutionContext {
    pub plan_id: String,
    pub task_id: String,
    pub variables: Arc<RwLock<serde_json::Map<String, Value>>>,
}

impl ExecutionContext {
    pub fn new(plan_id: String, task_id: String) -> Self {
        Self {
            plan_id,
            task_id,
            variables: Arc::new(RwLock::new(serde_json::Map::new())),
        }
    }

    pub async fn set_variable(&self, key: String, value: Value) {
        let mut vars = self.variables.write().await;
        vars.insert(key, value);
    }

    pub async fn get_variable(&self, key: &str) -> Option<Value> {
        let vars = self.variables.read().await;
        vars.get(key).cloned()
    }
}

pub struct ExecutionEngine {
    tools: Arc<ToolRegistry>,
    active_plan: Arc<RwLock<Option<Plan>>>,
}

impl ExecutionEngine {
    pub fn new(tools: ToolRegistry) -> Self {
        Self {
            tools: Arc::new(tools),
            active_plan: Arc::new(RwLock::new(None)),
        }
    }

    pub async fn load_plan(&self, plan: Plan) -> Result<()> {
        let mut active = self.active_plan.write().await;
        *active = Some(plan);
        Ok(())
    }

    pub async fn get_active_plan(&self) -> Option<Plan> {
        self.active_plan.read().await.clone()
    }

    pub fn run(&self) -> mpsc::Receiver<ExecutionEvent> {
        let (tx, rx) = mpsc::channel(100);
        let _tools = Arc::clone(&self.tools);
        let plan = Arc::clone(&self.active_plan);

        tokio::spawn(async move {
            let mut completed = HashSet::new();

            loop {
                let current_plan = { plan.read().await.clone() };

                if let Some(p) = current_plan {
                    let ready_tasks: Vec<_> = p
                        .tasks
                        .iter()
                        .filter(|t| t.can_start(&completed))
                        .map(|t| t.id.clone())
                        .collect();

                    if ready_tasks.is_empty() {
                        break;
                    }

                    for task_id in ready_tasks {
                        let task = p.get_task(&task_id).unwrap();

                        let _ = tx.send(ExecutionEvent::TaskStarted {
                            task_id: task_id.clone(),
                            title: task.title.clone(),
                        });

                        let context = ExecutionContext::new(p.id.clone(), task_id.clone());

                        match self::simulate_task(&task, &context).await {
                            Ok(_) => {
                                completed.insert(task_id.clone());
                                let _ = tx.send(ExecutionEvent::TaskCompleted {
                                    task_id: task_id.clone(),
                                    title: task.title.clone(),
                                });
                            }
                            Err(e) => {
                                let _ = tx.send(ExecutionEvent::TaskFailed {
                                    task_id: task_id.clone(),
                                    title: task.title.clone(),
                                    error: e.to_string(),
                                });
                            }
                        }
                    }
                } else {
                    break;
                }
            }

            let _ = tx.send(ExecutionEvent::PlanCompleted);
        });

        rx
    }
}

async fn simulate_task(_task: &crate::Task, _context: &ExecutionContext) -> Result<()> {
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    Ok(())
}

#[derive(Debug, Clone)]
pub enum ExecutionEvent {
    TaskStarted { task_id: String, title: String },
    TaskCompleted { task_id: String, title: String },
    TaskFailed { task_id: String, title: String, error: String },
    PlanCompleted,
}
