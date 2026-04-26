use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ApprovalDecision {
    Approved,
    Rejected,
    Deferred,
}

#[derive(Debug, Clone)]
pub struct ApprovalRequest {
    pub id: String,
    pub action_type: ActionType,
    pub description: String,
    pub risk_level: crate::safety::CommandRisk,
    pub decided: Option<ApprovalDecision>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionType {
    FileWrite,
    FileDelete,
    ShellCommand,
    GitCommit,
    NetworkCall,
    PackageInstall,
    McpToolCall,
}

impl std::fmt::Display for ActionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ActionType::FileWrite => write!(f, "file-write"),
            ActionType::FileDelete => write!(f, "file-delete"),
            ActionType::ShellCommand => write!(f, "shell-command"),
            ActionType::GitCommit => write!(f, "git-commit"),
            ActionType::NetworkCall => write!(f, "network-call"),
            ActionType::PackageInstall => write!(f, "package-install"),
            ActionType::McpToolCall => write!(f, "mcp-tool-call"),
        }
    }
}

impl ApprovalRequest {
    pub fn new(
        action_type: ActionType,
        description: impl Into<String>,
        risk_level: crate::safety::CommandRisk,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string()[..8].to_string(),
            action_type,
            description: description.into(),
            risk_level,
            decided: None,
        }
    }

    pub fn auto_approvable(&self, mode: crate::safety::ApprovalMode) -> bool {
        match mode {
            crate::safety::ApprovalMode::Ask => false,
            crate::safety::ApprovalMode::Auto => matches!(
                self.risk_level,
                crate::safety::CommandRisk::Safe
                    | crate::safety::CommandRisk::ShellRead
                    | crate::safety::CommandRisk::WorkspaceEdit
            ),
            crate::safety::ApprovalMode::Spray => !matches!(
                self.risk_level,
                crate::safety::CommandRisk::Destructive
                    | crate::safety::CommandRisk::Credential
                    | crate::safety::CommandRisk::Privileged
            ),
        }
    }

    pub fn render(&self) -> String {
        let status = match self.decided {
            Some(ApprovalDecision::Approved) => "[APPROVED]",
            Some(ApprovalDecision::Rejected) => "[REJECTED]",
            Some(ApprovalDecision::Deferred) => "[DEFERRED]",
            None => "[PENDING]",
        };
        format!(
            "{} {} ({}) — {}",
            status,
            self.action_type,
            risk_label(self.risk_level),
            self.description
        )
    }
}

fn risk_label(risk: crate::safety::CommandRisk) -> &'static str {
    match risk {
        crate::safety::CommandRisk::Safe => "safe",
        crate::safety::CommandRisk::WorkspaceEdit => "workspace-edit",
        crate::safety::CommandRisk::Network => "network",
        crate::safety::CommandRisk::ShellRead => "shell-read",
        crate::safety::CommandRisk::ShellWrite => "shell-write",
        crate::safety::CommandRisk::Destructive => "DESTRUCTIVE",
        crate::safety::CommandRisk::Credential => "CREDENTIAL",
        crate::safety::CommandRisk::Privileged => "PRIVILEGED",
    }
}
