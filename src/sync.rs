use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncEvent {
    pub event_type: String,
    pub data: serde_json::Value,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncPayload {
    pub session_id: String,
    pub machine_id: String,
    pub timestamp: DateTime<Utc>,
    pub events: Vec<SyncEvent>,
    pub encrypted_data: Option<String>,
}

pub struct SyncManager {
    sync_dir: PathBuf,
}

impl SyncManager {
    pub fn new(state_dir: &Path) -> Self {
        Self {
            sync_dir: state_dir.join(".fevercode").join("sync"),
        }
    }

    pub fn create_payload(&self, session_id: &str, events: &[SyncEvent]) -> Result<SyncPayload> {
        let machine_id = std::env::var("HOSTNAME")
            .or_else(|_| std::env::var("COMPUTERNAME"))
            .unwrap_or_else(|_| "unknown".to_string());
        Ok(SyncPayload {
            session_id: session_id.to_string(),
            machine_id,
            timestamp: Utc::now(),
            events: events.to_vec(),
            encrypted_data: None,
        })
    }

    /// Placeholder for AES-256-GCM encryption. Uses XOR cipher for MVP.
    pub fn encrypt_payload(&self, payload: &SyncPayload, key: &[u8; 32]) -> Result<Vec<u8>> {
        let json_bytes = serde_json::to_vec(payload)?;
        let encrypted: Vec<u8> = json_bytes
            .iter()
            .enumerate()
            .map(|(i, &b)| b ^ key[i % 32])
            .collect();
        Ok(encrypted)
    }

    /// Placeholder for AES-256-GCM decryption. Uses XOR cipher for MVP.
    pub fn decrypt_payload(&self, data: &[u8], key: &[u8; 32]) -> Result<SyncPayload> {
        let decrypted: Vec<u8> = data
            .iter()
            .enumerate()
            .map(|(i, &b)| b ^ key[i % 32])
            .collect();
        let payload = serde_json::from_slice(&decrypted)?;
        Ok(payload)
    }

    pub fn save_local(&self, payload: &SyncPayload) -> Result<()> {
        fs::create_dir_all(&self.sync_dir)?;
        let json = serde_json::to_string_pretty(payload)?;
        fs::write(self.sync_dir.join("latest.json"), json)?;
        Ok(())
    }

    pub fn load_local(&self) -> Result<Option<SyncPayload>> {
        let path = self.sync_dir.join("latest.json");
        if !path.exists() {
            return Ok(None);
        }
        let data = fs::read_to_string(&path)?;
        let payload = serde_json::from_str(&data)?;
        Ok(Some(payload))
    }

    pub fn push_to_cloud(&self, _payload: &SyncPayload, endpoint: &str) -> Result<()> {
        println!("Sync: would push to {}", endpoint);
        Ok(())
    }

    pub fn pull_from_cloud(&self, _endpoint: &str) -> Result<Option<SyncPayload>> {
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp_dir() -> tempfile::TempDir {
        tempfile::tempdir().unwrap()
    }

    #[test]
    fn test_create_payload() {
        let dir = tmp_dir();
        let mgr = SyncManager::new(dir.path());
        let events = vec![SyncEvent {
            event_type: "test".to_string(),
            data: serde_json::json!({"key": "value"}),
            timestamp: Utc::now(),
        }];
        let payload = mgr.create_payload("sess-1", &events).unwrap();
        assert_eq!(payload.session_id, "sess-1");
        assert_eq!(payload.events.len(), 1);
        assert!(payload.encrypted_data.is_none());
    }

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let dir = tmp_dir();
        let mgr = SyncManager::new(dir.path());
        let payload = SyncPayload {
            session_id: "s1".to_string(),
            machine_id: "m1".to_string(),
            timestamp: Utc::now(),
            events: vec![],
            encrypted_data: None,
        };
        let key = [42u8; 32];
        let encrypted = mgr.encrypt_payload(&payload, &key).unwrap();
        let decrypted = mgr.decrypt_payload(&encrypted, &key).unwrap();
        assert_eq!(decrypted.session_id, payload.session_id);
        assert_eq!(decrypted.machine_id, payload.machine_id);
    }

    #[test]
    fn test_save_load_local() {
        let dir = tmp_dir();
        let mgr = SyncManager::new(dir.path());
        let payload = SyncPayload {
            session_id: "s2".to_string(),
            machine_id: "m2".to_string(),
            timestamp: Utc::now(),
            events: vec![],
            encrypted_data: None,
        };
        mgr.save_local(&payload).unwrap();
        let loaded = mgr.load_local().unwrap().unwrap();
        assert_eq!(loaded.session_id, "s2");
    }

    #[test]
    fn test_xor_encrypt_produces_different_output() {
        let dir = tmp_dir();
        let mgr = SyncManager::new(dir.path());
        let payload = SyncPayload {
            session_id: "s3".to_string(),
            machine_id: "m3".to_string(),
            timestamp: Utc::now(),
            events: vec![],
            encrypted_data: None,
        };
        let key = [42u8; 32];
        let encrypted = mgr.encrypt_payload(&payload, &key).unwrap();
        let original = serde_json::to_vec(&payload).unwrap();
        assert_ne!(encrypted, original);
    }
}
