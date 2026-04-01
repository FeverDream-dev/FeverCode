use fever_onboard::profile::ProjectProfile;
use fever_onboard::questions::{
    all_questions, Question, QuestionBlock, QuestionOption, Validation,
};
use fever_onboard::scaffold::{GeneratedFile, ScaffoldGenerator};
use std::collections::HashMap;

#[test]
fn test_all_questions_count() {
    let qs = all_questions();
    assert_eq!(qs.len(), 21);
    assert_eq!(qs[0].id, "project_name");
    assert!(matches!(qs[0].block, QuestionBlock::Identity));
}

#[test]
fn test_profile_from_answers_roundtrip() {
    let mut answers = HashMap::new();
    answers.insert("project_name".to_string(), "demo-project".to_string());
    answers.insert(
        "description".to_string(),
        "A short demo project".to_string(),
    );
    answers.insert("end_user_type".to_string(), "Internal".to_string());
    answers.insert("current_state".to_string(), "2".to_string());
    answers.insert("primary_language".to_string(), "Rust".to_string());
    answers.insert("framework".to_string(), "Axum".to_string());
    answers.insert("database".to_string(), "PostgreSQL".to_string());
    answers.insert("frontend".to_string(), "React".to_string());
    answers.insert(
        "external_apis".to_string(),
        "https://api.example".to_string(),
    );
    answers.insert("hosting_platform".to_string(), "Railway".to_string());
    answers.insert("delivery_method".to_string(), "API".to_string());
    answers.insert("cicd_needed".to_string(), "GitHub Actions".to_string());
    answers.insert("env_vars".to_string(), "DB_HOST,DB_USER".to_string());
    answers.insert("custom_domain".to_string(), "".to_string());
    answers.insert("quality_level".to_string(), "Production".to_string());
    answers.insert("existing_tests".to_string(), "false".to_string());
    answers.insert("style_guide".to_string(), "Standard".to_string());
    answers.insert("documentation_needs".to_string(), "Docs".to_string());
    answers.insert("definition_of_done".to_string(), "Done".to_string());
    answers.insert("off_limits".to_string(), "security".to_string());
    answers.insert("urgency_level".to_string(), "Normal".to_string());

    let profile = ProjectProfile::from_answers(answers);
    assert_eq!(profile.project_name, "demo-project");
    assert_eq!(profile.primary_language, "Rust");
}

#[test]
fn test_scaffold_generation_contains_expected_files() {
    let mut answers = std::collections::HashMap::new();
    answers.insert("project_name".to_string(), "demo".to_string());
    answers.insert(
        "description".to_string(),
        "demo project for tests".to_string(),
    );
    answers.insert("hosting_platform".to_string(), "Railway".to_string());
    answers.insert("primary_language".to_string(), "Rust".to_string());
    answers.insert("cicd_needed".to_string(), "GitHub Actions".to_string());
    let profile = fever_onboard::profile::ProjectProfile::from_answers(answers);
    let scaffold = ScaffoldGenerator::new(profile);
    let files = scaffold.generate_all().unwrap();
    // Ensure a few expected files are present in the generated list
    let paths: Vec<&str> = files.iter().map(|f| f.path.as_str()).collect();
    assert!(paths.contains(&"railway.toml"));
    assert!(paths.contains(&"Dockerfile"));
    assert!(paths.iter().any(|p| p.ends_with("ci.yml")));
    assert!(paths.iter().any(|p| p.ends_with("README.md")));
}

#[test]
fn test_dockerfile_content_language_mapping() {
    let mut answers = std::collections::HashMap::new();
    answers.insert("project_name".to_string(), "demo".to_string());
    answers.insert("description".to_string(), "demo".to_string());
    answers.insert("hosting_platform".to_string(), "Railway".to_string());
    answers.insert("primary_language".to_string(), "Rust".to_string());
    let profile = fever_onboard::profile::ProjectProfile::from_answers(answers);
    let scaffold = ScaffoldGenerator::new(profile);
    let docker = scaffold.generate_dockerfile().unwrap();
    assert!(docker.content.contains("FROM"));
    assert!(docker.path == "Dockerfile");
}
