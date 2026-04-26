use anyhow::Result;
use serde_json::Value;
use std::path::PathBuf;

use super::ToolResult;

pub struct RunShellTool {
    workspace_root: PathBuf,
}

impl RunShellTool {
    pub fn new(workspace_root: PathBuf) -> Self {
        Self { workspace_root }
    }
}

impl super::Tool for RunShellTool {
    fn name(&self) -> &str {
        "run_shell"
    }

    fn execute(&self, args: Value) -> Result<ToolResult> {
        let cmd = args["command"].as_str().unwrap_or("");
        if cmd.is_empty() {
            return Ok(ToolResult::err("command is required"));
        }

        let _timeout_secs = args["timeout"].as_u64().unwrap_or(30);

        let output = std::process::Command::new("sh")
            .arg("-c")
            .arg(cmd)
            .current_dir(&self.workspace_root)
            .output();

        match output {
            Ok(out) => {
                let stdout = String::from_utf8_lossy(&out.stdout);
                let stderr = String::from_utf8_lossy(&out.stderr);
                let success = out.status.success();
                let mut result = String::new();
                if !stdout.is_empty() {
                    result.push_str(&stdout);
                }
                if !stderr.is_empty() {
                    if !result.is_empty() {
                        result.push('\n');
                    }
                    result.push_str("[stderr] ");
                    result.push_str(&stderr);
                }
                Ok(ToolResult {
                    output: result,
                    success,
                })
            }
            Err(e) => Ok(ToolResult::err(format!("command failed: {}", e))),
        }
    }
}
