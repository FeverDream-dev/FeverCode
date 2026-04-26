use regex::Regex;
use std::sync::LazyLock;

static SECRET_PATTERNS: LazyLock<Vec<Regex>> = LazyLock::new(|| {
    vec![
        Regex::new(r#"(?i)(api[_-]?key|apikey|secret|token|password|bearer|auth)\s*[=:]\s*['"]?[A-Za-z0-9+/=_-]{20,}['"]?"#).unwrap(),
        Regex::new(r"(?i)bearer\s+[A-Za-z0-9+/._-]{20,}").unwrap(),
        Regex::new(r"-----BEGIN[\s\w]+PRIVATE KEY-----[\s\S]*?-----END[\s\w]+PRIVATE KEY-----").unwrap(),
        Regex::new(r"(?i)(sk-|sk_live_|pk_live_|ghp_|gho_|github_pat_)[A-Za-z0-9+/=_-]{20,}").unwrap(),
        Regex::new(r"(?i)(aws_access_key_id|aws_secret_access_key)\s*[=:]\s*\S+").unwrap(),
    ]
});

pub fn redact_secrets(input: &str) -> String {
    let mut result = input.to_string();
    for pattern in SECRET_PATTERNS.iter() {
        result = pattern
            .replace_all(&result, |caps: &regex::Captures| {
                let full = caps.get(0).unwrap().as_str();
                if full.starts_with("-----BEGIN") {
                    return "-----REDACTED PRIVATE KEY-----".to_string();
                }
                if full.to_lowercase().starts_with("bearer ") {
                    return "bearer [REDACTED]".to_string();
                }
                if full.contains('=') || full.contains(':') {
                    let sep = if full.contains('=') { '=' } else { ':' };
                    let parts: Vec<&str> = full.splitn(2, sep).collect();
                    if !parts.is_empty() {
                        return format!("{}{}[REDACTED]", parts[0], sep);
                    }
                }
                "[REDACTED]".to_string()
            })
            .to_string();
    }
    result
}

pub fn truncate_output(output: &str, max_lines: usize) -> String {
    let lines: Vec<&str> = output.lines().collect();
    if lines.len() <= max_lines {
        return output.to_string();
    }

    let shown: Vec<&str> = lines.iter().take(max_lines).copied().collect();
    let mut result = shown.join("\n");
    result.push_str(&format!(
        "\n... ({} more lines truncated. Full output stored in session.)",
        lines.len() - max_lines
    ));
    result
}

pub fn format_context_stats(
    events_count: usize,
    summary_exists: bool,
    events_path: &std::path::Path,
    summary_path: &std::path::Path,
) -> String {
    let mut stats = String::new();
    stats.push_str("FeverCode Context Stats\n");
    stats.push_str("======================\n\n");
    stats.push_str(&format!("Session events: {}\n", events_count));
    stats.push_str(&format!(
        "Session summary: {}\n",
        if summary_exists {
            "available"
        } else {
            "not yet generated"
        }
    ));
    stats.push_str(&format!("Events log: {}\n", events_path.display()));
    stats.push_str(&format!("Summary file: {}\n", summary_path.display()));
    stats.push('\n');
    stats.push_str("Context economy features:\n");
    stats.push_str("  - Tool output truncation: active (200 lines default)\n");
    stats.push_str("  - Secret redaction: active\n");
    stats.push_str("  - Session event logging: active\n");
    stats.push_str("  - Session summaries: run 'fever context compact'\n");
    stats
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redacts_api_key() {
        let input = "api_key = sk-abc123def456ghi789jkl012mno345";
        let result = redact_secrets(input);
        assert!(result.contains("[REDACTED]"));
        assert!(!result.contains("sk-abc123"));
    }

    #[test]
    fn redacts_bearer_token() {
        let input = "Authorization: bearer abc123def456ghi789jkl012mno345pqr";
        let result = redact_secrets(input);
        assert!(result.contains("bearer [REDACTED]"));
        assert!(!result.contains("abc123def"));
    }

    #[test]
    fn redacts_private_key() {
        let input =
            "-----BEGIN RSA PRIVATE KEY-----\nMIIEpAIBAAKCAQEA...\n-----END RSA PRIVATE KEY-----";
        let result = redact_secrets(input);
        assert!(result.contains("REDACTED PRIVATE KEY"));
        assert!(!result.contains("MIIEpAIBAAKCAQEA"));
    }

    #[test]
    fn redacts_github_pat() {
        let input = "token = github_pat_abc123def456ghi789jkl";
        let result = redact_secrets(input);
        assert!(result.contains("[REDACTED]"));
        assert!(!result.contains("github_pat_"));
    }

    #[test]
    fn preserves_normal_text() {
        let input = "Hello world, this is a normal log line with no secrets.";
        let result = redact_secrets(input);
        assert_eq!(result, input);
    }

    #[test]
    fn truncates_long_output() {
        let long: String = (0..300).map(|i| format!("line {}\n", i)).collect();
        let result = truncate_output(&long, 10);
        assert!(result.contains("line 0"));
        assert!(result.contains("290 more lines"));
        assert!(!result.contains("line 11"));
    }

    #[test]
    fn no_truncation_when_short() {
        let short = "line 1\nline 2\nline 3";
        let result = truncate_output(short, 10);
        assert_eq!(result, short);
    }

    #[test]
    fn format_stats_basic() {
        let stats = format_context_stats(
            5,
            true,
            std::path::Path::new("/tmp/.fevercode/session/events.jsonl"),
            std::path::Path::new("/tmp/.fevercode/session/latest.md"),
        );
        assert!(stats.contains("Session events: 5"));
        assert!(stats.contains("available"));
    }

    #[test]
    fn redacts_env_style() {
        let input = "AWS_SECRET_ACCESS_KEY=wJalrXUtnFEMI/K7MDENG/bPxRfiCY";
        let result = redact_secrets(input);
        assert!(result.contains("[REDACTED]"));
        assert!(!result.contains("wJalrXUtnFEMI"));
    }

    #[test]
    fn redacts_colon_style() {
        let input = "password: supersecretvalue123456789012";
        let result = redact_secrets(input);
        assert!(result.contains("[REDACTED]"));
        assert!(!result.contains("supersecretvalue"));
    }
}
