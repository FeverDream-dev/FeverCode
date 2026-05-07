use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryConfig {
    pub enabled: bool,
    pub user_id: Option<String>,
    pub endpoint: String,
    pub last_ping: Option<DateTime<Utc>>,
}

impl Default for TelemetryConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            user_id: None,
            endpoint: "https://telemetry.fevercode.io/v1".to_string(),
            last_ping: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum TelemetryEvent {
    CommandUsed { name: String },
    ProviderUsed { name: String },
    SessionStarted,
    SessionEnded { duration_secs: u64 },
    ErrorOccurred { error_type: String },
}

pub struct TelemetryCollector {
    config: TelemetryConfig,
    state_dir: PathBuf,
    events: Vec<TelemetryEvent>,
}

impl TelemetryCollector {
    pub fn new(config: TelemetryConfig, state_dir: &Path) -> Self {
        Self {
            config,
            state_dir: state_dir.to_path_buf(),
            events: Vec::new(),
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    pub fn record(&mut self, event: TelemetryEvent) {
        if self.config.enabled {
            self.events.push(event);
        }
    }

    pub fn flush(&mut self) -> Result<()> {
        if !self.config.enabled || self.events.is_empty() {
            return Ok(());
        }
        let dir = self.state_dir.join(".fevercode").join("telemetry");
        fs::create_dir_all(&dir)?;
        let path = dir.join("events.jsonl");
        let mut file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)?;
        for event in &self.events {
            writeln!(file, "{}", serde_json::to_string(event)?)?;
        }
        self.events.clear();
        Ok(())
    }

    pub fn load_config(state_dir: &Path) -> Result<TelemetryConfig> {
        let path = state_dir.join(".fevercode").join("telemetry.toml");
        if !path.exists() {
            return Ok(TelemetryConfig::default());
        }
        let data = fs::read_to_string(&path)?;
        let config: TelemetryConfig = toml::from_str(&data)?;
        Ok(config)
    }

    pub fn save_config(config: &TelemetryConfig, state_dir: &Path) -> Result<()> {
        let dir = state_dir.join(".fevercode");
        fs::create_dir_all(&dir)?;
        let toml_str = toml::to_string_pretty(config)?;
        fs::write(dir.join("telemetry.toml"), toml_str)?;
        Ok(())
    }

    pub fn opt_in(state_dir: &Path) -> Result<()> {
        let mut config = Self::load_config(state_dir).unwrap_or_default();
        config.enabled = true;
        config.user_id = Some(uuid::Uuid::new_v4().to_string());
        Self::save_config(&config, state_dir)
    }

    pub fn opt_out(state_dir: &Path) -> Result<()> {
        let mut config = Self::load_config(state_dir).unwrap_or_default();
        config.enabled = false;
        Self::save_config(&config, state_dir)
    }

    pub fn format_status(&self) -> String {
        if self.config.enabled {
            let uid = self.config.user_id.as_deref().unwrap_or("unknown");
            format!("Telemetry: enabled (user: {})", uid)
        } else {
            "Telemetry: disabled".to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp_dir() -> tempfile::TempDir {
        tempfile::tempdir().unwrap()
    }

    #[test]
    fn test_default_is_disabled() {
        let config = TelemetryConfig::default();
        assert!(!config.enabled);
    }

    #[test]
    fn test_opt_in_enables() {
        let dir = tmp_dir();
        TelemetryCollector::opt_in(dir.path()).unwrap();
        let config = TelemetryCollector::load_config(dir.path()).unwrap();
        assert!(config.enabled);
        assert!(config.user_id.is_some());
    }

    #[test]
    fn test_opt_out_disables() {
        let dir = tmp_dir();
        TelemetryCollector::opt_in(dir.path()).unwrap();
        TelemetryCollector::opt_out(dir.path()).unwrap();
        let config = TelemetryCollector::load_config(dir.path()).unwrap();
        assert!(!config.enabled);
    }

    #[test]
    fn test_record_when_disabled_does_nothing() {
        let dir = tmp_dir();
        let config = TelemetryConfig::default();
        let mut collector = TelemetryCollector::new(config, dir.path());
        collector.record(TelemetryEvent::SessionStarted);
        assert!(collector.events.is_empty());
    }

    #[test]
    fn test_record_when_enabled_queues() {
        let dir = tmp_dir();
        let config = TelemetryConfig {
            enabled: true,
            ..TelemetryConfig::default()
        };
        let mut collector = TelemetryCollector::new(config, dir.path());
        collector.record(TelemetryEvent::SessionStarted);
        collector.record(TelemetryEvent::CommandUsed {
            name: "edit".to_string(),
        });
        assert_eq!(collector.events.len(), 2);
    }

    #[test]
    fn test_flush_writes_jsonl() {
        let dir = tmp_dir();
        let config = TelemetryConfig {
            enabled: true,
            ..TelemetryConfig::default()
        };
        let mut collector = TelemetryCollector::new(config, dir.path());
        collector.record(TelemetryEvent::SessionStarted);
        collector.record(TelemetryEvent::CommandUsed {
            name: "edit".to_string(),
        });
        collector.flush().unwrap();

        let path = dir
            .path()
            .join(".fevercode")
            .join("telemetry")
            .join("events.jsonl");
        let content = fs::read_to_string(&path).unwrap();
        let lines: Vec<&str> = content.trim().lines().collect();
        assert_eq!(lines.len(), 2);
        assert!(lines[0].contains("SessionStarted"));
        assert!(lines[1].contains("CommandUsed"));

        collector.flush().unwrap();
        let content_after = fs::read_to_string(&path).unwrap();
        assert_eq!(content_after, content);
    }

    #[test]
    fn test_load_config_default() {
        let dir = tmp_dir();
        let config = TelemetryCollector::load_config(dir.path()).unwrap();
        assert!(!config.enabled);
        assert!(config.user_id.is_none());
    }
}
