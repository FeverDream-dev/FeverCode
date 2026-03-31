use async_trait::async_trait;
use fever_core::{Error, ExecutionContext, Result, Tool, ToolSchema};
use serde_json::Value;

pub struct ShellTool {
    working_dir: std::path::PathBuf,
}

impl ShellTool {
    pub fn new() -> Self {
        Self {
            working_dir: std::env::current_dir().unwrap_or_default(),
        }
    }

    pub fn with_working_dir(mut self, dir: std::path::PathBuf) -> Self {
        self.working_dir = dir;
        self
    }
}

#[async_trait]
impl Tool for ShellTool {
    fn name(&self) -> &str {
        "shell"
    }

    fn description(&self) -> &str {
        "Execute shell commands"
    }

    async fn execute(&self, args: Value, _context: &ExecutionContext) -> Result<Value> {
        let command = args
            .get("command")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::InvalidRequest("command required".to_string()))?;

        let output = tokio::process::Command::new("bash")
            .args(["-c", command])
            .current_dir(&self.working_dir)
            .output()
            .await
            .map_err(|e| Error::Io(e))?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        Ok(serde_json::json!({
            "stdout": stdout,
            "stderr": stderr,
            "exit_code": output.status.code(),
            "success": output.status.success()
        }))
    }

    fn schema(&self) -> ToolSchema {
        ToolSchema {
            name: self.name().to_string(),
            description: self.description().to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "command": {
                        "type": "string",
                        "description": "Shell command to execute"
                    }
                },
                "required": ["command"]
            }),
        }
    }
}

impl Default for ShellTool {
    fn default() -> Self {
        Self::new()
    }
}
