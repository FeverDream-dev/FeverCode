use anyhow::Result;
use serde_json::Value;
use std::path::PathBuf;

use super::ToolResult;

pub struct CopyFileTool { workspace_root: PathBuf }
impl CopyFileTool { pub fn new(workspace_root: PathBuf) -> Self { Self { workspace_root } } }
impl super::Tool for CopyFileTool {
    fn name(&self) -> &str { "copy_file" }
    fn execute(&self, args: Value) -> Result<ToolResult> {
        let src = args["src"].as_str().unwrap_or("");
        let dst = args["dst"].as_str().unwrap_or("");
        if src.is_empty() || dst.is_empty() { return Ok(ToolResult::err("src and dst required")); }
        let src_path = self.workspace_root.join(src);
        let dst_path = self.workspace_root.join(dst);
        if !src_path.exists() { return Ok(ToolResult::err(format!("not found: {}", src))); }
        if let Some(parent) = dst_path.parent() { std::fs::create_dir_all(parent)?; }
        std::fs::copy(&src_path, &dst_path)?;
        Ok(ToolResult::ok(format!("copied {} -> {}", src, dst)))
    }
}

pub struct MoveFileTool { workspace_root: PathBuf }
impl MoveFileTool { pub fn new(workspace_root: PathBuf) -> Self { Self { workspace_root } } }
impl super::Tool for MoveFileTool {
    fn name(&self) -> &str { "move_file" }
    fn execute(&self, args: Value) -> Result<ToolResult> {
        let src = args["src"].as_str().unwrap_or("");
        let dst = args["dst"].as_str().unwrap_or("");
        if src.is_empty() || dst.is_empty() { return Ok(ToolResult::err("src and dst required")); }
        let src_path = self.workspace_root.join(src);
        let dst_path = self.workspace_root.join(dst);
        if !src_path.exists() { return Ok(ToolResult::err(format!("not found: {}", src))); }
        if let Some(parent) = dst_path.parent() { std::fs::create_dir_all(parent)?; }
        std::fs::rename(&src_path, &dst_path)?;
        Ok(ToolResult::ok(format!("moved {} -> {}", src, dst)))
    }
}

pub struct DeleteFileTool { workspace_root: PathBuf }
impl DeleteFileTool { pub fn new(workspace_root: PathBuf) -> Self { Self { workspace_root } } }
impl super::Tool for DeleteFileTool {
    fn name(&self) -> &str { "delete_file" }
    fn execute(&self, args: Value) -> Result<ToolResult> {
        let path = args["path"].as_str().unwrap_or("");
        if path.is_empty() { return Ok(ToolResult::err("path required")); }
        let full = self.workspace_root.join(path);
        if !full.exists() { return Ok(ToolResult::err(format!("not found: {}", path))); }
        if full.is_dir() { std::fs::remove_dir_all(&full)?; } else { std::fs::remove_file(&full)?; }
        Ok(ToolResult::ok(format!("deleted {}", path)))
    }
}

pub struct MkDirTool { workspace_root: PathBuf }
impl MkDirTool { pub fn new(workspace_root: PathBuf) -> Self { Self { workspace_root } } }
impl super::Tool for MkDirTool {
    fn name(&self) -> &str { "mkdir" }
    fn execute(&self, args: Value) -> Result<ToolResult> {
        let path = args["path"].as_str().unwrap_or("");
        if path.is_empty() { return Ok(ToolResult::err("path required")); }
        std::fs::create_dir_all(self.workspace_root.join(path))?;
        Ok(ToolResult::ok(format!("created directory {}", path)))
    }
}

pub struct FileExistsTool { workspace_root: PathBuf }
impl FileExistsTool { pub fn new(workspace_root: PathBuf) -> Self { Self { workspace_root } } }
impl super::Tool for FileExistsTool {
    fn name(&self) -> &str { "file_exists" }
    fn execute(&self, args: Value) -> Result<ToolResult> {
        let path = args["path"].as_str().unwrap_or("");
        let full = self.workspace_root.join(path);
        let exists = full.exists();
        let meta = if exists {
            let m = std::fs::metadata(&full)?;
            format!("exists ({} bytes, {})", m.len(), if m.is_dir() { "dir" } else { "file" })
        } else { "not found".to_string() };
        Ok(ToolResult::ok(format!("{}: {}", path, meta)))
    }
}

pub struct DirectoryTreeTool { workspace_root: PathBuf }
impl DirectoryTreeTool { pub fn new(workspace_root: PathBuf) -> Self { Self { workspace_root } } }
impl super::Tool for DirectoryTreeTool {
    fn name(&self) -> &str { "directory_tree" }
    fn execute(&self, args: Value) -> Result<ToolResult> {
        let subdir = args["path"].as_str().unwrap_or(".");
        let max_depth = args["max_depth"].as_u64().map(|d| d as usize).unwrap_or(4);
        let root = self.workspace_root.join(subdir);
        if !root.exists() { return Ok(ToolResult::err(format!("not found: {}", subdir))); }
        let mut lines = Vec::new();
        build_tree(&root, &self.workspace_root, 0, max_depth, &mut lines);
        Ok(ToolResult::ok(lines.join("\n")))
    }
}

fn build_tree(dir: &std::path::Path, _base: &std::path::Path, depth: usize, max_depth: usize, lines: &mut Vec<String>) {
    if depth > max_depth { return; }
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    let mut entries: Vec<_> = entries.flatten().collect();
    entries.sort_by_key(|e| e.file_name());
    for entry in &entries {
        let indent = "  ".repeat(depth);
        let name = entry.file_name().to_string_lossy().to_string();
        if entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false) {
            lines.push(format!("{}{}/", indent, name));
            build_tree(&entry.path(), _base, depth + 1, max_depth, lines);
        } else {
            let size = std::fs::metadata(entry.path()).map(|m| m.len()).unwrap_or(0);
            lines.push(format!("{}{} ({}b)", indent, name, size));
        }
    }
}

pub struct CodeStatsTool { workspace_root: PathBuf }
impl CodeStatsTool { pub fn new(workspace_root: PathBuf) -> Self { Self { workspace_root } } }
impl super::Tool for CodeStatsTool {
    fn name(&self) -> &str { "code_stats" }
    fn execute(&self, _args: Value) -> Result<ToolResult> {
        use ignore::WalkBuilder;
        let mut stats: std::collections::HashMap<String, (usize, usize)> = std::collections::HashMap::new();
        let walker = WalkBuilder::new(&self.workspace_root).hidden(false).git_ignore(true).build();
        for entry in walker.flatten() {
            if entry.file_type().map(|ft| ft.is_file()).unwrap_or(false) {
                let ext = entry.path().extension().and_then(|e| e.to_str()).unwrap_or("").to_string();
                if ext.is_empty() { continue; }
                if let Ok(content) = std::fs::read_to_string(entry.path()) {
                    let lines = content.lines().count();
                    let bytes = content.len();
                    let entry = stats.entry(ext).or_insert((0, 0));
                    entry.0 += lines;
                    entry.1 += bytes;
                }
            }
        }
        let mut result: Vec<String> = vec!["Language  | Lines   | Bytes".to_string(), "----------|---------|-------".to_string()];
        let mut sorted: Vec<_> = stats.iter().collect();
        sorted.sort_by_key(|(_, (l, _))| std::cmp::Reverse(*l));
        for (ext, (lines, bytes)) in sorted.iter().take(20) {
            result.push(format!("{:9} | {:>7} | {:>6}", ext, lines, bytes));
        }
        Ok(ToolResult::ok(result.join("\n")))
    }
}

pub struct EnvVarTool;
impl EnvVarTool { pub fn new() -> Self { Self } }
impl super::Tool for EnvVarTool {
    fn name(&self) -> &str { "env_var" }
    fn execute(&self, args: Value) -> Result<ToolResult> {
        let name = args["name"].as_str().unwrap_or("");
        if name.is_empty() { return Ok(ToolResult::err("name required")); }
        match std::env::var(name) {
            Ok(val) => {
                let masked = if val.len() > 8 { format!("{}****", &val[..4]) } else { "****".to_string() };
                Ok(ToolResult::ok(format!("{} = {} (set)", name, masked)))
            }
            Err(_) => Ok(ToolResult::ok(format!("{} = (not set)", name))),
        }
    }
}

pub struct TodoFinderTool { workspace_root: PathBuf }
impl TodoFinderTool { pub fn new(workspace_root: PathBuf) -> Self { Self { workspace_root } } }
impl super::Tool for TodoFinderTool {
    fn name(&self) -> &str { "find_todos" }
    fn execute(&self, args: Value) -> Result<ToolResult> {
        use ignore::WalkBuilder;
        let patterns = args["patterns"].as_str().unwrap_or("TODO,FIXME,HACK,XXX,BUG").to_string();
        let tags: Vec<&str> = patterns.split(',').map(|s| s.trim()).collect();
        let walker = WalkBuilder::new(&self.workspace_root).hidden(false).git_ignore(true).build();
        let mut results = Vec::new();
        for entry in walker.flatten() {
            if entry.file_type().map(|ft| ft.is_file()).unwrap_or(false) {
                if let Ok(content) = std::fs::read_to_string(entry.path()) {
                    for (i, line) in content.lines().enumerate() {
                        for tag in &tags {
                            if line.contains(tag) {
                                if let Ok(rel) = entry.path().strip_prefix(&self.workspace_root) {
                                    results.push(format!("{}:{}: {}", rel.display(), i + 1, line.trim()));
                                }
                                break;
                            }
                        }
                        if results.len() >= 200 { break; }
                    }
                }
                if results.len() >= 200 { break; }
            }
        }
        if results.is_empty() { Ok(ToolResult::ok("no TODOs/FIXMEs found")) } else { Ok(ToolResult::ok(results.join("\n"))) }
    }
}

pub struct DuplicateFinderTool { workspace_root: PathBuf }
impl DuplicateFinderTool { pub fn new(workspace_root: PathBuf) -> Self { Self { workspace_root } } }
impl super::Tool for DuplicateFinderTool {
    fn name(&self) -> &str { "find_duplicates" }
    fn execute(&self, _args: Value) -> Result<ToolResult> {
        use ignore::WalkBuilder;
        let walker = WalkBuilder::new(&self.workspace_root).hidden(false).git_ignore(true).build();
        let mut size_map: std::collections::HashMap<u64, Vec<String>> = std::collections::HashMap::new();
        for entry in walker.flatten() {
            if entry.file_type().map(|ft| ft.is_file()).unwrap_or(false) {
                if let Ok(meta) = std::fs::metadata(entry.path()) {
                    if let Ok(rel) = entry.path().strip_prefix(&self.workspace_root) {
                        size_map.entry(meta.len()).or_default().push(rel.display().to_string());
                    }
                }
            }
        }
        let dupes: Vec<String> = size_map.iter()
            .filter(|(_, paths)| paths.len() > 1)
            .map(|(size, paths)| format!("{} bytes: {}", size, paths.join(", ")))
            .take(50)
            .collect();
        if dupes.is_empty() { Ok(ToolResult::ok("no duplicates found")) } else { Ok(ToolResult::ok(format!("potential duplicates:\n{}", dupes.join("\n")))) }
    }
}

pub struct ImportAnalyzerTool { workspace_root: PathBuf }
impl ImportAnalyzerTool { pub fn new(workspace_root: PathBuf) -> Self { Self { workspace_root } } }
impl super::Tool for ImportAnalyzerTool {
    fn name(&self) -> &str { "analyze_imports" }
    fn execute(&self, args: Value) -> Result<ToolResult> {
        let path = args["path"].as_str().unwrap_or("");
        let full = self.workspace_root.join(path);
        if !full.exists() { return Ok(ToolResult::err(format!("not found: {}", path))); }
        let content = std::fs::read_to_string(&full)?;
        let imports: Vec<&str> = content.lines()
            .filter(|l| {
                let t = l.trim();
                t.starts_with("use ") || t.starts_with("import ") || t.starts_with("from ") ||
                t.starts_with("#include") || t.starts_with("require(") || t.starts_with("const ") && t.contains("require(")
            })
            .collect();
        if imports.is_empty() { Ok(ToolResult::ok("no imports found")) } else { Ok(ToolResult::ok(imports.join("\n"))) }
    }
}

pub struct FileStatTool { workspace_root: PathBuf }
impl FileStatTool { pub fn new(workspace_root: PathBuf) -> Self { Self { workspace_root } } }
impl super::Tool for FileStatTool {
    fn name(&self) -> &str { "file_stat" }
    fn execute(&self, args: Value) -> Result<ToolResult> {
        let path = args["path"].as_str().unwrap_or("");
        let full = self.workspace_root.join(path);
        if !full.exists() { return Ok(ToolResult::err(format!("not found: {}", path))); }
        let meta = std::fs::metadata(&full)?;
        let content = std::fs::read_to_string(&full).unwrap_or_default();
        let lines = content.lines().count();
        let words = content.split_whitespace().count();
        let result = format!(
            "path: {}\nsize: {} bytes\nlines: {}\nwords: {}\ntype: {}\nmodified: {:?}",
            path, meta.len(), lines, words,
            if meta.is_dir() { "directory" } else { "file" },
            meta.modified()
        );
        Ok(ToolResult::ok(result))
    }
}

pub struct AppendFileTool { workspace_root: PathBuf }
impl AppendFileTool { pub fn new(workspace_root: PathBuf) -> Self { Self { workspace_root } } }
impl super::Tool for AppendFileTool {
    fn name(&self) -> &str { "append_file" }
    fn execute(&self, args: Value) -> Result<ToolResult> {
        let path = args["path"].as_str().unwrap_or("");
        let content = args["content"].as_str().unwrap_or("");
        if path.is_empty() { return Ok(ToolResult::err("path required")); }
        let full = self.workspace_root.join(path);
        use std::io::Write;
        let mut file = std::fs::OpenOptions::new().create(true).append(true).open(&full)?;
        write!(file, "{}", content)?;
        Ok(ToolResult::ok(format!("appended {} bytes to {}", content.len(), path)))
    }
}

pub struct HeadTailTool { workspace_root: PathBuf }
impl HeadTailTool { pub fn new(workspace_root: PathBuf) -> Self { Self { workspace_root } } }
impl super::Tool for HeadTailTool {
    fn name(&self) -> &str { "head_tail" }
    fn execute(&self, args: Value) -> Result<ToolResult> {
        let path = args["path"].as_str().unwrap_or("");
        let n = args["lines"].as_u64().unwrap_or(10) as usize;
        let mode = args["mode"].as_str().unwrap_or("head");
        let full = self.workspace_root.join(path);
        if !full.exists() { return Ok(ToolResult::err(format!("not found: {}", path))); }
        let content = std::fs::read_to_string(&full)?;
        let lines: Vec<&str> = content.lines().collect();
        let result = if mode == "tail" {
            lines.iter().rev().take(n).map(|l| l.to_string()).collect::<Vec<_>>().into_iter().rev().collect::<Vec<_>>()
        } else {
            lines.iter().take(n).map(|l| l.to_string()).collect()
        };
        Ok(ToolResult::ok(result.join("\n")))
    }
}

pub struct RegexSearchTool { workspace_root: PathBuf }
impl RegexSearchTool { pub fn new(workspace_root: PathBuf) -> Self { Self { workspace_root } } }
impl super::Tool for RegexSearchTool {
    fn name(&self) -> &str { "regex_search" }
    fn execute(&self, args: Value) -> Result<ToolResult> {
        let pattern = args["pattern"].as_str().unwrap_or("");
        if pattern.is_empty() { return Ok(ToolResult::err("pattern required")); }
        let re = match regex::Regex::new(pattern) {
            Ok(r) => r,
            Err(e) => return Ok(ToolResult::err(format!("invalid regex: {}", e))),
        };
        use ignore::WalkBuilder;
        let walker = WalkBuilder::new(&self.workspace_root).hidden(false).git_ignore(true).build();
        let mut matches = Vec::new();
        for entry in walker.flatten() {
            if entry.file_type().map(|ft| ft.is_file()).unwrap_or(false) {
                if let Ok(content) = std::fs::read_to_string(entry.path()) {
                    for (i, line) in content.lines().enumerate() {
                        if re.is_match(line) {
                            if let Ok(rel) = entry.path().strip_prefix(&self.workspace_root) {
                                matches.push(format!("{}:{}: {}", rel.display(), i + 1, line.trim()));
                            }
                        }
                        if matches.len() >= 100 { break; }
                    }
                }
                if matches.len() >= 100 { break; }
            }
        }
        if matches.is_empty() { Ok(ToolResult::ok("no matches")) } else { Ok(ToolResult::ok(matches.join("\n"))) }
    }
}

pub struct ReplaceInFileTool { workspace_root: PathBuf }
impl ReplaceInFileTool { pub fn new(workspace_root: PathBuf) -> Self { Self { workspace_root } } }
impl super::Tool for ReplaceInFileTool {
    fn name(&self) -> &str { "replace_all" }
    fn execute(&self, args: Value) -> Result<ToolResult> {
        let path = args["path"].as_str().unwrap_or("");
        let from = args["from"].as_str().unwrap_or("");
        let to = args["to"].as_str().unwrap_or("");
        if path.is_empty() || from.is_empty() { return Ok(ToolResult::err("path and from required")); }
        let full = self.workspace_root.join(path);
        if !full.exists() { return Ok(ToolResult::err(format!("not found: {}", path))); }
        let content = std::fs::read_to_string(&full)?;
        let count = content.matches(from).count();
        let updated = content.replace(from, to);
        std::fs::write(&full, updated)?;
        Ok(ToolResult::ok(format!("replaced {} occurrences in {}", count, path)))
    }
}

pub struct DiffFilesTool { workspace_root: PathBuf }
impl DiffFilesTool { pub fn new(workspace_root: PathBuf) -> Self { Self { workspace_root } } }
impl super::Tool for DiffFilesTool {
    fn name(&self) -> &str { "diff_files" }
    fn execute(&self, args: Value) -> Result<ToolResult> {
        let a = args["file_a"].as_str().unwrap_or("");
        let b = args["file_b"].as_str().unwrap_or("");
        if a.is_empty() || b.is_empty() { return Ok(ToolResult::err("file_a and file_b required")); }
        let ca = std::fs::read_to_string(self.workspace_root.join(a))?;
        let cb = std::fs::read_to_string(self.workspace_root.join(b))?;
        let diff = similar::TextDiff::from_lines(&ca, &cb).unified_diff().header(a, b).to_string();
        Ok(ToolResult::ok(diff))
    }
}
