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

FeverCode is a Rust-powered terminal assistant that helps you write and manage code with AI-guided planning, drafting, and execution directly in your terminal. Built for **vibe coding** — ship fast, iterate loud, assume intent.

Key goals:
- Fast, deterministic interactions with clear, auditable results
- Local-first safety: workspace-scoped writes, explicit prompts, and guard rails
- A pluggable provider layer with streaming AI backends
- **Per-LLM presets** that force obedience, precision, and correct tool-use formatting
- **llama3.2 is HARD-LOCKED to test/research/internet-only tasks**

## What is FeverCode

FeverCode is a terminal AI coding agent designed to live in your shell. It exposes a tight set of commands that let you:
- plan coding tasks, run builds, and orchestrate a lightweight agent loop
- query AI providers, manage configuration, and inspect results
- operate a full-screen TUI for chat, slash commands, and quick interactions
- vibe code with `fever vibe "build me a thing"` — creative mode with spray safety

The project is implemented in Rust and ships a CLI (`fever` and `fevercode`) plus a themed, in-terminal UI.

## Current Status

| Item | Status | Notes |
|------|----------|-------|
| Cargo toolchain (fmt/clippy/test) | Working | cargo fmt, cargo clippy, cargo test all pass (91 tests) |
| Binaries | Working | fever and fevercode binary names both work |
| fever init | Working | creates .fevercode/config.toml and .fevercode/mcp.json |
| fever doctor | Working | runs comprehensive health checks |
| fever plan | Working | prints a plan outline (no AI call without provider key) |
| fever run | Working | prints build mode info (no AI call without provider key) |
| fever vibe | Working | creative one-shot mode with spray safety |
| fever endless | Working | prints loop outline (experimental, needs provider) |
| fever providers | Working | lists 53 configured providers |
| fever agents | Working | lists 7 agent roles |
| fever version | Working | prints version |
| fever (no args) | Working | opens full-screen Ratatui TUI with chat input and sidebar |
| Safety model | Working | workspace-only writes, risk classification, spray guards |
| TUI slash commands | Working | 40+ commands: /theme, /search, /file, /exec, /git, /build, /test, /mastermind, /index, /rag-status, /discover, and more |
| Tests and lint completeness | Working | 91 tests passing, lint clean |
| fever souls list/show/validate/init | Working | lists built-in souls and config |
| fever context stats | Working | shows session statistics (MVP) |
| fever context compact | Working | generates compact session summary |
| Secret redaction | Working | API keys, tokens, private keys redacted from logs |
| Session events | Working | JSONL event log under .fevercode/session/ |
| Release workflow | Working | cargo-dist configured, tag-triggered (no release published yet) |
| Per-LLM presets | Working | CloudStrong, LocalMedium, LocalSmall, Precise, VibeCoder, TestResearch |
| llama3.2 hard lock | Working | HARD-LOCKED to TestResearch. Cannot override. |
| Vibe Coder agent | Working | Single-agent shipping machine for `fever vibe` |
| One-line installer | Working | `curl ... | bash` or `cargo install --git` |
| Fever Souls | Working | see SOULS.md for agent constitution |
| Local Mastermind RAG | Working | `/index` and `/mastermind` for 100% local AI |
| Auto-model discovery | Working | `/discover` queries provider for available models |
| Questionnaire system | Working | Auto-clarifies vague requests before planning |
| 10 TUI themes | Working | DarkAero, Matrix, Ocean, Nord, Dracula, and more |
| SOULS.md | Working | agent constitution file |

> Experimental: fever endless "goal" prints loop outline (requires provider)

## Per-LLM Presets — Obedience Engine

FeverCode does not treat all models the same. Each model class gets a **preset** that injects aggressive obedience rules into the system prompt:

| Preset | Models | Temp | Key Behavior |
|--------|--------|------|--------------|
| `CloudStrong` | Claude, GPT-4/5, Gemini 2 | 0.3 | Minimal constraints. Obedience preamble only. |
| `LocalMedium` | Qwen2.5, Llama3.1/3.3, Mistral, DeepSeek | 0.2 | CoT instructions, few-shot examples, grammar hints. |
| `LocalSmall` | Gemma, Phi3, Qwen2 | 0.1 | Heavy few-shot, 3 retries, strict grammar constraints. |
| `Precise` | Any | 0.0 | Zero-temperature exact mode. Max reliability. |
| `VibeCoder` | Any | 0.85 | Ship fast. Assume intent. Iterate aggressively. |
| `TestResearch` | **llama3.2 ONLY** | 0.15 | **HARD-LOCKED.** Test/internet/research only. Blocks production coding. |

The **obedience preamble** is injected at the very top of every system prompt:

```
## CRITICAL — READ FIRST

You are an AI coding agent running inside FeverCode. Your ONLY job is to produce correct output.

ABSOLUTE RULES — VIOLATING ANY OF THESE IS A FAILURE:
1. When asked to perform a file or shell operation, you MUST emit a tool-call JSON object.
   NO prose, NO explanation, NO markdown fences.
2. The JSON MUST be exactly: {"name": "tool_name", "arguments": {"key": "value"}}
3. NEVER wrap tool calls in ```json blocks...
```

### llama3.2 Hard Lock

`llama3.2` is **forbidden from general production coding**. The `PresetRegistry` refuses any override that would move llama3.2 out of `TestResearch`. The TUI displays a red `[TEST-ONLY]` warning. The `fever vibe` command rejects llama3.2 with an error.

Allowed: unit tests, integration tests, API research, benchmarks, tech exploration, prototyping.
Forbidden: production features, security code, complex refactoring, release-branch work.

## Install

### One-liner (macOS / Linux)

```bash
curl -fsSL https://raw.githubusercontent.com/FeverDream-dev/FeverCode/main/install.sh | bash
```

### From source (all platforms)

```bash
cargo install --git https://github.com/FeverDream-dev/FeverCode fever
```

### Manual build

```bash
git clone https://github.com/FeverDream-dev/FeverCode
cd FeverCode/fevercode_starter
cargo build --release
# Binary at target/release/fever
cp target/release/fever ~/.cargo/bin/
```

Prerequisites: Rust 1.70+, Git. API keys needed for cloud providers.

## Quick Start

```bash
fever init          # create .fevercode/config.toml
fever doctor        # health checks
fever providers     # list 53 configured providers
fever --help        # all commands
fever               # launch full-screen TUI
```

In the TUI:
- Type a message to chat with the AI agent
- `/help` for slash commands
- `/theme darkaero` to switch themes
- `/index` to build Local Mastermind RAG from workspace docs
- `/mastermind "how does auth work?"` to query your docs with a small local model
- `/discover` to auto-list models from your provider

## Fever Souls

FeverCode uses SOULS.md as its agent constitution. It defines how the coding souls plan, edit, test, compress context, and protect your workspace.
- Ra plans.
- Thoth designs.
- Ptah builds.
- Maat verifies.
- Anubis guards.
- Seshat documents.
- **Vibe Coder ships.**

CLI: fever souls list, fever souls show ra, fever souls validate, fever souls init

## Context Economy

FeverCode prefers small, targeted context over raw dumps:
- Compact tool outputs (200-line default truncation)
- Session event logging (.fevercode/session/events.jsonl)
- Secret redaction (API keys, tokens, private keys)
- Generated analysis scripts over reading many files
- Session summaries (fever context compact)
- Searchable future memory (planned)

## Local Mastermind RAG

FeverCode includes **Local Mastermind** — a retrieval-augmented generation loop designed for small local models (Phi4 1.5B, Qwen2.5 3B, etc.). These models lack broad knowledge but reason well over retrieved documents. Through iterative search → read → reason → refine-query cycles, a small local model compensates with many steps over a local document database.

How it works:
1. `/index` — walks your workspace, chunks all docs (code, markdown, PDFs), embeds them with `nomic-embed-text`
2. `/mastermind "your question"` — the small LLM searches the vector store, reads chunks, decides if it needs more info, generates follow-up queries, and iterates up to 6 times
3. When confident, it synthesizes a final answer with source citations

Requirements: an Ollama-compatible embedding endpoint (default: `nomic-embed-text` on localhost:11434).

## Commands

CLI:
- `fever` — Launch full-screen TUI
- `fever init` — Create `.fevercode/config.toml` and `.fevercode/mcp.json`
- `fever doctor` — Health checks, safety boundary test, provider status
- `fever providers` — List 53 configured providers
- `fever agents` — List 7 agent roles
- `fever preset list/show/set` — Manage LLM presets
- `fever plan "task"` — Plan mode
- `fever run "task"` — Build mode
- `fever vibe "task"` — Creative one-shot with spray safety
- `fever endless "goal"` — Bounded autonomous loop
- `fever souls list/show/validate/init` — Manage agent constitution
- `fever context stats/compact` — Session economy
- `fever version` — Print version

TUI slash commands (40+):
- **Mode:** `/ask`, `/auto`, `/spray`, `/vibe`, `/mode`
- **Workflow:** `/plan`, `/run`, `/doctor`, `/diff`, `/approve`
- **Theme:** `/theme`, `/colors`
- **Chat:** `/history`, `/copy`, `/redo`, `/undo`, `/clear`
- **Info:** `/status`, `/version`, `/token`, `/compact`, `/config`
- **Model/Provider:** `/model`, `/provider`, `/providers`, `/models`, `/discover`
- **Presets:** `/preset`
- **Agents:** `/agents`, `/souls`
- **Tools:** `/tools`, `/search`, `/file`, `/exec`, `/explain`, `/refactor`
- **Git:** `/git`, `/branch`, `/commit`
- **Build:** `/build`, `/check`, `/test`, `/fmt`, `/clippy`, `/fix`, `/doc`, `/clean`, `/deps`, `/update`, `/bench`, `/run`
- **RAG:** `/index`, `/mastermind`, `/rag-status`, `/rag-clear`

## Architecture

- **TUI:** Ratatui full-screen terminal UI with 10 themes, chat, sidebar, slash commands
- **Agents:** Ra, Thoth, Ptah, Maat, Anubis, Seshat, Vibe Coder — with vibe variants
- **Presets:** per-LLM obedience rules, temperature, retry logic, few-shot examples
- **Safety:** guard-rails, risk classifications, spray mode guards, workspace boundary
- **Providers:** 53 streaming OpenAI-compatible backends and external bridges
- **Model Discovery:** auto-list models from provider `/models` endpoint
- **Clarification:** auto-detects vague requests, asks focused questions, checks readiness
- **Local Mastermind RAG:** document ingestion, embedding, vector search, iterative reasoning
- **Tools:** file read/write/edit, shell, git, search
- **Patch:** patch-based changes for safe edits
- **MCP:** client protocol abstraction
- **Config:** `.fevercode/config.toml` and `mcp.json`
- **Workspace:** writes are workspace-scoped
- **Events:** session event logging and compaction
- **Context economy:** output truncation, secret redaction

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
