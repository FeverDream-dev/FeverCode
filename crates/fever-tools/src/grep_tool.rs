use fever_core::{ExecutionContext, Tool, ToolSchema, Error, Result};
use async_trait::async_trait;
use serde_json::Value;
use regex::Regex;
use ignore::WalkBuilder;
use wildmatch::WildMatch;

pub struct GrepTool {
    root_dir: std::path::PathBuf,
}

impl GrepTool {
    pub fn new() -> Self {
        Self {
            root_dir: std::env::current_dir().unwrap_or_default(),
        }
    }

    pub fn with_root_dir(mut self, dir: std::path::PathBuf) -> Self {
        self.root_dir = dir;
        self
    }
}

#[async_trait]
impl Tool for GrepTool {
    fn name(&self) -> &str {
        "grep"
    }

    fn description(&self) -> &str {
        "Search for patterns in files"
    }

    async fn execute(&self, args: Value, _context: &ExecutionContext) -> Result<Value> {
        let pattern = args.get("pattern")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::InvalidRequest("pattern required".to_string()))?;

        let file_pattern = args.get("file_pattern")
            .and_then(|v| v.as_str());

        let max_results = args.get("max_results")
            .and_then(|v| v.as_u64())
            .unwrap_or(100);

        let matches: Vec<GrepMatch> = self.search(pattern, file_pattern, max_results).await?;

        Ok(serde_json::json!({
            "pattern": pattern,
            "matches": matches,
            "count": matches.len()
        }))
    }

    fn schema(&self) -> ToolSchema {
        ToolSchema {
            name: self.name().to_string(),
            description: self.description().to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "pattern": {
                        "type": "string",
                        "description": "Regex pattern to search for"
                    },
                    "file_pattern": {
                        "type": "string",
                        "description": "File pattern to filter (e.g., *.rs)"
                    },
                    "max_results": {
                        "type": "integer",
                        "description": "Maximum number of results"
                    }
                },
                "required": ["pattern"]
            }),
        }
    }
}

impl GrepTool {
    async fn search(
        &self,
        pattern: &str,
        file_pattern: Option<&str>,
        max_results: u64,
    ) -> Result<Vec<GrepMatch>> {
        let regex = Regex::new(pattern)
            .map_err(|e| Error::Parse(e.to_string()))?;

        let walker = WalkBuilder::new(&self.root_dir)
            .hidden(false)
            .build();

        let mut matches = Vec::new();

        for entry in walker {
            let entry = entry
                .map_err(|e| Error::Parse(e.to_string()))?;

            let path = entry.path();

            if !path.is_file() {
                continue;
            }

            if let Some(fp) = file_pattern {
                let file_name = path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("");

                if !WildMatch::new(fp).matches(file_name) {
                    continue;
                }
            }

            let content = tokio::fs::read_to_string(path).await
                .map_err(|e| Error::Io(e))?;

            for (line_num, line) in content.lines().enumerate() {
                if regex.is_match(line) {
                    matches.push(GrepMatch {
                        file: path.display().to_string(),
                        line: line_num + 1,
                        content: line.to_string(),
                    });

                    if matches.len() >= max_results as usize {
                        return Ok(matches);
                    }
                }
            }
        }

        Ok(matches)
    }
}

#[derive(Debug, Clone, serde::Serialize)]
struct GrepMatch {
    file: String,
    line: usize,
    content: String,
}

impl Default for GrepTool {
    fn default() -> Self {
        Self::new()
    }
}
