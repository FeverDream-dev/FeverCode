use anyhow::Result;
use serde_json::Value;
use std::path::PathBuf;

use super::ToolResult;

pub struct GitHubIssuesTool { workspace_root: PathBuf }
impl GitHubIssuesTool { pub fn new(r: PathBuf) -> Self { Self { workspace_root: r } } }
impl super::Tool for GitHubIssuesTool {
    fn name(&self) -> &str { "github_issues" }
    fn execute(&self, args: Value) -> Result<ToolResult> {
        let action = args["action"].as_str().unwrap_or("list");
        match action {
            "list" => {
                let state = args["state"].as_str().unwrap_or("open");
                let limit = args["limit"].as_u64().unwrap_or(20);
                let out = std::process::Command::new("gh")
                    .args(["issue", "list", "--state", state, "--limit", &limit.to_string(), "--json", "number,title,state,labels"])
                    .current_dir(&self.workspace_root)
                    .output()?;
                if !out.status.success() {
                    return Ok(ToolResult::err(format!("gh CLI error: {}", String::from_utf8_lossy(&out.stderr))));
                }
                let issues: Vec<Value> = serde_json::from_str(&String::from_utf8_lossy(&out.stdout)).unwrap_or_default();
                if issues.is_empty() { return Ok(ToolResult::ok("no issues found")); }
                let mut result = format!("Issues ({}):\n", state);
                for issue in &issues {
                    let num = issue["number"].as_u64().unwrap_or(0);
                    let title = issue["title"].as_str().unwrap_or("");
                    let labels: Vec<String> = issue["labels"].as_array()
                        .map(|arr| arr.iter().filter_map(|l| l["name"].as_str().map(String::from)).collect())
                        .unwrap_or_default();
                    result.push_str(&format!("  #{} {} {}\n", num, title, if labels.is_empty() { String::new() } else { format!("[{}]", labels.join(", ")) }));
                }
                Ok(ToolResult::ok(result))
            }
            "create" => {
                let title = args["title"].as_str().unwrap_or("");
                let body = args["body"].as_str().unwrap_or("");
                if title.is_empty() { return Ok(ToolResult::err("title required")); }
                let mut cmd = std::process::Command::new("gh");
                cmd.args(["issue", "create", "--title", title]);
                if !body.is_empty() { cmd.arg("--body"); cmd.arg(body); }
                let labels = args["labels"].as_str().unwrap_or("");
                if !labels.is_empty() { cmd.arg("--label"); cmd.arg(labels); }
                let out = cmd.current_dir(&self.workspace_root).output()?;
                if out.status.success() {
                    Ok(ToolResult::ok(String::from_utf8_lossy(&out.stdout).to_string()))
                } else {
                    Ok(ToolResult::err(String::from_utf8_lossy(&out.stderr).to_string()))
                }
            }
            "comment" => {
                let number = args["number"].as_u64().unwrap_or(0);
                let body = args["body"].as_str().unwrap_or("");
                if number == 0 || body.is_empty() { return Ok(ToolResult::err("number and body required")); }
                let out = std::process::Command::new("gh")
                    .args(["issue", "comment", &number.to_string(), "--body", body])
                    .current_dir(&self.workspace_root)
                    .output()?;
                if out.status.success() {
                    Ok(ToolResult::ok(format!("commented on #{}", number)))
                } else {
                    Ok(ToolResult::err(String::from_utf8_lossy(&out.stderr).to_string()))
                }
            }
            "close" => {
                let number = args["number"].as_u64().unwrap_or(0);
                if number == 0 { return Ok(ToolResult::err("number required")); }
                let out = std::process::Command::new("gh")
                    .args(["issue", "close", &number.to_string()])
                    .current_dir(&self.workspace_root)
                    .output()?;
                if out.status.success() {
                    Ok(ToolResult::ok(format!("closed #{}", number)))
                } else {
                    Ok(ToolResult::err(String::from_utf8_lossy(&out.stderr).to_string()))
                }
            }
            _ => Ok(ToolResult::err("actions: list, create, comment, close")),
        }
    }
}

pub struct GitHubPrTool { workspace_root: PathBuf }
impl GitHubPrTool { pub fn new(r: PathBuf) -> Self { Self { workspace_root: r } } }
impl super::Tool for GitHubPrTool {
    fn name(&self) -> &str { "github_pr" }
    fn execute(&self, args: Value) -> Result<ToolResult> {
        let action = args["action"].as_str().unwrap_or("list");
        match action {
            "list" => {
                let state = args["state"].as_str().unwrap_or("open");
                let limit = args["limit"].as_u64().unwrap_or(20);
                let out = std::process::Command::new("gh")
                    .args(["pr", "list", "--state", state, "--limit", &limit.to_string(), "--json", "number,title,author,headRefName"])
                    .current_dir(&self.workspace_root)
                    .output()?;
                if !out.status.success() {
                    return Ok(ToolResult::err(format!("gh CLI error: {}", String::from_utf8_lossy(&out.stderr))));
                }
                let prs: Vec<Value> = serde_json::from_str(&String::from_utf8_lossy(&out.stdout)).unwrap_or_default();
                if prs.is_empty() { return Ok(ToolResult::ok("no PRs found")); }
                let mut result = format!("Pull Requests ({}):\n", state);
                for pr in &prs {
                    let num = pr["number"].as_u64().unwrap_or(0);
                    let title = pr["title"].as_str().unwrap_or("");
                    let branch = pr["headRefName"].as_str().unwrap_or("");
                    result.push_str(&format!("  #{} [{}] {}\n", num, branch, title));
                }
                Ok(ToolResult::ok(result))
            }
            "create" => {
                let title = args["title"].as_str().unwrap_or("");
                let body = args["body"].as_str().unwrap_or("");
                let base = args["base"].as_str().unwrap_or("main");
                if title.is_empty() { return Ok(ToolResult::err("title required")); }
                let mut cmd = std::process::Command::new("gh");
                cmd.args(["pr", "create", "--title", title, "--base", base]);
                if !body.is_empty() { cmd.arg("--body"); cmd.arg(body); }
                let out = cmd.current_dir(&self.workspace_root).output()?;
                if out.status.success() {
                    Ok(ToolResult::ok(String::from_utf8_lossy(&out.stdout).to_string()))
                } else {
                    Ok(ToolResult::err(String::from_utf8_lossy(&out.stderr).to_string()))
                }
            }
            "merge" => {
                let number = args["number"].as_u64().unwrap_or(0);
                let method = args["method"].as_str().unwrap_or("merge");
                if number == 0 { return Ok(ToolResult::err("number required")); }
                let out = std::process::Command::new("gh")
                    .args(["pr", "merge", &number.to_string(), "--auto", &format!("--{}", method)])
                    .current_dir(&self.workspace_root)
                    .output()?;
                if out.status.success() {
                    Ok(ToolResult::ok(format!("PR #{} set to auto-merge ({})", number, method)))
                } else {
                    Ok(ToolResult::err(String::from_utf8_lossy(&out.stderr).to_string()))
                }
            }
            "review" => {
                let number = args["number"].as_u64().unwrap_or(0);
                if number == 0 { return Ok(ToolResult::err("number required")); }
                let out = std::process::Command::new("gh")
                    .args(["pr", "view", &number.to_string(), "--json", "title,body,additions,deletions,changedFiles,commits"])
                    .current_dir(&self.workspace_root)
                    .output()?;
                if out.status.success() {
                    Ok(ToolResult::ok(String::from_utf8_lossy(&out.stdout)))
                } else {
                    Ok(ToolResult::err(String::from_utf8_lossy(&out.stderr).to_string()))
                }
            }
            _ => Ok(ToolResult::err("actions: list, create, merge, review")),
        }
    }
}

pub struct GitLabTool { workspace_root: PathBuf }
impl GitLabTool { pub fn new(r: PathBuf) -> Self { Self { workspace_root: r } } }
impl super::Tool for GitLabTool {
    fn name(&self) -> &str { "gitlab" }
    fn execute(&self, args: Value) -> Result<ToolResult> {
        let action = args["action"].as_str().unwrap_or("info");
        match action {
            "info" => {
                let out = std::process::Command::new("git")
                    .args(["remote", "-v"])
                    .current_dir(&self.workspace_root)
                    .output()?;
                let remotes = String::from_utf8_lossy(&out.stdout);
                let has_gitlab = remotes.contains("gitlab.com") || remotes.contains("gitlab");
                if has_gitlab {
                    Ok(ToolResult::ok(format!("GitLab remote detected:\n{}", remotes)))
                } else {
                    Ok(ToolResult::ok("no GitLab remote configured.\nSet GITLAB_TOKEN env var and add gitlab remote to use GitLab integration."))
                }
            }
            "pipeline" => {
                let token = std::env::var("GITLAB_TOKEN").unwrap_or_default();
                if token.is_empty() { return Ok(ToolResult::err("GITLAB_TOKEN env var not set")); }
                let remote_out = std::process::Command::new("git")
                    .args(["remote", "get-url", "origin"])
                    .current_dir(&self.workspace_root)
                    .output()?;
                let remote = String::from_utf8_lossy(&remote_out.stdout).trim().to_string();
                let project = remote.trim_start_matches("https://gitlab.com/")
                    .trim_start_matches("git@gitlab.com:")
                    .trim_end_matches(".git")
                    .replace('/', "%2F");
                let branch_out = std::process::Command::new("git")
                    .args(["branch", "--show-current"])
                    .current_dir(&self.workspace_root)
                    .output()?;
                let branch = String::from_utf8_lossy(&branch_out.stdout).trim().to_string();
                let url = format!("https://gitlab.com/api/v4/projects/{}/pipelines?ref={}&per_page=5", project, branch);
                let curl_out = std::process::Command::new("curl")
                    .args(["-s", "-H", &format!("PRIVATE-TOKEN: {}", token), &url])
                    .output()?;
                let pipelines: Vec<Value> = serde_json::from_str(&String::from_utf8_lossy(&curl_out.stdout)).unwrap_or_default();
                if pipelines.is_empty() { return Ok(ToolResult::ok("no pipelines found")); }
                let mut result = String::from("Recent pipelines:\n");
                for p in &pipelines {
                    let id = p["id"].as_u64().unwrap_or(0);
                    let status = p["status"].as_str().unwrap_or("?");
                    let ref_name = p["ref"].as_str().unwrap_or("?");
                    result.push_str(&format!("  #{} — {} ({})\n", id, status, ref_name));
                }
                Ok(ToolResult::ok(result))
            }
            _ => Ok(ToolResult::err("actions: info, pipeline")),
        }
    }
}

pub struct SlackNotifyTool { workspace_root: PathBuf }
impl SlackNotifyTool { pub fn new(r: PathBuf) -> Self { Self { workspace_root: r } } }
impl super::Tool for SlackNotifyTool {
    fn name(&self) -> &str { "slack_notify" }
    fn execute(&self, args: Value) -> Result<ToolResult> {
        let webhook = std::env::var("SLACK_WEBHOOK_URL").unwrap_or_default();
        if webhook.is_empty() { return Ok(ToolResult::err("SLACK_WEBHOOK_URL env var not set")); }
        let message = args["message"].as_str().unwrap_or("");
        if message.is_empty() { return Ok(ToolResult::err("message required")); }
        let payload = serde_json::json!({ "text": message }).to_string();
        let out = std::process::Command::new("curl")
            .args(["-s", "-X", "POST", "-H", "Content-Type: application/json", "-d", &payload, &webhook])
            .output()?;
        if out.status.success() {
            let resp = String::from_utf8_lossy(&out.stdout);
            if resp == "ok" {
                Ok(ToolResult::ok("message sent to Slack"))
            } else {
                Ok(ToolResult::err(format!("Slack response: {}", resp)))
            }
        } else {
            Ok(ToolResult::err(String::from_utf8_lossy(&out.stderr).to_string()))
        }
    }
}

pub struct JiraTool { workspace_root: PathBuf }
impl JiraTool { pub fn new(r: PathBuf) -> Self { Self { workspace_root: r } } }
impl super::Tool for JiraTool {
    fn name(&self) -> &str { "jira" }
    fn execute(&self, args: Value) -> Result<ToolResult> {
        let base_url = std::env::var("JIRA_BASE_URL").unwrap_or_default();
        let token = std::env::var("JIRA_API_TOKEN").unwrap_or_default();
        let email = std::env::var("JIRA_EMAIL").unwrap_or_default();
        if base_url.is_empty() || token.is_empty() || email.is_empty() {
            return Ok(ToolResult::err("set JIRA_BASE_URL, JIRA_API_TOKEN, JIRA_EMAIL env vars"));
        }
        let action = args["action"].as_str().unwrap_or("search");
        match action {
            "search" => {
                let jql = args["jql"].as_str().unwrap_or("assignee = currentUser() AND resolution = Unresolved ORDER BY updated DESC");
                let url = format!("{}/rest/api/2/search?jql={}&maxResults=10", base_url, urlencoding::encode(jql));
                let out = std::process::Command::new("curl")
                    .args(["-s", "-u", &format!("{}:{}", email, token), &url])
                    .output()?;
                let data: Value = serde_json::from_str(&String::from_utf8_lossy(&out.stdout)).unwrap_or_default();
                let issues = data["issues"].as_array().cloned().unwrap_or_default();
                if issues.is_empty() { return Ok(ToolResult::ok("no Jira issues found")); }
                let mut result = String::from("Jira issues:\n");
                for issue in &issues {
                    let key = issue["key"].as_str().unwrap_or("?");
                    let summary = issue["fields"]["summary"].as_str().unwrap_or("?");
                    let status = issue["fields"]["status"]["name"].as_str().unwrap_or("?");
                    result.push_str(&format!("  {} — [{}] {}\n", key, status, summary));
                }
                Ok(ToolResult::ok(result))
            }
            "get" => {
                let key = args["key"].as_str().unwrap_or("");
                if key.is_empty() { return Ok(ToolResult::err("key required (e.g. PROJ-123)")); }
                let url = format!("{}/rest/api/2/issue/{}", base_url, key);
                let out = std::process::Command::new("curl")
                    .args(["-s", "-u", &format!("{}:{}", email, token), &url])
                    .output()?;
                let data: Value = serde_json::from_str(&String::from_utf8_lossy(&out.stdout)).unwrap_or_default();
                let summary = data["fields"]["summary"].as_str().unwrap_or("?");
                let status = data["fields"]["status"]["name"].as_str().unwrap_or("?");
                let desc = data["fields"]["description"].as_str().unwrap_or("no description");
                Ok(ToolResult::ok(format!("{} — [{}]\n{}\n\n{}", key, status, summary, desc)))
            }
            "comment" => {
                let key = args["key"].as_str().unwrap_or("");
                let body = args["body"].as_str().unwrap_or("");
                if key.is_empty() || body.is_empty() { return Ok(ToolResult::err("key and body required")); }
                let url = format!("{}/rest/api/2/issue/{}/comment", base_url, key);
                let payload = serde_json::json!({ "body": body }).to_string();
                let out = std::process::Command::new("curl")
                    .args(["-s", "-X", "POST", "-u", &format!("{}:{}", email, token), "-H", "Content-Type: application/json", "-d", &payload, &url])
                    .output()?;
                if out.status.success() {
                    Ok(ToolResult::ok(format!("comment added to {}", key)))
                } else {
                    Ok(ToolResult::err(String::from_utf8_lossy(&out.stderr).to_string()))
                }
            }
            "transitions" => {
                let key = args["key"].as_str().unwrap_or("");
                let transition_id = args["transition_id"].as_str().unwrap_or("");
                if key.is_empty() || transition_id.is_empty() { return Ok(ToolResult::err("key and transition_id required")); }
                let url = format!("{}/rest/api/2/issue/{}/transitions", base_url, key);
                let payload = serde_json::json!({ "transition": { "id": transition_id } }).to_string();
                let out = std::process::Command::new("curl")
                    .args(["-s", "-X", "POST", "-u", &format!("{}:{}", email, token), "-H", "Content-Type: application/json", "-d", &payload, &url])
                    .output()?;
                if out.status.success() {
                    Ok(ToolResult::ok(format!("transition applied to {}", key)))
                } else {
                    Ok(ToolResult::err(String::from_utf8_lossy(&out.stderr).to_string()))
                }
            }
            _ => Ok(ToolResult::err("actions: search, get, comment, transitions")),
        }
    }
}

pub struct DatabaseTool { workspace_root: PathBuf }
impl DatabaseTool { pub fn new(r: PathBuf) -> Self { Self { workspace_root: r } } }
impl super::Tool for DatabaseTool {
    fn name(&self) -> &str { "database" }
    fn execute(&self, args: Value) -> Result<ToolResult> {
        let db_type = args["type"].as_str().unwrap_or("");
        let command = args["command"].as_str().unwrap_or("");
        let database = args["database"].as_str().unwrap_or("");
        if db_type.is_empty() || command.is_empty() { return Ok(ToolResult::err("type and command required")); }
        let (cmd_name, cmd_args) = match db_type {
            "postgres" | "postgresql" | "psql" => ("psql", vec!["-c", command]),
            "mysql" => ("mysql", vec!["-e", command]),
            "sqlite" | "sqlite3" => {
                if database.is_empty() { return Ok(ToolResult::err("database file path required for sqlite")); }
                ("sqlite3", vec![database, command])
            }
            _ => return Ok(ToolResult::err("supported: postgres, mysql, sqlite")),
        };
        let mut proc = std::process::Command::new(cmd_name);
        proc.args(&cmd_args).current_dir(&self.workspace_root);
        if !database.is_empty() && db_type != "sqlite" && db_type != "sqlite3" {
            proc.arg(database);
        }
        let out = proc.output()?;
        if out.status.success() {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let truncated: String = stdout.chars().take(5000).collect();
            Ok(ToolResult::ok(truncated))
        } else {
            Ok(ToolResult::err(String::from_utf8_lossy(&out.stderr).to_string()))
        }
    }
}

pub struct KubernetesTool { workspace_root: PathBuf }
impl KubernetesTool { pub fn new(r: PathBuf) -> Self { Self { workspace_root: r } } }
impl super::Tool for KubernetesTool {
    fn name(&self) -> &str { "k8s" }
    fn execute(&self, args: Value) -> Result<ToolResult> {
        let action = args["action"].as_str().unwrap_or("pods");
        let namespace = args["namespace"].as_str().unwrap_or("default");
        let (cmd, kubectl_args): (&str, Vec<String>) = match action {
            "pods" => ("kubectl", ["get", "pods", "-n", namespace].iter().map(|s| s.to_string()).collect()),
            "deployments" | "deploys" => ("kubectl", ["get", "deployments", "-n", namespace].iter().map(|s| s.to_string()).collect()),
            "services" | "svc" => ("kubectl", ["get", "services", "-n", namespace].iter().map(|s| s.to_string()).collect()),
            "logs" => {
                let pod = args["pod"].as_str().unwrap_or("");
                if pod.is_empty() { return Ok(ToolResult::err("pod name required")); }
                let tail = args["tail"].as_u64().unwrap_or(50);
                let tail_str = tail.to_string();
                ("kubectl", ["logs", pod, "-n", namespace, "--tail", &tail_str].iter().map(|s| s.to_string()).collect())
            }
            "describe" => {
                let resource = args["resource"].as_str().unwrap_or("");
                let name = args["name"].as_str().unwrap_or("");
                if resource.is_empty() || name.is_empty() { return Ok(ToolResult::err("resource type and name required")); }
                ("kubectl", ["describe", resource, name, "-n", namespace].iter().map(|s| s.to_string()).collect())
            }
            "apply" => {
                let file = args["file"].as_str().unwrap_or("");
                if file.is_empty() { return Ok(ToolResult::err("file required")); }
                ("kubectl", ["apply", "-f", file].iter().map(|s| s.to_string()).collect())
            }
            _ => return Ok(ToolResult::err("actions: pods, deployments, services, logs, describe, apply")),
        };
        let out = std::process::Command::new(cmd)
            .args(&kubectl_args)
            .current_dir(&self.workspace_root)
            .output()?;
        if out.status.success() {
            Ok(ToolResult::ok(String::from_utf8_lossy(&out.stdout).to_string()))
        } else {
            Ok(ToolResult::err(String::from_utf8_lossy(&out.stderr).to_string()))
        }
    }
}
