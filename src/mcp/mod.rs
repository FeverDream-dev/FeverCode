use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServersConfig {
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
    #[serde(default)]
    pub disabled: bool,
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
        let config: McpServersConfig = serde_json::from_str(&raw)
            .with_context(|| format!("parsing MCP config {}", config_path.display()))?;

        let mut client = Self {
            servers: HashMap::new(),
            tools: Vec::new(),
            next_id: 1,
        };

        for (name, server_config) in &config.mcp_servers {
            if server_config.disabled {
                eprintln!("MCP: skipping disabled server '{}'", name);
                continue;
            }
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

    pub fn empty() -> Self {
        Self {
            servers: HashMap::new(),
            tools: Vec::new(),
            next_id: 1,
        }
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
            "clientInfo": { "name": "fevercode", "version": "1.0.0" }
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

    pub fn server_names(&self) -> Vec<&str> {
        self.servers.keys().map(|s| s.as_str()).collect()
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

pub fn default_mcp_config() -> McpServersConfig {
    let mut servers = HashMap::new();

    servers.insert("filesystem".to_string(), McpServerConfig {
        command: "npx".to_string(),
        args: vec!["-y".to_string(), "@modelcontextprotocol/server-filesystem".to_string(), ".".to_string()],
        env: HashMap::new(),
        disabled: false,
    });

    servers.insert("github".to_string(), McpServerConfig {
        command: "npx".to_string(),
        args: vec!["-y".to_string(), "@modelcontextprotocol/server-github".to_string()],
        env: {
            let mut e = HashMap::new();
            e.insert("GITHUB_PERSONAL_ACCESS_TOKEN".to_string(), "${GITHUB_TOKEN}".to_string());
            e
        },
        disabled: false,
    });

    servers.insert("fetch".to_string(), McpServerConfig {
        command: "npx".to_string(),
        args: vec!["-y".to_string(), "@modelcontextprotocol/server-fetch".to_string()],
        env: HashMap::new(),
        disabled: false,
    });

    servers.insert("memory".to_string(), McpServerConfig {
        command: "npx".to_string(),
        args: vec!["-y".to_string(), "@modelcontextprotocol/server-memory".to_string()],
        env: HashMap::new(),
        disabled: false,
    });

    servers.insert("sqlite".to_string(), McpServerConfig {
        command: "npx".to_string(),
        args: vec!["-y".to_string(), "@modelcontextprotocol/server-sqlite".to_string(), "--db-path".to_string(), ".fevercode/data.db".to_string()],
        env: HashMap::new(),
        disabled: false,
    });

    servers.insert("brave-search".to_string(), McpServerConfig {
        command: "npx".to_string(),
        args: vec!["-y".to_string(), "@modelcontextprotocol/server-brave-search".to_string()],
        env: {
            let mut e = HashMap::new();
            e.insert("BRAVE_API_KEY".to_string(), "${BRAVE_API_KEY}".to_string());
            e
        },
        disabled: true,
    });

    servers.insert("puppeteer".to_string(), McpServerConfig {
        command: "npx".to_string(),
        args: vec!["-y".to_string(), "@modelcontextprotocol/server-puppeteer".to_string()],
        env: HashMap::new(),
        disabled: true,
    });

    servers.insert("sequential-thinking".to_string(), McpServerConfig {
        command: "npx".to_string(),
        args: vec!["-y".to_string(), "@modelcontextprotocol/server-sequential-thinking".to_string()],
        env: HashMap::new(),
        disabled: false,
    });

    servers.insert("context7".to_string(), McpServerConfig {
        command: "npx".to_string(),
        args: vec!["-y".to_string(), "@upstash/context7-mcp@latest".to_string()],
        env: HashMap::new(),
        disabled: false,
    });

    servers.insert("playwright".to_string(), McpServerConfig {
        command: "npx".to_string(),
        args: vec!["-y".to_string(), "@anthropic-ai/mcp-playwright@latest".to_string()],
        env: HashMap::new(),
        disabled: false,
    });

    servers.insert("mempalace".to_string(), McpServerConfig {
        command: "npx".to_string(),
        args: vec!["-y".to_string(), "mempalace-mcp@latest".to_string()],
        env: HashMap::new(),
        disabled: false,
    });

    servers.insert("chrome-devtools".to_string(), McpServerConfig {
        command: "npx".to_string(),
        args: vec!["-y".to_string(), "@anthropic-ai/mcp-chrome-devtools@latest".to_string()],
        env: HashMap::new(),
        disabled: true,
    });

    servers.insert("postman".to_string(), McpServerConfig {
        command: "npx".to_string(),
        args: vec!["-y".to_string(), "@anthropic-ai/mcp-postman@latest".to_string()],
        env: HashMap::new(),
        disabled: true,
    });

    servers.insert("figma".to_string(), McpServerConfig {
        command: "npx".to_string(),
        args: vec!["-y".to_string(), "@anthropic-ai/mcp-figma@latest".to_string()],
        env: {
            let mut e = HashMap::new();
            e.insert("FIGMA_ACCESS_TOKEN".to_string(), "${FIGMA_ACCESS_TOKEN}".to_string());
            e
        },
        disabled: true,
    });

    servers.insert("google-workspace".to_string(), McpServerConfig {
        command: "npx".to_string(),
        args: vec!["-y".to_string(), "@anthropic-ai/mcp-google-workspace@latest".to_string()],
        env: HashMap::new(),
        disabled: true,
    });

    servers.insert("atlassian".to_string(), McpServerConfig {
        command: "npx".to_string(),
        args: vec!["-y".to_string(), "@anthropic-ai/mcp-atlassian@latest".to_string()],
        env: {
            let mut e = HashMap::new();
            e.insert("JIRA_BASE_URL".to_string(), "${JIRA_BASE_URL}".to_string());
            e.insert("JIRA_API_TOKEN".to_string(), "${JIRA_API_TOKEN}".to_string());
            e.insert("JIRA_EMAIL".to_string(), "${JIRA_EMAIL}".to_string());
            e
        },
        disabled: true,
    });

    servers.insert("linear".to_string(), McpServerConfig {
        command: "npx".to_string(),
        args: vec!["-y".to_string(), "@anthropic-ai/mcp-linear@latest".to_string()],
        env: {
            let mut e = HashMap::new();
            e.insert("LINEAR_API_KEY".to_string(), "${LINEAR_API_KEY}".to_string());
            e
        },
        disabled: true,
    });

    servers.insert("prompts-chat".to_string(), McpServerConfig {
        command: "npx".to_string(),
        args: vec!["-y".to_string(), "prompts-chat-mcp@latest".to_string()],
        env: HashMap::new(),
        disabled: true,
    });

    McpServersConfig { mcp_servers: servers }
}

pub fn generate_default_config(workspace_root: &Path) -> Result<PathBuf> {
    let fever_dir = workspace_root.join(".fevercode");
    std::fs::create_dir_all(&fever_dir)?;
    let config_path = fever_dir.join("mcp.json");
    if config_path.exists() {
        return Ok(config_path);
    }
    let config = default_mcp_config();
    let json = serde_json::to_string_pretty(&config)?;
    std::fs::write(&config_path, json)?;
    Ok(config_path)
}

pub fn add_server(config_path: &PathBuf, name: &str, command: &str, args: Vec<String>, env: HashMap<String, String>) -> Result<()> {
    let mut config = if config_path.exists() {
        let raw = std::fs::read_to_string(config_path)?;
        serde_json::from_str::<McpServersConfig>(&raw)?
    } else {
        McpServersConfig { mcp_servers: HashMap::new() }
    };
    config.mcp_servers.insert(name.to_string(), McpServerConfig {
        command: command.to_string(),
        args,
        env,
        disabled: false,
    });
    let json = serde_json::to_string_pretty(&config)?;
    std::fs::write(config_path, json)?;
    Ok(())
}

pub fn remove_server(config_path: &PathBuf, name: &str) -> Result<()> {
    if !config_path.exists() {
        return Err(anyhow::anyhow!("MCP config not found"));
    }
    let raw = std::fs::read_to_string(config_path)?;
    let mut config: McpServersConfig = serde_json::from_str(&raw)?;
    if config.mcp_servers.remove(name).is_none() {
        return Err(anyhow::anyhow!("server '{}' not found in config", name));
    }
    let json = serde_json::to_string_pretty(&config)?;
    std::fs::write(config_path, json)?;
    Ok(())
}

pub fn list_configured(config_path: &PathBuf) -> Result<Vec<(String, McpServerConfig)>> {
    if !config_path.exists() {
        return Ok(Vec::new());
    }
    let raw = std::fs::read_to_string(config_path)?;
    let config: McpServersConfig = serde_json::from_str(&raw)?;
    Ok(config.mcp_servers.into_iter().collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn default_config_has_expected_servers() {
        let config = default_mcp_config();
        assert!(config.mcp_servers.contains_key("filesystem"));
        assert!(config.mcp_servers.contains_key("github"));
        assert!(config.mcp_servers.contains_key("fetch"));
        assert!(config.mcp_servers.contains_key("memory"));
        assert!(config.mcp_servers.contains_key("sqlite"));
        assert!(config.mcp_servers.contains_key("brave-search"));
        assert!(config.mcp_servers.contains_key("puppeteer"));
        assert!(config.mcp_servers.contains_key("sequential-thinking"));
        assert!(config.mcp_servers.contains_key("context7"));
        assert!(config.mcp_servers.contains_key("playwright"));
        assert!(config.mcp_servers.contains_key("mempalace"));
        assert!(config.mcp_servers.contains_key("chrome-devtools"));
        assert!(config.mcp_servers.contains_key("postman"));
        assert!(config.mcp_servers.contains_key("figma"));
        assert!(config.mcp_servers.contains_key("google-workspace"));
        assert!(config.mcp_servers.contains_key("atlassian"));
        assert!(config.mcp_servers.contains_key("linear"));
        assert!(config.mcp_servers.contains_key("prompts-chat"));
        assert_eq!(config.mcp_servers.len(), 18);
    }

    #[test]
    fn default_config_serializes_valid_json() {
        let config = default_mcp_config();
        let json = serde_json::to_string_pretty(&config).unwrap();
        let parsed: McpServersConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.mcp_servers.len(), config.mcp_servers.len());
    }

    #[test]
    fn disabled_servers_are_flagged() {
        let config = default_mcp_config();
        assert!(!config.mcp_servers["filesystem"].disabled);
        assert!(config.mcp_servers["brave-search"].disabled);
        assert!(config.mcp_servers["puppeteer"].disabled);
    }

    #[test]
    fn add_and_remove_server() {
        let dir = tempfile::tempdir().unwrap();
        let config_path = PathBuf::from(dir.path()).join("mcp.json");
        add_server(&config_path, "test-server", "node", vec!["server.js".to_string()], HashMap::new()).unwrap();
        let servers = list_configured(&config_path).unwrap();
        assert!(servers.iter().any(|(n, _)| n == "test-server"));
        remove_server(&config_path, "test-server").unwrap();
        let servers = list_configured(&config_path).unwrap();
        assert!(!servers.iter().any(|(n, _)| n == "test-server"));
    }

    #[test]
    fn generate_default_config_creates_file() {
        let dir = tempfile::tempdir().unwrap();
        let root = PathBuf::from(dir.path());
        let path = generate_default_config(&root).unwrap();
        assert!(path.exists());
        let raw = fs::read_to_string(&path).unwrap();
        let config: McpServersConfig = serde_json::from_str(&raw).unwrap();
        assert!(!config.mcp_servers.is_empty());
    }

    #[test]
    fn generate_default_config_does_not_overwrite() {
        let dir = tempfile::tempdir().unwrap();
        let root = PathBuf::from(dir.path());
        generate_default_config(&root).unwrap();
        let path = root.join(".fevercode/mcp.json");
        fs::write(&path, r#"{"mcpServers":{}}"#).unwrap();
        generate_default_config(&root).unwrap();
        let raw = fs::read_to_string(&path).unwrap();
        assert!(raw.contains("mcpServers"));
    }
}
