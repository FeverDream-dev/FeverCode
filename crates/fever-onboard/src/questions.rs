/// Basic non-empty validation for string inputs.
fn validate_not_empty(s: &str) -> bool {
    !s.trim().is_empty()
}

#[allow(dead_code)]
/// Validates an input is in the range 1..=4 as a string.
fn validate_choice_1_to_4(s: &str) -> bool {
    matches!(s.trim(), "1" | "2" | "3" | "4")
}

#[allow(dead_code)]
/// Validates an input is in the range 1..=11 as a string.
fn validate_choice_1_to_11(s: &str) -> bool {
    matches!(
        s.trim(),
        "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" | "10" | "11"
    )
}

#[allow(dead_code)]
/// Validates an input is in the range 1..=5 as a string.
fn validate_choice_1_to_5(s: &str) -> bool {
    matches!(s.trim(), "1" | "2" | "3" | "4" | "5")
}

#[allow(dead_code)]
/// Validates an input is in the range 1..=3 as a string.
fn validate_choice_1_to_3(s: &str) -> bool {
    matches!(s.trim(), "1" | "2" | "3")
}

#[allow(dead_code)]
/// Validates an input is in the range 1..=5 as a string.
fn validate_choice_1_to_4_or_5(s: &str) -> bool {
    matches!(s.trim(), "1" | "2" | "3" | "4" | "5")
}

#[derive(Debug, Clone, PartialEq)]
pub struct Question {
    pub id: String,
    pub prompt: String,
    pub block: QuestionBlock,
    pub validation: Validation,
    pub default: Option<String>,
    pub options: Option<Vec<QuestionOption>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum QuestionBlock {
    Identity,
    TechStack,
    Deployment,
    Quality,
    Delivery,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Validation {
    Required,
    Optional,
    OneOf(Vec<String>),
    FilePath,
    Custom(fn(&str) -> bool),
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuestionOption {
    pub label: String,
    pub value: String,
    pub description: String,
}

pub fn all_questions() -> Vec<Question> {
    // A total of 21 questions across 5 blocks
    vec![
        Question {
            id: "project_name".to_string(),
            prompt: "What is the project name?".to_string(),
            block: QuestionBlock::Identity,
            validation: Validation::Custom(validate_not_empty),
            default: None,
            options: None,
        },
        Question {
            id: "description".to_string(),
            prompt: "Describe the project (at least 10 chars).".to_string(),
            block: QuestionBlock::Identity,
            validation: Validation::Custom(|s| s.trim().len() >= 10),
            default: None,
            options: None,
        },
        Question {
            id: "end_user_type".to_string(),
            prompt: "Who is the end user type?".to_string(),
            block: QuestionBlock::Identity,
            validation: Validation::Optional,
            default: Some("General users".to_string()),
            options: None,
        },
        Question {
            id: "current_state".to_string(),
            prompt: "Current state of onboarding (1-4)?".to_string(),
            block: QuestionBlock::Identity,
            validation: Validation::Custom(|s| match s.trim().parse::<i32>() {
                Ok(n) => (1..=4).contains(&n),
                Err(_) => false,
            }),
            default: Some("1".to_string()),
            options: None,
        },
        Question {
            id: "primary_language".to_string(),
            prompt: "Primary language (e.g., Rust, Node, Python)?".to_string(),
            block: QuestionBlock::TechStack,
            validation: Validation::Required,
            default: Some("Rust".to_string()),
            options: None,
        },
        Question {
            id: "framework".to_string(),
            prompt: "Framework (e.g., Axum, Actix, Rocket)?".to_string(),
            block: QuestionBlock::TechStack,
            validation: Validation::Optional,
            default: Some("Axum".to_string()),
            options: None,
        },
        Question {
            id: "database".to_string(),
            prompt: "Database (e.g., PostgreSQL, MySQL)?".to_string(),
            block: QuestionBlock::TechStack,
            validation: Validation::Optional,
            default: Some("PostgreSQL".to_string()),
            options: None,
        },
        Question {
            id: "frontend".to_string(),
            prompt: "Frontend (e.g., React, Vue)?".to_string(),
            block: QuestionBlock::TechStack,
            validation: Validation::Optional,
            default: Some("React".to_string()),
            options: None,
        },
        Question {
            id: "external_apis".to_string(),
            prompt: "External APIs (comma-separated)?".to_string(),
            block: QuestionBlock::TechStack,
            validation: Validation::Optional,
            default: Some("".to_string()),
            options: None,
        },
        Question {
            id: "hosting_platform".to_string(),
            prompt: "Hosting platform (Railway, Render, Fly.io, AWS etc.)?".to_string(),
            block: QuestionBlock::Deployment,
            validation: Validation::Required,
            default: Some("Railway".to_string()),
            options: None,
        },
        Question {
            id: "delivery_method".to_string(),
            prompt: "Delivery method (API, Web, Background jobs)?".to_string(),
            block: QuestionBlock::Deployment,
            validation: Validation::Optional,
            default: Some("API".to_string()),
            options: None,
        },
        Question {
            id: "cicd_needed".to_string(),
            prompt: "CI/CD needed? (None, GitHub Actions, GitLab CI)?".to_string(),
            block: QuestionBlock::Deployment,
            validation: Validation::OneOf(vec![
                "None".to_string(),
                "GitHub Actions".to_string(),
                "GitLab CI".to_string(),
            ]),
            default: Some("GitHub Actions".to_string()),
            options: None,
        },
        Question {
            id: "env_vars".to_string(),
            prompt: "Environment variables (comma-separated keys)?".to_string(),
            block: QuestionBlock::Deployment,
            validation: Validation::Optional,
            default: Some("".to_string()),
            options: None,
        },
        Question {
            id: "custom_domain".to_string(),
            prompt: "Custom domain (if any)?".to_string(),
            block: QuestionBlock::Deployment,
            validation: Validation::Optional,
            default: None,
            options: None,
        },
        Question {
            id: "quality_level".to_string(),
            prompt: "Quality level (Low, Medium, High, Production)?".to_string(),
            block: QuestionBlock::Quality,
            validation: Validation::OneOf(vec![
                "Low".to_string(),
                "Medium".to_string(),
                "High".to_string(),
                "Production".to_string(),
            ]),
            default: Some("Production".to_string()),
            options: None,
        },
        Question {
            id: "existing_tests".to_string(),
            prompt: "Existing tests? (true/false)".to_string(),
            block: QuestionBlock::Quality,
            validation: Validation::Custom(|s| {
                s.trim().eq_ignore_ascii_case("true") || s.trim().eq_ignore_ascii_case("false")
            }),
            default: Some("false".to_string()),
            options: None,
        },
        Question {
            id: "style_guide".to_string(),
            prompt: "Style guide to follow?".to_string(),
            block: QuestionBlock::Quality,
            validation: Validation::Optional,
            default: Some("Standard".to_string()),
            options: None,
        },
        Question {
            id: "documentation_needs".to_string(),
            prompt: "Documentation needs?".to_string(),
            block: QuestionBlock::Quality,
            validation: Validation::Optional,
            default: Some("Docs in repo".to_string()),
            options: None,
        },
        Question {
            id: "definition_of_done".to_string(),
            prompt: "Definition of done?".to_string(),
            block: QuestionBlock::Delivery,
            validation: Validation::Optional,
            default: Some("Feature complete".to_string()),
            options: None,
        },
        Question {
            id: "off_limits".to_string(),
            prompt: "Off-limits areas? (comma-separated)".to_string(),
            block: QuestionBlock::Delivery,
            validation: Validation::Optional,
            default: Some("security".to_string()),
            options: None,
        },
        Question {
            id: "urgency_level".to_string(),
            prompt: "Urgency level (Low, Normal, High)?".to_string(),
            block: QuestionBlock::Delivery,
            validation: Validation::OneOf(vec![
                "Low".to_string(),
                "Normal".to_string(),
                "High".to_string(),
            ]),
            default: Some("Normal".to_string()),
            options: None,
        },
    ]
}
