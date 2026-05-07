use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEntry {
    pub id: String,
    pub category: MemoryCategory,
    pub key: String,
    pub value: String,
    pub created_at: DateTime<Utc>,
    pub accessed_at: DateTime<Utc>,
    pub access_count: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MemoryCategory {
    ProjectConvention,
    PastDecision,
    CodingStyle,
    UserPreference,
    ProjectContext,
}

pub struct MemoryStore {
    state_dir: PathBuf,
    entries: HashMap<String, MemoryEntry>,
}

impl MemoryStore {
    pub fn new(state_dir: &Path) -> Self {
        Self {
            state_dir: state_dir.to_path_buf(),
            entries: HashMap::new(),
        }
    }

    fn store_path(&self) -> PathBuf {
        self.state_dir.join("memory").join("store.json")
    }

    pub fn store(&mut self, category: MemoryCategory, key: &str, value: &str) -> Result<String> {
        let lookup = format!("{:?}", category);
        let existing = self
            .entries
            .values()
            .find(|e| e.category == category && e.key == key);
        if let Some(existing) = existing {
            let id = existing.id.clone();
            let entry = self.entries.get_mut(&id).unwrap();
            entry.value = value.to_string();
            entry.accessed_at = Utc::now();
            entry.access_count += 1;
            return Ok(id);
        }
        let id = uuid::Uuid::new_v4().to_string();
        let now = Utc::now();
        self.entries.insert(
            id.clone(),
            MemoryEntry {
                id: id.clone(),
                category,
                key: key.to_string(),
                value: value.to_string(),
                created_at: now,
                accessed_at: now,
                access_count: 0,
            },
        );
        let _ = lookup;
        Ok(id)
    }

    pub fn retrieve(&mut self, category: MemoryCategory, key: &str) -> Result<Option<String>> {
        let found = self
            .entries
            .values()
            .find(|e| e.category == category && e.key == key);
        if let Some(found) = found {
            let id = found.id.clone();
            let val = found.value.clone();
            let entry = self.entries.get_mut(&id).unwrap();
            entry.accessed_at = Utc::now();
            entry.access_count += 1;
            Ok(Some(val))
        } else {
            Ok(None)
        }
    }

    pub fn search(&self, query: &str) -> Vec<&MemoryEntry> {
        let lower = query.to_ascii_lowercase();
        self.entries
            .values()
            .filter(|e| {
                e.key.to_ascii_lowercase().contains(&lower)
                    || e.value.to_ascii_lowercase().contains(&lower)
            })
            .collect()
    }

    pub fn list_by_category(&self, category: MemoryCategory) -> Vec<&MemoryEntry> {
        self.entries
            .values()
            .filter(|e| e.category == category)
            .collect()
    }

    pub fn delete(&mut self, id: &str) -> bool {
        self.entries.remove(id).is_some()
    }

    pub fn save(&self) -> Result<()> {
        let path = self.store_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let list: Vec<&MemoryEntry> = self.entries.values().collect();
        let json = serde_json::to_string_pretty(&list)?;
        fs::write(path, json)?;
        Ok(())
    }

    pub fn load(state_dir: &Path) -> Result<Self> {
        let store = Self::new(state_dir);
        let path = store.store_path();
        if !path.exists() {
            return Ok(store);
        }
        let content = fs::read_to_string(&path)?;
        let list: Vec<MemoryEntry> = serde_json::from_str(&content)?;
        let mut entries = HashMap::new();
        for e in list {
            entries.insert(e.id.clone(), e);
        }
        Ok(Self {
            state_dir: state_dir.to_path_buf(),
            entries,
        })
    }

    pub fn stats(&self) -> String {
        let total = self.entries.len();
        let conventions = self.list_by_category(MemoryCategory::ProjectConvention).len();
        let decisions = self.list_by_category(MemoryCategory::PastDecision).len();
        let styles = self.list_by_category(MemoryCategory::CodingStyle).len();
        let prefs = self.list_by_category(MemoryCategory::UserPreference).len();
        let ctx = self.list_by_category(MemoryCategory::ProjectContext).len();
        format!(
            "Memory: {} entries ({} conventions, {} decisions, {} styles, {} preferences, {} context)",
            total, conventions, decisions, styles, prefs, ctx
        )
    }

    pub fn compact(&mut self, max_age_days: u32) -> usize {
        let cutoff = Utc::now() - chrono::Duration::days(max_age_days as i64);
        let before = self.entries.len();
        self.entries.retain(|_, e| e.accessed_at >= cutoff);
        before - self.entries.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_store_and_retrieve() {
        let tmp = TempDir::new().unwrap();
        let mut store = MemoryStore::new(tmp.path());
        let id = store
            .store(MemoryCategory::ProjectConvention, "indent", "2 spaces")
            .unwrap();
        assert!(!id.is_empty());
        let val = store
            .retrieve(MemoryCategory::ProjectConvention, "indent")
            .unwrap();
        assert_eq!(val, Some("2 spaces".to_string()));
    }

    #[test]
    fn test_store_updates_existing() {
        let tmp = TempDir::new().unwrap();
        let mut store = MemoryStore::new(tmp.path());
        let id1 = store
            .store(MemoryCategory::ProjectConvention, "lang", "rust")
            .unwrap();
        let id2 = store
            .store(MemoryCategory::ProjectConvention, "lang", "python")
            .unwrap();
        assert_eq!(id1, id2);
        let val = store
            .retrieve(MemoryCategory::ProjectConvention, "lang")
            .unwrap();
        assert_eq!(val, Some("python".to_string()));
        assert_eq!(store.entries.len(), 1);
    }

    #[test]
    fn test_search() {
        let tmp = TempDir::new().unwrap();
        let mut store = MemoryStore::new(tmp.path());
        store
            .store(MemoryCategory::PastDecision, "database", "use postgres")
            .unwrap();
        store
            .store(MemoryCategory::PastDecision, "cache", "use redis")
            .unwrap();
        let results = store.search("postgres");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].value, "use postgres");
        let results = store.search("use");
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_list_by_category() {
        let tmp = TempDir::new().unwrap();
        let mut store = MemoryStore::new(tmp.path());
        store
            .store(MemoryCategory::PastDecision, "a", "1")
            .unwrap();
        store
            .store(MemoryCategory::PastDecision, "b", "2")
            .unwrap();
        store
            .store(MemoryCategory::CodingStyle, "c", "3")
            .unwrap();
        assert_eq!(store.list_by_category(MemoryCategory::PastDecision).len(), 2);
        assert_eq!(store.list_by_category(MemoryCategory::CodingStyle).len(), 1);
    }

    #[test]
    fn test_delete() {
        let tmp = TempDir::new().unwrap();
        let mut store = MemoryStore::new(tmp.path());
        let id = store
            .store(MemoryCategory::UserPreference, "theme", "dark")
            .unwrap();
        assert!(store.delete(&id));
        assert!(!store.delete(&id));
        assert_eq!(store.entries.len(), 0);
    }

    #[test]
    fn test_save_load() {
        let tmp = TempDir::new().unwrap();
        let mut store = MemoryStore::new(tmp.path());
        store
            .store(MemoryCategory::ProjectConvention, "k", "v")
            .unwrap();
        store.save().unwrap();
        let loaded = MemoryStore::load(tmp.path()).unwrap();
        assert_eq!(loaded.entries.len(), 1);
        let val = loaded
            .entries
            .values()
            .next()
            .map(|e| e.value.clone())
            .unwrap();
        assert_eq!(val, "v");
    }

    #[test]
    fn test_compact() {
        let tmp = TempDir::new().unwrap();
        let mut store = MemoryStore::new(tmp.path());
        store
            .store(MemoryCategory::ProjectConvention, "fresh", "yes")
            .unwrap();
        let old_id = uuid::Uuid::new_v4().to_string();
        let old_now = Utc::now();
        store.entries.insert(
            old_id.clone(),
            MemoryEntry {
                id: old_id,
                category: MemoryCategory::PastDecision,
                key: "old".to_string(),
                value: "stale".to_string(),
                created_at: old_now - chrono::Duration::days(100),
                accessed_at: old_now - chrono::Duration::days(100),
                access_count: 0,
            },
        );
        assert_eq!(store.entries.len(), 2);
        let removed = store.compact(30);
        assert_eq!(removed, 1);
        assert_eq!(store.entries.len(), 1);
    }

    #[test]
    fn test_stats() {
        let tmp = TempDir::new().unwrap();
        let mut store = MemoryStore::new(tmp.path());
        store
            .store(MemoryCategory::ProjectConvention, "a", "1")
            .unwrap();
        store
            .store(MemoryCategory::PastDecision, "b", "2")
            .unwrap();
        let stats = store.stats();
        assert!(stats.contains("2 entries"));
        assert!(stats.contains("1 conventions"));
        assert!(stats.contains("1 decisions"));
    }
}
