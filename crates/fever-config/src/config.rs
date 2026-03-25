use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub providers: HashMap<String, ProviderConfig>,
    pub defaults: DefaultConfig,
    pub ui: UiConfig,
    pub tools: ToolsConfig,
    pub search: SearchConfig,
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
