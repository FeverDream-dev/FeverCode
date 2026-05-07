use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionStats {
    pub total_commands: u64,
    pub total_files_changed: u64,
    pub total_tokens_used: u64,
    pub total_time_seconds: u64,
    pub provider_usage: HashMap<String, u64>,
    pub command_counts: HashMap<String, u64>,
    pub files_changed: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AnalyticsCollector {
    state_dir: PathBuf,
    stats: SessionStats,
}

impl AnalyticsCollector {
    pub fn new(state_dir: &Path) -> Self {
        Self {
            state_dir: state_dir.to_path_buf(),
            stats: SessionStats {
                total_commands: 0,
                total_files_changed: 0,
                total_tokens_used: 0,
                total_time_seconds: 0,
                provider_usage: HashMap::new(),
                command_counts: HashMap::new(),
                files_changed: Vec::new(),
            },
        }
    }

    pub fn record_command(&mut self, command: &str) {
        *self
            .stats
            .command_counts
            .entry(command.to_string())
            .or_insert(0) += 1;
        self.stats.total_commands += 1;
    }

    pub fn record_file_change(&mut self, file: &str) {
        if !self.stats.files_changed.iter().any(|f| f == file) {
            self.stats.files_changed.push(file.to_string());
            self.stats.total_files_changed = self.stats.files_changed.len() as u64;
        }
    }

    pub fn record_tokens(&mut self, provider: &str, count: u64) {
        *self
            .stats
            .provider_usage
            .entry(provider.to_string())
            .or_insert(0) += count;
        self.stats.total_tokens_used += count;
    }

    pub fn record_time(&mut self, seconds: u64) {
        self.stats.total_time_seconds += seconds;
    }

    pub fn get_stats(&self) -> SessionStats {
        self.stats.clone()
    }

    pub fn format_report(&self) -> String {
        let time_str = format_time(self.stats.total_time_seconds);
        let tokens_str = format_tokens_with_providers(
            &self.stats.provider_usage,
            self.stats.total_tokens_used,
        );
        let savings = self.stats.total_time_seconds as f64 * 5.0 / 3600.0;

        format!(
            "Session Analytics\n=================\nCommands run: {}\nFiles changed: {}\nTokens used: {}\nTime: {}\nEstimated savings: {:.1} hours",
            self.stats.total_commands,
            self.stats.total_files_changed,
            tokens_str,
            time_str,
            savings
        )
    }

    pub fn save(&self) -> Result<()> {
        let dir = self.state_dir.join(".fevercode").join("analytics");
        fs::create_dir_all(&dir)?;
        let json = serde_json::to_string_pretty(self)?;
        fs::write(dir.join("session.json"), json)?;
        Ok(())
    }

    pub fn load(state_dir: &Path) -> Result<Self> {
        let path = state_dir
            .join(".fevercode")
            .join("analytics")
            .join("session.json");
        if !path.exists() {
            return Ok(Self::new(state_dir));
        }
        let data = fs::read_to_string(&path)?;
        let mut collector: AnalyticsCollector = serde_json::from_str(&data)?;
        collector.state_dir = state_dir.to_path_buf();
        Ok(collector)
    }
}

fn format_time(seconds: u64) -> String {
    let h = seconds / 3600;
    let m = (seconds % 3600) / 60;
    let s = seconds % 60;
    if h > 0 {
        format!("{}h {}m {}s", h, m, s)
    } else {
        format!("{}m {}s", m, s)
    }
}

fn format_number(n: u64) -> String {
    let s = n.to_string();
    let mut result = String::new();
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(c);
    }
    result.chars().rev().collect()
}

fn format_tokens_with_providers(usage: &HashMap<String, u64>, total: u64) -> String {
    let total_str = format_number(total);
    if usage.is_empty() {
        return total_str;
    }
    let providers: Vec<String> = usage
        .iter()
        .map(|(k, v)| format!("{}: {}", k, format_number(*v)))
        .collect();
    format!("{} ({})", total_str, providers.join(", "))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp_dir() -> tempfile::TempDir {
        tempfile::tempdir().unwrap()
    }

    #[test]
    fn test_record_command() {
        let dir = tmp_dir();
        let mut c = AnalyticsCollector::new(dir.path());
        c.record_command("edit");
        c.record_command("edit");
        c.record_command("run");
        let stats = c.get_stats();
        assert_eq!(stats.total_commands, 3);
        assert_eq!(stats.command_counts.get("edit"), Some(&2));
        assert_eq!(stats.command_counts.get("run"), Some(&1));
    }

    #[test]
    fn test_record_tokens() {
        let dir = tmp_dir();
        let mut c = AnalyticsCollector::new(dir.path());
        c.record_tokens("openai", 1000);
        c.record_tokens("ollama-local", 500);
        c.record_tokens("openai", 500);
        let stats = c.get_stats();
        assert_eq!(stats.total_tokens_used, 2000);
        assert_eq!(stats.provider_usage.get("openai"), Some(&1500));
    }

    #[test]
    fn test_record_file_change() {
        let dir = tmp_dir();
        let mut c = AnalyticsCollector::new(dir.path());
        c.record_file_change("src/main.rs");
        c.record_file_change("src/lib.rs");
        c.record_file_change("src/main.rs");
        let stats = c.get_stats();
        assert_eq!(stats.total_files_changed, 2);
        assert_eq!(stats.files_changed.len(), 2);
    }

    #[test]
    fn test_format_report() {
        let dir = tmp_dir();
        let mut c = AnalyticsCollector::new(dir.path());
        for _ in 0..42 {
            c.record_command("edit");
        }
        c.record_tokens("openai", 12000);
        c.record_tokens("ollama-local", 3230);
        c.record_time(1425);
        c.record_file_change("a.rs");
        c.record_file_change("b.rs");
        c.record_file_change("c.rs");
        c.record_file_change("d.rs");
        c.record_file_change("e.rs");
        c.record_file_change("f.rs");
        c.record_file_change("g.rs");
        c.record_file_change("h.rs");
        let report = c.format_report();
        assert!(report.contains("Session Analytics"));
        assert!(report.contains("Commands run: 42"));
        assert!(report.contains("Files changed: 8"));
        assert!(report.contains("Tokens used: 15,230"));
        assert!(report.contains("openai: 12,000"));
        assert!(report.contains("23m 45s"));
    }

    #[test]
    fn test_save_load_roundtrip() {
        let dir = tmp_dir();
        let mut c = AnalyticsCollector::new(dir.path());
        c.record_command("test");
        c.record_tokens("provider", 100);
        c.record_file_change("a.rs");
        c.record_time(60);
        c.save().unwrap();

        let loaded = AnalyticsCollector::load(dir.path()).unwrap();
        let stats = loaded.get_stats();
        assert_eq!(stats.total_commands, 1);
        assert_eq!(stats.total_tokens_used, 100);
        assert_eq!(stats.total_files_changed, 1);
        assert_eq!(stats.total_time_seconds, 60);
    }

    #[test]
    fn test_empty_stats_report() {
        let dir = tmp_dir();
        let c = AnalyticsCollector::new(dir.path());
        let report = c.format_report();
        assert!(report.contains("Commands run: 0"));
        assert!(report.contains("Files changed: 0"));
        assert!(report.contains("Tokens used: 0"));
        assert!(report.contains("0m 0s"));
    }
}
