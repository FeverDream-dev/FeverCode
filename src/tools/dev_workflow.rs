use anyhow::Result;
use serde_json::Value;
use std::path::{Path, PathBuf};

use super::ToolResult;

pub struct TddEnforcementTool { workspace_root: PathBuf }
impl TddEnforcementTool { pub fn new(r: PathBuf) -> Self { Self { workspace_root: r } } }
impl super::Tool for TddEnforcementTool {
    fn name(&self) -> &str { "tdd_cycle" }
    fn execute(&self, args: Value) -> Result<ToolResult> {
        let phase = args["phase"].as_str().unwrap_or("red");
        let test_file = args["test_file"].as_str().unwrap_or("");
        let source_file = args["source_file"].as_str().unwrap_or("");
        let test_framework = args["framework"].as_str().unwrap_or("auto");
        match phase {
            "red" => {
                if test_file.is_empty() { return Ok(ToolResult::err("test_file required for red phase")); }
                let test_path = self.workspace_root.join(test_file);
                if !test_path.exists() { return Ok(ToolResult::err(format!("test file not found: {}", test_file))); }
                let cmd = detect_test_cmd(&self.workspace_root, test_framework, Some(test_file));
                let out = std::process::Command::new("sh").arg("-c").arg(&cmd).current_dir(&self.workspace_root).output()?;
                let stdout = String::from_utf8_lossy(&out.stdout);
                let stderr = String::from_utf8_lossy(&out.stderr);
                let combined = format!("{}\n{}", stdout, stderr);
                let has_failures = !out.status.success();
                if has_failures {
                    Ok(ToolResult::ok(format!("🔴 RED: Tests fail as expected.\n{}\nNow implement the minimum code to pass.", truncate_output(&combined, 3000))))
                } else {
                    Ok(ToolResult::ok(format!("🟢 Tests already pass! Write a failing test first.\n{}", truncate_output(&combined, 3000))))
                }
            }
            "green" => {
                if source_file.is_empty() { return Ok(ToolResult::err("source_file required for green phase")); }
                let cmd = detect_test_cmd(&self.workspace_root, test_framework, None);
                let out = std::process::Command::new("sh").arg("-c").arg(&cmd).current_dir(&self.workspace_root).output()?;
                let stdout = String::from_utf8_lossy(&out.stdout);
                let stderr = String::from_utf8_lossy(&out.stderr);
                let combined = format!("{}\n{}", stdout, stderr);
                if out.status.success() {
                    Ok(ToolResult::ok(format!("🟢 GREEN: All tests pass!\n{}\nNow refactor if needed.", truncate_output(&combined, 3000))))
                } else {
                    Ok(ToolResult::err(format!("🔴 Tests still failing:\n{}", truncate_output(&combined, 3000))))
                }
            }
            "refactor" => {
                let cmd = detect_test_cmd(&self.workspace_root, test_framework, None);
                let out = std::process::Command::new("sh").arg("-c").arg(&cmd).current_dir(&self.workspace_root).output()?;
                let combined = format!("{}\n{}", String::from_utf8_lossy(&out.stdout), String::from_utf8_lossy(&out.stderr));
                if out.status.success() {
                    let lint_out = run_lint(&self.workspace_root);
                    Ok(ToolResult::ok(format!("🔵 REFACTOR: Tests pass after refactor.\n{}\n{}", truncate_output(&combined, 3000), lint_out)))
                } else {
                    Ok(ToolResult::err(format!("🔴 Tests broke during refactor! Revert or fix:\n{}", truncate_output(&combined, 3000))))
                }
            }
            "verify" => {
                let cmd = detect_test_cmd(&self.workspace_root, test_framework, None);
                let test_out = std::process::Command::new("sh").arg("-c").arg(&cmd).current_dir(&self.workspace_root).output()?;
                let lint = run_lint(&self.workspace_root);
                let build_cmd = detect_build_cmd(&self.workspace_root);
                let build_out = std::process::Command::new("sh").arg("-c").arg(&build_cmd).current_dir(&self.workspace_root).output()?;
                let all_pass = test_out.status.success() && build_out.status.success();
                let report = format!(
                    "Tests: {}\nBuild: {}\nLint: {}\n{}",
                    if test_out.status.success() { "✅ PASS" } else { "❌ FAIL" },
                    if build_out.status.success() { "✅ PASS" } else { "❌ FAIL" },
                    lint,
                    if all_pass { "\n🟢 All checks pass. Ready to commit." } else { "\n🔴 Fix issues before committing." }
                );
                if all_pass { Ok(ToolResult::ok(report)) } else { Ok(ToolResult::err(report)) }
            }
            _ => Ok(ToolResult::err("phases: red, green, refactor, verify")),
        }
    }
}

pub struct PlanningTool { workspace_root: PathBuf }
impl PlanningTool { pub fn new(r: PathBuf) -> Self { Self { workspace_root: r } } }
impl super::Tool for PlanningTool {
    fn name(&self) -> &str { "planning" }
    fn execute(&self, args: Value) -> Result<ToolResult> {
        let action = args["action"].as_str().unwrap_or("list");
        let plan_dir = self.workspace_root.join(".fevercode").join("plans");
        std::fs::create_dir_all(&plan_dir)?;
        match action {
            "create" => {
                let title = args["title"].as_str().unwrap_or("");
                let tasks = args["tasks"].as_array();
                if title.is_empty() { return Ok(ToolResult::err("title required")); }
                let ts = chrono::Utc::now().format("%Y%m%d_%H%M%S");
                let filename = format!("{}_{}.md", title.replace(' ', "_"), ts);
                let mut content = format!("# {}\n\nCreated: {}\nStatus: in-progress\n\n## Tasks\n\n", title, chrono::Utc::now().to_rfc3339());
                if let Some(tasks) = tasks {
                    for (i, task) in tasks.iter().enumerate() {
                        let desc = task.as_str().unwrap_or("?");
                        content.push_str(&format!("{}. [ ] {}\n", i + 1, desc));
                    }
                }
                content.push_str("\n## Notes\n\n(Add notes here)\n\n## Done Criteria\n\n- [ ] All tasks complete\n- [ ] Tests pass\n- [ ] Code reviewed\n");
                std::fs::write(plan_dir.join(&filename), &content)?;
                Ok(ToolResult::ok(format!("Plan created: {}", filename)))
            }
            "list" => {
                let mut out = String::from("Plans:\n");
                let mut found = false;
                let mut entries: Vec<_> = std::fs::read_dir(&plan_dir)?.filter_map(|e| e.ok()).collect();
                entries.sort_by_key(|e| e.file_name());
                for entry in entries {
                    if entry.path().extension().and_then(|e| e.to_str()) == Some("md") {
                        found = true;
                        let name = entry.file_name().to_string_lossy().to_string();
                        let content = std::fs::read_to_string(entry.path())?;
                        let status = if content.contains("Status: completed") { "✅" } else if content.contains("Status: in-progress") { "🔄" } else { "📋" };
                        let title_line = content.lines().find(|l| l.starts_with("# ")).unwrap_or("# unknown");
                        out.push_str(&format!("  {} {} — {}\n", status, name, title_line.trim_start_matches("# ")));
                    }
                }
                if !found { out.push_str("  (none)\n"); }
                Ok(ToolResult::ok(out))
            }
            "update" => {
                let plan = args["plan"].as_str().unwrap_or("");
                let task_idx = args["task"].as_u64().unwrap_or(0);
                let status = args["status"].as_str().unwrap_or("done");
                if plan.is_empty() || task_idx == 0 { return Ok(ToolResult::err("plan name and task number required")); }
                let path = plan_dir.join(plan);
                if !path.exists() { return Ok(ToolResult::err(format!("plan '{}' not found", plan))); }
                let content = std::fs::read_to_string(&path)?;
                let mut lines: Vec<String> = content.lines().map(String::from).collect();
                let mut task_count = 0u64;
                for line in lines.iter_mut() {
                    if line.contains("[ ] ") || line.contains("[x] ") {
                        task_count += 1;
                        if task_count == task_idx {
                            let desc = if line.contains("[ ] ") { line.split("[ ] ").nth(1).unwrap_or("") } else { line.split("[x] ").nth(1).unwrap_or("") };
                            match status {
                                "done" => *line = format!("{}. [x] {}", task_idx, desc),
                                "todo" => *line = format!("{}. [ ] {}", task_idx, desc),
                                _ => {}
                            }
                            break;
                        }
                    }
                }
                std::fs::write(&path, lines.join("\n"))?;
                Ok(ToolResult::ok(format!("task {} in {} marked as {}", task_idx, plan, status)))
            }
            "complete" => {
                let plan = args["plan"].as_str().unwrap_or("");
                if plan.is_empty() { return Ok(ToolResult::err("plan name required")); }
                let path = plan_dir.join(plan);
                if !path.exists() { return Ok(ToolResult::err(format!("plan '{}' not found", plan))); }
                let content = std::fs::read_to_string(&path)?;
                let updated = content.replace("Status: in-progress", "Status: completed").replace("Status: todo", "Status: completed");
                std::fs::write(&path, updated)?;
                Ok(ToolResult::ok(format!("plan {} marked completed", plan)))
            }
            "show" => {
                let plan = args["plan"].as_str().unwrap_or("");
                if plan.is_empty() { return Ok(ToolResult::err("plan name required")); }
                let path = plan_dir.join(plan);
                if !path.exists() { return Ok(ToolResult::err(format!("plan '{}' not found", plan))); }
                Ok(ToolResult::ok(std::fs::read_to_string(&path)?))
            }
            _ => Ok(ToolResult::err("actions: create, list, update, complete, show")),
        }
    }
}

pub struct C4ArchitectureTool { workspace_root: PathBuf }
impl C4ArchitectureTool { pub fn new(r: PathBuf) -> Self { Self { workspace_root: r } } }
impl super::Tool for C4ArchitectureTool {
    fn name(&self) -> &str { "c4_diagram" }
    fn execute(&self, args: Value) -> Result<ToolResult> {
        let level = args["level"].as_str().unwrap_or("all");
        let out_dir = self.workspace_root.join(".fevercode").join("architecture");
        std::fs::create_dir_all(&out_dir)?;
        let mut result = String::new();
        match level {
            "context" | "all" => {
                let mut ctx = String::from("# System Context Diagram\n\n");
                ctx.push_str("```mermaid\ngraph TB\n");
                ctx.push_str("    User[Developer] --> FC[FeverCode Agent]\n");
                ctx.push_str("    FC --> LLM[LLM Provider API]\n");
                ctx.push_str("    FC --> FS[File System]\n");
                ctx.push_str("    FC --> Git[Git Repository]\n");
                ctx.push_str("    FC --> Shell[Shell / Terminal]\n");
                ctx.push_str("    FC --> MCP[MCP Servers]\n");
                ctx.push_str("```\n");
                std::fs::write(out_dir.join("01-context.md"), &ctx)?;
                result.push_str("Generated: 01-context.md\n");
                if level == "context" { return Ok(ToolResult::ok(result)); }
            }
            _ => {}
        }
        let mut containers = String::from("# Container Diagram\n\n```mermaid\ngraph TB\n");
        containers.push_str("    CLI[CLI Parser<br/>clap] --> AL[Agent Loop<br/>Core Orchestrator]\n");
        containers.push_str("    AL --> TR[Tool Registry<br/>72 Built-in + MCP]\n");
        containers.push_str("    AL --> Prov[Provider Layer<br/>OpenAI/Ollama/Anthropic]\n");
        containers.push_str("    AL --> Safety[Safety Policy<br/>Workspace Boundary]\n");
        containers.push_str("    AL --> Events[Session Log<br/>JSONL Events]\n");
        containers.push_str("    TR --> FileTools[File Tools<br/>22 tools]\n");
        containers.push_str("    TR --> GitTools[Git Tools<br/>17 tools]\n");
        containers.push_str("    TR --> QATools[Quality Tools<br/>9 tools]\n");
        containers.push_str("    TR --> IntTools[Integration Tools<br/>13 tools]\n");
        containers.push_str("    TR --> UXTools[UX Tools<br/>10 tools]\n");
        containers.push_str("    TR --> MCP[MCP Bridge<br/>Dynamic]\n");
        containers.push_str("```\n");
        std::fs::write(out_dir.join("02-containers.md"), &containers)?;
        result.push_str("Generated: 02-containers.md\n");

        let mut components = String::from("# Component Diagram - Tool Registry\n\n```mermaid\ngraph TB\n");
        components.push_str("    TR[ToolRegistry] --> FileTools\n");
        components.push_str("    TR --> GitTools\n");
        components.push_str("    TR --> ShellTools\n");
        components.push_str("    TR --> QAUtils\n");
        components.push_str("    TR --> MCPBridge[MCP Bridge Tool]\n");
        components.push_str("    MCPBridge --> McpClient[MCP Client<br/>JSON-RPC]\n");
        components.push_str("    McpClient --> Server1[MCP Server 1]\n");
        components.push_str("    McpClient --> Server2[MCP Server 2]\n");
        components.push_str("    McpClient --> ServerN[MCP Server N]\n");
        components.push_str("```\n");
        std::fs::write(out_dir.join("03-components.md"), &components)?;
        result.push_str("Generated: 03-components.md\n");

        let code_stats = analyze_code_structure(&self.workspace_root)?;
        std::fs::write(out_dir.join("04-code-analysis.md"), &code_stats)?;
        result.push_str("Generated: 04-code-analysis.md\n");

        Ok(ToolResult::ok(result))
    }
}

pub struct CodeReviewTool { workspace_root: PathBuf }
impl CodeReviewTool { pub fn new(r: PathBuf) -> Self { Self { workspace_root: r } } }
impl super::Tool for CodeReviewTool {
    fn name(&self) -> &str { "code_review" }
    fn execute(&self, args: Value) -> Result<ToolResult> {
        let action = args["action"].as_str().unwrap_or("diff");
        match action {
            "diff" => {
                let out = std::process::Command::new("git")
                    .args(["diff", "--stat", "HEAD~1"])
                    .current_dir(&self.workspace_root)
                    .output()?;
                let stat = String::from_utf8_lossy(&out.stdout);
                let diff = std::process::Command::new("git")
                    .args(["diff", "HEAD~1"])
                    .current_dir(&self.workspace_root)
                    .output()?;
                let full_diff = String::from_utf8_lossy(&diff.stdout);
                let mut findings = Vec::new();
                for line in full_diff.lines() {
                    let stripped = line.trim_start_matches('+').trim_start();
                    if stripped.starts_with("unwrap()") { findings.push(("WARN".to_string(), "unwrap() — consider proper error handling".to_string())); }
                    if stripped.contains("as any") || stripped.contains("#[allow(") { findings.push(("WARN".to_string(), "type suppression detected".to_string())); }
                    if stripped.contains("todo!()") || stripped.contains("TODO:") { findings.push(("INFO".to_string(), "TODO marker found".to_string())); }
                    if stripped.contains("println!") && !stripped.contains("//") { findings.push(("INFO".to_string(), "println! in production code — consider logging".to_string())); }
                    if stripped.contains("expect(\"") { findings.push(("OK".to_string(), "expect() with message — good error handling".to_string())); }
                }
                let mut report = format!("## Code Review: HEAD~1 → HEAD\n\n{}\n\n### Findings ({}):\n\n", stat, findings.len());
                for (severity, msg) in &findings {
                    report.push_str(&format!("- [{}] {}\n", severity, msg));
                }
                if findings.is_empty() { report.push_str("No issues found in diff.\n"); }
                Ok(ToolResult::ok(report))
            }
            "files" => {
                let target = args["path"].as_str().unwrap_or(".");
                let path = self.workspace_root.join(target);
                if !path.exists() { return Ok(ToolResult::err(format!("path not found: {}", target))); }
                let content = std::fs::read_to_string(&path)?;
                let lines = content.lines().count();
                let mut issues = Vec::new();
                let mut fn_count = 0u32;
                for (i, line) in content.lines().enumerate() {
                    let trimmed = line.trim();
                    if trimmed.starts_with("fn ") || trimmed.starts_with("pub fn ") || trimmed.starts_with("async fn ") || trimmed.starts_with("pub async fn ") { fn_count += 1; }
                    if trimmed.contains("unwrap()") { issues.push(format!("L{}: unwrap() — use expect() or proper error handling", i + 1)); }
                    if trimmed.contains(".clone()") && trimmed.contains("&") { issues.push(format!("L{}: clone on reference — check if borrow suffices", i + 1)); }
                    if trimmed.contains("println!") { issues.push(format!("L{}: println! — consider structured logging", i + 1)); }
                    if trimmed.len() > 120 { issues.push(format!("L{}: line too long ({} chars)", i + 1, trimmed.len())); }
                }
                let mut report = format!("## File Review: {}\n\nLines: {}, Functions: {}\n\n### Issues ({}):\n\n", target, lines, fn_count, issues.len());
                for issue in &issues { report.push_str(&format!("- {}\n", issue)); }
                if issues.is_empty() { report.push_str("No issues found.\n"); }
                Ok(ToolResult::ok(report))
            }
            "security" => {
                let patterns = [
                    ("password", "potential hardcoded password"),
                    ("secret_key", "potential hardcoded secret"),
                    ("api_key", "potential hardcoded API key"),
                    ("private_key", "potential hardcoded private key"),
                    ("BEGIN RSA", "embedded RSA private key"),
                    ("BEGIN PRIVATE", "embedded private key"),
                    ("eval(", "eval usage — security risk"),
                    ("unsafe", "unsafe block — verify correctness"),
                    (".env", ".env reference — ensure not committed"),
                ];
                let mut findings = Vec::new();
                use ignore::WalkBuilder;
                let walker = WalkBuilder::new(&self.workspace_root).hidden(false).git_ignore(true).max_depth(Some(8)).build();
                for entry in walker.flatten() {
                    if entry.file_type().map(|ft| ft.is_file()).unwrap_or(false) {
                        if let Ok(content) = std::fs::read_to_string(entry.path()) {
                            let rel = entry.path().strip_prefix(&self.workspace_root).unwrap_or(entry.path()).display();
                            for (pattern, desc) in &patterns {
                                if content.contains(pattern) {
                                    findings.push(format!("{}: {} found — {}", rel, pattern, desc));
                                }
                            }
                        }
                    }
                }
                let mut report = format!("## Security Scan\n\n### Findings ({}):\n\n", findings.len());
                for f in &findings { report.push_str(&format!("- ⚠️ {}\n", f)); }
                if findings.is_empty() { report.push_str("✅ No obvious security issues found.\n"); }
                Ok(ToolResult::ok(report))
            }
            _ => Ok(ToolResult::err("actions: diff, files, security")),
        }
    }
}

pub struct PerfProfilerTool { workspace_root: PathBuf }
impl PerfProfilerTool { pub fn new(r: PathBuf) -> Self { Self { workspace_root: r } } }
impl super::Tool for PerfProfilerTool {
    fn name(&self) -> &str { "perf_profile" }
    fn execute(&self, args: Value) -> Result<ToolResult> {
        let action = args["action"].as_str().unwrap_or("analyze");
        match action {
            "analyze" => {
                use ignore::WalkBuilder;
                let mut file_sizes: Vec<(String, u64)> = Vec::new();
                let mut ext_stats: std::collections::HashMap<String, (u32, u64)> = std::collections::HashMap::new();
                let walker = WalkBuilder::new(&self.workspace_root).hidden(false).git_ignore(true).max_depth(Some(8)).build();
                for entry in walker.flatten() {
                    if entry.file_type().map(|ft| ft.is_file()).unwrap_or(false) {
                        if let Ok(meta) = std::fs::metadata(entry.path()) {
                            let size = meta.len();
                            if let Ok(rel) = entry.path().strip_prefix(&self.workspace_root) {
                                let rel_str = rel.display().to_string();
                                if size > 100_000 { file_sizes.push((rel_str.clone(), size)); }
                                let ext = rel.extension().and_then(|e| e.to_str()).unwrap_or("other").to_string();
                                let entry = ext_stats.entry(ext).or_insert((0, 0));
                                entry.0 += 1;
                                entry.1 += size;
                            }
                        }
                    }
                }
                file_sizes.sort_by_key(|(_, s)| std::cmp::Reverse(*s));
                let mut report = String::from("## Performance Profile\n\n### Large Files (>100KB):\n\n");
                for (path, size) in file_sizes.iter().take(20) {
                    report.push_str(&format!("- {} ({:.1} KB)\n", path, *size as f64 / 1024.0));
                }
                report.push_str("\n### By Extension:\n\n");
                let mut ext_sorted: Vec<_> = ext_stats.iter().collect();
                ext_sorted.sort_by_key(|(_, (_, s))| std::cmp::Reverse(*s));
                for (ext, (count, size)) in ext_sorted.iter().take(15) {
                    report.push_str(&format!("- .{}: {} files, {:.1} KB total\n", ext, count, *size as f64 / 1024.0));
                }
                Ok(ToolResult::ok(report))
            }
            "bundle" => {
                let out = std::process::Command::new("sh")
                    .arg("-c")
                    .arg("cargo bloat --release 2>&1 || echo 'cargo-bloat not installed; install with: cargo install cargo-bloat'")
                    .current_dir(&self.workspace_root)
                    .output()?;
                Ok(ToolResult::ok(String::from_utf8_lossy(&out.stdout).to_string()))
            }
            _ => Ok(ToolResult::err("actions: analyze, bundle")),
        }
    }
}

pub struct GitFlowTool { workspace_root: PathBuf }
impl GitFlowTool { pub fn new(r: PathBuf) -> Self { Self { workspace_root: r } } }
impl super::Tool for GitFlowTool {
    fn name(&self) -> &str { "git_flow" }
    fn execute(&self, args: Value) -> Result<ToolResult> {
        let action = args["action"].as_str().unwrap_or("status");
        match action {
            "feature" => {
                let name = args["name"].as_str().unwrap_or("");
                if name.is_empty() { return Ok(ToolResult::err("feature name required")); }
                let branch = format!("feature/{}", name);
                let out = std::process::Command::new("git")
                    .args(["checkout", "-b", &branch])
                    .current_dir(&self.workspace_root)
                    .output()?;
                if out.status.success() {
                    Ok(ToolResult::ok(format!("created feature branch: {}", branch)))
                } else {
                    Ok(ToolResult::err(String::from_utf8_lossy(&out.stderr).to_string()))
                }
            }
            "release" => {
                let version = args["version"].as_str().unwrap_or("");
                if version.is_empty() { return Ok(ToolResult::err("version required")); }
                let branch = format!("release/{}", version);
                let out = std::process::Command::new("git")
                    .args(["checkout", "-b", &branch])
                    .current_dir(&self.workspace_root)
                    .output()?;
                if out.status.success() {
                    Ok(ToolResult::ok(format!("created release branch: {}", branch)))
                } else {
                    Ok(ToolResult::err(String::from_utf8_lossy(&out.stderr).to_string()))
                }
            }
            "finish" => {
                let branch_out = std::process::Command::new("git")
                    .args(["branch", "--show-current"])
                    .current_dir(&self.workspace_root)
                    .output()?;
                let current = String::from_utf8_lossy(&branch_out.stdout).trim().to_string();
                let target = args["target"].as_str().unwrap_or("main");
                let merge = std::process::Command::new("git")
                    .args(["checkout", target])
                    .current_dir(&self.workspace_root)
                    .output()?;
                if !merge.status.success() { return Ok(ToolResult::err(String::from_utf8_lossy(&merge.stderr).to_string())); }
                let merge2 = std::process::Command::new("git")
                    .args(["merge", "--no-ff", &current, "-m", &format!("Merge {}", current)])
                    .current_dir(&self.workspace_root)
                    .output()?;
                if merge2.status.success() {
                    Ok(ToolResult::ok(format!("merged {} into {}", current, target)))
                } else {
                    Ok(ToolResult::err(String::from_utf8_lossy(&merge2.stderr).to_string()))
                }
            }
            "hotfix" => {
                let name = args["name"].as_str().unwrap_or("");
                if name.is_empty() { return Ok(ToolResult::err("hotfix name required")); }
                let branch = format!("hotfix/{}", name);
                let main_out = std::process::Command::new("git")
                    .args(["checkout", "main"])
                    .current_dir(&self.workspace_root)
                    .output()?;
                if !main_out.status.success() { return Ok(ToolResult::err(String::from_utf8_lossy(&main_out.stderr).to_string())); }
                let out = std::process::Command::new("git")
                    .args(["checkout", "-b", &branch])
                    .current_dir(&self.workspace_root)
                    .output()?;
                if out.status.success() {
                    Ok(ToolResult::ok(format!("created hotfix branch: {} from main", branch)))
                } else {
                    Ok(ToolResult::err(String::from_utf8_lossy(&out.stderr).to_string()))
                }
            }
            "status" => {
                let branch_out = std::process::Command::new("git")
                    .args(["branch", "--show-current"])
                    .current_dir(&self.workspace_root)
                    .output()?;
                let current = String::from_utf8_lossy(&branch_out.stdout).trim().to_string();
                let branch_type = if current.starts_with("feature/") { "feature" }
                    else if current.starts_with("release/") { "release" }
                    else if current.starts_with("hotfix/") { "hotfix" }
                    else { "other" };
                Ok(ToolResult::ok(format!("Current branch: {} ({})\nGit flow actions: feature, release, hotfix, finish", current, branch_type)))
            }
            _ => Ok(ToolResult::err("actions: feature, release, hotfix, finish, status")),
        }
    }
}

pub struct N8nWorkflowTool { workspace_root: PathBuf }
impl N8nWorkflowTool { pub fn new(r: PathBuf) -> Self { Self { workspace_root: r } } }
impl super::Tool for N8nWorkflowTool {
    fn name(&self) -> &str { "n8n" }
    fn execute(&self, args: Value) -> Result<ToolResult> {
        let action = args["action"].as_str().unwrap_or("info");
        match action {
            "info" => {
                let base_url = std::env::var("N8N_BASE_URL").unwrap_or_else(|_| "http://localhost:5678".to_string());
                let api_key = std::env::var("N8N_API_KEY").unwrap_or_default();
                if api_key.is_empty() {
                    return Ok(ToolResult::ok(format!("n8n base URL: {}\nSet N8N_API_KEY env var for API access.", base_url)));
                }
                let out = std::process::Command::new("curl")
                    .args(["-s", "-H", &format!("X-N8N-API-KEY: {}", api_key), &format!("{}/api/v1/workflows", base_url)])
                    .output()?;
                Ok(ToolResult::ok(String::from_utf8_lossy(&out.stdout).to_string()))
            }
            _ => Ok(ToolResult::err("actions: info")),
        }
    }
}

pub struct LinearTool { workspace_root: PathBuf }
impl LinearTool { pub fn new(r: PathBuf) -> Self { Self { workspace_root: r } } }
impl super::Tool for LinearTool {
    fn name(&self) -> &str { "linear" }
    fn execute(&self, args: Value) -> Result<ToolResult> {
        let token = std::env::var("LINEAR_API_KEY").unwrap_or_default();
        if token.is_empty() { return Ok(ToolResult::err("LINEAR_API_KEY env var not set")); }
        let action = args["action"].as_str().unwrap_or("my_issues");
        match action {
            "my_issues" => {
                let query = r#"{"query":"{ viewer { assignedIssues(filter: {state: {name: {neq: \"Done\"}}}) { nodes { identifier title state { name } priority } } } }"}"#;
                let out = std::process::Command::new("curl")
                    .args(["-s", "-X", "POST", "https://api.linear.app/graphql",
                           "-H", "Content-Type: application/json",
                           "-H", &format!("Authorization: Bearer {}", token),
                           "-d", query])
                    .output()?;
                let data: Value = serde_json::from_str(&String::from_utf8_lossy(&out.stdout)).unwrap_or_default();
                let issues = data["data"]["viewer"]["assignedIssues"]["nodes"].as_array().cloned().unwrap_or_default();
                if issues.is_empty() { return Ok(ToolResult::ok("no active issues assigned to you")); }
                let mut result = String::from("Your Linear issues:\n");
                for issue in &issues {
                    let id = issue["identifier"].as_str().unwrap_or("?");
                    let title = issue["title"].as_str().unwrap_or("?");
                    let state = issue["state"]["name"].as_str().unwrap_or("?");
                    result.push_str(&format!("  {} — [{}] {}\n", id, state, title));
                }
                Ok(ToolResult::ok(result))
            }
            "create" => {
                let title = args["title"].as_str().unwrap_or("");
                let team_id = args["team_id"].as_str().unwrap_or("");
                if title.is_empty() || team_id.is_empty() { return Ok(ToolResult::err("title and team_id required")); }
                let mutation = serde_json::json!({"query": format!("mutation {{ issueCreate(input: {{title: \"{}\", teamId: \"{}\"}}) {{ success issue {{ identifier title }} }} }}", title, team_id)}).to_string();
                let out = std::process::Command::new("curl")
                    .args(["-s", "-X", "POST", "https://api.linear.app/graphql",
                           "-H", "Content-Type: application/json",
                           "-H", &format!("Authorization: Bearer {}", token),
                           "-d", &mutation])
                    .output()?;
                Ok(ToolResult::ok(String::from_utf8_lossy(&out.stdout).to_string()))
            }
            _ => Ok(ToolResult::err("actions: my_issues, create")),
        }
    }
}

fn detect_test_cmd(root: &Path, framework: &str, filter: Option<&str>) -> String {
    match framework {
        "cargo" | "rust" => format!("cargo test {} 2>&1", filter.unwrap_or("")),
        "npm" | "jest" => format!("npm test {} 2>&1", filter.map(|f| format!("-- --testPathPattern={}", f)).unwrap_or_default()),
        "pytest" | "python" => format!("pytest {} -v 2>&1", filter.unwrap_or("")),
        "go" => format!("go test {} -v 2>&1", filter.unwrap_or("./...")),
        _ => {
            if root.join("Cargo.toml").exists() { format!("cargo test {} 2>&1", filter.unwrap_or("")) }
            else if root.join("package.json").exists() { "npm test 2>&1".to_string() }
            else if root.join("go.mod").exists() { "go test ./... -v 2>&1".to_string() }
            else if root.join("pytest.ini").exists() || root.join("pyproject.toml").exists() { "pytest -v 2>&1".to_string() }
            else { "echo 'no test framework detected'".to_string() }
        }
    }
}

fn detect_build_cmd(root: &Path) -> String {
    if root.join("Cargo.toml").exists() { "cargo build 2>&1".to_string() }
    else if root.join("package.json").exists() { "npm run build 2>&1 || echo 'no build script'".to_string() }
    else if root.join("go.mod").exists() { "go build ./... 2>&1".to_string() }
    else { "echo 'no build tool detected'".to_string() }
}

fn run_lint(root: &Path) -> String {
    if root.join("Cargo.toml").exists() {
        let out = std::process::Command::new("cargo").args(["clippy", "--", "-D", "warnings"]).current_dir(root).output();
        match out {
            Ok(o) if o.status.success() => "✅ Clippy clean".to_string(),
            Ok(o) => format!("⚠️ Clippy issues:\n{}", truncate_output(&String::from_utf8_lossy(&o.stderr), 2000)),
            Err(_) => "Clippy not available".to_string(),
        }
    } else if root.join("package.json").exists() {
        let out = std::process::Command::new("npx").args(["eslint", ".", "--max-warnings=0"]).current_dir(root).output();
        match out {
            Ok(o) if o.status.success() => "✅ ESLint clean".to_string(),
            Ok(_) => "⚠️ ESLint issues found".to_string(),
            Err(_) => "ESLint not configured".to_string(),
        }
    } else {
        "No linter configured".to_string()
    }
}

fn truncate_output(s: &str, max: usize) -> String {
    if s.len() > max { let mut t: String = s.chars().take(max).collect(); t.push_str("\n... (truncated)"); t } else { s.to_string() }
}

fn analyze_code_structure(root: &Path) -> Result<String> {
    use ignore::WalkBuilder;
    let mut modules: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();
    let walker = WalkBuilder::new(root).hidden(false).git_ignore(true).max_depth(Some(3)).build();
    for entry in walker.flatten() {
        if entry.file_type().map(|ft| ft.is_file()).unwrap_or(false) {
            if let Ok(rel) = entry.path().strip_prefix(root) {
                let parent = rel.parent().map(|p| p.display().to_string()).unwrap_or(".".to_string());
                let filename = rel.file_name().and_then(|n| n.to_str()).unwrap_or("").to_string();
                if filename.ends_with(".rs") || filename.ends_with(".ts") || filename.ends_with(".js") || filename.ends_with(".py") || filename.ends_with(".go") {
                    modules.entry(parent).or_default().push(filename);
                }
            }
        }
    }
    let mut result = String::from("# Code Structure Analysis\n\n```\n");
    let mut sorted: Vec<_> = modules.iter().collect();
    sorted.sort_by_key(|(k, _)| k.to_string());
    for (dir, files) in sorted {
        result.push_str(&format!("{}/\n", dir));
        for f in files {
            result.push_str(&format!("  - {}\n", f));
        }
    }
    result.push_str("```\n");
    Ok(result)
}
