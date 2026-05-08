use anyhow::Result;
use serde_json::Value;
use std::path::PathBuf;

use super::ToolResult;

pub struct DockerTool { workspace_root: PathBuf }
impl DockerTool { pub fn new(r: PathBuf) -> Self { Self { workspace_root: r } } }
impl super::Tool for DockerTool {
    fn name(&self) -> &str { "docker" }
    fn execute(&self, args: Value) -> Result<ToolResult> {
        let action = args["action"].as_str().unwrap_or("ps");
        let cmd = match action {
            "build" => {
                let tag = args["tag"].as_str().unwrap_or("app");
                format!("docker build -t {} . 2>&1", tag)
            }
            "run" => {
                let image = args["image"].as_str().unwrap_or("app");
                let port = args["port"].as_str().unwrap_or("");
                if port.is_empty() { format!("docker run --rm {} 2>&1", image) }
                else { format!("docker run --rm -p {} {} 2>&1", port, image) }
            }
            "ps" => "docker ps -a 2>&1".to_string(),
            "images" => "docker images 2>&1".to_string(),
            "logs" => {
                let container = args["container"].as_str().unwrap_or("");
                if container.is_empty() { return Ok(ToolResult::err("container name/id required")); }
                format!("docker logs --tail 50 {} 2>&1", container)
            }
            "compose_up" => "docker compose up -d 2>&1".to_string(),
            "compose_down" => "docker compose down 2>&1".to_string(),
            "compose_logs" => "docker compose logs --tail 50 2>&1".to_string(),
            _ => return Ok(ToolResult::err("actions: build, run, ps, images, logs, compose_up, compose_down, compose_logs")),
        };
        let out = std::process::Command::new("sh").arg("-c").arg(&cmd).current_dir(&self.workspace_root).output();
        match out {
            Ok(o) => {
                let output = String::from_utf8_lossy(&o.stdout);
                let truncated = if output.len() > 5000 { format!("{}...(truncated)", &output[..5000]) } else { output.to_string() };
                Ok(ToolResult { output: truncated, success: o.status.success() })
            }
            Err(e) => Ok(ToolResult::err(format!("docker failed: {}. Is Docker installed?", e))),
        }
    }
}

pub struct WebFetchTool;
impl WebFetchTool { pub fn new() -> Self { Self } }
impl super::Tool for WebFetchTool {
    fn name(&self) -> &str { "web_fetch" }
    fn execute(&self, args: Value) -> Result<ToolResult> {
        let url = args["url"].as_str().unwrap_or("");
        if url.is_empty() { return Ok(ToolResult::err("url required")); }
        let cmd = format!("curl -sL --max-time 10 {} 2>&1", url);
        let out = std::process::Command::new("sh").arg("-c").arg(&cmd).output();
        match out {
            Ok(o) => {
                let body = String::from_utf8_lossy(&o.stdout).to_string();
                let truncated = if body.len() > 5000 { format!("{}...(truncated at 5KB)", &body[..5000]) } else { body };
                Ok(ToolResult::ok(truncated))
            }
            Err(e) => Ok(ToolResult::err(format!("fetch failed: {}", e))),
        }
    }
}

pub struct PackageJsonTool { workspace_root: PathBuf }
impl PackageJsonTool { pub fn new(r: PathBuf) -> Self { Self { workspace_root: r } } }
impl super::Tool for PackageJsonTool {
    fn name(&self) -> &str { "manage_deps" }
    fn execute(&self, args: Value) -> Result<ToolResult> {
        let action = args["action"].as_str().unwrap_or("list");
        match action {
            "list" => {
                let cmd = if self.workspace_root.join("Cargo.toml").exists() { "cargo tree --depth 1 2>&1 | head -50" }
                else if self.workspace_root.join("package.json").exists() { "npm list --depth=0 2>&1" }
                else if self.workspace_root.join("go.mod").exists() { "go list -m all 2>&1 | head -50" }
                else { "echo 'no package manager detected'" };
                let out = std::process::Command::new("sh").arg("-c").arg(cmd).current_dir(&self.workspace_root).output();
                match out { Ok(o) => Ok(ToolResult::ok(String::from_utf8_lossy(&o.stdout).to_string())), Err(e) => Ok(ToolResult::err(format!("failed: {}", e))) }
            }
            "outdated" => {
                let cmd = if self.workspace_root.join("Cargo.toml").exists() { "cargo outdated 2>&1 || echo 'cargo-outdated not installed'" }
                else if self.workspace_root.join("package.json").exists() { "npm outdated 2>&1 || echo 'all up to date'" }
                else { "echo 'no package manager'" };
                let out = std::process::Command::new("sh").arg("-c").arg(cmd).current_dir(&self.workspace_root).output();
                match out { Ok(o) => Ok(ToolResult::ok(String::from_utf8_lossy(&o.stdout).to_string())), Err(e) => Ok(ToolResult::err(format!("failed: {}", e))) }
            }
            "update" => {
                let cmd = if self.workspace_root.join("Cargo.toml").exists() { "cargo update 2>&1" }
                else if self.workspace_root.join("package.json").exists() { "npm update 2>&1" }
                else { "echo 'no package manager'" };
                let out = std::process::Command::new("sh").arg("-c").arg(cmd).current_dir(&self.workspace_root).output();
                match out { Ok(o) => Ok(ToolResult::ok(String::from_utf8_lossy(&o.stdout).to_string())), Err(e) => Ok(ToolResult::err(format!("failed: {}", e))) }
            }
            _ => Ok(ToolResult::err("actions: list, outdated, update")),
        }
    }
}

pub struct CiStatusTool { workspace_root: PathBuf }
impl CiStatusTool { pub fn new(r: PathBuf) -> Self { Self { workspace_root: r } } }
impl super::Tool for CiStatusTool {
    fn name(&self) -> &str { "ci_status" }
    fn execute(&self, _args: Value) -> Result<ToolResult> {
        let out = std::process::Command::new("sh").arg("-c").arg("gh run list --limit 5 2>&1")
            .current_dir(&self.workspace_root).output();
        match out {
            Ok(o) => Ok(ToolResult::ok(String::from_utf8_lossy(&o.stdout).to_string())),
            Err(e) => Ok(ToolResult::err(format!("gh CLI failed: {}", e))),
        }
    }
}

pub struct SnippetExecTool { workspace_root: PathBuf }
impl SnippetExecTool { pub fn new(r: PathBuf) -> Self { Self { workspace_root: r } } }
impl super::Tool for SnippetExecTool {
    fn name(&self) -> &str { "run_snippet" }
    fn execute(&self, args: Value) -> Result<ToolResult> {
        let lang = args["language"].as_str().unwrap_or("bash");
        let code = args["code"].as_str().unwrap_or("");
        if code.is_empty() { return Ok(ToolResult::err("code required")); }
        let tmp = std::env::temp_dir().join(format!("fever_snippet.{}", match lang { "python" | "py" => "py", "rust" => "rs", "js" | "javascript" => "js", "go" => "go", _ => "sh" }));
        std::fs::write(&tmp, code)?;
        let cmd = match lang {
            "python" | "py" => format!("python {} 2>&1", tmp.display()),
            "rust" => format!("rustc {} -o /tmp/fever_snippet_out && /tmp/fever_snippet_out 2>&1", tmp.display()),
            "js" | "javascript" | "node" => format!("node {} 2>&1", tmp.display()),
            "go" => format!("go run {} 2>&1", tmp.display()),
            _ => format!("sh {} 2>&1", tmp.display()),
        };
        let out = std::process::Command::new("sh").arg("-c").arg(&cmd).output();
        let _ = std::fs::remove_file(&tmp);
        match out {
            Ok(o) => Ok(ToolResult { output: String::from_utf8_lossy(&o.stdout).to_string(), success: o.status.success() }),
            Err(e) => Ok(ToolResult::err(format!("execution failed: {}", e))),
        }
    }
}

pub struct MarkdownRendererTool { workspace_root: PathBuf }
impl MarkdownRendererTool { pub fn new(r: PathBuf) -> Self { Self { workspace_root: r } } }
impl super::Tool for MarkdownRendererTool {
    fn name(&self) -> &str { "render_markdown" }
    fn execute(&self, args: Value) -> Result<ToolResult> {
        let path = args["path"].as_str().unwrap_or("");
        if path.is_empty() { return Ok(ToolResult::err("path required")); }
        let full = self.workspace_root.join(path);
        if !full.exists() { return Ok(ToolResult::err(format!("not found: {}", path))); }
        let content = std::fs::read_to_string(&full)?;
        let mut rendered = String::new();
        for line in content.lines() {
            if let Some(s) = line.strip_prefix("# ") { rendered.push_str(&format!("═══ {} ═══\n", s)); }
            else if let Some(s) = line.strip_prefix("## ") { rendered.push_str(&format!("─── {} ───\n", s)); }
            else if let Some(s) = line.strip_prefix("### ") { rendered.push_str(&format!("▸ {}\n", s)); }
            else if let Some(s) = line.strip_prefix("- ").or_else(|| line.strip_prefix("* ")) { rendered.push_str(&format!("  • {}\n", s)); }
            else if line.starts_with("```") { rendered.push_str("┌───\n"); }
            else if line.starts_with("---") { rendered.push_str("─────────────────────\n"); }
            else if let Some(s) = line.strip_prefix("> ") { rendered.push_str(&format!("  ┃ {}\n", s)); }
            else { rendered.push_str(&format!("{}\n", line)); }
        }
        Ok(ToolResult::ok(rendered))
    }
}
