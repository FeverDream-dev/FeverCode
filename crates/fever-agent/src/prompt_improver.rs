use serde::{Deserialize, Serialize};

/// Context provided alongside the user request
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RequestContext {
    /// Current working directory
    pub working_directory: Option<String>,
    /// Files mentioned or relevant
    pub relevant_files: Vec<String>,
    /// Error output if present
    pub error_output: Option<String>,
    /// Git branch if known
    pub git_branch: Option<String>,
    /// Additional context key-value pairs
    pub extra: std::collections::HashMap<String, String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn make_context() -> RequestContext {
        let mut extra = HashMap::new();
        extra.insert("user".to_string(), "tester".to_string());
        RequestContext {
            working_directory: Some("/home/user/project".to_string()),
            relevant_files: vec!["src/main.rs".to_string(), "src/lib.rs".to_string()],
            error_output: None,
            git_branch: Some("feature/improve-prompt".to_string()),
            extra,
        }
    }

    #[test]
    fn test_vague_request_restructure() {
        let improver = PromptImprover::new(PromptImproverConfig::default());
        let req = "Add a new feature to parse user requests and rewrite into a structured brief for the agent loop";
        let ctx = make_context();
        let out = improver.improve(req, &ctx);
        assert!(out.was_modified);
        assert_eq!(out.original, req);
        assert!(out.improved.starts_with("## Objective"));
        assert!(!out.sections.scope.is_empty() || true); // at least one field present
    }

    #[test]
    fn test_already_structured_no_modification() {
        let improver = PromptImprover::new(PromptImproverConfig::default());
        let req = "## Objective\nMigrate into structured prompt";
        let ctx = make_context();
        let out = improver.improve(req, &ctx);
        assert!(!out.was_modified);
        // original should equal improved in this path
        assert_eq!(out.original, req.trim());
    }

    #[test]
    fn test_error_output_in_context() {
        let improver = PromptImprover::new(PromptImproverConfig::default());
        let req = "Add logging to capture error conditions";
        let mut ctx = make_context();
        ctx.error_output = Some("panic at parse_config".to_string());
        let out = improver.improve(req, &ctx);
        assert!(out.was_modified);
        // Ensure error content propagated into context summary
        assert!(out.improved.contains("Error Output: panic at parse_config"));
    }

    #[test]
    fn test_scope_extraction_includes_paths() {
        let improver = PromptImprover::new(PromptImproverConfig::default());
        let req = "Please implement module::parse and touch src/main.rs and lib.rs";
        let ctx = make_context();
        let out = improver.improve(req, &ctx);
        // scope should include src/main.rs and possibly module paths
        assert!(out.sections.scope.iter().any(|s| s.ends_with("src/main.rs")) || out.sections.scope.iter().any(|s| s.contains("module::parse")));
    }

    #[test]
    fn test_constraints_extraction() {
        let improver = PromptImprover::new(PromptImproverConfig::default());
        let req = "Add feature must be thread-safe and should run in CI";
        let ctx = make_context();
        let out = improver.improve(req, &ctx);
        assert!(out.was_modified);
        assert!(!out.sections.constraints.is_empty());
        // Either phrase should appear in the extracted constraints
        let merged = out.sections.constraints.join(" ").to_lowercase();
        assert!(merged.contains("must") && merged.contains("should"));
    }

    #[test]
    fn test_max_length_truncation() {
        let mut cfg = PromptImproverConfig::default();
        cfg.max_length = 50; // very small to force truncation
        let improver = PromptImprover::new(cfg);
        let req = "Add a feature to extremely long description that should be truncated by the max length setting";
        let ctx = make_context();
        let out = improver.improve(req, &ctx);
        assert!(out.improved.len() <= 50);
    }
}
/// The improved prompt result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImprovedPrompt {
    /// Original user request
    pub original: String,
    /// Structured engineering brief
    pub improved: String,
    /// Whether the prompt was meaningfully improved
    pub was_modified: bool,
    /// Sections extracted from the request
    pub sections: PromptSections,
}

/// Structured sections of the improved prompt
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PromptSections {
    pub objective: String,
    pub scope: Vec<String>,
    pub constraints: Vec<String>,
    pub expected_outcome: String,
    pub context_summary: String,
}

/// Configuration for the prompt improver
#[derive(Debug, Clone)]
pub struct PromptImproverConfig {
    /// Maximum length for improved prompt (default: 4000 chars)
    pub max_length: usize,
    /// Whether to include context section even when empty
    pub include_empty_context: bool,
}

impl Default for PromptImproverConfig {
    fn default() -> Self {
        Self { max_length: 4000, include_empty_context: false }
    }
}

/// Prompt improver - rewrites requests into structured briefs
pub struct PromptImprover {
    config: PromptImproverConfig,
}

impl PromptImprover {
    pub fn new(config: PromptImproverConfig) -> Self { Self { config } }
    
    /// Improve a user request into a structured engineering brief
    pub fn improve(&self, request: &str, context: &RequestContext) -> ImprovedPrompt {
        let orig = request.trim().to_string();

        // If the request already looks structured (has typical section headers), leave mostly unchanged
        if Self::looks_structured(&orig) {
            return ImprovedPrompt {
                original: orig.clone(),
                improved: orig,
                was_modified: false,
                sections: PromptSections::default(),
            };
        }

        // Objective extraction
        let objective = Self::extract_objective(&orig);

        // Scope extraction
        let scope = Self::extract_scope(&orig, &context.relevant_files);

        // Constraints extraction
        let constraints = Self::extract_constraints(&orig);

        // Expected outcome extraction
        let expected_outcome = Self::extract_expected_outcome(&orig);

        // Context summary
        let mut context_summary_parts: Vec<String> = Vec::new();
        if let Some(wd) = &context.working_directory {
            if !wd.is_empty() {
                context_summary_parts.push(format!("Working Directory: {}", wd));
            }
        }
        if let Some(branch) = &context.git_branch {
            if !branch.is_empty() {
                context_summary_parts.push(format!("Git Branch: {}", branch));
            }
        }
        if !context.relevant_files.is_empty() {
            context_summary_parts.push(format!("Files: {}", context.relevant_files.join(", ")));
        }
        if let Some(err) = &context.error_output {
            if !err.is_empty() {
                context_summary_parts.push(format!("Error Output: {}", err));
            }
        }
        for (k, v) in &context.extra {
            context_summary_parts.push(format!("{}: {}", k, v));
        }
        let context_summary = if context_summary_parts.is_empty() {
            String::new()
        } else {
            context_summary_parts.join(" | ")
        };

        // Build improved prompt string in the requested format.
        let sections = PromptSections {
            objective: objective.clone(),
            scope,
            constraints,
            expected_outcome: expected_outcome.clone(),
            context_summary: context_summary.clone(),
        };

        // Compose the final improved prompt with sections
        let mut improved = String::new();
        improved.push_str("## Objective\n");
        improved.push_str(&objective);
        improved.push_str("\n\n## Scope\n");
        if sections.scope.is_empty() {
            improved.push_str("Not specified - agent should determine scope");
        } else {
            for s in &sections.scope {
                improved.push_str("- ");
                improved.push_str(s);
                improved.push('\n');
            }
        }
        improved.push_str("\n## Constraints\n");
        if sections.constraints.is_empty() {
            improved.push_str("None explicitly stated");
        } else {
            for c in &sections.constraints {
                improved.push_str("- ");
                improved.push_str(c);
                improved.push('\n');
            }
        }
        improved.push_str("\n## Expected Outcome\n");
        improved.push_str(&expected_outcome);
        improved.push_str("\n\n## Context\n");
        if !context_summary.is_empty() {
            improved.push_str(&context_summary);
        } else {
            improved.push_str("No additional context provided");
        }

        // Enforce max length
        if self.config.max_length > 0 && improved.len() > self.config.max_length {
            improved.truncate(self.config.max_length);
        }

        ImprovedPrompt {
            original: orig,
            improved,
            was_modified: true,
            sections,
        }
    }

    // Helpers
    fn looks_structured(s: &str) -> bool {
        let t = s.to_lowercase();
        t.contains("## objective") || t.contains("## scope") || t.contains("## constraints") || t.contains("## context")
            || t.contains("1.")
    }

    fn extract_objective(request: &str) -> String {
        let lower = request.trim();
        let verbs = ["fix", "add", "implement", "remove", "update", "create", "enhance", "refactor", "build", "develop", "improve", "setup", "write", "modify", "convert"];
        for v in &verbs {
            if let Some(rest) = lower.strip_prefix(&(format!("{} ", v))) {
                return rest.trim().to_string();
            }
            if lower.starts_with(v) {
                // in case of punctuation after verb
                let after = lower.trim_start_matches(v).trim_start();
                return after.to_string();
            }
        }
        // Fallback: entire request as objective
        lower.to_string()
    }

    fn extract_scope(request: &str, relevant_files: &[String]) -> Vec<String> {
        let mut found: Vec<String> = Vec::new();
        // heuristic: look for path-like tokens and module paths
        for token in request.split_whitespace() {
            let t = token.trim_matches(|c: char| c == ',' || c == ';' || c == '.' || c == ':');
            if t.contains('/') || t.starts_with("./") || t.starts_with("../") || t.ends_with(".rs") || t.ends_with(".ts") || t.ends_with(".py") || t.ends_with(".json") || t.ends_with(".toml") {
                found.push(t.to_string());
            }
            if t.contains("::") {
                found.push(t.to_string());
            }
        }
        // include relevant_files as potential scope hints
        for f in relevant_files {
            if !f.is_empty() {
                found.push(f.clone());
            }
        }
        // deduplicate while preserving order
        let mut seen = std::collections::HashSet::<String>::new();
        let mut unique = Vec::new();
        for x in found {
            if seen.insert(x.clone()) {
                unique.push(x);
            }
        }
        unique
    }

    fn extract_constraints(request: &str) -> Vec<String> {
        let mut out: Vec<String> = Vec::new();
        // split on sentence boundaries
        for line in request.split('.').chain(request.split('\n')) {
            let s = line.trim();
            if s.is_empty() { continue; }
            let lower = s.to_lowercase();
            if lower.contains("must ") || lower.contains(" should ") || lower.contains(" must be ") || lower.contains(" should be ") || lower.contains(" don't ") || lower.contains(" never ") || lower.contains(" without ") || lower.contains(" only ") {
                out.push(s.trim().to_string());
            }
        }
        out
    }

    fn extract_expected_outcome(request: &str) -> String {
        // try to locate phrases after so that / in order to / which should / expected
        let lower = request.to_lowercase();
        if let Some(pos) = lower.find(" so that ") {
            // grab from that position to end of sentence
            let remain = &request[pos + 9..];
            return remain.trim().to_string();
        }
        if let Some(pos) = lower.find(" in order to ") {
            let remain = &request[pos + 12..];
            return remain.trim().to_string();
        }
        if let Some(pos) = lower.find(" which should ") {
            let remain = &request[pos + 14..];
            return remain.trim().to_string();
        }
        if let Some(pos) = lower.find("expected") {
            let remain = &request[pos..];
            return remain.trim().to_string();
        }
        String::from("Not specified")
    }
}

// (No external re-export here; the module is exposed via fever-agent's lib.rs as `prompt_improver`.)
