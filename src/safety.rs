use serde::{Deserialize, Serialize};
use std::{
    path::{Path, PathBuf},
    str::FromStr,
};
use thiserror::Error;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum ApprovalMode {
    #[default]
    Ask,
    Auto,
    Spray,
}

impl FromStr for ApprovalMode {
    type Err = SafetyError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "ask" => Ok(Self::Ask),
            "auto" => Ok(Self::Auto),
            "spray" => Ok(Self::Spray),
            other => Err(SafetyError::UnknownMode(other.into())),
        }
    }
}

impl std::fmt::Display for ApprovalMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApprovalMode::Ask => write!(f, "ask"),
            ApprovalMode::Auto => write!(f, "auto"),
            ApprovalMode::Spray => write!(f, "spray"),
        }
    }
}

#[derive(Debug, Error)]
pub enum SafetyError {
    #[error("unknown approval mode: {0}")]
    UnknownMode(String),
    #[error("path escapes workspace: {0}")]
    PathEscapesWorkspace(String),
    #[error("blocked command: {0}")]
    BlockedCommand(String),
    #[error("writes outside workspace are forbidden")]
    WritesOutsideWorkspace,
    #[error("destructive command blocked: {0}")]
    DestructiveCommand(String),
    #[error("credential exfiltration blocked")]
    CredentialExfiltration,
    #[error("privileged operation blocked: {0}")]
    PrivilegedOperation(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandRisk {
    Safe,
    WorkspaceEdit,
    Network,
    ShellRead,
    ShellWrite,
    Destructive,
    Credential,
    Privileged,
}

pub fn classify_command(cmd: &str) -> CommandRisk {
    let lower = cmd.to_ascii_lowercase();
    let trimmed = lower.trim();

    if trimmed.starts_with("sudo ")
        || trimmed.starts_with("doas ")
        || trimmed.starts_with("pkexec ")
    {
        return CommandRisk::Privileged;
    }

    let destructive_prefixes = [
        "rm -rf /",
        "rm -rf /*",
        "mkfs.",
        "dd if=",
        "format ",
        "shred ",
        ":(){:|:&};:",
        "> /dev/sd",
        "chmod -r 777 /",
        "chown ",
    ];
    for prefix in &destructive_prefixes {
        if trimmed.starts_with(prefix) {
            return CommandRisk::Destructive;
        }
    }
    if trimmed.starts_with("rm ") && trimmed.contains("-rf") && !trimmed.contains("--") {
        return CommandRisk::Destructive;
    }

    let cred_patterns = [
        "cat /etc/shadow",
        "cat ~/.ssh/",
        "cat ~/.gnupg/",
        "printenv",
        "env |",
        "export ",
        "aws ",
        "gcloud ",
        "echo $",
        "scp ~/.ssh",
        "rsync ~/.ssh",
    ];
    for pat in &cred_patterns {
        if trimmed.contains(pat) {
            return CommandRisk::Credential;
        }
    }

    let net_prefixes = ["curl ", "wget ", "nc ", "ncat ", "ssh ", "rsync ", "scp "];
    for prefix in &net_prefixes {
        if trimmed.starts_with(prefix) {
            return CommandRisk::Network;
        }
    }

    let write_prefixes = [
        "cargo publish",
        "npm publish",
        "pip upload",
        "git push",
        "docker push",
        "docker rm",
        "docker rmi",
        "git commit",
        "git merge",
        "git rebase",
        "apt ",
        "apt-get ",
        "yum ",
        "brew ",
        "pacman ",
    ];
    for prefix in &write_prefixes {
        if trimmed.starts_with(prefix) {
            return CommandRisk::ShellWrite;
        }
    }

    let read_prefixes = [
        "ls",
        "cat ",
        "head ",
        "tail ",
        "grep ",
        "find ",
        "rg ",
        "fd ",
        "git status",
        "git log",
        "git diff",
        "git branch",
        "git tag",
        "cargo check",
        "cargo test",
        "cargo clippy",
        "cargo fmt",
        "cargo build",
        "cargo doc",
        "npm test",
        "npm run",
        "npm list",
        "python ",
        "node ",
        "rustc ",
        "echo ",
        "which ",
        "file ",
        "wc ",
        "sort ",
        "uniq ",
        "cargo ",
        "rustup ",
    ];
    for prefix in &read_prefixes {
        if trimmed.starts_with(prefix) {
            return CommandRisk::ShellRead;
        }
    }

    CommandRisk::ShellWrite
}

pub fn is_command_blocked(cmd: &str) -> Result<(), SafetyError> {
    match classify_command(cmd) {
        CommandRisk::Privileged => Err(SafetyError::PrivilegedOperation(cmd.to_string())),
        CommandRisk::Destructive => Err(SafetyError::DestructiveCommand(cmd.to_string())),
        CommandRisk::Credential => Err(SafetyError::CredentialExfiltration),
        _ => Ok(()),
    }
}

#[derive(Debug, Clone)]
pub struct SafetyPolicy {
    workspace_root: PathBuf,
    cfg: crate::config::SafetyConfig,
}

impl SafetyPolicy {
    pub fn new(workspace_root: PathBuf, cfg: crate::config::SafetyConfig) -> Self {
        Self {
            workspace_root,
            cfg,
        }
    }

    pub fn mode(&self) -> ApprovalMode {
        self.cfg.mode
    }

    pub fn workspace_root(&self) -> &Path {
        &self.workspace_root
    }

    pub fn ensure_inside_workspace(&self, candidate: &Path) -> Result<(), SafetyError> {
        let absolute = if candidate.is_absolute() {
            candidate.to_path_buf()
        } else {
            self.workspace_root.join(candidate)
        };
        let normalized = normalize_path(&absolute);
        let root = normalize_path(&self.workspace_root);
        if normalized.starts_with(&root) {
            Ok(())
        } else {
            Err(SafetyError::PathEscapesWorkspace(
                candidate.display().to_string(),
            ))
        }
    }

    pub fn can_write(&self, candidate: &Path) -> Result<bool, SafetyError> {
        self.ensure_inside_workspace(candidate)?;
        Ok(self.cfg.allow_writes_inside_workspace)
    }

    pub fn can_run_command(&self, cmd: &str) -> Result<CommandRisk, SafetyError> {
        is_command_blocked(cmd)?;
        let risk = classify_command(cmd);
        match self.cfg.mode {
            ApprovalMode::Ask => Ok(risk),
            ApprovalMode::Auto => match risk {
                CommandRisk::Safe | CommandRisk::ShellRead => Ok(risk),
                CommandRisk::WorkspaceEdit if self.cfg.allow_writes_inside_workspace => Ok(risk),
                _ => Ok(risk),
            },
            ApprovalMode::Spray => match risk {
                CommandRisk::Safe
                | CommandRisk::ShellRead
                | CommandRisk::ShellWrite
                | CommandRisk::WorkspaceEdit => Ok(risk),
                CommandRisk::Network if self.cfg.allow_network => Ok(risk),
                CommandRisk::Network => Err(SafetyError::BlockedCommand(cmd.to_string())),
                _ => Ok(risk),
            },
        }
    }

    pub fn can_git_commit(&self) -> bool {
        self.cfg.allow_git_commit || self.cfg.mode == ApprovalMode::Spray
    }

    pub fn can_install_packages(&self) -> bool {
        self.cfg.allow_package_install || self.cfg.mode == ApprovalMode::Spray
    }

    pub fn max_endless_iterations(&self) -> u32 {
        self.cfg.max_endless_iterations
    }

    pub fn checkpoint_interval(&self) -> u32 {
        self.cfg.checkpoint_every_iterations
    }
}

fn normalize_path(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for component in path.components() {
        match component {
            std::path::Component::ParentDir => {
                out.pop();
            }
            std::path::Component::CurDir => {}
            other => out.push(other.as_os_str()),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_policy() -> SafetyPolicy {
        let cfg = crate::config::FeverConfig::default().safety;
        SafetyPolicy::new(PathBuf::from("/tmp/project"), cfg)
    }

    fn spray_policy() -> SafetyPolicy {
        let mut cfg = crate::config::FeverConfig::default().safety;
        cfg.mode = ApprovalMode::Spray;
        SafetyPolicy::new(PathBuf::from("/tmp/project"), cfg)
    }

    #[test]
    fn rejects_parent_escape() {
        let policy = test_policy();
        assert!(policy
            .ensure_inside_workspace(Path::new("../secrets.txt"))
            .is_err());
    }

    #[test]
    fn rejects_double_parent_escape() {
        let policy = test_policy();
        assert!(policy
            .ensure_inside_workspace(Path::new("../../etc/passwd"))
            .is_err());
    }

    #[test]
    fn rejects_absolute_outside_root() {
        let policy = test_policy();
        assert!(policy
            .ensure_inside_workspace(Path::new("/etc/passwd"))
            .is_err());
    }

    #[test]
    fn rejects_absolute_home_dir() {
        let policy = test_policy();
        assert!(policy
            .ensure_inside_workspace(Path::new("/home/user/.ssh/id_rsa"))
            .is_err());
    }

    #[test]
    fn accepts_workspace_file() {
        let policy = test_policy();
        assert!(policy
            .ensure_inside_workspace(Path::new("src/main.rs"))
            .is_ok());
    }

    #[test]
    fn accepts_nested_workspace_file() {
        let policy = test_policy();
        assert!(policy
            .ensure_inside_workspace(Path::new("src/deep/nested/file.rs"))
            .is_ok());
    }

    #[test]
    fn accepts_dotfile_in_workspace() {
        let policy = test_policy();
        assert!(policy
            .ensure_inside_workspace(Path::new(".fevercode/config.toml"))
            .is_ok());
    }

    #[test]
    fn rejects_symlink_style_escape() {
        let policy = test_policy();
        assert!(policy
            .ensure_inside_workspace(Path::new("src/../../../etc/shadow"))
            .is_err());
    }

    #[test]
    fn can_write_inside_workspace() {
        let policy = test_policy();
        assert!(policy.can_write(Path::new("src/main.rs")).unwrap());
    }

    #[test]
    fn cannot_write_outside_workspace() {
        let policy = test_policy();
        assert!(policy.can_write(Path::new("/tmp/other/file.txt")).is_err());
    }

    #[test]
    fn classifies_sudo_as_privileged() {
        assert_eq!(classify_command("sudo rm -rf /"), CommandRisk::Privileged);
    }

    #[test]
    fn classifies_rm_rf_as_destructive() {
        assert_eq!(classify_command("rm -rf /"), CommandRisk::Destructive);
    }

    #[test]
    fn classifies_cat_shadow_as_credential() {
        assert_eq!(classify_command("cat /etc/shadow"), CommandRisk::Credential);
    }

    #[test]
    fn classifies_curl_as_network() {
        assert_eq!(
            classify_command("curl https://example.com"),
            CommandRisk::Network
        );
    }

    #[test]
    fn classifies_cargo_test_as_shell_read() {
        assert_eq!(classify_command("cargo test"), CommandRisk::ShellRead);
    }

    #[test]
    fn classifies_ls_as_shell_read() {
        assert_eq!(classify_command("ls -la"), CommandRisk::ShellRead);
    }

    #[test]
    fn classifies_git_push_as_shell_write() {
        assert_eq!(
            classify_command("git push origin main"),
            CommandRisk::ShellWrite
        );
    }

    #[test]
    fn blocks_destructive_even_in_spray() {
        let policy = spray_policy();
        assert!(policy.can_run_command("rm -rf /").is_err());
    }

    #[test]
    fn blocks_sudo_even_in_spray() {
        let policy = spray_policy();
        assert!(policy.can_run_command("sudo apt install foo").is_err());
    }

    #[test]
    fn blocks_credential_in_spray() {
        let policy = spray_policy();
        assert!(policy.can_run_command("cat /etc/shadow").is_err());
    }

    #[test]
    fn allows_cargo_test_in_spray() {
        let policy = spray_policy();
        assert!(policy.can_run_command("cargo test").is_ok());
    }

    #[test]
    fn is_command_blocked_rejects_mkfs() {
        assert!(is_command_blocked("mkfs.ext4 /dev/sda1").is_err());
    }

    #[test]
    fn is_command_blocked_rejects_dd() {
        assert!(is_command_blocked("dd if=/dev/zero of=/dev/sda").is_err());
    }

    #[test]
    fn is_command_blocked_allows_ls() {
        assert!(is_command_blocked("ls").is_ok());
    }

    #[test]
    fn parses_ask_mode() {
        assert_eq!("ask".parse::<ApprovalMode>().unwrap(), ApprovalMode::Ask);
    }

    #[test]
    fn parses_auto_mode() {
        assert_eq!("auto".parse::<ApprovalMode>().unwrap(), ApprovalMode::Auto);
    }

    #[test]
    fn parses_spray_mode() {
        assert_eq!(
            "spray".parse::<ApprovalMode>().unwrap(),
            ApprovalMode::Spray
        );
    }

    #[test]
    fn rejects_invalid_mode() {
        assert!("yolo".parse::<ApprovalMode>().is_err());
    }
}
