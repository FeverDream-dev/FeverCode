pub fn build_release_notes(version: &str) -> String {
    format!(
        r#"# Fever Code {version}

## Installation

### Linux
```bash
curl -fsSL https://github.com/FeverDream-dev/FeverCode/releases/download/{version}/install.sh | bash
```

### From Source
```bash
git clone https://github.com/FeverDream-dev/FeverCode.git
cd FeverCode
cargo install --path .
```

## Usage

```bash
# Start the TUI
fever

# or
fever code

# List roles
fever roles

# Show version
fever version

# Show configuration
fever config
```

## What's New

This release includes:

- Core orchestration engine with task execution
- Provider abstraction with OpenAI, Anthropic, Gemini, and Ollama adapters
- OpenRouter integration for 348+ models
- No-paid-API search tool (DuckDuckGo)
- Chrome MCP integration framework
- 10 specialist roles for different coding tasks
- Full-featured TUI with chat, plans, tasks, logs, and browser panel
- Modular crate architecture (10 crates)
- Iterative agent loop (plan, execute, verify, iterate)
- Requirements interrogator with confidence scoring
- Prompt improver for structured engineering briefs
- Operational verifier (build, test, lint, format, custom checks)
- Fighting mode with solution arbitration and rule-based scoring
- Security-first permission guard system

## Known Limitations

- Chrome MCP requires manual setup
- Some providers need additional configuration
- Browser panel shows placeholder without Chrome MCP connection

See [ARCHITECTURE.md](ARCHITECTURE.md) for technical details.
"#
    )
}
