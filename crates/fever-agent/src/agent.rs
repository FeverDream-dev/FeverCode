use crate::role::{RoleRegistry, SpecialistRole};
use crate::{LoopConfig, LoopDriver};
use fever_core::{
    Agent, AgentContext, AgentResponse, Message, PermissionGuard, Result, ToolCall, ToolResult,
    ToolResultData, redact_secrets,
};
use fever_providers::{ChatRequest, ChatResponse, ProviderClient, ToolDefinition};
use std::path::PathBuf;
use std::sync::Arc;
pub struct AgentConfig {
    pub default_model: String,
    pub default_temperature: f32,
    pub max_tokens: u32,
    pub stream: bool,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            default_model: "openai/gpt-4o".to_string(),
            default_temperature: 0.7,
            max_tokens: 4096,
            stream: false,
        }
    }
}

pub struct FeverAgent {
    provider: Arc<ProviderClient>,
    roles: RoleRegistry,
    config: AgentConfig,
    current_role: String,
    pub tools: Option<Arc<fever_core::ToolRegistry>>,
    permission_guard: Option<Arc<std::sync::RwLock<PermissionGuard>>>,
    workspace_root: PathBuf,
}

impl FeverAgent {
    pub fn new(provider: Arc<ProviderClient>, config: AgentConfig) -> Self {
        Self {
            provider,
            roles: RoleRegistry::new(),
            config,
            current_role: "default".to_string(),
            tools: None,
            permission_guard: None,
            workspace_root: std::env::current_dir().unwrap_or_default(),
        }
    }

    pub fn with_roles(mut self, roles: RoleRegistry) -> Self {
        self.roles = roles;
        self
    }

    pub fn with_tools(mut self, tools: Arc<fever_core::ToolRegistry>) -> Self {
        self.tools = Some(tools);
        self
    }

    pub fn with_permissions(mut self, guard: Arc<std::sync::RwLock<PermissionGuard>>) -> Self {
        self.permission_guard = Some(guard);
        self
    }

    pub fn default_model(&self) -> &str {
        &self.config.default_model
    }

    pub fn with_workspace_root(mut self, root: PathBuf) -> Self {
        self.workspace_root = root;
        self
    }

    // Iterative loop entry point: run the loop using the LoopDriver to orchestrate
    pub async fn run_loop(
        &self,
        messages: &[fever_core::Message],
        context: &fever_core::AgentContext,
    ) -> fever_core::Result<crate::LoopResult> {
        // Ensure tools exist
        if self.tools.is_none() {
            return Err(fever_core::Error::Agent("No tools registered".to_string()));
        }

        // Construct a LoopDriver borrowed from self
        let mut driver = LoopDriver::new(self, LoopConfig::default());
        driver.run(messages, context).await
    }

    pub fn set_role(&mut self, role_id: &str) -> Result<()> {
        if self.roles.get(role_id).is_none() {
            return Err(fever_core::Error::Agent(format!(
                "Role '{}' not found",
                role_id
            )));
        }
        self.current_role = role_id.to_string();
        Ok(())
    }

    pub fn get_current_role(&self) -> &SpecialistRole {
        self.roles
            .get(&self.current_role)
            .unwrap_or(self.roles.get("default").unwrap())
    }

    pub fn list_roles(&self) -> Vec<String> {
        self.roles.list_ids()
    }

    fn build_system_prompt(&self, user_context: &str) -> String {
        let role = self.get_current_role();
        let mut prompt = role.system_prompt.clone();

        if !user_context.is_empty() {
            prompt.push_str("\n\n");
            prompt.push_str("Context:\n");
            prompt.push_str(user_context);
        }

        let instructions = fever_core::discover_instructions(&self.workspace_root);
        if !instructions.is_empty() {
            prompt.push_str("\n\nProject Instructions:\n");
            prompt.push_str(&instructions);
        }

        prompt
    }

    pub async fn prepare_request(
        &self,
        messages: &[Message],
        context: &AgentContext,
    ) -> ChatRequest {
        let role = self.get_current_role();
        let system_content = self.build_system_prompt(&context.metadata.to_string());

        let mut chat_messages = vec![fever_providers::ChatMessage {
            role: "system".to_string(),
            content: system_content,
            tool_calls: None,
            tool_call_id: None,
        }];

        for msg in messages {
            chat_messages.push(fever_providers::ChatMessage {
                role: msg.role.clone(),
                content: msg.content.clone(),
                tool_calls: None,
                tool_call_id: None,
            });
        }

        // Convert ToolSchema to ToolDefinition for the provider
        let tool_definitions: Option<Vec<ToolDefinition>> = self.tools.as_ref().map(|tools| {
            tools
                .schemas()
                .into_iter()
                .map(|schema| ToolDefinition {
                    name: schema.name,
                    description: schema.description,
                    parameters: schema.parameters,
                })
                .collect()
        });

        ChatRequest {
            model: self.config.default_model.clone(),
            messages: chat_messages,
            tools: tool_definitions,
            temperature: Some(role.temperature.unwrap_or(self.config.default_temperature)),
            max_tokens: Some(self.config.max_tokens),
            stream: self.config.stream,
        }
    }
}

#[async_trait::async_trait]
impl Agent for FeverAgent {
    async fn chat(&self, messages: &[Message], context: &AgentContext) -> Result<AgentResponse> {
        let request = self.prepare_request(messages, context).await;

        let response: ChatResponse = self
            .provider
            .chat(&request)
            .await
            .map_err(|e| fever_core::Error::Provider(e.to_string()))?;

        let choice = response
            .choices
            .first()
            .ok_or_else(|| fever_core::Error::Agent("No response from provider".to_string()))?;

        let tool_calls = if let Some(calls) = &choice.message.tool_calls {
            calls
                .iter()
                .map(|c| ToolCall {
                    id: c.id.clone(),
                    name: c.name.clone(),
                    arguments: c.arguments.clone(),
                })
                .collect()
        } else {
            Vec::new()
        };

        Ok(AgentResponse {
            content: choice.message.content.clone(),
            tool_calls,
            finish_reason: Some(choice.finish_reason.clone()),
        })
    }

    async fn call_tools(
        &self,
        calls: &[ToolCall],
        context: &fever_core::ExecutionContext,
    ) -> Result<Vec<ToolResult>> {
        let tools = match &self.tools {
            Some(t) => t,
            None => return Ok(Vec::new()),
        };

        let mut results: Vec<ToolResult> = Vec::new();

        for call in calls {
            // Permission check before execution
            let permission_denied = self.check_tool_permission(call);
            if let Some((verdict, scope)) = permission_denied {
                results.push(ToolResult {
                    call_id: call.id.clone(),
                    result: ToolResultData::Error {
                        message: format!("Permission denied for {:?}: {}", scope, verdict.reason),
                    },
                    duration_ms: 0,
                });
                continue;
            }

            // Execute the tool
            let result = match tools.execute_call(call, context).await {
                Ok(mut result) => {
                    // Apply secret redaction to the output
                    Self::redact_tool_result(&mut result);
                    result
                }
                Err(e) => ToolResult {
                    call_id: call.id.clone(),
                    result: ToolResultData::Error {
                        message: e.to_string(),
                    },
                    duration_ms: 0,
                },
            };
            results.push(result);
        }

        Ok(results)
    }

    fn name(&self) -> &str {
        "Fever Agent"
    }
}

impl FeverAgent {
    /// Check if a tool call is permitted. Returns Some((verdict, scope)) if denied.
    fn check_tool_permission(
        &self,
        call: &ToolCall,
    ) -> Option<(fever_core::PermissionVerdict, fever_core::PermissionScope)> {
        let guard = match &self.permission_guard {
            Some(g) => g,
            None => return None, // No guard = allow all (for backward compatibility)
        };

        let guard = match guard.read() {
            Ok(g) => g,
            Err(_) => return None, // Poisoned lock = allow to avoid blocking
        };

        match call.name.as_str() {
            "shell" => {
                // Extract command from arguments
                if let Some(command) = call.arguments.get("command").and_then(|v| v.as_str()) {
                    let verdict = guard.check_command(command);
                    if !verdict.allowed {
                        return Some((verdict, fever_core::PermissionScope::ShellExec));
                    }
                }
            }
            "filesystem" => {
                // Extract path from arguments
                if let Some(path) = call.arguments.get("path").and_then(|v| v.as_str()) {
                    use std::path::Path;
                    let path = Path::new(path);
                    let verdict = guard.check_path(path);
                    if !verdict.allowed {
                        return Some((verdict, fever_core::PermissionScope::FilesystemRead));
                    }
                }
            }
            _ => {}
        }

        None
    }

    /// Apply secret redaction to a tool result's output
    fn redact_tool_result(result: &mut ToolResult) {
        match &mut result.result {
            ToolResultData::Success { output } => {
                if let serde_json::Value::String(s) = output {
                    let redacted = redact_secrets(s);
                    *output = serde_json::Value::String(redacted);
                } else if let serde_json::Value::Object(map) = output {
                    if let Some(serde_json::Value::String(s)) = map.get_mut("content") {
                        *s = redact_secrets(s);
                    }
                    if let Some(serde_json::Value::String(s)) = map.get_mut("stdout") {
                        *s = redact_secrets(s);
                    }
                    if let Some(serde_json::Value::String(s)) = map.get_mut("stderr") {
                        *s = redact_secrets(s);
                    }
                }
            }
            ToolResultData::Error { message } => {
                *message = redact_secrets(message);
            }
        }
    }
}
