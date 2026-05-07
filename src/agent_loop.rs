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
        if system_prompt.contains("llama3.2") || system_prompt.contains("llama-3.2")
            || user_prompt.contains("llama3.2") || user_prompt.contains("llama-3.2")
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
                tools: if tool_defs.is_empty() { None } else { Some(tool_defs.clone()) },
                temperature: Some(self.preset.temperature()),
                max_tokens: None,
            };

            self.log.append(
                &SessionEvent::new(
                        SessionEventType::BeforeCommand,
                        "ra",
                        &format!("agent loop iteration {}", iteration + 1),
                    )
                    .with_detail(&format!("messages: {}, tools: {}", messages.len(), tool_defs.len()))
            )?;

            let response = self.fetch_with_retry(request, &mut messages, &mut on_delta).await?;
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

                if !approval_req.auto_approvable(self.safety.mode()) {
                    if !self.safety.can_execute(&tool_name, &args) {
                        let block_msg =
                            format!("Tool {} blocked by safety policy.", tool_name);
                        messages.push(ChatMessage {
                            role: MessageRole::Tool,
                            content: block_msg.clone(),
                            tool_calls: None,
                            tool_call_id: Some(tool_id),
                        });
                        on_delta(&format!("\n[Anubis blocked {}]\n", tool_name));
                        continue;
                    }
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
                            if tr.success { out } else { format!("ERROR: {}", out) }
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
                on_delta(&format!("\n[Retry {}/{}]\n", attempt, max_retries),
                );
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
        "read_file" => Some("Read a file from the workspace. Optionally specify offset (line number) and limit (max lines).".to_string()),
        "list_files" => Some("List files in a directory within the workspace. Optionally specify max_depth.".to_string()),
        "search_text" => Some("Search for text pattern across files in the workspace. Supports glob filtering and case insensitivity.".to_string()),
        "write_file" => Some("Write content to a file in the workspace. Creates parent directories if needed.".to_string()),
        "edit_file" => Some("Edit a specific region of a file by replacing exact original text with new text.".to_string()),
        "run_shell" => Some("Run a shell command inside the workspace. Timeout defaults to 30 seconds.".to_string()),
        "git_status" => Some("Show git status for the workspace.".to_string()),
        "git_diff" => Some("Show git diff for the workspace.".to_string()),
        "git_checkpoint" => Some("Create a git checkpoint commit with a given message.".to_string()),
        "git_branch" => Some("Create and switch to a new git branch.".to_string()),
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
                "glob": { "type": "string", "description": "File extension glob, e.g. '*.rs'" },
                "case_insensitive": { "type": "boolean", "description": "Case-insensitive search" }
            },
            "required": ["pattern"]
        }),
        "write_file" => serde_json::json!({
            "type": "object",
            "properties": {
                "path": { "type": "string", "description": "Relative path to file" },
                "content": { "type": "string", "description": "File content to write" }
            },
            "required": ["path", "content"]
        }),
        "edit_file" => serde_json::json!({
            "type": "object",
            "properties": {
                "path": { "type": "string", "description": "Relative path to file" },
                "old_string": { "type": "string", "description": "Exact text to replace" },
                "new_string": { "type": "string", "description": "Replacement text" }
            },
            "required": ["path", "old_string", "new_string"]
        }),
        "run_shell" => serde_json::json!({
            "type": "object",
            "properties": {
                "command": { "type": "string", "description": "Shell command to execute" },
                "timeout": { "type": "integer", "description": "Timeout in seconds" }
            },
            "required": ["command"]
        }),
        "git_status" => serde_json::json!({
            "type": "object",
            "properties": {}
        }),
        "git_diff" => serde_json::json!({
            "type": "object",
            "properties": {
                "staged": { "type": "boolean", "description": "Show staged diff" },
                "path": { "type": "string", "description": "Path filter" }
            }
        }),
        "git_checkpoint" => serde_json::json!({
            "type": "object",
            "properties": {
                "message": { "type": "string", "description": "Commit message" }
            },
            "required": ["message"]
        }),
        "git_branch" => serde_json::json!({
            "type": "object",
            "properties": {
                "name": { "type": "string", "description": "Branch name to create and checkout" }
            },
            "required": ["name"]
        }),
        _ => Value::Null,
    }
}

fn classify_tool_risk(tool_name: &str, args: &Value, safety: &SafetyPolicy) -> CommandRisk {
    match tool_name {
        "read_file" | "list_files" | "search_text" | "git_status" | "git_diff" => {
            CommandRisk::Safe
        }
        "write_file" | "edit_file" => {
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
        "git_checkpoint" | "git_branch" => CommandRisk::Safe,
        _ => CommandRisk::Safe,
    }
}

fn map_action_type(tool_name: &str) -> ActionType {
    match tool_name {
        "write_file" | "edit_file" => ActionType::FileWrite,
        "run_shell" => ActionType::ShellCommand,
        "git_checkpoint" => ActionType::GitCommit,
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
    let re = Regex::new(r#"\{\s*"name"\s*:\s*"([^"]+)"\s*,\s*"arguments"\s*:\s*(\{[\s\S]*?\})\s*\}"#).ok()?;
    let mut calls = Vec::new();
    for caps in re.captures_iter(&cleaned) {
        let name = caps.get(1)?.as_str().to_string();
        let args_str = caps.get(2)?.as_str().to_string();
        // Validate it's actually JSON
        if serde_json::from_str::<Value>(&args_str).is_ok() {
            calls.push(crate::providers::CompleteToolCall {
                id: format!("salvage-{}", uuid::Uuid::new_v4().to_string()[..8].to_string()),
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
