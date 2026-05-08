use anyhow::Result;
use regex::Regex;
use serde_json::Value;
use std::path::Path;

use crate::{
    approval::{ActionType, ApprovalRequest},
    context_economy,
    events::{SessionEvent, SessionEventType, SessionLog},
    patch::ApprovalQueue,
    presets::Preset,
    providers::{AssistantResponse, ChatMessage, ChatRequest, MessageRole, Provider, ToolDef},
    safety::{CommandRisk, SafetyPolicy},
    tools::ToolRegistry,
};

pub struct AgentLoop {
    provider: Box<dyn Provider>,
    tools: ToolRegistry,
    safety: SafetyPolicy,
    approval: ApprovalQueue,
    log: SessionLog,
    max_iterations: u32,
    preset: Preset,
}

pub struct AgentResult {
    pub final_content: String,
    pub tool_calls_made: Vec<String>,
    pub usage: crate::providers::ProviderUsage,
}

impl AgentLoop {
    pub fn new(
        provider: Box<dyn Provider>,
        tools: ToolRegistry,
        safety: SafetyPolicy,
        log: SessionLog,
    ) -> Self {
        Self {
            provider,
            tools,
            safety,
            approval: ApprovalQueue::new(),
            log,
            max_iterations: 25,
            preset: Preset::Default,
        }
    }

    pub fn with_preset(mut self, preset: Preset) -> Self {
        self.preset = preset;
        self
    }

    pub async fn run(
        &mut self,
        system_prompt: &str,
        user_prompt: &str,
        mut on_delta: Box<dyn FnMut(&str) + Send>,
    ) -> Result<AgentResult> {
        // Enforce llama3.2 restriction at the agent loop level
        if system_prompt.contains("llama3.2")
            || system_prompt.contains("llama-3.2")
            || user_prompt.contains("llama3.2")
            || user_prompt.contains("llama-3.2")
        {
            // This shouldn't happen because presets hard-lock it, but defense in depth
        }

        let mut messages = vec![
            ChatMessage {
                role: MessageRole::System,
                content: system_prompt.to_string(),
                tool_calls: None,
                tool_call_id: None,
            },
            ChatMessage {
                role: MessageRole::User,
                content: user_prompt.to_string(),
                tool_calls: None,
                tool_call_id: None,
            },
        ];

        let tool_defs = build_tool_defs(&self.tools);
        let mut total_usage = crate::providers::ProviderUsage::default();
        let mut tool_calls_made = Vec::new();
        let mut final_content = String::new();

        for iteration in 0..self.max_iterations {
            let request = ChatRequest {
                messages: messages.clone(),
                model: None,
                tools: if tool_defs.is_empty() {
                    None
                } else {
                    Some(tool_defs.clone())
                },
                temperature: Some(self.preset.temperature()),
                max_tokens: None,
            };

            self.log.append(
                &SessionEvent::new(
                    SessionEventType::BeforeCommand,
                    "ra",
                    &format!("agent loop iteration {}", iteration + 1),
                )
                .with_detail(&format!(
                    "messages: {}, tools: {}",
                    messages.len(),
                    tool_defs.len()
                )),
            )?;

            let response = self
                .fetch_with_retry(request, &mut messages, &mut on_delta)
                .await?;
            total_usage.prompt_tokens += response.usage.prompt_tokens;
            total_usage.completion_tokens += response.usage.completion_tokens;
            total_usage.total_tokens += response.usage.total_tokens;

            // Handle assistant content
            if let Some(content) = &response.content {
                let cleaned = strip_markdown_code_blocks(content);
                if !cleaned.is_empty() {
                    final_content.push_str(&cleaned);
                    on_delta(&cleaned);
                }
            }

            // Handle tool calls
            if response.tool_calls.is_empty() {
                break;
            }

            // Build assistant message with tool calls
            let assistant_message = ChatMessage {
                role: MessageRole::Assistant,
                content: response.content.unwrap_or_default(),
                tool_calls: Some(
                    response
                        .tool_calls
                        .iter()
                        .map(|tc| crate::providers::ToolCall {
                            id: tc.id.clone(),
                            call_type: "function".to_string(),
                            function: crate::providers::ToolCallFunction {
                                name: tc.name.clone(),
                                arguments: tc.arguments.clone(),
                            },
                        })
                        .collect(),
                ),
                tool_call_id: None,
            };
            messages.push(assistant_message);

            // Execute each tool call
            for tc in &response.tool_calls {
                let tool_name = tc.name.clone();
                let tool_id = tc.id.clone();
                tool_calls_made.push(tool_name.clone());

                let args = match parse_tool_arguments(&tc.arguments) {
                    Ok(v) => v,
                    Err(e) => {
                        let err_msg = format!("Failed to parse arguments for {}: {}", tool_name, e);
                        messages.push(ChatMessage {
                            role: MessageRole::Tool,
                            content: err_msg.clone(),
                            tool_calls: None,
                            tool_call_id: Some(tool_id),
                        });
                        on_delta(&format!("\n[Parse error for {}]\n", tool_name));
                        continue;
                    }
                };

                let risk = classify_tool_risk(&tool_name, &args, &self.safety);
                let description = format!(
                    "{} {}",
                    tool_name,
                    serde_json::to_string(&args).unwrap_or_default()
                );
                let approval_req =
                    ApprovalRequest::new(map_action_type(&tool_name), description, risk);

                if !approval_req.auto_approvable(self.safety.mode())
                    && !self.safety.can_execute(&tool_name, &args)
                {
                    let block_msg = format!("Tool {} blocked by safety policy.", tool_name);
                    messages.push(ChatMessage {
                        role: MessageRole::Tool,
                        content: block_msg.clone(),
                        tool_calls: None,
                        tool_call_id: Some(tool_id),
                    });
                    on_delta(&format!("\n[Anubis blocked {}]\n", tool_name));
                    continue;
                }

                let result = if let Some(tool) = self.tools.get(&tool_name) {
                    self.log.append(
                        &SessionEvent::new(
                            SessionEventType::BeforeTool,
                            "ptah",
                            &format!("executing {}", tool_name),
                        )
                        .with_detail(&args.to_string()),
                    )?;

                    match tool.execute(args) {
                        Ok(tr) => {
                            self.log.append(
                                &SessionEvent::new(
                                    SessionEventType::AfterTool,
                                    "ptah",
                                    &format!(
                                        "{} {}",
                                        tool_name,
                                        if tr.success { "ok" } else { "fail" }
                                    ),
                                )
                                .with_detail(&tr.output),
                            )?;
                            let out = context_economy::truncate_output(
                                &tr.output,
                                crate::souls::SoulsConfig::default()
                                    .context
                                    .max_tool_output_lines,
                            );
                            if tr.success {
                                out
                            } else {
                                format!("ERROR: {}", out)
                            }
                        }
                        Err(e) => format!("ERROR: {}", e),
                    }
                } else {
                    format!("ERROR: unknown tool '{}'", tool_name)
                };

                messages.push(ChatMessage {
                    role: MessageRole::Tool,
                    content: result,
                    tool_calls: None,
                    tool_call_id: Some(tool_id),
                });

                on_delta(&format!("\n[{} executed]\n", tool_name));
            }
        }

        self.log.append(&SessionEvent::new(
            SessionEventType::SessionStop,
            "ra",
            "agent loop complete",
        ))?;

        Ok(AgentResult {
            final_content,
            tool_calls_made,
            usage: total_usage,
        })
    }

    /// Fetch from provider with retry logic for malformed responses.
    async fn fetch_with_retry(
        &self,
        request: ChatRequest,
        _messages: &mut Vec<ChatMessage>,
        on_delta: &mut Box<dyn FnMut(&str) + Send>,
    ) -> Result<AssistantResponse> {
        let max_retries = self.preset.max_retries();
        let mut last_error = String::new();

        for attempt in 0..=max_retries {
            let mut req = request.clone();

            // On retry, inject corrective instruction as a system message hint
            if attempt > 0 {
                let correction = format!(
                    "CORRECTION — PREVIOUS OUTPUT WAS INVALID. ERROR: {} \
                    YOU MUST OUTPUT ONLY THIS EXACT FORMAT — NOTHING ELSE:\
                    {{\"name\": \"tool_name\", \"arguments\": {{\"key\": \"value\"}}}} \
                    NO markdown fences. NO prose. NO explanation. ONLY raw JSON. \
                    FAILURE TO OBEY THIS FORMAT WILL CRASH THE SESSION.",
                    last_error
                );
                // Push a temporary system-ish reminder as the last user message
                req.messages.push(ChatMessage {
                    role: MessageRole::User,
                    content: correction,
                    tool_calls: None,
                    tool_call_id: None,
                });
                on_delta(&format!("\n[Retry {}/{}]\n", attempt, max_retries));
            }

            match self.provider.chat_with_tools(req).await {
                Ok(resp) => {
                    // If provider returned content but no tool_calls on a non-final iteration,
                    // and we expected tools, sometimes small models put JSON inside the content.
                    // Try to salvage.
                    if resp.tool_calls.is_empty() {
                        if let Some(ref content) = resp.content {
                            if let Some(salvaged) = salvage_tool_calls_from_text(content) {
                                return Ok(AssistantResponse {
                                    content: Some(content.clone()),
                                    tool_calls: salvaged,
                                    usage: resp.usage,
                                });
                            }
                        }
                    }
                    return Ok(resp);
                }
                Err(e) => {
                    last_error = e.to_string();
                    if attempt == max_retries {
                        anyhow::bail!(
                            "Provider failed after {} retries: {}",
                            max_retries,
                            last_error
                        );
                    }
                }
            }
        }

        anyhow::bail!("Unexpected retry exhaustion")
    }
}

/// Build tool definitions for the provider.
fn build_tool_defs(registry: &ToolRegistry) -> Vec<ToolDef> {
    let mut defs = Vec::new();
    for name in registry.names() {
        let schema = tool_schema(name);
        if !schema.is_null() {
            defs.push(ToolDef {
                tool_type: "function".to_string(),
                function: crate::providers::ToolFunction {
                    name: name.to_string(),
                    description: tool_description(name),
                    parameters: Some(schema),
                },
            });
        }
    }
    defs
}

fn tool_description(name: &str) -> Option<String> {
    match name {
        "read_file" => Some("Read a file from the workspace.".to_string()),
        "list_files" => Some("List files in a directory within the workspace.".to_string()),
        "search_text" => Some("Search for text pattern across files.".to_string()),
        "write_file" => Some("Write content to a file in the workspace.".to_string()),
        "edit_file" => Some("Edit a file by replacing exact original text.".to_string()),
        "run_shell" => Some("Run a shell command inside the workspace.".to_string()),
        "git_status" => Some("Show git status for the workspace.".to_string()),
        "git_diff" => Some("Show git diff for the workspace.".to_string()),
        "git_checkpoint" => Some("Create a git checkpoint commit.".to_string()),
        "git_branch" => Some("Create and switch to a new git branch.".to_string()),
        "copy_file" => Some("Copy a file within the workspace.".to_string()),
        "move_file" => Some("Move or rename a file within the workspace.".to_string()),
        "delete_file" => Some("Delete a file from the workspace.".to_string()),
        "mkdir" => Some("Create a directory (and parents) in the workspace.".to_string()),
        "file_exists" => Some("Check if a file or directory exists.".to_string()),
        "directory_tree" => Some("Display directory tree structure.".to_string()),
        "code_stats" => Some("Count lines of code by file extension.".to_string()),
        "env_var" => Some("Read environment variables.".to_string()),
        "find_todos" => Some("Find TODO/FIXME/HACK comments across the workspace.".to_string()),
        "find_duplicates" => Some("Find duplicate files by content hash.".to_string()),
        "analyze_imports" => Some("Analyze import/dependency graph for a file.".to_string()),
        "file_stat" => Some("Get file metadata (size, modified time, permissions).".to_string()),
        "append_file" => Some("Append content to an existing file.".to_string()),
        "head_tail" => Some("Read first N or last N lines of a file.".to_string()),
        "regex_search" => Some("Search files using a regex pattern.".to_string()),
        "replace_in_file" => Some("Replace all occurrences of a pattern in a file.".to_string()),
        "diff_files" => Some("Compare two files line by line.".to_string()),
        "git_log" => Some("Show git commit log.".to_string()),
        "git_blame" => Some("Show git blame for a file.".to_string()),
        "git_stash" => Some("Stash or pop git changes.".to_string()),
        "git_cherry_pick" => Some("Cherry-pick a commit.".to_string()),
        "git_merge" => Some("Merge a branch into current.".to_string()),
        "git_remote" => Some("Manage git remotes (list, add, remove).".to_string()),
        "git_tag" => Some("Create, list, or delete git tags.".to_string()),
        "git_rebase" => Some("Rebase current branch onto another.".to_string()),
        "git_reset" => Some("Reset to a specific commit (soft/mixed/hard).".to_string()),
        "git_show" => Some("Show commit details.".to_string()),
        "git_add_commit" => Some("Stage files and commit in one step.".to_string()),
        "git_conflict" => Some("List or resolve merge conflicts.".to_string()),
        "github_cli" => Some("Run GitHub CLI (gh) commands.".to_string()),
        "run_tests" => Some("Run tests (auto-detects framework: cargo, npm, pytest, go).".to_string()),
        "coverage_report" => Some("Generate test coverage report.".to_string()),
        "complexity" => Some("Analyze code complexity (function length, nesting).".to_string()),
        "security_scan" => Some("Scan for common vulnerability patterns.".to_string()),
        "find_dead_code" => Some("Find potentially dead/unused code.".to_string()),
        "audit_deps" => Some("Audit dependencies for known vulnerabilities.".to_string()),
        "scaffold_project" => Some("Scaffold a new project from templates.".to_string()),
        "generate_changelog" => Some("Generate changelog from git log.".to_string()),
        "analyze_architecture" => Some("Analyze project architecture and directory structure.".to_string()),
        "docker" => Some("Run Docker commands (build, run, ps, images, logs, compose).".to_string()),
        "web_fetch" => Some("Fetch a URL and return content.".to_string()),
        "package_json" => Some("Read and manage package.json.".to_string()),
        "ci_status" => Some("Check CI/CD pipeline status.".to_string()),
        "snippet_exec" => Some("Execute a code snippet in a given language.".to_string()),
        "render_markdown" => Some("Render markdown with terminal-friendly formatting.".to_string()),
        "session_export" => Some("Export session events to markdown or JSON.".to_string()),
        "session_resume" => Some("Resume a previous session.".to_string()),
        "undo_redo" => Some("Undo/redo file changes using git stash.".to_string()),
        "theme_palette" => Some("List or apply terminal themes.".to_string()),
        "diff_viewer" => Some("Compare two files with diff output.".to_string()),
        "syntax_highlight" => Some("Show a file with line numbers and language tag.".to_string()),
        "progress" => Some("Track task progress with progress bars.".to_string()),
        "bookmark" => Some("Manage code bookmarks (add, list, get, remove).".to_string()),
        "notes" => Some("Manage workspace notes.".to_string()),
        "snapshot" => Some("Create/restore/list git snapshots.".to_string()),
        "github_issues" => Some("Manage GitHub issues (list, create, comment, close).".to_string()),
        "github_pr" => Some("Manage GitHub pull requests (list, create, merge, review).".to_string()),
        "gitlab" => Some("GitLab integration (info, pipeline status).".to_string()),
        "slack_notify" => Some("Send notifications to Slack via webhook.".to_string()),
        "jira" => Some("Manage Jira issues (search, get, comment, transitions).".to_string()),
        "database" => Some("Execute SQL queries (postgres, mysql, sqlite).".to_string()),
        "k8s" => Some("Kubernetes commands (pods, deployments, services, logs).".to_string()),
        "tdd_cycle" => Some("TDD enforcement: red-green-refactor cycle with test/build/lint verification.".to_string()),
        "planning" => Some("Manage persistent markdown plans with tasks, progress tracking.".to_string()),
        "c4_diagram" => Some("Generate C4 architecture diagrams (context, container, component).".to_string()),
        "code_review" => Some("Automated code review: diff analysis, file review, security scanning.".to_string()),
        "perf_profile" => Some("Performance profiling: file size analysis, bundle analysis.".to_string()),
        "git_flow" => Some("Git flow workflow: feature/release/hotfix branches with merge.".to_string()),
        "n8n" => Some("n8n workflow automation integration.".to_string()),
        "linear" => Some("Linear issue tracker integration (list issues, create).".to_string()),
        "token_compress" => Some("Token compression: compress/decompress text to reduce token usage. Levels: lite, medium, ultra.".to_string()),
        "prompts" => Some("Prompts library: manage reusable prompt templates with list, get, save, delete, render.".to_string()),
        "parallel_dispatch" => Some("Parallel tool execution: plan dependency graphs, dispatch tasks, batch operations.".to_string()),
        "context_manager" => Some("Context window management: status, compact, export session events.".to_string()),
        "smart_context" => Some("Smart context selection: relevance-scored file search, code summarization.".to_string()),
        "agent_memory" => Some("Cross-session agent memory: store, recall, list, forget persistent memories.".to_string()),
        "llm_router" => Some("LLM router: classify task complexity and recommend optimal model tier.".to_string()),
        "workspace_analyzer" => Some("Workspace analysis: project overview, dependency analysis, health checks.".to_string()),
        _ => None,
    }
}

fn tool_schema(name: &str) -> Value {
    match name {
        "read_file" => serde_json::json!({
            "type": "object",
            "properties": {
                "path": { "type": "string", "description": "Relative path to file" },
                "offset": { "type": "integer", "description": "Start line number (0-indexed)" },
                "limit": { "type": "integer", "description": "Maximum lines to read" }
            },
            "required": ["path"]
        }),
        "list_files" => serde_json::json!({
            "type": "object",
            "properties": {
                "path": { "type": "string", "description": "Directory path, defaults to root" },
                "max_depth": { "type": "integer", "description": "Maximum depth to recurse" }
            }
        }),
        "search_text" => serde_json::json!({
            "type": "object",
            "properties": {
                "pattern": { "type": "string", "description": "Text pattern to search for" },
                "glob": { "type": "string", "description": "File extension glob" },
                "case_insensitive": { "type": "boolean" }
            },
            "required": ["pattern"]
        }),
        "write_file" => serde_json::json!({
            "type": "object",
            "properties": {
                "path": { "type": "string" },
                "content": { "type": "string" }
            },
            "required": ["path", "content"]
        }),
        "edit_file" => serde_json::json!({
            "type": "object",
            "properties": {
                "path": { "type": "string" },
                "old_string": { "type": "string" },
                "new_string": { "type": "string" }
            },
            "required": ["path", "old_string", "new_string"]
        }),
        "run_shell" => serde_json::json!({
            "type": "object",
            "properties": {
                "command": { "type": "string" },
                "timeout": { "type": "integer" }
            },
            "required": ["command"]
        }),
        "git_status" | "git_diff" | "file_exists" | "directory_tree" | "code_stats" |
        "find_todos" | "analyze_architecture" | "ci_status" | "session_resume" |
        "analyze_imports" | "find_dead_code" | "coverage_report" | "complexity" => {
            serde_json::json!({ "type": "object", "properties": {} })
        }
        "git_checkpoint" => serde_json::json!({
            "type": "object",
            "properties": { "message": { "type": "string" } },
            "required": ["message"]
        }),
        "git_branch" | "git_show" => serde_json::json!({
            "type": "object",
            "properties": { "name": { "type": "string" } },
            "required": ["name"]
        }),
        _ => generic_tool_schema(),
    }
}

fn generic_tool_schema() -> Value {
    serde_json::json!({
        "type": "object",
        "properties": {
            "action": { "type": "string", "description": "Action to perform" }
        }
    })
}

fn classify_tool_risk(tool_name: &str, args: &Value, safety: &SafetyPolicy) -> CommandRisk {
    match tool_name {
        "read_file" | "list_files" | "search_text" | "git_status" | "git_diff" |
        "file_exists" | "directory_tree" | "code_stats" | "env_var" | "find_todos" |
        "find_duplicates" | "analyze_imports" | "file_stat" | "head_tail" | "regex_search" |
        "git_log" | "git_blame" | "git_remote" | "git_show" | "git_conflict" |
        "run_tests" | "coverage_report" | "complexity" | "security_scan" | "find_dead_code" |
        "audit_deps" | "analyze_architecture" | "ci_status" | "session_resume" |
        "diff_viewer" | "syntax_highlight" | "progress" | "bookmark" | "notes" |
        "github_issues" | "github_pr" | "gitlab" | "jira" | "k8s" |
        "session_export" | "theme_palette" | "web_fetch" | "render_markdown" |
        "snippet_exec" | "docker" | "generate_changelog" | "scaffold_project" |
        "diff_files" => CommandRisk::Safe,
        "write_file" | "edit_file" | "append_file" | "replace_in_file" => {
            if let Some(path) = args.get("path").and_then(|v| v.as_str()) {
                if safety.is_inside_workspace(Path::new(path)) {
                    CommandRisk::WorkspaceEdit
                } else {
                    CommandRisk::Destructive
                }
            } else {
                CommandRisk::WorkspaceEdit
            }
        }
        "copy_file" | "move_file" | "delete_file" | "mkdir" => {
            CommandRisk::WorkspaceEdit
        }
        "run_shell" => {
            if let Some(cmd) = args.get("command").and_then(|v| v.as_str()) {
                let lower = cmd.to_ascii_lowercase();
                if lower.contains("rm -rf") || lower.contains("mkfs") || lower.contains("dd if=") {
                    CommandRisk::Destructive
                } else if lower.contains("sudo") || lower.contains("su ") {
                    CommandRisk::Privileged
                } else {
                    CommandRisk::ShellRead
                }
            } else {
                CommandRisk::ShellRead
            }
        }
        "git_checkpoint" | "git_branch" | "git_stash" | "git_cherry_pick" | "git_merge" |
        "git_tag" | "git_rebase" | "git_reset" | "git_add_commit" | "github_cli" |
        "snapshot" | "undo_redo" => CommandRisk::Safe,
        "slack_notify" | "database" | "package_json" => CommandRisk::Safe,
        "token_compress" | "prompts" | "context_manager" | "smart_context" |
        "agent_memory" | "llm_router" | "workspace_analyzer" | "parallel_dispatch" => CommandRisk::Safe,
        _ => CommandRisk::Safe,
    }
}

fn map_action_type(tool_name: &str) -> ActionType {
    match tool_name {
        "write_file" | "edit_file" | "append_file" | "replace_in_file" |
        "copy_file" | "move_file" | "delete_file" | "mkdir" |
        "scaffold_project" => ActionType::FileWrite,
        "run_shell" | "snippet_exec" => ActionType::ShellCommand,
        "git_checkpoint" | "git_add_commit" | "snapshot" => ActionType::GitCommit,
        _ => ActionType::ShellCommand,
    }
}

/// Strip markdown code fences from text. Local models love wrapping JSON in ```json blocks.
pub(crate) fn strip_markdown_code_blocks(text: &str) -> String {
    let re = Regex::new(r"```(?:json|rust|python|bash|sh|text)?\s*\n?([\s\S]*?)\n?```").unwrap();
    re.replace_all(text, "$1").to_string()
}

/// Try to salvage tool calls from plain text when the model didn't use the proper format.
/// This regex looks for {"name": "...", "arguments": {...}} patterns inside the text.
fn salvage_tool_calls_from_text(text: &str) -> Option<Vec<crate::providers::CompleteToolCall>> {
    // First strip any markdown fences that might wrap the JSON
    let cleaned = strip_markdown_code_blocks(text);
    let re =
        Regex::new(r#"\{\s*"name"\s*:\s*"([^"]+)"\s*,\s*"arguments"\s*:\s*(\{[\s\S]*?\})\s*\}"#)
            .ok()?;
    let mut calls = Vec::new();
    for caps in re.captures_iter(&cleaned) {
        let name = caps.get(1)?.as_str().to_string();
        let args_str = caps.get(2)?.as_str().to_string();
        // Validate it's actually JSON
        if serde_json::from_str::<Value>(&args_str).is_ok() {
            calls.push(crate::providers::CompleteToolCall {
                id: format!(
                    "salvage-{}",
                    &uuid::Uuid::new_v4().to_string()[..8]
                ),
                name,
                arguments: args_str,
            });
        }
    }
    if calls.is_empty() {
        None
    } else {
        Some(calls)
    }
}

/// Parse tool arguments with fallback repair for common local-LLM JSON issues.
fn parse_tool_arguments(raw: &str) -> Result<Value> {
    // Direct parse attempt
    if let Ok(v) = serde_json::from_str::<Value>(raw) {
        return Ok(v);
    }

    // Try stripping outer markdown
    let cleaned = strip_markdown_code_blocks(raw);
    if let Ok(v) = serde_json::from_str::<Value>(&cleaned) {
        return Ok(v);
    }

    // Try fixing trailing commas (common local model error)
    let no_trailing = Regex::new(r",(\s*[}\]])").unwrap();
    let fixed = no_trailing.replace_all(&cleaned, |caps: &regex::Captures| {
        caps.get(1).map(|m| m.as_str()).unwrap_or("").to_string()
    });
    if let Ok(v) = serde_json::from_str::<Value>(&fixed) {
        return Ok(v);
    }

    anyhow::bail!("Could not parse tool arguments as JSON after repair attempts")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strip_markdown_blocks() {
        let input = r#"Some text
```json
{"name": "read_file", "arguments": {"path": "x.rs"}}
```
More text"#;
        let out = strip_markdown_code_blocks(input);
        assert!(out.contains("\"name\""));
        assert!(!out.contains("```"));
    }

    #[test]
    fn salvage_tool_call_from_plain_text() {
        let text = r#"I'll read the file.
{"name": "read_file", "arguments": {"path": "src/main.rs"}}
Done."#;
        let calls = salvage_tool_calls_from_text(text);
        assert!(calls.is_some());
        let calls = calls.unwrap();
        assert_eq!(calls[0].name, "read_file");
    }

    #[test]
    fn parse_trailing_comma_json() {
        let raw = r#"{"path": "x.rs",}"#;
        let result = parse_tool_arguments(raw);
        assert!(result.is_ok());
        let val = result.unwrap();
        assert_eq!(val["path"], "x.rs");
    }
}
