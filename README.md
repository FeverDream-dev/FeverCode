# Fever Code

An open-source terminal coding agent for Linux.

**Fever Code** is a CLI/TUI-first coding agent that runs in your terminal, understands your local repository, reads and edits files, runs shell commands and tests, and and monitors progress from Telegram on your phone.

## Features

### Agent Core
- **CLI Commands**: `fever` (TUI), `fever chat`, `fever run`, `fever doctor`, `fever config`, `fever models`, `fever providers`, `fever session`, `fever version`, `fever init`, `fever --re-onboard`
- **Core Tools**: Shell execution, filesystem operations (read/write/list), git operations, code search (grep)
- **TUI**: Terminal UI with chat, settings, command palette (Ctrl+K), help overlay (?), input history, session auto-save
- **Configuration**: TOML-based config in `~/.config/fevercode/`, validated on startup
- **Diagnostics**: `fever doctor` health check, `fever config --validate`
- **Role System**: 10+ specialist roles for different tasks
- **Logging**: Structured tracing with `-v`/`-vv`/`-vvv` verbosity levels

### Project Onboarding (`--init`)
First-time setup via a21 targeted questions across 5 blocks:
- **Block A — Identity**: project name, description, end user, current state
- **Block B — Tech Stack**: language, framework, database, frontend, external APIs
- **Block C — Deployment**: hosting platform (Railway/Render/Fly.io/AWS/etc.), CI/CD, env vars, custom domain
- **Block D — Quality**: quality level, testing, style guide, documentation
- **Block E — Delivery**: definition of done, off-limits, urgency level

Auto-generates deployment scaffolds per platform:
- `railway.toml`, `render.yaml`, `fly.toml`, `Dockerfile`, `.github/workflows/ci.yml`, `.env.example`

Profile stored in `.fevercode/project.json` (git-ignored).

### Telegram Integration (Loop Monitor)
Monitor your agent from Telegram on your phone while it runs:
- **Outbound events**: agent started, thinking, file read/modified, command run, errors, task complete, agent idle
- **Inbound commands**: `/status`, `/pause`, `/resume`, `/stop`, `/summary`, `/files`, `/log`, `/help`
- **Rate limiting**: configurable minimum interval between non-critical messages
- **Auto-activation**: enabled when `TELEGRAM_BOT_TOKEN` is set in `.env`

### LLM Providers
12 provider adapters with OpenAI-compatible API support:
- OpenAI, OpenRouter, Anthropic, Gemini, Groq, Together
- DeepSeek, Mistral, Fireworks, Perplexity, Minimax
- Local Ollama (automatic detection)

## Usage

```bash
# Start the TUI
fever

# One-shot chat message
fever chat "explain the auth module" --model gpt-4o

# Run a prompt non-interactively (with timing)
fever run "fix the build error in src/main.rs"

# List configured providers
fever providers

# List available models
fever models
fever models --provider openai

# Check system health and configuration
fever doctor

# Show or manage configuration
fever config --show
fever config --validate
fever config --path

# Manage chat sessions
fever session list
fever session clear

# Show version
fever version

# Project onboarding (first time)
fever init

# Re-run onboarding with existing profile
fever --re-onboard

# Verbose logging (-v info, -vv debug, -vvv trace)
fever -vv run "debug this"
```

## Configuration

Configuration is stored in `~/.config/fevercode/config.toml`:

```toml
[defaults]
provider = "openai"
model = "gpt-4o"
temperature = 0.7
max_tokens = 4096
```

Environment variables (see `.env.example`):

| Variable | Purpose |
|----------|---------|
| `OPENAI_API_KEY` | OpenAI provider |
| `OPENROUTER_API_KEY` | OpenRouter provider |
| `ANTHROPIC_API_KEY` | Anthropic/Claude provider |
| `GEMINI_API_KEY` | Google Gemini provider |
| `GROQ_API_KEY` | Groq provider |
| `TELEGRAM_BOT_TOKEN` | Telegram bot token (via @BotFather) |
| `TELEGRAM_CHAT_ID` | Your Telegram chat ID (via @userinfobot) |
| `TELEGRAM_NOTIFY_INTERVAL` | Min seconds between messages (default: 5) |
| `TELEGRAM_LOOP_MODE` | Step-by-step updates (default: true) |

## Architecture

Fever Code is built with Rust and organized into focused crates:

| Crate | Purpose |
|-------|---------|
| **fever-cli** | Command-line interface (10 subcommands) |
| **fever-tui** | Terminal user interface (Elm-style) ratatui) |
| **fever-core** | Core abstractions (Task, Plan, Tool, EventBus) |
| **fever-agent** | Coding agent with role system |
| **fever-providers** | LLM provider abstraction (12 adapters) |
| **fever-tools** | Local tools (shell, filesystem, git, grep) |
| **fever-config** | Configuration management |
| **fever-search** | Web search (DuckDuckGo) |
| **fever-telegram** | Telegram bot integration with rate limiting |
| **fever-onboard** | Project onboarding with scaffold generation |
| **fever-browser** | Browser integration (Chrome MCP) |
| **fever-release** | Release notes generation |

## Development

```bash
# Run tests (193 tests, full suite)
cargo test

# Format code
cargo fmt

# Run linter
cargo clippy

# Build release
cargo build --release
```

## Installation

### Quick Install (Linux & macOS)

```bash
curl -sL https://raw.githubusercontent.com/FeverDream-dev/FeverCode/main/install.sh | bash
```

### From Source

Requires **Rust 1.85 or newer**.

```bash
git clone https://github.com/FeverDream-dev/FeverCode.git
cd FeverCode
cargo build --release
cp target/release/fever ~/.local/bin/
```

## License

Dual licensed under MIT OR Apache-2.0.

---

**Fever Code** - Code like fever, ship like dream.
