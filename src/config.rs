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
                        models: Some(vec!["gpt-5.5-codex".into(), "gpt-5.5".into()]),
                    },
                    ProviderConfig {
                        name: "ollama-local".into(),
                        kind: "openai_compatible".into(),
                        base_url: Some("http://localhost:11434/v1".into()),
                        api_key_env: None,
                        command: None,
                        model: None,
                        models: Some(vec!["qwen2.5-coder".into(), "deepseek-coder".into()]),
                    },
                    ProviderConfig {
                        name: "ollama-cloud".into(),
                        kind: "openai_compatible".into(),
                        base_url: Some("https://ollama.com/v1".into()),
                        api_key_env: Some("OLLAMA_CLOUD_API_KEY".into()),
                        command: None,
                        model: None,
                        models: Some(vec!["gpt-oss".into(), "qwen3-coder".into()]),
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
        assert_eq!(parsed.providers.available.len(), 5);
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
