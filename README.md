<div align="center">

<pre>
    ╔══════════════════════════════════════════════════════════════╗
    ║                                                              ║
    ║   ░▒▓█ F E V E R   C O D E █▓▒░                           ║
    ║                                                              ║
    ║         ◢◣  The Eye of Horus watches your commits  ◢◣      ║
    ║                                                              ║
    ╚══════════════════════════════════════════════════════════════╝
</pre>

<i>The terminal coding agent forged in the temples of productivity</i>

[![CI](https://github.com/FeverDream-dev/FeverCode/actions/workflows/ci.yml/badge.svg)](https://github.com/FeverDream-dev/FeverCode/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust 1.85+](https://img.shields.io/badge/Rust-1.85%2B-orange.svg)](https://www.rust-lang.org/)
[![Platform](https://img.shields.io/badge/Platform-Linux%20%7C%20macOS-blue.svg)]()

[Website](https://feverdream-dev.github.io/FeverCode/) &nbsp;·&nbsp; [Install](#quick-start) &nbsp;·&nbsp; [Features](#feature-showcase) &nbsp;·&nbsp; [Config](#configuration) &nbsp;·&nbsp; [Docs](#cli-reference)

</div>

---

> **Fever Code** is a full-stack terminal coding agent that lives in your shell. It reads your repo, edits files, runs tests, and ships features — all while you watch from your phone via Telegram. Built in Rust, zero bloat, maximum speed.

## Why Fever Code?

- **Cursor-based workflows are slow.** Fever Code automates repetitive tasks and ships features at your command — no IDE required.
- **Security is not an afterthought.** 3-tier permission modes, per-tool allow/deny lists, path sandboxing, and secret redaction keep your codebase safe.
- **It works where you work.** Lives in the terminal, monitors from Telegram, persists sessions in JSONL, and discovers project instructions automatically.

## Quick Start

```bash
# Install (one line)
curl -sL https://raw.githubusercontent.com/FeverDream-dev/FeverCode/main/install.sh | bash

# Set your API key
export OPENAI_API_KEY="sk-..."

# Launch
fever
```

That's it. Fever Code boots the TUI, connects to your provider, and starts collaborating on your project.

<details>
<summary>📦 Alternative install methods</summary>

### From Source

Requires **Rust 1.85+**.

```bash
git clone https://github.com/FeverDream-dev/FeverCode.git
cd FeverCode
cargo build --release
cp target/release/fever ~/.local/bin/
```

### Mock Mode (no API key needed)

```bash
fever --mock
```

</details>

---

## Feature Showcase

### 🧠 AI-Powered Coding
50+ LLM provider profiles across 4 protocol adapters (OpenAI, Anthropic, Gemini, Ollama) with streaming responses and a robust tool-execution loop. OpenAI, Anthropic, Gemini, Groq, DeepSeek, Mistral, Z.ai, Ollama, and many more — switch instantly with `/model`.

### 🖥️ Premium TUI
11 hand-crafted themes (including the signature **anubis** Egyptian theme), mouse support, slash commands with fuzzy search, command palette (`Ctrl+K`), tool activity panel (`Ctrl+T`), diff viewer (`Ctrl+D`), and a segmented status bar.

### 🔒 Security-First Permissions
Three-tier permission mode (read → write → full), per-tool allow/deny lists, path sandboxing, command risk classification (low/medium/high/critical), and automatic secret redaction in tool output.

### 📁 Project Intelligence
Config cascade merges project-level `.fevercode/config.toml` over user config. Instruction files discovered by walking upward from your workspace root. Sessions persisted in append-only JSONL format.

### 🧪 Mock Mode
`fever --mock` for zero-config testing and experimentation. Deterministic streaming responses, no API key required. Perfect for CI, development, and demos.

### 📊 Diagnostics
19-check `/doctor` command across 5 categories: Environment, Provider, Workspace, Tools, and System. Includes live `cargo check` integration and git state detection.

### 📱 Telegram Monitor
Watch your agent work from your phone. Receive real-time events (file edits, command output, errors) and send commands back (`/status`, `/pause`, `/resume`, `/stop`).

### 🚀 Project Onboarding
21-question guided setup across 5 blocks (Identity, Tech Stack, Deployment, Quality, Delivery). Auto-generates deployment scaffolds for Railway, Render, Fly.io, Docker, and GitHub Actions.

---

## Slash Commands

| Command | Description | Example |
|---------|-------------|---------|
| `/clear` | Clear chat messages | `/clear` |
| `/diff` | Toggle git diff panel | `/diff` |
| `/doctor` | Run 19-check diagnostics | `/doctor` |
| `/help` | Show help overlay | `/help` |
| `/mock` | Toggle mock mode | `/mock` |
| `/model` | Switch AI model | `/model gpt-4o` |
| `/new` | Start new session | `/new` |
| `/permissions` | Cycle permission mode | `/permissions` |
| `/provider` | Switch provider | `/provider openai` |
| `/quit` | Exit Fever Code | `/quit` |
| `/readonly` | Enter read-only mode | `/readonly` |
| `/role` | Set agent role | `/role architect` |
| `/save` | Save session | `/save` |
| `/session` | Manage sessions | `/session list` |
| `/settings` | Open settings | `/settings` |
| `/status` | Show status | `/status` |
| `/theme` | Change UI theme | `/theme anubis` |
| `/tools` | Show tool panel | `/tools` |
| `/version` | Show version | `/version` |

Navigate commands with `Tab`, `↑`/`↓`, and `Enter`.

---

## Configuration

### User Config

`~/.config/fevercode/config.toml`:

```toml
[defaults]
provider = "openai"
model = "gpt-4o"
temperature = 0.7
max_tokens = 4096

[ui]
theme = "dark"  # dark, light, dracula, nord, gruvbox, solarized, tokyo, catppuccin, rose-pine, monokai, anubis
auto_scroll = true
show_thinking = true

[permissions]
mode = "write"  # read, write, full
allow = ["read_file", "list_dir"]
deny = ["shell"]

[tools]
shell_enabled = true
git_enabled = true
search_enabled = true
```

### Project Config

`.fevercode/config.toml` — merged on top of user config:

```toml
[defaults]
model = "claude-sonnet-4-20250514"  # project-specific model
temperature = 0.3

[permissions]
mode = "read"  # restrict to read-only for this project
```

### Environment Variables

| Variable | Purpose |
|----------|---------|
| `OPENAI_API_KEY` | OpenAI provider |
| `OPENROUTER_API_KEY` | OpenRouter provider |
| `ANTHROPIC_API_KEY` | Anthropic/Claude provider |
| `GEMINI_API_KEY` | Google Gemini provider |
| `GROQ_API_KEY` | Groq provider |
| `DEEPSEEK_API_KEY` | DeepSeek provider |
| `MISTRAL_API_KEY` | Mistral provider |
| `ZAI_API_KEY` | Z.ai (GLM) provider |
| `TOGETHER_API_KEY` | Together AI provider |
| `XAI_API_KEY` | xAI (Grok) provider |
| `TELEGRAM_BOT_TOKEN` | Telegram loop monitor |
| `TELEGRAM_CHAT_ID` | Telegram recipient |
| `TELEGRAM_NOTIFY_INTERVAL` | Min seconds between messages (default: 5) |
| `TELEGRAM_LOOP_MODE` | Step-by-step updates (default: true) |

---

## Architecture

```
┌──────────────────────────────────────────────────────────┐
│                     fever (binary)                        │
├────────────┬────────────┬────────────┬────────────────────┤
│ fever-cli  │ fever-tui  │fever-core  │   fever-agent      │
│ (commands) │   (TUI)    │  (traits)  │  (loop + roles)   │
├────────────┴────────────┴────────────┴────────────────────┤
│ fever-providers    │ fever-tools    │ fever-config       │
│  (4 adapters, 50+  │  (shell/fs)    │  (TOML cascade)    │
│   profiles)        │                │                     │
├────────────────────┴────────────────┴────────────────────┤
│ fever-search  │ fever-telegram  │ fever-onboard          │
│ (DuckDuckGo) │  (bot monitor)  │ (project setup)       │
└──────────────────────────────────────────────────────────┘
```

| Crate | Purpose |
|-------|---------|
| `fever-cli` | CLI interface — 10 subcommands, mock mode flag |
| `fever-tui` | Terminal UI — Elm-style architecture, ratatui, 11 themes |
| `fever-core` | Core traits — Task, Plan, Tool, Permissions; EventBus (stub), MemoryStore (stub), Telemetry (stub) |
| `fever-agent` | Coding agent — role system, tool execution loop, prepare_request |
| `fever-providers` | LLM abstraction — 4 protocol adapters, 50+ provider profiles, MockProvider, streaming |
| `fever-tools` | Local tools — shell, filesystem, git, grep |
| `fever-config` | Configuration — TOML, cascade merge, PermissionsConfig |
| `fever-search` | Web search — DuckDuckGo client (not wired into agent) |
| `fever-telegram` | Telegram bot — loop monitor, rate limiting, remote commands |
| `fever-onboard` | Onboarding — 21-question setup, deployment scaffolds |
| `fever-browser` | Browser — placeholder (requires Chrome MCP) |
| `fever-release` | Release notes — changelog generation |

---

## CLI Reference

```bash
# Launch the TUI
fever

# One-shot chat
fever chat "explain the auth module" --model gpt-4o

# Non-interactive execution (with timing)
fever run "fix the build error in src/main.rs"

# System diagnostics (19 checks)
fever doctor

# Configuration
fever config --show
fever config --validate
fever config --edit

# Provider management
fever providers              # list all
fever providers --test openai  # verify connectivity

# Model listing
fever models
fever models --provider anthropic

# Session management
fever session list
fever session clear

# Project onboarding
fever init
fever --re-onboard

# Mock mode (no API key)
fever --mock

# Verbose logging
fever -v              # info
fever -vv             # debug
fever -vvv            # trace

# Version
fever version
```

---

## Development

```bash
# Build
cargo build --release

# Test (221 tests)
cargo test

# Format
cargo fmt

# Lint (0 warnings)
cargo clippy

# Check all
cargo check --workspace
```

---

## License

Dual licensed under [MIT](LICENSE-MIT) OR [Apache-2.0](LICENSE-APACHE-2.0).

---

<div align="center">

```
══════════════════════════════════════════════════════════
 Built with 🔥 by FeverDream
 Code like fever, ship like dream.
 The Eye of Horus watches your commits.
══════════════════════════════════════════════════════════
```

</div>
