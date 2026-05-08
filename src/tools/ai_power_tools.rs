use anyhow::Result;
use serde_json::Value;
use std::path::{Path, PathBuf};

use super::ToolResult;

// ---------------------------------------------------------------------------
// TokenCompressionTool
// ---------------------------------------------------------------------------

pub struct TokenCompressionTool { workspace_root: PathBuf }
impl TokenCompressionTool { pub fn new(r: PathBuf) -> Self { Self { workspace_root: r } } }

impl super::Tool for TokenCompressionTool {
    fn name(&self) -> &str { "token_compress" }

    fn execute(&self, args: Value) -> Result<ToolResult> {
        let action = args["action"].as_str().unwrap_or("compress");
        match action {
            "compress" => {
                let text = args["text"].as_str().unwrap_or("");
                let level = args["level"].as_str().unwrap_or("medium");
                if text.is_empty() {
                    return Ok(ToolResult::err("text field required for compress action"));
                }
                let compressed = compress_text(text, level);
                let ratio = if text.is_empty() { 0.0 } else {
                    (compressed.len() as f64 / text.len() as f64 * 100.0).round()
                };
                Ok(ToolResult::ok(format!(
                    "Compressed to {}% of original ({}→{} chars)\nLevel: {}\n---\n{}",
                    ratio, text.len(), compressed.len(), level, compressed
                )))
            }
            "decompress" => {
                let text = args["text"].as_str().unwrap_or("");
                if text.is_empty() {
                    return Ok(ToolResult::err("text field required for decompress action"));
                }
                let restored = decompress_text(text);
                Ok(ToolResult::ok(restored))
            }
            "estimate_tokens" => {
                let text = args["text"].as_str().unwrap_or("");
                if text.is_empty() {
                    return Ok(ToolResult::err("text field required for estimate_tokens action"));
                }
                let estimate = estimate_token_count(text);
                let compressed = compress_text(text, "medium");
                let compressed_estimate = estimate_token_count(&compressed);
                let savings = estimate.saturating_sub(compressed_estimate);
                Ok(ToolResult::ok(format!(
                    "Original: ~{} tokens\nCompressed: ~{} tokens\nSavings: ~{} tokens (~{}%)",
                    estimate,
                    compressed_estimate,
                    savings,
                    if estimate == 0 { 0 } else { savings * 100 / estimate }
                )))
            }
            "config" => {
                let dir = self.workspace_root.join(".fevercode");
                std::fs::create_dir_all(&dir)?;
                let config_path = dir.join("compression.toml");
                let level = args["level"].as_str().unwrap_or("medium");
                let config = format!(
                    "# FeverCode Token Compression Config\nlevel = \"{}\"\n\n# Levels: lite, medium, ultra\n# lite   — remove comments, blank lines\n# medium — compress identifiers, remove boilerplate\n# ultra  — aggressive abbreviation, remove all non-essential\n",
                    level
                );
                std::fs::write(&config_path, &config)?;
                Ok(ToolResult::ok(format!("Compression config saved: {} (level={})", config_path.display(), level)))
            }
            _ => Ok(ToolResult::err("actions: compress, decompress, estimate_tokens, config")),
        }
    }
}

fn compress_text(text: &str, level: &str) -> String {
    match level {
        "lite" => compress_lite(text),
        "ultra" => compress_ultra(text),
        _ => compress_medium(text),
    }
}

fn compress_lite(text: &str) -> String {
    text.lines()
        .filter(|l| {
            let trimmed = l.trim();
            !trimmed.is_empty()
                && !trimmed.starts_with("//")
                && !trimmed.starts_with('#')
                && !trimmed.starts_with("/*")
                && !trimmed.starts_with('*')
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn compress_medium(text: &str) -> String {
    let lite = compress_lite(text);
    let mut result = String::with_capacity(lite.len());
    for line in lite.lines() {
        let trimmed = line.trim();
        let collapsed: String = trimmed.split_whitespace().collect::<Vec<_>>().join(" ");
        let cleaned = collapsed
            .replace("public ", "")
            .replace("private ", "")
            .replace("protected ", "")
            .replace("function ", "fn ")
            .replace("return ", "");
        result.push_str(&cleaned);
        result.push('\n');
    }
    result.trim_end().to_string()
}

fn compress_ultra(text: &str) -> String {
    let medium = compress_medium(text);
    let mut result = String::with_capacity(medium.len());
    for line in medium.lines() {
        let trimmed = line.trim();
        if trimmed.len() < 2 { continue; }
        let abbreviated = trimmed
            .replace("function", "fn")
            .replace("variable", "var")
            .replace("parameter", "prm")
            .replace("return", "ret")
            .replace("string", "str")
            .replace("number", "num")
            .replace("boolean", "bool")
            .replace("object", "obj")
            .replace("array", "arr")
            .replace("result", "res")
            .replace("error", "err")
            .replace("value", "val")
            .replace("index", "idx")
            .replace("length", "len")
            .replace("count", "cnt")
            .replace("message", "msg")
            .replace("response", "resp")
            .replace("request", "req")
            .replace("context", "ctx")
            .replace("config", "cfg")
            .replace("environment", "env")
            .replace("document", "doc")
            .replace("information", "info")
            .replace("description", "desc")
            .replace("attribute", "attr")
            .replace("property", "prop")
            .replace("element", "elem")
            .replace("component", "comp");
        result.push_str(abbreviated.trim());
        result.push('\n');
    }
    result.trim_end().to_string()
}

fn decompress_text(text: &str) -> String {
    let expanded = text
        .replace(" fn ", " function ")
        .replace(" var ", " variable ")
        .replace(" prm ", " parameter ")
        .replace(" ret ", " return ")
        .replace(" str ", " string ")
        .replace(" num ", " number ")
        .replace(" bool ", " boolean ")
        .replace(" obj ", " object ")
        .replace(" arr ", " array ")
        .replace(" res ", " result ")
        .replace(" err ", " error ")
        .replace(" val ", " value ")
        .replace(" idx ", " index ")
        .replace(" len ", " length ")
        .replace(" cnt ", " count ")
        .replace(" msg ", " message ")
        .replace(" resp ", " response ")
        .replace(" req ", " request ")
        .replace(" ctx ", " context ")
        .replace(" cfg ", " config ")
        .replace(" env ", " environment ")
        .replace(" doc ", " document ")
        .replace(" info ", " information ")
        .replace(" desc ", " description ")
        .replace(" attr ", " attribute ")
        .replace(" prop ", " property ")
        .replace(" elem ", " element ")
        .replace(" comp ", " component ");
    format!("Decompressed text:\n{}", expanded.trim())
}

fn estimate_token_count(text: &str) -> usize {
    (text.len() as f64 / 4.0).ceil() as usize
}

// ---------------------------------------------------------------------------
// PromptsLibraryTool
// ---------------------------------------------------------------------------

pub struct PromptsLibraryTool { workspace_root: PathBuf }
impl PromptsLibraryTool { pub fn new(r: PathBuf) -> Self { Self { workspace_root: r } } }

impl super::Tool for PromptsLibraryTool {
    fn name(&self) -> &str { "prompts" }

    fn execute(&self, args: Value) -> Result<ToolResult> {
        let action = args["action"].as_str().unwrap_or("list");
        let prompts_dir = self.workspace_root.join(".fevercode").join("prompts");
        std::fs::create_dir_all(&prompts_dir)?;

        match action {
            "list" => {
                let mut entries = Vec::new();
                let mut categories = std::collections::BTreeSet::new();
                if let Ok(rd) = std::fs::read_dir(&prompts_dir) {
                    for entry in rd.flatten() {
                        let path = entry.path();
                        if path.extension().is_some_and(|e| e == "md") {
                            if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                                let parts: Vec<&str> = name.splitn(2, '-').collect();
                                let category = parts.first().unwrap_or(&"general");
                                categories.insert(category.to_string());
                                let content = std::fs::read_to_string(&path).unwrap_or_default();
                                let title = content.lines().next().unwrap_or(name).trim_start_matches('#').trim();
                                entries.push(format!("  [{:>10}] {} — {}", category, name, title));
                            }
                        }
                    }
                }
                if entries.is_empty() {
                    seed_default_prompts(&prompts_dir)?;
                    return self.execute(args);
                }
                entries.sort();
                let cats: Vec<String> = categories.into_iter().collect();
                Ok(ToolResult::ok(format!(
                    "Prompts library ({} prompts, {} categories: {})\n{}",
                    entries.len(),
                    cats.len(),
                    cats.join(", "),
                    entries.join("\n")
                )))
            }
            "get" => {
                let name = args["name"].as_str().unwrap_or("");
                if name.is_empty() {
                    return Ok(ToolResult::err("name field required for get action"));
                }
                let path = prompts_dir.join(format!("{}.md", name));
                if !path.exists() {
                    if let Ok(rd) = std::fs::read_dir(&prompts_dir) {
                        for entry in rd.flatten() {
                            let p = entry.path();
                            if p.file_stem().and_then(|s| s.to_str()).is_some_and(|s| s.contains(name)) {
                                let content = std::fs::read_to_string(&p)?;
                                return Ok(ToolResult::ok(content));
                            }
                        }
                    }
                    return Ok(ToolResult::err(format!("prompt '{}' not found", name)));
                }
                let content = std::fs::read_to_string(&path)?;
                Ok(ToolResult::ok(content))
            }
            "save" => {
                let name = args["name"].as_str().unwrap_or("");
                let content = args["content"].as_str().unwrap_or("");
                if name.is_empty() || content.is_empty() {
                    return Ok(ToolResult::err("name and content fields required for save action"));
                }
                let path = prompts_dir.join(format!("{}.md", name));
                std::fs::write(&path, content)?;
                Ok(ToolResult::ok(format!("Prompt saved: {}", path.display())))
            }
            "delete" => {
                let name = args["name"].as_str().unwrap_or("");
                if name.is_empty() {
                    return Ok(ToolResult::err("name field required for delete action"));
                }
                let path = prompts_dir.join(format!("{}.md", name));
                if path.exists() {
                    std::fs::remove_file(&path)?;
                    Ok(ToolResult::ok(format!("Prompt deleted: {}", name)))
                } else {
                    Ok(ToolResult::err(format!("prompt '{}' not found", name)))
                }
            }
            "render" => {
                let name = args["name"].as_str().unwrap_or("");
                if name.is_empty() {
                    return Ok(ToolResult::err("name field required for render action"));
                }
                let path = prompts_dir.join(format!("{}.md", name));
                if !path.exists() {
                    return Ok(ToolResult::err(format!("prompt '{}' not found", name)));
                }
                let mut template = std::fs::read_to_string(&path)?;
                if let Some(vars) = args["variables"].as_object() {
                    for (k, v) in vars {
                        let placeholder = format!("{{{{{}}}}}", k);
                        let replacement = v.as_str().unwrap_or("");
                        template = template.replace(&placeholder, replacement);
                    }
                }
                Ok(ToolResult::ok(template))
            }
            _ => Ok(ToolResult::err("actions: list, get, save, delete, render")),
        }
    }
}

fn seed_default_prompts(dir: &Path) -> Result<()> {
    let defaults = [
        ("code-review-default.md", "# Code Review\n\nReview the following code changes for:\n- Bugs and logic errors\n- Security vulnerabilities\n- Performance issues\n- Code style and readability\n- Missing error handling\n\n## Code to review:\n{{code}}"),
        ("commit-message.md", "# Commit Message Generator\n\nGenerate a concise commit message following Conventional Commits format.\n\n## Changes:\n{{diff}}\n\n## Format:\ntype(scope): description\n\nTypes: feat, fix, refactor, docs, test, chore, perf"),
        ("debug-helper.md", "# Debug Helper\n\nAnalyze the following error and suggest fixes.\n\n## Error:\n{{error}}\n\n## Context:\n{{context}}\n\n## Steps:\n1. Identify root cause\n2. Propose minimal fix\n3. Explain why the fix works"),
        ("refactor-plan.md", "# Refactor Plan\n\nCreate a refactoring plan for the following code.\n\n## Code:\n{{code}}\n\n## Goals:\n- Improve readability\n- Reduce complexity\n- Maintain behavior\n- Add tests if missing"),
        ("test-generator.md", "# Test Generator\n\nGenerate comprehensive tests for the following code.\n\n## Code:\n{{code}}\n\n## Requirements:\n- Cover happy path\n- Cover edge cases\n- Cover error cases\n- Use appropriate test framework"),
        ("docs-generator.md", "# Documentation Generator\n\nGenerate documentation for the following code.\n\n## Code:\n{{code}}\n\n## Format:\n- Module-level doc comment\n- Function/method doc comments\n- Example usage\n- Parameter descriptions"),
    ];
    for (name, content) in &defaults {
        let path = dir.join(name);
        if !path.exists() {
            std::fs::write(&path, content)?;
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// ParallelDispatchTool
// ---------------------------------------------------------------------------

pub struct ParallelDispatchTool { workspace_root: PathBuf }
impl ParallelDispatchTool { pub fn new(r: PathBuf) -> Self { Self { workspace_root: r } } }

impl super::Tool for ParallelDispatchTool {
    fn name(&self) -> &str { "parallel_dispatch" }

    fn execute(&self, args: Value) -> Result<ToolResult> {
        let action = args["action"].as_str().unwrap_or("plan");
        match action {
            "plan" => {
                let tasks = args["tasks"].as_array();
                if tasks.is_none() || tasks.unwrap().is_empty() {
                    return Ok(ToolResult::err("tasks array required for plan action. Each task: {\"tool\": \"...\", \"args\": {...}}"));
                }
                let tasks = tasks.unwrap();
                let mut plan = Vec::new();
                let mut deps = Vec::new();
                for (i, task) in tasks.iter().enumerate() {
                    let tool = task["tool"].as_str().unwrap_or("unknown");
                    let desc = task["description"].as_str().unwrap_or(tool);
                    let depends_on = task["depends_on"].as_array();
                    let dep_str = if let Some(d) = depends_on {
                        let indices: Vec<String> = d.iter()
                            .filter_map(|v| v.as_u64().map(|n| format!("#{}", n)))
                            .collect();
                        deps.push(indices.clone());
                        if indices.is_empty() { "none".to_string() } else { indices.join(", ") }
                    } else {
                        deps.push(Vec::new());
                        "none".to_string()
                    };
                    plan.push(format!("  #{} [{}] {} (depends: {})", i, tool, desc, dep_str));
                }
                let mut groups: Vec<Vec<usize>> = Vec::new();
                let mut assigned = vec![false; tasks.len()];
                for _round in 0..tasks.len() {
                    let mut group = Vec::new();
                    for i in 0..tasks.len() {
                        if assigned[i] { continue; }
                        let all_deps_met = deps[i].iter().all(|d| {
                            let dep_idx: usize = d.trim_start_matches('#').parse().unwrap_or(usize::MAX);
                            assigned[dep_idx]
                        });
                        if all_deps_met {
                            group.push(i);
                            assigned[i] = true;
                        }
                    }
                    if group.is_empty() { break; }
                    groups.push(group);
                }
                let mut schedule = String::new();
                for (round, group) in groups.iter().enumerate() {
                    schedule.push_str(&format!("Round {} (parallel):\n", round + 1));
                    for &idx in group {
                        schedule.push_str(&plan[idx]);
                        schedule.push('\n');
                    }
                }
                Ok(ToolResult::ok(format!(
                    "Parallel execution plan ({} tasks, {} rounds):\n{}", tasks.len(), groups.len(), schedule
                )))
            }
            "dispatch" => {
                let tasks = args["tasks"].as_array();
                if tasks.is_none() || tasks.unwrap().is_empty() {
                    return Ok(ToolResult::err("tasks array required for dispatch action"));
                }
                let tasks = tasks.unwrap();
                let mut results = Vec::new();
                let registry = super::ToolRegistry::build_default(self.workspace_root.clone());
                for (i, task) in tasks.iter().enumerate() {
                    let tool_name = task["tool"].as_str().unwrap_or("unknown");
                    let tool_args = task.get("args").cloned().unwrap_or(Value::Object(serde_json::Map::new()));
                    match registry.get(tool_name) {
                        Some(tool) => {
                            match tool.execute(tool_args) {
                                Ok(result) => {
                                    results.push(format!(
                                        "#{} [{}] ✅ {}\n{}",
                                        i, tool_name,
                                        if result.success { "success" } else { "error" },
                                        truncate_result(&result.output, 500)
                                    ));
                                }
                                Err(e) => {
                                    results.push(format!("#{} [{}] ❌ error: {}", i, tool_name, e));
                                }
                            }
                        }
                        None => {
                            results.push(format!("#{} [{}] ❌ tool not found", i, tool_name));
                        }
                    }
                }
                let success_count = results.iter().filter(|r| r.contains("✅ success")).count();
                Ok(ToolResult::ok(format!(
                    "Dispatched {} tasks ({} succeeded):\n{}",
                    tasks.len(), success_count, results.join("\n\n")
                )))
            }
            "batch" => {
                let tool_name = args["tool"].as_str().unwrap_or("");
                let inputs = args["inputs"].as_array();
                if tool_name.is_empty() || inputs.is_none() || inputs.unwrap().is_empty() {
                    return Ok(ToolResult::err("tool name and inputs array required for batch action"));
                }
                let inputs = inputs.unwrap();
                let registry = super::ToolRegistry::build_default(self.workspace_root.clone());
                match registry.get(tool_name) {
                    Some(tool) => {
                        let mut results = Vec::new();
                        for (i, input_args) in inputs.iter().enumerate() {
                            match tool.execute(input_args.clone()) {
                                Ok(result) => {
                                    results.push(format!("#{} ✅ {}", i, truncate_result(&result.output, 300)));
                                }
                                Err(e) => {
                                    results.push(format!("#{} ❌ {}", i, e));
                                }
                            }
                        }
                        let ok_count = results.iter().filter(|r| r.contains("✅")).count();
                        Ok(ToolResult::ok(format!(
                            "Batch {} × {} ({} ok):\n{}",
                            inputs.len(), tool_name, ok_count, results.join("\n")
                        )))
                    }
                    None => Ok(ToolResult::err(format!("tool '{}' not found", tool_name))),
                }
            }
            _ => Ok(ToolResult::err("actions: plan, dispatch, batch")),
        }
    }
}

fn truncate_result(text: &str, max_len: usize) -> String {
    if text.len() <= max_len {
        text.to_string()
    } else {
        format!("{}... (truncated, {} chars total)", &text[..max_len], text.len())
    }
}

// ---------------------------------------------------------------------------
// ContextManagerTool
// ---------------------------------------------------------------------------

pub struct ContextManagerTool { workspace_root: PathBuf }
impl ContextManagerTool { pub fn new(r: PathBuf) -> Self { Self { workspace_root: r } } }

impl super::Tool for ContextManagerTool {
    fn name(&self) -> &str { "context_manager" }

    fn execute(&self, args: Value) -> Result<ToolResult> {
        let action = args["action"].as_str().unwrap_or("status");
        let session_dir = self.workspace_root.join(".fevercode").join("session");
        std::fs::create_dir_all(&session_dir)?;

        match action {
            "status" => {
                let events_path = session_dir.join("events.jsonl");
                let event_count = if events_path.exists() {
                    std::io::BufRead::lines(std::io::BufReader::new(std::fs::File::open(&events_path)?))
                        .count()
                } else {
                    0
                };
                let compact_path = session_dir.join("compact-summary.md");
                let has_summary = compact_path.exists();
                Ok(ToolResult::ok(format!(
                    "Context status:\n  Events logged: {}\n  Compact summary: {}\n  Session dir: {}",
                    event_count,
                    if has_summary { "yes" } else { "no" },
                    session_dir.display()
                )))
            }
            "compact" => {
                let events_path = session_dir.join("events.jsonl");
                if !events_path.exists() {
                    return Ok(ToolResult::err("No session events to compact"));
                }
                let content = std::fs::read_to_string(&events_path)?;
                let lines: Vec<&str> = content.lines().collect();
                let total = lines.len();
                let keep = args["keep_last"].as_u64().unwrap_or(50) as usize;
                let summary_path = session_dir.join("compact-summary.md");
                let mut summary = format!(
                    "# Session Compact Summary\n\nOriginal events: {}\nCompacted at: kept last {} events\n\n## Earlier activity archived\n",
                    total, keep.min(total)
                );
                let mut type_counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
                for line in &lines[..total.saturating_sub(keep)] {
                    if let Ok(evt) = serde_json::from_str::<Value>(line) {
                        let evt_type = evt["type"].as_str().unwrap_or("unknown").to_string();
                        *type_counts.entry(evt_type).or_insert(0) += 1;
                    }
                }
                if !type_counts.is_empty() {
                    summary.push_str("\n| Event Type | Count |\n|---|---|\n");
                    let mut sorted: Vec<_> = type_counts.iter().collect();
                    sorted.sort_by(|a, b| b.1.cmp(a.1));
                    for (k, v) in sorted {
                        summary.push_str(&format!("| {} | {} |\n", k, v));
                    }
                }
                let remaining: Vec<&str> = lines.into_iter().rev().take(keep).collect::<Vec<_>>().into_iter().rev().collect();
                std::fs::write(&events_path, remaining.join("\n") + "\n")?;
                std::fs::write(&summary_path, &summary)?;
                Ok(ToolResult::ok(format!("Compacted: {}→{} events. Summary saved.", total, keep.min(total))))
            }
            "export" => {
                let events_path = session_dir.join("events.jsonl");
                if !events_path.exists() {
                    return Ok(ToolResult::err("No session events to export"));
                }
                let format = args["format"].as_str().unwrap_or("markdown");
                let content = std::fs::read_to_string(&events_path)?;
                let output = match format {
                    "json" => content,
                    _ => {
                        let mut md = String::from("# Session Export\n\n");
                        for line in content.lines() {
                            if let Ok(evt) = serde_json::from_str::<Value>(line) {
                                let ts = evt["timestamp"].as_str().unwrap_or("-");
                                let evt_type = evt["type"].as_str().unwrap_or("event");
                                let data = evt["data"].as_str().unwrap_or("");
                                md.push_str(&format!("**{}** [{}] {}\n\n", ts, evt_type, data));
                            }
                        }
                        md
                    }
                };
                let export_path = session_dir.join(format!("export.{}", if format == "json" { "jsonl" } else { "md" }));
                std::fs::write(&export_path, &output)?;
                Ok(ToolResult::ok(format!("Exported to: {}", export_path.display())))
            }
            _ => Ok(ToolResult::err("actions: status, compact, export")),
        }
    }
}

// ---------------------------------------------------------------------------
// SmartContextTool
// ---------------------------------------------------------------------------

pub struct SmartContextTool { workspace_root: PathBuf }
impl SmartContextTool { pub fn new(r: PathBuf) -> Self { Self { workspace_root: r } } }

impl super::Tool for SmartContextTool {
    fn name(&self) -> &str { "smart_context" }

    fn execute(&self, args: Value) -> Result<ToolResult> {
        let action = args["action"].as_str().unwrap_or("select");
        match action {
            "select" => {
                let query = args["query"].as_str().unwrap_or("");
                let max_files = args["max_files"].as_u64().unwrap_or(10) as usize;
                if query.is_empty() {
                    return Ok(ToolResult::err("query field required for select action"));
                }
                let query_lower = query.to_lowercase();
                let query_terms: Vec<&str> = query_lower.split_whitespace().collect();
                let mut scored: Vec<(std::path::PathBuf, usize)> = Vec::new();
                if let Ok(entries) = ignore::WalkBuilder::new(&self.workspace_root)
                    .hidden(true)
                    .git_ignore(true)
                    .build()
                    .collect::<Result<Vec<_>, _>>()
                {
                    for entry in entries {
                        let path = entry.path().to_path_buf();
                        if path.is_dir() { continue; }
                        let rel = path.strip_prefix(&self.workspace_root).unwrap_or(&path);
                        let rel_str = rel.to_string_lossy().to_lowercase();
                        if rel_str.starts_with("target/") || rel_str.starts_with(".git/") || rel_str.starts_with("node_modules/") {
                            continue;
                        }
                        let mut score: usize = 0;
                        let file_name = path.file_name().map(|f| f.to_string_lossy().to_lowercase()).unwrap_or_default();
                        for term in &query_terms {
                            if file_name.contains(term) { score += 10; }
                            if rel_str.contains(term) { score += 3; }
                        }
                        if score > 0 || query_terms.len() <= 2 {
                            if let Ok(content) = std::fs::read_to_string(&path) {
                                let content_lower = content.to_lowercase();
                                let chunk = &content_lower[..content_lower.len().min(5000)];
                                for term in &query_terms {
                                    let count = chunk.matches(term).count();
                                    score += count;
                                }
                            }
                        }
                        if score > 0 {
                            scored.push((rel.to_path_buf(), score));
                        }
                    }
                }
                scored.sort_by(|a, b| b.1.cmp(&a.1));
                scored.truncate(max_files);
                let results: Vec<String> = scored.iter()
                    .map(|(p, s)| format!("  [{:>4}] {}", s, p.display()))
                    .collect();
                if results.is_empty() {
                    Ok(ToolResult::ok(format!("No relevant files found for: {}", query)))
                } else {
                    Ok(ToolResult::ok(format!(
                        "Top {} files for '{}':\n{}",
                        results.len(), query, results.join("\n")
                    )))
                }
            }
            "summarize" => {
                let file_path = args["path"].as_str().unwrap_or("");
                if file_path.is_empty() {
                    return Ok(ToolResult::err("path field required for summarize action"));
                }
                let full_path = self.workspace_root.join(file_path);
                if !full_path.exists() {
                    return Ok(ToolResult::err(format!("file not found: {}", file_path)));
                }
                let content = std::fs::read_to_string(&full_path)?;
                let lines = content.lines().count();
                let chars = content.len();
                let tokens = estimate_token_count(&content);
                let mut functions = Vec::new();
                let mut structs = Vec::new();
                let mut imports = Vec::new();
                for line in content.lines() {
                    let trimmed = line.trim();
                    if trimmed.starts_with("fn ") || trimmed.starts_with("pub fn ") || trimmed.starts_with("async fn ") || trimmed.starts_with("pub async fn ") {
                        functions.push(trimmed.split('(').next().unwrap_or(trimmed).to_string());
                    }
                    if trimmed.starts_with("struct ") || trimmed.starts_with("pub struct ") {
                        structs.push(trimmed.split('{').next().unwrap_or(trimmed).split(':').next().unwrap_or(trimmed).to_string());
                    }
                    if trimmed.starts_with("use ") || trimmed.starts_with("import ") || trimmed.starts_with("#include") {
                        imports.push(trimmed.to_string());
                    }
                }
                let mut summary = format!(
                    "# {} Summary\n\n{} lines, {} chars, ~{} tokens\n",
                    file_path, lines, chars, tokens
                );
                if !imports.is_empty() {
                    summary.push_str(&format!("\n## Imports ({}):\n{}\n", imports.len(), imports[..imports.len().min(10)].join("\n")));
                }
                if !structs.is_empty() {
                    summary.push_str(&format!("\n## Structs ({}):\n{}\n", structs.len(), structs.join("\n")));
                }
                if !functions.is_empty() {
                    summary.push_str(&format!("\n## Functions ({}):\n{}\n", functions.len(), functions.join("\n")));
                }
                Ok(ToolResult::ok(summary))
            }
            _ => Ok(ToolResult::err("actions: select, summarize")),
        }
    }
}

// ---------------------------------------------------------------------------
// AgentMemoryTool
// ---------------------------------------------------------------------------

pub struct AgentMemoryTool { workspace_root: PathBuf }
impl AgentMemoryTool { pub fn new(r: PathBuf) -> Self { Self { workspace_root: r } } }

impl super::Tool for AgentMemoryTool {
    fn name(&self) -> &str { "agent_memory" }

    fn execute(&self, args: Value) -> Result<ToolResult> {
        let action = args["action"].as_str().unwrap_or("list");
        let memory_dir = self.workspace_root.join(".fevercode").join("memory");
        std::fs::create_dir_all(&memory_dir)?;

        match action {
            "store" => {
                let key = args["key"].as_str().unwrap_or("");
                let value = args["value"].as_str().unwrap_or("");
                let category = args["category"].as_str().unwrap_or("general");
                if key.is_empty() || value.is_empty() {
                    return Ok(ToolResult::err("key and value fields required for store action"));
                }
                let entry = serde_json::json!({
                    "key": key,
                    "value": value,
                    "category": category,
                    "timestamp": chrono::Utc::now().to_rfc3339(),
                });
                let file_name = format!("{}-{}.json", category, key.replace(['/', ' '], "_"));
                let path = memory_dir.join(&file_name);
                std::fs::write(&path, serde_json::to_string_pretty(&entry)?)?;
                Ok(ToolResult::ok(format!("Memory stored: {} [{}]", key, category)))
            }
            "recall" => {
                let query = args["query"].as_str().unwrap_or("");
                if query.is_empty() {
                    return Ok(ToolResult::err("query field required for recall action"));
                }
                let query_lower = query.to_lowercase();
                let mut results = Vec::new();
                if let Ok(rd) = std::fs::read_dir(&memory_dir) {
                    for entry in rd.flatten() {
                        let path = entry.path();
                        if path.extension().is_some_and(|e| e == "json") {
                            if let Ok(content) = std::fs::read_to_string(&path) {
                                if let Ok(data) = serde_json::from_str::<Value>(&content) {
                                    let key = data["key"].as_str().unwrap_or("");
                                    let val = data["value"].as_str().unwrap_or("");
                                    let cat = data["category"].as_str().unwrap_or("");
                                    let key_match = key.to_lowercase().contains(&query_lower);
                                    let val_match = val.to_lowercase().contains(&query_lower);
                                    if key_match || val_match {
                                        results.push(format!("[{}] {} = {}", cat, key, truncate_result(val, 200)));
                                    }
                                }
                            }
                        }
                    }
                }
                if results.is_empty() {
                    Ok(ToolResult::ok(format!("No memory found for: {}", query)))
                } else {
                    Ok(ToolResult::ok(format!("Found {} memories:\n{}", results.len(), results.join("\n"))))
                }
            }
            "list" => {
                let category = args["category"].as_str().unwrap_or("");
                let mut entries = Vec::new();
                if let Ok(rd) = std::fs::read_dir(&memory_dir) {
                    for entry in rd.flatten() {
                        let path = entry.path();
                        if path.extension().is_some_and(|e| e == "json") {
                            if let Ok(content) = std::fs::read_to_string(&path) {
                                if let Ok(data) = serde_json::from_str::<Value>(&content) {
                                    let cat = data["category"].as_str().unwrap_or("general");
                                    if category.is_empty() || cat == category {
                                        let key = data["key"].as_str().unwrap_or("?");
                                        let ts = data["timestamp"].as_str().unwrap_or("?");
                                        entries.push(format!("[{}] {} (stored {})", cat, key, ts));
                                    }
                                }
                            }
                        }
                    }
                }
                if entries.is_empty() {
                    Ok(ToolResult::ok("Memory store is empty."))
                } else {
                    Ok(ToolResult::ok(format!("Agent memory ({} entries):\n{}", entries.len(), entries.join("\n"))))
                }
            }
            "forget" => {
                let key = args["key"].as_str().unwrap_or("");
                if key.is_empty() {
                    return Ok(ToolResult::err("key field required for forget action"));
                }
                let mut deleted = 0;
                if let Ok(rd) = std::fs::read_dir(&memory_dir) {
                    for entry in rd.flatten() {
                        let path = entry.path();
                        if path.extension().is_some_and(|e| e == "json") {
                            if let Ok(content) = std::fs::read_to_string(&path) {
                                if let Ok(data) = serde_json::from_str::<Value>(&content) {
                                    if data["key"].as_str() == Some(key) {
                                        std::fs::remove_file(&path)?;
                                        deleted += 1;
                                    }
                                }
                            }
                        }
                    }
                }
                if deleted > 0 {
                    Ok(ToolResult::ok(format!("Forgot {} memory entries for: {}", deleted, key)))
                } else {
                    Ok(ToolResult::ok(format!("No memory found for: {}", key)))
                }
            }
            _ => Ok(ToolResult::err("actions: store, recall, list, forget")),
        }
    }
}

// ---------------------------------------------------------------------------
// LlmRouterTool
// ---------------------------------------------------------------------------

pub struct LlmRouterTool { workspace_root: PathBuf }
impl LlmRouterTool { pub fn new(r: PathBuf) -> Self { Self { workspace_root: r } } }

impl super::Tool for LlmRouterTool {
    fn name(&self) -> &str { "llm_router" }

    fn execute(&self, args: Value) -> Result<ToolResult> {
        let action = args["action"].as_str().unwrap_or("route");
        match action {
            "route" => {
                let task = args["task"].as_str().unwrap_or("");
                if task.is_empty() {
                    return Ok(ToolResult::err("task field required for route action"));
                }
                let task_lower = task.to_lowercase();
                let (recommended, reason) = classify_task_complexity(&task_lower);
                Ok(ToolResult::ok(format!(
                    "Task: {}\nRecommended model tier: {}\nReason: {}\n\nTiers:\n  fast    — simple lookups, formatting, small edits\n  balanced — general coding, debugging, refactoring\n  heavy   — architecture, complex reasoning, multi-file changes",
                    task, recommended, reason
                )))
            }
            "config" => {
                let dir = self.workspace_root.join(".fevercode");
                std::fs::create_dir_all(&dir)?;
                let config_path = dir.join("llm-router.toml");
                let config = r#"# FeverCode LLM Router Config
[defaults]
simple_edit = "fast"
refactoring = "balanced"
architecture = "heavy"
debugging = "balanced"
code_review = "balanced"
test_generation = "balanced"
documentation = "fast"

[tiers.fast]
description = "Fast, cheap models for simple tasks"
max_tokens = 4096

[tiers.balanced]
description = "Balanced speed and quality"
max_tokens = 8192

[tiers.heavy]
description = "Most capable models for complex tasks"
max_tokens = 16384
"#;
                std::fs::write(&config_path, config)?;
                Ok(ToolResult::ok(format!("LLM router config saved: {}", config_path.display())))
            }
            _ => Ok(ToolResult::err("actions: route, config")),
        }
    }
}

fn classify_task_complexity(task: &str) -> (&'static str, &'static str) {
    let heavy_keywords = ["architect", "design", "refactor", "migrate", "rewrite", "plan", "strategy", "multi-file", "system"];
    let fast_keywords = ["fix typo", "rename", "format", "lint", "comment", "log", "print", "list", "show", "read"];
    let balanced_keywords = ["implement", "debug", "fix", "feature", "test", "review", "optimize", "parse"];

    for kw in &heavy_keywords {
        if task.contains(kw) { return ("heavy", "Complex reasoning/architecture task"); }
    }
    for kw in &fast_keywords {
        if task.contains(kw) { return ("fast", "Simple/lookup task"); }
    }
    for kw in &balanced_keywords {
        if task.contains(kw) { return ("balanced", "General coding task"); }
    }
    ("balanced", "Default — balanced quality and speed")
}

// ---------------------------------------------------------------------------
// WorkspaceAnalyzerTool
// ---------------------------------------------------------------------------

pub struct WorkspaceAnalyzerTool { workspace_root: PathBuf }
impl WorkspaceAnalyzerTool { pub fn new(r: PathBuf) -> Self { Self { workspace_root: r } } }

impl super::Tool for WorkspaceAnalyzerTool {
    fn name(&self) -> &str { "workspace_analyzer" }

    fn execute(&self, args: Value) -> Result<ToolResult> {
        let action = args["action"].as_str().unwrap_or("overview");
        match action {
            "overview" => {
                let mut total_files = 0usize;
                let mut total_lines = 0usize;
                let mut total_size = 0u64;
                let mut languages: std::collections::HashMap<String, (usize, usize)> = std::collections::HashMap::new();
                if let Ok(entries) = ignore::WalkBuilder::new(&self.workspace_root)
                    .hidden(false)
                    .git_ignore(true)
                    .build()
                    .collect::<Result<Vec<_>, _>>()
                {
                    for entry in entries {
                        let path = entry.path();
                        if path.is_dir() { continue; }
                        total_files += 1;
                        if let Ok(meta) = std::fs::metadata(path) {
                            total_size += meta.len();
                        }
                        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("other");
                        if let Ok(content) = std::fs::read_to_string(path) {
                            let line_count = content.lines().count();
                            total_lines += line_count;
                            let lang = match ext {
                                "rs" => "Rust",
                                "ts" | "tsx" => "TypeScript",
                                "js" | "jsx" => "JavaScript",
                                "py" => "Python",
                                "go" => "Go",
                                "java" => "Java",
                                "toml" => "TOML",
                                "json" => "JSON",
                                "md" => "Markdown",
                                "html" => "HTML",
                                "css" | "scss" => "CSS",
                                "yaml" | "yml" => "YAML",
                                _ => "Other",
                            };
                            let entry = languages.entry(lang.to_string()).or_insert((0, 0));
                            entry.0 += 1;
                            entry.1 += line_count;
                        }
                    }
                }
                let mut lang_summary: Vec<String> = languages.iter()
                    .map(|(lang, (files, lines))| format!("  {:>12}: {} files, {} lines", lang, files, lines))
                    .collect();
                lang_summary.sort();
                Ok(ToolResult::ok(format!(
                    "# Workspace Overview\n\nTotal files: {}\nTotal lines: {}\nTotal size: {} KB ({:.1} MB)\n\n## Languages:\n{}",
                    total_files,
                    total_lines,
                    total_size / 1024,
                    total_size as f64 / (1024.0 * 1024.0),
                    lang_summary.join("\n")
                )))
            }
            "deps" => {
                let mut results = Vec::new();
                let cargo = self.workspace_root.join("Cargo.toml");
                if cargo.exists() {
                    if let Ok(content) = std::fs::read_to_string(&cargo) {
                        let dep_count = content.lines().filter(|l| l.contains("= ") || l.contains("version")).count();
                        results.push(format!("Cargo.toml: ~{} dependency entries", dep_count));
                    }
                }
                let pkg = self.workspace_root.join("package.json");
                if pkg.exists() {
                    if let Ok(content) = std::fs::read_to_string(&pkg) {
                        if let Ok(data) = serde_json::from_str::<Value>(&content) {
                            let deps = data.get("dependencies").and_then(|d| d.as_object()).map(|m| m.len()).unwrap_or(0);
                            let dev_deps = data.get("devDependencies").and_then(|d| d.as_object()).map(|m| m.len()).unwrap_or(0);
                            results.push(format!("package.json: {} deps, {} devDeps", deps, dev_deps));
                        }
                    }
                }
                let req = self.workspace_root.join("requirements.txt");
                if req.exists() {
                    if let Ok(content) = std::fs::read_to_string(&req) {
                        let count = content.lines().filter(|l| !l.trim().is_empty() && !l.trim().starts_with('#')).count();
                        results.push(format!("requirements.txt: {} packages", count));
                    }
                }
                if results.is_empty() {
                    results.push("No recognized dependency files found".to_string());
                }
                Ok(ToolResult::ok(format!("Dependency analysis:\n{}", results.join("\n"))))
            }
            "health" => {
                let mut checks = Vec::new();
                let git_dir = self.workspace_root.join(".git");
                checks.push(if git_dir.exists() { "✅ Git repository" } else { "⚠️ No git repository" });
                let readme = self.workspace_root.join("README.md");
                checks.push(if readme.exists() { "✅ README.md" } else { "⚠️ No README.md" });
                let license = self.workspace_root.join("LICENSE");
                let license_md = self.workspace_root.join("LICENSE.md");
                checks.push(if license.exists() || license_md.exists() { "✅ License file" } else { "⚠️ No license file" });
                let ci_dir = self.workspace_root.join(".github").join("workflows");
                checks.push(if ci_dir.exists() { "✅ CI/CD configured" } else { "⚠️ No CI/CD" });
                let fc_dir = self.workspace_root.join(".fevercode");
                checks.push(if fc_dir.exists() { "✅ FeverCode initialized" } else { "⚠️ FeverCode not initialized" });
                let tests = self.workspace_root.join("tests");
                checks.push(if tests.exists() { "✅ Test directory" } else { "⚠️ No tests/ directory" });
                let warnings: Vec<&&str> = checks.iter().filter(|c| c.starts_with("⚠")).collect();
                Ok(ToolResult::ok(format!(
                    "Project health ({} checks, {} warnings):\n{}",
                    checks.len(), warnings.len(), checks.join("\n")
                )))
            }
            _ => Ok(ToolResult::err("actions: overview, deps, health")),
        }
    }
}
