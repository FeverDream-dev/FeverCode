# FeverCode Reality Audit

**Date**: 2026-03-31 (Updated)
**Auditor**: Automated codebase audit + implementation session
**Commit**: 84bb3fd (Refactor: Align Fever Code with terminal coding agent vision)

---

## Executive Summary

FeverCode is a **Rust workspace with 10 crates (~6,400 LOC)** that has progressed from pure scaffold to **partially operational**. Provider adapters work (4 adapters, 348 models fetchable via OpenRouter), security is production-quality (PermissionGuard with 16 tests), and the CLI has real subcommands. However, **the agent loop is still single-shot** — there's no iterative plan→execute→verify→iterate cycle. The system can call an LLM once and dispatch tools, but cannot observe results and decide next actions autonomously.

**Honesty assessment**: Strong foundation. Provider layer and security are real. Agent loop is the #1 blocker. Everything else builds on that loop.

---

## 1. What Actually Works (Verified in Code + Runtime)

### Provider Layer (VERIFIED WITH REAL API CALLS)
- **4 provider adapters**: OpenAI-compatible (431 lines, covers 10+ providers), Anthropic (294 lines), Gemini (341 lines), Ollama (408 lines)
- **ProviderClient**: HashMap-based dispatch, auto-discovery from 13 env vars
- **`fever providers --fetch`**: Fetches model catalogs — verified 348 models from OpenRouter
- **`fever chat --model <model> <message>`**: Single-shot chat to any discovered model
- **3 integration tests**: OpenRouter fetch_models, model_info, chat (require FEVER_ZAI_KEY env var)
- **RetryPolicy**: exponential/linear/fixed backoff (exists, NOT yet wired to providers)

### Security / Permissions (PRODUCTION QUALITY)
- **PermissionGuard** (542 lines): deny-by-default, grant/revoke scopes
- **Path allowlisting**: restrict filesystem operations to repo root + configured paths
- **Command risk classification**: Low/Medium/High/Critical levels
- **Secret redaction**: OpenAI keys, GitHub PATs, key=value pairs — all scrubbed from output
- **Path normalization**: traversal protection (../, symlinks)
- **16 unit tests**: All passing, comprehensive coverage

### CLI (VERIFIED WORKING)
- `fever` / `fever code` — launches TUI (ratatui-based)
- `fever version` — prints version
- `fever version --local` — reads `.fever/local/version.json`
- `fever version --bump` — increments local version (major/minor/patch)
- `fever roles` — lists 10 builtin roles from RoleRegistry
- `fever config` — reads/prints config
- `fever providers [--fetch]` — lists/fetches provider models
- `fever chat --model <model> <message>` — single-shot LLM chat

### Agent Layer (REAL BUT SINGLE-SHOT)
- **FeverAgent** (180 lines): Implements Agent trait with `chat()` (calls provider) and `call_tools()` (dispatches to ToolRegistry)
- **10 builtin roles** (154 lines): coder, architect, debugger, planner, reviewer, researcher, tester, default, refactorer, doc_writer
- **Role-aware system prompts**: Each role has system_prompt, capabilities, tools list, optional temperature
- **Tool call parsing**: Extracts tool_calls from LLM responses and maps to ToolCall structs
- **NO iterative loop**: chat() is single-shot, no observe→decide→act cycle

### Tool System (4 REAL + 1 PLACEHOLDER)
- **ShellTool**: Executes bash commands (WORKS, no sandboxing beyond PermissionGuard)
- **FilesystemTool**: read/write/list/exists/delete (WORKS)
- **GrepTool**: regex search using `ignore` crate (WORKS)
- **GitTool**: status/log/diff/commit/branch via git CLI (WORKS)
- **BrowserTool**: ALL PLACEHOLDER — returns "requires Chrome MCP" for every action

### Search (SCAFFOLD, NOT INTEGRATED)
- `SearchClient` with DuckDuckGo HTML parsing (functional scaffold)
- `SearchCache` — SQLite-backed cache with TTL
- **NOT integrated into agent loop or TUI** — standalone only

### Configuration (WORKS)
- TOML-based config in `~/.config/fevercode/config.toml`
- `ConfigManager` reads/writes config
- Provider config structure with api_key, base_url, model, extra

### Data Models (COMPREHENSIVE)
- `Task`, `Plan`, `Todo` — UUIDs, timestamps, status tracking, dependency graphs
- `Message`, `AgentResponse`, `AgentContext` — agent message types
- `ChatRequest`, `ChatResponse`, `StreamChunk`, `Usage` — provider types
- `ToolCall`, `ToolResult`, `ToolResultData` — tool execution types
- `Event`, `EventBus` — pub/sub event system

### Memory Store (REAL, NOT INTEGRATED)
- SQLite-backed `MemoryStore` with context key-value and message history
- Session-scoped storage
- **NOT wired into agent loop** — standalone only

### TUI (DISPLAY ONLY)
- Renders 5-panel layout: Chat, Plan, Tasks, Tool Log, Browser
- Basic keyboard navigation (1-5 for focus, arrows, Enter, q/Esc)
- Chat input buffer works
- **No LLM backend connected** — messages go nowhere
- **No agent loop running** — on_tick() is empty

### Local Versioning (WORKS)
- `.fever/local/version.json` — `{"major":1,"minor":1,"patch":0}`
- CLI commands: `--local` to read, `--bump` to increment
- `.fever/` in `.git/info/exclude`

---

## 2. What's Broken or Incomplete

### Critical: No Iterative Agent Loop
- `ExecutionEngine::run()` contains `simulate_task()` which is `tokio::time::sleep(100ms)` — **it does nothing**
- `FeverAgent::chat()` is single-shot: call LLM, return response, done
- No loop that: calls LLM → gets tool_calls → executes tools → feeds results back → calls LLM again → repeats until done
- No termination conditions (max iterations, finish_reason detection)
- No message history accumulation across loop iterations
- The TUI `on_tick()` is empty — no background processing

### Critical: No Requirements Interrogation
- No confidence scoring of user requests
- No clarification question generation
- No structured engineering brief generation from vague requests

### Critical: No Verification Layer
- No build/test/lint verification after code changes
- No "did this work?" step in the agent loop
- No diff review or self-check

### High: No Multi-Agent Orchestration
- No fighting agents (multiple solutions compared)
- No solution arbiter/judge
- No comparative scoring

### High: Misleading Documentation (fever-release)
- `fever-release/src/lib.rs` claims "50+ specialist roles" — actually 10
- Claims "30+ LLM providers" — actually 4 adapters with 13 env var auto-discovery
- Claims capabilities not in code

### Medium: Integration Gaps
- MemoryStore not wired into agent loop
- RetryPolicy not wired to providers
- SearchClient not integrated
- EventBus not used by any component
- Tools lack PermissionGuard integration (tools work, but don't go through permission checks)

### Medium: Security Integration
- PermissionGuard exists and is tested but **tools don't use it yet**
- ShellTool executes commands without going through PermissionGuard
- FilesystemTool doesn't validate paths through PermissionGuard
- Tools trust their callers completely

### Low: Code Quality
- Cargo.toml duplicate targets warning (fever + fever-code both point to main.rs)
- BrowserTool is entirely placeholder
- No cargo-fmt or cargo-clippy on this system

---

## 3. Architecture Inconsistencies

| Doc Claims | Code Reality | Gap |
|---|---|---|
| "50+ specialist roles" | 10 hardcoded roles | fever-release inflates by 5x |
| "30+ LLM providers" | 4 adapters, 13 env vars | Closer to truth but overstated |
| "Core orchestration engine" | `simulate_task()` sleeps 100ms | No real execution loop |
| "Chrome MCP integration" | All browser actions return placeholders | Not functional |
| "Full-featured TUI" | Renders but does nothing functional | Shell only |
| "Search caching with TTL" | Cache code exists, not wired | Isolated module |
| "Provider health monitoring" | No health check code exists | Roadmap item only |

---

## 4. Crate-by-Crate Status

| Crate | Lines of Rust | Status | Functional? |
|---|---|---|---|
| fever-core | ~1,000 | Good abstractions, permission system excellent, execution engine is stub | Partial |
| fever-agent | ~334 | Agent wrapper + roles, NO loop driver | Partial (single-shot only) |
| fever-providers | ~1,760 | 4 real adapters, model fetching, auto-discovery | **Yes** |
| fever-tools | ~550 | 4 working tools, 1 placeholder | Mostly (standalone) |
| fever-config | ~240 | Config read/write works | **Yes** |
| fever-search | ~400 | DuckDuckGo + cache scaffold | Partial (not integrated) |
| fever-browser | ~250 | Data models + placeholder tool | No |
| fever-tui | ~538 | Renders UI, accepts input | Partial (display only) |
| fever-cli | ~392 | 8 subcommands, provider/chat/version | **Yes** |
| fever-release | ~60 | Release note template (inaccurate) | Yes but misleading |
| **Total** | **~6,400** | | |

---

## 5. Test Coverage

| Crate | Unit Tests | Integration Tests | Status |
|---|---|---|---|
| fever-core | 16 (permission module) | 0 | Permission well-tested, rest untested |
| fever-providers | 0 | 3 (OpenRouter, requires env var) | Integration tests exist |
| fever-agent | 0 | 0 | No tests |
| fever-tools | 0 | 0 | No tests |
| fever-config | 0 | 0 | No tests |
| fever-search | 0 | 0 | No tests |
| fever-cli | 1 (local_version) | 0 | Minimal |
| **Total** | **17** | **3** | |

---

## 6. Highest-Leverage Next Steps (Priority Order)

### P0: Agent Loop — The One Blocker
1. **Implement iterative agent loop**: LLM call → parse response → if tool_calls, execute tools → feed results as tool messages → call LLM again → repeat until no tool_calls or finish_reason="stop"
2. **Add termination conditions**: max iterations (default 20), finish_reason detection, token budget
3. **Wire message history accumulation**: Each iteration appends to conversation history
4. **Connect PermissionGuard to tools**: Every tool execution goes through permission check first

### P1: Requirements Intelligence
5. **Requirements interrogator**: Confidence scoring (0-100), clarification questions, structured brief generation
6. **Prompt improver**: Rewrite vague requests into engineering briefs before agent loop starts

### P2: Verification Layer
7. **Operational verifier**: Build/test/lint check after code changes, diff review, self-check
8. **Wire verification into agent loop**: After tool execution, verify results

### P3: Multi-Agent
9. **Fighting agents**: 2-3 independent solution agents
10. **Solution arbiter**: Judge comparing correctness, security, speed, maintainability

### P4: Integration & Transport
11. **Wire MemoryStore** into agent loop for session persistence
12. **Wire RetryPolicy** to provider calls
13. **Wire SearchClient** as a tool
14. **Telegram transport** for remote control
15. **Fix fever-release** claims to match reality

---

## 7. What the Codebase Does Well

- **Crate structure** is clean and modular
- **Type system** is well-used (traits, enums, strong typing)
- **Error types** are thorough with thiserror
- **Provider layer** is genuinely functional (4 adapters, 348 models)
- **Security/PermissionGuard** is production-quality (16 tests, deny-by-default)
- **Role system** is well-designed (10 roles with proper system prompts)
- **Tool trait** is extensible and clean
- **EventBus** provides good decoupling potential
- **Config system** is functional
- **Retry logic** is generic and reusable
- **CLI** has real subcommands that do real work

## 8. What Needs Immediate Attention

- **Agent loop is single-shot** — this is the #1 blocker, everything depends on it
- **Tools lack PermissionGuard integration** — security exists but isn't enforced
- **Memory/Retry/Search/EventBus** all exist but aren't wired to anything
- **Zero tests outside fever-core** — no confidence in changes
- **fever-release** makes wildly inaccurate claims
