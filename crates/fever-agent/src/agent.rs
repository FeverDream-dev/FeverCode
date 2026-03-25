use crate::role::{RoleRegistry, SpecialistRole};
use fever_core::{Agent, AgentContext, AgentResponse, Message, Result, ToolCall, ToolResult};
use fever_providers::{ChatRequest, ChatResponse, ModelInfo, ProviderClient};
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
}

impl FeverAgent {
    pub fn new(provider: Arc<ProviderClient>, config: AgentConfig) -> Self {
        Self {
            provider,
            roles: RoleRegistry::new(),
            config,
            current_role: "default".to_string(),
        }
    }

    pub fn with_roles(mut self, roles: RoleRegistry) -> Self {
        self.roles = roles;
        self
    }

    pub fn set_role(&mut self, role_id: &str) -> Result<()> {
        if self.roles.get(role_id).is_none() {
            return Err(fever_core::Error::Agent(format!("Role '{}' not found", role_id)).into());
        }
        self.current_role = role_id.to_string();
        Ok(())
    }

    pub fn get_current_role(&self) -> &SpecialistRole {
        self.roles.get(&self.current_role).unwrap_or(self.roles.get("default").unwrap())
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

        prompt
    }

    async fn prepare_request(
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

        ChatRequest {
            model: self.config.default_model.clone(),
            messages: chat_messages,
            tools: None,
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

    async fn call_tools(&self, calls: &[ToolCall], _context: &fever_core::ExecutionContext) -> Result<Vec<ToolResult>> {
        Ok(calls
            .iter()
            .map(|call| ToolResult {
                call_id: call.id.clone(),
                result: fever_core::ToolResultData::Error {
                    message: "Tool execution not implemented in agent".to_string(),
                },
                duration_ms: 0,
            })
            .collect())
    }

    fn name(&self) -> &str {
        "Fever Agent"
    }
}
