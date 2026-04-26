use anyhow::Result;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::Path,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SessionEventType {
    SessionStart,
    BeforeTool,
    AfterTool,
    BeforeEdit,
    AfterEdit,
    BeforeCommand,
    AfterCommand,
    BeforeCompact,
    AfterCompact,
    SessionStop,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionEvent {
    pub timestamp: String,
    pub event_type: SessionEventType,
    pub soul: String,
    pub summary: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

impl SessionEvent {
    pub fn new(event_type: SessionEventType, soul: &str, summary: &str) -> Self {
        Self {
            timestamp: Utc::now().to_rfc3339(),
            event_type,
            soul: soul.to_string(),
            summary: summary.to_string(),
            detail: None,
        }
    }

    pub fn with_detail(mut self, detail: &str) -> Self {
        self.detail = Some(detail.to_string());
        self
    }
}

pub struct SessionLog {
    events_path: std::path::PathBuf,
    summary_path: std::path::PathBuf,
}

impl SessionLog {
    pub fn new(state_dir: &Path) -> Self {
        let session_dir = state_dir.join("session");
        Self {
            events_path: session_dir.join("events.jsonl"),
            summary_path: session_dir.join("latest.md"),
        }
    }

    pub fn ensure_dir(&self) -> Result<()> {
        if let Some(parent) = self.events_path.parent() {
            fs::create_dir_all(parent)?;
        }
        Ok(())
    }

    pub fn append(&self, event: &SessionEvent) -> Result<()> {
        self.ensure_dir()?;
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.events_path)?;
        let mut line = serde_json::to_string(event)?;
        line.push('\n');
        file.write_all(line.as_bytes())?;
        Ok(())
    }

    pub fn read_events(&self) -> Result<Vec<SessionEvent>> {
        if !self.events_path.exists() {
            return Ok(Vec::new());
        }
        let content = fs::read_to_string(&self.events_path)?;
        let events = content
            .lines()
            .filter(|l| !l.trim().is_empty())
            .filter_map(|l| serde_json::from_str(l).ok())
            .collect();
        Ok(events)
    }

    pub fn write_summary(&self, summary: &str) -> Result<()> {
        self.ensure_dir()?;
        fs::write(&self.summary_path, summary)?;
        Ok(())
    }

    pub fn read_summary(&self) -> Result<Option<String>> {
        if !self.summary_path.exists() {
            return Ok(None);
        }
        let content = fs::read_to_string(&self.summary_path)?;
        Ok(Some(content))
    }

    pub fn events_path(&self) -> &Path {
        &self.events_path
    }

    pub fn summary_path(&self) -> &Path {
        &self.summary_path
    }

    pub fn generate_summary(&self) -> Result<String> {
        let events = self.read_events()?;
        let mut files_changed: Vec<String> = Vec::new();
        let mut commands_run: Vec<String> = Vec::new();
        let mut errors: Vec<String> = Vec::new();
        let mut start_time: Option<&str> = None;

        for event in &events {
            if start_time.is_none() {
                start_time = Some(&event.timestamp);
            }

            match event.event_type {
                SessionEventType::BeforeEdit | SessionEventType::AfterEdit => {
                    if let Some(ref d) = event.detail {
                        if !files_changed.contains(d) {
                            files_changed.push(d.clone());
                        }
                    }
                }
                SessionEventType::BeforeCommand => {
                    if let Some(ref d) = event.detail {
                        commands_run.push(d.clone());
                    }
                }
                SessionEventType::AfterCommand => {
                    if let Some(ref d) = event.detail {
                        if d.contains("error") || d.contains("FAIL") || d.contains("failed") {
                            errors.push(d.clone());
                        }
                    }
                }
                _ => {}
            }
        }

        let mut summary = String::new();
        summary.push_str("# FeverCode Session Summary\n\n");
        summary.push_str(&format!("Started: {}\n", start_time.unwrap_or("unknown")));
        summary.push_str(&format!("Events: {}\n", events.len()));
        summary.push_str(&format!("Files changed: {}\n", files_changed.len()));
        summary.push_str(&format!("Commands run: {}\n", commands_run.len()));
        summary.push_str(&format!("Errors: {}\n", errors.len()));

        if !files_changed.is_empty() {
            summary.push_str("\n## Files Changed\n");
            for f in &files_changed {
                summary.push_str(&format!("- {}\n", f));
            }
        }

        if !errors.is_empty() {
            summary.push_str("\n## Errors\n");
            for e in &errors {
                summary.push_str(&format!("- {}\n", e));
            }
        }

        if !commands_run.is_empty() {
            summary.push_str("\n## Commands\n");
            for c in &commands_run {
                summary.push_str(&format!("- {}\n", c));
            }
        }

        Ok(summary)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_event_serializes() {
        let event = SessionEvent::new(SessionEventType::SessionStart, "ra", "started session");
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("session_start"));
        assert!(json.contains("started session"));
    }

    #[test]
    fn session_event_roundtrips() {
        let event = SessionEvent::new(SessionEventType::BeforeEdit, "ptah", "editing main.rs")
            .with_detail("src/main.rs");
        let json = serde_json::to_string(&event).unwrap();
        let parsed: SessionEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.event_type, SessionEventType::BeforeEdit);
        assert_eq!(parsed.detail.unwrap(), "src/main.rs");
    }

    #[test]
    fn session_log_append_and_read() {
        let dir = tempfile::tempdir().unwrap();
        let log = SessionLog::new(dir.path());

        log.append(&SessionEvent::new(
            SessionEventType::SessionStart,
            "ra",
            "test session",
        ))
        .unwrap();

        log.append(&SessionEvent::new(
            SessionEventType::AfterCommand,
            "maat",
            "cargo test",
        ))
        .unwrap();

        let events = log.read_events().unwrap();
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].event_type, SessionEventType::SessionStart);
        assert_eq!(events[1].soul, "maat");
    }

    #[test]
    fn session_log_generates_summary() {
        let dir = tempfile::tempdir().unwrap();
        let log = SessionLog::new(dir.path());

        log.append(&SessionEvent::new(
            SessionEventType::SessionStart,
            "ra",
            "task start",
        ))
        .unwrap();

        let evt1 = SessionEvent::new(SessionEventType::BeforeEdit, "ptah", "edit file")
            .with_detail("src/main.rs");
        log.append(&evt1).unwrap();

        let evt2 = SessionEvent::new(SessionEventType::AfterEdit, "ptah", "edited file")
            .with_detail("src/main.rs");
        log.append(&evt2).unwrap();

        let evt3 = SessionEvent::new(SessionEventType::BeforeCommand, "maat", "run test")
            .with_detail("cargo test");
        log.append(&evt3).unwrap();

        let summary = log.generate_summary().unwrap();
        assert!(summary.contains("Events: 4"));
        assert!(summary.contains("Files changed: 1"));
        assert!(summary.contains("Commands run: 1"));
        assert!(summary.contains("src/main.rs"));
    }

    #[test]
    fn empty_events_gives_basic_summary() {
        let dir = tempfile::tempdir().unwrap();
        let log = SessionLog::new(dir.path());
        let summary = log.generate_summary().unwrap();
        assert!(summary.contains("Events: 0"));
    }

    #[test]
    fn read_events_missing_file_returns_empty() {
        let dir = tempfile::tempdir().unwrap();
        let log = SessionLog::new(dir.path());
        let events = log.read_events().unwrap();
        assert!(events.is_empty());
    }

    #[test]
    fn write_and_read_summary() {
        let dir = tempfile::tempdir().unwrap();
        let log = SessionLog::new(dir.path());
        log.write_summary("test summary").unwrap();
        let content = log.read_summary().unwrap();
        assert_eq!(content, Some("test summary".to_string()));
    }
}
