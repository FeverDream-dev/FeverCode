# FeverCode — Terminal AI coding agent

[![CI](https://github.com/FeverDream-dev/FeverCode/actions/workflows/ci.yml/badge.svg)](https://github.com/FeverDream-dev/FeverCode/actions)
[![License MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://github.com/FeverDream-dev/FeverCode/blob/main/LICENSE.md)
[![Rust](https://img.shields.io/badge/Rust-1.70%2B-blue.svg)](https://www.rust-lang.org)
[![MVP](https://img.shields.io/badge/MVP-Premium-green.svg)](https://github.com/FeverDream-dev/FeverCode)

```
______  ______  ______  ______  ______
|  __  ||  __  ||  __  ||  __  ||  __  |
| |  | || |  | || |  | || |  | || |  | |
| |__| || |__| || |__| || |__| || |__| |
|_____|_|_____|__|_____|_____|__|_____| 
```

FeverCode is a Rust-powered terminal assistant that helps you write and manage code with AI-guided planning, drafting, and execution directly in your terminal.

Key goals:
- Fast, deterministic interactions with clear, auditable results
- Local-first safety: workspace-scoped writes, explicit prompts, and guard rails
- A pluggable provider layer with streaming AI backends

## What is FeverCode

FeverCode is a terminal AI coding agent designed to live in your shell. It exposes a tight set of commands that let you:
- plan coding tasks, run builds, and orchestrate a lightweight agent loop
- query AI providers, manage configuration, and inspect results
- operate a full-screen TUI for chat, slash commands, and quick interactions

The project is implemented in Rust and ships a CLI (fever and fevercode) plus a themed, in-terminal UI.

## Current Status

| Item | Status | Notes |
|------|----------|-------|
| Cargo toolchain (fmt/clippy/test) | Working | cargo fmt, cargo clippy, cargo test all pass (43 tests) |
| Binaries | Working | fever and fevercode binary names both work |
| fever init | Working | creates .fevercode/config.toml and .fevercode/mcp.json |
| fever doctor | Working | runs comprehensive health checks |
| fever plan | Working | prints a plan outline (no AI call without provider key) |
| fever run | Working | prints build mode info (no AI call without provider key) |
| fever endless | Working | prints loop outline (experimental, needs provider) |
| fever providers | Working | lists 5 configured providers |
| fever agents | Working | lists 6 agent roles |
| fever version | Working | prints version |
| fever (no args) | Working | opens full-screen Ratatui TUI with chat input and sidebar |
| Safety model | Working | workspace-only writes, risk classification, spray guards |
| TUI slash commands | Working | /help, /plan, /run, /spray, /ask, /auto, /doctor, /diff, /approve, /status, /model, /clear, /exit |
| fever --help (general) | Working | shows usage and commands |
| Tests and lint completeness | Working | 43 tests passing, lint clean |
| fever souls list/show/validate/init | Working | lists built-in souls and config |
| fever context stats | Working | shows session statistics (MVP) |
| fever context compact | Working | generates compact session summary |
| Secret redaction | Working | API keys, tokens, private keys redacted from logs |
| Session events | Working | JSONL event log under .fevercode/session/ |
| Release workflow | Working | cargo-dist configured, tag-triggered (no release published yet) |
| One-line installer | Planned | scripts exist, requires first release tag |
| Fever Souls (new) | Planned | see SOULS.md for agent constitution |
| SOULS.md | Working | agent constitution file |

> Experimental: fever endless "goal" prints loop outline (requires provider)

## Install

Three pathways are provided to install FeverCode. Content below reflects current release status and available artifacts.

### macOS / Linux
Available after first release

```
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/FeverDream-dev/FeverCode/releases/latest/download/fever-installer.sh | sh
```

### Windows
Available after first release

```
powershell -ExecutionPolicy Bypass -c "irm https://github.com/FeverDream-dev/FeverCode/releases/latest/download/fever-installer.ps1 | iex"
```

### From source
This one works NOW

```
cargo install --git https://github.com/FeverDream-dev/FeverCode fever
```

Notes:
- Building from source is the current path for FeverCode; prebuilt releases are not yet published to crates.io.
- All commands assume API keys are configured for the chosen providers.

## Quick Start

- Run the CLI: fever
- Show help: fever --help
- Initialize FeverCode config: fever init
- Health checks: fever doctor
- Manage providers: fever providers
- Print version: fever version

Advanced: interact with the in-terminal UI by running fever with no arguments. This opens a full-screen Ratatui-based UI with chat input, a sidebar, and slash commands.

## Fever Souls

FeverCode uses SOULS.md as its agent constitution. It defines how the coding souls plan, edit, test, compress context, and protect your workspace.
- Ra plans.
- Thoth designs.
- Ptah builds.
- Maat verifies.
- Anubis guards.
- Seshat documents.

CLI: fever souls list, fever souls show ra, fever souls validate, fever souls init

## Context Economy

FeverCode prefers small, targeted context over raw dumps:
- Compact tool outputs (200-line default truncation)
- Session event logging (.fevercode/session/events.jsonl)
- Secret redaction (API keys, tokens, private keys)
- Generated analysis scripts over reading many files
- Session summaries (fever context compact)
- Searchable future memory (planned)

## Commands

- fever: Launch the CLI with the TUI and chat interface
- fever --help: Show help and command list
- fever init: Create .fevercode/config.toml and .fevercode/mcp.json
- fever doctor: Run health checks
- fever providers: List configured providers
- fever agents: List agent roles
- fever plan "task": Print plan outline (no AI call without provider key)
- fever run "task": Print build mode info (no AI call without provider key)
- fever endless "goal": Print loop outline (experimental, needs provider)
- fever version: Print version

TUI slash commands:
- /help /plan /run /spray /ask /auto /doctor /diff /approve /status /model /clear /exit
fever souls list/show/validate/init
fever context stats/compact

## Architecture

- Souls: agent constitution and soul config
- Events: session event logging and compaction
- Context economy: output truncation, secret redaction
- Safety: guard-rails, risk classifications, and spray mode guards
- Config: loads .fevercode/config.toml and mcp.json; provider settings live here
- Workspace: writes are workspace-scoped
- Providers: streaming OpenAI-compatible backends and external bridges
- Tools: internal utilities for planning and execution
- Agents: Ra, Thoth, Ptah, Maat, Anubis, Seshat
- Patch: patch-based changes supported for safe edits
- MCP: client protocol abstraction and wiring plan (not fully wired to UI yet)
- Approval: change reviews and plan approvals
- TUI: Ratatui-based interface with chat, sidebar, slash commands

## Roadmap

- MVP stabilization: confirm core flows, tests pass
- Agent loop: wiring for continuous planning/execution with providers (requires API keys)
- MCP wiring: connect stdio MCP client to the UI flow


## Contributing

See CONTRIBUTING.md for how to contribute:

https://github.com/FeverDream-dev/FeverCode/blob/main/CONTRIBUTING.md

## License

FeverCode is released under the MIT License. See LICENSE.md for details.

https://github.com/FeverDream-dev/FeverCode/blob/main/LICENSE.md

## No warranty

FeverCode is provided as-is. Use it responsibly, review commands before approving them, and keep backups of important work.
