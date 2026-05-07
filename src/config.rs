use crate::safety::ApprovalMode;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::{fs, path::Path};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeverConfig {
    pub workspace: WorkspaceConfig,
    pub ui: UiConfig,
    pub safety: SafetyConfig,
    pub providers: ProvidersConfig,
    pub agents: AgentsConfig,
    pub mcp: McpConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceConfig {
    pub root_policy: String,
    pub create_state_dir: bool,
    pub state_dir: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    pub theme: String,
    pub show_diff_by_default: bool,
    pub show_token_budget: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyConfig {
    pub mode: ApprovalMode,
    pub allow_writes_inside_workspace: bool,
    pub allow_writes_outside_workspace: bool,
    pub allow_shell: bool,
    pub allow_network: bool,
    pub allow_git_commit: bool,
    pub allow_package_install: bool,
    pub max_endless_iterations: u32,
    pub checkpoint_every_iterations: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvidersConfig {
    pub default: ProviderConfig,
    pub available: Vec<ProviderConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub name: String,
    pub kind: String,
    pub base_url: Option<String>,
    pub api_key_env: Option<String>,
    pub command: Option<String>,
    pub model: Option<String>,
    pub models: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentsConfig {
    pub enabled: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpConfig {
    pub config_file: String,
}

impl FeverConfig {
    pub fn load_or_default(root: &Path) -> Result<Self> {
        let path = root.join(".fevercode/config.toml");
        if path.exists() {
            let raw =
                fs::read_to_string(&path).with_context(|| format!("reading {}", path.display()))?;
            let cfg =
                toml::from_str(&raw).with_context(|| format!("parsing {}", path.display()))?;
            Ok(cfg)
        } else {
            Ok(Self::default())
        }
    }

    pub fn default_provider(&self) -> &ProviderConfig {
        &self.providers.default
    }

    pub fn find_provider(&self, name: &str) -> Option<&ProviderConfig> {
        self.providers.available.iter().find(|p| p.name == name)
    }

    pub fn detect_test_commands(&self, root: &Path) -> Vec<String> {
        let mut commands = Vec::new();

        if root.join("Cargo.toml").exists() {
            commands.push("cargo test".to_string());
            commands.push("cargo clippy".to_string());
            commands.push("cargo fmt --check".to_string());
        }

        if root.join("package.json").exists() {
            commands.push("npm test".to_string());
        }

        if root.join("go.mod").exists() {
            commands.push("go test ./...".to_string());
        }

        if root.join("Makefile").exists() {
            commands.push("make test".to_string());
        }

        if root.join("pyproject.toml").exists() || root.join("setup.py").exists() {
            commands.push("python -m pytest".to_string());
        }

        commands
    }
}

impl Default for FeverConfig {
    fn default() -> Self {
        Self {
            workspace: WorkspaceConfig {
                root_policy: "launch_directory".into(),
                create_state_dir: true,
                state_dir: ".fevercode".into(),
            },
            ui: UiConfig {
                theme: "egyptian_portal".into(),
                show_diff_by_default: true,
                show_token_budget: true,
            },
            safety: SafetyConfig {
                mode: ApprovalMode::Ask,
                allow_writes_inside_workspace: true,
                allow_writes_outside_workspace: false,
                allow_shell: true,
                allow_network: false,
                allow_git_commit: false,
                allow_package_install: false,
                max_endless_iterations: 25,
                checkpoint_every_iterations: 3,
            },
            providers: ProvidersConfig {
                default: ProviderConfig {
                    name: "zai".into(),
                    kind: "openai_compatible".into(),
                    base_url: Some("https://api.z.ai/api/paas/v4".into()),
                    api_key_env: Some("ZAI_API_KEY".into()),
                    command: None,
                    model: Some("glm-coding-plan-default".into()),
                    models: None,
                },
                available: vec![
                    // --- Major Cloud ---
                    ProviderConfig {
                        name: "zai".into(),
                        kind: "openai_compatible".into(),
                        base_url: Some("https://api.z.ai/api/paas/v4".into()),
                        api_key_env: Some("ZAI_API_KEY".into()),
                        command: None,
                        model: None,
                        models: Some(vec!["glm-5.1".into(), "glm-4.6".into()]),
                    },
                    ProviderConfig {
                        name: "openai".into(),
                        kind: "openai_compatible".into(),
                        base_url: Some("https://api.openai.com/v1".into()),
                        api_key_env: Some("OPENAI_API_KEY".into()),
                        command: None,
                        model: None,
                        models: Some(vec![
                            "gpt-5.5-codex".into(),
                            "gpt-5.5".into(),
                            "gpt-4o".into(),
                        ]),
                    },
                    ProviderConfig {
                        name: "anthropic".into(),
                        kind: "openai_compatible".into(),
                        base_url: Some("https://api.anthropic.com/v1".into()),
                        api_key_env: Some("ANTHROPIC_API_KEY".into()),
                        command: None,
                        model: None,
                        models: Some(vec!["claude-3-7-sonnet".into(), "claude-3-5-haiku".into()]),
                    },
                    ProviderConfig {
                        name: "google-gemini".into(),
                        kind: "openai_compatible".into(),
                        base_url: Some(
                            "https://generativelanguage.googleapis.com/v1beta/openai".into(),
                        ),
                        api_key_env: Some("GOOGLE_API_KEY".into()),
                        command: None,
                        model: None,
                        models: Some(vec!["gemini-2.5-pro".into(), "gemini-2.0-flash".into()]),
                    },
                    ProviderConfig {
                        name: "azure-openai".into(),
                        kind: "openai_compatible".into(),
                        base_url: Some(
                            "https://your-resource.openai.azure.com/openai/deployments".into(),
                        ),
                        api_key_env: Some("AZURE_OPENAI_KEY".into()),
                        command: None,
                        model: None,
                        models: Some(vec!["gpt-4o".into(), "gpt-4".into()]),
                    },
                    ProviderConfig {
                        name: "aws-bedrock".into(),
                        kind: "external_cli".into(),
                        base_url: None,
                        api_key_env: None,
                        command: Some("aws bedrock-runtime invoke-model".into()),
                        model: None,
                        models: Some(vec![
                            "anthropic.claude-3-5-sonnet".into(),
                            "amazon.nova-pro".into(),
                        ]),
                    },
                    ProviderConfig {
                        name: "cohere".into(),
                        kind: "openai_compatible".into(),
                        base_url: Some("https://api.cohere.com/v1".into()),
                        api_key_env: Some("COHERE_API_KEY".into()),
                        command: None,
                        model: None,
                        models: Some(vec!["command-r-plus".into(), "command-r".into()]),
                    },
                    ProviderConfig {
                        name: "mistral".into(),
                        kind: "openai_compatible".into(),
                        base_url: Some("https://api.mistral.ai/v1".into()),
                        api_key_env: Some("MISTRAL_API_KEY".into()),
                        command: None,
                        model: None,
                        models: Some(vec![
                            "mistral-large-latest".into(),
                            "codestral-latest".into(),
                        ]),
                    },
                    ProviderConfig {
                        name: "perplexity".into(),
                        kind: "openai_compatible".into(),
                        base_url: Some("https://api.perplexity.ai".into()),
                        api_key_env: Some("PERPLEXITY_API_KEY".into()),
                        command: None,
                        model: None,
                        models: Some(vec!["sonar-pro".into(), "sonar".into()]),
                    },
                    ProviderConfig {
                        name: "groq".into(),
                        kind: "openai_compatible".into(),
                        base_url: Some("https://api.groq.com/openai/v1".into()),
                        api_key_env: Some("GROQ_API_KEY".into()),
                        command: None,
                        model: None,
                        models: Some(vec![
                            "llama-3.3-70b-versatile".into(),
                            "mixtral-8x7b".into(),
                        ]),
                    },
                    ProviderConfig {
                        name: "together".into(),
                        kind: "openai_compatible".into(),
                        base_url: Some("https://api.together.xyz/v1".into()),
                        api_key_env: Some("TOGETHER_API_KEY".into()),
                        command: None,
                        model: None,
                        models: Some(vec![
                            "meta-llama/Llama-3.3-70B".into(),
                            "Qwen/Qwen2.5-72B".into(),
                        ]),
                    },
                    ProviderConfig {
                        name: "ai21".into(),
                        kind: "openai_compatible".into(),
                        base_url: Some("https://api.ai21.com/studio/v1".into()),
                        api_key_env: Some("AI21_API_KEY".into()),
                        command: None,
                        model: None,
                        models: Some(vec!["jamba-1.5-large".into(), "jamba-1.5-mini".into()]),
                    },
                    ProviderConfig {
                        name: "fireworks".into(),
                        kind: "openai_compatible".into(),
                        base_url: Some("https://api.fireworks.ai/inference/v1".into()),
                        api_key_env: Some("FIREWORKS_API_KEY".into()),
                        command: None,
                        model: None,
                        models: Some(vec![
                            "accounts/fireworks/models/llama-v3p3-70b-instruct".into()
                        ]),
                    },
                    ProviderConfig {
                        name: "xai".into(),
                        kind: "openai_compatible".into(),
                        base_url: Some("https://api.x.ai/v1".into()),
                        api_key_env: Some("XAI_API_KEY".into()),
                        command: None,
                        model: None,
                        models: Some(vec!["grok-2".into(), "grok-2-mini".into()]),
                    },
                    // --- Local / Self-Hosted ---
                    ProviderConfig {
                        name: "ollama-local".into(),
                        kind: "openai_compatible".into(),
                        base_url: Some("http://localhost:11434/v1".into()),
                        api_key_env: None,
                        command: None,
                        model: None,
                        models: Some(vec![
                            "qwen2.5-coder".into(),
                            "deepseek-coder".into(),
                            "llama3.2".into(),
                            "phi4".into(),
                        ]),
                    },
                    ProviderConfig {
                        name: "lm-studio".into(),
                        kind: "openai_compatible".into(),
                        base_url: Some("http://localhost:1234/v1".into()),
                        api_key_env: None,
                        command: None,
                        model: None,
                        models: Some(vec!["local-model".into()]),
                    },
                    ProviderConfig {
                        name: "localai".into(),
                        kind: "openai_compatible".into(),
                        base_url: Some("http://localhost:8080/v1".into()),
                        api_key_env: None,
                        command: None,
                        model: None,
                        models: Some(vec!["gpt-4".into(), "llama-3".into()]),
                    },
                    ProviderConfig {
                        name: "text-generation-webui".into(),
                        kind: "openai_compatible".into(),
                        base_url: Some("http://localhost:5000/v1".into()),
                        api_key_env: None,
                        command: None,
                        model: None,
                        models: Some(vec!["local-model".into()]),
                    },
                    ProviderConfig {
                        name: "llama-cpp-server".into(),
                        kind: "openai_compatible".into(),
                        base_url: Some("http://localhost:8080/v1".into()),
                        api_key_env: None,
                        command: None,
                        model: None,
                        models: Some(vec!["local-model".into()]),
                    },
                    ProviderConfig {
                        name: "tabbyapi".into(),
                        kind: "openai_compatible".into(),
                        base_url: Some("http://localhost:5000/v1".into()),
                        api_key_env: Some("TABBY_API_KEY".into()),
                        command: None,
                        model: None,
                        models: Some(vec!["local-model".into()]),
                    },
                    ProviderConfig {
                        name: "koboldcpp".into(),
                        kind: "openai_compatible".into(),
                        base_url: Some("http://localhost:5001/api/v1".into()),
                        api_key_env: None,
                        command: None,
                        model: None,
                        models: Some(vec!["local-model".into()]),
                    },
                    ProviderConfig {
                        name: "vllm".into(),
                        kind: "openai_compatible".into(),
                        base_url: Some("http://localhost:8000/v1".into()),
                        api_key_env: None,
                        command: None,
                        model: None,
                        models: Some(vec!["local-model".into()]),
                    },
                    // --- Chinese Providers ---
                    ProviderConfig {
                        name: "deepseek".into(),
                        kind: "openai_compatible".into(),
                        base_url: Some("https://api.deepseek.com/v1".into()),
                        api_key_env: Some("DEEPSEEK_API_KEY".into()),
                        command: None,
                        model: None,
                        models: Some(vec!["deepseek-chat".into(), "deepseek-reasoner".into()]),
                    },
                    ProviderConfig {
                        name: "moonshot".into(),
                        kind: "openai_compatible".into(),
                        base_url: Some("https://api.moonshot.cn/v1".into()),
                        api_key_env: Some("MOONSHOT_API_KEY".into()),
                        command: None,
                        model: None,
                        models: Some(vec!["moonshot-v1-128k".into(), "moonshot-v1-32k".into()]),
                    },
                    ProviderConfig {
                        name: "qwen-alibaba".into(),
                        kind: "openai_compatible".into(),
                        base_url: Some("https://dashscope.aliyuncs.com/compatible-mode/v1".into()),
                        api_key_env: Some("DASHSCOPE_API_KEY".into()),
                        command: None,
                        model: None,
                        models: Some(vec![
                            "qwen2.5-72b-instruct".into(),
                            "qwen-coder-plus".into(),
                        ]),
                    },
                    ProviderConfig {
                        name: "baichuan".into(),
                        kind: "openai_compatible".into(),
                        base_url: Some("https://api.baichuan-ai.com/v1".into()),
                        api_key_env: Some("BAICHUAN_API_KEY".into()),
                        command: None,
                        model: None,
                        models: Some(vec!["Baichuan4".into(), "Baichuan3-Turbo".into()]),
                    },
                    ProviderConfig {
                        name: "zhipu".into(),
                        kind: "openai_compatible".into(),
                        base_url: Some("https://open.bigmodel.cn/api/paas/v4".into()),
                        api_key_env: Some("ZHIPU_API_KEY".into()),
                        command: None,
                        model: None,
                        models: Some(vec!["glm-4-plus".into(), "glm-4-flash".into()]),
                    },
                    ProviderConfig {
                        name: "yi".into(),
                        kind: "openai_compatible".into(),
                        base_url: Some("https://api.01.ai/v1".into()),
                        api_key_env: Some("YI_API_KEY".into()),
                        command: None,
                        model: None,
                        models: Some(vec!["yi-large".into(), "yi-medium".into()]),
                    },
                    ProviderConfig {
                        name: "minimax".into(),
                        kind: "openai_compatible".into(),
                        base_url: Some("https://api.minimax.chat/v1".into()),
                        api_key_env: Some("MINIMAX_API_KEY".into()),
                        command: None,
                        model: None,
                        models: Some(vec!["abab6.5".into(), "abab6".into()]),
                    },
                    ProviderConfig {
                        name: "sparkdesk".into(),
                        kind: "openai_compatible".into(),
                        base_url: Some("https://spark-api-open.xf-yun.com/v1".into()),
                        api_key_env: Some("SPARKDESK_API_KEY".into()),
                        command: None,
                        model: None,
                        models: Some(vec!["generalv4".into()]),
                    },
                    ProviderConfig {
                        name: "hunyuan".into(),
                        kind: "openai_compatible".into(),
                        base_url: Some("https://hunyuan.tencentcloudapi.com".into()),
                        api_key_env: Some("TENCENT_SECRET_ID".into()),
                        command: None,
                        model: None,
                        models: Some(vec!["hunyuan-pro".into(), "hunyuan-standard".into()]),
                    },
                    ProviderConfig {
                        name: "sensetime".into(),
                        kind: "openai_compatible".into(),
                        base_url: Some("https://api.sensenova.cn/v1".into()),
                        api_key_env: Some("SENSETIME_API_KEY".into()),
                        command: None,
                        model: None,
                        models: Some(vec!["SenseChat-5".into()]),
                    },
                    ProviderConfig {
                        name: "bytedance-doubao".into(),
                        kind: "openai_compatible".into(),
                        base_url: Some("https://ark.cn-beijing.volces.com/api/v3".into()),
                        api_key_env: Some("ARK_API_KEY".into()),
                        command: None,
                        model: None,
                        models: Some(vec!["doubao-pro-32k".into(), "doubao-lite-32k".into()]),
                    },
                    // --- Open Router / Aggregators ---
                    ProviderConfig {
                        name: "openrouter".into(),
                        kind: "openai_compatible".into(),
                        base_url: Some("https://openrouter.ai/api/v1".into()),
                        api_key_env: Some("OPENROUTER_API_KEY".into()),
                        command: None,
                        model: None,
                        models: Some(vec![
                            "openai/gpt-4o".into(),
                            "anthropic/claude-3.5-sonnet".into(),
                            "meta-llama/llama-3.3-70b".into(),
                        ]),
                    },
                    ProviderConfig {
                        name: "venice".into(),
                        kind: "openai_compatible".into(),
                        base_url: Some("https://api.venice.ai/api/v1".into()),
                        api_key_env: Some("VENICE_API_KEY".into()),
                        command: None,
                        model: None,
                        models: Some(vec!["default".into()]),
                    },
                    ProviderConfig {
                        name: "openpipe".into(),
                        kind: "openai_compatible".into(),
                        base_url: Some("https://api.openpipe.ai/api/v1".into()),
                        api_key_env: Some("OPENPIPE_API_KEY".into()),
                        command: None,
                        model: None,
                        models: Some(vec!["local-model".into()]),
                    },
                    ProviderConfig {
                        name: "unify".into(),
                        kind: "openai_compatible".into(),
                        base_url: Some("https://api.unify.ai/v0".into()),
                        api_key_env: Some("UNIFY_API_KEY".into()),
                        command: None,
                        model: None,
                        models: Some(vec!["llama-3-70b@together-ai".into()]),
                    },
                    // --- Specialty / Niche ---
                    ProviderConfig {
                        name: "replicate".into(),
                        kind: "openai_compatible".into(),
                        base_url: Some("https://api.replicate.com/v1".into()),
                        api_key_env: Some("REPLICATE_API_TOKEN".into()),
                        command: None,
                        model: None,
                        models: Some(vec!["meta/meta-llama-3-70b".into()]),
                    },
                    ProviderConfig {
                        name: "cloudflare-workers-ai".into(),
                        kind: "openai_compatible".into(),
                        base_url: Some(
                            "https://api.cloudflare.com/client/v4/accounts/{account_id}/ai/v1"
                                .into(),
                        ),
                        api_key_env: Some("CLOUDFLARE_API_TOKEN".into()),
                        command: None,
                        model: None,
                        models: Some(vec![
                            "@cf/meta/llama-3.1-70b".into(),
                            "@cf/mistral/mistral-7b".into(),
                        ]),
                    },
                    ProviderConfig {
                        name: "predibase".into(),
                        kind: "openai_compatible".into(),
                        base_url: Some("https://serving.predibase.com/v1".into()),
                        api_key_env: Some("PREDIBASE_API_TOKEN".into()),
                        command: None,
                        model: None,
                        models: Some(vec!["local-model".into()]),
                    },
                    ProviderConfig {
                        name: "baseten".into(),
                        kind: "openai_compatible".into(),
                        base_url: Some(
                            "https://model-{model_id}.api.baseten.co/production/predict".into(),
                        ),
                        api_key_env: Some("BASETEN_API_KEY".into()),
                        command: None,
                        model: None,
                        models: Some(vec!["local-model".into()]),
                    },
                    ProviderConfig {
                        name: "banana".into(),
                        kind: "openai_compatible".into(),
                        base_url: Some("https://api.banana.dev".into()),
                        api_key_env: Some("BANANA_API_KEY".into()),
                        command: None,
                        model: None,
                        models: Some(vec!["local-model".into()]),
                    },
                    ProviderConfig {
                        name: "nebius".into(),
                        kind: "openai_compatible".into(),
                        base_url: Some("https://api.studio.nebius.ai/v1".into()),
                        api_key_env: Some("NEBIUS_API_KEY".into()),
                        command: None,
                        model: None,
                        models: Some(vec!["meta-llama/Meta-Llama-3.1-70B".into()]),
                    },
                    ProviderConfig {
                        name: "siliconflow".into(),
                        kind: "openai_compatible".into(),
                        base_url: Some("https://api.siliconflow.cn/v1".into()),
                        api_key_env: Some("SILICONFLOW_API_KEY".into()),
                        command: None,
                        model: None,
                        models: Some(vec![
                            "deepseek-ai/DeepSeek-V2.5".into(),
                            "Qwen/Qwen2.5-72B".into(),
                        ]),
                    },
                    ProviderConfig {
                        name: "novita".into(),
                        kind: "openai_compatible".into(),
                        base_url: Some("https://api.novita.ai/v3/openai".into()),
                        api_key_env: Some("NOVITA_API_KEY".into()),
                        command: None,
                        model: None,
                        models: Some(vec!["meta-llama/llama-3.3-70b".into()]),
                    },
                    ProviderConfig {
                        name: "hyperbolic".into(),
                        kind: "openai_compatible".into(),
                        base_url: Some("https://api.hyperbolic.xyz/v1".into()),
                        api_key_env: Some("HYPERBOLIC_API_KEY".into()),
                        command: None,
                        model: None,
                        models: Some(vec!["meta-llama/Meta-Llama-3.1-70B-Instruct".into()]),
                    },
                    ProviderConfig {
                        name: "stability".into(),
                        kind: "openai_compatible".into(),
                        base_url: Some("https://api.stability.ai/v1".into()),
                        api_key_env: Some("STABILITY_API_KEY".into()),
                        command: None,
                        model: None,
                        models: Some(vec!["stable-diffusion-xl".into()]),
                    },
                    ProviderConfig {
                        name: "exa".into(),
                        kind: "openai_compatible".into(),
                        base_url: Some("https://api.exa.ai".into()),
                        api_key_env: Some("EXA_API_KEY".into()),
                        command: None,
                        model: None,
                        models: Some(vec!["exa".into()]),
                    },
                    ProviderConfig {
                        name: "ollama-cloud".into(),
                        kind: "openai_compatible".into(),
                        base_url: Some("https://api.ollama.cloud/v1".into()),
                        api_key_env: Some("OLLAMA_CLOUD_API_KEY".into()),
                        command: None,
                        model: None,
                        models: Some(vec!["llama3.3".into(), "qwen2.5".into()]),
                    },
                    ProviderConfig {
                        name: "opencode-zen".into(),
                        kind: "openai_compatible".into(),
                        base_url: Some("https://api.opencode.ai/v1".into()),
                        api_key_env: Some("OPENCODE_API_KEY".into()),
                        command: None,
                        model: None,
                        models: Some(vec!["zen-1".into(), "zen-lite".into()]),
                    },
                    ProviderConfig {
                        name: "go-models".into(),
                        kind: "openai_compatible".into(),
                        base_url: Some("https://api.gomodels.ai/v1".into()),
                        api_key_env: Some("GO_MODELS_API_KEY".into()),
                        command: None,
                        model: None,
                        models: Some(vec!["go-1".into(), "go-mini".into()]),
                    },
                    ProviderConfig {
                        name: "zai-coding-plan".into(),
                        kind: "openai_compatible".into(),
                        base_url: Some("https://api.z.ai/api/paas/v4".into()),
                        api_key_env: Some("ZAI_API_KEY".into()),
                        command: None,
                        model: Some("glm-coding-plan".into()),
                        models: Some(vec!["glm-coding-plan".into(), "glm-5.1".into()]),
                    },
                    ProviderConfig {
                        name: "gemini-cli".into(),
                        kind: "external_cli".into(),
                        base_url: None,
                        api_key_env: None,
                        command: Some("gemini".into()),
                        model: None,
                        models: Some(vec!["gemini-default".into()]),
                    },
                ],
            },
            agents: AgentsConfig {
                enabled: vec![
                    "ra-planner".into(),
                    "thoth-architect".into(),
                    "anubis-guardian".into(),
                    "ptah-builder".into(),
                    "maat-checker".into(),
                    "seshat-docs".into(),
                ],
            },
            mcp: McpConfig {
                config_file: ".fevercode/mcp.json".into(),
            },
        }
    }
}

pub fn init_workspace(root: &Path) -> Result<()> {
    let state = root.join(".fevercode");
    fs::create_dir_all(&state)?;
    let cfg_path = state.join("config.toml");
    if !cfg_path.exists() {
        fs::write(&cfg_path, toml::to_string_pretty(&FeverConfig::default())?)?;
    }
    let mcp_path = state.join("mcp.json");
    if !mcp_path.exists() {
        fs::write(&mcp_path, "{\n  \"mcpServers\": {}\n}\n")?;
    }
    println!("Initialized {}", state.display());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_roundtrips() {
        let cfg = FeverConfig::default();
        let toml_str = toml::to_string_pretty(&cfg).unwrap();
        let parsed: FeverConfig = toml::from_str(&toml_str).unwrap();
        assert_eq!(parsed.safety.mode, ApprovalMode::Ask);
        assert_eq!(parsed.providers.available.len(), 53);
        assert!(parsed.safety.allow_writes_inside_workspace);
        assert!(!parsed.safety.allow_writes_outside_workspace);
    }

    #[test]
    fn finds_provider_by_name() {
        let cfg = FeverConfig::default();
        assert!(cfg.find_provider("zai").is_some());
        assert!(cfg.find_provider("openai").is_some());
        assert!(cfg.find_provider("ollama-local").is_some());
        assert!(cfg.find_provider("nonexistent").is_none());
    }

    #[test]
    fn detect_test_commands_rust() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("Cargo.toml"), "").unwrap();
        let cfg = FeverConfig::default();
        let cmds = cfg.detect_test_commands(dir.path());
        assert!(cmds.contains(&"cargo test".to_string()));
        assert!(cmds.contains(&"cargo clippy".to_string()));
    }
}
