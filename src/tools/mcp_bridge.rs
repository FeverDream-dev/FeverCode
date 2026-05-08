use anyhow::Result;
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::mcp::McpClient;
use super::ToolResult;

pub struct McpBridgeTool {
    server_name: String,
    tool_name: String,
    description: String,
    input_schema: Option<Value>,
    client: Arc<Mutex<McpClient>>,
}

impl McpBridgeTool {
    pub fn new(
        server_name: String,
        tool_name: String,
        description: String,
        input_schema: Option<Value>,
        client: Arc<Mutex<McpClient>>,
    ) -> Self {
        Self {
            server_name,
            tool_name,
            description,
            input_schema,
            client,
        }
    }

    pub fn mcp_tool_name(&self) -> &str {
        &self.tool_name
    }

    pub fn server_name(&self) -> &str {
        &self.server_name
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn input_schema(&self) -> Option<&Value> {
        self.input_schema.as_ref()
    }
}

impl super::Tool for McpBridgeTool {
    fn name(&self) -> &str {
        &self.tool_name
    }

    fn execute(&self, args: Value) -> Result<ToolResult> {
        let server = self.server_name.clone();
        let tool = self.tool_name.clone();
        let client = self.client.clone();

        let rt = tokio::runtime::Handle::current();
        let result = rt.block_on(async {
            let mut client = client.lock().await;
            client.call_tool(&server, &tool, args).await
        });

        match result {
            Ok(val) => {
                let output = if val.is_null() {
                    "MCP tool returned null".to_string()
                } else if let Some(content) = val.get("content").and_then(|c| c.as_array()) {
                    content
                        .iter()
                        .filter_map(|item| {
                            if item.get("type").and_then(|t| t.as_str()) == Some("text") {
                                item.get("text").and_then(|t| t.as_str()).map(String::from)
                            } else {
                                None
                            }
                        })
                        .collect::<Vec<_>>()
                        .join("\n")
                } else {
                    serde_json::to_string_pretty(&val).unwrap_or_else(|_| format!("{:?}", val))
                };
                Ok(ToolResult::ok(output))
            }
            Err(e) => Ok(ToolResult::err(format!("MCP error ({}): {}", server, e))),
        }
    }
}

pub fn register_mcp_tools(
    registry: &mut super::ToolRegistry,
    client: Arc<Mutex<McpClient>>,
) {
    let rt = tokio::runtime::Handle::current();
    let tools_info: Vec<(String, String, String, Option<Value>)> = rt.block_on(async {
        let c = client.lock().await;
        c.tools()
            .iter()
            .map(|(server, info)| {
                (
                    server.clone(),
                    info.name.clone(),
                    info.description.clone().unwrap_or_default(),
                    info.input_schema.clone(),
                )
            })
            .collect()
    });

    for (server_name, tool_name, description, schema) in tools_info {
        let bridge = McpBridgeTool::new(
            server_name,
            tool_name,
            description,
            schema,
            client.clone(),
        );
        registry.register(Box::new(bridge));
    }
}
