use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Presets define how FeverCode interacts with a specific model class.
/// They control system prompts, sampling, retry behavior, and output formatting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Preset {
    /// Generic balanced preset. Works for most capable models.
    Default,
    /// Creative / vibe coding mode. Higher temperature, relaxed constraints.
    Creative,
    /// Maximum precision. Low temperature, strict tool-use formatting.
    Precise,
    /// Small local models (< 7B parameters). Heavy few-shot prompting, grammar hints.
    LocalSmall,
    /// Medium local models (7B–14B). Moderate constraints, optional grammar.
    LocalMedium,
    /// Cloud-grade strong models (Claude, GPT-4 class). Minimal constraints.
    CloudStrong,
    /// Test and research only. For llama3.2 — internet tech, tests, research agents.
    TestResearch,
    /// Vibe coder mode. Ship fast, assume intent, iterate in public.
    VibeCoder,
}

impl Default for Preset {
    fn default() -> Self {
        Preset::Default
    }
}

impl Preset {
    /// Detect the best preset for a given model name.
    /// llama3.2 is HARD-LOCKED to TestResearch regardless of override.
    pub fn detect(model_name: &str) -> Self {
        let lower = model_name.to_ascii_lowercase();

        // HARD RULE: llama3.2 is FORBIDDEN from general coding. Test/research/internet only.
        if lower.contains("llama3.2") || lower.contains("llama-3.2") {
            return Preset::TestResearch;
        }

        // Cloud-grade detection
        if lower.contains("claude")
            || lower.contains("gpt-4")
            || lower.contains("gpt-5")
            || lower.contains("gemini-2")
            || lower.contains("gemini-1.5-pro")
            || lower.contains("glm-5")
        {
            return Preset::CloudStrong;
        }

        // Medium local detection (7B–14B class)
        if lower.contains("qwen2.5")
            || lower.contains("qwen3")
            || lower.contains("llama3.1")
            || lower.contains("llama-3.1")
            || lower.contains("llama3.3")
            || lower.contains("mistral")
            || lower.contains("mixtral")
            || lower.contains("gemma2")
            || lower.contains("gemma3")
            || lower.contains("deepseek")
            || lower.contains("phi4")
        {
            return Preset::LocalMedium;
        }

        // Small local detection (< 7B) — llama3.2 already handled above
        if lower.contains("gemma")
            || lower.contains("phi3")
            || lower.contains("qwen2")
            || lower.contains("qwen2.5-0.5")
            || lower.contains("qwen2.5-1.5")
            || lower.contains("qwen2.5-3")
        {
            return Preset::LocalSmall;
        }

        Preset::Default
    }

    /// Temperature override for this preset.
    pub fn temperature(&self) -> f64 {
        match self {
            Preset::Default => 0.3,
            Preset::Creative => 0.7,
            Preset::Precise => 0.0,
            Preset::LocalSmall => 0.1,
            Preset::LocalMedium => 0.2,
            Preset::CloudStrong => 0.3,
            Preset::TestResearch => 0.15,
            Preset::VibeCoder => 0.85,
        }
    }

    /// Top-p sampling override.
    pub fn top_p(&self) -> f64 {
        match self {
            Preset::Creative | Preset::VibeCoder => 0.95,
            _ => 1.0, // Greedy or near-greedy for structured tasks
        }
    }

    /// Number of retries when a tool call is malformed.
    pub fn max_retries(&self) -> u32 {
        match self {
            Preset::LocalSmall => 3,
            Preset::LocalMedium => 2,
            Preset::Precise => 1,
            Preset::TestResearch => 2,
            Preset::VibeCoder => 1,
            _ => 1,
        }
    }

    /// Whether to add a few-shot tool-use example block to the system prompt.
    pub fn needs_few_shot(&self) -> bool {
        matches!(self, Preset::LocalSmall | Preset::LocalMedium | Preset::Precise | Preset::TestResearch)
    }

    /// Whether to request grammar-constrained JSON from the provider.
    pub fn wants_grammar_constraints(&self) -> bool {
        matches!(self, Preset::LocalSmall | Preset::LocalMedium | Preset::Precise | Preset::TestResearch)
    }

    /// Whether to prepend chain-of-thought instructions.
    pub fn wants_cot(&self) -> bool {
        matches!(self, Preset::LocalSmall | Preset::LocalMedium | Preset::TestResearch)
    }

    /// Build the full system prompt for this preset, appending tool-use rules.
    pub fn build_system_prompt(&self, base_prompt: &str) -> String {
        let mut parts = Vec::new();

        // Obedience preamble — forces the model to comply with output format
        parts.push(self.obedience_preamble().to_string());
        parts.push(String::new());
        parts.push(base_prompt.to_string());
        parts.push(String::new());
        parts.push(self.tool_use_rules().to_string());

        if self.wants_cot() {
            parts.push(String::new());
            parts.push(self.cot_instruction().to_string());
        }

        if self.needs_few_shot() {
            parts.push(String::new());
            parts.push(self.few_shot_examples().to_string());
        }

        if matches!(self, Preset::Creative | Preset::VibeCoder) {
            parts.push(String::new());
            parts.push(self.vibe_rules().to_string());
        }

        if matches!(self, Preset::TestResearch) {
            parts.push(String::new());
            parts.push(self.test_research_rules().to_string());
        }

        parts.join("\n")
    }

    /// Aggressive obedience preamble that MUST appear first in system prompt.
    /// Derived from vibe-rules-collection philosophy: the model must obey format exactly.
    fn obedience_preamble(&self) -> &'static str {
        r#"## CRITICAL — READ FIRST

You are an AI coding agent running inside FeverCode. Your ONLY job is to produce correct output.

ABSOLUTE RULES — VIOLATING ANY OF THESE IS A FAILURE:
1. When asked to perform a file or shell operation, you MUST emit a tool-call JSON object. NO prose, NO explanation, NO markdown fences.
2. The JSON MUST be exactly: {"name": "tool_name", "arguments": {"key": "value"}}
3. NEVER wrap tool calls in ```json blocks. NEVER add text before or after the JSON.
4. If you cannot perform an operation, emit ONE sentence reason — then STOP.
5. You MUST follow the user's request precisely. Do not ask clarifying questions unless the task is genuinely impossible.
6. Assume reasonable defaults. Ship working code. Perfection is the enemy of progress.
7. Your output is parsed by a machine. If you deviate from the required format, the parser will crash and the session will fail.

DISOBEDIENCE = SESSION FAILURE. OBEY THE FORMAT EXACTLY."#
    }

    /// Core tool-use formatting rules shared by all presets.
    fn tool_use_rules(&self) -> &'static str {
        r#"## Tool Use Rules — OBEY THESE EXACTLY

You have access to tools. To call a tool, you MUST output a JSON object in the following format ONLY:

{"name": "tool_name", "arguments": {"key": "value"}}

Rules:
1. When you need to read, write, edit, list, search, or run a command, you MUST use a tool. Do NOT describe what you would do.
2. The JSON MUST be valid. No markdown code blocks, no triple backticks, no preamble, no explanation before or after the JSON.
3. The tool name MUST exactly match one of the available tools.
4. All required arguments MUST be present in the arguments object.
5. If a user asks for a file operation, always use `read_file` first before proposing edits.
6. For precise edits, use `edit_file` with exact `old_string` and `new_string`. The old_string must match the file content exactly.
7. For creating new files, use `write_file`.
8. After editing, use `run_shell` to verify (e.g., `cargo check`, `cargo test`, `npm test`).
9. If you are unsure about a file path, use `list_files` to explore.
10. NEVER call a tool with arguments that would write outside the workspace.
11. Return ONLY the tool call JSON. No extra text."#
    }

    /// Chain-of-thought instruction for weaker models.
    fn cot_instruction(&self) -> &'static str {
        r#"## Thinking Process

Before calling a tool, silently think step-by-step about what needs to happen. Then output ONLY the final tool call JSON. Do NOT include your reasoning in the output."#
    }

    /// Few-shot examples for small models that struggle with tool formatting.
    fn few_shot_examples(&self) -> &'static str {
        r#"## Examples

User: Read src/main.rs
Assistant:
{"name": "read_file", "arguments": {"path": "src/main.rs"}}

User: Add a hello function to src/main.rs
Assistant:
{"name": "read_file", "arguments": {"path": "src/main.rs"}}

[Tool result shows file content]

Assistant:
{"name": "edit_file", "arguments": {"path": "src/main.rs", "old_string": "fn main() {}", "new_string": "fn hello() { println!(\"Hello!\"); }\n\nfn main() {}"}}

User: Run tests
Assistant:
{"name": "run_shell", "arguments": {"command": "cargo test"}}

User: What files are in the project?
Assistant:
{"name": "list_files", "arguments": {}}"#
    }

    /// Creative / vibe coding additional rules.
    fn vibe_rules(&self) -> &'static str {
        r#"## Vibe Mode

You are in creative vibe-coding mode. The user wants to build fast and iterate.
- Prioritize shipping over perfection.
- Make reasonable assumptions instead of asking clarifying questions.
- Keep changes minimal but don't be afraid to refactor if it makes the code cleaner.
- Use idiomatic patterns from the existing codebase.
- After edits, always run the test/lint command to verify nothing broke.
- If something fails, fix it immediately in the next tool call.
- Build the feature first. Polish later. Working > perfect.
- When in doubt, choose the simpler implementation.
- The user trusts you. Don't break that trust by being timid."#
    }

    /// Test/research-only rules for llama3.2.
    fn test_research_rules(&self) -> &'static str {
        r#"## MODEL RESTRICTION — Test & Research Only

You are running as a TEST/RESEARCH/INTERNET-TECHNOLOGY agent.
You are NOT authorized for general production coding.

Allowed tasks:
- Writing unit tests, integration tests, benchmarks
- Researching APIs, technologies, libraries on the internet
- Building research prototypes, proofs-of-concept
- Testing tool formatting and agent loop behavior
- Exploratory data analysis and experimentation

FORBIDDEN tasks:
- Production feature implementation
- Critical path code in release branches
- Security-sensitive operations (auth, crypto, payment)
- Complex refactoring without human review

If the user asks for production code, emit a one-line refusal stating this model is test-only, then STOP."#
    }

    /// Human-readable description of the preset.
    pub fn description(&self) -> &'static str {
        match self {
            Preset::Default => "Balanced preset for general-purpose models.",
            Preset::Creative => "High-temperature creative mode for vibe coding and exploration.",
            Preset::Precise => "Zero-temperature exact mode. Maximum tool-use reliability.",
            Preset::LocalSmall => "Heavy few-shot prompting for small local models (< 7B).",
            Preset::LocalMedium => "Moderate constraints for medium local models (7B–14B).",
            Preset::CloudStrong => "Minimal constraints for Claude/GPT-4 class models.",
            Preset::TestResearch => "HARD-LOCKED: llama3.2 test/research/internet-only mode.",
            Preset::VibeCoder => "Maximum creative flow. Ship fast. Iterate loud.",
        }
    }
}

/// A registry that maps model name patterns to presets.
pub struct PresetRegistry {
    overrides: HashMap<String, Preset>,
}

impl PresetRegistry {
    pub fn new() -> Self {
        Self {
            overrides: HashMap::new(),
        }
    }

    pub fn set(&mut self, model_name: impl Into<String>, preset: Preset) {
        let name = model_name.into();
        // llama3.2 cannot be overridden out of TestResearch
        let lower = name.to_ascii_lowercase();
        if lower.contains("llama3.2") || lower.contains("llama-3.2") {
            return;
        }
        self.overrides.insert(name, preset);
    }

    pub fn get(&self, model_name: &str) -> Preset {
        // llama3.2 is HARD-LOCKED
        let lower = model_name.to_ascii_lowercase();
        if lower.contains("llama3.2") || lower.contains("llama-3.2") {
            return Preset::TestResearch;
        }
        if let Some(&preset) = self.overrides.get(model_name) {
            return preset;
        }
        Preset::detect(model_name)
    }

    pub fn list_all() -> Vec<(Preset, &'static str)> {
        vec![
            (Preset::Default, "default"),
            (Preset::Creative, "creative"),
            (Preset::Precise, "precise"),
            (Preset::LocalSmall, "local_small"),
            (Preset::LocalMedium, "local_medium"),
            (Preset::CloudStrong, "cloud_strong"),
            (Preset::TestResearch, "test_research"),
            (Preset::VibeCoder, "vibe_coder"),
        ]
    }
}

impl Default for PresetRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_cloud_models() {
        assert_eq!(Preset::detect("claude-3-5-sonnet"), Preset::CloudStrong);
        assert_eq!(Preset::detect("gpt-4o"), Preset::CloudStrong);
        assert_eq!(Preset::detect("gpt-5.5-codex"), Preset::CloudStrong);
        assert_eq!(Preset::detect("glm-5.1"), Preset::CloudStrong);
    }

    #[test]
    fn detects_local_medium() {
        assert_eq!(Preset::detect("qwen2.5-coder"), Preset::LocalMedium);
        assert_eq!(Preset::detect("llama3.1:8b"), Preset::LocalMedium);
        assert_eq!(Preset::detect("deepseek-coder"), Preset::LocalMedium);
        assert_eq!(Preset::detect("mistral-7b"), Preset::LocalMedium);
    }

    #[test]
    fn detects_local_small() {
        assert_eq!(Preset::detect("gemma4:e2b"), Preset::LocalSmall);
        assert_eq!(Preset::detect("phi3"), Preset::LocalSmall);
    }

    #[test]
    fn llama32_is_hard_locked_to_test_research() {
        assert_eq!(Preset::detect("llama3.2:latest"), Preset::TestResearch);
        assert_eq!(Preset::detect("llama-3.2-vision"), Preset::TestResearch);
        assert_eq!(Preset::detect("meta-llama-3.2-1b"), Preset::TestResearch);
    }

    #[test]
    fn llama32_cannot_be_overridden() {
        let mut reg = PresetRegistry::new();
        reg.set("llama3.2:latest", Preset::CloudStrong);
        assert_eq!(reg.get("llama3.2:latest"), Preset::TestResearch);
    }

    #[test]
    fn precise_is_zero_temp() {
        assert_eq!(Preset::Precise.temperature(), 0.0);
    }

    #[test]
    fn creative_is_high_temp() {
        assert!(Preset::Creative.temperature() > Preset::Default.temperature());
    }

    #[test]
    fn vibe_coder_is_highest_temp() {
        assert!(Preset::VibeCoder.temperature() > Preset::Creative.temperature());
    }

    #[test]
    fn small_models_get_more_retries() {
        assert!(Preset::LocalSmall.max_retries() >= Preset::CloudStrong.max_retries());
    }

    #[test]
    fn system_prompt_contains_rules() {
        let prompt = Preset::Default.build_system_prompt("You are Ra.");
        assert!(prompt.contains("Tool Use Rules"));
        assert!(prompt.contains("You are Ra."));
        assert!(prompt.contains("CRITICAL"));
    }

    #[test]
    fn local_small_has_few_shot() {
        let prompt = Preset::LocalSmall.build_system_prompt("You are Ptah.");
        assert!(prompt.contains("Examples"));
        assert!(prompt.contains("read_file"));
    }

    #[test]
    fn vibe_mode_has_vibe_rules() {
        let prompt = Preset::Creative.build_system_prompt("You are Ptah.");
        assert!(prompt.contains("Vibe Mode"));
    }

    #[test]
    fn test_research_has_restriction_block() {
        let prompt = Preset::TestResearch.build_system_prompt("You are test agent.");
        assert!(prompt.contains("MODEL RESTRICTION"));
        assert!(prompt.contains("FORBIDDEN"));
    }
}
