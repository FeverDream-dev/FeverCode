use anyhow::Result;
use futures::Stream;
use serde::{Deserialize, Serialize};
use std::pin::Pin;

pub mod external_cli;
pub mod openai_compat;
pub mod model_discovery;

pub trait Provider: Send + Sync {
    fn name(&self) -> &str;
    fn kind(&self) -> &str;
    fn chat_stream(
        &self,
        request: ChatRequest,
    ) -> Pin<Box<dyn Stream<Item = Result<ProviderEvent>> + Send>>;

    fn chat_with_tools(
        &self,
        request: ChatRequest,
    ) -> Pin<Box<dyn std::future::Future<Output = Result<AssistantResponse>> + Send + '_>>;

    fn list_models(
        &self,
    ) -> Pin<Box<dyn std::future::Future<Output = Result<Vec<String>>> + Send + '_>>;
}

#[derive(Debug, Clone)]
pub struct AssistantResponse {
    pub content: Option<String>,
    pub tool_calls: Vec<CompleteToolCall>,
    pub usage: ProviderUsage,
}

#[derive(Debug, Clone)]
pub struct CompleteToolCall {
    pub id: String,
    pub name: String,
    pub arguments: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatRequest {
    pub messages: Vec<ChatMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<ToolDef>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: MessageRole,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    System,
    User,
    Assistant,
    Tool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDef {
    #[serde(rename = "type")]
    pub tool_type: String,
    pub function: ToolFunction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolFunction {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    #[serde(rename = "type")]
    pub call_type: String,
    pub function: ToolCallFunction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallFunction {
    pub name: String,
    pub arguments: String,
}

#[derive(Debug, Clone)]
pub enum ProviderEvent {
    Delta(String),
    ToolCallBegin { id: String, name: String },
    ToolCallDelta { id: String, delta: String },
    Done(ProviderUsage),
    Error(String),
}

#[derive(Debug, Clone, Default)]
pub struct ProviderUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

#[derive(Debug, Clone)]
pub struct ProviderDescriptor {
    pub name: String,
    pub kind: String,
    pub base_url: Option<String>,
    pub api_key_env: Option<String>,
    pub command: Option<String>,
    pub models: Vec<String>,
}

pub fn descriptors(cfg: &crate::config::FeverConfig) -> Vec<ProviderDescriptor> {
    cfg.providers
        .available
        .iter()
        .map(|p| ProviderDescriptor {
            name: p.name.clone(),
            kind: p.kind.clone(),
            base_url: p.base_url.clone(),
            api_key_env: p.api_key_env.clone(),
            command: p.command.clone(),
            models: p.models.clone().unwrap_or_default(),
        })
        .collect()
}

pub fn build_provider(cfg: &crate::config::ProviderConfig) -> Result<Box<dyn Provider>> {
    match cfg.kind.as_str() {
        "openai_compatible" => {
            let base_url = cfg
                .base_url
                .clone()
                .unwrap_or_else(|| "http://localhost:11434/v1".to_string());
            let api_key = cfg
                .api_key_env
                .as_ref()
                .and_then(|env_var| std::env::var(env_var).ok());
            let model = cfg.model.clone().unwrap_or_else(|| "default".to_string());
            Ok(Box::new(openai_compat::OpenAiCompatProvider::new(
                cfg.name.clone(),
                base_url,
                api_key,
                model,
            )))
        }
        "external_cli" => {
            let command = cfg.command.clone().unwrap_or_else(|| "echo".to_string());
            Ok(Box::new(external_cli::ExternalCliProvider::new(
                cfg.name.clone(),
                command,
            )))
        }
        other => Err(anyhow::anyhow!("unknown provider kind: {}", other)),
    }
}

pub fn print_providers(cfg: &crate::config::FeverConfig) -> Result<()> {
    println!("Configured providers:");
    for p in descriptors(cfg) {
        let models = if p.models.is_empty() {
            "models not listed".to_string()
        } else {
            p.models.join(", ")
        };
        println!("- {} [{}] {}", p.name, p.kind, models);
        if let Some(base) = p.base_url {
            println!("  base_url: {}", base);
        }
        if let Some(env) = p.api_key_env {
            println!("  api_key_env: {}", env);
        }
        if let Some(command) = p.command {
            println!("  command: {}", command);
        }
    }
    Ok(())
}
