use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs, path::Path};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoulsConfig {
    pub version: u32,
    #[serde(default)]
    pub context: ContextConfig,
    #[serde(default)]
    pub safety: SoulsSafetyConfig,
    #[serde(default)]
    pub souls: HashMap<String, SoulEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextConfig {
    #[serde(default = "default_max_lines")]
    pub max_tool_output_lines: usize,
    #[serde(default = "default_true")]
    pub prefer_scripted_analysis: bool,
    #[serde(default = "default_true")]
    pub summarize_large_outputs: bool,
    #[serde(default = "default_true")]
    pub store_session_events: bool,
}

impl Default for ContextConfig {
    fn default() -> Self {
        Self {
            max_tool_output_lines: default_max_lines(),
            prefer_scripted_analysis: true,
            summarize_large_outputs: true,
            store_session_events: true,
        }
    }
}

fn default_max_lines() -> usize {
    200
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoulsSafetyConfig {
    #[serde(default = "default_true")]
    pub workspace_only: bool,
    #[serde(default = "default_true")]
    pub deny_path_traversal: bool,
    #[serde(default = "default_true")]
    pub deny_absolute_outside_workspace: bool,
    #[serde(default = "default_true")]
    pub spray_mode_workspace_only: bool,
}

impl Default for SoulsSafetyConfig {
    fn default() -> Self {
        Self {
            workspace_only: true,
            deny_path_traversal: true,
            deny_absolute_outside_workspace: true,
            spray_mode_workspace_only: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoulEntry {
    pub title: String,
    #[serde(default)]
    pub style: String,
    #[serde(default)]
    pub risk: String,
    #[serde(default)]
    pub responsibilities: Vec<String>,
    #[serde(default)]
    pub allowed_tools: Vec<String>,
    #[serde(default)]
    pub escalation_rules: Vec<String>,
    #[serde(default)]
    pub context_budget: Option<u32>,
}

pub struct BuiltinSoul {
    pub id: &'static str,
    pub name: &'static str,
    pub egyptian_name: &'static str,
    pub title: &'static str,
    pub style: &'static str,
    pub risk: &'static str,
    pub responsibilities: &'static [&'static str],
}

pub fn builtin_souls() -> Vec<BuiltinSoul> {
    vec![
        BuiltinSoul {
            id: "ra",
            name: "ra-planner",
            egyptian_name: "Ra",
            title: "Planner",
            style: "structured, concise",
            risk: "low",
            responsibilities: &["plan", "clarify", "decompose", "define_done"],
        },
        BuiltinSoul {
            id: "thoth",
            name: "thoth-architect",
            egyptian_name: "Thoth",
            title: "Architect",
            style: "precise, system-level",
            risk: "medium",
            responsibilities: &["design", "module_boundaries", "provider_abstractions"],
        },
        BuiltinSoul {
            id: "ptah",
            name: "ptah-builder",
            egyptian_name: "Ptah",
            title: "Builder",
            style: "implementation-focused",
            risk: "medium",
            responsibilities: &["edit", "patch", "refactor", "compile"],
        },
        BuiltinSoul {
            id: "maat",
            name: "maat-checker",
            egyptian_name: "Maat",
            title: "Checker",
            style: "strict, evidence-based",
            risk: "low",
            responsibilities: &["test", "lint", "verify_docs", "audit_claims"],
        },
        BuiltinSoul {
            id: "anubis",
            name: "anubis-guardian",
            egyptian_name: "Anubis",
            title: "Guardian",
            style: "safety-first",
            risk: "high",
            responsibilities: &["permissions", "path_guard", "danger_review"],
        },
        BuiltinSoul {
            id: "seshat",
            name: "seshat-docs",
            egyptian_name: "Seshat",
            title: "Chronicler",
            style: "clear, user-facing",
            risk: "low",
            responsibilities: &["readme", "docs", "changelog", "session_summary"],
        },
    ]
}

pub fn find_builtin(id: &str) -> Option<BuiltinSoul> {
    builtin_souls()
        .into_iter()
        .find(|s| s.id == id || s.name == id)
}

impl SoulsConfig {
    pub fn load(dir: &Path) -> Result<Option<Self>> {
        let path = dir.join(".fevercode/souls.toml");
        if !path.exists() {
            return Ok(None);
        }
        let raw =
            fs::read_to_string(&path).with_context(|| format!("reading {}", path.display()))?;
        let cfg = toml::from_str(&raw).with_context(|| format!("parsing {}", path.display()))?;
        Ok(Some(cfg))
    }

    pub fn load_or_default(dir: &Path) -> Result<Self> {
        match Self::load(dir)? {
            Some(cfg) => Ok(cfg),
            None => Ok(Self::default()),
        }
    }

    pub fn save(dir: &Path) -> Result<()> {
        let cfg = Self::default();
        let state = dir.join(".fevercode");
        fs::create_dir_all(&state)?;
        let path = state.join("souls.toml");
        fs::write(&path, toml::to_string_pretty(&cfg)?)?;
        println!("Created {}", path.display());
        Ok(())
    }
}

impl Default for SoulsConfig {
    fn default() -> Self {
        let mut souls = HashMap::new();

        souls.insert(
            "ra".into(),
            SoulEntry {
                title: "Planner".into(),
                style: "structured, concise".into(),
                risk: "low".into(),
                responsibilities: vec![
                    "plan".into(),
                    "clarify".into(),
                    "decompose".into(),
                    "define_done".into(),
                ],
                allowed_tools: vec![
                    "read_file".into(),
                    "list_files".into(),
                    "search_text".into(),
                ],
                escalation_rules: vec!["Ask user if goal is ambiguous".into()],
                context_budget: Some(2000),
            },
        );

        souls.insert(
            "thoth".into(),
            SoulEntry {
                title: "Architect".into(),
                style: "precise, system-level".into(),
                risk: "medium".into(),
                responsibilities: vec![
                    "design".into(),
                    "module_boundaries".into(),
                    "provider_abstractions".into(),
                ],
                allowed_tools: vec![
                    "read_file".into(),
                    "list_files".into(),
                    "search_text".into(),
                    "git_status".into(),
                    "git_diff".into(),
                ],
                escalation_rules: vec!["Escalate breaking changes to user".into()],
                context_budget: Some(3000),
            },
        );

        souls.insert(
            "ptah".into(),
            SoulEntry {
                title: "Builder".into(),
                style: "implementation-focused".into(),
                risk: "medium".into(),
                responsibilities: vec![
                    "edit".into(),
                    "patch".into(),
                    "refactor".into(),
                    "compile".into(),
                ],
                allowed_tools: vec![
                    "read_file".into(),
                    "write_file".into(),
                    "run_shell".into(),
                    "git_checkpoint".into(),
                ],
                escalation_rules: vec![
                    "Ask before deleting files".into(),
                    "Ask before modifying config".into(),
                ],
                context_budget: Some(4000),
            },
        );

        souls.insert(
            "maat".into(),
            SoulEntry {
                title: "Checker".into(),
                style: "strict, evidence-based".into(),
                risk: "low".into(),
                responsibilities: vec![
                    "test".into(),
                    "lint".into(),
                    "verify_docs".into(),
                    "audit_claims".into(),
                ],
                allowed_tools: vec!["run_shell".into(), "read_file".into(), "search_text".into()],
                escalation_rules: vec!["Report all failures, never suppress".into()],
                context_budget: Some(2000),
            },
        );

        souls.insert(
            "anubis".into(),
            SoulEntry {
                title: "Guardian".into(),
                style: "safety-first".into(),
                risk: "high".into(),
                responsibilities: vec![
                    "permissions".into(),
                    "path_guard".into(),
                    "danger_review".into(),
                ],
                allowed_tools: vec!["read_file".into(), "list_files".into()],
                escalation_rules: vec![
                    "Block all outside-workspace writes".into(),
                    "Block credential access unless explicitly requested".into(),
                ],
                context_budget: Some(1000),
            },
        );

        souls.insert(
            "seshat".into(),
            SoulEntry {
                title: "Chronicler".into(),
                style: "clear, user-facing".into(),
                risk: "low".into(),
                responsibilities: vec![
                    "readme".into(),
                    "docs".into(),
                    "changelog".into(),
                    "session_summary".into(),
                ],
                allowed_tools: vec![
                    "read_file".into(),
                    "write_file".into(),
                    "search_text".into(),
                ],
                escalation_rules: vec!["Mark planned features clearly".into()],
                context_budget: Some(2000),
            },
        );

        Self {
            version: 1,
            context: ContextConfig::default(),
            safety: SoulsSafetyConfig::default(),
            souls,
        }
    }
}

pub fn init_souls_file(dir: &Path) -> Result<()> {
    let path = dir.join(".fevercode/souls.toml");
    if path.exists() {
        println!(
            "{} already exists. Use --force to overwrite.",
            path.display()
        );
        return Ok(());
    }
    SoulsConfig::save(dir)
}

pub fn init_souls_md(dir: &Path) -> Result<()> {
    let path = dir.join("SOULS.md");
    if path.exists() {
        println!("{} already exists.", path.display());
        return Ok(());
    }
    let content = include_str!("../SOULS.md");
    fs::write(&path, content)?;
    println!("Created {}", path.display());
    Ok(())
}

pub fn list_souls(cfg: Option<&SoulsConfig>) {
    println!("FeverCode Souls:");
    println!();

    for soul in builtin_souls() {
        let config_info = cfg.and_then(|c| c.souls.get(soul.id));
        let config_tag = if config_info.is_some() {
            " [config loaded]"
        } else {
            ""
        };
        println!(
            "  {} ({}) — {} — risk: {}{}",
            soul.egyptian_name, soul.name, soul.title, soul.risk, config_tag
        );
        let responsibilities: Vec<&str> = soul.responsibilities.to_vec();
        println!("    responsibilities: {}", responsibilities.join(", "));
        println!();
    }
}

pub fn show_soul(name: &str, cfg: Option<&SoulsConfig>) -> Result<()> {
    let soul = find_builtin(name).ok_or_else(|| {
        anyhow::anyhow!(
            "Unknown soul: '{}'. Available: ra, thoth, ptah, maat, anubis, seshat",
            name
        )
    })?;

    println!("{} — {} ({})", soul.egyptian_name, soul.title, soul.name);
    println!("Style: {}", soul.style);
    println!("Risk: {}", soul.risk);
    println!("Responsibilities: {}", soul.responsibilities.join(", "));

    if let Some(c) = cfg.and_then(|c| c.souls.get(soul.id)) {
        println!();
        println!("Config override:");
        if !c.allowed_tools.is_empty() {
            println!("  Allowed tools: {}", c.allowed_tools.join(", "));
        }
        if !c.escalation_rules.is_empty() {
            println!("  Escalation rules:");
            for rule in &c.escalation_rules {
                println!("    - {}", rule);
            }
        }
        if let Some(budget) = c.context_budget {
            println!("  Context budget: {} tokens", budget);
        }
    }

    Ok(())
}

pub fn validate_souls(dir: &Path) -> Result<()> {
    let mut issues = Vec::new();

    let souls_md = dir.join("SOULS.md");
    if souls_md.exists() {
        println!("SOULS.md: found");
    } else {
        issues.push("SOULS.md: missing (run 'fever souls init' to create)");
    }

    match SoulsConfig::load(dir)? {
        Some(cfg) => {
            println!("souls.toml: found (version {})", cfg.version);
            println!(
                "  context max_output_lines: {}",
                cfg.context.max_tool_output_lines
            );
            println!("  workspace_only: {}", cfg.safety.workspace_only);
            println!("  souls configured: {}", cfg.souls.len());

            for soul in builtin_souls() {
                if cfg.souls.contains_key(soul.id) {
                    println!("  {} — config present", soul.egyptian_name);
                } else {
                    println!(
                        "  {} — using built-in defaults (no config override)",
                        soul.egyptian_name
                    );
                }
            }
        }
        None => {
            issues.push("souls.toml: missing (run 'fever souls init' to create)");
        }
    }

    if issues.is_empty() {
        println!("\nAll checks passed.");
    } else {
        println!("\nIssues:");
        for issue in &issues {
            println!("  - {}", issue);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_souls_config_roundtrips() {
        let cfg = SoulsConfig::default();
        let toml_str = toml::to_string_pretty(&cfg).unwrap();
        let parsed: SoulsConfig = toml::from_str(&toml_str).unwrap();
        assert_eq!(parsed.version, 1);
        assert_eq!(parsed.souls.len(), 6);
        assert!(parsed.context.store_session_events);
        assert!(parsed.safety.workspace_only);
    }

    #[test]
    fn default_config_has_all_souls() {
        let cfg = SoulsConfig::default();
        let ids = vec!["ra", "thoth", "ptah", "maat", "anubis", "seshat"];
        for id in ids {
            assert!(cfg.souls.contains_key(id), "missing soul: {}", id);
        }
    }

    #[test]
    fn builtin_souls_count() {
        assert_eq!(builtin_souls().len(), 6);
    }

    #[test]
    fn find_builtin_by_id() {
        assert!(find_builtin("ra").is_some());
        assert!(find_builtin("ptah").is_some());
        assert!(find_builtin("nonexistent").is_none());
    }

    #[test]
    fn find_builtin_by_name() {
        assert!(find_builtin("ra-planner").is_some());
        assert!(find_builtin("thoth-architect").is_some());
    }

    #[test]
    fn missing_souls_config_returns_none() {
        let dir = tempfile::tempdir().unwrap();
        let result = SoulsConfig::load(dir.path()).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn load_or_default_returns_default_when_missing() {
        let dir = tempfile::tempdir().unwrap();
        let cfg = SoulsConfig::load_or_default(dir.path()).unwrap();
        assert_eq!(cfg.version, 1);
    }

    #[test]
    fn init_does_not_overwrite() {
        let dir = tempfile::tempdir().unwrap();
        let state = dir.path().join(".fevercode");
        fs::create_dir_all(&state).unwrap();
        fs::write(state.join("souls.toml"), "version = 99").unwrap();
        init_souls_file(dir.path()).unwrap();
        let content = fs::read_to_string(state.join("souls.toml")).unwrap();
        assert_eq!(content, "version = 99");
    }

    #[test]
    fn soul_entry_has_fields() {
        let cfg = SoulsConfig::default();
        let ra = cfg.souls.get("ra").unwrap();
        assert_eq!(ra.title, "Planner");
        assert_eq!(ra.risk, "low");
        assert!(!ra.responsibilities.is_empty());
    }

    #[test]
    fn invalid_soul_name_error() {
        let result = show_soul("nonexistent", None);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Unknown soul"));
    }
}
