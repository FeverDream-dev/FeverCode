use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub providers: HashMap<String, ProviderConfig>,
    pub defaults: DefaultConfig,
    pub ui: UiConfig,
    pub tools: ToolsConfig,
    pub search: SearchConfig,
    pub permissions: PermissionsConfig,
    pub instructions: Option<String>, // path to custom instructions file
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub enabled: bool,
    pub api_key: Option<String>,
    pub base_url: Option<String>,
    pub model: Option<String>,
    pub extra: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefaultConfig {
    pub provider: Option<String>,
    pub model: Option<String>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    pub theme: Option<String>,
    pub auto_scroll: Option<bool>,
    pub show_thinking: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolsConfig {
    pub browser_enabled: Option<bool>,
    pub search_enabled: Option<bool>,
    pub shell_enabled: Option<bool>,
    pub git_enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchConfig {
    pub engine: Option<String>,
    pub searxng_url: Option<String>,
    pub max_results: Option<usize>,
    pub cache_enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PermissionsConfig {
    pub mode: Option<String>,       // "read", "write", "full"
    pub allow: Option<Vec<String>>, // tool names always allowed
    pub deny: Option<Vec<String>>,  // tool names always denied
}

impl PermissionsConfig {
    fn merge(&mut self, overlay: &PermissionsConfig) {
        if overlay.mode.is_some() {
            self.mode = overlay.mode.clone();
        }
        if overlay.allow.is_some() {
            self.allow = overlay.allow.clone();
        }
        if overlay.deny.is_some() {
            self.deny = overlay.deny.clone();
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            providers: HashMap::new(),
            defaults: DefaultConfig {
                provider: None,
                model: None,
                temperature: Some(0.7),
                max_tokens: Some(4096),
            },
            ui: UiConfig {
                theme: Some("dark".to_string()),
                auto_scroll: Some(true),
                show_thinking: Some(true),
            },
            tools: ToolsConfig {
                browser_enabled: Some(true),
                search_enabled: Some(true),
                shell_enabled: Some(true),
                git_enabled: Some(true),
            },
            search: SearchConfig {
                engine: Some("duckduckgo".to_string()),
                searxng_url: None,
                max_results: Some(10),
                cache_enabled: Some(true),
            },
            permissions: PermissionsConfig::default(),
            instructions: None,
        }
    }
}

impl DefaultConfig {
    fn merge(&self, overlay: &DefaultConfig) -> Self {
        Self {
            provider: overlay.provider.clone().or(self.provider.clone()),
            model: overlay.model.clone().or(self.model.clone()),
            temperature: overlay.temperature.or(self.temperature),
            max_tokens: overlay.max_tokens.or(self.max_tokens),
        }
    }
}

impl UiConfig {
    fn merge(&self, overlay: &UiConfig) -> Self {
        Self {
            theme: overlay.theme.clone().or(self.theme.clone()),
            auto_scroll: overlay.auto_scroll.or(self.auto_scroll),
            show_thinking: overlay.show_thinking.or(self.show_thinking),
        }
    }
}

impl ToolsConfig {
    fn merge(&self, overlay: &ToolsConfig) -> Self {
        Self {
            browser_enabled: overlay.browser_enabled.or(self.browser_enabled),
            search_enabled: overlay.search_enabled.or(self.search_enabled),
            shell_enabled: overlay.shell_enabled.or(self.shell_enabled),
            git_enabled: overlay.git_enabled.or(self.git_enabled),
        }
    }
}

impl SearchConfig {
    fn merge(&self, overlay: &SearchConfig) -> Self {
        Self {
            engine: overlay.engine.clone().or(self.engine.clone()),
            searxng_url: overlay.searxng_url.clone().or(self.searxng_url.clone()),
            max_results: overlay.max_results.or(self.max_results),
            cache_enabled: overlay.cache_enabled.or(self.cache_enabled),
        }
    }
}

impl ProviderConfig {
    fn merge(&mut self, overlay: &ProviderConfig) {
        self.enabled = overlay.enabled;
        if overlay.api_key.is_some() {
            self.api_key = overlay.api_key.clone();
        }
        if overlay.base_url.is_some() {
            self.base_url = overlay.base_url.clone();
        }
        if overlay.model.is_some() {
            self.model = overlay.model.clone();
        }
        // Merge extras (overlay can add/override keys)
        for (k, v) in &overlay.extra {
            self.extra.insert(k.clone(), v.clone());
        }
    }
}

impl Config {
    // Deep merge overlay into self. Overlay wins for Some values; HashMaps merge entries
    pub fn deep_merge(&mut self, overlay: &Config) {
        // merge defaults
        self.defaults = self.defaults.merge(&overlay.defaults);
        // merge ui
        self.ui = self.ui.merge(&overlay.ui);
        // merge tools
        self.tools = self.tools.merge(&overlay.tools);
        // merge search
        self.search = self.search.merge(&overlay.search);
        // merge permissions
        self.permissions.merge(&overlay.permissions);
        // merge instructions
        if overlay.instructions.is_some() {
            self.instructions = overlay.instructions.clone();
        }
        // merge providers (by key)
        for (name, ov) in &overlay.providers {
            self.providers
                .entry(name.clone())
                .or_insert_with(|| ov.clone());
            if let Some(base) = self.providers.get_mut(name) {
                base.merge(ov);
            }
        }
    }
}

pub struct ConfigManager {
    config_path: PathBuf,
    config_dir: PathBuf,
}

impl ConfigManager {
    pub fn new() -> anyhow::Result<Self> {
        let config_dir = directories::ProjectDirs::from("org", "feverdream", "fevercode")
            .map(|d| d.config_dir().to_path_buf())
            .unwrap_or_else(|| {
                let mut path = std::env::var("HOME")
                    .map(PathBuf::from)
                    .unwrap_or(PathBuf::from("."));
                path.push(".config");
                path.push("fevercode");
                path
            });

        std::fs::create_dir_all(&config_dir)?;

        let config_path = config_dir.join("config.toml");

        Ok(Self {
            config_path,
            config_dir,
        })
    }

    pub fn load(&self) -> anyhow::Result<Config> {
        if self.config_path.exists() {
            let content = std::fs::read_to_string(&self.config_path)?;
            let config: Config = toml::from_str(&content)?;
            Ok(config)
        } else {
            Ok(Config::default())
        }
    }

    pub fn load_with_workspace(&self, workspace: &Path) -> anyhow::Result<Config> {
        // Start with defaults
        let mut merged = Config::default();

        // 1) Project workspace config: <workspace>/.fevercode/config.toml
        let workspace_config_path = workspace.join(".fevercode").join("config.toml");
        if workspace_config_path.exists() {
            let content = std::fs::read_to_string(&workspace_config_path)?;
            let project_config: Config = toml::from_str(&content)?;
            merged.deep_merge(&project_config);
        }

        // 2) User config: ~/.config/fevercode/config.toml (existing self.config_path)
        if self.config_path.exists() {
            let content = std::fs::read_to_string(&self.config_path)?;
            let user_config: Config = toml::from_str(&content)?;
            merged.deep_merge(&user_config);
        }

        Ok(merged)
    }

    pub fn save(&self, config: &Config) -> anyhow::Result<()> {
        let content = toml::to_string_pretty(config)?;
        std::fs::write(&self.config_path, content)?;
        Ok(())
    }

    pub fn config_dir(&self) -> &PathBuf {
        &self.config_dir
    }

    pub fn config_path(&self) -> &PathBuf {
        &self.config_path
    }

    pub fn data_dir(&self) -> PathBuf {
        let data_dir = directories::ProjectDirs::from("org", "feverdream", "fevercode")
            .map(|d| d.data_dir().to_path_buf())
            .unwrap_or_else(|| {
                let mut path = std::env::var("HOME")
                    .map(PathBuf::from)
                    .unwrap_or(PathBuf::from("."));
                path.push(".local");
                path.push("share");
                path.push("fevercode");
                path
            });

        std::fs::create_dir_all(&data_dir).ok();
        data_dir
    }

    pub fn cache_dir(&self) -> PathBuf {
        let cache_dir = directories::ProjectDirs::from("org", "feverdream", "fevercode")
            .map(|d: directories::ProjectDirs| d.cache_dir().to_path_buf())
            .unwrap_or_else(|| {
                let mut path = std::env::var("HOME")
                    .map(PathBuf::from)
                    .unwrap_or(PathBuf::from("."));
                path.push(".cache");
                path.push("fevercode");
                path
            });

        std::fs::create_dir_all(&cache_dir).ok();
        cache_dir
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_default_config() {
        let cfg = Config::default();
        assert!(cfg.permissions.mode.is_none());
        assert!(cfg.permissions.allow.is_none());
        assert!(cfg.permissions.deny.is_none());
        assert!(cfg.instructions.is_none());
        assert_eq!(cfg.defaults.temperature, Some(0.7));
        assert_eq!(cfg.ui.theme, Some("dark".to_string()));
    }

    #[test]
    fn test_deep_merge_none_values() {
        let mut base = Config {
            defaults: DefaultConfig {
                provider: Some("base".to_string()),
                model: Some("m1".to_string()),
                temperature: Some(0.5),
                max_tokens: Some(2048),
            },
            ui: UiConfig {
                theme: Some("light".to_string()),
                auto_scroll: Some(true),
                show_thinking: Some(true),
            },
            ..Config::default()
        };

        let overlay = Config {
            providers: HashMap::new(),
            defaults: DefaultConfig {
                provider: None,
                model: None,
                temperature: None,
                max_tokens: None,
            },
            ui: UiConfig {
                theme: None,
                auto_scroll: None,
                show_thinking: None,
            },
            tools: ToolsConfig {
                browser_enabled: None,
                search_enabled: None,
                shell_enabled: None,
                git_enabled: None,
            },
            search: SearchConfig {
                engine: None,
                searxng_url: None,
                max_results: None,
                cache_enabled: None,
            },
            permissions: PermissionsConfig::default(),
            instructions: None,
        };

        base.deep_merge(&overlay);

        assert_eq!(base.defaults.provider, Some("base".to_string()));
        assert_eq!(base.defaults.model, Some("m1".to_string()));
        assert_eq!(base.defaults.temperature, Some(0.5));
        assert_eq!(base.ui.theme, Some("light".to_string()));
    }

    #[test]
    fn test_deep_merge_some_values() {
        let mut base = Config {
            defaults: DefaultConfig {
                provider: Some("base".to_string()),
                model: None,
                temperature: None,
                max_tokens: None,
            },
            ..Config::default()
        };

        let overlay = Config {
            providers: HashMap::new(),
            defaults: DefaultConfig {
                provider: Some("overlay".to_string()),
                model: Some("model2".to_string()),
                temperature: Some(0.9),
                max_tokens: Some(8192),
            },
            ui: UiConfig {
                theme: None,
                auto_scroll: None,
                show_thinking: None,
            },
            tools: ToolsConfig {
                browser_enabled: None,
                search_enabled: None,
                shell_enabled: None,
                git_enabled: None,
            },
            search: SearchConfig {
                engine: None,
                searxng_url: None,
                max_results: None,
                cache_enabled: None,
            },
            permissions: PermissionsConfig::default(),
            instructions: None,
        };

        base.deep_merge(&overlay);
        assert_eq!(base.defaults.provider, Some("overlay".to_string()));
        assert_eq!(base.defaults.model, Some("model2".to_string()));
        assert_eq!(base.defaults.temperature, Some(0.9));
        assert_eq!(base.defaults.max_tokens, Some(8192));
    }

    #[test]
    fn test_deep_merge_providers() {
        let mut base = Config::default();
        let p = ProviderConfig {
            enabled: true,
            api_key: Some("base-key".to_string()),
            base_url: Some("https://base".to_string()),
            model: Some("gpt-3".to_string()),
            extra: HashMap::new(),
        };
        base.providers.insert("openai".to_string(), p);

        let overlay = Config {
            providers: {
                let mut m = HashMap::new();
                m.insert(
                    "openai".to_string(),
                    ProviderConfig {
                        enabled: false,
                        api_key: None,
                        base_url: Some("https://new".to_string()),
                        model: None,
                        extra: {
                            let mut e = HashMap::new();
                            e.insert("timeout".to_string(), "30s".to_string());
                            e
                        },
                    },
                );
                m
            },
            ..Config::default()
        };

        base.deep_merge(&overlay);
        let ov = base.providers.get("openai").unwrap();
        assert!(!ov.enabled);
        assert_eq!(ov.base_url.as_ref().unwrap(), "https://new");
        assert_eq!(ov.api_key.as_ref().unwrap(), "base-key");
        assert_eq!(ov.extra.get("timeout").unwrap(), &"30s".to_string());
    }

    #[test]
    fn test_permissions_config_default() {
        let cfg = Config::default();
        assert!(cfg.permissions.mode.is_none());
        assert!(cfg.permissions.allow.is_none());
        assert!(cfg.permissions.deny.is_none());
    }
}
