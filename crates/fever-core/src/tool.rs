use crate::error::{Result, ToolCall, ToolResult, ToolResultData};
use crate::execution::ExecutionContext;
use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;
use uuid::Uuid;

#[async_trait]
pub trait Tool: Send + Sync {
    fn name(&self) -> &str;

    fn description(&self) -> &str;

    async fn execute(&self, args: Value, context: &ExecutionContext) -> Result<Value>;

    fn schema(&self) -> ToolSchema;
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ToolSchema {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

pub struct ToolRegistry {
    tools: HashMap<String, Box<dyn Tool>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
        }
    }

    pub fn register(&mut self, tool: Box<dyn Tool>) -> Result<()> {
        let name = tool.name().to_string();
        self.tools.insert(name.clone(), tool);
        Ok(())
    }

    pub fn get(&self, name: &str) -> Option<&dyn Tool> {
        self.tools.get(name).map(|t| t.as_ref())
    }

    pub fn list(&self) -> Vec<String> {
        self.tools.keys().cloned().collect()
    }

    pub async fn execute_call(
        &self,
        call: &ToolCall,
        context: &ExecutionContext,
    ) -> Result<ToolResult> {
        let start = std::time::Instant::now();

        let tool = self.get(&call.name).ok_or_else(|| {
            crate::error::Error::ToolExecution(format!("Tool '{}' not found", call.name))
        })?;

        let result = tool.execute(call.arguments.clone(), context).await;

        let duration = start.elapsed().as_millis() as u64;

        Ok(ToolResult {
            call_id: call.id.clone(),
            result: match result {
                Ok(output) => ToolResultData::Success { output },
                Err(e) => ToolResultData::Error {
                    message: e.to_string(),
                },
            },
            duration_ms: duration,
        })
    }

    pub async fn execute_calls(
        &self,
        calls: &[ToolCall],
        context: &ExecutionContext,
    ) -> Vec<ToolResult> {
        let mut results = Vec::new();

        for call in calls {
            let result = self.execute_call(call, context).await;
            results.push(result.unwrap_or_else(|e| ToolResult {
                call_id: call.id.clone(),
                result: ToolResultData::Error {
                    message: e.to_string(),
                },
                duration_ms: 0,
            }));
        }

        results
    }

    pub fn schemas(&self) -> Vec<ToolSchema> {
        self.tools.values().map(|t| t.schema()).collect()
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

pub fn generate_call_id() -> String {
    format!(
        "call_{}",
        Uuid::new_v4().to_string().split('-').next().unwrap()
    )
}
