use fever_core::{ExecutionContext, Tool, ToolSchema, Error, Result};
use async_trait::async_trait;
use serde_json::Value;

use std::path::Path;

pub struct FilesystemTool;

impl FilesystemTool {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for FilesystemTool {
    fn name(&self) -> &str {
        "filesystem"
    }

    fn description(&self) -> &str {
        "Read, write, list, and search files in the filesystem"
    }

    async fn execute(&self, args: Value, _context: &ExecutionContext) -> Result<Value> {
        let action = args.get("action")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let result = match action {
            "read" => self.read(args).await?,
            "write" => self.write(args).await?,
            "list" => self.list(args).await?,
            "exists" => self.exists(args).await?,
            "delete" => self.delete(args).await?,
            _ => return Err(Error::InvalidRequest("Unknown filesystem action".to_string()).into()),
        };

        Ok(result)
    }

    fn schema(&self) -> ToolSchema {
        ToolSchema {
            name: self.name().to_string(),
            description: self.description().to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "action": {
                        "type": "string",
                        "enum": ["read", "write", "list", "exists", "delete"],
                        "description": "Filesystem action to perform"
                    },
                    "path": {
                        "type": "string",
                        "description": "File or directory path"
                    },
                    "content": {
                        "type": "string",
                        "description": "Content to write (for write action)"
                    }
                }
            }),
        }
    }
}

impl FilesystemTool {
    async fn read(&self, args: Value) -> Result<Value> {
        let path = args.get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::InvalidRequest("path required".to_string()))?;

        let content = tokio::fs::read_to_string(path).await
            .map_err(|e| Error::Io(e))?;

        Ok(serde_json::json!({
            "content": content,
            "path": path
        }))
    }

    async fn write(&self, args: Value) -> Result<Value> {
        let path = args.get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::InvalidRequest("path required".to_string()))?;

        let content = args.get("content")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::InvalidRequest("content required".to_string()))?;

        tokio::fs::write(path, content).await
            .map_err(|e| Error::Io(e))?;

        Ok(serde_json::json!({
            "success": true,
            "path": path,
            "bytes_written": content.len()
        }))
    }

    async fn list(&self, args: Value) -> Result<Value> {
        let path = args.get("path")
            .and_then(|v| v.as_str())
            .unwrap_or(".");

        let mut entries = tokio::fs::read_dir(path).await
            .map_err(|e| Error::Io(e))?;

        let mut files = Vec::new();
        let mut dirs = Vec::new();

        while let Some(entry) = entries.next_entry().await
            .map_err(|e| Error::Io(e))? {
            let file_type = entry.file_type().await
                .map_err(|e| Error::Io(e))?;

            let name = entry.file_name().to_string_lossy().to_string();

            if file_type.is_file() {
                files.push(name);
            } else if file_type.is_dir() {
                dirs.push(name);
            }
        }

        Ok(serde_json::json!({
            "directories": dirs,
            "files": files,
            "path": path
        }))
    }

    async fn exists(&self, args: Value) -> Result<Value> {
        let path = args.get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::InvalidRequest("path required".to_string()))?;

        let exists = Path::new(path).exists();

        Ok(serde_json::json!({
            "exists": exists,
            "path": path
        }))
    }

    async fn delete(&self, args: Value) -> Result<Value> {
        let path = args.get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::InvalidRequest("path required".to_string()))?;

        tokio::fs::remove_file(path).await
            .map_err(|e| Error::Io(e))?;

        Ok(serde_json::json!({
            "success": true,
            "path": path
        }))
    }
}

impl Default for FilesystemTool {
    fn default() -> Self {
        Self::new()
    }
}
