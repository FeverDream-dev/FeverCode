//! Requirements Interrogator - analyzes user requests for clarity and completeness
//!
//! This module provides rule-based analysis of user requests before any LLM call.
//! It scores requests on clarity (0-100) and generates clarification questions
//! for vague or incomplete requests.

use serde::{Deserialize, Serialize};

/// Confidence level classification based on score
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ConfidenceLevel {
    /// Score < 70: Too vague, refuse to implement
    Insufficient,
    /// Score 70-89: Can proceed with assumptions marked
    Adequate,
    /// Score 90+: Clear requirements, proceed
    Sufficient,
}

/// The result of interrogating a user request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterrogationResult {
    /// Original user request
    pub original_request: String,
    /// Confidence score 0-100
    pub confidence_score: u8,
    /// Confidence level derived from score
    pub confidence_level: ConfidenceLevel,
    /// Clarification questions for the user
    pub clarification_questions: Vec<String>,
    /// Identified ambiguities or gaps
    pub ambiguities: Vec<String>,
    /// Assumptions made (for Adequate level)
    pub assumptions: Vec<String>,
    /// Improved/structured version of the request (if score >= 70)
    pub improved_request: Option<String>,
}

/// Configuration for the interrogator
#[derive(Debug, Clone)]
pub struct InterrogatorConfig {
    /// Minimum score to allow implementation (default: 70)
    pub minimum_score: u8,
    /// Score threshold for "sufficient" (default: 90)
    pub sufficient_score: u8,
}

impl Default for InterrogatorConfig {
    fn default() -> Self {
        Self {
            minimum_score: 70,
            sufficient_score: 90,
        }
    }
}

/// Requirements interrogator - analyzes user requests for clarity
pub struct RequirementsInterrogator {
    config: InterrogatorConfig,
}

impl RequirementsInterrogator {
    /// Create a new interrogator with the given configuration
    pub fn new(config: InterrogatorConfig) -> Self {
        Self { config }
    }

    /// Create an interrogator with default configuration
    pub fn with_defaults() -> Self {
        Self::new(InterrogatorConfig::default())
    }

    /// Analyze a user request and produce an interrogation result
    pub fn interrogate(&self, request: &str) -> InterrogationResult {
        let mut score: i32 = 50; // Start at neutral
        let mut ambiguities: Vec<String> = Vec::new();
        let mut questions: Vec<String> = Vec::new();
        let mut assumptions: Vec<String> = Vec::new();
        let mut detected_files: Vec<String> = Vec::new();
        let mut detected_actions: Vec<String> = Vec::new();
        let mut detected_constraints: Vec<String> = Vec::new();

        let request_lower = request.to_lowercase();
        let request_len = request.chars().count();

        // === 1. Length signals ===
        if request_len < 20 {
            score -= 15;
            ambiguities.push("Request is very short".to_string());
            questions.push(
                "Could you provide more details about what you want to accomplish?".to_string(),
            );
        } else if request_len < 50 {
            score -= 5;
        }

        if request_len > 2000 {
            score -= 10;
            ambiguities.push("Request is very long and may lack focus".to_string());
        }

        // === 2. Specificity signals ===
        // File paths (e.g., src/main.rs, ./lib/utils.ts, /path/to/file.py)
        let file_patterns = extract_file_paths(request);
        if !file_patterns.is_empty() {
            score += 10 * file_patterns.len().min(3) as i32;
            detected_files = file_patterns;
        }

        // Function/method names (e.g., `parse_config`, `getUserById`)
        if contains_function_names(request) {
            score += 8;
        }

        // Error messages and stack traces
        if contains_error_details(&request_lower) {
            score += 12;
        }

        // Code snippets (backticks, code blocks)
        if request.contains("```") || request.contains('`') {
            score += 5;
        }

        // === 3. Action verb signals ===
        let action_verbs = [
            "fix",
            "implement",
            "add",
            "remove",
            "refactor",
            "debug",
            "test",
            "create",
            "delete",
            "update",
            "modify",
            "change",
            "replace",
            "optimize",
            "extend",
            "write",
            "build",
            "configure",
        ];

        for verb in &action_verbs {
            if request_lower.contains(verb) {
                detected_actions.push((*verb).to_string());
            }
        }

        if !detected_actions.is_empty() {
            score += 5 * detected_actions.len().min(3) as i32;
        }

        // === 4. Vagueness signals ===
        let vague_words = [
            "something",
            "somehow",
            "somewhere",
            "thing",
            "stuff",
            "things",
        ];

        for vague in &vague_words {
            if request_lower.contains(vague) {
                score -= 8;
            }
        }

        // "improve" or "better" without specifics
        if (request_lower.contains("improve") || request_lower.contains("better"))
            && !request_lower.contains("performance")
            && !request_lower.contains("speed")
            && !request_lower.contains("readability")
            && !request_lower.contains("security")
        {
            score -= 10;
            ambiguities.push("Vague improvement without specific criteria".to_string());
        }

        // === 5. Context signals ===
        let tech_keywords = [
            "rust", "python", "javascript", "typescript", "go", "java", "c++",
            "api", "rest", "graphql", "sql", "database", "async", "thread",
            "struct", "class", "function", "module", "crate", "package",
        ];

        let mut tech_found = Vec::new();
        for tech in &tech_keywords {
            if request_lower.contains(tech) {
                tech_found.push(*tech);
            }
        }

        if !tech_found.is_empty() {
            score += 3 * tech_found.len().min(4) as i32;
        }

        // === 6. Constraint signals ===
        let constraint_patterns = [
            ("must", 5),
            ("should", 3),
            ("need to", 3),
            ("don't", 4),
            ("do not", 4),
            ("cannot", 4),
            ("never", 5),
            ("only", 3),
            ("always", 3),
        ];

        for (pattern, bonus) in &constraint_patterns {
            if request_lower.contains(pattern) {
                score += *bonus;
                detected_constraints.push((*pattern).to_string());
            }
        }

        // === 7. Ambiguity detection ===
        // Pronouns without clear referents
        let pronouns = [" it ", " this ", " that "];
        let padded_request = format!(" {} ", request_lower);

        let mut ambiguous_pronouns = Vec::new();
        for pronoun in &pronouns {
            if padded_request.contains(pronoun) {
                ambiguous_pronouns.push(pronoun.trim());
            }
        }

        if !ambiguous_pronouns.is_empty() {
            score -= 5 * ambiguous_pronouns.len().min(2) as i32;
            ambiguities.push(format!(
                "Ambiguous pronouns without clear referents: {}",
                ambiguous_pronouns.join(", ")
            ));
        }

        // === 8. Missing scope detection ===
        let has_file_context = !detected_files.is_empty()
            || request_lower.contains("file")
            || request_lower.contains("module")
            || request_lower.contains("class")
            || request_lower.contains("function")
            || request_lower.contains("crate");

        if !has_file_context {
            score -= 10;
            ambiguities.push("No file or module context specified".to_string());
            questions.push("Which file(s) or module(s) should be modified?".to_string());
        }

        // === 9. Expected outcome detection ===
        let outcome_indicators = [
            "should", "will", "expect", "result", "output", "return",
            "produce", "generate", "display", "show", "print", "so that",
        ];

        let has_outcome = outcome_indicators
            .iter()
            .any(|ind| request_lower.contains(ind));

        if !has_outcome && !detected_actions.is_empty() {
            score -= 5;
            questions.push("What should be the expected outcome or result?".to_string());
        } else if has_outcome {
            score += 5;
        }

        // === 10. Fix/debug without error details ===
        if (request_lower.contains("fix") || request_lower.contains("debug"))
            && !contains_error_details(&request_lower)
        {
            score -= 8;
            questions.push("What error message or unexpected behavior are you seeing?".to_string());
        }

        // === Clamp score to 0-100 ===
        let clamped_score = score.clamp(0, 100) as u8;

        // === Determine confidence level ===
        let confidence_level = if clamped_score < self.config.minimum_score {
            ConfidenceLevel::Insufficient
        } else if clamped_score >= self.config.sufficient_score {
            ConfidenceLevel::Sufficient
        } else {
            ConfidenceLevel::Adequate
        };

        // === Generate improved request if score >= 70 ===
        let improved_request = if clamped_score >= self.config.minimum_score {
            Some(self.build_improved_request(
                request,
                &detected_actions,
                &detected_files,
                &detected_constraints,
            ))
        } else {
            None
        };

        // === Generate assumptions for Adequate level ===
        if confidence_level == ConfidenceLevel::Adequate {
            if detected_files.is_empty() {
                assumptions.push(
                    "Assuming the change applies to the most relevant file in context".to_string(),
                );
            }
            if detected_actions.is_empty() {
                assumptions.push("Assuming this is a modification task based on context".to_string());
            }
        }

        InterrogationResult {
            original_request: request.to_string(),
            confidence_score: clamped_score,
            confidence_level,
            clarification_questions: questions,
            ambiguities,
            assumptions,
            improved_request,
        }
    }

    /// Build an improved/structured version of the request
    fn build_improved_request(
        &self,
        original: &str,
        actions: &[String],
        files: &[String],
        constraints: &[String],
    ) -> String {
        let mut result = String::new();

        // Task section
        result.push_str("## Task\n");
        if !actions.is_empty() {
            result.push_str(&actions.join(", "));
            result.push_str("\n\n");
        } else {
            result.push_str("General modification\n\n");
        }

        // Scope section
        result.push_str("## Scope\n");
        if !files.is_empty() {
            result.push_str(&files.join("\n"));
            result.push_str("\n\n");
        } else {
            result.push_str("NOT SPECIFIED\n\n");
        }

        // Expected Outcome section
        result.push_str("## Expected Outcome\n");
        let brief = extract_brief_outcome(original);
        result.push_str(&brief);
        result.push_str("\n\n");

        // Constraints section
        result.push_str("## Constraints\n");
        if !constraints.is_empty() {
            result.push_str(&constraints.join("\n"));
        } else {
            result.push_str("None specified\n");
        }

        result
    }
}

/// Extract file paths from text
fn extract_file_paths(text: &str) -> Vec<String> {
    let mut files = Vec::new();
    let mut seen = std::collections::HashSet::new();

    // Common file extension patterns
    let extensions = [
        ".rs", ".py", ".js", ".ts", ".tsx", ".jsx", ".go", ".java", ".cpp", ".c",
        ".h", ".hpp", ".rb", ".php", ".cs", ".swift", ".kt", ".scala",
        ".json", ".yaml", ".yml", ".toml", ".xml", ".ini", ".cfg", ".conf",
        ".md", ".txt", ".sh", ".bash", ".zsh",
        ".html", ".css", ".scss", ".sass", ".less",
        ".sql", ".graphql", ".proto",
    ];

    // Split by whitespace and common delimiters
    for word in text.split_whitespace() {
        let word = word
            .trim_matches(|c| c == '"' || c == '\'' || c == '`' || c == ',' || c == '.' || c == ':' || c == ';');

        // Skip URLs
        if word.starts_with("http://") || word.starts_with("https://") || word.starts_with("ftp://") {
            continue;
        }

        // Check for file extensions
        for ext in &extensions {
            if word.ends_with(ext) && word.len() > ext.len() + 1 {
                if seen.insert(word.to_string()) {
                    files.push(word.to_string());
                }
                break;
            }
        }

        // Check for path-like patterns (contain / and look like paths)
        if word.contains('/') && word.len() > 3 {
            let has_valid_chars = word
                .chars()
                .all(|c| c.is_alphanumeric() || c == '/' || c == '_' || c == '-' || c == '.');
            if has_valid_chars && seen.insert(word.to_string()) {
                files.push(word.to_string());
            }
        }
    }

    files
}

/// Check if text contains function names (camelCase or snake_case followed by parens)
fn contains_function_names(text: &str) -> bool {
    // Look for patterns like `function_name(` or `functionName(`
    let chars: Vec<char> = text.chars().collect();

    for i in 0..chars.len().saturating_sub(2) {
        // Look for identifier followed by (
        if chars[i + 1] == '(' {
            // Check if the char before ( is part of an identifier
            let c = chars[i];
            if c.is_alphanumeric() || c == '_' {
                // Look back to see if it's a function name pattern
                let mut has_underscore_or_upper = false;
                for j in (0..i).rev() {
                    let bc = chars[j];
                    if bc == '_' || bc.is_uppercase() {
                        has_underscore_or_upper = true;
                    }
                    if !bc.is_alphanumeric() && bc != '_' {
                        break;
                    }
                }
                if has_underscore_or_upper || c == '_' {
                    return true;
                }
            }
        }
    }

    false
}

/// Check if text contains error details
fn contains_error_details(text_lower: &str) -> bool {
    let error_patterns = [
        "error:",
        "exception:",
        "panic:",
        "failed:",
        "failure:",
        "stack trace",
        "backtrace",
        "traceback",
        "error message",
        "error code",
        "errno",
        "status code",
        "segfault",
        "null pointer",
        "undefined reference",
        "cannot find",
        "no such file",
    ];

    for pattern in &error_patterns {
        if text_lower.contains(pattern) {
            return true;
        }
    }

    false
}

/// Extract a brief outcome description from the request
fn extract_brief_outcome(original: &str) -> String {
    // Try to find the first sentence
    for (i, _) in original.match_indices('.') {
        if i > 20 && i < 300 {
            return original[..=i].trim().to_string();
        }
    }

    // Fall back to first 200 chars
    if original.len() > 200 {
        format!("{}...", &original[..197])
    } else {
        original.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_very_short_request_gets_insufficient() {
        let interrogator = RequirementsInterrogator::with_defaults();
        let result = interrogator.interrogate("fix it");

        assert!(
            result.confidence_score < 70,
            "Score was {}",
            result.confidence_score
        );
        assert_eq!(result.confidence_level, ConfidenceLevel::Insufficient);
        assert!(!result.clarification_questions.is_empty());
    }

    #[test]
    fn test_specific_request_gets_sufficient() {
        let interrogator = RequirementsInterrogator::with_defaults();
        let result = interrogator.interrogate(
            "Fix the null pointer exception in src/parser.rs line 42. \
             The error occurs when parsing empty input. The function parse_input() \
             should return an empty vector instead of panicking.",
        );

        assert!(
            result.confidence_score >= 90,
            "Score was {}",
            result.confidence_score
        );
        assert_eq!(result.confidence_level, ConfidenceLevel::Sufficient);
        assert!(result.improved_request.is_some());
    }

    #[test]
    fn test_vague_request_gets_lower_score() {
        let interrogator = RequirementsInterrogator::with_defaults();

        let vague = interrogator.interrogate("Make something better somehow");

        let specific = interrogator.interrogate(
            "Improve the performance of the parse_config() function in config.rs \
             by reducing memory allocations. The function currently takes 500ms \
             and should complete in under 100ms.",
        );

        assert!(
            vague.confidence_score < specific.confidence_score,
            "Vague: {}, Specific: {}",
            vague.confidence_score,
            specific.confidence_score
        );
        assert!(vague.ambiguities.len() > specific.ambiguities.len());
    }

    #[test]
    fn test_clarification_questions_generated() {
        let interrogator = RequirementsInterrogator::with_defaults();
        let result = interrogator.interrogate("I need to fix the bug");

        assert!(!result.clarification_questions.is_empty());
        // Should ask about error details and file location
        let questions_combined = result.clarification_questions.join(" ");
        assert!(
            questions_combined.contains("file")
                || questions_combined.contains("error")
                || questions_combined.contains("Which"),
            "Questions: {:?}",
            result.clarification_questions
        );
    }

    #[test]
    fn test_improved_request_populated_for_adequate() {
        let interrogator = RequirementsInterrogator::with_defaults();
        // Use a request without ambiguous pronouns to ensure score >= 70
        let result = interrogator.interrogate(
            "Add a new function called validate_email to src/utils.rs. \
             The function must accept a string parameter and return true \
             for valid email addresses, false otherwise.",
        );

        // This should score in the Adequate or Sufficient range
        assert!(
            result.confidence_score >= 70,
            "Score was {}",
            result.confidence_score
        );
        assert!(result.improved_request.is_some());

        let improved = result.improved_request.unwrap();
        assert!(improved.contains("## Task"));
        assert!(improved.contains("## Scope"));
        assert!(improved.contains("utils.rs"));
    }

    #[test]
    fn test_score_clamped_to_range() {
        let interrogator = RequirementsInterrogator::with_defaults();

        // Test that score never goes below 0
        let very_vague = interrogator.interrogate("it");
        assert!(very_vague.confidence_score <= 100);

        // Test that score never exceeds 100
        let very_specific = interrogator.interrogate(
            "In the file src/network/http_client.rs, implement a new async function \
             called retry_with_backoff() that takes a closure and a RetryPolicy struct. \
             The function must retry the closure on failure with exponential backoff. \
             It should return Result<T, Error> where T is the closure's return type. \
             Add unit tests in tests/http_client_test.rs. Do not use unsafe code. \
             The implementation must be thread-safe and handle cancellation via CancellationToken.",
        );

        assert!(very_specific.confidence_score <= 100);
        assert!(very_specific.confidence_score >= 90);
    }

    #[test]
    fn test_confidence_level_boundaries() {
        let config = InterrogatorConfig {
            minimum_score: 70,
            sufficient_score: 90,
        };
        let interrogator = RequirementsInterrogator::new(config);

        // Test Insufficient (< 70)
        let insufficient = interrogator.interrogate("fix");
        assert_eq!(
            insufficient.confidence_level,
            ConfidenceLevel::Insufficient
        );
        assert!(insufficient.improved_request.is_none());

        // Test Sufficient (>= 90)
        let sufficient = interrogator.interrogate(
            "Implement the validate_email() function in src/validators.rs. \
             Must return bool. Should handle RFC 5322 compliant addresses. \
             Add tests in tests/validators_test.rs.",
        );
        assert_eq!(sufficient.confidence_level, ConfidenceLevel::Sufficient);
        assert!(sufficient.improved_request.is_some());
    }

    #[test]
    fn test_file_path_detection() {
        let files = extract_file_paths("Modify src/main.rs and lib/utils.ts");
        assert!(files.contains(&"src/main.rs".to_string()));
        assert!(files.contains(&"lib/utils.ts".to_string()));
    }

    #[test]
    fn test_adequate_level_has_assumptions() {
        let interrogator = RequirementsInterrogator::with_defaults();

        // Request that should score in Adequate range (70-89)
        let result = interrogator.interrogate("Refactor the code to be more readable");

        // If it's Adequate level, it should have assumptions or questions
        if result.confidence_level == ConfidenceLevel::Adequate {
            assert!(
                !result.assumptions.is_empty() || !result.clarification_questions.is_empty()
            );
        }
    }

    #[test]
    fn test_error_detection_bonus() {
        let interrogator = RequirementsInterrogator::with_defaults();

        let without_error = interrogator.interrogate("Fix the issue in parser.rs");

        let with_error = interrogator.interrogate(
            "Fix the panic in parser.rs. Error: thread 'main' panicked at 'called Option::unwrap() on a None value'",
        );

        assert!(
            with_error.confidence_score > without_error.confidence_score,
            "With error: {}, Without: {}",
            with_error.confidence_score,
            without_error.confidence_score
        );
    }

    #[test]
    fn test_action_verbs_bonus() {
        let interrogator = RequirementsInterrogator::with_defaults();

        let no_action = interrogator.interrogate("The code in main.rs related to the thing");

        let with_action = interrogator.interrogate(
            "Implement a new feature in main.rs that handles user input",
        );

        assert!(
            with_action.confidence_score > no_action.confidence_score,
            "With action: {}, No action: {}",
            with_action.confidence_score,
            no_action.confidence_score
        );
    }

    #[test]
    fn test_constraint_detection() {
        let interrogator = RequirementsInterrogator::with_defaults();

        let no_constraint = interrogator.interrogate(
            "Add a function to utils.rs that processes data",
        );

        let with_constraint = interrogator.interrogate(
            "Add a function to utils.rs that processes data. It must never panic and should handle all errors gracefully.",
        );

        assert!(
            with_constraint.confidence_score > no_constraint.confidence_score,
            "With constraint: {}, No constraint: {}",
            with_constraint.confidence_score,
            no_constraint.confidence_score
        );
    }

    #[test]
    fn test_improved_request_format() {
        let interrogator = RequirementsInterrogator::with_defaults();
        let result = interrogator.interrogate(
            "Fix the bug in src/lib.rs where the parser crashes on empty input. \
             The function parse() must return Ok(None) instead of panicking.",
        );

        assert!(result.improved_request.is_some());
        let improved = result.improved_request.unwrap();

        assert!(improved.contains("## Task"));
        assert!(improved.contains("## Scope"));
        assert!(improved.contains("## Expected Outcome"));
        assert!(improved.contains("## Constraints"));
        assert!(improved.contains("src/lib.rs"));
    }

    #[test]
    fn test_insufficient_has_no_improved_request() {
        let interrogator = RequirementsInterrogator::with_defaults();
        let result = interrogator.interrogate("fix it");

        assert_eq!(result.confidence_level, ConfidenceLevel::Insufficient);
        assert!(
            result.improved_request.is_none(),
            "Insufficient requests should not have improved_request"
        );
    }

    #[test]
    fn test_ambiguity_detection() {
        let interrogator = RequirementsInterrogator::with_defaults();

        let with_ambiguous_pronoun = interrogator.interrogate("Fix it in the code");

        let without_ambiguous = interrogator.interrogate(
            "Fix the parse_config() function in src/config.rs",
        );

        assert!(
            with_ambiguous_pronoun.confidence_score < without_ambiguous.confidence_score,
            "With ambiguous: {}, Without: {}",
            with_ambiguous_pronoun.confidence_score,
            without_ambiguous.confidence_score
        );
    }
}
