use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::sync::Arc;

use crate::adapter::ProviderAdapter;
use crate::adapters::anthropic::AnthropicAdapter;
use crate::adapters::gemini::GeminiAdapter;
use crate::adapters::ollama::OllamaAdapter;
use crate::adapters::openai::OpenAiAdapter;
use crate::error::ProviderError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AdapterType {
    OpenAi,
    Anthropic,
    Gemini,
    Ollama,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProviderTier {
    FirstClass,
    Compatible,
    Community,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderProfile {
    pub id: String,
    pub display_name: String,
    pub adapter_type: AdapterType,
    pub base_url: String,
    pub env_var: String,
    pub default_model: String,
    pub models: Vec<String>,
    pub supports_streaming: bool,
    pub supports_tools: bool,
    pub supports_vision: bool,
    pub requires_auth: bool,
    pub tier: ProviderTier,
}

impl ProviderProfile {
    /// Returns true if the provider's required env var is set (or no auth needed).
    pub fn is_configured(&self) -> bool {
        if !self.requires_auth {
            return true;
        }
        env::var(&self.env_var).is_ok()
    }

    /// Create a live adapter instance from this profile.
    pub fn create_adapter(&self) -> Result<Arc<dyn ProviderAdapter>, ProviderError> {
        if self.requires_auth && !self.is_configured() {
            return Err(ProviderError::Config(format!(
                "Provider '{}' requires env var {}",
                self.id, self.env_var
            )));
        }

        let api_key = if self.requires_auth {
            env::var(&self.env_var).unwrap_or_default()
        } else {
            String::new()
        };

        let adapter: Arc<dyn ProviderAdapter> = match self.adapter_type {
            AdapterType::OpenAi => Arc::new(OpenAiAdapter::custom(
                self.id.clone(),
                api_key,
                self.base_url.clone(),
            )),
            AdapterType::Anthropic => Arc::new(AnthropicAdapter::custom(
                &self.display_name,
                &api_key,
                &self.base_url,
            )),
            AdapterType::Gemini => Arc::new(GeminiAdapter::custom(
                &self.id,
                api_key.clone(),
                crate::adapters::gemini::GeminiConfig {
                    api_key,
                    base_url: self.base_url.clone(),
                    default_model: Some(self.default_model.clone()),
                },
            )),
            AdapterType::Ollama => {
                let url = env::var(&self.env_var).unwrap_or_else(|_| self.base_url.clone());
                Arc::new(OllamaAdapter::with_url(url))
            }
        };

        Ok(adapter)
    }
}

pub struct ProviderRegistry {
    profiles: HashMap<String, ProviderProfile>,
}

impl ProviderRegistry {
    pub fn new() -> Self {
        Self {
            profiles: HashMap::new(),
        }
    }

    /// Load all built-in provider profiles.
    pub fn builtin() -> Self {
        let mut reg = Self::new();
        for profile in builtin_profiles() {
            reg.profiles.insert(profile.id.clone(), profile);
        }
        reg
    }

    pub fn register(&mut self, profile: ProviderProfile) {
        self.profiles.insert(profile.id.clone(), profile);
    }

    pub fn get(&self, id: &str) -> Option<&ProviderProfile> {
        self.profiles.get(id)
    }

    /// List all profiles sorted by tier then display name.
    pub fn list(&self) -> Vec<&ProviderProfile> {
        let mut profiles: Vec<_> = self.profiles.values().collect();
        profiles.sort_by(|a, b| {
            let tier_order = |t: &ProviderTier| match t {
                ProviderTier::FirstClass => 0,
                ProviderTier::Compatible => 1,
                ProviderTier::Community => 2,
            };
            tier_order(&a.tier)
                .cmp(&tier_order(&b.tier))
                .then_with(|| a.display_name.cmp(&b.display_name))
        });
        profiles
    }

    /// List only providers whose env vars are set (or that need no auth).
    pub fn list_configured(&self) -> Vec<&ProviderProfile> {
        self.list()
            .into_iter()
            .filter(|p| p.is_configured())
            .collect()
    }

    pub fn list_by_tier(&self, tier: ProviderTier) -> Vec<&ProviderProfile> {
        self.list().into_iter().filter(|p| p.tier == tier).collect()
    }

    pub fn create_adapter(&self, id: &str) -> Result<Arc<dyn ProviderAdapter>, ProviderError> {
        self.get(id)
            .ok_or_else(|| ProviderError::ModelNotFound(id.to_string()))?
            .create_adapter()
    }

    pub fn create_configured_adapters(&self) -> Vec<(String, Arc<dyn ProviderAdapter>)> {
        self.list_configured()
            .into_iter()
            .filter_map(|p| p.create_adapter().ok().map(|a| (p.id.clone(), a)))
            .collect()
    }

    pub fn len(&self) -> usize {
        self.profiles.len()
    }

    pub fn is_empty(&self) -> bool {
        self.profiles.is_empty()
    }
}

impl Default for ProviderRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ── Helper to build OpenAI-compatible profiles in bulk ──

#[allow(clippy::too_many_arguments)]
fn openai_compatible(
    id: &str,
    display_name: &str,
    base_url: &str,
    env_var: &str,
    default_model: &str,
    models: Vec<&str>,
    tier: ProviderTier,
    supports_tools: bool,
) -> ProviderProfile {
    ProviderProfile {
        id: id.into(),
        display_name: display_name.into(),
        adapter_type: AdapterType::OpenAi,
        base_url: base_url.into(),
        env_var: env_var.into(),
        default_model: default_model.into(),
        models: models.into_iter().map(String::from).collect(),
        supports_streaming: true,
        supports_tools,
        supports_vision: false,
        requires_auth: !env_var.is_empty(),
        tier,
    }
}

fn builtin_profiles() -> Vec<ProviderProfile> {
    // ── First-Class Providers (10) ──
    let first_class = vec![
        ProviderProfile {
            id: "openai".into(),
            display_name: "OpenAI".into(),
            adapter_type: AdapterType::OpenAi,
            base_url: "https://api.openai.com/v1".into(),
            env_var: "OPENAI_API_KEY".into(),
            default_model: "gpt-4o".into(),
            models: vec![
                "gpt-4o".into(),
                "gpt-4o-mini".into(),
                "gpt-4.1".into(),
                "gpt-4.1-mini".into(),
                "gpt-4.1-nano".into(),
                "o3".into(),
                "o3-mini".into(),
                "o4-mini".into(),
            ],
            supports_streaming: true,
            supports_tools: true,
            supports_vision: true,
            requires_auth: true,
            tier: ProviderTier::FirstClass,
        },
        ProviderProfile {
            id: "anthropic".into(),
            display_name: "Anthropic Claude".into(),
            adapter_type: AdapterType::Anthropic,
            base_url: "https://api.anthropic.com".into(),
            env_var: "ANTHROPIC_API_KEY".into(),
            default_model: "claude-sonnet-4-20250514".into(),
            models: vec![
                "claude-sonnet-4-20250514".into(),
                "claude-opus-4-20250514".into(),
                "claude-3-5-sonnet-20241022".into(),
                "claude-3-5-haiku-20241022".into(),
            ],
            supports_streaming: true,
            supports_tools: true,
            supports_vision: true,
            requires_auth: true,
            tier: ProviderTier::FirstClass,
        },
        ProviderProfile {
            id: "gemini".into(),
            display_name: "Google Gemini".into(),
            adapter_type: AdapterType::Gemini,
            base_url: "https://generativelanguage.googleapis.com/v1beta/openai".into(),
            env_var: "GEMINI_API_KEY".into(),
            default_model: "gemini-2.5-flash-preview-05-20".into(),
            models: vec![
                "gemini-2.5-pro-preview-06-05".into(),
                "gemini-2.5-flash-preview-05-20".into(),
                "gemini-2.0-flash".into(),
                "gemini-2.0-flash-lite".into(),
            ],
            supports_streaming: true,
            supports_tools: true,
            supports_vision: true,
            requires_auth: true,
            tier: ProviderTier::FirstClass,
        },
        ProviderProfile {
            id: "ollama".into(),
            display_name: "Ollama (Local)".into(),
            adapter_type: AdapterType::Ollama,
            base_url: "http://localhost:11434".into(),
            env_var: "OLLAMA_BASE_URL".into(),
            default_model: "llama3.2".into(),
            models: vec![
                "llama3.2".into(),
                "llama3.1".into(),
                "llama3".into(),
                "mistral".into(),
                "codellama".into(),
                "phi3".into(),
                "gemma2".into(),
                "qwen2.5".into(),
                "deepseek-r1".into(),
            ],
            supports_streaming: true,
            supports_tools: true,
            supports_vision: false,
            requires_auth: false,
            tier: ProviderTier::FirstClass,
        },
        ProviderProfile {
            id: "zai".into(),
            display_name: "Z.ai (GLM)".into(),
            adapter_type: AdapterType::OpenAi,
            base_url: "https://api.z.ai/api/paas/v4".into(),
            env_var: "ZAI_API_KEY".into(),
            default_model: "glm-5-turbo".into(),
            models: vec![
                "glm-5-turbo".into(),
                "glm-5".into(),
                "glm-4.7".into(),
                "glm-4.7-flash".into(),
                "glm-4.6".into(),
                "glm-4.5".into(),
            ],
            supports_streaming: true,
            supports_tools: true,
            supports_vision: false,
            requires_auth: true,
            tier: ProviderTier::FirstClass,
        },
        ProviderProfile {
            id: "groq".into(),
            display_name: "Groq".into(),
            adapter_type: AdapterType::OpenAi,
            base_url: "https://api.groq.com/openai/v1".into(),
            env_var: "GROQ_API_KEY".into(),
            default_model: "llama-3.3-70b-versatile".into(),
            models: vec![
                "llama-3.3-70b-versatile".into(),
                "llama-3.1-8b-instant".into(),
                "mixtral-8x7b-32768".into(),
                "gemma2-9b-it".into(),
            ],
            supports_streaming: true,
            supports_tools: true,
            supports_vision: false,
            requires_auth: true,
            tier: ProviderTier::FirstClass,
        },
        ProviderProfile {
            id: "deepseek".into(),
            display_name: "DeepSeek".into(),
            adapter_type: AdapterType::OpenAi,
            base_url: "https://api.deepseek.com/v1".into(),
            env_var: "DEEPSEEK_API_KEY".into(),
            default_model: "deepseek-chat".into(),
            models: vec!["deepseek-chat".into(), "deepseek-reasoner".into()],
            supports_streaming: true,
            supports_tools: true,
            supports_vision: false,
            requires_auth: true,
            tier: ProviderTier::FirstClass,
        },
        ProviderProfile {
            id: "mistral".into(),
            display_name: "Mistral AI".into(),
            adapter_type: AdapterType::OpenAi,
            base_url: "https://api.mistral.ai/v1".into(),
            env_var: "MISTRAL_API_KEY".into(),
            default_model: "mistral-large-latest".into(),
            models: vec![
                "mistral-large-latest".into(),
                "mistral-medium-latest".into(),
                "codestral-latest".into(),
                "pixtral-large-latest".into(),
            ],
            supports_streaming: true,
            supports_tools: true,
            supports_vision: true,
            requires_auth: true,
            tier: ProviderTier::FirstClass,
        },
        ProviderProfile {
            id: "together".into(),
            display_name: "Together AI".into(),
            adapter_type: AdapterType::OpenAi,
            base_url: "https://api.together.xyz/v1".into(),
            env_var: "TOGETHER_API_KEY".into(),
            default_model: "meta-llama/Llama-3.3-70B-Instruct-Turbo".into(),
            models: vec![
                "meta-llama/Llama-3.3-70B-Instruct-Turbo".into(),
                "meta-llama/Llama-3.3-8B-Instruct-Turbo".into(),
                "mistralai/Mixtral-8x7B-Instruct-v0.1".into(),
            ],
            supports_streaming: true,
            supports_tools: true,
            supports_vision: false,
            requires_auth: true,
            tier: ProviderTier::FirstClass,
        },
        ProviderProfile {
            id: "openrouter".into(),
            display_name: "OpenRouter".into(),
            adapter_type: AdapterType::OpenAi,
            base_url: "https://openrouter.ai/api/v1".into(),
            env_var: "OPENROUTER_API_KEY".into(),
            default_model: "anthropic/claude-sonnet-4".into(),
            models: vec!["anthropic/claude-sonnet-4".into()],
            supports_streaming: true,
            supports_tools: true,
            supports_vision: false,
            requires_auth: true,
            tier: ProviderTier::FirstClass,
        },
    ];

    // ── Compatible Providers (OpenAI-compatible, 26) ──
    let compatible = vec![
        openai_compatible(
            "fireworks",
            "Fireworks AI",
            "https://api.fireworks.ai/inference/v1",
            "FIREWORKS_API_KEY",
            "accounts/fireworks/models/llama-v3p1-70b-instruct",
            vec!["accounts/fireworks/models/llama-v3p1-70b-instruct"],
            ProviderTier::Compatible,
            true,
        ),
        openai_compatible(
            "perplexity",
            "Perplexity",
            "https://api.perplexity.ai",
            "PERPLEXITY_API_KEY",
            "sonar-pro",
            vec!["sonar-pro", "sonar"],
            ProviderTier::Compatible,
            true,
        ),
        openai_compatible(
            "minimax",
            "MiniMax",
            "https://api.minimax.chat/v1",
            "MINIMAX_API_KEY",
            "MiniMax-Text-01",
            vec!["MiniMax-Text-01"],
            ProviderTier::Compatible,
            true,
        ),
        openai_compatible(
            "ai21",
            "AI21 Labs",
            "https://api.ai21.com/studio/v1",
            "AI21_API_KEY",
            "jamba-1.5-large",
            vec!["jamba-1.5-large"],
            ProviderTier::Compatible,
            true,
        ),
        openai_compatible(
            "cerebras",
            "Cerebras",
            "https://api.cerebras.ai/v1",
            "CEREBRAS_API_KEY",
            "llama-3.3-70b",
            vec!["llama-3.3-70b"],
            ProviderTier::Compatible,
            true,
        ),
        openai_compatible(
            "sambanova",
            "SambaNova",
            "https://api.sambanova.ai/v1",
            "SAMBANOVA_API_KEY",
            "Meta-Llama-3.3-70B-Instruct",
            vec!["Meta-Llama-3.3-70B-Instruct"],
            ProviderTier::Compatible,
            true,
        ),
        openai_compatible(
            "deepinfra",
            "DeepInfra",
            "https://api.deepinfra.com/v1/openai",
            "DEEPINFRA_API_KEY",
            "meta-llama/Meta-Llama-3.3-70B-Instruct",
            vec!["meta-llama/Meta-Llama-3.3-70B-Instruct"],
            ProviderTier::Compatible,
            true,
        ),
        openai_compatible(
            "novita",
            "Novita AI",
            "https://api.novita.ai/v3/openai",
            "NOVITA_API_KEY",
            "meta-llama-3.3-70b-instruct",
            vec!["meta-llama-3.3-70b-instruct"],
            ProviderTier::Compatible,
            true,
        ),
        openai_compatible(
            "moonshot",
            "Moonshot AI (Kimi)",
            "https://api.moonshot.cn/v1",
            "MOONSHOT_API_KEY",
            "moonshot-v1-128k",
            vec!["moonshot-v1-128k", "moonshot-v1-32k", "moonshot-v1-8k"],
            ProviderTier::Compatible,
            true,
        ),
        openai_compatible(
            "01ai",
            "01.AI (Yi)",
            "https://api.lingyiwanwu.com/v1",
            "YI_API_KEY",
            "yi-lightning",
            vec!["yi-lightning", "yi-large"],
            ProviderTier::Compatible,
            true,
        ),
        openai_compatible(
            "baichuan",
            "Baichuan AI",
            "https://api.baichuan-ai.com/v1",
            "BAICHUAN_API_KEY",
            "Baichuan4",
            vec!["Baichuan4"],
            ProviderTier::Compatible,
            true,
        ),
        openai_compatible(
            "siliconflow",
            "SiliconFlow",
            "https://api.siliconflow.cn/v1",
            "SILICONFLOW_API_KEY",
            "Qwen/Qwen2.5-72B-Instruct",
            vec!["Qwen/Qwen2.5-72B-Instruct", "deepseek-ai/DeepSeek-V3"],
            ProviderTier::Compatible,
            true,
        ),
        openai_compatible(
            "stepfun",
            "StepFun (Step-2)",
            "https://api.stepfun.com/v1",
            "STEPFUN_API_KEY",
            "step-2-16k",
            vec!["step-2-16k"],
            ProviderTier::Compatible,
            true,
        ),
        openai_compatible(
            "chutes",
            "Chutes AI",
            "https://chutes.ai/v1",
            "CHUTES_API_KEY",
            "meta-llama/Meta-Llama-3.3-70B-Instruct",
            vec!["meta-llama/Meta-Llama-3.3-70B-Instruct"],
            ProviderTier::Compatible,
            true,
        ),
        openai_compatible(
            "groqcompat",
            "Groq (OpenAI-compat)",
            "https://api.groq.com/openai/v1",
            "GROQ_API_KEY",
            "llama-3.3-70b-versatile",
            vec!["llama-3.3-70b-versatile"],
            ProviderTier::Compatible,
            true,
        ),
        openai_compatible(
            "anyscale",
            "Anyscale",
            "https://api.endpoints.anyscale.com/v1",
            "ANYSCALE_API_KEY",
            "meta-llama/Llama-2-70b-chat-fine-tuned",
            vec!["meta-llama/Llama-2-70b-chat-fine-tuned"],
            ProviderTier::Compatible,
            true,
        ),
        openai_compatible(
            "octoai",
            "OctoAI",
            "https://text.octoai.run/v1",
            "OCTOAI_API_KEY",
            "meta-llama-3-8b-instruct",
            vec!["meta-llama-3-8b-instruct"],
            ProviderTier::Compatible,
            true,
        ),
        openai_compatible(
            "lepton",
            "Lepton AI",
            "https://llama2-7b.lepton.run/api/v1",
            "LEPTON_API_KEY",
            "llama2-7b",
            vec!["llama2-7b"],
            ProviderTier::Compatible,
            true,
        ),
        openai_compatible(
            "replicate",
            "Replicate",
            "https://api.replicate.com/v1",
            "REPLICATE_API_TOKEN",
            "meta/llama-2-70b-chat",
            vec!["meta/llama-2-70b-chat"],
            ProviderTier::Compatible,
            true,
        ),
        openai_compatible(
            "writer",
            "Writer",
            "https://api.writer.com/v1",
            "WRITER_API_KEY",
            "palmyra-x-003",
            vec!["palmyra-x-003"],
            ProviderTier::Compatible,
            true,
        ),
        openai_compatible(
            "segmind",
            "Segmind",
            "https://api.segmind.com/v1",
            "SEGMIND_API_KEY",
            "segmind/llama-3.3-70b-instruct",
            vec!["segmind/llama-3.3-70b-instruct"],
            ProviderTier::Compatible,
            true,
        ),
        openai_compatible(
            "volcengine",
            "Volcengine (Doubao)",
            "https://ark.cn-beijing.volces.com/api/v3",
            "VOLCENGINE_API_KEY",
            "doubao-pro-32k",
            vec!["doubao-pro-32k"],
            ProviderTier::Compatible,
            true,
        ),
        openai_compatible(
            "zhipuai",
            "Zhipu AI (ChatGLM)",
            "https://open.bigmodel.cn/api/paas/v4",
            "ZHIPU_API_KEY",
            "glm-4-plus",
            vec!["glm-4-plus", "glm-4-flash", "glm-4-long"],
            ProviderTier::Compatible,
            true,
        ),
        openai_compatible(
            "qwen",
            "Qwen (Alibaba Cloud)",
            "https://dashscope.aliyuncs.com/compatible-mode/v1",
            "DASHSCOPE_API_KEY",
            "qwen-plus",
            vec!["qwen-plus", "qwen-turbo", "qwen-max", "qwen-long"],
            ProviderTier::Compatible,
            true,
        ),
        openai_compatible(
            "xai",
            "xAI (Grok)",
            "https://api.x.ai/v1",
            "XAI_API_KEY",
            "grok-3",
            vec!["grok-3", "grok-3-mini"],
            ProviderTier::Compatible,
            true,
        ),
        openai_compatible(
            "cloudflare",
            "Cloudflare Workers AI",
            "https://api.cloudflare.com/client/v4/accounts/{ACCOUNT_ID}/ai/v1",
            "CLOUDFLARE_API_TOKEN",
            "@cf/meta/llama-3.3-70b-instruct",
            vec!["@cf/meta/llama-3.3-70b-instruct"],
            ProviderTier::Compatible,
            false,
        ),
    ];

    // ── Community / Local Providers (14) ──
    let community = vec![
        openai_compatible(
            "lmstudio",
            "LM Studio",
            "http://localhost:1234/v1",
            "LMSTUDIO_API_KEY",
            "default",
            vec!["default"],
            ProviderTier::Community,
            false,
        ),
        openai_compatible(
            "koboldcpp",
            "KoboldCpp",
            "http://localhost:5001/v1",
            "KOBOLDCPP_API_KEY",
            "default",
            vec!["default"],
            ProviderTier::Community,
            false,
        ),
        openai_compatible(
            "tabbyapi",
            "TabbyAPI",
            "http://localhost:5000/v1",
            "TABBY_API_KEY",
            "default",
            vec!["default"],
            ProviderTier::Community,
            false,
        ),
        openai_compatible(
            "localai",
            "LocalAI",
            "http://localhost:8080/v1",
            "LOCALAI_API_KEY",
            "default",
            vec!["default"],
            ProviderTier::Community,
            false,
        ),
        openai_compatible(
            "vllm",
            "vLLM",
            "http://localhost:8000/v1",
            "VLLM_API_KEY",
            "default",
            vec!["default"],
            ProviderTier::Community,
            false,
        ),
        openai_compatible(
            "ollama-compat",
            "Ollama (OpenAI-compat)",
            "http://localhost:11434/v1",
            "",
            "llama3.2",
            vec!["llama3.2"],
            ProviderTier::Community,
            false,
        ),
        openai_compatible(
            "textgen",
            "Text Generation WebUI",
            "http://localhost:5000/v1",
            "",
            "default",
            vec!["default"],
            ProviderTier::Community,
            false,
        ),
        openai_compatible(
            "aphrodite",
            "Aphrodite Engine",
            "http://localhost:2242/v1",
            "",
            "default",
            vec!["default"],
            ProviderTier::Community,
            false,
        ),
        openai_compatible(
            "llamacpp",
            "llama.cpp Server",
            "http://localhost:8080/v1",
            "",
            "default",
            vec!["default"],
            ProviderTier::Community,
            false,
        ),
        openai_compatible(
            "exllamav2",
            "ExLlamaV2",
            "http://localhost:8001/v1",
            "",
            "default",
            vec!["default"],
            ProviderTier::Community,
            false,
        ),
        openai_compatible(
            "tensorrt-llm",
            "TensorRT-LLM",
            "http://localhost:8000/v1",
            "",
            "default",
            vec!["default"],
            ProviderTier::Community,
            false,
        ),
        openai_compatible(
            "ctransformers",
            "ctransformers",
            "http://localhost:8002/v1",
            "",
            "default",
            vec!["default"],
            ProviderTier::Community,
            false,
        ),
        openai_compatible(
            "ollama-remote",
            "Ollama (Remote)",
            "http://192.168.1.100:11434",
            "",
            "llama3.2",
            vec!["llama3.2"],
            ProviderTier::Community,
            false,
        ),
        openai_compatible(
            "custom-openai",
            "Custom OpenAI-compat",
            "http://localhost:8000/v1",
            "CUSTOM_API_KEY",
            "default",
            vec!["default"],
            ProviderTier::Community,
            false,
        ),
    ];

    let mut all = Vec::with_capacity(first_class.len() + compatible.len() + community.len());
    all.extend(first_class);
    all.extend(compatible);
    all.extend(community);
    all
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builtin_registry_has_50_plus_profiles() {
        let reg = ProviderRegistry::builtin();
        assert!(reg.len() >= 50, "Expected 50+ profiles, got {}", reg.len());
    }

    #[test]
    fn test_first_class_providers() {
        let reg = ProviderRegistry::builtin();
        let first_class = reg.list_by_tier(ProviderTier::FirstClass);
        let ids: Vec<&str> = first_class.iter().map(|p| p.id.as_str()).collect();
        assert!(ids.contains(&"openai"));
        assert!(ids.contains(&"anthropic"));
        assert!(ids.contains(&"gemini"));
        assert!(ids.contains(&"ollama"));
        assert!(ids.contains(&"zai"));
        assert!(ids.contains(&"groq"));
        assert!(ids.contains(&"deepseek"));
        assert!(ids.contains(&"mistral"));
        assert!(ids.contains(&"together"));
        assert!(ids.contains(&"openrouter"));
        assert_eq!(
            first_class.len(),
            10,
            "Expected exactly 10 first-class providers, got {}",
            first_class.len()
        );
    }

    #[test]
    fn test_compatible_providers() {
        let reg = ProviderRegistry::builtin();
        let compatible = reg.list_by_tier(ProviderTier::Compatible);
        assert!(
            compatible.len() >= 20,
            "Expected 20+ compatible providers, got {}",
            compatible.len()
        );
    }

    #[test]
    fn test_community_providers() {
        let reg = ProviderRegistry::builtin();
        let community = reg.list_by_tier(ProviderTier::Community);
        assert!(
            community.len() >= 10,
            "Expected 10+ community providers, got {}",
            community.len()
        );
    }

    #[test]
    fn test_profile_is_configured() {
        let p = ProviderProfile {
            id: "test-auth".into(),
            display_name: "Test".into(),
            adapter_type: AdapterType::OpenAi,
            base_url: "https://example.com/v1".into(),
            env_var: "NONEXISTENT_TEST_VAR_XYZ".into(),
            default_model: "test".into(),
            models: vec![],
            supports_streaming: true,
            supports_tools: true,
            supports_vision: false,
            requires_auth: true,
            tier: ProviderTier::Compatible,
        };
        assert!(!p.is_configured());

        let p_no_auth = ProviderProfile {
            id: "test-noauth".into(),
            display_name: "Test".into(),
            adapter_type: AdapterType::Ollama,
            base_url: "http://localhost:11434".into(),
            env_var: "UNUSED".into(),
            default_model: "test".into(),
            models: vec![],
            supports_streaming: true,
            supports_tools: true,
            supports_vision: false,
            requires_auth: false,
            tier: ProviderTier::Community,
        };
        assert!(p_no_auth.is_configured());
    }

    #[test]
    fn test_list_sorts_by_tier() {
        let reg = ProviderRegistry::builtin();
        let list = reg.list();
        let mut prev_tier = 0u8;
        for p in &list {
            let tier_num = match p.tier {
                ProviderTier::FirstClass => 0,
                ProviderTier::Compatible => 1,
                ProviderTier::Community => 2,
            };
            assert!(tier_num >= prev_tier, "Profiles not sorted by tier");
            prev_tier = tier_num;
        }
    }

    #[test]
    fn test_list_configured_filters() {
        let reg = ProviderRegistry::builtin();
        let configured = reg.list_configured();
        let all = reg.list();
        assert!(configured.len() <= all.len());
    }

    #[test]
    fn test_zai_profile() {
        let reg = ProviderRegistry::builtin();
        let zai = reg.get("zai").expect("zai profile should exist");
        assert_eq!(zai.base_url, "https://api.z.ai/api/paas/v4");
        assert_eq!(zai.env_var, "ZAI_API_KEY");
        assert!(zai.models.contains(&"glm-5-turbo".to_string()));
        assert!(zai.supports_tools);
    }

    #[test]
    fn test_ollama_no_auth_required() {
        let reg = ProviderRegistry::builtin();
        let ollama = reg.get("ollama").expect("ollama profile should exist");
        assert!(!ollama.requires_auth);
        assert_eq!(ollama.adapter_type, AdapterType::Ollama);
    }

    #[test]
    fn test_get_nonexistent_returns_none() {
        let reg = ProviderRegistry::builtin();
        assert!(reg.get("nonexistent_provider_xyz").is_none());
    }

    #[test]
    fn test_register_custom_profile() {
        let mut reg = ProviderRegistry::builtin();
        let count_before = reg.len();
        reg.register(ProviderProfile {
            id: "my-custom".into(),
            display_name: "My Custom".into(),
            adapter_type: AdapterType::OpenAi,
            base_url: "https://my-api.example.com/v1".into(),
            env_var: "MY_CUSTOM_KEY".into(),
            default_model: "custom-model".into(),
            models: vec!["custom-model".into()],
            supports_streaming: true,
            supports_tools: false,
            supports_vision: false,
            requires_auth: true,
            tier: ProviderTier::Community,
        });
        assert_eq!(reg.len(), count_before + 1);
        assert!(reg.get("my-custom").is_some());
    }

    #[test]
    fn test_is_empty() {
        let reg = ProviderRegistry::new();
        assert!(reg.is_empty());
        assert_eq!(reg.len(), 0);

        let builtin = ProviderRegistry::builtin();
        assert!(!builtin.is_empty());
        assert!(!builtin.is_empty());
    }
}
