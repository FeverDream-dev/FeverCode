# FeverCode Feature Matrix

**Date**: 2026-04-02
**Based on**: Code-grounded audit of working tree

## Legend
- **REAL**: Implemented, tested, and functional
- **PARTIAL**: Implemented but incomplete or not integrated
- **STUB**: Interface exists, no real logic
- **MISSING**: Not implemented at all

---

## Core Agent System

| Feature | Status | Evidence | Tests |
|---------|--------|----------|-------|
| Agent trait (chat, call_tools) | REAL | fever-core/src/agent.rs | 36 core tests |
| FeverAgent implementation | REAL | fever-agent/src/agent.rs (202 LOC) | Via loop tests |
| Iterative loop (LoopDriver) | REAL | fever-agent/src/loop_driver.rs (189 LOC) | 13 tests |
| Loop termination conditions | REAL | max_iterations, finish_reason, empty tool_calls | 3 tests |
| Loop event emission | REAL | LoopEvent enum, 6 event types | 2 tests |
| 10 specialist roles | REAL | fever-agent/src/role.rs (154 LOC) | Via agent tests |
| Role system prompts | REAL | Each role has system_prompt, capabilities, tools | — |
| Role switching at runtime | REAL | set_role(), get_current_role() | — |
| Multi-agent orchestration | MISSING | No inter-agent handoff mechanism | 0 |
| Agent handoff protocol | MISSING | — | 0 |
| Plan-driven execution | STUB | ExecutionEngine.simulate_task() sleeps 100ms | 0 |

## Intelligence & Planning

| Feature | Status | Evidence | Tests |
|---------|--------|----------|-------|
| Requirements interrogator | REAL | fever-agent/src/requirements_interrogator.rs (784 LOC) | 14 tests |
| Prompt improver | REAL | fever-agent/src/prompt_improver.rs (405 LOC) | 6 tests |
| Confidence scoring | REAL | 0-100 scale with clamping | 4 tests |
| Clarification questions | REAL | Generated for low-confidence requests | 2 tests |
| Fighting mode / arbiter | REAL | fever-agent/src/fighting_mode.rs (398 LOC) | 7 tests |
| Rule-based scoring | REAL | Correctness, security, speed, maintainability | 3 tests |
| Task graph (Plan/Tasks) | REAL | fever-core/src/task.rs (133 LOC) | 7 tests |
| Task dependencies | REAL | can_start() checks completed deps | 3 tests |
| Todo tracking | REAL | fever-core/src/task.rs Todo struct | 1 test |

## Provider Layer

| Feature | Status | Evidence | Tests |
|---------|--------|----------|-------|
| Provider trait (ProviderAdapter) | REAL | fever-providers/src/adapter.rs | — |
| ProviderClient dispatch | REAL | fever-providers/src/client.rs (102 LOC) | 41 tests |
| OpenAI adapter | REAL | fever-providers/src/adapters/openai.rs (554 LOC) | Integration |
| OpenRouter support | REAL | Via OpenAI adapter with different base_url | Integration |
| Anthropic adapter | REAL | fever-providers/src/adapters/anthropic.rs (313 LOC) | — |
| Gemini adapter | REAL | fever-providers/src/adapters/gemini.rs (375 LOC) | — |
| Ollama adapter | REAL | fever-providers/src/adapters/ollama.rs (417 LOC) | — |
| 13 env var auto-discovery | REAL | fever-cli/src/main.rs | — |
| Model prefix routing | REAL | "openai/gpt-4o" → openai provider | 3 tests |
| Streaming (OpenAI/Ollama) | REAL | SSE-based StreamChunk emission | Via unit tests |
| Streaming (Anthropic) | MISSING | Code says "not yet implemented" | 0 |
| Streaming (Gemini) | MISSING | chat_stream returns non-streaming | 0 |
| Function calling / tool schemas | PARTIAL | ToolDefinition exists in models, NOT sent in requests | 0 |
| RetryPolicy | REAL | fever-core/src/retry.rs (exp/linear/fixed) | 0 |
| RetryPolicy wired to providers | MISSING | — | 0 |
| Rate limit handling | REAL | ProviderError::RateLimit variant | 0 |

## Tool System

| Feature | Status | Evidence | Tests |
|---------|--------|----------|-------|
| Tool trait | REAL | fever-core/src/tool.rs | 7 tests |
| ToolRegistry | REAL | Register, get, execute_call, schemas | 6 tests |
| ShellTool | REAL | fever-tools/src/shell.rs | — |
| FilesystemTool | REAL | fever-tools/src/filesystem.rs (165 LOC) | 14 tests |
| GrepTool | REAL | fever-tools/src/grep_tool.rs (145 LOC) | — |
| GitTool | REAL | fever-tools/src/git_tool.rs (173 LOC) | — |
| BrowserTool | STUB | fever-browser/src/tool.rs (144 LOC) — all placeholder | 0 |
| SearchTool | PARTIAL | fever-search crate exists, not registered as tool | 0 |
| PermissionGuard | REAL | fever-core/src/permission.rs (573 LOC) | 16 tests |
| PermissionGuard in tool execution | MISSING | Tools bypass PermissionGuard | 0 |
| Secret redaction | REAL | OpenAI keys, GitHub PATs, key=value pairs | 4 tests |
| Path allowlisting | REAL | normalize_and_validate_path | 4 tests |
| Command risk classification | REAL | Low/Medium/High/Critical | 2 tests |

## Verification

| Feature | Status | Evidence | Tests |
|---------|--------|----------|-------|
| OperationalVerifier | REAL | fever-agent/src/operational_verifier.rs (365 LOC) | 5 tests |
| Build check (cargo build) | REAL | — | 1 test |
| Test check (cargo test) | REAL | — | 1 test |
| Lint check (cargo clippy) | REAL | — | 1 test |
| Format check (cargo fmt) | REAL | — | 1 test |
| Custom command check | REAL | — | 1 test |
| Timeout enforcement | REAL | Per-check timeout with kill | 1 test |
| Missing tool graceful | REAL | Clippy not installed → passed | 1 test |
| Verification in agent loop | MISSING | Verifier exists, not called after code changes | 0 |
| Diff review | MISSING | — | 0 |
| Self-critique | MISSING | — | 0 |

## TUI / Terminal UI

| Feature | Status | Evidence | Tests |
|---------|--------|----------|-------|
| Elm architecture (AppState/Message/Command) | REAL | fever-tui/src/app.rs (654 LOC) | — |
| Async event loop (tokio::select!) | REAL | run_loop_async with mpsc channels | — |
| Crossterm backend | REAL | Keyboard, mouse, terminal control | — |
| Ratatui rendering | REAL | Layout, widgets, styling | — |
| Home screen | REAL | Hero, provider/model display, navigation hints | — |
| Chat screen | REAL | Message list + input area | — |
| Settings screen | REAL | Tabbed (Providers/Models/Behavior/Theme) | — |
| Onboarding screen | REAL | Provider/model selection | — |
| MessageBubble component | REAL | Per-role styling, streaming indicator | — |
| InputBar component | REAL | Multi-line, mode indicator (chat/command/search) | — |
| ToolCard component | REAL | Running/Completed/Failed states | — |
| StatusBar component | REAL | Provider, model, workspace, streaming | — |
| Progress/Spinner component | REAL | Animated frames | — |
| Command palette (Ctrl+K) | REAL | Fuzzy search over slash commands | 6 tests |
| Slash commands | REAL | /help, /clear, /settings, /status, /version, /model, /role, /provider, /quit | — |
| Theme: Cold Sacred | REAL | Truecolor + 16-color fallback | — |
| Agent bridge (streaming) | REAL | FeverAgentHandle → LoopDriver → TUI | — |
| Plan/task panel | MISSING | — | 0 |
| Verification results panel | MISSING | — | 0 |
| Tool execution log panel | MISSING | — | 0 |
| Diff preview panel | MISSING | — | 0 |
| Agent activity / handoff panel | MISSING | — | 0 |
| Notification center (Telegram) | MISSING | — | 0 |
| Real-time tool events during loop | PARTIAL | Events emitted AFTER loop completes | 0 |
| Egypt-themed panel names | MISSING | No Mission/Scrolls/Archive/Forge naming | 0 |

## Telegram Integration

| Feature | Status | Evidence | Tests |
|---------|--------|----------|-------|
| TelegramService | REAL | fever-telegram/src/service.rs (191 LOC) | — |
| TelegramApi (Bot API wrapper) | REAL | fever-telegram/src/api.rs | — |
| TelegramClient trait | REAL | Real + Mock implementations | — |
| sendMessage | REAL | With retry backoff (5 attempts) | — |
| getUpdates / polling | REAL | Long polling with offset tracking | — |
| Auto-link chat_id | REAL | Polls for first message, links automatically | 1 test |
| Rate limiting | REAL | Configurable minimum interval | 1 test |
| BotCommand parsing | REAL | /status, /pause, /resume, /stop, /summary, /files, /log, /help | 2 tests |
| Reconnect with backoff | REAL | Exponential backoff | 1 test |
| TelegramEvent formatting | REAL | Event → message string | 1 test |
| Config from env | REAL | TELEGRAM_BOT_TOKEN, TELEGRAM_CHAT_ID, etc. | 2 tests |
| Connected to EventBus | MISSING | — | 0 |
| Connected to LoopDriver | MISSING | — | 0 |
| Approval request flow | MISSING | — | 0 |
| Remote pause/resume/abort | MISSING | — | 0 |

## Memory / Persistence

| Feature | Status | Evidence | Tests |
|---------|--------|----------|-------|
| MemoryStore | REAL | fever-core/src/memory.rs (140 LOC), SQLite-backed | 0 |
| Context key-value storage | REAL | session_id + key → JSON value | 0 |
| Message history storage | REAL | session_id + role + content + timestamp | 0 |
| Session persistence | MISSING | MemoryStore never instantiated in runtime | 0 |
| Session resume | MISSING | — | 0 |
| Plan snapshots | MISSING | — | 0 |
| Decision logs | MISSING | — | 0 |

## Configuration

| Feature | Status | Evidence | Tests |
|---------|--------|----------|-------|
| TOML config | REAL | ~/.config/fevercode/config.toml | — |
| ConfigManager | REAL | Read/write/create dirs | — |
| Provider config | REAL | api_key, base_url, model, extra | — |
| Default provider/model | REAL | Config.defaults section | — |
| Feature flags | MISSING | — | 0 |
| Theme selection | MISSING | Hard-coded to Cold Sacred | 0 |

## Event System

| Feature | Status | Evidence | Tests |
|---------|--------|----------|-------|
| EventBus | REAL | fever-core/src/event.rs, pub/sub | 3 tests |
| Event enum | REAL | PlanCreated, TaskStarted, ToolCalled, etc. | — |
| Dead subscriber cleanup | REAL | Automatic | 1 test |
| Used by any component | MISSING | — | 0 |

## Project Understanding

| Feature | Status | Evidence | Tests |
|---------|--------|----------|-------|
| Repo structure scanning | MISSING | — | 0 |
| Language/framework detection | MISSING | — | 0 |
| Architecture analysis | MISSING | — | 0 |
| Codebase summary | MISSING | — | 0 |
| Entry point identification | MISSING | — | 0 |
| Build/test command detection | MISSING | — | 0 |
| Risk area detection | MISSING | — | 0 |
| Codebase map | MISSING | — | 0 |

---

## Summary Counts

| Category | REAL | PARTIAL | STUB | MISSING |
|----------|------|---------|------|---------|
| Core Agent | 8 | 0 | 1 | 2 |
| Intelligence | 7 | 0 | 0 | 1 |
| Providers | 10 | 1 | 0 | 3 |
| Tools | 8 | 1 | 1 | 1 |
| Verification | 8 | 0 | 0 | 3 |
| TUI | 17 | 1 | 0 | 8 |
| Telegram | 10 | 0 | 0 | 4 |
| Memory | 3 | 0 | 0 | 4 |
| Config | 4 | 0 | 0 | 2 |
| Events | 3 | 0 | 0 | 1 |
| Understanding | 0 | 0 | 0 | 8 |
| **Total** | **78** | **3** | **2** | **36** |
