use anyhow::Result;
use serde_json::Value;
use std::path::PathBuf;

use super::ToolResult;

fn run_git(root: &PathBuf, args: &[&str]) -> std::io::Result<std::process::Output> {
    std::process::Command::new("git").args(args).current_dir(root).output()
}

fn git_output(root: &PathBuf, args: &[&str]) -> Result<ToolResult> {
    match run_git(root, args) {
        Ok(out) if out.status.success() => {
            Ok(ToolResult::ok(String::from_utf8_lossy(&out.stdout).to_string()))
        }
        Ok(out) => Ok(ToolResult::err(String::from_utf8_lossy(&out.stderr).to_string())),
        Err(e) => Ok(ToolResult::err(format!("git failed: {}", e))),
    }
}

pub struct GitLogTool { workspace_root: PathBuf }
impl GitLogTool { pub fn new(r: PathBuf) -> Self { Self { workspace_root: r } } }
impl super::Tool for GitLogTool {
    fn name(&self) -> &str { "git_log" }
    fn execute(&self, args: Value) -> Result<ToolResult> {
        let n = args["count"].as_u64().unwrap_or(20);
        let oneline = args["oneline"].as_bool().unwrap_or(true);
        let pathspec = args["path"].as_str().unwrap_or("");
        let n_arg = format!("-{}", n);
        let mut cmd_args: Vec<&str> = vec!["log", &n_arg];
        if oneline { cmd_args.push("--oneline"); }
        if !pathspec.is_empty() { cmd_args.push("--"); cmd_args.push(pathspec); }
        git_output(&self.workspace_root, &cmd_args.to_vec())
    }
}

pub struct GitBlameTool { workspace_root: PathBuf }
impl GitBlameTool { pub fn new(r: PathBuf) -> Self { Self { workspace_root: r } } }
impl super::Tool for GitBlameTool {
    fn name(&self) -> &str { "git_blame" }
    fn execute(&self, args: Value) -> Result<ToolResult> {
        let path = args["path"].as_str().unwrap_or("");
        if path.is_empty() { return Ok(ToolResult::err("path required")); }
        git_output(&self.workspace_root, &["blame", path])
    }
}

pub struct GitStashTool { workspace_root: PathBuf }
impl GitStashTool { pub fn new(r: PathBuf) -> Self { Self { workspace_root: r } } }
impl super::Tool for GitStashTool {
    fn name(&self) -> &str { "git_stash" }
    fn execute(&self, args: Value) -> Result<ToolResult> {
        let action = args["action"].as_str().unwrap_or("push");
        match action {
            "push" | "save" => {
                let msg = args["message"].as_str().unwrap_or("");
                if msg.is_empty() { git_output(&self.workspace_root, &["stash", "push"]) }
                else { git_output(&self.workspace_root, &["stash", "push", "-m", msg]) }
            }
            "pop" => git_output(&self.workspace_root, &["stash", "pop"]),
            "list" => git_output(&self.workspace_root, &["stash", "list"]),
            "drop" => git_output(&self.workspace_root, &["stash", "drop"]),
            "show" => git_output(&self.workspace_root, &["stash", "show", "-p"]),
            _ => Ok(ToolResult::err("actions: push, pop, list, drop, show")),
        }
    }
}

pub struct GitCherryPickTool { workspace_root: PathBuf }
impl GitCherryPickTool { pub fn new(r: PathBuf) -> Self { Self { workspace_root: r } } }
impl super::Tool for GitCherryPickTool {
    fn name(&self) -> &str { "git_cherry_pick" }
    fn execute(&self, args: Value) -> Result<ToolResult> {
        let commit = args["commit"].as_str().unwrap_or("");
        if commit.is_empty() { return Ok(ToolResult::err("commit hash required")); }
        git_output(&self.workspace_root, &["cherry-pick", commit])
    }
}

pub struct GitMergeTool { workspace_root: PathBuf }
impl GitMergeTool { pub fn new(r: PathBuf) -> Self { Self { workspace_root: r } } }
impl super::Tool for GitMergeTool {
    fn name(&self) -> &str { "git_merge" }
    fn execute(&self, args: Value) -> Result<ToolResult> {
        let branch = args["branch"].as_str().unwrap_or("");
        if branch.is_empty() { return Ok(ToolResult::err("branch required")); }
        let no_ff = args["no_ff"].as_bool().unwrap_or(false);
        if no_ff { git_output(&self.workspace_root, &["merge", "--no-ff", branch]) }
        else { git_output(&self.workspace_root, &["merge", branch]) }
    }
}

pub struct GitRemoteTool { workspace_root: PathBuf }
impl GitRemoteTool { pub fn new(r: PathBuf) -> Self { Self { workspace_root: r } } }
impl super::Tool for GitRemoteTool {
    fn name(&self) -> &str { "git_remote" }
    fn execute(&self, args: Value) -> Result<ToolResult> {
        let action = args["action"].as_str().unwrap_or("list");
        match action {
            "list" | "v" | "verbose" => git_output(&self.workspace_root, &["remote", "-v"]),
            "add" => {
                let name = args["name"].as_str().unwrap_or("origin");
                let url = args["url"].as_str().unwrap_or("");
                if url.is_empty() { return Ok(ToolResult::err("url required")); }
                git_output(&self.workspace_root, &["remote", "add", name, url])
            }
            "remove" | "rm" => {
                let name = args["name"].as_str().unwrap_or("origin");
                git_output(&self.workspace_root, &["remote", "remove", name])
            }
            _ => Ok(ToolResult::err("actions: list, add, remove")),
        }
    }
}

pub struct GitTagTool { workspace_root: PathBuf }
impl GitTagTool { pub fn new(r: PathBuf) -> Self { Self { workspace_root: r } } }
impl super::Tool for GitTagTool {
    fn name(&self) -> &str { "git_tag" }
    fn execute(&self, args: Value) -> Result<ToolResult> {
        let action = args["action"].as_str().unwrap_or("list");
        match action {
            "list" => git_output(&self.workspace_root, &["tag", "-l"]),
            "create" => {
                let name = args["name"].as_str().unwrap_or("");
                if name.is_empty() { return Ok(ToolResult::err("name required")); }
                let msg = args["message"].as_str().unwrap_or("");
                if msg.is_empty() { git_output(&self.workspace_root, &["tag", name]) }
                else { git_output(&self.workspace_root, &["tag", "-a", name, "-m", msg]) }
            }
            "delete" => {
                let name = args["name"].as_str().unwrap_or("");
                if name.is_empty() { return Ok(ToolResult::err("name required")); }
                git_output(&self.workspace_root, &["tag", "-d", name])
            }
            _ => Ok(ToolResult::err("actions: list, create, delete")),
        }
    }
}

pub struct GitRebaseTool { workspace_root: PathBuf }
impl GitRebaseTool { pub fn new(r: PathBuf) -> Self { Self { workspace_root: r } } }
impl super::Tool for GitRebaseTool {
    fn name(&self) -> &str { "git_rebase" }
    fn execute(&self, args: Value) -> Result<ToolResult> {
        let branch = args["onto"].as_str().unwrap_or("main");
        git_output(&self.workspace_root, &["rebase", branch])
    }
}

pub struct GitResetTool { workspace_root: PathBuf }
impl GitResetTool { pub fn new(r: PathBuf) -> Self { Self { workspace_root: r } } }
impl super::Tool for GitResetTool {
    fn name(&self) -> &str { "git_reset" }
    fn execute(&self, args: Value) -> Result<ToolResult> {
        let target = args["ref"].as_str().unwrap_or("HEAD~1");
        let soft = args["soft"].as_bool().unwrap_or(false);
        if soft { git_output(&self.workspace_root, &["reset", "--soft", target]) }
        else { git_output(&self.workspace_root, &["reset", "--mixed", target]) }
    }
}

pub struct GitShowTool { workspace_root: PathBuf }
impl GitShowTool { pub fn new(r: PathBuf) -> Self { Self { workspace_root: r } } }
impl super::Tool for GitShowTool {
    fn name(&self) -> &str { "git_show" }
    fn execute(&self, args: Value) -> Result<ToolResult> {
        let commit = args["commit"].as_str().unwrap_or("HEAD");
        let stat = args["stat"].as_bool().unwrap_or(true);
        if stat { git_output(&self.workspace_root, &["show", "--stat", commit]) }
        else { git_output(&self.workspace_root, &["show", commit]) }
    }
}

pub struct GitAddCommitTool { workspace_root: PathBuf }
impl GitAddCommitTool { pub fn new(r: PathBuf) -> Self { Self { workspace_root: r } } }
impl super::Tool for GitAddCommitTool {
    fn name(&self) -> &str { "git_add_commit" }
    fn execute(&self, args: Value) -> Result<ToolResult> {
        let files = args["files"].as_str().unwrap_or(".");
        let message = args["message"].as_str().unwrap_or("fever: automated commit");
        let add = run_git(&self.workspace_root, &["add", files])?;
        if !add.status.success() { return Ok(ToolResult::err(format!("add failed: {}", String::from_utf8_lossy(&add.stderr)))); }
        let commit = run_git(&self.workspace_root, &["commit", "-m", message])?;
        if commit.status.success() {
            let hash = run_git(&self.workspace_root, &["rev-parse", "--short", "HEAD"])
                .ok().map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string()).unwrap_or_default();
            Ok(ToolResult::ok(format!("committed {} ({})", message, hash)))
        } else {
            Ok(ToolResult::err(format!("commit failed: {}", String::from_utf8_lossy(&commit.stderr))))
        }
    }
}

pub struct GitConflictTool { workspace_root: PathBuf }
impl GitConflictTool { pub fn new(r: PathBuf) -> Self { Self { workspace_root: r } } }
impl super::Tool for GitConflictTool {
    fn name(&self) -> &str { "git_list_conflicts" }
    fn execute(&self, _args: Value) -> Result<ToolResult> {
        let out = run_git(&self.workspace_root, &["diff", "--name-only", "--diff-filter=U"])?;
        let conflicts = String::from_utf8_lossy(&out.stdout).to_string();
        if conflicts.trim().is_empty() { Ok(ToolResult::ok("no merge conflicts")) } else { Ok(ToolResult::ok(conflicts)) }
    }
}

pub struct GitHubCliTool { workspace_root: PathBuf }
impl GitHubCliTool { pub fn new(r: PathBuf) -> Self { Self { workspace_root: r } } }
impl super::Tool for GitHubCliTool {
    fn name(&self) -> &str { "github_cli" }
    fn execute(&self, args: Value) -> Result<ToolResult> {
        let action = args["action"].as_str().unwrap_or("");
        let extra = args["args"].as_str().unwrap_or("");
        let cmd = match action {
            "pr_create" => {
                let title = args["title"].as_str().unwrap_or("fever: changes");
                let body = args["body"].as_str().unwrap_or("");
                format!("gh pr create --title \"{}\" --body \"{}\" {}", title, body, extra)
            }
            "pr_list" => "gh pr list".to_string(),
            "issue_list" => "gh issue list".to_string(),
            "issue_create" => {
                let title = args["title"].as_str().unwrap_or("New issue");
                let body = args["body"].as_str().unwrap_or("");
                format!("gh issue create --title \"{}\" --body \"{}\"", title, body)
            }
            "release_create" => {
                let tag = args["tag"].as_str().unwrap_or("v1.0.0");
                let title = args["title"].as_str().unwrap_or(tag);
                format!("gh release create {} --title \"{}\" --generate-notes", tag, title)
            }
            "ci_status" => "gh run list --limit 5".to_string(),
            _ => return Ok(ToolResult::err("actions: pr_create, pr_list, issue_list, issue_create, release_create, ci_status")),
        };
        let out = std::process::Command::new("sh").arg("-c").arg(&cmd).current_dir(&self.workspace_root).output();
        match out {
            Ok(o) => {
                let stdout = String::from_utf8_lossy(&o.stdout);
                let stderr = String::from_utf8_lossy(&o.stderr);
                let mut result = stdout.to_string();
                if !stderr.is_empty() && !o.status.success() { result.push_str(&format!("\n[stderr] {}", stderr)); }
                Ok(ToolResult { output: result, success: o.status.success() })
            }
            Err(e) => Ok(ToolResult::err(format!("gh CLI failed: {}. Is gh installed?", e))),
        }
    }
}
