pub mod files;
pub mod git_tools;
pub mod shell;

use anyhow::Result;
use serde_json::Value;

#[derive(Debug, Clone)]
pub struct ToolResult {
    pub output: String,
    pub success: bool,
}

impl ToolResult {
    pub fn ok(output: impl Into<String>) -> Self {
        Self {
            output: output.into(),
            success: true,
        }
    }

    pub fn err(output: impl Into<String>) -> Self {
        Self {
            output: output.into(),
            success: false,
        }
    }
}

pub trait Tool: Send + Sync {
    fn name(&self) -> &str;
    fn execute(&self, args: Value) -> Result<ToolResult>;
}

pub struct ToolRegistry {
    tools: Vec<Box<dyn Tool>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self { tools: Vec::new() }
    }

    pub fn register(&mut self, tool: Box<dyn Tool>) {
        self.tools.push(tool);
    }

    pub fn get(&self, name: &str) -> Option<&dyn Tool> {
        self.tools
            .iter()
            .find(|t| t.name() == name)
            .map(|t| t.as_ref())
    }

    pub fn names(&self) -> Vec<&str> {
        self.tools.iter().map(|t| t.name()).collect()
    }

    pub fn build_default(workspace_root: std::path::PathBuf) -> Self {
        let mut registry = Self::new();
        registry.register(Box::new(files::ReadFileTool::new(workspace_root.clone())));
        registry.register(Box::new(files::ListFilesTool::new(workspace_root.clone())));
        registry.register(Box::new(files::SearchTextTool::new(workspace_root.clone())));
        registry.register(Box::new(files::WriteFileTool::new(workspace_root.clone())));
        registry.register(Box::new(files::EditFileTool::new(workspace_root.clone())));
        registry.register(Box::new(shell::RunShellTool::new(workspace_root.clone())));
        registry.register(Box::new(git_tools::GitStatusTool::new(
            workspace_root.clone(),
        )));
        registry.register(Box::new(git_tools::GitDiffTool::new(
            workspace_root.clone(),
        )));
        registry.register(Box::new(git_tools::GitCheckpointTool::new(
            workspace_root.clone(),
        )));
        registry.register(Box::new(git_tools::GitBranchTool::new(workspace_root)));
        registry
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}
