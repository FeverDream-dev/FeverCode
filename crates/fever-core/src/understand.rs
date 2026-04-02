//! Project understanding module - scans a repository and produces a structured summary
//! that can be injected into agent system prompts.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// A structured summary of a project, produced by scanning the repository.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectSummary {
    /// Detected programming languages with file counts
    pub languages: Vec<LanguageInfo>,
    /// Detected build system
    pub build_system: Option<BuildSystem>,
    /// Detected test framework
    pub test_framework: Option<String>,
    /// Detected test commands
    pub test_commands: Vec<String>,
    /// Build commands
    pub build_commands: Vec<String>,
    /// Likely entrypoints (main.rs, index.ts, etc.)
    pub entrypoints: Vec<String>,
    /// Key configuration files found
    pub config_files: Vec<String>,
    /// Framework detection (if any)
    pub frameworks: Vec<String>,
    /// Total file count
    pub file_count: usize,
    /// Total directory count
    pub dir_count: usize,
    /// Top-level directory structure (names only)
    pub top_dirs: Vec<String>,
    /// Package manager detected
    pub package_manager: Option<String>,
    /// Brief text summary suitable for injecting into a prompt
    pub summary_text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageInfo {
    pub name: String,
    pub extensions: Vec<String>,
    pub file_count: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BuildSystem {
    Cargo,
    Npm,
    Yarn,
    Pnpm,
    Make,
    Cmake,
    Gradle,
    Maven,
    Pip,
    GoModules,
    None,
}

/// Directories to skip during scanning
const SKIP_DIRS: &[&str] = &[
    ".git",
    "target",
    "node_modules",
    "__pycache__",
    ".venv",
    "venv",
    "dist",
    "build",
    ".next",
    ".nuxt",
];

/// Maximum file size to read (1MB)
const MAX_FILE_SIZE: u64 = 1_000_000;

pub struct ProjectUnderstanding {
    root: PathBuf,
}

impl ProjectUnderstanding {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    /// Scan the repository and produce a structured summary.
    pub async fn analyze(&self) -> Result<ProjectSummary> {
        let mut languages: HashMap<String, (Vec<String>, usize)> = HashMap::new();
        let mut build_system: Option<BuildSystem> = None;
        let mut test_framework: Option<String> = None;
        let mut test_commands: Vec<String> = Vec::new();
        let mut build_commands: Vec<String> = Vec::new();
        let mut entrypoints: Vec<String> = Vec::new();
        let mut config_files: Vec<String> = Vec::new();
        let mut frameworks: Vec<String> = Vec::new();
        let mut file_count: usize = 0;
        let mut dir_count: usize = 0;
        let mut top_dirs: Vec<String> = Vec::new();
        let mut package_manager: Option<String> = None;
        let mut has_yarn_lock = false;
        let mut has_pnpm_lock = false;

        if let Ok(entries) = fs::read_dir(&self.root) {
            for entry in entries.flatten() {
                if let Ok(file_type) = entry.file_type() {
                    let name = entry.file_name().to_string_lossy().to_string();
                    if file_type.is_dir() && !name.starts_with('.') {
                        top_dirs.push(name);
                    }
                }
            }
        }
        top_dirs.sort();

        // Walk the directory tree
        self.walk_dir(
            &self.root,
            &mut languages,
            &mut build_system,
            &mut test_framework,
            &mut test_commands,
            &mut build_commands,
            &mut entrypoints,
            &mut config_files,
            &mut frameworks,
            &mut file_count,
            &mut dir_count,
            &mut package_manager,
            &mut has_yarn_lock,
            &mut has_pnpm_lock,
        )?;

        // Refine build system based on lock files
        if build_system == Some(BuildSystem::Npm) {
            if has_pnpm_lock {
                build_system = Some(BuildSystem::Pnpm);
            } else if has_yarn_lock {
                build_system = Some(BuildSystem::Yarn);
            }
        }

        // Set commands based on build system
        if test_commands.is_empty() {
            test_commands = Self::detect_test_commands(&build_system);
        }
        if build_commands.is_empty() {
            build_commands = Self::detect_build_commands(&build_system);
        }

        // Set package manager based on build system
        if package_manager.is_none() {
            package_manager = Self::detect_package_manager(&build_system);
        }

        // Convert language map to sorted vec
        let mut languages_vec: Vec<LanguageInfo> = languages
            .into_iter()
            .map(|(name, (extensions, count))| LanguageInfo {
                name,
                extensions,
                file_count: count,
            })
            .collect();
        languages_vec.sort_by(|a, b| b.file_count.cmp(&a.file_count));

        // Generate summary text
        let summary_text = Self::generate_summary_text(
            &languages_vec,
            &build_system,
            &entrypoints,
            &frameworks,
            file_count,
            dir_count,
        );

        // Deduplicate and sort
        frameworks.sort();
        frameworks.dedup();
        entrypoints.sort();
        entrypoints.dedup();
        config_files.sort();
        config_files.dedup();

        Ok(ProjectSummary {
            languages: languages_vec,
            build_system,
            test_framework,
            test_commands,
            build_commands,
            entrypoints,
            config_files,
            frameworks,
            file_count,
            dir_count,
            top_dirs,
            package_manager,
            summary_text,
        })
    }

    fn walk_dir(
        &self,
        dir: &Path,
        languages: &mut HashMap<String, (Vec<String>, usize)>,
        build_system: &mut Option<BuildSystem>,
        test_framework: &mut Option<String>,
        test_commands: &mut Vec<String>,
        build_commands: &mut Vec<String>,
        entrypoints: &mut Vec<String>,
        config_files: &mut Vec<String>,
        frameworks: &mut Vec<String>,
        file_count: &mut usize,
        dir_count: &mut usize,
        package_manager: &mut Option<String>,
        has_yarn_lock: &mut bool,
        has_pnpm_lock: &mut bool,
    ) -> Result<()> {
        let entries = fs::read_dir(dir)?;

        for entry in entries.flatten() {
            let path = entry.path();
            let file_name = entry.file_name().to_string_lossy().to_string();

            // Skip hidden files and directories
            if file_name.starts_with('.') && file_name != ".env" && file_name != ".env.example" {
                continue;
            }

            let file_type = entry.file_type()?;

            if file_type.is_dir() {
                // Skip certain directories
                if SKIP_DIRS.contains(&file_name.as_str()) {
                    continue;
                }

                *dir_count += 1;

                // Recurse
                self.walk_dir(
                    &path,
                    languages,
                    build_system,
                    test_framework,
                    test_commands,
                    build_commands,
                    entrypoints,
                    config_files,
                    frameworks,
                    file_count,
                    dir_count,
                    package_manager,
                    has_yarn_lock,
                    has_pnpm_lock,
                )?;
            } else if file_type.is_file() {
                *file_count += 1;

                // Check file size
                if let Ok(metadata) = fs::metadata(&path) {
                    if metadata.len() > MAX_FILE_SIZE {
                        continue;
                    }
                }

                // Detect language by extension
                if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                    let ext_lower = ext.to_lowercase();
                    if let Some((lang_name, _extensions)) = Self::detect_language(&ext_lower) {
                        let entry = languages.entry(lang_name).or_insert((vec![], 0));
                        if !entry.0.contains(&ext_lower) {
                            entry.0.push(ext_lower.clone());
                        }
                        entry.1 += 1;
                    }
                }

                // Check for build system files
                let relative_path = path.strip_prefix(&self.root).unwrap_or(&path);
                let relative_str = relative_path.to_string_lossy();

                // Build system detection
                match file_name.as_str() {
                    "Cargo.toml" => {
                        *build_system = Some(BuildSystem::Cargo);
                        *package_manager = Some("cargo".to_string());

                        // Read file for framework detection
                        if let Ok(content) = fs::read_to_string(&path) {
                            Self::detect_rust_frameworks(&content, frameworks);
                        }
                    }
                    "package.json" => {
                        *build_system = Some(BuildSystem::Npm);
                        *package_manager = Some("npm".to_string());

                        // Read file for framework detection
                        if let Ok(content) = fs::read_to_string(&path) {
                            Self::detect_js_frameworks(&content, frameworks, test_framework);
                        }
                    }
                    "yarn.lock" => {
                        *has_yarn_lock = true;
                        *package_manager = Some("yarn".to_string());
                    }
                    "pnpm-lock.yaml" => {
                        *has_pnpm_lock = true;
                        *package_manager = Some("pnpm".to_string());
                    }
                    "Makefile" => {
                        *build_system = Some(BuildSystem::Make);
                    }
                    "CMakeLists.txt" => {
                        *build_system = Some(BuildSystem::Cmake);
                    }
                    "build.gradle" | "settings.gradle" => {
                        *build_system = Some(BuildSystem::Gradle);
                    }
                    "pom.xml" => {
                        *build_system = Some(BuildSystem::Maven);
                    }
                    "go.mod" => {
                        *build_system = Some(BuildSystem::GoModules);
                        *package_manager = Some("go".to_string());
                    }
                    "requirements.txt" => {
                        if build_system.is_none() {
                            *build_system = Some(BuildSystem::Pip);
                            *package_manager = Some("pip".to_string());
                        }
                    }
                    "pyproject.toml" => {
                        if let Ok(content) = fs::read_to_string(&path) {
                            let has_poetry = content.contains("[tool.poetry]");
                            let has_build_system = content.contains("[build-system]");
                            if has_poetry || has_build_system {
                                if build_system.is_none() {
                                    *build_system = Some(BuildSystem::Pip);
                                    *package_manager = Some("pip".to_string());
                                }
                            }
                            Self::detect_python_frameworks(&content, frameworks);
                        }
                    }
                    _ => {}
                }

                // Check for entrypoints
                if Self::is_entrypoint(&file_name) {
                    entrypoints.push(relative_str.to_string());
                }

                // Check for config files
                if Self::is_config_file(&file_name) {
                    config_files.push(relative_str.to_string());
                }
            }
        }

        Ok(())
    }

    fn detect_language(ext: &str) -> Option<(String, Vec<String>)> {
        match ext {
            "rs" => Some(("Rust".to_string(), vec!["rs".to_string()])),
            "ts" | "tsx" => Some((
                "TypeScript".to_string(),
                vec!["ts".to_string(), "tsx".to_string()],
            )),
            "js" | "jsx" => Some((
                "JavaScript".to_string(),
                vec!["js".to_string(), "jsx".to_string()],
            )),
            "py" => Some(("Python".to_string(), vec!["py".to_string()])),
            "go" => Some(("Go".to_string(), vec!["go".to_string()])),
            "java" => Some(("Java".to_string(), vec!["java".to_string()])),
            "rb" => Some(("Ruby".to_string(), vec!["rb".to_string()])),
            "cpp" | "cc" | "cxx" | "hpp" => Some((
                "C++".to_string(),
                vec![
                    "cpp".to_string(),
                    "cc".to_string(),
                    "cxx".to_string(),
                    "hpp".to_string(),
                ],
            )),
            "c" | "h" => Some(("C".to_string(), vec!["c".to_string(), "h".to_string()])),
            "swift" => Some(("Swift".to_string(), vec!["swift".to_string()])),
            "kt" | "kts" => Some((
                "Kotlin".to_string(),
                vec!["kt".to_string(), "kts".to_string()],
            )),
            _ => None,
        }
    }

    fn detect_rust_frameworks(content: &str, frameworks: &mut Vec<String>) {
        if content.contains("actix") {
            frameworks.push("Actix Web".to_string());
        }
        if content.contains("axum") {
            frameworks.push("Axum".to_string());
        }
        if content.contains("rocket") {
            frameworks.push("Rocket".to_string());
        }
        if content.contains("warp") {
            frameworks.push("Warp".to_string());
        }
        if content.contains("tokio") {
            frameworks.push("tokio".to_string());
        }
        if content.contains("ratatui") {
            frameworks.push("ratatui".to_string());
        }
        if content.contains("crossterm") {
            frameworks.push("crossterm".to_string());
        }
    }

    fn detect_js_frameworks(
        content: &str,
        frameworks: &mut Vec<String>,
        test_framework: &mut Option<String>,
    ) {
        // Parse JSON to check dependencies
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(content) {
            if let Some(deps) = json.get("dependencies").and_then(|d| d.as_object()) {
                let dep_names: Vec<&str> = deps.keys().map(|s| s.as_str()).collect();

                if dep_names.iter().any(|d| *d == "react") {
                    frameworks.push("React".to_string());
                }
                if dep_names.iter().any(|d| *d == "vue") {
                    frameworks.push("Vue".to_string());
                }
                if dep_names.iter().any(|d| d.starts_with("@angular")) {
                    frameworks.push("Angular".to_string());
                }
                if dep_names.iter().any(|d| *d == "next") {
                    frameworks.push("Next.js".to_string());
                }
                if dep_names.iter().any(|d| *d == "express") {
                    frameworks.push("Express".to_string());
                }
                if dep_names.iter().any(|d| *d == "fastify") {
                    frameworks.push("Fastify".to_string());
                }
            }

            // Check devDependencies for test framework
            if let Some(dev_deps) = json.get("devDependencies").and_then(|d| d.as_object()) {
                let dep_names: Vec<&str> = dev_deps.keys().map(|s| s.as_str()).collect();

                if dep_names.iter().any(|d| *d == "jest") {
                    *test_framework = Some("jest".to_string());
                }
                if dep_names.iter().any(|d| *d == "vitest") {
                    *test_framework = Some("vitest".to_string());
                }
                if dep_names.iter().any(|d| *d == "mocha") {
                    *test_framework = Some("mocha".to_string());
                }
            }
        }
    }

    fn detect_python_frameworks(content: &str, frameworks: &mut Vec<String>) {
        if content.contains("django") {
            frameworks.push("Django".to_string());
        }
        if content.contains("flask") {
            frameworks.push("Flask".to_string());
        }
        if content.contains("fastapi") {
            frameworks.push("FastAPI".to_string());
        }
        if content.contains("pytest") {
            // Could be detected from pyproject.toml dependencies
        }
    }

    fn is_entrypoint(file_name: &str) -> bool {
        matches!(
            file_name,
            "main.rs"
                | "main.ts"
                | "main.tsx"
                | "main.go"
                | "main.py"
                | "index.ts"
                | "index.tsx"
                | "index.js"
                | "app.ts"
                | "app.tsx"
                | "app.js"
                | "lib.rs"
        ) || file_name.ends_with("/main.rs")
            || file_name.ends_with("/main.ts")
            || file_name.ends_with("/index.ts")
            || file_name.ends_with("/lib.rs")
    }

    fn is_config_file(file_name: &str) -> bool {
        matches!(
            file_name,
            ".env"
                | ".env.example"
                | "config.toml"
                | "config.yaml"
                | "config.json"
                | "tsconfig.json"
                | "jest.config.js"
                | "jest.config.ts"
                | "pyproject.toml"
                | ".eslintrc"
                | ".eslintrc.js"
                | ".eslintrc.json"
                | ".prettierrc"
                | ".prettierrc.js"
                | ".prettierrc.json"
        )
    }

    fn detect_test_commands(build_system: &Option<BuildSystem>) -> Vec<String> {
        match build_system {
            Some(BuildSystem::Cargo) => vec!["cargo test".to_string()],
            Some(BuildSystem::Npm) | Some(BuildSystem::Yarn) | Some(BuildSystem::Pnpm) => {
                vec!["npm test".to_string()]
            }
            Some(BuildSystem::Pip) => vec!["pytest".to_string(), "python -m pytest".to_string()],
            Some(BuildSystem::GoModules) => vec!["go test ./...".to_string()],
            Some(BuildSystem::Make) => vec!["make test".to_string()],
            _ => vec![],
        }
    }

    fn detect_build_commands(build_system: &Option<BuildSystem>) -> Vec<String> {
        match build_system {
            Some(BuildSystem::Cargo) => vec!["cargo build".to_string()],
            Some(BuildSystem::Npm) | Some(BuildSystem::Yarn) | Some(BuildSystem::Pnpm) => {
                vec!["npm run build".to_string()]
            }
            Some(BuildSystem::GoModules) => vec!["go build ./...".to_string()],
            Some(BuildSystem::Make) => vec!["make".to_string()],
            _ => vec![],
        }
    }

    fn detect_package_manager(build_system: &Option<BuildSystem>) -> Option<String> {
        match build_system {
            Some(BuildSystem::Cargo) => Some("cargo".to_string()),
            Some(BuildSystem::Npm) => Some("npm".to_string()),
            Some(BuildSystem::Yarn) => Some("yarn".to_string()),
            Some(BuildSystem::Pnpm) => Some("pnpm".to_string()),
            Some(BuildSystem::Pip) => Some("pip".to_string()),
            Some(BuildSystem::GoModules) => Some("go".to_string()),
            Some(BuildSystem::Gradle) => Some("gradle".to_string()),
            Some(BuildSystem::Maven) => Some("maven".to_string()),
            _ => None,
        }
    }

    fn generate_summary_text(
        languages: &[LanguageInfo],
        build_system: &Option<BuildSystem>,
        entrypoints: &[String],
        frameworks: &[String],
        file_count: usize,
        dir_count: usize,
    ) -> String {
        let primary_lang = languages
            .first()
            .map(|l| l.name.as_str())
            .unwrap_or("Unknown");
        let primary_count = languages.first().map(|l| l.file_count).unwrap_or(0);

        let build_str = match build_system {
            Some(BuildSystem::Cargo) => "Cargo (cargo build / cargo test)",
            Some(BuildSystem::Npm) => "Npm (npm run build / npm test)",
            Some(BuildSystem::Yarn) => "Yarn (yarn build / yarn test)",
            Some(BuildSystem::Pnpm) => "Pnpm (pnpm build / pnpm test)",
            Some(BuildSystem::GoModules) => "Go modules (go build / go test)",
            Some(BuildSystem::Make) => "Make (make / make test)",
            Some(BuildSystem::Pip) => "Pip (python setup)",
            Some(BuildSystem::Gradle) => "Gradle (gradle build)",
            Some(BuildSystem::Maven) => "Maven (mvn build)",
            Some(BuildSystem::Cmake) => "CMake (cmake / make)",
            Some(BuildSystem::None) | None => "None detected",
        };

        let entry_str = if entrypoints.is_empty() {
            "Not detected".to_string()
        } else {
            entrypoints.join(", ")
        };

        let framework_str = if frameworks.is_empty() {
            "None".to_string()
        } else {
            frameworks.join(", ")
        };

        format!(
            "This is a {} project. Primary language: {} ({} files). Build system: {}. Entrypoints: {}. Key frameworks: {}. The project has {} files across {} directories.",
            primary_lang,
            primary_lang,
            primary_count,
            build_str,
            entry_str,
            framework_str,
            file_count,
            dir_count
        )
    }

    /// Generate a concise text block suitable for injecting into an LLM system prompt.
    pub fn prompt_context(summary: &ProjectSummary) -> String {
        let mut context = String::new();

        context.push_str("## Project Context\n\n");

        // Languages
        if !summary.languages.is_empty() {
            context.push_str("**Languages:**\n");
            for lang in &summary.languages {
                context.push_str(&format!("- {} ({} files)\n", lang.name, lang.file_count));
            }
            context.push('\n');
        }

        // Build system
        if let Some(bs) = &summary.build_system {
            let bs_name = match bs {
                BuildSystem::Cargo => "Cargo",
                BuildSystem::Npm => "NPM",
                BuildSystem::Yarn => "Yarn",
                BuildSystem::Pnpm => "PNPM",
                BuildSystem::Make => "Make",
                BuildSystem::Cmake => "CMake",
                BuildSystem::Gradle => "Gradle",
                BuildSystem::Maven => "Maven",
                BuildSystem::Pip => "Pip",
                BuildSystem::GoModules => "Go Modules",
                BuildSystem::None => "None",
            };
            context.push_str(&format!("**Build System:** {}\n\n", bs_name));
        }

        // Commands
        if !summary.build_commands.is_empty() {
            context.push_str(&format!(
                "**Build Commands:** {}\n\n",
                summary.build_commands.join(", ")
            ));
        }
        if !summary.test_commands.is_empty() {
            context.push_str(&format!(
                "**Test Commands:** {}\n\n",
                summary.test_commands.join(", ")
            ));
        }

        // Entrypoints
        if !summary.entrypoints.is_empty() {
            context.push_str(&format!(
                "**Entrypoints:** {}\n\n",
                summary.entrypoints.join(", ")
            ));
        }

        // Frameworks
        if !summary.frameworks.is_empty() {
            context.push_str(&format!(
                "**Frameworks:** {}\n\n",
                summary.frameworks.join(", ")
            ));
        }

        // Summary
        context.push_str("**Summary:**\n");
        context.push_str(&summary.summary_text);

        context
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use std::io::Write;
    use std::path::PathBuf;

    fn create_temp_dir() -> PathBuf {
        let temp = std::env::temp_dir().join(format!("fever_test_{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&temp).expect("Failed to create temp dir");
        temp
    }

    fn cleanup(temp: &Path) {
        let _ = fs::remove_dir_all(temp);
    }

    #[tokio::test]
    async fn test_language_detection() {
        let temp = create_temp_dir();

        // Create files with known extensions
        File::create(temp.join("main.rs")).expect("Failed to create file");
        File::create(temp.join("lib.ts")).expect("Failed to create file");
        File::create(temp.join("app.py")).expect("Failed to create file");
        File::create(temp.join("main.go")).expect("Failed to create file");

        let understanding = ProjectUnderstanding::new(temp.clone());
        let summary = understanding.analyze().await.expect("Analysis failed");

        assert!(
            summary
                .languages
                .iter()
                .any(|l| l.name == "Rust" && l.file_count == 1)
        );
        assert!(
            summary
                .languages
                .iter()
                .any(|l| l.name == "TypeScript" && l.file_count == 1)
        );
        assert!(
            summary
                .languages
                .iter()
                .any(|l| l.name == "Python" && l.file_count == 1)
        );
        assert!(
            summary
                .languages
                .iter()
                .any(|l| l.name == "Go" && l.file_count == 1)
        );

        cleanup(&temp);
    }

    #[tokio::test]
    async fn test_build_system_detection() {
        let temp = create_temp_dir();

        // Create a Cargo.toml file
        let cargo_content = r#"[package]
name = "test"
version = "0.1.0"

[dependencies]
axum = "0.7"
"#;
        let mut file = File::create(temp.join("Cargo.toml")).expect("Failed to create file");
        file.write_all(cargo_content.as_bytes())
            .expect("Failed to write");

        let understanding = ProjectUnderstanding::new(temp.clone());
        let summary = understanding.analyze().await.expect("Analysis failed");

        assert_eq!(summary.build_system, Some(BuildSystem::Cargo));
        assert!(summary.frameworks.contains(&"Axum".to_string()));

        cleanup(&temp);
    }

    #[tokio::test]
    async fn test_prompt_context_nonempty() {
        // Use fever-core directory itself
        let fever_core_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let understanding = ProjectUnderstanding::new(fever_core_path);
        let summary = understanding.analyze().await.expect("Analysis failed");

        let context = ProjectUnderstanding::prompt_context(&summary);

        assert!(!context.is_empty());
        assert!(context.contains("Rust"));
        assert!(context.contains("Cargo"));
    }

    #[tokio::test]
    async fn test_skips_hidden_dirs() {
        let temp = create_temp_dir();

        // Create files in .git and target (should be skipped)
        let git_dir = temp.join(".git");
        let target_dir = temp.join("target");
        fs::create_dir_all(&git_dir).expect("Failed to create .git");
        fs::create_dir_all(&target_dir).expect("Failed to create target");

        File::create(git_dir.join("config")).expect("Failed to create file");
        File::create(target_dir.join("main.rs")).expect("Failed to create file");

        // Create a file that should be counted
        File::create(temp.join("actual_file.rs")).expect("Failed to create file");

        let understanding = ProjectUnderstanding::new(temp.clone());
        let summary = understanding.analyze().await.expect("Analysis failed");

        // Should only count the actual_file.rs
        assert_eq!(summary.file_count, 1);
        assert!(
            summary
                .languages
                .iter()
                .any(|l| l.name == "Rust" && l.file_count == 1)
        );

        cleanup(&temp);
    }

    #[tokio::test]
    async fn test_js_framework_detection() {
        let temp = create_temp_dir();

        // Create a package.json with React
        let package_json = r#"{
            "dependencies": {
                "react": "^18.0.0",
                "next": "^14.0.0"
            },
            "devDependencies": {
                "jest": "^29.0.0"
            }
        }"#;
        let mut file = File::create(temp.join("package.json")).expect("Failed to create file");
        file.write_all(package_json.as_bytes())
            .expect("Failed to write");

        let understanding = ProjectUnderstanding::new(temp.clone());
        let summary = understanding.analyze().await.expect("Analysis failed");

        assert!(summary.frameworks.contains(&"React".to_string()));
        assert!(summary.frameworks.contains(&"Next.js".to_string()));
        assert_eq!(summary.test_framework, Some("jest".to_string()));

        cleanup(&temp);
    }

    #[tokio::test]
    async fn test_entrypoint_detection() {
        let temp = create_temp_dir();

        // Create entrypoint files
        fs::create_dir_all(temp.join("src")).expect("Failed to create src");
        File::create(temp.join("src/main.rs")).expect("Failed to create file");
        File::create(temp.join("lib.rs")).expect("Failed to create file");

        let understanding = ProjectUnderstanding::new(temp.clone());
        let summary = understanding.analyze().await.expect("Analysis failed");

        assert!(summary.entrypoints.len() >= 2);
        assert!(summary.entrypoints.iter().any(|e| e.contains("main.rs")));
        assert!(summary.entrypoints.iter().any(|e| e.contains("lib.rs")));

        cleanup(&temp);
    }
}
