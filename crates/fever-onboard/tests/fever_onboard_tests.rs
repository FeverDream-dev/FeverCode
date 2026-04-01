use fever_onboard::profile::ProjectProfile;
use fever_onboard::questions::{QuestionBlock, all_questions};
use fever_onboard::scaffold::ScaffoldGenerator;
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
fn test_deployment_config_aws_generates_aws_tf() {
    let mut answers = std::collections::HashMap::new();
    answers.insert("project_name".to_string(), "demo-aws".to_string());
    answers.insert("description".to_string(), "Demo for AWS".to_string());
    answers.insert("hosting_platform".to_string(), "AWS".to_string());
    answers.insert("primary_language".to_string(), "Rust".to_string());
    let profile = fever_onboard::profile::ProjectProfile::from_answers(answers);
    let scaffold = ScaffoldGenerator::new(profile);
    let dep = scaffold.generate_deployment_config().expect("aws file");
    assert_eq!(dep.path, "aws.tf");
    assert!(dep.content.contains("aws_instance"));
    assert!(dep.content.contains("aws_lb"));
}

#[test]
fn test_deployment_config_gcp_generates_cloudbuild_yaml() {
    let mut answers = std::collections::HashMap::new();
    answers.insert("project_name".to_string(), "demo-gcp".to_string());
    answers.insert("description".to_string(), "Demo for GCP".to_string());
    answers.insert("hosting_platform".to_string(), "GCP".to_string());
    answers.insert("primary_language".to_string(), "Rust".to_string());
    let profile = fever_onboard::profile::ProjectProfile::from_answers(answers);
    let scaffold = ScaffoldGenerator::new(profile);
    let dep = scaffold
        .generate_deployment_config()
        .expect("cloudbuild.yaml");
    assert_eq!(dep.path, "cloudbuild.yaml");
    assert!(dep.content.contains("steps:"));
    assert!(dep.content.contains("gcr.io"));
}

#[test]
fn test_deployment_config_digitalocean_generates_do_app_yaml() {
    let mut answers = std::collections::HashMap::new();
    answers.insert("project_name".to_string(), "demo-do".to_string());
    answers.insert("description".to_string(), "Demo for DO".to_string());
    answers.insert("hosting_platform".to_string(), "DigitalOcean".to_string());
    answers.insert("primary_language".to_string(), "Rust".to_string());
    let profile = fever_onboard::profile::ProjectProfile::from_answers(answers);
    let scaffold = ScaffoldGenerator::new(profile);
    let dep = scaffold.generate_deployment_config().expect("do-app.yaml");
    assert_eq!(dep.path, "do-app.yaml");
    assert!(dep.content.contains("name:"));
}

#[test]
fn test_deployment_config_vps_generates_deploy_sh() {
    let mut answers = std::collections::HashMap::new();
    answers.insert("project_name".to_string(), "demo-vps".to_string());
    answers.insert("description".to_string(), "Demo for VPS".to_string());
    answers.insert("hosting_platform".to_string(), "VPS".to_string());
    answers.insert("primary_language".to_string(), "Rust".to_string());
    let profile = fever_onboard::profile::ProjectProfile::from_answers(answers);
    let scaffold = ScaffoldGenerator::new(profile);
    let dep = scaffold.generate_deployment_config().expect("deploy.sh");
    assert_eq!(dep.path, "deploy.sh");
    assert!(dep.content.contains("#!/bin/bash"));
}

#[test]
fn test_deployment_config_docker_generates_docker_compose() {
    let mut answers = std::collections::HashMap::new();
    answers.insert("project_name".to_string(), "demo-docker".to_string());
    answers.insert("description".to_string(), "Docker Compose".to_string());
    answers.insert("hosting_platform".to_string(), "Docker".to_string());
    answers.insert("primary_language".to_string(), "Rust".to_string());
    let profile = fever_onboard::profile::ProjectProfile::from_answers(answers);
    let scaffold = ScaffoldGenerator::new(profile);
    let dep = scaffold
        .generate_deployment_config()
        .expect("docker-compose.yml");
    assert_eq!(dep.path, "docker-compose.yml");
    assert!(dep.content.contains("version: \"3.8\""));
}

#[test]
fn test_ci_cd_yaml_is_real_yaml() {
    let mut answers = std::collections::HashMap::new();
    answers.insert("project_name".to_string(), "demo-ci".to_string());
    answers.insert("description".to_string(), "CI/CD test".to_string());
    answers.insert("hosting_platform".to_string(), "Railway".to_string());
    answers.insert("primary_language".to_string(), "Rust".to_string());
    answers.insert("cicd_needed".to_string(), "GitHub Actions".to_string());
    let profile = fever_onboard::profile::ProjectProfile::from_answers(answers);
    let scaffold = ScaffoldGenerator::new(profile);
    let ci_parts = scaffold.generate_ci_cd();
    assert_eq!(ci_parts.len(), 2);
    let ci_yaml = &ci_parts[0].content;
    assert!(ci_yaml.contains("runs-on"));
    assert!(ci_yaml.contains("steps"));
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
