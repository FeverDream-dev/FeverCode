use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamConfig {
    pub team_id: String,
    pub team_name: String,
    pub admin_email: String,
    pub seats: u32,
    pub shared_souls: Vec<String>,
    pub approved_providers: Vec<String>,
    pub safety_preset: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TeamRole {
    Admin,
    Member,
    Viewer,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamMember {
    pub email: String,
    pub role: TeamRole,
    pub joined_at: DateTime<Utc>,
    pub last_active: Option<DateTime<Utc>>,
}

pub struct TeamManager {
    state_dir: PathBuf,
}

impl TeamManager {
    pub fn new(state_dir: &Path) -> Self {
        Self {
            state_dir: state_dir.to_path_buf(),
        }
    }

    fn config_path(&self) -> PathBuf {
        self.state_dir.join("team").join("config.toml")
    }

    fn members_path(&self) -> PathBuf {
        self.state_dir.join("team").join("members.json")
    }

    fn ensure_dir(&self) -> Result<()> {
        let dir = self.state_dir.join("team");
        fs::create_dir_all(&dir)?;
        Ok(())
    }

    pub fn load_config(&self) -> Result<Option<TeamConfig>> {
        let path = self.config_path();
        if !path.exists() {
            return Ok(None);
        }
        let content = fs::read_to_string(&path)?;
        let config: TeamConfig = toml::from_str(&content)?;
        Ok(Some(config))
    }

    pub fn save_config(&self, config: &TeamConfig) -> Result<()> {
        self.ensure_dir()?;
        let content = toml::to_string_pretty(config)?;
        fs::write(self.config_path(), content)?;
        Ok(())
    }

    pub fn load_members(&self) -> Result<Vec<TeamMember>> {
        let path = self.members_path();
        if !path.exists() {
            return Ok(Vec::new());
        }
        let content = fs::read_to_string(&path)?;
        let members: Vec<TeamMember> = serde_json::from_str(&content)?;
        Ok(members)
    }

    pub fn save_members(&self, members: &[TeamMember]) -> Result<()> {
        self.ensure_dir()?;
        let content = serde_json::to_string_pretty(members)?;
        fs::write(self.members_path(), content)?;
        Ok(())
    }

    pub fn add_member(&self, email: &str, role: TeamRole) -> Result<()> {
        let mut members = self.load_members()?;
        members.push(TeamMember {
            email: email.to_string(),
            role,
            joined_at: Utc::now(),
            last_active: None,
        });
        self.save_members(&members)?;
        Ok(())
    }

    pub fn remove_member(&self, email: &str) -> Result<bool> {
        let mut members = self.load_members()?;
        let before = members.len();
        members.retain(|m| m.email != email);
        let removed = members.len() < before;
        if removed {
            self.save_members(&members)?;
        }
        Ok(removed)
    }

    pub fn is_approved_provider(&self, provider: &str) -> bool {
        match self.load_config() {
            Ok(Some(config)) => {
                if config.approved_providers.is_empty() {
                    return true;
                }
                let set: HashSet<&str> = config.approved_providers.iter().map(|s| s.as_str()).collect();
                set.contains(provider)
            }
            _ => true,
        }
    }

    pub fn shared_souls(&self) -> Result<Vec<String>> {
        match self.load_config()? {
            Some(config) => Ok(config.shared_souls),
            None => Ok(Vec::new()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_team_config_roundtrip() {
        let tmp = TempDir::new().unwrap();
        let mgr = TeamManager::new(tmp.path());
        let config = TeamConfig {
            team_id: "team-1".to_string(),
            team_name: "Alpha".to_string(),
            admin_email: "admin@test.com".to_string(),
            seats: 10,
            shared_souls: vec!["ra".to_string()],
            approved_providers: vec!["openai".to_string()],
            safety_preset: Some("strict".to_string()),
            created_at: Utc::now(),
        };
        mgr.save_config(&config).unwrap();
        let loaded = mgr.load_config().unwrap().unwrap();
        assert_eq!(loaded.team_id, "team-1");
        assert_eq!(loaded.team_name, "Alpha");
        assert_eq!(loaded.seats, 10);
        assert_eq!(loaded.shared_souls, vec!["ra"]);
        assert_eq!(loaded.approved_providers, vec!["openai"]);
    }

    #[test]
    fn test_add_remove_member() {
        let tmp = TempDir::new().unwrap();
        let mgr = TeamManager::new(tmp.path());
        mgr.add_member("alice@test.com", TeamRole::Admin).unwrap();
        mgr.add_member("bob@test.com", TeamRole::Member).unwrap();
        let members = mgr.load_members().unwrap();
        assert_eq!(members.len(), 2);
        let removed = mgr.remove_member("alice@test.com").unwrap();
        assert!(removed);
        let members = mgr.load_members().unwrap();
        assert_eq!(members.len(), 1);
        assert_eq!(members[0].email, "bob@test.com");
        let removed = mgr.remove_member("nobody@test.com").unwrap();
        assert!(!removed);
    }

    #[test]
    fn test_member_roles() {
        let tmp = TempDir::new().unwrap();
        let mgr = TeamManager::new(tmp.path());
        mgr.add_member("a@t.com", TeamRole::Admin).unwrap();
        mgr.add_member("b@t.com", TeamRole::Member).unwrap();
        mgr.add_member("c@t.com", TeamRole::Viewer).unwrap();
        let members = mgr.load_members().unwrap();
        assert_eq!(members[0].role, TeamRole::Admin);
        assert_eq!(members[1].role, TeamRole::Member);
        assert_eq!(members[2].role, TeamRole::Viewer);
    }

    #[test]
    fn test_approved_providers_empty_allows_all() {
        let tmp = TempDir::new().unwrap();
        let mgr = TeamManager::new(tmp.path());
        let config = TeamConfig {
            team_id: "t1".to_string(),
            team_name: "T".to_string(),
            admin_email: "a@t.com".to_string(),
            seats: 5,
            shared_souls: vec![],
            approved_providers: vec![],
            safety_preset: None,
            created_at: Utc::now(),
        };
        mgr.save_config(&config).unwrap();
        assert!(mgr.is_approved_provider("anything"));
        assert!(mgr.is_approved_provider("openai"));
    }

    #[test]
    fn test_approved_providers_list_restricts() {
        let tmp = TempDir::new().unwrap();
        let mgr = TeamManager::new(tmp.path());
        let config = TeamConfig {
            team_id: "t1".to_string(),
            team_name: "T".to_string(),
            admin_email: "a@t.com".to_string(),
            seats: 5,
            shared_souls: vec![],
            approved_providers: vec!["openai".to_string(), "anthropic".to_string()],
            safety_preset: None,
            created_at: Utc::now(),
        };
        mgr.save_config(&config).unwrap();
        assert!(mgr.is_approved_provider("openai"));
        assert!(mgr.is_approved_provider("anthropic"));
        assert!(!mgr.is_approved_provider("google"));
    }
}
