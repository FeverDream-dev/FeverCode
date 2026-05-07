use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

const BUILTIN_SOULS: &[&str] = &["ra", "thoth", "ptah", "maat", "anubis", "seshat"];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomSoul {
    pub name: String,
    pub role: String,
    pub system_prompt: String,
    pub allowed_tools: Vec<String>,
    pub risk_level: String,
    pub context_budget: u32,
    pub created_by: String,
    pub created_at: DateTime<Utc>,
}

pub struct CustomSoulManager {
    state_dir: PathBuf,
}

impl CustomSoulManager {
    pub fn new(state_dir: &Path) -> Self {
        Self {
            state_dir: state_dir.to_path_buf(),
        }
    }

    fn souls_dir(&self) -> PathBuf {
        self.state_dir.join("souls")
    }

    fn soul_path(&self, name: &str) -> PathBuf {
        self.souls_dir().join(format!("{}.toml", name))
    }

    pub fn create(&self, soul: CustomSoul) -> Result<()> {
        let dir = self.souls_dir();
        fs::create_dir_all(&dir)?;
        let path = self.soul_path(&soul.name);
        let content = toml::to_string_pretty(&soul)?;
        fs::write(path, content)?;
        Ok(())
    }

    pub fn load(&self, name: &str) -> Result<CustomSoul> {
        let path = self.soul_path(name);
        let content = fs::read_to_string(&path)?;
        let soul: CustomSoul = toml::from_str(&content)?;
        Ok(soul)
    }

    pub fn list(&self) -> Result<Vec<String>> {
        let dir = self.souls_dir();
        if !dir.exists() {
            return Ok(Vec::new());
        }
        let mut names = Vec::new();
        for entry in fs::read_dir(&dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().map(|e| e == "toml").unwrap_or(false) {
                if let Some(stem) = path.file_stem() {
                    names.push(stem.to_string_lossy().to_string());
                }
            }
        }
        names.sort();
        Ok(names)
    }

    pub fn delete(&self, name: &str) -> Result<bool> {
        let path = self.soul_path(name);
        if path.exists() {
            fs::remove_file(path)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn validate(&self, soul: &CustomSoul) -> Vec<String> {
        let mut warnings = Vec::new();
        if soul.name.trim().is_empty() {
            warnings.push("name is empty".to_string());
        }
        if soul.system_prompt.trim().is_empty() {
            warnings.push("system_prompt is empty".to_string());
        }
        if !["low", "medium", "high"].contains(&soul.risk_level.as_str()) {
            warnings.push(format!(
                "risk_level '{}' is not one of: low, medium, high",
                soul.risk_level
            ));
        }
        let lower = soul.name.to_ascii_lowercase();
        if BUILTIN_SOULS.contains(&lower.as_str()) {
            warnings.push(format!(
                "name '{}' conflicts with built-in soul",
                soul.name
            ));
        }
        warnings
    }

    pub fn create_default(
        name: &str,
        role: &str,
        prompt: &str,
        created_by: &str,
    ) -> CustomSoul {
        CustomSoul {
            name: name.to_string(),
            role: role.to_string(),
            system_prompt: prompt.to_string(),
            allowed_tools: vec![
                "read".to_string(),
                "write".to_string(),
                "search".to_string(),
            ],
            risk_level: "medium".to_string(),
            context_budget: 8000,
            created_by: created_by.to_string(),
            created_at: Utc::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn make_soul(name: &str) -> CustomSoul {
        CustomSoul {
            name: name.to_string(),
            role: "builder".to_string(),
            system_prompt: "You build things.".to_string(),
            allowed_tools: vec!["read".to_string(), "write".to_string()],
            risk_level: "medium".to_string(),
            context_budget: 8000,
            created_by: "test".to_string(),
            created_at: Utc::now(),
        }
    }

    #[test]
    fn test_create_and_load() {
        let tmp = TempDir::new().unwrap();
        let mgr = CustomSoulManager::new(tmp.path());
        let soul = make_soul("custom-builder");
        mgr.create(soul).unwrap();
        let loaded = mgr.load("custom-builder").unwrap();
        assert_eq!(loaded.name, "custom-builder");
        assert_eq!(loaded.role, "builder");
        assert_eq!(loaded.system_prompt, "You build things.");
    }

    #[test]
    fn test_list_souls() {
        let tmp = TempDir::new().unwrap();
        let mgr = CustomSoulManager::new(tmp.path());
        assert!(mgr.list().unwrap().is_empty());
        mgr.create(make_soul("alpha")).unwrap();
        mgr.create(make_soul("beta")).unwrap();
        let names = mgr.list().unwrap();
        assert_eq!(names, vec!["alpha", "beta"]);
    }

    #[test]
    fn test_delete() {
        let tmp = TempDir::new().unwrap();
        let mgr = CustomSoulManager::new(tmp.path());
        mgr.create(make_soul("deleteme")).unwrap();
        assert!(mgr.delete("deleteme").unwrap());
        assert!(!mgr.delete("deleteme").unwrap());
        assert!(mgr.list().unwrap().is_empty());
    }

    #[test]
    fn test_validate_good() {
        let tmp = TempDir::new().unwrap();
        let mgr = CustomSoulManager::new(tmp.path());
        let soul = make_soul("good-soul");
        let warnings = mgr.validate(&soul);
        assert!(warnings.is_empty());
    }

    #[test]
    fn test_validate_empty_name() {
        let tmp = TempDir::new().unwrap();
        let mgr = CustomSoulManager::new(tmp.path());
        let mut soul = make_soul("");
        soul.system_prompt = "ok".to_string();
        let warnings = mgr.validate(&soul);
        assert!(warnings.iter().any(|w| w.contains("empty")));
    }

    #[test]
    fn test_validate_builtin_conflict() {
        let tmp = TempDir::new().unwrap();
        let mgr = CustomSoulManager::new(tmp.path());
        let soul = make_soul("ra");
        let warnings = mgr.validate(&soul);
        assert!(warnings.iter().any(|w| w.contains("built-in")));
    }

    #[test]
    fn test_create_default() {
        let soul = CustomSoulManager::create_default(
            "helper",
            "assistant",
            "You help.",
            "admin@test.com",
        );
        assert_eq!(soul.name, "helper");
        assert_eq!(soul.role, "assistant");
        assert_eq!(soul.risk_level, "medium");
        assert_eq!(soul.context_budget, 8000);
        assert_eq!(soul.allowed_tools.len(), 3);
    }

    #[test]
    fn test_toml_roundtrip() {
        let tmp = TempDir::new().unwrap();
        let mgr = CustomSoulManager::new(tmp.path());
        let original = CustomSoul {
            name: "toml-test".to_string(),
            role: "reviewer".to_string(),
            system_prompt: "Review code carefully.".to_string(),
            allowed_tools: vec!["read".to_string(), "search".to_string()],
            risk_level: "low".to_string(),
            context_budget: 12000,
            created_by: "alice".to_string(),
            created_at: Utc::now(),
        };
        mgr.create(original.clone()).unwrap();
        let loaded = mgr.load("toml-test").unwrap();
        assert_eq!(loaded.name, original.name);
        assert_eq!(loaded.role, original.role);
        assert_eq!(loaded.system_prompt, original.system_prompt);
        assert_eq!(loaded.allowed_tools, original.allowed_tools);
        assert_eq!(loaded.risk_level, original.risk_level);
        assert_eq!(loaded.context_budget, original.context_budget);
        assert_eq!(loaded.created_by, original.created_by);
    }
}
