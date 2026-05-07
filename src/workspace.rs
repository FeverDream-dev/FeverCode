#![allow(dead_code)]
use anyhow::{Context, Result};
use ignore::WalkBuilder;
use std::{
    collections::BTreeSet,
    env,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone)]
pub struct Workspace {
    pub root: PathBuf,
    pub state_dir: PathBuf,
}

#[derive(Debug, Clone)]
pub struct WorkspaceSummary {
    pub files_seen: usize,
    pub languages: Vec<String>,
    pub has_git: bool,
    pub project_type: Vec<String>,
}

impl Workspace {
    pub fn detect(override_root: Option<PathBuf>) -> Result<Self> {
        let root = match override_root {
            Some(p) => p,
            None => env::current_dir().context("detecting current directory")?,
        };
        let root = root
            .canonicalize()
            .context("canonicalizing workspace root")?;
        Ok(Self {
            state_dir: root.join(".fevercode"),
            root,
        })
    }

    pub fn is_inside(&self, path: &Path) -> bool {
        let absolute = if path.is_absolute() {
            path.to_path_buf()
        } else {
            self.root.join(path)
        };
        let normalized = normalize_path_buf(&absolute);
        let root = normalize_path_buf(&self.root);
        normalized.starts_with(&root)
    }
}

pub fn summarize(root: &Path) -> Result<WorkspaceSummary> {
    let mut files_seen = 0usize;
    let mut langs = BTreeSet::new();
    let walker = WalkBuilder::new(root)
        .hidden(false)
        .git_ignore(true)
        .git_global(true)
        .git_exclude(true)
        .max_depth(Some(5))
        .build();

    for entry in walker
        .flatten()
        .filter(|e| e.file_type().map(|ft| ft.is_file()).unwrap_or(false))
        .take(1000)
    {
        files_seen += 1;
        if let Some(ext) = entry.path().extension().and_then(|s| s.to_str()) {
            if let Some(lang) = ext_to_lang(ext) {
                langs.insert(lang.to_string());
            }
        }
    }

    let languages = if langs.is_empty() {
        vec!["unknown".into()]
    } else {
        langs.into_iter().collect()
    };

    let has_git = root.join(".git").exists();

    let mut project_type = Vec::new();
    if root.join("Cargo.toml").exists() {
        project_type.push("Rust".to_string());
    }
    if root.join("package.json").exists() {
        project_type.push("Node.js".to_string());
    }
    if root.join("go.mod").exists() {
        project_type.push("Go".to_string());
    }
    if root.join("pyproject.toml").exists() || root.join("setup.py").exists() {
        project_type.push("Python".to_string());
    }
    if root.join("pom.xml").exists() || root.join("build.gradle").exists() {
        project_type.push("Java".to_string());
    }
    if project_type.is_empty() {
        project_type.push("unknown".to_string());
    }

    Ok(WorkspaceSummary {
        files_seen,
        languages,
        has_git,
        project_type,
    })
}

fn ext_to_lang(ext: &str) -> Option<&'static str> {
    match ext {
        "rs" => Some("Rust"),
        "ts" | "tsx" => Some("TypeScript"),
        "js" | "jsx" => Some("JavaScript"),
        "py" => Some("Python"),
        "go" => Some("Go"),
        "java" => Some("Java"),
        "kt" => Some("Kotlin"),
        "swift" => Some("Swift"),
        "rb" => Some("Ruby"),
        "php" => Some("PHP"),
        "c" | "h" => Some("C"),
        "cpp" | "hpp" | "cc" => Some("C++"),
        "toml" => Some("TOML"),
        "json" => Some("JSON"),
        "yaml" | "yml" => Some("YAML"),
        "md" => Some("Markdown"),
        _ => None,
    }
}

fn normalize_path_buf(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for component in path.components() {
        match component {
            std::path::Component::ParentDir => {
                out.pop();
            }
            std::path::Component::CurDir => {}
            other => out.push(other.as_os_str()),
        }
    }
    out
}

/// Load persistent project context files if they exist.
/// Searches for AGENTS.md, CLAUDE.md, and .cursor/rules/*.mdc in the workspace.
/// Returns concatenated content up to a reasonable size limit.
pub fn load_project_context(root: &Path) -> String {
    let mut parts = Vec::new();
    let candidates = [
        root.join(".fevercode/AGENTS.md"),
        root.join("AGENTS.md"),
        root.join("CLAUDE.md"),
        root.join(".cursor/rules/default.mdc"),
        root.join(".cursor/rules/global.mdc"),
    ];

    for path in &candidates {
        if let Ok(content) = std::fs::read_to_string(path) {
            let trimmed = content.trim();
            if !trimmed.is_empty() {
                parts.push(format!(
                    "## Project context from {}\n{}",
                    path.file_name().unwrap_or_default().to_string_lossy(),
                    trimmed
                ));
            }
        }
    }

    // Also check for any .mdc files in .cursor/rules/
    let cursor_rules = root.join(".cursor/rules");
    if cursor_rules.is_dir() {
        if let Ok(entries) = std::fs::read_dir(cursor_rules) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) == Some("mdc") {
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        let trimmed = content.trim();
                        if !trimmed.is_empty() {
                            parts.push(format!(
                                "## Cursor rule: {}\n{}",
                                path.file_name().unwrap_or_default().to_string_lossy(),
                                trimmed
                            ));
                        }
                    }
                }
            }
        }
    }

    let combined = parts.join("\n\n");
    // Cap at ~8k chars to avoid flooding small model contexts
    if combined.len() > 8192 {
        format!(
            "{}\n\n[Context truncated; total would be {} chars]",
            &combined[..8192],
            combined.len()
        )
    } else {
        combined
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn workspace_detects_current_dir() {
        let ws = Workspace::detect(None).unwrap();
        assert!(ws.root.exists());
        assert!(ws.state_dir.to_string_lossy().contains(".fevercode"));
    }

    #[test]
    fn workspace_is_inside_accepts_child() {
        let ws = Workspace::detect(None).unwrap();
        assert!(ws.is_inside(Path::new("src/main.rs")));
    }

    #[test]
    fn workspace_is_inside_rejects_escape() {
        let ws = Workspace::detect(None).unwrap();
        assert!(!ws.is_inside(Path::new("../../etc/passwd")));
    }

    #[test]
    fn summarize_rust_project() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("Cargo.toml"), "").unwrap();
        std::fs::create_dir_all(dir.path().join("src")).unwrap();
        std::fs::write(dir.path().join("src/main.rs"), "fn main() {}").unwrap();

        let summary = summarize(dir.path()).unwrap();
        assert!(summary.files_seen >= 2);
        assert!(summary.languages.contains(&"Rust".to_string()));
        assert!(summary.project_type.contains(&"Rust".to_string()));
        assert!(!summary.has_git);
    }

    #[test]
    fn summarize_detects_git() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join(".git")).unwrap();
        let summary = summarize(dir.path()).unwrap();
        assert!(summary.has_git);
    }
}
