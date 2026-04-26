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

> Experimental: fever endless "goal" prints loop outline (requires provider)

## Install

From source only. Do not install from crates.io or rely on prebuilt releases.

1) Clone the repository

```bash
git clone https://github.com/FeverDream-dev/FeverCode.git
```

2) Build the fever codebase

```bash
cd FeverCode/fevercode_starter
cargo build
```

3) (Optional) Verify with formatting, linting, and tests

```bash
cargo fmt
cargo clippy
cargo test
```

Notes:
- Building from source is required for FeverCode in this early stage; no cargo install or release artifacts are published yet.
- All commands in this README reflect current, verifiable behavior when API keys are configured for providers.

## Quick Start

- Run the CLI: fever
- Show help: fever --help
- Initialize FeverCode config: fever init
- Health checks: fever doctor
- Manage providers: fever providers
- Print version: fever version

Advanced: interact with the in-terminal UI by running fever with no arguments. This opens a full-screen Ratatui-based UI with chat input, a sidebar, and slash commands.

## Safety Model

FeverCode operates in three modes:
- ask: prompt-driven planning where you ask for a specific outcome
- auto: automated planning and execution within guardrails
- spray: bulk command execution with safety checks

Workspace-only rule:
- FeverCode writes are restricted to the workspace it runs in. All changes are scoped to the repository you operate on unless you explicitly opt into cross-workspace actions. This keeps changes auditable and reversible.

## Provider Setup

| Provider | Type | Env Var | Config Key | Status |
|----------|------|---------|------------|--------|
| Z.ai (openai_compatible) | OpenAI-compatible streaming client | ZAI_API_KEY | providers.z_ai.api_key | Implemented |
| OpenAI (openai_compatible) | OpenAI-compatible streaming client | OPENAI_API_KEY | providers.openai.api_key | Implemented |
| Ollama Local (openai_compatible) | Ollama running locally | OLLAMA_HOST | providers.ollama_local.host | Implemented |
| Ollama Cloud (openai_compatible) | Ollama Cloud service | OLLAMA_CLOUD_API_KEY | providers.ollama_cloud.api_key | Implemented |
| Gemini CLI (external_cli) | External CLI bridge | GEMINI_CLI_PATH | providers.gemini_cli.path | Implemented |
| Generic OpenAI-compatible endpoint | OpenAI-compatible streaming client | OPENAI_API_KEY | providers.generic_openai.api_endpoint | Implemented |

Notes:
- The provider abstraction is real and streaming works when API keys are configured.
- MCP stdio client is implemented but not wired to the TUI agent loop yet.

## Agent Design

- Ra Planner: orchestrates task planning and resource allocation.
- Thoth Architect: designs the solution structure and interfaces.
- Ptah Builder: translates plans into executable steps.
- Maat Checker: validates correctness and safety at each stage.
- Anubis Guardian: enforces guardrails and safety policies.
- Seshat Docs: maintains documentation and traceability of decisions.

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

## Architecture

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
- Release artifacts: build artifacts and docs site
- Docs site: publish a centralized docs site

## Contributing

See CONTRIBUTING.md for how to contribute:

https://github.com/FeverDream-dev/FeverCode/blob/main/CONTRIBUTING.md

## License

FeverCode is released under the MIT License. See LICENSE.md for details.

https://github.com/FeverDream-dev/FeverCode/blob/main/LICENSE.md

## No warranty

FeverCode is provided as-is. Use it responsibly, review commands before approving them, and keep backups of important work.
