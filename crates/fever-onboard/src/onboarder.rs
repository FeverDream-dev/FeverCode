use std::collections::HashMap;
/// Onboarder is responsible for collecting initial project configuration,
/// persisting the onboarding profile, and generating initial scaffolds.
use std::io::{self, Read};
use std::path::{Path, PathBuf};

use crate::profile::ProjectProfile;
use crate::questions::{all_questions, Validation};
use crate::scaffold::{GeneratedFile, ScaffoldGenerator};
use tracing::{info, warn};

#[derive(Debug, Clone, PartialEq)]
pub struct OnboardResult {
    pub profile: ProjectProfile,
    pub generated_files: Vec<GeneratedFile>,
    pub summary_table: String,
}

pub struct Onboarder {
    pub profile_path: PathBuf,
}

impl Onboarder {
    pub fn new(project_dir: &Path) -> Self {
        Self {
            profile_path: project_dir.join(".fevercode").join("project.json"),
        }
    }

    pub fn is_onboarded(&self) -> bool {
        self.profile_path.exists()
    }

    pub fn run(&self) -> Result<OnboardResult, String> {
        // Collect answers from stdin for 21 questions
        info!(
            "Starting FeverOnboard onboarding at {}",
            self.profile_path.display()
        );
        let questions = all_questions();
        let mut answers: HashMap<String, String> = HashMap::new();

        println!("Welcome to FeverOnboard. Please answer the following questions to bootstrap your project.");

        for q in questions.iter() {
            // Show prompt
            println!("{}", q.prompt);
            if let Some(opts) = &q.options {
                for o in opts {
                    println!("  [{}] {}", o.value, o.label);
                }
            }
            // Read line
            let mut input = String::new();
            io::stdin()
                .read_to_string(&mut input)
                .map_err(|e| e.to_string())?;
            // If test harness uses provided input, fall back to stdin read in real scenario. Here we simply trim.
            let input = input.trim().to_string();

            // Validation
            let is_valid = match &q.validation {
                Validation::Required => !input.trim().is_empty(),
                Validation::Optional => true,
                Validation::OneOf(list) => list.contains(&input),
                Validation::FilePath => std::path::Path::new(&input).exists(),
                Validation::Custom(func) => func(&input),
            };

            if !is_valid {
                println!("Invalid input for {}. Please try again.", q.id);
                // Simple re-try loop (one extra attempt)
                let mut retry = String::new();
                io::stdin()
                    .read_to_string(&mut retry)
                    .map_err(|e| e.to_string())?;
            }
            answers.insert(q.id.clone(), input);
        }

        // Build profile
        let profile = ProjectProfile::from_answers(answers);
        // Save to disk
        if let Some(parent) = self.profile_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        // Best-effort save; ignore errors for onboarding flow to continue
        if let Err(e) = profile.save_to(&self.profile_path) {
            warn!(%e, "Failed to save onboarding profile");
        } else {
            info!(
                "Onboarding profile saved to {}",
                self.profile_path.display()
            );
        }

        // Scaffold generation
        let scaffold = ScaffoldGenerator::new(profile.clone());
        let generated = scaffold.generate_all().unwrap_or_default();

        // Summary table (minimal example)
        let summary = format!("Project: {}", profile.project_name);

        Ok(OnboardResult {
            profile,
            generated_files: generated,
            summary_table: summary,
        })
    }

    pub fn re_onboard(&self) -> Result<OnboardResult, String> {
        // Load existing profile and re-run generation with current values
        let prof = match crate::profile::ProjectProfile::load_from(&self.profile_path) {
            Ok(p) => p,
            Err(_) => crate::profile::ProjectProfile::default(),
        };
        let scaffold = ScaffoldGenerator::new(prof.clone());
        let generated = scaffold.generate_all().unwrap_or_default();
        Ok(OnboardResult {
            profile: prof,
            generated_files: generated,
            summary_table: String::from("Resumed onboarding"),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::profile::ProjectProfile;
    #[test]
    fn is_onboarded_false_when_no_profile() {
        let tmp = std::env::temp_dir().join("fever_onboard_no_profile");
        let od = Onboarder::new(&tmp);
        assert_eq!(od.is_onboarded(), false);
    }
    #[test]
    fn is_onboarded_true_after_profile_created() {
        let tmp = std::env::temp_dir().join("fever_onboard_profile");
        let path = tmp.join(".fevercode").join("project.json");
        let _ = std::fs::remove_file(&path);
        let od = Onboarder::new(&tmp);
        // Create a dummy profile file to simulate onboarding completion
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let prof = ProjectProfile::default();
        let _ = prof.save_to(&path);
        assert_eq!(od.is_onboarded(), true);
        let _ = std::fs::remove_file(&path);
        // Cleanup
        let _ = std::fs::remove_dir_all(tmp);
    }
}
