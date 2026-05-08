use anyhow::Result;
use serde_json::Value;
use std::path::PathBuf;

use super::ToolResult;

pub struct TestRunnerTool { workspace_root: PathBuf }
impl TestRunnerTool { pub fn new(r: PathBuf) -> Self { Self { workspace_root: r } } }
impl super::Tool for TestRunnerTool {
    fn name(&self) -> &str { "run_tests" }
    fn execute(&self, args: Value) -> Result<ToolResult> {
        let framework = args["framework"].as_str().unwrap_or("auto");
        let filter = args["filter"].as_str().unwrap_or("");
        let cmd = match framework {
            "cargo" | "rust" => format!("cargo test {} 2>&1", filter),
            "npm" | "node" | "jest" => format!("npm test {} 2>&1", if filter.is_empty() { "".to_string() } else { format!("-- --testPathPattern={}", filter) }),
            "pytest" | "python" => format!("pytest {} -v 2>&1", if filter.is_empty() { "".to_string() } else { format!("-k {}", filter) }),
            "go" => format!("go test {} -v 2>&1", if filter.is_empty() { "./..." } else { filter }),
            "auto" => {
                if self.workspace_root.join("Cargo.toml").exists() { format!("cargo test {} 2>&1", filter) }
                else if self.workspace_root.join("package.json").exists() { "npm test 2>&1".to_string() }
                else if self.workspace_root.join("go.mod").exists() { "go test ./... -v 2>&1".to_string() }
                else if self.workspace_root.join("pytest.ini").exists() || self.workspace_root.join("pyproject.toml").exists() { "pytest -v 2>&1".to_string() }
                else { return Ok(ToolResult::err("no test framework detected")); }
            }
            _ => return Ok(ToolResult::err("frameworks: cargo, npm, pytest, go, auto")),
        };
        let out = std::process::Command::new("sh").arg("-c").arg(&cmd).current_dir(&self.workspace_root).output();
        match out {
            Ok(o) => {
                let stdout = String::from_utf8_lossy(&o.stdout);
                let truncated = if stdout.len() > 5000 { format!("{}...(truncated)", &stdout[..5000]) } else { stdout.to_string() };
                Ok(ToolResult { output: truncated, success: o.status.success() })
            }
            Err(e) => Ok(ToolResult::err(format!("test run failed: {}", e))),
        }
    }
}

pub struct CoverageReportTool { workspace_root: PathBuf }
impl CoverageReportTool { pub fn new(r: PathBuf) -> Self { Self { workspace_root: r } } }
impl super::Tool for CoverageReportTool {
    fn name(&self) -> &str { "coverage_report" }
    fn execute(&self, _args: Value) -> Result<ToolResult> {
        let cmd = if self.workspace_root.join("Cargo.toml").exists() {
            "cargo tarpaulin --out Stdout 2>&1 || echo 'tarpaulin not installed, run: cargo install cargo-tarpaulin'"
        } else if self.workspace_root.join("package.json").exists() {
            "npx nyc --reporter=text npm test 2>&1 || echo 'nyc not available'"
        } else {
            "echo 'coverage not supported for this project type'"
        };
        let out = std::process::Command::new("sh").arg("-c").arg(cmd).current_dir(&self.workspace_root).output();
        match out {
            Ok(o) => Ok(ToolResult::ok(String::from_utf8_lossy(&o.stdout).to_string())),
            Err(e) => Ok(ToolResult::err(format!("coverage failed: {}", e))),
        }
    }
}

pub struct ComplexityAnalyzerTool { workspace_root: PathBuf }
impl ComplexityAnalyzerTool { pub fn new(r: PathBuf) -> Self { Self { workspace_root: r } } }
impl super::Tool for ComplexityAnalyzerTool {
    fn name(&self) -> &str { "complexity_analysis" }
    fn execute(&self, args: Value) -> Result<ToolResult> {
        let path = args["path"].as_str().unwrap_or(".");
        let root = self.workspace_root.join(path);
        if !root.exists() { return Ok(ToolResult::err(format!("not found: {}", path))); }
        let content = if root.is_dir() {
            let mut s = String::new();
            for entry in walkdir::WalkDir::new(&root).max_depth(5).into_iter().filter_map(|e| e.ok()) {
                if entry.file_type().is_file() {
                    if let Ok(c) = std::fs::read_to_string(entry.path()) { s.push_str(&c); s.push('\n'); }
                }
            }
            s
        } else { std::fs::read_to_string(&root)? };
        let lines = content.lines().count();
        let functions = content.matches("fn ").count() + content.matches("function ").count() + content.matches("def ").count() + content.matches("func ").count();
        let classes = content.matches("struct ").count() + content.matches("class ").count() + content.matches("impl ").count();
        let comments = content.lines().filter(|l| l.trim().starts_with("//") || l.trim().starts_with("#") || l.trim().starts_with("/*")).count();
        let avg_fn_len = if functions > 0 { lines / functions } else { 0 };
        let result = format!(
            "Lines: {}\nFunctions: {}\nClasses/Structs: {}\nComments: {}\nComment ratio: {:.1}%\nAvg function length: {} lines\nComplexity estimate: {}",
            lines, functions, classes, comments,
            if lines > 0 { (comments as f64 / lines as f64) * 100.0 } else { 0.0 },
            avg_fn_len,
            if avg_fn_len > 50 { "HIGH — consider refactoring" } else if avg_fn_len > 25 { "MODERATE" } else { "LOW — good" }
        );
        Ok(ToolResult::ok(result))
    }
}

pub struct SecurityScanTool { workspace_root: PathBuf }
impl SecurityScanTool { pub fn new(r: PathBuf) -> Self { Self { workspace_root: r } } }
impl super::Tool for SecurityScanTool {
    fn name(&self) -> &str { "security_scan" }
    fn execute(&self, _args: Value) -> Result<ToolResult> {
        let patterns = [
            ("API key exposed", "api_key\\s*[:=]\\s*['\"][^'\"]{8,}['\"]"),
            ("Password in source", "password\\s*[:=]\\s*['\"][^'\"]{4,}['\"]"),
            ("Secret in source", "secret\\s*[:=]\\s*['\"][^'\"]{8,}['\"]"),
            ("Private key", "-----BEGIN (RSA |EC )?PRIVATE KEY-----"),
            ("Hardcoded token", "token\\s*[:=]\\s*['\"][^'\"]{16,}['\"]"),
            ("AWS key pattern", "AKIA[0-9A-Z]{16}"),
            ("eval() usage", "eval\\("),
            ("SQL injection risk", "format!\\(.*SELECT.*\\+"),
            ("unsafe block", "unsafe\\s*\\{"),
        ];
        let mut findings = Vec::new();
        use ignore::WalkBuilder;
        let walker = WalkBuilder::new(&self.workspace_root).hidden(false).git_ignore(true).build();
        for entry in walker.flatten() {
            if entry.file_type().map(|ft| ft.is_file()).unwrap_or(false) {
                let ext = entry.path().extension().and_then(|e| e.to_str()).unwrap_or("");
                if ["png","jpg","gif","bin","exe","so","dylib","lock","min.js","min.css"].contains(&ext) { continue; }
                if let Ok(content) = std::fs::read_to_string(entry.path()) {
                    for (name, pattern) in &patterns {
                        if let Ok(re) = regex::Regex::new(pattern) {
                            for (i, line) in content.lines().enumerate() {
                                if re.is_match(line) {
                                    if let Ok(rel) = entry.path().strip_prefix(&self.workspace_root) {
                                        findings.push(format!("[!] {}:{}: {}", rel.display(), i + 1, name));
                                    }
                                    if findings.len() >= 50 { break; }
                                }
                            }
                        }
                        if findings.len() >= 50 { break; }
                    }
                }
                if findings.len() >= 50 { break; }
            }
        }
        if findings.is_empty() { Ok(ToolResult::ok("no obvious security issues found")) }
        else { Ok(ToolResult::ok(format!("Security findings:\n{}", findings.join("\n")))) }
    }
}

pub struct DeadCodeFinderTool { workspace_root: PathBuf }
impl DeadCodeFinderTool { pub fn new(r: PathBuf) -> Self { Self { workspace_root: r } } }
impl super::Tool for DeadCodeFinderTool {
    fn name(&self) -> &str { "find_dead_code" }
    fn execute(&self, _args: Value) -> Result<ToolResult> {
        if self.workspace_root.join("Cargo.toml").exists() {
            let out = std::process::Command::new("cargo")
                .args(["+nightly", "udeps", "--all-targets", "2>&1"])
                .current_dir(&self.workspace_root).output();
            match out {
                Ok(o) => Ok(ToolResult::ok(String::from_utf8_lossy(&o.stdout).to_string())),
                Err(_) => Ok(ToolResult::ok("cargo-udeps not available. Install: cargo install cargo-udeps")),
            }
        } else {
            Ok(ToolResult::ok("dead code analysis available for Rust projects via cargo-udeps"))
        }
    }
}

pub struct DependencyAuditTool { workspace_root: PathBuf }
impl DependencyAuditTool { pub fn new(r: PathBuf) -> Self { Self { workspace_root: r } } }
impl super::Tool for DependencyAuditTool {
    fn name(&self) -> &str { "dependency_audit" }
    fn execute(&self, _args: Value) -> Result<ToolResult> {
        let cmd = if self.workspace_root.join("Cargo.toml").exists() { "cargo audit 2>&1" }
        else if self.workspace_root.join("package.json").exists() { "npm audit 2>&1" }
        else if self.workspace_root.join("go.mod").exists() { "go list -m -json all 2>&1 | grep -A2 '\"Version\"' || echo 'go mod audit not available'" }
        else { "echo 'no supported package manager found'" };
        let out = std::process::Command::new("sh").arg("-c").arg(cmd).current_dir(&self.workspace_root).output();
        match out {
            Ok(o) => Ok(ToolResult { output: String::from_utf8_lossy(&o.stdout).to_string(), success: o.status.success() }),
            Err(e) => Ok(ToolResult::err(format!("audit failed: {}", e))),
        }
    }
}

pub struct ProjectScaffolderTool { workspace_root: PathBuf }
impl ProjectScaffolderTool { pub fn new(r: PathBuf) -> Self { Self { workspace_root: r } } }
impl super::Tool for ProjectScaffolderTool {
    fn name(&self) -> &str { "scaffold_project" }
    fn execute(&self, args: Value) -> Result<ToolResult> {
        let kind = args["kind"].as_str().unwrap_or("");
        let name = args["name"].as_str().unwrap_or("new-project");
        let target = self.workspace_root.join(name);
        match kind {
            "rust" | "cargo" => {
                std::fs::create_dir_all(&target)?;
                std::fs::write(target.join("Cargo.toml"), format!("[package]\nname = \"{}\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\n", name))?;
                std::fs::create_dir_all(target.join("src"))?;
                std::fs::write(target.join("src/main.rs"), "fn main() {\n    println!(\"Hello, world!\");\n}\n")?;
                Ok(ToolResult::ok(format!("scaffolded Rust project: {}", name)))
            }
            "node" | "npm" | "typescript" | "ts" => {
                std::fs::create_dir_all(&target)?;
                std::fs::write(target.join("package.json"), format!("{{\"name\": \"{}\", \"version\": \"1.0.0\", \"scripts\": {{\"start\": \"node index.js\"}}}}", name))?;
                std::fs::write(target.join("index.js"), "console.log('Hello, world!');\n")?;
                Ok(ToolResult::ok(format!("scaffolded Node project: {}", name)))
            }
            "python" | "py" => {
                std::fs::create_dir_all(target.join("src"))?;
                std::fs::write(target.join("pyproject.toml"), format!("[project]\nname = \"{}\"\nversion = \"0.1.0\"\nrequires-python = \">=3.8\"\n", name))?;
                std::fs::write(target.join("src/main.py"), "def main():\n    print('Hello, world!')\n\nif __name__ == '__main__':\n    main()\n")?;
                Ok(ToolResult::ok(format!("scaffolded Python project: {}", name)))
            }
            "go" => {
                std::fs::create_dir_all(target.join("cmd"))?;
                std::fs::write(target.join("go.mod"), format!("module {}\n\ngo 1.21\n", name))?;
                std::fs::write(target.join("cmd/main.go"), "package main\n\nimport \"fmt\"\n\nfunc main() {\n    fmt.Println(\"Hello, world!\")\n}\n")?;
                Ok(ToolResult::ok(format!("scaffolded Go project: {}", name)))
            }
            _ => Ok(ToolResult::err("kinds: rust, node, python, go")),
        }
    }
}

pub struct ChangelogGeneratorTool { workspace_root: PathBuf }
impl ChangelogGeneratorTool { pub fn new(r: PathBuf) -> Self { Self { workspace_root: r } } }
impl super::Tool for ChangelogGeneratorTool {
    fn name(&self) -> &str { "generate_changelog" }
    fn execute(&self, args: Value) -> Result<ToolResult> {
        let from = args["from"].as_str().unwrap_or("");
        let to = args["to"].as_str().unwrap_or("HEAD");
        let range = if from.is_empty() { to.to_string() } else { format!("{}..{}", from, to) };
        let out = run_git(&self.workspace_root, &["log", &range, "--pretty=format:- %s (%h)"])?;
        if out.status.success() {
            let log = String::from_utf8_lossy(&out.stdout).to_string();
            Ok(ToolResult::ok(format!("## Changelog\n\n{}\n", log)))
        } else {
            Ok(ToolResult::err(String::from_utf8_lossy(&out.stderr).to_string()))
        }
    }
}

fn run_git(root: &PathBuf, args: &[&str]) -> std::io::Result<std::process::Output> {
    std::process::Command::new("git").args(args).current_dir(root).output()
}

pub struct ArchitectureAnalyzerTool { workspace_root: PathBuf }
impl ArchitectureAnalyzerTool { pub fn new(r: PathBuf) -> Self { Self { workspace_root: r } } }
impl super::Tool for ArchitectureAnalyzerTool {
    fn name(&self) -> &str { "analyze_architecture" }
    fn execute(&self, _args: Value) -> Result<ToolResult> {
        use ignore::WalkBuilder;
        let mut dirs: std::collections::HashMap<String, (usize, usize)> = std::collections::HashMap::new();
        let walker = WalkBuilder::new(&self.workspace_root).hidden(false).git_ignore(true).max_depth(Some(4)).build();
        for entry in walker.flatten() {
            if entry.file_type().map(|ft| ft.is_file()).unwrap_or(false) {
                if let Ok(rel) = entry.path().strip_prefix(&self.workspace_root) {
                    let parent = rel.parent().map(|p| p.display().to_string()).unwrap_or(".".to_string());
                    if let Ok(meta) = std::fs::metadata(entry.path()) {
                        let entry = dirs.entry(parent).or_insert((0, 0));
                        entry.0 += 1;
                        entry.1 += meta.len() as usize;
                    }
                }
            }
        }
        let mut result = vec!["Directory         | Files | Bytes".to_string(), "------------------|-------|-------".to_string()];
        let mut sorted: Vec<_> = dirs.iter().collect();
        sorted.sort_by_key(|(_, (f, _))| std::cmp::Reverse(*f));
        for (dir, (files, bytes)) in sorted.iter().take(30) {
            result.push(format!("{:18}| {:>5} | {:>6}", dir, files, bytes));
        }
        Ok(ToolResult::ok(result.join("\n")))
    }
}
