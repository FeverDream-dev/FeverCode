# FeverCode — Terminal AI Coding Agent

[![CI](https://github.com/FeverDream-dev/FeverCode/actions/workflows/ci.yml/badge.svg)](https://github.com/FeverDream-dev/FeverCode/actions)
[![License: BSL 1.1](https://img.shields.io/badge/License-BSL%201.1-blue.svg)](https://github.com/FeverDream-dev/FeverCode/blob/main/LICENSE.md)
[![Rust](https://img.shields.io/badge/Rust-1.70%2B-orange.svg)](https://www.rust-lang.org)
[![Tests](https://img.shields.io/badge/tests-157%20passing-brightgreen.svg)]()
[![Tools](https://img.shields.io/badge/tools-87%20built--in-9cf.svg)]()
[![GitHub stars](https://img.shields.io/github/stars/FeverDream-dev/FeverCode?style=social)](https://github.com/FeverDream-dev/FeverCode/stargazers)

```
______  ______  ______  ______  ______
|  __  ||  __  ||  __  ||  __  ||  __  |
| |  | || |  | || |  | || |  | || |  | |
| |__| || |__| || |__| || |__| || |__| |
|_____|_|_____|__|_____|_____|__|_____|
```

**FeverCode** is a Rust terminal AI coding agent with 87 built-in tools, 18 MCP servers, 53 LLM provider configs, workspace safety, per-LLM obedience presets, and an open-core monetization model.

Built for **vibe coding** — ship fast, iterate loud, assume intent — or run in safe mode with full approval queues.

## Table of Contents

- [What is FeverCode](#what-is-fevercode)
- [Why FeverCode](#why-fevercode)
- [87 Built-in Tools](#87-built-in-tools)
- [18 Default MCP Servers](#18-default-mcp-servers)
- [Current Status](#current-status)
- [Per-LLM Presets](#per-llm-presets--obedience-engine)
- [Monetization and Licensing](#monetization-and-licensing)
- [Install](#install)
- [Quick Start](#quick-start)
- [CLI Commands](#cli-commands)
- [Fever Souls](#fever-souls)
- [Context Economy](#context-economy)
- [Local Mastermind RAG](#local-mastermind-rag)
- [Architecture](#architecture)
- [What's Next](#whats-next)
- [Contributing](#contributing)
- [License](#license)

## What is FeverCode

FeverCode is a terminal-first AI coding assistant for developers who live in the shell. It ships as a fast CLI (`fever`) with a full-screen Ratatui TUI:

- **87 built-in tools** across 11 modules — file ops, git power tools, code quality, dev workflow, AI power tools, external integrations
- **18 MCP servers** preconfigured for filesystem, GitHub, databases, browsers, design tools, and more
- **53 LLM providers** — cloud, local, Chinese, aggregators
- **7 agent souls** with defined behavioral contracts
- **Per-LLM obedience presets** — each model class gets tailored instructions
- **Monetization ready** — BSL 1.1 license, Pro/Team/Enterprise tiers, feature gates
- **Workspace safety** — architecture-level boundary enforcement, never writes outside your project

## Why FeverCode

| Feature | FeverCode | Generic AI Chat | Browser IDE AI |
|---------|-----------|---------------|----------------|
| 87 built-in tools | ✅ 11 modules | ❌ Minimal | ⚠️ 5-10 |
| 18 MCP servers | ✅ Preconfigured | ❌ None | ⚠️ Limited |
| Runs in terminal | ✅ Native TUI | ❌ Web only | ❌ Browser required |
| Local LLM support | ✅ 53 providers | ⚠️ Limited | ⚠️ Limited |
| Workspace safety | ✅ Architecture-level | ❌ None | ⚠️ Config-dependent |
| Per-LLM presets | ✅ Obedience engine | ❌ One-size-fits-all | ❌ One-size-fits-all |
| Monetization ready | ✅ BSL 1.1 + feature gates | ❌ N/A | ❌ Proprietary |
| Approval queue | ✅ Patch-based diff review | ❌ None | ⚠️ Varies |
| Token compression | ✅ 3 levels | ❌ None | ❌ None |
| Parallel dispatch | ✅ Dependency graphs | ❌ None | ❌ None |
| Open source | ✅ BSL 1.1 | ❌ Proprietary | ⚠️ Varies |

## 87 Built-in Tools

### Core File Tools (5)
| Tool | Description |
|------|-------------|
| `read_file` | Read file contents with offset/limit |
| `list_files` | Directory listing with depth control |
| `search_text` | Text pattern search with glob filtering |
| `write_file` | Create or overwrite files |
| `edit_file` | Surgical string replacement in files |

### Shell (1)
| Tool | Description |
|------|-------------|
| `run_shell` | Execute shell commands in workspace |

### Core Git (4)
| Tool | Description |
|------|-------------|
| `git_status` | Working tree status |
| `git_diff` | Show unstaged/staged changes |
| `git_checkpoint` | Create safety checkpoint (stash + commit) |
| `git_branch` | Create, switch, list branches |

### Extended Files (17)
`copy_file` `move_file` `delete_file` `mkdir` `file_exists` `directory_tree` `code_stats` `env_var` `find_todos` `find_duplicates` `analyze_imports` `file_stat` `append_file` `head_tail` `regex_search` `replace_in_file` `diff_files`

### Extended Git (13)
`git_log` `git_blame` `git_stash` `git_cherry_pick` `git_merge` `git_remote` `git_tag` `git_rebase` `git_reset` `git_show` `git_add_commit` `git_conflict` `github_cli`

### Code Quality (9)
`run_tests` `coverage_report` `complexity` `security_scan` `find_dead_code` `audit_deps` `scaffold_project` `generate_changelog` `analyze_architecture`

### Integrations (6)
`docker` `web_fetch` `package_json` `ci_status` `snippet_exec` `render_markdown`

### UX / TUI (10)
`session_export` `session_resume` `undo_redo` `theme_palette` `diff_viewer` `syntax_highlight` `progress` `bookmark` `notes` `snapshot`

### External Integrations (7)
`github_issues` `github_pr` `gitlab` `slack_notify` `jira` `database` `k8s`

### Dev Workflow (8)
| Tool | Description |
|------|-------------|
| `tdd_cycle` | Red-green-refactor enforcement with test/build/lint verification |
| `planning` | Persistent markdown plans with tasks and progress tracking |
| `c4_diagram` | C4 architecture diagrams (context, container, component) as Mermaid |
| `code_review` | Automated code review: diff analysis, file review, security scanning |
| `perf_profile` | Performance profiling: file size analysis, bundle analysis |
| `git_flow` | Git flow workflow: feature/release/hotfix branches with merge |
| `n8n` | n8n workflow automation integration |
| `linear` | Linear issue tracker integration |

### AI Power Tools (8)
| Tool | Description |
|------|-------------|
| `token_compress` | 3-level text compression (lite/medium/ultra) with token estimation |
| `prompts` | Reusable prompt template library with variable substitution |
| `parallel_dispatch` | Dependency-graph task planning, batch execution, sequential dispatch |
| `context_manager` | Session event compaction, status, export |
| `smart_context` | Relevance-scored file search and code structure summarization |
| `agent_memory` | Cross-session persistent key-value memory store |
| `llm_router` | Task complexity classifier recommending optimal model tier |
| `workspace_analyzer` | Project overview, dependency analysis, health checks |

### MCP Bridge (dynamic)
Dynamically discovered tools from connected MCP servers are registered at startup via the JSON-RPC 2.0 stdio bridge.

## 18 Default MCP Servers

Preconfigured in `.fevercode/mcp.json` via `fever mcp init`:

| Server | Category | Default |
|--------|----------|---------|
| `filesystem` | File operations | Enabled |
| `github` | GitHub API | Enabled |
| `fetch` | HTTP requests | Enabled |
| `memory` | Key-value memory | Enabled |
| `sqlite` | Database queries | Enabled |
| `brave-search` | Web search | Opt-in |
| `puppeteer` | Browser automation | Opt-in |
| `sequential-thinking` | Chain-of-thought | Enabled |
| `context7` | Documentation lookup | Enabled |
| `playwright` | Browser testing | Opt-in |
| `mempalace` | Memory palace technique | Enabled |
| `chrome-devtools` | Chrome DevTools Protocol | Opt-in |
| `postman` | API testing | Opt-in |
| `figma` | Design tool integration | Opt-in |
| `google-workspace` | Google Docs/Sheets/Drive | Opt-in |
| `atlassian` | Jira/Confluence | Opt-in |
| `linear` | Linear issue tracker | Opt-in |
| `prompts-chat` | Prompt marketplace | Opt-in |

Manage with: `fever mcp init`, `fever mcp list`, `fever mcp add`, `fever mcp remove`

## Current Status

| Item | Status |
|------|--------|
| 87 built-in tools | 11 modules, all compiling, clippy clean |
| 18 MCP servers | Preconfigured, dynamic tool discovery |
| 157 tests passing | 0 failures, 2 ignored (ollama-only) |
| Build | `cargo build`, `cargo clippy -- -D warnings` clean |
| CLI | 20+ subcommands fully wired |
| TUI | Full-screen Ratatui with 10 themes, 40+ slash commands |
| Safety | Workspace boundary, risk classification, spray guards |
| Monetization | License system, feature gates, Pro/Team/Enterprise tiers |
| Version | 1.0.0 |

## Per-LLM Presets — Obedience Engine

Each model class gets a **preset** that injects obedience rules into the system prompt:

| Preset | Models | Temp | Behavior |
|--------|--------|------|----------|
| `CloudStrong` | Claude, GPT-4/5, Gemini 2 | 0.3 | Minimal constraints. Obedience preamble only. |
| `LocalMedium` | Qwen2.5, Llama3.1/3.3, Mistral, DeepSeek | 0.2 | CoT instructions, few-shot examples, grammar hints. |
| `LocalSmall` | Gemma, Phi3, Qwen2 | 0.1 | Heavy few-shot, 3 retries, strict grammar constraints. |
| `Precise` | Any | 0.0 | Zero-temperature exact mode. Max reliability. |
| `VibeCoder` | Any | 0.85 | Ship fast. Assume intent. Iterate aggressively. |
| `TestResearch` | **llama3.2 ONLY** | 0.15 | **HARD-LOCKED.** Test/research only. Blocks production coding. |

## Monetization and Licensing

FeverCode uses a **BSL 1.1 (Business Source License)** with a 3-year change date. After 3 years, code becomes Apache 2.0.

| Tier | Price | Features |
|------|-------|----------|
| Community | Free | 87 tools, 18 MCP servers, 53 providers, TUI, safety, RAG |
| Pro | $19/mo | Analytics, cloud sync, custom souls, priority support |
| Team | $39/seat/mo | Team config, audit log, approved provider policies |
| Enterprise | Custom | SSO/SAML, on-premise, dedicated support, custom SLA |

Feature gates are enforced via license keys. Free for individuals and companies under $1M annual revenue.

CLI: `fever auth activate <key>`, `fever auth status`, `fever auth deactivate`

## Install

### One-liner (macOS / Linux)

```bash
curl -fsSL https://raw.githubusercontent.com/FeverDream-dev/FeverCode/main/install.sh | bash
```

### From source (all platforms)

```bash
cargo install --git https://github.com/FeverDream-dev/FeverCode fevercode
```

### Manual build

```bash
git clone https://github.com/FeverDream-dev/FeverCode
cd FeverCode/fevercode_starter
cargo build --release
cp target/release/fever ~/.cargo/bin/
```

Prerequisites: Rust 1.70+, Git. API keys needed for cloud providers.

## Quick Start

```bash
fever init          # create .fevercode/config.toml + mcp.json
fever doctor        # health checks
fever providers     # list 53 configured providers
fever mcp init      # initialize 18 default MCP servers
fever mcp list      # show configured servers
fever tools         # list 87 built-in tools
fever               # launch full-screen TUI
```

In the TUI:
- Type a message to chat with the AI agent
- `/help` for 40+ slash commands
- `/theme darkaero` to switch themes
- `/index` to build Local Mastermind RAG from workspace docs
- `/mastermind "how does auth work?"` to query your docs
- `/discover` to auto-list models from your provider

## CLI Commands

| Command | Description |
|---------|-------------|
| `fever` | Launch full-screen TUI |
| `fever init` | Create `.fevercode/config.toml` and `mcp.json` |
| `fever doctor` | Health checks, safety, providers, test detection |
| `fever plan "task"` | Plan mode (read-only) |
| `fever run "task"` | Plan, approve, edit, test |
| `fever vibe "task"` | Creative one-shot with spray safety |
| `fever endless "goal"` | Bounded autonomous loop |
| `fever providers` | List 53 configured providers |
| `fever agents` | List 7 agent roles |
| `fever souls list/show/validate/init` | Manage agent constitution |
| `fever preset list/show/set` | Manage LLM presets |
| `fever context stats/compact` | Session economy |
| `fever auth activate/status/deactivate` | License management |
| `fever analytics` | Session analytics and usage stats |
| `fever telemetry enable/disable/status` | Opt-in telemetry |
| `fever sync` | Cloud session sync |
| `fever custom-soul create/list/delete` | Custom souls (Pro+) |
| `fever audit-log export/query` | Audit log (Team+) |
| `fever team init/add-member/remove-member` | Team config |
| `fever memory store/recall/list/forget` | Persistent memory |
| `fever mcp init/list/add/remove` | MCP server management |
| `fever version` | Print version |
| `fever update` | Update to latest release |

## Fever Souls

FeverCode uses SOULS.md as its agent constitution. Each soul has a defined identity and behavioral contract:

- **Ra** — Planner. Clarifies goals, creates plans, defines done criteria.
- **Thoth** — Architect. Designs modules, preserves consistency.
- **Ptah** — Builder. Implements patches, keeps diffs small.
- **Maat** — Checker. Runs fmt/clippy/tests, verifies docs claims.
- **Anubis** — Guardian. Enforces safety, protects workspace boundary.
- **Seshat** — Chronicler. Updates docs, maintains changelog.
- **Vibe Coder** — Ships. Fast. No questions asked.

CLI: `fever souls list`, `fever souls show ra`, `fever souls validate`, `fever souls init`

## Context Economy

FeverCode keeps token usage lean:
- 3-level token compression (lite/medium/ultra)
- Output truncation (200-line default)
- Secret redaction (API keys, tokens, private keys)
- Session event logging (`.fevercode/session/events.jsonl`)
- Smart context selection with relevance scoring
- Session compaction (`fever context compact`)
- Cross-session agent memory (`fever memory`)

## Local Mastermind RAG

Small local models (Phi4 1.5B, Qwen2.5 3B) match top LLMs through iterative document retrieval:

1. `/index` — walks workspace, chunks docs, embeds with `nomic-embed-text`
2. `/mastermind "your question"` — iterative search, read, reason, refine up to 6 cycles
3. Synthesizes final answer with source citations

Requirements: Ollama-compatible embedding endpoint (default: `nomic-embed-text` on localhost:11434).

## Architecture

- **TUI:** Ratatui full-screen terminal UI with 10 themes, chat, sidebar, 40+ slash commands
- **87 Tools:** 11 modules — file ops, git, code quality, dev workflow, AI power tools, external integrations
- **18 MCP Servers:** JSON-RPC 2.0 stdio transport, dynamic tool discovery via `McpBridgeTool`
- **Agents:** Ra, Thoth, Ptah, Maat, Anubis, Seshat, Vibe Coder — with vibe variants
- **Presets:** Per-LLM obedience rules, temperature, retry logic, few-shot examples
- **Safety:** Workspace boundary, risk classification (Safe/WorkspaceEdit/Destructive/Privileged/ShellRead/Network), spray mode guards
- **Providers:** 53 streaming OpenAI-compatible backends
- **Monetization:** License system, feature gates (Community/Pro/Team/Enterprise), HMAC-SHA256 key verification
- **Context:** Token compression, smart file selection, session compaction, agent memory
- **Config:** `.fevercode/config.toml`, `.fevercode/mcp.json`, `.fevercode/souls.toml`

## What's Next

- **Crates.io publishing** — `cargo install fevercode` one command
- **Pre-built binaries** — Linux, macOS, Windows via cargo-dist
- **Browser automation MCP** — Puppeteer/Playwright integration for web testing
- **Plugin marketplace** — Community-contributed tools and MCP servers
- **Fever Cloud** — Cloud sync, team analytics, centralized billing

## Contributing

See [CONTRIBUTING.md](https://github.com/FeverDream-dev/FeverCode/blob/main/CONTRIBUTING.md) for how to contribute.

## License

FeverCode is released under the **Business Source License 1.1 (BSL 1.1)**. After the change date (3 years), code becomes Apache 2.0. Free for individuals and companies under $1M annual revenue. See [LICENSE.md](https://github.com/FeverDream-dev/FeverCode/blob/main/LICENSE.md) for details.

## No warranty

FeverCode is provided as-is. Use it responsibly, review commands before approving them, and keep backups of important work.

---

**Keywords:** AI coding agent, terminal AI, CLI AI tool, Rust AI, vibe coding, local LLM coding, AI code assistant, terminal IDE, open source AI coding, FeverCode, MCP servers, AI tools, developer tools, code review, TDD, git workflow, token compression.
