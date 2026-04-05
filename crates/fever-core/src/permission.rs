//! Security-first permission model for Fever Code.
//!
//! Design principles:
//! - **Deny by default**: all scopes are off unless explicitly granted
//! - **Explicit scopes**: shell_exec, filesystem_read, filesystem_write,
//!   filesystem_delete, git_operations, network_access, browser_access
//! - **Command risk classification**: Low, Medium, High, Critical
//! - **Secret redaction**: detect and mask secrets in tool output
//! - **Path allowlisting**: constrain filesystem operations to allowed directories

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// Hierarchical permission modes, inspired by claw-code's permission model.
///
/// Modes provide a convenient way to set multiple scopes at once,
/// while still allowing fine-grained scope control via `grant()`/`revoke()`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum PermissionMode {
    /// Read-only: no shell, no filesystem writes, no git modifications.
    /// Only read tools (file read, directory listing, grep, git status/log).
    #[default]
    ReadOnly,
    /// Workspace-write: + file writes within the workspace directory.
    /// Shell execution for read-only commands only.
    WorkspaceWrite,
    /// Full access: + shell execution, + file deletion, + git push/commit.
    /// Use with caution.
    DangerFullAccess,
}

impl PermissionMode {
    /// Display label for the status bar.
    pub fn label(&self) -> &'static str {
        match self {
            Self::ReadOnly => "read",
            Self::WorkspaceWrite => "write",
            Self::DangerFullAccess => "full",
        }
    }

    /// Parse a mode string (case-insensitive, multiple aliases).
    pub fn from_str_fuzzy(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "read" | "read-only" | "ro" | "safe" => Some(Self::ReadOnly),
            "write" | "workspace-write" | "workspace" => Some(Self::WorkspaceWrite),
            "full" | "danger-full-access" | "danger" | "auto" => Some(Self::DangerFullAccess),
            _ => None,
        }
    }
}

/// Apply a permission mode to a guard, granting the appropriate scopes.
///
/// This does NOT revoke existing scopes — it only adds new ones.
/// To switch modes cleanly, create a new PermissionGuard.
pub fn apply_mode(guard: &mut PermissionGuard, mode: PermissionMode) {
    match mode {
        PermissionMode::ReadOnly => {
            guard.grant(PermissionScope::FilesystemRead);
        }
        PermissionMode::WorkspaceWrite => {
            guard.grant(PermissionScope::FilesystemRead);
            guard.grant(PermissionScope::FilesystemWrite);
            guard.grant(PermissionScope::GitOperations);
        }
        PermissionMode::DangerFullAccess => {
            guard.grant(PermissionScope::FilesystemRead);
            guard.grant(PermissionScope::FilesystemWrite);
            guard.grant(PermissionScope::FilesystemDelete);
            guard.grant(PermissionScope::ShellExec);
            guard.grant(PermissionScope::GitOperations);
        }
    }
}

/// Create a fresh PermissionGuard pre-configured for the given mode.
pub fn guard_for_mode(mode: PermissionMode) -> PermissionGuard {
    let mut guard = PermissionGuard::new();
    apply_mode(&mut guard, mode);
    guard
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PermissionScope {
    ShellExec,
    FilesystemRead,
    FilesystemWrite,
    FilesystemDelete,
    GitOperations,
    NetworkAccess,
    BrowserAccess,
}

/// Risk classification for commands and operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CommandRisk {
    Low,
    Medium,
    High,
    Critical,
}

/// Result of a permission check.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PermissionVerdict {
    pub allowed: bool,
    pub scope: PermissionScope,
    pub reason: String,
    pub risk: Option<CommandRisk>,
}

/// A deny-by-default permission guard.
///
/// All scopes start disabled. The caller must explicitly grant scopes
/// and configure allowlists before operations can proceed.
#[derive(Debug, Clone)]
pub struct PermissionGuard {
    granted_scopes: HashSet<PermissionScope>,
    path_allowlist: Vec<PathBuf>,
    require_confirmation_risk: CommandRisk,
    denied_commands: HashSet<String>,
}

impl Default for PermissionGuard {
    fn default() -> Self {
        Self::new()
    }
}

impl PermissionGuard {
    /// Create a new permission guard with all scopes denied.
    pub fn new() -> Self {
        Self {
            granted_scopes: HashSet::new(),
            path_allowlist: Vec::new(),
            require_confirmation_risk: CommandRisk::High,
            denied_commands: HashSet::new(),
        }
    }

    /// Grant a specific permission scope.
    pub fn grant(&mut self, scope: PermissionScope) {
        self.granted_scopes.insert(scope);
    }

    /// Revoke a specific permission scope.
    pub fn revoke(&mut self, scope: PermissionScope) {
        self.granted_scopes.remove(&scope);
    }

    /// Add a directory to the path allowlist (canonicalized).
    pub fn allow_path<P: AsRef<Path>>(&mut self, dir: P) {
        if let Ok(canonical) = dir.as_ref().canonicalize() {
            self.path_allowlist.push(canonical);
        } else {
            // If path doesn't exist yet, store it as-is for future resolution
            self.path_allowlist.push(dir.as_ref().to_path_buf());
        }
    }

    /// Deny a specific command by name (e.g., "rm -rf", "format").
    pub fn deny_command(&mut self, command: &str) {
        self.denied_commands.insert(command.to_lowercase());
    }

    /// Set the risk threshold above which confirmation is required.
    pub fn set_confirmation_threshold(&mut self, risk: CommandRisk) {
        self.require_confirmation_risk = risk;
    }

    /// Check if a given scope is permitted.
    pub fn check_scope(&self, scope: PermissionScope) -> PermissionVerdict {
        let allowed = self.granted_scopes.contains(&scope);
        PermissionVerdict {
            allowed,
            scope,
            reason: if allowed {
                format!("{:?} scope is granted", scope)
            } else {
                format!("{:?} scope is denied (not explicitly granted)", scope)
            },
            risk: None,
        }
    }

    /// Check if a path is within the allowlisted directories.
    pub fn check_path(&self, path: &Path) -> PermissionVerdict {
        if self.path_allowlist.is_empty() {
            return PermissionVerdict {
                allowed: false,
                scope: PermissionScope::FilesystemRead,
                reason: "No paths allowlisted — all filesystem access denied".to_string(),
                risk: None,
            };
        }

        // Try to canonicalize the input path
        let resolved = if path.is_absolute() {
            path.to_path_buf()
        } else {
            std::env::current_dir().unwrap_or_default().join(path)
        };

        for allowed_dir in &self.path_allowlist {
            if resolved.starts_with(allowed_dir) {
                return PermissionVerdict {
                    allowed: true,
                    scope: PermissionScope::FilesystemRead,
                    reason: format!("Path {:?} is within allowed dir {:?}", path, allowed_dir),
                    risk: None,
                };
            }
        }

        PermissionVerdict {
            allowed: false,
            scope: PermissionScope::FilesystemRead,
            reason: format!(
                "Path {:?} is outside all allowed directories ({:?})",
                path, self.path_allowlist
            ),
            risk: None,
        }
    }

    /// Check if a shell command is allowed based on risk and deny list.
    pub fn check_command(&self, command: &str) -> PermissionVerdict {
        let lower = command.to_lowercase();

        // Check deny list
        for denied in &self.denied_commands {
            if lower.contains(denied) {
                return PermissionVerdict {
                    allowed: false,
                    scope: PermissionScope::ShellExec,
                    reason: format!("Command matches denied pattern: '{}'", denied),
                    risk: Some(CommandRisk::Critical),
                };
            }
        }

        // Classify risk
        let risk = classify_command_risk(command);
        let scope_granted = self.granted_scopes.contains(&PermissionScope::ShellExec);

        if !scope_granted {
            return PermissionVerdict {
                allowed: false,
                scope: PermissionScope::ShellExec,
                reason: "Shell execution scope is not granted".to_string(),
                risk: Some(risk),
            };
        }

        PermissionVerdict {
            allowed: true,
            scope: PermissionScope::ShellExec,
            reason: format!("Command allowed with {:?} risk", risk),
            risk: Some(risk),
        }
    }

    /// Quick check: is a scope granted?
    pub fn is_granted(&self, scope: PermissionScope) -> bool {
        self.granted_scopes.contains(&scope)
    }
}

/// Classify the risk level of a shell command.
pub fn classify_command_risk(command: &str) -> CommandRisk {
    let lower = command.to_lowercase();
    let trimmed = lower.trim();

    // Critical: destructive, irreversible
    let critical_patterns = [
        "rm -rf /",
        "mkfs.",
        "dd if=",
        ":(){ :|:& };:",
        "> /dev/sd",
        "chmod -r 777 /",
        "chown -r root /",
    ];
    for pattern in &critical_patterns {
        if trimmed.contains(pattern) {
            return CommandRisk::Critical;
        }
    }

    // High: system-wide modifications, credential exposure
    let high_patterns = [
        "rm -rf",
        "sudo ",
        "curl ",
        "wget ",
        "chmod 777",
        "passwd",
        "shadow",
        "ssh-keygen",
        "apt-get install",
        "yum install",
        "pip install",
        "npm install -g",
    ];
    for pattern in &high_patterns {
        if trimmed.contains(pattern) {
            return CommandRisk::High;
        }
    }

    // Medium: file modifications, git operations with side effects
    let medium_patterns = [
        "git push",
        "git commit",
        "git merge",
        "git rebase",
        "git reset",
        "docker ",
        "make install",
        "cargo publish",
    ];
    for pattern in &medium_patterns {
        if trimmed.contains(pattern) {
            return CommandRisk::Medium;
        }
    }

    // Everything else is low risk (read operations, builds, tests)
    CommandRisk::Low
}

/// Patterns for detecting secrets in strings.
const SECRET_PATTERNS: &[&str] = &[
    // API key patterns
    "sk-",      // OpenAI
    "sk_live_", // Stripe
    "sk_test_", // Stripe test
    "ghp_",     // GitHub PAT
    "gho_",     // GitHub OAuth
    "ghs_",     // GitHub App
    "ghu_",     // GitHub user-to-server
    "xoxb-",    // Slack bot token
    "xoxp-",    // Slack user token
    "AKIA",     // AWS access key ID
    "AIza",     // Google API key
    "eyJ",      // JWT token prefix (base64)
    // Key names
    "api_key",
    "api_key=",
    "apikey",
    "secret_key",
    "secret_key=",
    "access_token",
    "access_token=",
    "private_key",
    "private_key=",
    "password",
    "password=",
    "credentials",
    "authorization: bearer",
];

/// Redact potential secrets from a string, replacing them with `[REDACTED]`.
pub fn redact_secrets(input: &str) -> String {
    let mut result = input.to_string();

    for pattern in SECRET_PATTERNS {
        // For key-value patterns (containing =), redact the value portion
        if pattern.contains('=') {
            if let Some(pos) = result.to_lowercase().find(*pattern) {
                let end_of_key = pos + pattern.len();
                // Redact from after the = to end of token
                let value_start = end_of_key;
                if let Some(value_end) = result[value_start..].find(|c: char| {
                    c.is_whitespace() || c == '"' || c == '\'' || c == ',' || c == '}' || c == ']'
                }) {
                    result.replace_range(value_start..value_start + value_end, "[REDACTED]");
                } else if value_start < result.len() {
                    let remaining = result.len() - value_start;
                    if remaining > 0 && remaining < 200 {
                        result.replace_range(value_start.., "[REDACTED]");
                    }
                }
            }
        } else if let Some(pos) = result.find(*pattern) {
            let prefix_end = pos + pattern.len();
            if let Some(token_end) = result[prefix_end..].find(|c: char| {
                c.is_whitespace()
                    || c == '"'
                    || c == '\''
                    || c == ','
                    || c == '}'
                    || c == ']'
                    || c == ')'
            }) {
                result.replace_range(prefix_end..prefix_end + token_end, "[REDACTED]");
            } else if prefix_end < result.len() {
                let remaining = result.len() - prefix_end;
                if remaining > 0 && remaining < 200 {
                    result.replace_range(prefix_end.., "[REDACTED]");
                }
            }
        }
    }

    result
}

/// Normalize a path and check if it stays within a base directory.
/// Returns the canonical path if safe, or an error message if not.
pub fn normalize_and_validate_path(
    path: &Path,
    base: &Path,
) -> std::result::Result<PathBuf, String> {
    // Resolve relative paths
    let resolved = if path.is_absolute() {
        path.to_path_buf()
    } else {
        base.join(path)
    };

    // Remove .. components manually before canonicalization
    // (canonicalize requires the path to exist)
    let mut normalized = PathBuf::new();
    for component in resolved.components() {
        match component {
            std::path::Component::ParentDir => {
                if !normalized.pop() {
                    return Err(format!(
                        "Path traversal detected: {:?} escapes base {:?}",
                        path, base
                    ));
                }
            }
            std::path::Component::CurDir => { /* skip */ }
            _ => normalized.push(component),
        }
    }

    // Verify the normalized path is within base
    let base_normalized = {
        let mut b = PathBuf::new();
        for component in base.components() {
            match component {
                std::path::Component::ParentDir => {
                    b.pop();
                }
                std::path::Component::CurDir => {}
                _ => b.push(component),
            }
        }
        b
    };

    if !normalized.starts_with(&base_normalized) {
        return Err(format!(
            "Path {:?} is outside allowed base {:?}",
            path, base
        ));
    }

    Ok(normalized)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_deny_by_default() {
        let guard = PermissionGuard::new();
        assert!(!guard.is_granted(PermissionScope::ShellExec));
        assert!(!guard.is_granted(PermissionScope::FilesystemWrite));
        assert!(!guard.is_granted(PermissionScope::NetworkAccess));

        let verdict = guard.check_scope(PermissionScope::ShellExec);
        assert!(!verdict.allowed);
        assert!(verdict.reason.contains("denied"));
    }

    #[test]
    fn test_grant_and_check() {
        let mut guard = PermissionGuard::new();
        guard.grant(PermissionScope::ShellExec);
        assert!(guard.is_granted(PermissionScope::ShellExec));
        assert!(!guard.is_granted(PermissionScope::FilesystemWrite));

        let verdict = guard.check_scope(PermissionScope::ShellExec);
        assert!(verdict.allowed);
    }

    #[test]
    fn test_revoke() {
        let mut guard = PermissionGuard::new();
        guard.grant(PermissionScope::ShellExec);
        assert!(guard.is_granted(PermissionScope::ShellExec));
        guard.revoke(PermissionScope::ShellExec);
        assert!(!guard.is_granted(PermissionScope::ShellExec));
    }

    #[test]
    fn test_path_allowlist_empty() {
        let guard = PermissionGuard::new();
        let verdict = guard.check_path(Path::new("/etc/passwd"));
        assert!(!verdict.allowed);
    }

    #[test]
    fn test_path_allowlist_allowed() {
        let mut guard = PermissionGuard::new();
        let tmp = std::env::temp_dir();
        guard.allow_path(&tmp);
        let test_file = tmp.join("test.rs");
        let verdict = guard.check_path(&test_file);
        assert!(verdict.allowed);
    }

    #[test]
    fn test_command_deny_list() {
        let mut guard = PermissionGuard::new();
        guard.grant(PermissionScope::ShellExec);
        guard.deny_command("rm -rf /");
        let verdict = guard.check_command("rm -rf / --no-preserve-root");
        assert!(!verdict.allowed);
        assert!(verdict.reason.contains("denied pattern"));
    }

    #[test]
    fn test_command_risk_classification() {
        assert_eq!(classify_command_risk("ls -la"), CommandRisk::Low);
        assert_eq!(classify_command_risk("cargo test"), CommandRisk::Low);
        assert_eq!(
            classify_command_risk("git push origin main"),
            CommandRisk::Medium
        );
        assert_eq!(
            classify_command_risk("sudo apt-get install foo"),
            CommandRisk::High
        );
        assert_eq!(
            classify_command_risk("rm -rf / --no-preserve-root"),
            CommandRisk::Critical
        );
        assert_eq!(
            classify_command_risk("curl http://example.com"),
            CommandRisk::High
        );
    }

    #[test]
    fn test_command_requires_scope() {
        let guard = PermissionGuard::new();
        let verdict = guard.check_command("ls");
        assert!(!verdict.allowed);
        assert!(verdict.reason.contains("not granted"));
    }

    #[test]
    fn test_redact_secrets_openai_key() {
        let input = "Using key sk-1234567890abcdef for authentication";
        let redacted = redact_secrets(input);
        assert!(redacted.contains("[REDACTED]"));
        assert!(!redacted.contains("1234567890abcdef"));
        assert!(redacted.contains("sk-"));
    }

    #[test]
    fn test_redact_secrets_github_pat() {
        let input = "Token: ghp_ABCDEFGHIJKLMNOPQRSTUVWXYZ";
        let redacted = redact_secrets(input);
        assert!(redacted.contains("REDACTED"));
        assert!(!redacted.contains("ABCDEFGHIJKLMNOPQRSTUVWXYZ"));
    }

    #[test]
    fn test_redact_secrets_key_value() {
        let input = r#"{"api_key": "sk-secret123", "model": "gpt-4"}"#;
        let redacted = redact_secrets(input);
        assert!(redacted.contains("[REDACTED]"));
    }

    #[test]
    fn test_redact_secrets_no_false_positives() {
        let input = "This is a normal string with no secrets";
        let redacted = redact_secrets(input);
        assert_eq!(input, redacted);
    }

    #[test]
    fn test_normalize_path_safe() {
        let base = PathBuf::from("/home/user/project");
        let result = normalize_and_validate_path(Path::new("src/main.rs"), &base);
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            PathBuf::from("/home/user/project/src/main.rs")
        );
    }

    #[test]
    fn test_normalize_path_traversal_blocked() {
        let base = PathBuf::from("/home/user/project");
        let result = normalize_and_validate_path(Path::new("../../etc/passwd"), &base);
        assert!(result.is_err());
        let err_msg = result.unwrap_err();
        assert!(err_msg.contains("traversal") || err_msg.contains("outside"));
    }

    #[test]
    fn test_normalize_path_within_base() {
        let base = PathBuf::from("/home/user/project");
        let result = normalize_and_validate_path(Path::new("src/../tests/test.rs"), &base);
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            PathBuf::from("/home/user/project/tests/test.rs")
        );
    }

    #[test]
    fn test_full_permission_workflow() {
        let mut guard = PermissionGuard::new();
        let tmp = std::env::temp_dir();

        guard.grant(PermissionScope::FilesystemRead);
        guard.grant(PermissionScope::FilesystemWrite);
        guard.grant(PermissionScope::ShellExec);
        guard.allow_path(&tmp);

        let path_verdict = guard.check_path(&tmp.join("output.txt"));
        assert!(path_verdict.allowed);

        let cmd_verdict = guard.check_command("cargo build");
        assert!(cmd_verdict.allowed);
        assert_eq!(cmd_verdict.risk, Some(CommandRisk::Low));

        assert!(!guard.is_granted(PermissionScope::NetworkAccess));
    }
}
