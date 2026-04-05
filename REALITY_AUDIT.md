# FeverCode Reality Audit v2

**Date**: 2026-04-05
**Scope**: Full repo-wide truth audit — providers, features, tests, docs
**Commit**: 6adfcc9 (fix(ci): resolve clippy strict errors in config tests)

---

## Executive Summary

FeverCode has **5 adapter implementations** (openai, anthropic, gemini, ollama, mock) but README claims "12 provider adapters" — **MISLEADING**. The OpenAI adapter has factory methods for 9 providers (openai, openrouter, together, groq, fireworks, mistral, deepseek, minimax, perplexity), all sharing the same `OpenAiAdapter` struct. Z.ai is registered with the **wrong base URL** (openrouter.ai instead of api.z.ai). Ollama streaming is **BROKEN** (returns InvalidRequest). Anthropic tools are **NOT WIRED** (request body has no tools field).

Of 190 tests, most are in fever-core/permission (21), fever-providers (36), fever-agent (loop_driver tests), and fever-telegram (7). fever-browser and fever-search have **ZERO tests**. Browser tool is a **PLACEHOLDER** (all methods return hardcoded JSON). Search/DuckDuckGo exists as a standalone client but is **NOT WIRED** as a tool. Telegram is a standalone service — **NOT WIRED** into the agent loop.

---

## 1. Provider Architecture Truth Matrix

| Provider | Adapter | Streaming | Tools | Model Listing | Config Discovery | Status |
|----------|---------|-----------|-------|---------------|-----------------|--------|
| OpenAI | OpenAiAdapter | ✅ SSE | ✅ | ✅ Real API | `OPENAI_API_KEY` | **REAL** |
| Anthropic | AnthropicAdapter | ✅ SSE | ❌ Not wired | ❌ Hardcoded | `ANTHROPIC_API_KEY` | **PARTIAL** |
| Gemini | GeminiAdapter | ✅ | ✅ | ✅ Real API | `GEMINI_API_KEY` | **REAL** |
| Ollama | OllamaAdapter | ❌ Returns InvalidRequest | ⚠️ Partial | ❌ Hardcoded/cache | Always registered | **BROKEN** (streaming) |
| Mock | MockProvider | ✅ Word-by-word | ❌ Claimed | ❌ Hardcoded | `--mock` flag | **REAL** |
| OpenRouter | OpenAiAdapter | ✅ (via OpenAI) | ✅ (via OpenAI) | ✅ Real API | `OPENROUTER_API_KEY` | **REAL** (profile) |
| Groq | OpenAiAdapter | ✅ (via OpenAI) | ✅ (via OpenAI) | ✅ Real API | `GROQ_API_KEY` | **REAL** (profile) |
| Together | OpenAiAdapter | ✅ (via OpenAI) | ✅ (via OpenAI) | ✅ Real API | `TOGETHER_API_KEY` | **REAL** (profile) |
| DeepSeek | OpenAiAdapter | ✅ (via OpenAI) | ✅ (via OpenAI) | ✅ Real API | `DEEPSEEK_API_KEY` | **REAL** (profile) |
| Mistral | OpenAiAdapter | ✅ (via OpenAI) | ✅ (via OpenAI) | ✅ Real API | `MISTRAL_API_KEY` | **REAL** (profile) |
| Fireworks | OpenAiAdapter | ✅ (via OpenAI) | ✅ (via OpenAI) | ✅ Real API | `FIREWORKS_API_KEY` | **REAL** (profile) |
| Perplexity | OpenAiAdapter | ✅ (via OpenAI) | ✅ (via OpenAI) | ✅ Real API | `PERPLEXITY_API_KEY` | **REAL** (profile) |
| MiniMax | OpenAiAdapter | ✅ (via OpenAI) | ✅ (via OpenAI) | ✅ Real API | `MINIMAX_API_KEY` | **REAL** (profile) |
| Z.ai | OpenAiAdapter | ❌ Wrong base URL | ❌ Wrong base URL | ❌ Wrong base URL | `FEVER_ZAI_KEY` | **BROKEN** |

### Key Issues
- **5 adapter implementations**, not 12. The other 8 "providers" are factory methods on OpenAiAdapter.
- **README claims "12 LLM providers with streaming"** — only 10 have working streaming (Ollama broken, Z.ai broken)
- **README claims "12 provider adapters"** — should say "5 adapters, 14 provider profiles"
- **Z.ai uses `OpenAiAdapter::openrouter()`** which points to `openrouter.ai` — completely wrong
- **Ollama uses OpenAI-compat endpoint** (`/v1/chat/completions`) instead of native (`/api/chat`) — streaming not implemented because it would need NDJSON parsing
- **Ollama uses `/v1/chat/completions`** for chat but the native endpoint is `/api/chat` with different response format
- **Anthropic `AnthropicRequestBody` struct has NO tools field** — capabilities claim `supports_tools: true` but tools are silently dropped

---

## 2. Feature Truth Matrix

| Feature | Claimed | Status | Evidence |
|---------|---------|--------|----------|
| PermissionGuard | ✅ | **REAL** | Defined in `fever-core/src/permission.rs` (725 lines, 21 tests). Wired in `fever-cli/main.rs:946-955` (grants all scopes). Wired in `fever-agent/src/agent.rs:267-292` (checks commands and paths). |
| EventBus | ✅ | **STUB** | Defined in `fever-core/src/event.rs` (90 lines, 3 tests). Exported from `fever-core/src/lib.rs`. **NEVER used outside tests** — not wired into TUI, agent, or any runtime path. |
| MemoryStore (SQLite) | ✅ | **STUB** | Defined in `fever-core/src/memory.rs` (15 lines). Exported from lib.rs. **NEVER instantiated** anywhere — no runtime usage, no tests. |
| ExecutionEngine | ✅ | **STUB** | Defined in `fever-core/src/execution.rs` (50 lines). One test in `core_tests.rs:332` just creates it. **Never used in runtime**. |
| LoopDriver | ✅ | **PARTIAL** | Defined in `fever-agent/src/loop_driver.rs`. Has tests (loop_driver_tests.rs, loop_driver_edge_cases.rs). `agent.rs:87` creates one, but `agent_handle.rs` **bypasses it entirely** with its own `run_streaming_loop()`. |
| Browser tool | ✅ | **PLACEHOLDER** | `fever-browser/src/tool.rs` (144 lines). Implements `Tool` trait but **all methods return hardcoded JSON** with "placeholder - requires Chrome MCP". Not registered in `build_tool_registry()`. |
| Search (DuckDuckGo) | ✅ | **PARTIAL** | `fever-search/src/` has real HTTP client, DuckDuckGo parser, Searxng parser, cache. But `SearchClient` is **NOT registered as a tool** in `build_tool_registry()`. |
| Telegram | ✅ | **PARTIAL** | `fever-telegram/src/service.rs` has full `TelegramService` with start/stop/send_event/poll_commands. 7 test files. But **not wired into agent loop** — standalone service only. |
| Config cascade | ✅ | **REAL** | `fever-config/src/config.rs:239` `load_with_workspace()` exists. Called in `main.rs`? No — `main.rs:759-763` uses `cm.load()` (single config, no cascade). **`load_with_workspace` is never called in production code.** |
| Instruction discovery | ✅ | **PARTIAL** | `fever-core/src/instructions.rs` `discover_instructions()` works (4 tests). But **never called in runtime** — not wired into agent prepare_request or TUI. |
| Telemetry | ✅ | **STUB** | `fever-core/src/telemetry.rs` has TelemetryEvent, MemorySink, JsonlSink, Telemetry struct (5 tests). **Never instantiated in runtime** — only in test functions. |
| Session persistence | ✅ | **REAL** | JSONL save/load implemented in TUI `app.rs`. `fever session list/clear` works in CLI. |
| Slash commands | ✅ | **REAL** | 19 commands in `fever-tui/src/slash/commands.rs` (13 tests). All implemented in app.rs handlers. |
| Doctor diagnostics | ✅ | **REAL** | `run_doctor()` in main.rs checks config, env vars, git, TTY. TUI `/doctor` has 19 checks. |
| Command palette | ✅ | **REAL** | Ctrl+K fuzzy search implemented in app.rs. |
| Mock provider | ✅ | **REAL** | `--mock` flag works end-to-end. MockProvider with deterministic streaming (1 test). |
| Provider switching | ✅ | **REAL** | `/provider` and `/model` commands work in TUI. |
| Theme system | ✅ | **REAL** | 11 themes including anubis. |
| Mouse support | ✅ | **REAL** | Enabled in TUI. |
| Config file providers | ✅ | **REAL** | `config.toml` `[providers]` section parsed in main.rs:236-273. Custom adapters created via `OpenAiAdapter::custom()`. |

---

## 3. Test Landscape

**Total: 190 test functions across 22 files**

| Crate | Test Files | Test Count | Quality |
|-------|-----------|------------|---------|
| fever-providers | 2 (provider_unit_tests, openrouter_integration) | 36 | Unit + mock |
| fever-core | 1 (core_tests) | 29 | Unit (EventBus, ExecutionEngine, permissions) |
| fever-core/permission.rs | inline | 21 | Thorough unit tests |
| fever-agent | 2 (loop_driver_tests, loop_driver_edge_cases) | ~25 | Unit with mock |
| fever-agent/src/requirements_interrogator.rs | inline | 15 | Unit |
| fever-agent/src/fighting_mode.rs | inline | 10 | Unit |
| fever-tui | 2 (commands, glyphs) | 18 | Unit |
| fever-telegram | 7 test files | ~10 | Unit with mocks |
| fever-config | 1 (config.rs inline) | 5 | Unit |
| fever-onboard | 2 (fever_onboard_tests, scaffold inline) | 20 | Unit |
| fever-cli | 1 (cli_integration) | 6 | Integration (basic) |
| fever-tools | 1 (filesystem_tests) | ~3 | Unit |
| **fever-browser** | **NONE** | **0** | **NO TESTS** |
| **fever-search** | **NONE** | **0** | **NO TESTS** |

### Test Gaps
- No streaming tests for OpenAI adapter
- No streaming tests for Anthropic adapter
- No tests for Gemini adapter at all
- No conformance tests (same ChatRequest → verify response shape per adapter)
- No tests for `build_provider_client()` registration logic
- No tests for Z.ai, Ollama streaming, Anthropic tools

---

## 4. CLI Wiring Analysis

### Provider Registration Flow (main.rs:112-276)
```
build_provider_client(fetch_models) →
  for each env var:
    create adapter → optionally fetch_models → register(Arc<adapter>, is_first)
  then:
    load config.toml → for each enabled provider with api_key:
      create OpenAiAdapter::custom(name, key, base_url) → register
    set default provider from config
```

### Issues
1. **Z.ai registered as `OpenAiAdapter::openrouter(key)`** — wrong factory, wrong URL
2. **Ollama always registered** even when not running — no health check
3. **Model routing is fragile** — `model.split('/')` prefix match. "openai/gpt-4o" → resolves to "openai". But "gpt-4o" (no prefix) falls to default provider
4. **Default model hardcoded** — `openai/gpt-4o` in multiple places (main.rs:601, 787, 959)
5. **Config cascade not used** — `load_with_workspace()` exists but `main.rs` calls `cm.load()`

### TUI Provider Wiring
- `main.rs:928-983`: Creates ProviderClient → FeverAgentHandle → AppState
- `app.provider_name` and `app.model_name` set from handle.default_model()
- `/provider` and `/model` commands in app.rs modify `app.provider_name`/`app.model_name`
- But switching provider/model in TUI **does NOT actually change the ProviderClient** — it only changes display strings

---

## 5. Doc Claims vs Reality

| Claim | Reality | Fix |
|-------|---------|-----|
| "12 LLM providers" | 5 adapters, 14 profiles (2 broken) | Fix count or fix Z.ai/Ollama |
| "12 provider adapters" | 5 adapter implementations | Say "5 adapters, 14 profiles" |
| "streaming responses" | Ollama streaming broken, Z.ai broken | Fix implementations |
| "robust tool-execution loop" | Loop exists but bypasses LoopDriver, tool detection via separate non-streaming call | Document accurately |
| "Browser — Chrome MCP integration" | Placeholder returning hardcoded JSON | Mark as TODO or implement |
| "DuckDuckGo search" | Client exists but not wired as tool | Wire or mark TODO |
| "Telegram loop monitor" | Standalone service, not wired into agent | Wire or document standalone |
| "Project Intelligence — config cascade" | `load_with_workspace()` never called | Wire or remove claim |
| "instruction files discovered" | `discover_instructions()` never called in runtime | Wire or remove claim |

---

## 6. Critical Fixes (Priority Order)

### P0 — Broken Functionality
1. **Fix Z.ai**: Change from `OpenAiAdapter::openrouter()` to `OpenAiAdapter::custom("zai", key, "https://api.z.ai/api/paas/v4")` with env var `ZAI_API_KEY`
2. **Fix Ollama streaming**: Implement NDJSON streaming using native `/api/chat` endpoint
3. **Wire Anthropic tools**: Add `tools` field to `AnthropicRequestBody` and handle tool_use response blocks

### P1 — Architecture
4. **ProviderProfile struct**: Separate configuration/metadata from adapter implementation
5. **ProviderRegistry**: Central discovery, validation, health checks, capability queries
6. **Fix TUI provider switching**: Actually change the active adapter, not just display strings

### P2 — Honesty
7. **Fix README**: "5 adapters, 14 profiles" not "12 providers"
8. **Mark stubs**: Browser, MemoryStore, EventBus, ExecutionEngine, Telemetry
9. **Wire or remove**: Config cascade, instruction discovery, search tool

### P3 — Testing
10. **Adapter conformance tests**: Same request → verify response shape
11. **Streaming tests**: OpenAI, Anthropic, Ollama
12. **Registration tests**: Verify all env vars produce correct adapters
