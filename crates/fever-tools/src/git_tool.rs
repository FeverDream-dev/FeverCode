use async_trait::async_trait;
use fever_core::{Error, ExecutionContext, Result, Tool, ToolSchema};
use serde_json::Value;

pub struct GitTool {
    repo_path: std::path::PathBuf,
}

impl GitTool {
    pub fn new() -> Self {
        Self {
            repo_path: std::env::current_dir().unwrap_or_default(),
        }
    }

    pub fn with_repo_path(mut self, path: std::path::PathBuf) -> Self {
        self.repo_path = path;
        self
    }
}

#[async_trait]
impl Tool for GitTool {
    fn name(&self) -> &str {
        "git"
    }

    fn description(&self) -> &str {
        "Interact with Git repositories"
    }

    async fn execute(&self, args: Value, _context: &ExecutionContext) -> Result<Value> {
        let action = args.get("action").and_then(|v| v.as_str()).unwrap_or("");

        let result = match action {
            "status" => self.status(args).await?,
            "log" => self.log(args).await?,
            "diff" => self.diff(args).await?,
            "commit" => self.commit(args).await?,
            "branch" => self.branch(args).await?,
            _ => return Err(Error::InvalidRequest("Unknown git action".to_string())),
        };

        Ok(result)
    }

    fn schema(&self) -> ToolSchema {
        ToolSchema {
            name: self.name().to_string(),
            description: self.description().to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "action": {
                        "type": "string",
                        "enum": ["status", "log", "diff", "commit", "branch"],
                        "description": "Git action to perform"
                    },
                    "message": {
                        "type": "string",
                        "description": "Commit message (for commit action)"
                    },
                    "ref": {
                        "type": "string",
                        "description": "Git reference (for log, diff, branch)"
                    }
                }
            }),
        }
    }
}

impl GitTool {
    async fn status(&self, _args: Value) -> Result<Value> {
        let output = tokio::process::Command::new("git")
            .args(["status", "--porcelain"])
            .current_dir(&self.repo_path)
            .output()
            .await
            .map_err(Error::Io)?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let lines: Vec<&str> = stdout.lines().collect();

        Ok(serde_json::json!({
            "files_changed": lines.len(),
            "changes": lines,
            "in_repo": output.status.success()
        }))
    }

    async fn log(&self, args: Value) -> Result<Value> {
        let git_ref = args.get("ref").and_then(|v| v.as_str()).unwrap_or("HEAD");

        let output = tokio::process::Command::new("git")
            .args(["log", "--oneline", "-20", git_ref])
            .current_dir(&self.repo_path)
            .output()
            .await
            .map_err(Error::Io)?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let commits: Vec<&str> = stdout.lines().collect();

        Ok(serde_json::json!({
            "commits": commits,
            "ref": git_ref
        }))
    }

    async fn diff(&self, args: Value) -> Result<Value> {
        let git_ref = args.get("ref").and_then(|v| v.as_str());

        let mut cmd = tokio::process::Command::new("git");
        cmd.arg("diff");

        if let Some(r) = git_ref {
            cmd.arg(r);
        }

        cmd.current_dir(&self.repo_path);

        let output = cmd.output().await.map_err(Error::Io)?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();

        Ok(serde_json::json!({
            "diff": stdout,
            "ref": git_ref
        }))
    }

    async fn commit(&self, args: Value) -> Result<Value> {
        let message = args
            .get("message")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::InvalidRequest("commit message required".to_string()))?;

        let output = tokio::process::Command::new("git")
            .args(["commit", "-m", message])
            .current_dir(&self.repo_path)
            .output()
            .await
            .map_err(Error::Io)?;

        Ok(serde_json::json!({
            "success": output.status.success(),
            "message": message
        }))
    }

    async fn branch(&self, _args: Value) -> Result<Value> {
        let output = tokio::process::Command::new("git")
            .args(["branch", "-a"])
            .current_dir(&self.repo_path)
            .output()
            .await
            .map_err(Error::Io)?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let branches: Vec<&str> = stdout.lines().collect();

        Ok(serde_json::json!({
            "branches": branches
        }))
    }
}

impl Default for GitTool {
    fn default() -> Self {
        Self::new()
    }
}
