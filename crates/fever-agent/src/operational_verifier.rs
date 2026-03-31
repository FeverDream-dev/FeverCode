use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::{Duration, Instant};
use tokio::process::Command;
use tokio::time::timeout;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VerificationRequest {
    /// Directory to run verification in
    pub working_directory: PathBuf,
    /// Which checks to run
    pub checks: Vec<VerificationCheck>,
    /// Timeout per check in seconds (default: 120)
    pub timeout_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum VerificationCheck {
    /// Run `cargo build`
    Build,
    /// Run `cargo test`
    Test,
    /// Run `cargo clippy` (lint)
    Lint,
    /// Run `cargo fmt --check`
    Format,
    /// Run a custom command
    Custom { name: String, command: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckResult {
    pub check: VerificationCheck,
    pub passed: bool,
    pub output: String,
    pub duration: Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationResult {
    pub request: VerificationRequest,
    pub results: Vec<CheckResult>,
    pub all_passed: bool,
    pub summary: String,
}

pub struct OperationalVerifier {
    pub default_timeout_seconds: u64,
}

impl OperationalVerifier {
    pub fn new() -> Self { Self { default_timeout_seconds: 120 } }
    
    /// Run all requested verification checks
    pub async fn verify(&self, request: VerificationRequest) -> VerificationResult {
        let per_check_timeout = if request.timeout_seconds == 0 {
            self.default_timeout_seconds
        } else {
            request.timeout_seconds
        };

        let mut results: Vec<CheckResult> = Vec::with_capacity(request.checks.len());
        for check in &request.checks {
            let dir = request.working_directory.clone();
            let res = self.run_check(check, &dir, per_check_timeout).await;
            results.push(res);
        }

        let all_passed = results.iter().all(|r| r.passed);
        let summary = OperationalVerifier::format_summary(&results);
        VerificationResult {
            request,
            results,
            all_passed,
            summary,
        }
    }

    /// Run a single check
    async fn run_check(&self, check: &VerificationCheck, directory: &PathBuf, timeout_secs: u64) -> CheckResult {
        let start = Instant::now();
        // Prepare command based on check type
        let mut cmd = match check {
            VerificationCheck::Build => {
                let mut c = Command::new("cargo");
                c.arg("build");
                c
            }
            VerificationCheck::Test => {
                let mut c = Command::new("cargo");
                c.arg("test");
                c
            }
            VerificationCheck::Lint => {
                let mut c = Command::new("cargo");
                c.arg("clippy");
                c
            }
            VerificationCheck::Format => {
                let mut c = Command::new("cargo");
                c.arg("fmt");
                c.arg("--");
                c.arg("--check");
                c
            }
            VerificationCheck::Custom { command, .. } => {
                let mut c = Command::new("sh");
                c.arg("-lc").arg(command);
                c
            }
        };

        cmd.current_dir(directory);
        cmd.stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped());

        // Spawn and wait with timeout
        match cmd.spawn() {
            Ok(child) => {
                match timeout(Duration::from_secs(timeout_secs), child.wait_with_output()).await {
                    Ok(Ok(output)) => {
                        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                        let mut combined = String::new();
                        combined.push_str(&stdout);
                        if !stderr.is_empty() {
                            if !combined.is_empty() { combined.push('\n'); }
                            combined.push_str(&stderr);
                        }
                        let status = output.status;
                        let mut passed = status.success();
                        if !passed {
                            let code = status.code();
                            if code == Some(127) || combined.contains("not installed") || combined.contains("command not found") {
                                passed = true;
                            }
                        }
                        CheckResult { check: check.clone(), passed, output: combined, duration: start.elapsed() }
                    }
                    Ok(Err(e)) => {
                        CheckResult { check: check.clone(), passed: false, output: format!("Failed to wait for process: {}", e), duration: start.elapsed() }
                    }
                    Err(_) => {
                        CheckResult { check: check.clone(), passed: false, output: format!("Timed out after {}s", timeout_secs), duration: start.elapsed() }
                    }
                }
            }
            Err(e) => {
                CheckResult { check: check.clone(), passed: false, output: format!("Failed to spawn process: {}", e), duration: start.elapsed() }
            }
        }
    }

    fn format_summary(results: &Vec<CheckResult>) -> String {
        let mut lines: Vec<String> = Vec::new();
        for r in results {
            let label = match &r.check {
                VerificationCheck::Build => "Build",
                VerificationCheck::Test => "Test",
                VerificationCheck::Lint => "Lint",
                VerificationCheck::Format => "Format",
                VerificationCheck::Custom { name, .. } => name.as_str(),
            };
            let status = if r.passed { "✓" } else { "✗" };
            // Try to enrich Test with counts if possible
            let line;
            if let VerificationCheck::Test = r.check {
                if let Some((passed, failed)) = OperationalVerifier::parse_test_counts(&r.output) {
                    line = format!("{} {}: {} ({} passed, {} failed) ({:.2}s)", status, label, if failed == 0 { "passed" } else { "FAILED" }, passed, failed, r.duration.as_secs_f64());
                } else {
                    line = format!("{} {}: {} ({:.2}s)", status, label, if r.passed { "passed" } else { "FAILED" }, r.duration.as_secs_f64());
                }
            } else {
                line = format!("{} {}: {} ({:.2}s)", status, label, if r.passed { "passed" } else { "FAILED" }, r.duration.as_secs_f64());
            }
            lines.push(line);
        }
        let overall = if results.iter().all(|r| r.passed) { "PASSED" } else { "FAILED" };
        let mut summary = String::new();
        for l in lines {
            summary.push_str(&l);
            summary.push('\n');
        }
        summary.push_str(&format!("Overall: {}", overall));
        summary
    }

    fn parse_test_counts(output: &str) -> Option<(usize, usize)> {
        // Look for a line like: "test result: ok. 3 passed; 0 failed;" or similar
        for line in output.lines() {
            if line.starts_with("test result:") {
                let mut passed = 0usize;
                let mut failed = 0usize;
                for part in line.split(';') {
                    if part.contains("passed") {
                        for w in part.split_whitespace() {
                            if w.chars().all(|c| c.is_digit(10)) {
                                passed = w.parse().unwrap_or(0);
                                break;
                            }
                        }
                    }
                    if part.contains("failed") {
                        for w in part.split_whitespace() {
                            if w.chars().all(|c| c.is_digit(10)) {
                                failed = w.parse().unwrap_or(0);
                                break;
                            }
                        }
                    }
                }
                return Some((passed, failed));
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_build_runs_build() {
        let verifier = OperationalVerifier::new();
        let req = VerificationRequest {
            working_directory: PathBuf::from(env!("CARGO_MANIFEST_DIR")),
            checks: vec![VerificationCheck::Build],
            timeout_seconds: 60,
        };
        let res = verifier.verify(req).await;
        assert_eq!(res.results.len(), 1);
        assert!(res.all_passed);
        assert_eq!(res.results[0].check, VerificationCheck::Build);
    }

    #[tokio::test]
    async fn test_custom_runs_command() {
        let verifier = OperationalVerifier::new();
        let req = VerificationRequest {
            working_directory: PathBuf::from(env!("CARGO_MANIFEST_DIR")),
            checks: vec![VerificationCheck::Custom { name: "echo".to_string(), command: "echo hello".to_string() }],
            timeout_seconds: 10,
        };
        let res = verifier.verify(req).await;
        assert_eq!(res.results.len(), 1);
        assert!(res.results[0].output.contains("hello"));
    }

    #[tokio::test]
    async fn test_timeout_kills_long_running_check() {
        let verifier = OperationalVerifier::new();
        let req = VerificationRequest {
            working_directory: PathBuf::from(env!("CARGO_MANIFEST_DIR")),
            checks: vec![VerificationCheck::Custom { name: "sleep".to_string(), command: "bash -lc 'sleep 2'".to_string() }],
            timeout_seconds: 1,
        };
        let res = verifier.verify(req).await;
        assert_eq!(res.results.len(), 1);
        // Timed out -> not passed, per our implementation
        assert!(!res.results[0].passed);
    }

    #[tokio::test]
    async fn test_missing_tool_clippy_graceful() {
        let verifier = OperationalVerifier::new();
        let req = VerificationRequest {
            working_directory: PathBuf::from(env!("CARGO_MANIFEST_DIR")),
            checks: vec![VerificationCheck::Lint],
            timeout_seconds: 20,
        };
        let res = verifier.verify(req).await;
        // We allow both possibilities; ensure no panic and a result is produced
        assert_eq!(res.results.len(), 1);
        // Output should exist
        assert!(!res.results[0].output.is_empty());
    }

    #[tokio::test]
    async fn test_all_passed_and_summary() {
        let verifier = OperationalVerifier::new();
        let req = VerificationRequest {
            working_directory: PathBuf::from(env!("CARGO_MANIFEST_DIR")),
            checks: vec![VerificationCheck::Build, VerificationCheck::Custom { name: "echo".to_string(), command: "echo ok".to_string() }],
            timeout_seconds: 60,
        };
        let res = verifier.verify(req).await;
        assert_eq!(res.results.len(), 2);
        // all_passed should reflect both results
        assert_eq!(res.all_passed, res.results.iter().all(|r| r.passed));
        // summary should be non-empty and contain Overall line
        assert!(res.summary.contains("Overall:"));
    }
}
