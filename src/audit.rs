use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub user: String,
    pub action: String,
    pub target: String,
    pub result: String,
    pub risk_level: String,
    pub details: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AuditFilters {
    pub since: Option<DateTime<Utc>>,
    pub until: Option<DateTime<Utc>>,
    pub user: Option<String>,
    pub action: Option<String>,
    pub risk_level: Option<String>,
}

pub struct AuditLog {
    state_dir: PathBuf,
}

pub fn create_entry(
    user: &str,
    action: &str,
    target: &str,
    result: &str,
    risk: &str,
) -> AuditEntry {
    AuditEntry {
        id: uuid::Uuid::new_v4().to_string(),
        timestamp: Utc::now(),
        user: user.to_string(),
        action: action.to_string(),
        target: target.to_string(),
        result: result.to_string(),
        risk_level: risk.to_string(),
        details: None,
    }
}

impl AuditLog {
    pub fn new(state_dir: &Path) -> Self {
        Self {
            state_dir: state_dir.to_path_buf(),
        }
    }

    fn audit_path(&self) -> PathBuf {
        self.state_dir.join("audit").join("audit.jsonl")
    }

    pub fn record(&self, entry: AuditEntry) -> Result<()> {
        let path = self.audit_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)?;
        let json = serde_json::to_string(&entry)?;
        writeln!(file, "{}", json)?;
        Ok(())
    }

    fn load_all(&self) -> Result<Vec<AuditEntry>> {
        let path = self.audit_path();
        if !path.exists() {
            return Ok(Vec::new());
        }
        let file = File::open(&path)?;
        let reader = BufReader::new(file);
        let mut entries = Vec::new();
        for line in reader.lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }
            if let Ok(entry) = serde_json::from_str::<AuditEntry>(&line) {
                entries.push(entry);
            }
        }
        Ok(entries)
    }

    pub fn query(&self, filters: AuditFilters) -> Result<Vec<AuditEntry>> {
        let entries = self.load_all()?;
        Ok(entries
            .into_iter()
            .filter(|e| {
                if let Some(since) = filters.since {
                    if e.timestamp < since {
                        return false;
                    }
                }
                if let Some(until) = filters.until {
                    if e.timestamp > until {
                        return false;
                    }
                }
                if let Some(ref user) = filters.user {
                    if e.user != *user {
                        return false;
                    }
                }
                if let Some(ref action) = filters.action {
                    if e.action != *action {
                        return false;
                    }
                }
                if let Some(ref risk) = filters.risk_level {
                    if e.risk_level != *risk {
                        return false;
                    }
                }
                true
            })
            .collect())
    }

    pub fn export_json(&self, path: &Path) -> Result<()> {
        let entries = self.load_all()?;
        let json = serde_json::to_string_pretty(&entries)?;
        fs::write(path, json)?;
        Ok(())
    }

    pub fn export_csv(&self, path: &Path) -> Result<()> {
        let entries = self.load_all()?;
        let mut file = File::create(path)?;
        writeln!(
            file,
            "id,timestamp,user,action,target,result,risk_level,details"
        )?;
        for e in &entries {
            let details = e
                .details
                .as_ref()
                .map(|v| v.to_string())
                .unwrap_or_default();
            writeln!(
                file,
                "{},{},{},{},{},{},{},{}",
                e.id,
                e.timestamp.to_rfc3339(),
                e.user,
                e.action,
                e.target,
                e.result,
                e.risk_level,
                details
            )?;
        }
        Ok(())
    }

    pub fn retention_check(&self, days: u32) -> Result<usize> {
        let cutoff = Utc::now() - chrono::Duration::days(days as i64);
        let entries = self.load_all()?;
        Ok(entries.iter().filter(|e| e.timestamp < cutoff).count())
    }

    pub fn prune(&self, days: u32) -> Result<usize> {
        let cutoff = Utc::now() - chrono::Duration::days(days as i64);
        let entries = self.load_all()?;
        let (keep, removed): (Vec<_>, Vec<_>) = entries
            .into_iter()
            .partition(|e| e.timestamp >= cutoff);
        let count = removed.len();
        let path = self.audit_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let mut file = File::create(&path)?;
        for entry in &keep {
            let json = serde_json::to_string(entry)?;
            writeln!(file, "{}", json)?;
        }
        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_create_entry() {
        let entry = create_entry("alice", "login", "system", "success", "low");
        assert_eq!(entry.user, "alice");
        assert_eq!(entry.action, "login");
        assert_eq!(entry.target, "system");
        assert_eq!(entry.result, "success");
        assert_eq!(entry.risk_level, "low");
        assert!(!entry.id.is_empty());
        assert!(entry.details.is_none());
    }

    #[test]
    fn test_record_and_query() {
        let tmp = TempDir::new().unwrap();
        let log = AuditLog::new(tmp.path());
        let entry = create_entry("bob", "file_write", "src/main.rs", "ok", "medium");
        log.record(entry).unwrap();
        let results = log.query(AuditFilters::default()).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].user, "bob");
    }

    #[test]
    fn test_query_filters_by_user() {
        let tmp = TempDir::new().unwrap();
        let log = AuditLog::new(tmp.path());
        log.record(create_entry("alice", "a", "t", "ok", "low"))
            .unwrap();
        log.record(create_entry("bob", "b", "t", "ok", "low"))
            .unwrap();
        log.record(create_entry("alice", "c", "t", "ok", "low"))
            .unwrap();
        let results = log.query(AuditFilters {
            user: Some("alice".to_string()),
            ..Default::default()
        }).unwrap();
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_query_filters_by_date_range() {
        let tmp = TempDir::new().unwrap();
        let log = AuditLog::new(tmp.path());
        let old = AuditEntry {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: Utc::now() - chrono::Duration::days(10),
            user: "alice".to_string(),
            action: "old".to_string(),
            target: "t".to_string(),
            result: "ok".to_string(),
            risk_level: "low".to_string(),
            details: None,
        };
        log.record(old).unwrap();
        log.record(create_entry("bob", "new", "t", "ok", "low"))
            .unwrap();
        let since = Utc::now() - chrono::Duration::days(1);
        let results = log.query(AuditFilters {
            since: Some(since),
            ..Default::default()
        }).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].user, "bob");
    }

    #[test]
    fn test_export_json() {
        let tmp = TempDir::new().unwrap();
        let log = AuditLog::new(tmp.path());
        log.record(create_entry("alice", "a", "t", "ok", "low"))
            .unwrap();
        let out = tmp.path().join("export.json");
        log.export_json(&out).unwrap();
        let data = fs::read_to_string(&out).unwrap();
        let parsed: Vec<AuditEntry> = serde_json::from_str(&data).unwrap();
        assert_eq!(parsed.len(), 1);
    }

    #[test]
    fn test_export_csv() {
        let tmp = TempDir::new().unwrap();
        let log = AuditLog::new(tmp.path());
        log.record(create_entry("alice", "a", "t", "ok", "low"))
            .unwrap();
        let out = tmp.path().join("export.csv");
        log.export_csv(&out).unwrap();
        let data = fs::read_to_string(&out).unwrap();
        let lines: Vec<&str> = data.lines().collect();
        assert_eq!(lines.len(), 2);
        assert!(lines[0].starts_with("id,timestamp"));
        assert!(lines[1].contains("alice"));
    }

    #[test]
    fn test_retention_check() {
        let tmp = TempDir::new().unwrap();
        let log = AuditLog::new(tmp.path());
        let old = AuditEntry {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: Utc::now() - chrono::Duration::days(100),
            user: "old".to_string(),
            action: "a".to_string(),
            target: "t".to_string(),
            result: "ok".to_string(),
            risk_level: "low".to_string(),
            details: None,
        };
        log.record(old).unwrap();
        log.record(create_entry("new", "a", "t", "ok", "low"))
            .unwrap();
        let count = log.retention_check(30).unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_prune_old_entries() {
        let tmp = TempDir::new().unwrap();
        let log = AuditLog::new(tmp.path());
        let old = AuditEntry {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: Utc::now() - chrono::Duration::days(100),
            user: "old".to_string(),
            action: "a".to_string(),
            target: "t".to_string(),
            result: "ok".to_string(),
            risk_level: "low".to_string(),
            details: None,
        };
        log.record(old).unwrap();
        log.record(create_entry("new", "a", "t", "ok", "low"))
            .unwrap();
        let removed = log.prune(30).unwrap();
        assert_eq!(removed, 1);
        let remaining = log.query(AuditFilters::default()).unwrap();
        assert_eq!(remaining.len(), 1);
        assert_eq!(remaining[0].user, "new");
    }
}
