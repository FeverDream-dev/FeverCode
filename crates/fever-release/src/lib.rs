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
- Provider abstraction supporting 30+ LLM providers
- No-paid-API search tool (DuckDuckGo)
- Chrome MCP integration framework
- Internal specialist role system (50+ roles)
- Full-featured TUI with chat, plans, tasks, logs, and browser panel
- Modular crate architecture

## Known Limitations

- Chrome MCP requires manual setup
- Some providers need additional configuration
- Browser panel shows placeholder without Chrome MCP connection

See [ARCHITECTURE.md](ARCHITECTURE.md) for technical details.
"#
    )
}
