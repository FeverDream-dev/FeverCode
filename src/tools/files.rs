use anyhow::Result;
use ignore::WalkBuilder;
use serde_json::Value;
use std::path::{Path, PathBuf};

use super::ToolResult;

pub struct ReadFileTool {
    workspace_root: PathBuf,
}

impl ReadFileTool {
    pub fn new(workspace_root: PathBuf) -> Self {
        Self { workspace_root }
    }
}

impl super::Tool for ReadFileTool {
    fn name(&self) -> &str {
        "read_file"
    }

    fn execute(&self, args: Value) -> Result<ToolResult> {
        let path_str = args["path"].as_str().unwrap_or("");
        let full_path = self.workspace_root.join(path_str);

        if !full_path.exists() {
            return Ok(ToolResult::err(format!("file not found: {}", path_str)));
        }

        let offset = args["offset"].as_u64().map(|o| o as usize);
        let limit = args["limit"].as_u64().map(|l| l as usize);

        let content = std::fs::read_to_string(&full_path)?;
        let lines: Vec<&str> = content.lines().collect();

        let start = offset.unwrap_or(0).min(lines.len());
        let end = limit
            .map(|l| (start + l).min(lines.len()))
            .unwrap_or(lines.len());

        let result: Vec<String> = lines[start..end]
            .iter()
            .enumerate()
            .map(|(i, line)| format!("{}: {}", start + i + 1, line))
            .collect();

        Ok(ToolResult::ok(result.join("\n")))
    }
}

pub struct ListFilesTool {
    workspace_root: PathBuf,
}

impl ListFilesTool {
    pub fn new(workspace_root: PathBuf) -> Self {
        Self { workspace_root }
    }
}

impl super::Tool for ListFilesTool {
    fn name(&self) -> &str {
        "list_files"
    }

    fn execute(&self, args: Value) -> Result<ToolResult> {
        let subdir = args["path"].as_str().unwrap_or(".");
        let max_depth = args["max_depth"].as_u64().map(|d| d as usize).unwrap_or(5);
        let root = self.workspace_root.join(subdir);

        if !root.exists() {
            return Ok(ToolResult::err(format!("directory not found: {}", subdir)));
        }

        let walker = WalkBuilder::new(&root)
            .max_depth(Some(max_depth))
            .hidden(false)
            .git_ignore(true)
            .build();

        let mut entries = Vec::new();
        for entry in walker.flatten().take(500) {
            if let Ok(rel) = entry.path().strip_prefix(&self.workspace_root) {
                let prefix = if entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false) {
                    "[DIR]  "
                } else {
                    "[FILE] "
                };
                entries.push(format!("{}{}", prefix, rel.display()));
            }
        }

        Ok(ToolResult::ok(entries.join("\n")))
    }
}

pub struct SearchTextTool {
    workspace_root: PathBuf,
}

impl SearchTextTool {
    pub fn new(workspace_root: PathBuf) -> Self {
        Self { workspace_root }
    }
}

impl super::Tool for SearchTextTool {
    fn name(&self) -> &str {
        "search_text"
    }

    fn execute(&self, args: Value) -> Result<ToolResult> {
        let pattern = args["pattern"].as_str().unwrap_or("");
        if pattern.is_empty() {
            return Ok(ToolResult::err("pattern is required"));
        }

        let glob = args["glob"].as_str().unwrap_or("*");
        let case_insensitive = args["case_insensitive"].as_bool().unwrap_or(true);

        let walker = WalkBuilder::new(&self.workspace_root)
            .max_depth(Some(10))
            .hidden(false)
            .git_ignore(true)
            .build();

        let mut matches = Vec::new();
        let search_pattern = if case_insensitive {
            pattern.to_ascii_lowercase()
        } else {
            pattern.to_string()
        };

        for entry in walker
            .flatten()
            .filter(|e| e.file_type().map(|ft| ft.is_file()).unwrap_or(false))
        {
            let path = entry.path();
            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
            if !glob_matches_path(glob, ext, path) {
                continue;
            }

            if let Ok(content) = std::fs::read_to_string(path) {
                for (line_num, line) in content.lines().enumerate() {
                    let haystack = if case_insensitive {
                        line.to_ascii_lowercase()
                    } else {
                        line.to_string()
                    };
                    if haystack.contains(&search_pattern) {
                        if let Ok(rel) = path.strip_prefix(&self.workspace_root) {
                            matches.push(format!("{}:{}: {}", rel.display(), line_num + 1, line));
                        }
                    }
                    if matches.len() >= 100 {
                        break;
                    }
                }
            }
            if matches.len() >= 100 {
                break;
            }
        }

        if matches.is_empty() {
            Ok(ToolResult::ok("no matches found"))
        } else {
            Ok(ToolResult::ok(matches.join("\n")))
        }
    }
}

pub struct WriteFileTool {
    workspace_root: PathBuf,
}

impl WriteFileTool {
    pub fn new(workspace_root: PathBuf) -> Self {
        Self { workspace_root }
    }
}

impl super::Tool for WriteFileTool {
    fn name(&self) -> &str {
        "write_file"
    }

    fn execute(&self, args: Value) -> Result<ToolResult> {
        let path_str = args["path"].as_str().unwrap_or("");
        let content = args["content"].as_str().unwrap_or("");
        let full_path = self.workspace_root.join(path_str);

        if let Some(parent) = full_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        std::fs::write(&full_path, content)?;
        Ok(ToolResult::ok(format!(
            "wrote {} bytes to {}",
            content.len(),
            path_str
        )))
    }
}

pub struct EditFileTool {
    workspace_root: PathBuf,
}

impl EditFileTool {
    pub fn new(workspace_root: PathBuf) -> Self {
        Self { workspace_root }
    }
}

impl super::Tool for EditFileTool {
    fn name(&self) -> &str {
        "edit_file"
    }

    fn execute(&self, args: Value) -> Result<ToolResult> {
        let path_str = args["path"].as_str().unwrap_or("");
        let old_string = args["old_string"].as_str().unwrap_or("");
        let new_string = args["new_string"].as_str().unwrap_or("");

        if path_str.is_empty() {
            return Ok(ToolResult::err("path is required"));
        }

        let full_path = self.workspace_root.join(path_str);
        if !full_path.exists() {
            return Ok(ToolResult::err(format!("file not found: {}", path_str)));
        }

        let current = std::fs::read_to_string(&full_path)?;
        if !current.contains(old_string) {
            return Ok(ToolResult::err(format!(
                "original text not found in {}. The file may have changed.",
                path_str
            )));
        }

        let updated = current.replacen(old_string, new_string, 1);
        std::fs::write(&full_path, updated)?;
        Ok(ToolResult::ok(format!(
            "edited {}: replaced {} bytes with {} bytes",
            path_str,
            old_string.len(),
            new_string.len()
        )))
    }
}

fn glob_matches_path(glob: &str, ext: &str, _path: &Path) -> bool {
    if glob == "*" || glob.is_empty() {
        return true;
    }
    let patterns: Vec<&str> = glob
        .split(',')
        .map(|s| s.trim().trim_start_matches("*."))
        .collect();
    if patterns.contains(&ext) || patterns.contains(&glob) {
        return true;
    }
    false
}
