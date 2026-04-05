use serde::{Deserialize, Serialize};
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryEvent {
    pub timestamp: String,
    pub session_id: String,
    pub event_type: TelemetryEventType,
    pub data: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TelemetryEventType {
    SessionStarted,
    UserMessage,
    AssistantMessage,
    ToolCallStarted,
    ToolCallCompleted,
    ToolCallFailed,
    PermissionChecked,
    Error,
}

pub trait TelemetrySink: Send + Sync {
    fn record(&self, event: &TelemetryEvent);
    fn flush(&self);
}

pub struct MemorySink {
    events: Mutex<Vec<TelemetryEvent>>,
}

impl MemorySink {
    pub fn new() -> Self {
        MemorySink {
            events: Mutex::new(Vec::new()),
        }
    }

    pub fn events(&self) -> Vec<TelemetryEvent> {
        self.events.lock().unwrap().clone()
    }

    pub fn clear(&self) {
        self.events.lock().unwrap().clear();
    }
}

impl TelemetrySink for MemorySink {
    fn record(&self, event: &TelemetryEvent) {
        self.events.lock().unwrap().push(event.clone());
    }
    fn flush(&self) {}
}

pub struct JsonlSink {
    #[allow(dead_code)]
    path: PathBuf,
    file: Mutex<std::fs::File>,
}

impl JsonlSink {
    pub fn new(path: PathBuf) -> std::io::Result<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .write(true)
            .open(&path)?;
        Ok(JsonlSink {
            path,
            file: Mutex::new(file),
        })
    }
}

impl TelemetrySink for JsonlSink {
    fn record(&self, event: &TelemetryEvent) {
        let line = serde_json::to_string(event).unwrap();
        let mut f = self.file.lock().unwrap();
        let _ = writeln!(f, "{}", line);
    }
    fn flush(&self) {
        let mut f = self.file.lock().unwrap();
        let _ = f.flush();
    }
}

pub struct Telemetry {
    sinks: Vec<Box<dyn TelemetrySink>>,
    session_id: String,
}

impl Telemetry {
    pub fn new(session_id: String) -> Self {
        Telemetry {
            sinks: Vec::new(),
            session_id,
        }
    }

    pub fn add_sink(&mut self, sink: Box<dyn TelemetrySink>) {
        self.sinks.push(sink);
    }

    pub fn record(&self, event_type: TelemetryEventType, data: serde_json::Value) {
        let event = TelemetryEvent {
            timestamp: chrono::Utc::now().to_rfc3339(),
            session_id: self.session_id.clone(),
            event_type,
            data,
        };
        for sink in &self.sinks {
            sink.record(&event);
        }
    }

    pub fn flush(&self) {
        for sink in &self.sinks {
            sink.flush();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::fs;
    use std::path::PathBuf;

    #[test]
    fn test_memory_sink_records_events() {
        let sink = MemorySink::new();
        let e1 = TelemetryEvent {
            timestamp: "2020-01-01T00:00:00Z".to_string(),
            session_id: "sess1".to_string(),
            event_type: TelemetryEventType::SessionStarted,
            data: json!({"a": 1}),
        };
        let e2 = TelemetryEvent {
            timestamp: "2020-01-01T00:00:01Z".to_string(),
            session_id: "sess1".to_string(),
            event_type: TelemetryEventType::UserMessage,
            data: json!({"b": 2}),
        };
        sink.record(&e1);
        sink.record(&e2);
        let events = sink.events();
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].event_type, TelemetryEventType::SessionStarted);
        assert_eq!(events[1].event_type, TelemetryEventType::UserMessage);
    }

    #[test]
    fn test_memory_sink_clear() {
        let sink = MemorySink::new();
        let e = TelemetryEvent {
            timestamp: "t".to_string(),
            session_id: "sess".to_string(),
            event_type: TelemetryEventType::Error,
            data: json!({}),
        };
        sink.record(&e);
        assert_eq!(sink.events().len(), 1);
        sink.clear();
        assert_eq!(sink.events().len(), 0);
    }

    #[test]
    fn test_telemetry_auto_fills_timestamp() {
        let mut t = Telemetry::new("sess-auto".to_string());
        let mem = MemorySink::new();
        t.add_sink(Box::new(mem));
        t.record(TelemetryEventType::SessionStarted, json!({}));
        let e = TelemetryEvent {
            timestamp: chrono::Utc::now().to_rfc3339(),
            session_id: "sess-auto".to_string(),
            event_type: TelemetryEventType::SessionStarted,
            data: json!({}),
        };
        assert!(!e.timestamp.is_empty());
        assert_eq!(e.session_id, "sess-auto");
    }

    #[test]
    fn test_telemetry_auto_fills_session_id() {
        let sess = "session-xyz".to_string();
        let te = Telemetry::new(sess.clone());
        let mem = MemorySink::new();
        let mut te_owned = te;
        te_owned.add_sink(Box::new(mem));
        te_owned.record(TelemetryEventType::SessionStarted, json!({}));
        let e = TelemetryEvent {
            timestamp: chrono::Utc::now().to_rfc3339(),
            session_id: sess.clone(),
            event_type: TelemetryEventType::SessionStarted,
            data: json!({}),
        };
        assert_eq!(e.session_id, sess);
    }

    #[test]
    fn test_jsonl_sink_writes_file() {
        let path = PathBuf::from("/tmp/fever_telemetry_test.jsonl");
        let _ = fs::remove_file(&path);
        let jsonl = JsonlSink::new(path.clone()).expect("failed to create jsonl sink");
        let mut t = Telemetry::new("sess-jsonl".to_string());
        t.add_sink(Box::new(jsonl));
        t.record(
            TelemetryEventType::SessionStarted,
            json!({"hello": "world"}),
        );
        t.flush();
        let content = fs::read_to_string(path).expect("read file");
        assert!(!content.trim().is_empty());
        let first_line = content.lines().next().unwrap();
        let parsed: TelemetryEvent = serde_json::from_str(first_line).unwrap();
        assert_eq!(parsed.session_id, "sess-jsonl");
        assert_eq!(parsed.event_type, TelemetryEventType::SessionStarted);
    }
}
