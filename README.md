# Fever Code

An open-source terminal coding agent for Linux.

**Fever Code** is a CLI/TUI-first coding agent that runs in your terminal, understands your local repository, reads and edits files, runs shell commands and tests, and helps you complete real software work end-to-end.

## What It Does

A terminal coding agent should:
- Open a repo and understand its structure
- Read and search files
- Edit code safely
- Run shell commands
- Run tests and lint
- Show diffs and logs
- Maintain a task list and plan
- Iterate until the work is done

Fever Code is building toward this vision.

## Current Status

**This is early-stage software.** Here's what actually works:

### Working Now
- **CLI Entry Points**: `fever`, `fever code`, `fever version`, `fever roles`, `fever config`
- **Core Tools**: Shell execution, filesystem operations (read/write/list), git operations, code search (grep)
- **TUI**: Terminal UI with chat, plan, tasks, tool log, and browser panels
- **Configuration**: TOML-based config in `~/.config/fevercode/`
- **Role System**: 10+ specialist roles for different tasks

### In Progress
- **LLM Integration**: Provider abstraction exists; OpenAI and Ollama implementations pending
- **Agent Loop**: The core plan→execute→verify→iterate loop is being built
- **Chat Input**: TUI display works; typing messages is being added

### Planned (Not Yet)
- Chrome MCP integration
- Streaming responses
- Session persistence
- Additional LLM providers

## Installation

### From Source

```bash
git clone https://github.com/FeverDream-dev/FeverCode.git
cd FeverCode
cargo build --release
```

### Usage

```bash
# Start the TUI
fever

# or
fever code

# List available roles
fever roles

# Show configuration
fever config

# Show version
fever version
```

## Architecture

Fever Code is built with Rust and organized into focused crates:

- **fever-cli**: Command-line interface
- **fever-tui**: Terminal user interface
- **fever-core**: Core abstractions (Task, Plan, Tool, EventBus)
- **fever-agent**: Coding agent with role system
- **fever-providers**: LLM provider abstraction (OpenAI, Ollama planned)
- **fever-tools**: Local tools (shell, filesystem, git, grep)
- **fever-config**: Configuration management
- **fever-search**: Web search (DuckDuckGo)
- **fever-browser**: Browser integration (Chrome MCP, planned)

See [ARCHITECTURE.md](ARCHITECTURE.md) for technical details.

## Configuration

Configuration is stored in `~/.config/fevercode/config.toml`:

```toml
[defaults]
provider = "openai"
model = "gpt-4o"
temperature = 0.7
max_tokens = 4096

[providers.openai]
enabled = true
api_key = "your-api-key"

[providers.ollama]
enabled = true
base_url = "http://localhost:11434"
```

## Specialist Roles

The agent can operate in different specialist modes:

| Role | Purpose |
|------|---------|
| **coder** | Code implementation and modification |
| **researcher** | Research and information gathering |
| **planner** | Strategic planning and task breakdown |
| **architect** | System architecture and design |
| **debugger** | Debugging and troubleshooting |
| **tester** | Testing and quality assurance |
| **reviewer** | Code review and quality assessment |
| **refactorer** | Code refactoring and improvement |
| **shell_executor** | Shell command execution |
| **git_operator** | Git operations |

## Development

```bash
# Run tests
cargo test

# Format code
cargo fmt

# Run linter
cargo clippy

# Build release
cargo build --release
```

See [CONTRIBUTING.md](CONTRIBUTING.md) for contribution guidelines.

## Philosophy

Fever Code is built on these principles:

1. **Honest**: No fake claims. If something doesn't work, we we say so.
2. **Terminal-first**: Optimized for Linux terminal usage.
3. **Practical**: Real tools that do real work, not abstractions.
4. **Focused**: A tight core loop, not a sprawling platform.
5. **Extensible**: Clean interfaces for adding providers and tools.

## License

Dual licensed under MIT OR Apache-2.0. See [LICENSE](LICENSE) for details.

## Acknowledgments

Inspired by tools like OpenCode, Claude Code, and other terminal-native coding agents.

Built with:
- Rust
- ratatui (TUI)
- crossterm (terminal)
- tokio (async runtime)
- clap (CLI)

---

**Fever Code** - Code like fever, ship like dream.
