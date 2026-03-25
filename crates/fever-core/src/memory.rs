use crate::error::{Result, Error};
use rusqlite::{Connection, params};
use serde_json::Value;
use std::path::Path;

pub struct MemoryStore {
    conn: Connection,
}

impl MemoryStore {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let conn = Connection::open(path)?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS context (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                session_id TEXT NOT NULL,
                key TEXT NOT NULL,
                value TEXT NOT NULL,
                timestamp INTEGER NOT NULL,
                UNIQUE(session_id, key)
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS messages (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                session_id TEXT NOT NULL,
                role TEXT NOT NULL,
                content TEXT NOT NULL,
                timestamp INTEGER NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_context_session ON context(session_id)",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_messages_session ON messages(session_id)",
            [],
        )?;

        Ok(Self { conn })
    }

    pub fn set(&self, session_id: &str, key: &str, value: &Value) -> Result<()> {
        let timestamp = chrono::Utc::now().timestamp();
        let json_value = serde_json::to_string(value)?;

        self.conn.execute(
            "INSERT OR REPLACE INTO context (session_id, key, value, timestamp) VALUES (?1, ?2, ?3, ?4)",
            params![session_id, key, json_value, timestamp],
        )?;

        Ok(())
    }

    pub fn get(&self, session_id: &str, key: &str) -> Result<Option<Value>> {
        let mut stmt = self.conn.prepare(
            "SELECT value FROM context WHERE session_id = ?1 AND key = ?2"
        )?;

        let mut rows = stmt.query(params![session_id, key])?;

        if let Some(row) = rows.next()? {
            let json_str: String = row.get(0)?;
            let value: Value = serde_json::from_str(&json_str)?;
            Ok(Some(value))
        } else {
            Ok(None)
        }
    }

    pub fn list_session_keys(&self, session_id: &str) -> Result<Vec<String>> {
        let mut stmt = self.conn.prepare(
            "SELECT key FROM context WHERE session_id = ?1 ORDER BY timestamp DESC"
        )?;

        let mut keys = Vec::new();
        let mut rows = stmt.query(params![session_id])?;

        while let Some(row) = rows.next()? {
            keys.push(row.get(0)?);
        }

        Ok(keys)
    }

    pub fn add_message(&self, session_id: &str, role: &str, content: &str) -> Result<()> {
        let timestamp = chrono::Utc::now().timestamp();

        self.conn.execute(
            "INSERT INTO messages (session_id, role, content, timestamp) VALUES (?1, ?2, ?3, ?4)",
            params![session_id, role, content, timestamp],
        )?;

        Ok(())
    }

    pub fn get_messages(&self, session_id: &str, limit: Option<usize>) -> Result<Vec<StoredMessage>> {
        let query = if let Some(limit) = limit {
            format!(
                "SELECT role, content, timestamp FROM messages WHERE session_id = ?1 ORDER BY timestamp DESC LIMIT {}",
                limit
            )
        } else {
            "SELECT role, content, timestamp FROM messages WHERE session_id = ?1 ORDER BY timestamp DESC".to_string()
        };

        let mut stmt = self.conn.prepare(&query)?;
        let mut rows = stmt.query(params![session_id])?;

        let mut messages = Vec::new();
        while let Some(row) = rows.next()? {
            messages.push(StoredMessage {
                role: row.get(0)?,
                content: row.get(1)?,
                timestamp: row.get(2)?,
            });
        }

        messages.reverse();
        Ok(messages)
    }
}

#[derive(Debug, Clone)]
pub struct StoredMessage {
    pub role: String,
    pub content: String,
    pub timestamp: i64,
}
