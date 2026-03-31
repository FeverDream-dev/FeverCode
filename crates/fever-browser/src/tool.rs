use crate::client::PageSnapshot;
use async_trait::async_trait;
use fever_core::{ExecutionContext, Result, Tool, ToolSchema};
use serde_json::Value;

pub struct BrowserTool {
    enabled: bool,
}

impl BrowserTool {
    pub fn new() -> Self {
        Self { enabled: true }
    }

    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
}

#[async_trait]
impl Tool for BrowserTool {
    fn name(&self) -> &str {
        "browser"
    }

    fn description(&self) -> &str {
        "Interact with Chrome/Chromium browser for web debugging, DOM inspection, and screenshots"
    }

    async fn execute(&self, args: Value, _context: &ExecutionContext) -> Result<Value> {
        if !self.enabled {
            return Ok(serde_json::json!({
                "error": "Browser tool is disabled",
                "success": false
            }));
        }

        let action = args
            .get("action")
            .and_then(|v| v.as_str())
            .unwrap_or("snapshot");

        let result = match action {
            "snapshot" => self.get_snapshot(args).await?,
            "navigate" => self.navigate(args).await?,
            "click" => self.click(args).await?,
            "screenshot" => self.screenshot(args).await?,
            "evaluate" => self.evaluate(args).await?,
            _ => {
                return Ok(serde_json::json!({
                    "error": format!("Unknown action: {}", action),
                    "success": false
                }));
            }
        };

        Ok(serde_json::json!({
            "success": true,
            "result": result
        }))
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
                        "enum": ["snapshot", "navigate", "click", "screenshot", "evaluate"],
                        "description": "Browser action to perform"
                    },
                    "url": {
                        "type": "string",
                        "description": "URL to navigate to (for navigate action)"
                    },
                    "uid": {
                        "type": "string",
                        "description": "Element UID to interact with (for click action)"
                    },
                    "script": {
                        "type": "string",
                        "description": "JavaScript to evaluate (for evaluate action)"
                    }
                }
            }),
        }
    }
}

impl BrowserTool {
    async fn get_snapshot(&self, _args: Value) -> Result<Value> {
        Ok(serde_json::json!({
            "snapshot": PageSnapshot {
                url: "about:blank".to_string(),
                title: "Browser Demo".to_string(),
                elements: vec![],
                console_messages: vec![],
                network_requests: vec![],
                timestamp: chrono::Utc::now(),
            },
            "message": "Browser snapshot (placeholder - requires Chrome MCP connection)"
        }))
    }

    async fn navigate(&self, args: Value) -> Result<Value> {
        let url = args.get("url").and_then(|v| v.as_str()).unwrap_or("");

        Ok(serde_json::json!({
            "message": format!("Navigate to {} (placeholder - requires Chrome MCP)", url)
        }))
    }

    async fn click(&self, args: Value) -> Result<Value> {
        let uid = args.get("uid").and_then(|v| v.as_str()).unwrap_or("");

        Ok(serde_json::json!({
            "message": format!("Click element {} (placeholder - requires Chrome MCP)", uid)
        }))
    }

    async fn screenshot(&self, _args: Value) -> Result<Value> {
        Ok(serde_json::json!({
            "message": "Screenshot captured (placeholder - requires Chrome MCP)"
        }))
    }

    async fn evaluate(&self, args: Value) -> Result<Value> {
        let script = args.get("script").and_then(|v| v.as_str()).unwrap_or("");

        Ok(serde_json::json!({
            "message": format!("Evaluate script (placeholder - requires Chrome MCP): {}", script)
        }))
    }
}

impl Default for BrowserTool {
    fn default() -> Self {
        Self::new()
    }
}
