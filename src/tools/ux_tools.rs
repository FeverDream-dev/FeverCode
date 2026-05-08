use anyhow::Result;
use serde_json::Value;
use std::path::PathBuf;

use super::ToolResult;

pub struct SessionExportTool { workspace_root: PathBuf }
impl SessionExportTool { pub fn new(r: PathBuf) -> Self { Self { workspace_root: r } } }
impl super::Tool for SessionExportTool {
    fn name(&self) -> &str { "session_export" }
    fn execute(&self, args: Value) -> Result<ToolResult> {
        let format = args["format"].as_str().unwrap_or("markdown");
        let session_dir = self.workspace_root.join(".fevercode").join("session");
        let events_file = session_dir.join("events.jsonl");
        if !events_file.exists() {
            return Ok(ToolResult::err("no session events found"));
        }
        let content = std::fs::read_to_string(&events_file)?;
        let events: Vec<Value> = content.lines()
            .filter(|l| !l.is_empty())
            .filter_map(|l| serde_json::from_str(l).ok())
            .collect();
        let output = match format {
            "json" => serde_json::to_string_pretty(&events)?,
            "markdown" | "md" => {
                let mut md = String::from("# FeverCode Session Export\n\n");
                for ev in &events {
                    let role = ev["role"].as_str().unwrap_or("unknown");
                    let ts = ev["timestamp"].as_str().unwrap_or("");
                    let content = ev["content"].as_str().unwrap_or("");
                    md.push_str(&format!("## {} ({})\n{}\n\n", role, ts, content));
                }
                md
            }
            _ => return Ok(ToolResult::err("formats: markdown, json")),
        };
        let out_path = self.workspace_root.join(format!("fevercode-session.{}", if format == "json" { "json" } else { "md" }));
        std::fs::write(&out_path, &output)?;
        Ok(ToolResult::ok(format!("exported {} events to {}", events.len(), out_path.display())))
    }
}

pub struct SessionResumeTool { workspace_root: PathBuf }
impl SessionResumeTool { pub fn new(r: PathBuf) -> Self { Self { workspace_root: r } } }
impl super::Tool for SessionResumeTool {
    fn name(&self) -> &str { "session_resume" }
    fn execute(&self, _args: Value) -> Result<ToolResult> {
        let session_dir = self.workspace_root.join(".fevercode").join("session");
        let events_file = session_dir.join("events.jsonl");
        if !events_file.exists() {
            return Ok(ToolResult::err("no previous session found"));
        }
        let content = std::fs::read_to_string(&events_file)?;
        let count = content.lines().filter(|l| !l.is_empty()).count();
        let mut summary = format!("Found {} events from previous session.\n", count);
        let last_n: Vec<&str> = content.lines().rev().take(5).collect();
        if !last_n.is_empty() {
            summary.push_str("\nLast 5 events:\n");
            for line in last_n.iter().rev() {
                if let Ok(ev) = serde_json::from_str::<Value>(line) {
                    let role = ev["role"].as_str().unwrap_or("?");
                    let msg = ev["content"].as_str().unwrap_or("");
                    let truncated: String = msg.chars().take(100).collect();
                    summary.push_str(&format!("- [{}] {}\n", role, truncated));
                }
            }
        }
        Ok(ToolResult::ok(summary))
    }
}

pub struct UndoRedoTool { workspace_root: PathBuf }
impl UndoRedoTool { pub fn new(r: PathBuf) -> Self { Self { workspace_root: r } } }
impl super::Tool for UndoRedoTool {
    fn name(&self) -> &str { "undo_redo" }
    fn execute(&self, args: Value) -> Result<ToolResult> {
        let action = args["action"].as_str().unwrap_or("undo");
        match action {
            "undo" => {
                let out = std::process::Command::new("git")
                    .args(["stash", "list"])
                    .current_dir(&self.workspace_root)
                    .output()?;
                if !out.status.success() {
                    return Ok(ToolResult::err("not a git repo or git unavailable"));
                }
                let list = String::from_utf8_lossy(&out.stdout);
                let count = list.lines().count();
                let restore = std::process::Command::new("git")
                    .args(["checkout", "HEAD", "--", "."])
                    .current_dir(&self.workspace_root)
                    .output()?;
                if restore.status.success() {
                    Ok(ToolResult::ok(format!("reverted working tree to HEAD ({} stash entries available)", count)))
                } else {
                    Ok(ToolResult::err(String::from_utf8_lossy(&restore.stderr).to_string()))
                }
            }
            "redo" => {
                let out = std::process::Command::new("git")
                    .args(["stash", "pop"])
                    .current_dir(&self.workspace_root)
                    .output()?;
                if out.status.success() {
                    Ok(ToolResult::ok("restored stashed changes"))
                } else {
                    Ok(ToolResult::err(String::from_utf8_lossy(&out.stderr).to_string()))
                }
            }
            _ => Ok(ToolResult::err("actions: undo, redo")),
        }
    }
}

pub struct ThemePaletteTool { workspace_root: PathBuf }
impl ThemePaletteTool { pub fn new(r: PathBuf) -> Self { Self { workspace_root: r } } }
impl super::Tool for ThemePaletteTool {
    fn name(&self) -> &str { "theme_palette" }
    fn execute(&self, args: Value) -> Result<ToolResult> {
        let action = args["action"].as_str().unwrap_or("list");
        let config_dir = self.workspace_root.join(".fevercode");
        let themes = vec![
            ("desert_gold", vec![("#c4a35a", "gold"), ("#1a1a2e", "bg"), ("#e0e0e0", "fg"), ("#e74c3c", "error"), ("#2ecc71", "success")]),
            ("nile_blue", vec![("#2980b9", "primary"), ("#0d1b2a", "bg"), ("#e0e0e0", "fg"), ("#e74c3c", "error"), ("#27ae60", "success")]),
            ("pharaoh_red", vec![("#c0392b", "primary"), ("#1a0a0a", "bg"), ("#f5e6cc", "fg"), ("#ff6b6b", "error"), ("#f39c12", "success")]),
            ("obsidian", vec![("#a0a0a0", "fg"), ("#0a0a0a", "bg"), ("#d4d4d4", "fg"), ("#ff5555", "error"), ("#50fa7b", "success")]),
        ];
        match action {
            "list" => {
                let mut out = String::from("Available themes:\n");
                for (name, colors) in &themes {
                    out.push_str(&format!("\n  {}:\n", name));
                    for (hex, label) in colors {
                        out.push_str(&format!("    {} = {}\n", label, hex));
                    }
                }
                Ok(ToolResult::ok(out))
            }
            "apply" => {
                let theme_name = args["theme"].as_str().unwrap_or("");
                let found = themes.iter().find(|(n, _)| *n == theme_name);
                match found {
                    Some((name, colors)) => {
                        std::fs::create_dir_all(&config_dir)?;
                        let theme_json: Vec<(&str, &str)> = colors.iter().map(|(h, l)| (*l, *h)).collect();
                        let content = serde_json::to_string_pretty(&serde_json::json!({
                            "theme": name,
                            "colors": serde_json::to_value(&theme_json)?,
                        }))?;
                        std::fs::write(config_dir.join("theme.json"), content)?;
                        Ok(ToolResult::ok(format!("applied theme: {}", name)))
                    }
                    None => Ok(ToolResult::err(format!("unknown theme: {}. Use 'list' to see available.", theme_name))),
                }
            }
            _ => Ok(ToolResult::err("actions: list, apply")),
        }
    }
}

pub struct DiffViewerTool { workspace_root: PathBuf }
impl DiffViewerTool { pub fn new(r: PathBuf) -> Self { Self { workspace_root: r } } }
impl super::Tool for DiffViewerTool {
    fn name(&self) -> &str { "diff_viewer" }
    fn execute(&self, args: Value) -> Result<ToolResult> {
        let file1 = args["file1"].as_str().unwrap_or("");
        let file2 = args["file2"].as_str().unwrap_or("");
        if file1.is_empty() || file2.is_empty() {
            return Ok(ToolResult::err("file1 and file2 required"));
        }
        let path1 = self.workspace_root.join(file1);
        let path2 = self.workspace_root.join(file2);
        if !path1.exists() { return Ok(ToolResult::err(format!("not found: {}", file1))); }
        if !path2.exists() { return Ok(ToolResult::err(format!("not found: {}", file2))); }
        let content1 = std::fs::read_to_string(&path1)?;
        let content2 = std::fs::read_to_string(&path2)?;
        use similar::{ChangeTag, TextDiff};
        let diff = TextDiff::from_lines(&content1, &content2);
        let mut result = String::new();
        for change in diff.iter_all_changes() {
            let sign = match change.tag() {
                ChangeTag::Delete => "-",
                ChangeTag::Insert => "+",
                ChangeTag::Equal => " ",
            };
            result.push_str(&format!("{}{}", sign, change));
        }
        if result.is_empty() {
            result = String::from("files are identical");
        }
        let mut out = format!("diff: {} vs {}\n\n", file1, file2);
        out.push_str(&result);
        Ok(ToolResult::ok(out))
    }
}

pub struct SyntaxHighlightTool { workspace_root: PathBuf }
impl SyntaxHighlightTool { pub fn new(r: PathBuf) -> Self { Self { workspace_root: r } } }
impl super::Tool for SyntaxHighlightTool {
    fn name(&self) -> &str { "syntax_highlight" }
    fn execute(&self, args: Value) -> Result<ToolResult> {
        let file = args["file"].as_str().unwrap_or("");
        if file.is_empty() { return Ok(ToolResult::err("file required")); }
        let path = self.workspace_root.join(file);
        if !path.exists() { return Ok(ToolResult::err(format!("not found: {}", file))); }
        let content = std::fs::read_to_string(&path)?;
        let ext = std::path::Path::new(file).extension()
            .and_then(|e| e.to_str())
            .unwrap_or("txt");
        let lang = match ext {
            "rs" => "rust",
            "js" | "mjs" | "cjs" => "javascript",
            "ts" => "typescript",
            "py" => "python",
            "go" => "go",
            "java" => "java",
            "rb" => "ruby",
            "sh" | "bash" => "bash",
            "json" | "jsonl" => "json",
            "toml" => "toml",
            "yaml" | "yml" => "yaml",
            "md" => "markdown",
            "html" | "htm" => "html",
            "css" => "css",
            "sql" => "sql",
            "c" | "h" => "c",
            "cpp" | "cc" | "cxx" | "hpp" => "cpp",
            _ => ext,
        };
        let lines: Vec<&str> = content.lines().collect();
        let total = lines.len();
        let start = args["start"].as_u64().unwrap_or(1).max(1) as usize;
        let end = args["end"].as_u64().unwrap_or(0);
        let end = if end == 0 { total } else { (end as usize).min(total) };
        let shown: Vec<&str> = lines.iter().skip(start - 1).take(end - start + 1).copied().collect();
        let mut out = format!("```{}\n", lang);
        for (i, line) in shown.iter().enumerate() {
            out.push_str(&format!("{:>4} | {}\n", start + i, line));
        }
        out.push_str("```\n");
        out.push_str(&format!("\nShowing lines {}-{} of {} ({})", start, (start - 1 + shown.len()).min(total), total, lang));
        Ok(ToolResult::ok(out))
    }
}

pub struct ProgressTrackerTool { workspace_root: PathBuf }
impl ProgressTrackerTool { pub fn new(r: PathBuf) -> Self { Self { workspace_root: r } } }
impl super::Tool for ProgressTrackerTool {
    fn name(&self) -> &str { "progress" }
    fn execute(&self, args: Value) -> Result<ToolResult> {
        let action = args["action"].as_str().unwrap_or("status");
        let progress_dir = self.workspace_root.join(".fevercode").join("progress");
        std::fs::create_dir_all(&progress_dir)?;
        match action {
            "start" => {
                let task = args["task"].as_str().unwrap_or("unnamed");
                let total = args["total"].as_u64().unwrap_or(100);
                let entry = serde_json::json!({
                    "task": task,
                    "total": total,
                    "done": 0,
                    "started_at": chrono::Utc::now().to_rfc3339(),
                });
                let filename = task.replace(' ', "_");
                std::fs::write(progress_dir.join(format!("{}.json", filename)), serde_json::to_string_pretty(&entry)?)?;
                Ok(ToolResult::ok(format!("started tracking: {} (0/{})", task, total)))
            }
            "update" => {
                let task = args["task"].as_str().unwrap_or("unnamed");
                let done = args["done"].as_u64().unwrap_or(0);
                let filename = task.replace(' ', "_");
                let path = progress_dir.join(format!("{}.json", filename));
                if !path.exists() { return Ok(ToolResult::err(format!("no progress tracker for: {}", task))); }
                let mut entry: Value = serde_json::from_str(&std::fs::read_to_string(&path)?)?;
                entry["done"] = serde_json::json!(done);
                let total = entry["total"].as_u64().unwrap_or(100);
                let pct = if total > 0 { (done as f64 / total as f64 * 100.0) as u32 } else { 100 };
                let bar: String = std::iter::repeat_n('=', (pct / 2) as usize)
                    .chain(std::iter::repeat_n(' ', 50 - (pct / 2) as usize))
                    .collect();
                std::fs::write(&path, serde_json::to_string_pretty(&entry)?)?;
                Ok(ToolResult::ok(format!("[{}] {}% ({}/{}) {}", bar, pct, done, total, task)))
            }
            "status" => {
                let mut out = String::from("Active progress trackers:\n");
                let mut found = false;
                if progress_dir.exists() {
                    for entry in std::fs::read_dir(&progress_dir)? {
                        let entry = entry?;
                        if entry.path().extension().and_then(|e| e.to_str()) == Some("json") {
                            found = true;
                            let content = std::fs::read_to_string(entry.path())?;
                            if let Ok(data) = serde_json::from_str::<Value>(&content) {
                                let task = data["task"].as_str().unwrap_or("?");
                                let done = data["done"].as_u64().unwrap_or(0);
                                let total = data["total"].as_u64().unwrap_or(0);
                                let pct = if total > 0 { (done as f64 / total as f64 * 100.0) as u32 } else { 100 };
                                out.push_str(&format!("  {} — {}/{} ({}%)\n", task, done, total, pct));
                            }
                        }
                    }
                }
                if !found { out.push_str("  (none)\n"); }
                Ok(ToolResult::ok(out))
            }
            _ => Ok(ToolResult::err("actions: start, update, status")),
        }
    }
}

pub struct BookmarkTool { workspace_root: PathBuf }
impl BookmarkTool { pub fn new(r: PathBuf) -> Self { Self { workspace_root: r } } }
impl super::Tool for BookmarkTool {
    fn name(&self) -> &str { "bookmark" }
    fn execute(&self, args: Value) -> Result<ToolResult> {
        let action = args["action"].as_str().unwrap_or("list");
        let bm_dir = self.workspace_root.join(".fevercode").join("bookmarks");
        std::fs::create_dir_all(&bm_dir)?;
        match action {
            "add" => {
                let name = args["name"].as_str().unwrap_or("");
                let path = args["path"].as_str().unwrap_or("");
                let line = args["line"].as_u64().unwrap_or(0);
                let note = args["note"].as_str().unwrap_or("");
                if name.is_empty() { return Ok(ToolResult::err("name required")); }
                let entry = serde_json::json!({ "path": path, "line": line, "note": note });
                std::fs::write(bm_dir.join(format!("{}.json", name)), serde_json::to_string_pretty(&entry)?)?;
                Ok(ToolResult::ok(format!("bookmark '{}' saved", name)))
            }
            "get" => {
                let name = args["name"].as_str().unwrap_or("");
                if name.is_empty() { return Ok(ToolResult::err("name required")); }
                let path = bm_dir.join(format!("{}.json", name));
                if !path.exists() { return Ok(ToolResult::err(format!("bookmark '{}' not found", name))); }
                let content = std::fs::read_to_string(&path)?;
                let data: Value = serde_json::from_str(&content)?;
                Ok(ToolResult::ok(format!("{}:{} — {}", data["path"].as_str().unwrap_or("?"), data["line"].as_u64().unwrap_or(0), data["note"].as_str().unwrap_or(""))))
            }
            "list" => {
                let mut out = String::from("Bookmarks:\n");
                let mut found = false;
                for entry in std::fs::read_dir(&bm_dir)? {
                    let entry = entry?;
                    if entry.path().extension().and_then(|e| e.to_str()) == Some("json") {
                        found = true;
                        let name = entry.path().file_stem().and_then(|n| n.to_str()).unwrap_or("?").to_string();
                        let content = std::fs::read_to_string(entry.path())?;
                        let data: Value = serde_json::from_str(&content).unwrap_or_default();
                        out.push_str(&format!("  {} → {}:{} — {}\n", name, data["path"].as_str().unwrap_or("?"), data["line"].as_u64().unwrap_or(0), data["note"].as_str().unwrap_or("")));
                    }
                }
                if !found { out.push_str("  (none)\n"); }
                Ok(ToolResult::ok(out))
            }
            "remove" => {
                let name = args["name"].as_str().unwrap_or("");
                if name.is_empty() { return Ok(ToolResult::err("name required")); }
                let path = bm_dir.join(format!("{}.json", name));
                if !path.exists() { return Ok(ToolResult::err(format!("bookmark '{}' not found", name))); }
                std::fs::remove_file(path)?;
                Ok(ToolResult::ok(format!("bookmark '{}' removed", name)))
            }
            _ => Ok(ToolResult::err("actions: add, get, list, remove")),
        }
    }
}

pub struct WorkspaceNotesTool { workspace_root: PathBuf }
impl WorkspaceNotesTool { pub fn new(r: PathBuf) -> Self { Self { workspace_root: r } } }
impl super::Tool for WorkspaceNotesTool {
    fn name(&self) -> &str { "notes" }
    fn execute(&self, args: Value) -> Result<ToolResult> {
        let action = args["action"].as_str().unwrap_or("list");
        let notes_dir = self.workspace_root.join(".fevercode").join("notes");
        std::fs::create_dir_all(&notes_dir)?;
        match action {
            "add" => {
                let title = args["title"].as_str().unwrap_or("");
                let body = args["body"].as_str().unwrap_or("");
                if title.is_empty() { return Ok(ToolResult::err("title required")); }
                let entry = serde_json::json!({
                    "title": title,
                    "body": body,
                    "created": chrono::Utc::now().to_rfc3339(),
                });
                let filename = title.replace(' ', "_");
                std::fs::write(notes_dir.join(format!("{}.json", filename)), serde_json::to_string_pretty(&entry)?)?;
                Ok(ToolResult::ok(format!("note '{}' saved", title)))
            }
            "list" => {
                let mut out = String::from("Workspace notes:\n");
                let mut found = false;
                for entry in std::fs::read_dir(&notes_dir)? {
                    let entry = entry?;
                    if entry.path().extension().and_then(|e| e.to_str()) == Some("json") {
                        found = true;
                        let name = entry.path().file_stem().and_then(|n| n.to_str()).unwrap_or("?").to_string();
                        let content = std::fs::read_to_string(entry.path())?;
                        let data: Value = serde_json::from_str(&content).unwrap_or_default();
                        out.push_str(&format!("  {} — {} (created: {})\n", name, data["title"].as_str().unwrap_or("?"), data["created"].as_str().unwrap_or("?")));
                    }
                }
                if !found { out.push_str("  (none)\n"); }
                Ok(ToolResult::ok(out))
            }
            "read" => {
                let title = args["title"].as_str().unwrap_or("");
                if title.is_empty() { return Ok(ToolResult::err("title required")); }
                let filename = title.replace(' ', "_");
                let path = notes_dir.join(format!("{}.json", filename));
                if !path.exists() { return Ok(ToolResult::err(format!("note '{}' not found", title))); }
                let content = std::fs::read_to_string(&path)?;
                let data: Value = serde_json::from_str(&content)?;
                Ok(ToolResult::ok(format!("{}\n\n{}", data["title"].as_str().unwrap_or(""), data["body"].as_str().unwrap_or(""))))
            }
            "remove" => {
                let title = args["title"].as_str().unwrap_or("");
                if title.is_empty() { return Ok(ToolResult::err("title required")); }
                let filename = title.replace(' ', "_");
                let path = notes_dir.join(format!("{}.json", filename));
                if !path.exists() { return Ok(ToolResult::err(format!("note '{}' not found", title))); }
                std::fs::remove_file(path)?;
                Ok(ToolResult::ok(format!("note '{}' removed", title)))
            }
            _ => Ok(ToolResult::err("actions: add, list, read, remove")),
        }
    }
}

pub struct SnapshotTool { workspace_root: PathBuf }
impl SnapshotTool { pub fn new(r: PathBuf) -> Self { Self { workspace_root: r } } }
impl super::Tool for SnapshotTool {
    fn name(&self) -> &str { "snapshot" }
    fn execute(&self, args: Value) -> Result<ToolResult> {
        let action = args["action"].as_str().unwrap_or("create");
        let snap_dir = self.workspace_root.join(".fevercode").join("snapshots");
        std::fs::create_dir_all(&snap_dir)?;
        match action {
            "create" => {
                let name = args["name"].as_str().unwrap_or("unnamed");
                let out = std::process::Command::new("git")
                    .args(["stash", "push", "-m", &format!("fever-snapshot:{}", name)])
                    .current_dir(&self.workspace_root)
                    .output()?;
                if out.status.success() {
                    let ts = chrono::Utc::now().format("%Y%m%d_%H%M%S");
                    let entry = serde_json::json!({ "name": name, "timestamp": ts.to_string(), "stash": true });
                    std::fs::write(snap_dir.join(format!("{}_{}.json", name, ts)), serde_json::to_string_pretty(&entry)?)?;
                    Ok(ToolResult::ok(format!("snapshot '{}' created", name)))
                } else {
                    Ok(ToolResult::err(String::from_utf8_lossy(&out.stderr).to_string()))
                }
            }
            "list" => {
                let mut out = String::from("Snapshots:\n");
                let mut found = false;
                for entry in std::fs::read_dir(&snap_dir)? {
                    let entry = entry?;
                    if entry.path().extension().and_then(|e| e.to_str()) == Some("json") {
                        found = true;
                        let content = std::fs::read_to_string(entry.path())?;
                        let data: Value = serde_json::from_str(&content).unwrap_or_default();
                        out.push_str(&format!("  {} — {}\n", data["name"].as_str().unwrap_or("?"), data["timestamp"].as_str().unwrap_or("?")));
                    }
                }
                if !found { out.push_str("  (none)\n"); }
                Ok(ToolResult::ok(out))
            }
            "restore" => {
                let name = args["name"].as_str().unwrap_or("");
                if name.is_empty() { return Ok(ToolResult::err("name required")); }
                let out = std::process::Command::new("git")
                    .args(["stash", "list"])
                    .current_dir(&self.workspace_root)
                    .output()?;
                let list = String::from_utf8_lossy(&out.stdout);
                let stash_line = list.lines().find(|l| l.contains(&format!("fever-snapshot:{}", name)));
                match stash_line {
                    Some(line) => {
                        let stash_ref = line.split(':').next().unwrap_or("stash@{0}");
                        let pop = std::process::Command::new("git")
                            .args(["stash", "pop", stash_ref])
                            .current_dir(&self.workspace_root)
                            .output()?;
                        if pop.status.success() {
                            Ok(ToolResult::ok(format!("snapshot '{}' restored", name)))
                        } else {
                            Ok(ToolResult::err(String::from_utf8_lossy(&pop.stderr).to_string()))
                        }
                    }
                    None => Ok(ToolResult::err(format!("snapshot '{}' not found in stash", name))),
                }
            }
            _ => Ok(ToolResult::err("actions: create, list, restore")),
        }
    }
}
