use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::Path;

/// Persisted project onboarding profile used to bootstrap a FeverCode project.
#[derive(Serialize, Deserialize, Default, Debug, Clone, PartialEq)]
pub struct ProjectProfile {
    /// Human-readable project name.
    pub project_name: String,
    /// Short description of the project.
    pub description: String,
    /// Target end-user type (e.g., end users, admins).
    pub end_user_type: String,
    /// Onboarding state indicator (e.g., draft, finalized).
    pub current_state: String,
    /// Primary programming language used by the project.
    pub primary_language: String,
    /// Web framework used (e.g., Axum, Actix).
    pub framework: String,
    /// Database technology (e.g., PostgreSQL).
    pub database: String,
    /// Frontend technology (e.g., React).
    pub frontend: String,
    /// External API integrations referenced by the project.
    pub external_apis: Vec<String>,
    /// Hosting platform (Railway, Render, Fly.io, etc.).
    pub hosting_platform: String,
    /// Delivery method (API, Web, Background jobs).
    pub delivery_method: String,
    /// CI/CD requirement (GitHub Actions, GitLab CI, None).
    pub cicd_needed: String,
    /// Environment variable keys used by the project.
    pub env_vars: Vec<String>,
    /// Optional custom domain for deployment.
    pub custom_domain: Option<String>,
    /// Quality level descriptor (Low, Medium, High, Production).
    pub quality_level: String,
    /// Flag indicating whether existing tests exist.
    pub existing_tests: bool,
    /// Style guide to follow for code formatting and conventions.
    pub style_guide: String,
    /// Documentation needs description.
    pub documentation_needs: String,
    /// Definition of done for the project.
    pub definition_of_done: String,
    /// Areas or topics that are off-limits for onboarding.
    pub off_limits: Vec<String>,
    /// Urgency level for onboarding (Low, Normal, High).
    pub urgency_level: String,
    pub created_at: String,
    pub updated_at: String,
}

impl ProjectProfile {
    pub fn save_to(&self, path: &Path) -> Result<(), std::io::Error> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let s = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        let mut f = fs::File::create(path)?;
        f.write_all(s.as_bytes())?;
        Ok(())
    }

    pub fn load_from(path: &Path) -> Result<Self, std::io::Error> {
        let data = fs::read_to_string(path)?;
        let profile: ProjectProfile = serde_json::from_str(&data).unwrap_or_default();
        Ok(profile)
    }

    pub fn from_answers(answers: std::collections::HashMap<String, String>) -> Self {
        let now = Utc::now().to_rfc3339();
        ProjectProfile {
            project_name: answers.get("project_name").cloned().unwrap_or_default(),
            description: answers.get("description").cloned().unwrap_or_default(),
            end_user_type: answers.get("end_user_type").cloned().unwrap_or_default(),
            current_state: answers.get("current_state").cloned().unwrap_or_default(),
            primary_language: answers.get("primary_language").cloned().unwrap_or_default(),
            framework: answers.get("framework").cloned().unwrap_or_default(),
            database: answers.get("database").cloned().unwrap_or_default(),
            frontend: answers.get("frontend").cloned().unwrap_or_default(),
            external_apis: answers
                .get("external_apis")
                .map(|s| {
                    s.split(',')
                        .map(|x| x.trim().to_string())
                        .filter(|t| !t.is_empty())
                        .collect()
                })
                .unwrap_or_default(),
            hosting_platform: answers.get("hosting_platform").cloned().unwrap_or_default(),
            delivery_method: answers.get("delivery_method").cloned().unwrap_or_default(),
            cicd_needed: answers.get("cicd_needed").cloned().unwrap_or_default(),
            env_vars: answers
                .get("env_vars")
                .map(|s| {
                    s.split(',')
                        .map(|x| x.trim().to_string())
                        .filter(|t| !t.is_empty())
                        .collect()
                })
                .unwrap_or_default(),
            custom_domain: answers
                .get("custom_domain")
                .and_then(|v| if v.is_empty() { None } else { Some(v.clone()) }),
            quality_level: answers.get("quality_level").cloned().unwrap_or_default(),
            existing_tests: matches!(
                answers.get("existing_tests").map(|s| s.as_str()),
                Some("true")
            ),
            style_guide: answers.get("style_guide").cloned().unwrap_or_default(),
            documentation_needs: answers
                .get("documentation_needs")
                .cloned()
                .unwrap_or_default(),
            definition_of_done: answers
                .get("definition_of_done")
                .cloned()
                .unwrap_or_default(),
            off_limits: answers
                .get("off_limits")
                .map(|s| {
                    s.split(',')
                        .map(|x| x.trim().to_string())
                        .filter(|t| !t.is_empty())
                        .collect()
                })
                .unwrap_or_default(),
            urgency_level: answers.get("urgency_level").cloned().unwrap_or_default(),
            created_at: now.clone(),
            updated_at: now,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    #[test]
    fn save_and_load_profile_roundtrip() {
        let profile = ProjectProfile {
            project_name: "demo".to_string(),
            description: "desc".to_string(),
            end_user_type: "General".to_string(),
            current_state: "1".to_string(),
            primary_language: "Rust".to_string(),
            framework: "Axum".to_string(),
            database: "PostgreSQL".to_string(),
            frontend: "React".to_string(),
            external_apis: vec!["api1".to_string()],
            hosting_platform: "Railway".to_string(),
            delivery_method: "API".to_string(),
            cicd_needed: "GitHub Actions".to_string(),
            env_vars: vec!["VAR1".to_string()],
            custom_domain: None,
            quality_level: "High".to_string(),
            existing_tests: false,
            style_guide: "Standard".to_string(),
            documentation_needs: "Docs".to_string(),
            definition_of_done: "Done".to_string(),
            off_limits: vec![],
            urgency_level: "Normal".to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
        };

        let dir = std::env::temp_dir();
        let mut path = PathBuf::from(dir);
        path.push("fever_onboard_profile_test.json");

        // Ensure no existing file
        let _ = fs::remove_file(&path);

        profile.save_to(&path).expect("save");
        let loaded = ProjectProfile::load_from(&path).expect("load");
        assert_eq!(profile.project_name, loaded.project_name);
        // Cleanup
        let _ = fs::remove_file(&path);
    }
}
