# FeverCode Implementation Strategy

**Date**: 2026-04-02
**Based on**: Code-grounded audit, FEATURE_MATRIX.md

---

## Target Architecture Delta

The current crate structure is **correct and should be preserved**. No crate restructuring needed. The gaps are integration wiring, not missing architecture.

### Current Architecture (Keep As-Is)

```
fever-cli        → entrypoints, command parsing, session bootstrapping
fever-tui        → terminal GUI, Elm architecture, panels, input modes
fever-core       → shared domain types, task graph, event bus, permissions, tools
fever-agent      → agent loop, roles, verifier, prompt improver, fighting mode
fever-providers  → LLM provider abstraction, 4 adapters, streaming
fever-tools      → shell, filesystem, git, grep tools
fever-config     → TOML configuration management
fever-search     → DuckDuckGo web search (scaffold)
fever-browser    → Chrome MCP placeholder
fever-telegram   → Telegram bot service with rate limiting
fever-onboard    → Project onboarding wizard
fever-release    → Release notes (inaccurate — needs fix)
```

### What Changes (Integration Wiring + New Modules)

```
fever-core
  + understand/     → NEW: project understanding subsystem
  + verify/         → MOVE from fever-agent: OperationalVerifier → fever-verify
                      (or keep in fever-agent but wire into loop)

fever-agent
  LoopDriver        → ENHANCE: emit real-time events, wire verification
  FeverAgent        → ENHANCE: inject PermissionGuard, send tool schemas
  agent_handle.rs   → ENHANCE: stream tool events during loop, not after

fever-tui
  app.rs            → ENHANCE: add plan panel, task panel, verification panel
  screens/          → ADD: mission screen, archive screen, forge screen
  components/       → ADD: plan_tree, task_list, verification_panel, diff_preview

fever-telegram
  service.rs        → ENHANCE: connect to EventBus, report loop state
```

### What Does NOT Change
- Crate boundaries — they're correct
- Provider adapters — they work
- Tool implementations — they work
- Permission types — they're production-quality
- Role system — well-designed
- TUI Elm architecture — sound pattern
- Config system — functional

---

## Implementation Phases

### Phase 0: Security Wiring (Batch 1 — IMMEDIATE)
**Goal**: Make the existing system safe to use
**Effort**: ~2 hours

1. Wire PermissionGuard into FeverAgent.call_tools()
2. Add workspace root as default allowed path
3. Add risk classification to shell commands before execution
4. Redact secrets in tool output before returning to LLM
5. Add tool schemas to ChatRequest (enables function calling)

**Deliverables**: Tools execute with permission checks, providers receive tool definitions

### Phase 1: Project Understanding (Batch 2)
**Goal**: Agent understands the repo before acting
**Effort**: ~4 hours

1. Build `fever-core/src/understand.rs` module:
   - Scan repo structure (directories, file types, sizes)
   - Detect languages from extensions and config files
   - Identify build system (Cargo.toml, package.json, Makefile, etc.)
   - Find test setup and commands
   - Detect entrypoints (main.rs, index.ts, etc.)
   - Identify framework from dependencies
   - Produce structured `ProjectSummary` type
2. Inject ProjectSummary into agent system prompt
3. Add `/understand` slash command to TUI

**Deliverables**: Agent starts every session with repo context

### Phase 2: Real-Time Loop Visibility (Batch 3)
**Goal**: User sees what's happening during multi-step execution
**Effort**: ~4 hours

1. Enhance FeverAgentHandle to emit Message::ToolCallStarted/Completed during loop
2. Add plan panel to TUI (left sidebar or toggle)
3. Add task list to plan panel
4. Add tool execution log panel
5. Wire LoopEvent stream to TUI via agent bridge

**Deliverables**: TUI shows live tool execution, plan progress, task status

### Phase 3: Verification Gating (Batch 4)
**Goal**: Agent verifies after code changes
**Effort**: ~3 hours

1. Wire OperationalVerifier into LoopDriver (after tool execution)
2. Add verification panel to TUI
3. Auto-detect verification commands from project type
4. Show verification results in TUI and (optionally) Telegram
5. Block loop progression on verification failure (configurable)

**Deliverables**: Build/test/lint runs after code changes, results visible

### Phase 4: Telegram Integration (Batch 5)
**Goal**: Remote monitoring from phone
**Effort**: ~3 hours

1. Wire TelegramService to EventBus
2. Send loop events to Telegram (iteration started, tool executed, verification result)
3. Send approval requests for risky actions
4. Handle inbound commands (/status, /pause, /resume, /stop)
5. Add Telegram status indicator to TUI status bar

**Deliverables**: Full remote monitoring, status queries, pause/resume

### Phase 5: Session Persistence (Batch 6)
**Goal**: Resumable sessions
**Effort**: ~3 hours

1. Instantiate MemoryStore in FeverAgent
2. Persist message history per session
3. Save plan/task state on each iteration
4. Add `/resume` CLI command
5. Save project understanding cache

**Deliverables**: Sessions survive restarts, can be resumed

### Phase 6: TUI Polish (Batch 7)
**Goal**: Premium terminal experience
**Effort**: ~6 hours

1. Add Egypt-themed panel names (Mission, Scrolls, Archive, Forge, Tribunal, Herald)
2. Redesign layout with configurable panels
3. Add diff preview panel
4. Add notification center for Telegram events
5. Improve keyboard shortcuts (tab switching, panel focus, jump to error)
6. Add terminal size detection and graceful degradation
7. Improve color palette (sand/gold/lapis/obsidian options)

**Deliverables**: TUI feels like a command temple, not a chatbot

### Phase 7: Hardening (Batch 8)
**Goal**: Production-ready
**Effort**: ~4 hours

1. Add integration tests for full loop with mocked provider
2. Add tests for permission enforcement in tool execution
3. Fix fever-release inaccurate claims
4. Consolidate duplicate ProviderConfig
5. Remove dead code in main.rs
6. Add audit logging
7. Improve error recovery in loop

**Deliverables**: Honest docs, comprehensive tests, clean codebase

---

## Estimated Total Effort

| Phase | Hours | Cumulative |
|-------|-------|------------|
| Batch 1: Security + Function Calling | 2 | 2 |
| Batch 2: Project Understanding | 4 | 6 |
| Batch 3: Real-Time Loop Visibility | 4 | 10 |
| Batch 4: Verification Gating | 3 | 13 |
| Batch 5: Telegram Integration | 3 | 16 |
| Batch 6: Session Persistence | 3 | 19 |
| Batch 7: TUI Polish | 6 | 25 |
| Batch 8: Hardening | 4 | 29 |

---

## Key Design Decisions

### D1: Keep ExecutionEngine, Don't Replace
The ExecutionEngine exists with a clean Plan/Task model. Replace `simulate_task()` with real agent-driven execution rather than building a new execution system.

### D2: PermissionGuard as Decorator, Not Core Change
Wrap tool execution in PermissionGuard checks without modifying tool implementations. This keeps tools simple and security composable.

### D3: Tool Schemas via Provider, Not Custom Protocol
Send tool schemas through the existing ChatRequest.tools field using OpenAI-compatible function calling. This works with OpenAI, Ollama, and can be adapted for Anthropic/Gemini.

### D4: EventBus as Glue, Not Backbone
Use EventBus for cross-component communication (Telegram notifications, TUI updates, logging) but keep the core loop synchronous within LoopDriver. Don't over-architect with event-driven everything.

### D5: Incremental TUI Panels
Add panels one at a time: chat works → add tool log → add plan view → add verification → add notifications. Each panel is independently testable.

### D6: Project Understanding as Structured Data, Not Prose
The understand module produces typed Rust structs (ProjectSummary, LanguageInfo, BuildConfig) that can be serialized, displayed, and injected into prompts. Not just a text summary.

---

## Risk Mitigations

| Risk | Mitigation |
|------|------------|
| Arbitrary shell execution | PermissionGuard + risk classification (Phase 0) |
| Runaway agent loops | Max iterations + timeout + user abort (already exists) |
| Secret leakage | Redaction in tool output (Phase 0) |
| TUI freezing during long ops | Async event loop already handles this; tool events stream in real-time (Phase 3) |
| Telegram command spoofing | Command whitelist + no arbitrary execution from Telegram (Phase 5) |
| Stale plan state | Plan updated on every iteration; verification gates prevent silent failures (Phase 4) |
