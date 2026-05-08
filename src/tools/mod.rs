pub mod ai_power_tools;
pub mod code_quality;
pub mod dev_workflow;
pub mod extended_files;
pub mod extended_git;
pub mod external_integrations;
pub mod files;
pub mod git_tools;
pub mod integrations;
pub mod mcp_bridge;
pub mod shell;
pub mod ux_tools;

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
        // Core file tools (5)
        registry.register(Box::new(files::ReadFileTool::new(workspace_root.clone())));
        registry.register(Box::new(files::ListFilesTool::new(workspace_root.clone())));
        registry.register(Box::new(files::SearchTextTool::new(workspace_root.clone())));
        registry.register(Box::new(files::WriteFileTool::new(workspace_root.clone())));
        registry.register(Box::new(files::EditFileTool::new(workspace_root.clone())));
        // Shell tool (1)
        registry.register(Box::new(shell::RunShellTool::new(workspace_root.clone())));
        // Core git tools (4)
        registry.register(Box::new(git_tools::GitStatusTool::new(workspace_root.clone())));
        registry.register(Box::new(git_tools::GitDiffTool::new(workspace_root.clone())));
        registry.register(Box::new(git_tools::GitCheckpointTool::new(workspace_root.clone())));
        registry.register(Box::new(git_tools::GitBranchTool::new(workspace_root.clone())));
        // Extended file tools (17)
        registry.register(Box::new(extended_files::CopyFileTool::new(workspace_root.clone())));
        registry.register(Box::new(extended_files::MoveFileTool::new(workspace_root.clone())));
        registry.register(Box::new(extended_files::DeleteFileTool::new(workspace_root.clone())));
        registry.register(Box::new(extended_files::MkDirTool::new(workspace_root.clone())));
        registry.register(Box::new(extended_files::FileExistsTool::new(workspace_root.clone())));
        registry.register(Box::new(extended_files::DirectoryTreeTool::new(workspace_root.clone())));
        registry.register(Box::new(extended_files::CodeStatsTool::new(workspace_root.clone())));
        registry.register(Box::new(extended_files::EnvVarTool));
        registry.register(Box::new(extended_files::TodoFinderTool::new(workspace_root.clone())));
        registry.register(Box::new(extended_files::DuplicateFinderTool::new(workspace_root.clone())));
        registry.register(Box::new(extended_files::ImportAnalyzerTool::new(workspace_root.clone())));
        registry.register(Box::new(extended_files::FileStatTool::new(workspace_root.clone())));
        registry.register(Box::new(extended_files::AppendFileTool::new(workspace_root.clone())));
        registry.register(Box::new(extended_files::HeadTailTool::new(workspace_root.clone())));
        registry.register(Box::new(extended_files::RegexSearchTool::new(workspace_root.clone())));
        registry.register(Box::new(extended_files::ReplaceInFileTool::new(workspace_root.clone())));
        registry.register(Box::new(extended_files::DiffFilesTool::new(workspace_root.clone())));
        // Extended git tools (13)
        registry.register(Box::new(extended_git::GitLogTool::new(workspace_root.clone())));
        registry.register(Box::new(extended_git::GitBlameTool::new(workspace_root.clone())));
        registry.register(Box::new(extended_git::GitStashTool::new(workspace_root.clone())));
        registry.register(Box::new(extended_git::GitCherryPickTool::new(workspace_root.clone())));
        registry.register(Box::new(extended_git::GitMergeTool::new(workspace_root.clone())));
        registry.register(Box::new(extended_git::GitRemoteTool::new(workspace_root.clone())));
        registry.register(Box::new(extended_git::GitTagTool::new(workspace_root.clone())));
        registry.register(Box::new(extended_git::GitRebaseTool::new(workspace_root.clone())));
        registry.register(Box::new(extended_git::GitResetTool::new(workspace_root.clone())));
        registry.register(Box::new(extended_git::GitShowTool::new(workspace_root.clone())));
        registry.register(Box::new(extended_git::GitAddCommitTool::new(workspace_root.clone())));
        registry.register(Box::new(extended_git::GitConflictTool::new(workspace_root.clone())));
        registry.register(Box::new(extended_git::GitHubCliTool::new(workspace_root.clone())));
        // Code quality tools (9)
        registry.register(Box::new(code_quality::TestRunnerTool::new(workspace_root.clone())));
        registry.register(Box::new(code_quality::CoverageReportTool::new(workspace_root.clone())));
        registry.register(Box::new(code_quality::ComplexityAnalyzerTool::new(workspace_root.clone())));
        registry.register(Box::new(code_quality::SecurityScanTool::new(workspace_root.clone())));
        registry.register(Box::new(code_quality::DeadCodeFinderTool::new(workspace_root.clone())));
        registry.register(Box::new(code_quality::DependencyAuditTool::new(workspace_root.clone())));
        registry.register(Box::new(code_quality::ProjectScaffolderTool::new(workspace_root.clone())));
        registry.register(Box::new(code_quality::ChangelogGeneratorTool::new(workspace_root.clone())));
        registry.register(Box::new(code_quality::ArchitectureAnalyzerTool::new(workspace_root.clone())));
        // Integration tools (6)
        registry.register(Box::new(integrations::DockerTool::new(workspace_root.clone())));
        registry.register(Box::new(integrations::WebFetchTool));
        registry.register(Box::new(integrations::PackageJsonTool::new(workspace_root.clone())));
        registry.register(Box::new(integrations::CiStatusTool::new(workspace_root.clone())));
        registry.register(Box::new(integrations::SnippetExecTool::new(workspace_root.clone())));
        registry.register(Box::new(integrations::MarkdownRendererTool::new(workspace_root.clone())));
        // UX/TUI tools (9)
        registry.register(Box::new(ux_tools::SessionExportTool::new(workspace_root.clone())));
        registry.register(Box::new(ux_tools::SessionResumeTool::new(workspace_root.clone())));
        registry.register(Box::new(ux_tools::UndoRedoTool::new(workspace_root.clone())));
        registry.register(Box::new(ux_tools::ThemePaletteTool::new(workspace_root.clone())));
        registry.register(Box::new(ux_tools::DiffViewerTool::new(workspace_root.clone())));
        registry.register(Box::new(ux_tools::SyntaxHighlightTool::new(workspace_root.clone())));
        registry.register(Box::new(ux_tools::ProgressTrackerTool::new(workspace_root.clone())));
        registry.register(Box::new(ux_tools::BookmarkTool::new(workspace_root.clone())));
        registry.register(Box::new(ux_tools::WorkspaceNotesTool::new(workspace_root.clone())));
        registry.register(Box::new(ux_tools::SnapshotTool::new(workspace_root.clone())));
        // External integrations (7)
        registry.register(Box::new(external_integrations::GitHubIssuesTool::new(workspace_root.clone())));
        registry.register(Box::new(external_integrations::GitHubPrTool::new(workspace_root.clone())));
        registry.register(Box::new(external_integrations::GitLabTool::new(workspace_root.clone())));
        registry.register(Box::new(external_integrations::SlackNotifyTool::new(workspace_root.clone())));
        registry.register(Box::new(external_integrations::JiraTool::new(workspace_root.clone())));
        registry.register(Box::new(external_integrations::DatabaseTool::new(workspace_root.clone())));
        registry.register(Box::new(external_integrations::KubernetesTool::new(workspace_root.clone())));
        // Dev workflow tools (8)
        registry.register(Box::new(dev_workflow::TddEnforcementTool::new(workspace_root.clone())));
        registry.register(Box::new(dev_workflow::PlanningTool::new(workspace_root.clone())));
        registry.register(Box::new(dev_workflow::C4ArchitectureTool::new(workspace_root.clone())));
        registry.register(Box::new(dev_workflow::CodeReviewTool::new(workspace_root.clone())));
        registry.register(Box::new(dev_workflow::PerfProfilerTool::new(workspace_root.clone())));
        registry.register(Box::new(dev_workflow::GitFlowTool::new(workspace_root.clone())));
        registry.register(Box::new(dev_workflow::N8nWorkflowTool::new(workspace_root.clone())));
        registry.register(Box::new(dev_workflow::LinearTool::new(workspace_root.clone())));
        // AI power tools (7)
        registry.register(Box::new(ai_power_tools::TokenCompressionTool::new(workspace_root.clone())));
        registry.register(Box::new(ai_power_tools::PromptsLibraryTool::new(workspace_root.clone())));
        registry.register(Box::new(ai_power_tools::ParallelDispatchTool::new(workspace_root.clone())));
        registry.register(Box::new(ai_power_tools::ContextManagerTool::new(workspace_root.clone())));
        registry.register(Box::new(ai_power_tools::SmartContextTool::new(workspace_root.clone())));
        registry.register(Box::new(ai_power_tools::AgentMemoryTool::new(workspace_root.clone())));
        registry.register(Box::new(ai_power_tools::WorkspaceAnalyzerTool::new(workspace_root)));
        registry
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}
