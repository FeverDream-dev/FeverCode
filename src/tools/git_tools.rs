use anyhow::Result;
use serde_json::Value;
use std::path::PathBuf;

use super::ToolResult;

pub struct GitStatusTool {
    workspace_root: PathBuf,
}

impl GitStatusTool {
    pub fn new(workspace_root: PathBuf) -> Self {
        Self { workspace_root }
    }
}

impl super::Tool for GitStatusTool {
    fn name(&self) -> &str {
        "git_status"
    }

    fn execute(&self, _args: Value) -> Result<ToolResult> {
        let output = std::process::Command::new("git")
            .args(["status", "--porcelain=v1", "--branch"])
            .current_dir(&self.workspace_root)
            .output();

        match output {
            Ok(out) => {
                let stdout = String::from_utf8_lossy(&out.stdout);
                Ok(ToolResult::ok(stdout))
            }
            Err(e) => Ok(ToolResult::err(format!("git status failed: {}", e))),
        }
    }
}

pub struct GitDiffTool {
    workspace_root: PathBuf,
}

impl GitDiffTool {
    pub fn new(workspace_root: PathBuf) -> Self {
        Self { workspace_root }
    }
}

impl super::Tool for GitDiffTool {
    fn name(&self) -> &str {
        "git_diff"
    }

    fn execute(&self, args: Value) -> Result<ToolResult> {
        let staged = args["staged"].as_bool().unwrap_or(false);
        let pathspec = args["path"].as_str().unwrap_or("");

        let mut cmd_args = vec!["diff"];
        if staged {
            cmd_args.push("--staged");
        }
        if !pathspec.is_empty() {
            cmd_args.push("--");
            cmd_args.push(pathspec);
        }

        let output = std::process::Command::new("git")
            .args(&cmd_args)
            .current_dir(&self.workspace_root)
            .output();

        match output {
            Ok(out) => {
                let stdout = String::from_utf8_lossy(&out.stdout);
                Ok(ToolResult::ok(stdout))
            }
            Err(e) => Ok(ToolResult::err(format!("git diff failed: {}", e))),
        }
    }
}

pub struct GitCheckpointTool {
    workspace_root: PathBuf,
}

impl GitCheckpointTool {
    pub fn new(workspace_root: PathBuf) -> Self {
        Self { workspace_root }
    }
}

impl super::Tool for GitCheckpointTool {
    fn name(&self) -> &str {
        "git_checkpoint"
    }

    fn execute(&self, args: Value) -> Result<ToolResult> {
        let message = args["message"]
            .as_str()
            .unwrap_or("fevercode: auto checkpoint");

        let is_git_repo = std::process::Command::new("git")
            .args(["rev-parse", "--is-inside-work-tree"])
            .current_dir(&self.workspace_root)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !is_git_repo {
            return Ok(ToolResult::err("not a git repository"));
        }

        let add_result = std::process::Command::new("git")
            .args(["add", "-A"])
            .current_dir(&self.workspace_root)
            .output();

        match add_result {
            Ok(_) => {}
            Err(e) => return Ok(ToolResult::err(format!("git add failed: {}", e))),
        }

        let commit_result = std::process::Command::new("git")
            .args(["commit", "-m", message, "--allow-empty"])
            .current_dir(&self.workspace_root)
            .output();

        match commit_result {
            Ok(out) => {
                let stdout = String::from_utf8_lossy(&out.stdout);
                if out.status.success() {
                    let hash_output = std::process::Command::new("git")
                        .args(["rev-parse", "--short", "HEAD"])
                        .current_dir(&self.workspace_root)
                        .output();
                    let hash = hash_output
                        .ok()
                        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
                        .unwrap_or_else(|| "unknown".to_string());
                    Ok(ToolResult::ok(format!("checkpoint {} created", hash)))
                } else {
                    Ok(ToolResult::err(format!("commit failed: {}", stdout)))
                }
            }
            Err(e) => Ok(ToolResult::err(format!("git commit failed: {}", e))),
        }
    }
}
