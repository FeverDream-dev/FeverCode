use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpConfig {
    #[serde(rename = "mcpServers")]
    pub mcp_servers: HashMap<String, McpServerConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerConfig {
    pub command: String,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub env: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    id: u64,
    method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    params: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    #[serde(default)]
    id: u64,
    #[serde(default)]
    result: Option<Value>,
    #[serde(default)]
    error: Option<JsonRpcError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct JsonRpcError {
    code: i64,
    message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpToolInfo {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub input_schema: Option<Value>,
}

pub struct McpClient {
    servers: HashMap<String, McpServerConnection>,
    tools: Vec<(String, McpToolInfo)>,
    next_id: u64,
}

struct McpServerConnection {
    child: Child,
    stdin: Mutex<tokio::process::ChildStdin>,
    stdout: Mutex<BufReader<tokio::process::ChildStdout>>,
}

impl McpClient {
    pub async fn from_config(config_path: &PathBuf) -> Result<Self> {
        if !config_path.exists() {
            return Ok(Self {
                servers: HashMap::new(),
                tools: Vec::new(),
                next_id: 1,
            });
        }

        let raw = std::fs::read_to_string(config_path)
            .with_context(|| format!("reading MCP config {}", config_path.display()))?;
        let config: McpConfig = serde_json::from_str(&raw)
            .with_context(|| format!("parsing MCP config {}", config_path.display()))?;

        let mut client = Self {
            servers: HashMap::new(),
            tools: Vec::new(),
            next_id: 1,
        };

        for (name, server_config) in &config.mcp_servers {
            match client.connect(name, server_config).await {
                Ok(()) => {}
                Err(e) => {
                    eprintln!("MCP: failed to connect to '{}': {}", name, e);
                }
            }
        }

        client.discover_tools().await?;
        Ok(client)
    }

    async fn connect(&mut self, name: &str, config: &McpServerConfig) -> Result<()> {
        let mut cmd = Command::new(&config.command);
        cmd.args(&config.args)
            .envs(&config.env)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped());

        let mut child = cmd.spawn().with_context(|| {
            format!(
                "spawning MCP server '{}' with command '{}'",
                name, config.command
            )
        })?;

        let stdin = child.stdin.take().context("taking stdin from MCP server")?;
        let stdout = child
            .stdout
            .take()
            .context("taking stdout from MCP server")?;

        let conn = McpServerConnection {
            child,
            stdin: Mutex::new(stdin),
            stdout: Mutex::new(BufReader::new(stdout)),
        };

        self.servers.insert(name.to_string(), conn);

        self.initialize_server(name).await?;
        Ok(())
    }

    async fn initialize_server(&mut self, name: &str) -> Result<()> {
        let params = serde_json::json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": { "name": "fevercode", "version": "0.1.0" }
        });
        self.send_rpc(name, "initialize", Some(params)).await?;
        self.send_rpc(name, "notifications/initialized", None)
            .await?;
        Ok(())
    }

    async fn discover_tools(&mut self) -> Result<()> {
        let server_names: Vec<String> = self.servers.keys().cloned().collect();
        for name in server_names {
            match self.send_rpc(&name, "tools/list", None).await {
                Ok(response) => {
                    if let Some(tools_val) = response.result {
                        if let Some(tools_arr) = tools_val.get("tools").and_then(|t| t.as_array()) {
                            for tool_val in tools_arr {
                                if let Ok(info) =
                                    serde_json::from_value::<McpToolInfo>(tool_val.clone())
                                {
                                    self.tools.push((name.clone(), info));
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!("MCP: failed to list tools from '{}': {}", name, e);
                }
            }
        }
        Ok(())
    }

    pub async fn call_tool(
        &mut self,
        server: &str,
        tool_name: &str,
        arguments: Value,
    ) -> Result<Value> {
        let params = serde_json::json!({
            "name": tool_name,
            "arguments": arguments
        });
        let response = self.send_rpc(server, "tools/call", Some(params)).await?;
        match response.error {
            Some(err) => Err(anyhow::anyhow!(
                "MCP tool error: {} ({})",
                err.message,
                err.code
            )),
            None => Ok(response.result.unwrap_or(Value::Null)),
        }
    }

    pub fn tools(&self) -> &[(String, McpToolInfo)] {
        &self.tools
    }

    async fn send_rpc(
        &mut self,
        server: &str,
        method: &str,
        params: Option<Value>,
    ) -> Result<JsonRpcResponse> {
        let conn = self
            .servers
            .get_mut(server)
            .ok_or_else(|| anyhow::anyhow!("MCP server '{}' not connected", server))?;

        let id = self.next_id;
        self.next_id += 1;

        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id,
            method: method.to_string(),
            params,
        };

        let mut request_json = serde_json::to_string(&request)?;
        request_json.push('\n');

        {
            let mut stdin = conn.stdin.lock().await;
            stdin.write_all(request_json.as_bytes()).await?;
            stdin.flush().await?;
        }

        {
            let mut stdout = conn.stdout.lock().await;
            let mut line = String::new();
            stdout.read_line(&mut line).await?;
            let response: JsonRpcResponse = serde_json::from_str(line.trim())?;
            Ok(response)
        }
    }
}

impl Drop for McpClient {
    fn drop(&mut self) {
        for (_, conn) in self.servers.iter_mut() {
            let _ = conn.child.start_kill();
        }
    }
}
