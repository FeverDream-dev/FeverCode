# FeverCode MVP Specification

**Date**: 2026-04-02
**Version**: 0.2.0-target

---

## MVP Definition

The MVP is the **minimum set of features that makes FeverCode a genuinely useful terminal coding agent** — not a demo, not a scaffold, but something a real developer would reach for.

### Core Principle
A user opens FeverCode, types a coding goal, and the system:
1. Understands their repo
2. Plans how to achieve the goal
3. Executes changes with tool calls
4. Verifies each change compiles/tests
5. Shows everything happening in real-time
6. Can be monitored from Telegram

---

## MVP Feature Set

### Must Have (Ships in MVP)

| # | Feature | Crate | Current Status | Work Required |
|---|---------|-------|----------------|---------------|
| 1 | Permission-enforced tool execution | fever-agent | PermissionGuard exists, not wired | Wire into call_tools() |
| 2 | Function calling (tool schemas to provider) | fever-agent + fever-providers | ToolDefinition exists, not sent | Add schemas to ChatRequest |
| 3 | Project understanding on session start | fever-core (new module) | Missing entirely | Build understand.rs |
| 4 | Repo context in agent system prompt | fever-agent | Partial (metadata only) | Inject ProjectSummary |
| 5 | Real-time tool event streaming to TUI | fever-cli + fever-tui | Events shown after loop | Stream during loop |
| 6 | Verification after code changes | fever-agent | Verifier exists, not wired | Wire into LoopDriver |
| 7 | Verification results in TUI | fever-tui | Missing | Add panel/component |
| 8 | Telegram notifications for loop events | fever-telegram | Service exists, not connected | Wire to agent events |
| 9 | Telegram status query | fever-telegram | BotCommand parsing exists | Handle in main loop |
| 10 | Session message history | fever-core | MemoryStore exists, not wired | Instantiate in agent |

### Nice to Have (If Time Allows)

| # | Feature | Crate | Current Status |
|---|---------|-------|----------------|
| 11 | Plan/task panel in TUI | fever-tui | Missing |
| 12 | Diff preview in TUI | fever-tui | Missing |
| 13 | Egypt-themed panel names | fever-tui | Missing |
| 14 | Session resume | fever-core | MemoryStore exists |
| 15 | RetryPolicy wired to providers | fever-providers | Policy exists |
| 16 | `/understand` slash command | fever-tui | Missing |

### Explicitly Out of Scope (Post-MVP)

- Multi-agent orchestration (handoffs between specialized agents)
- Fighting mode in production (arbiter comparing solutions)
- Browser integration (Chrome MCP)
- Web search tool integration
- Custom themes / theme selection UI
- Notification center in TUI
- Remote approval flow from Telegram
- Remote pause/resume/abort from Telegram
- Per-provider streaming for Anthropic/Gemini
- ADR documentation system
- Benchmarker agent
- UX Priest / UI Curator agent

---

## MVP User Flow

```
1. User opens terminal: fever
2. TUI loads, shows Home screen with provider/model/workspace
3. Agent scans repo in background → produces ProjectSummary
4. User types: "Add error handling to the filesystem tool"
5. TUI shows user message, agent starts thinking
6. Loop begins:
   a. Agent calls LLM with repo context + user goal + tool schemas
   b. LLM responds: "I'll read the filesystem tool first" + tool_call(read_file)
   c. TUI shows: ◈ Reading filesystem.rs...
   d. Tool executes with permission check → returns file content
   e. TUI shows: ✓ Read filesystem.rs (42ms)
   f. LLM responds: "Now I'll add error handling" + tool_call(write_file)
   g. TUI shows: ◈ Writing filesystem.rs...
   h. Tool writes file with permission check
   i. TUI shows: ✓ Wrote filesystem.rs (12ms)
   j. Loop continues: LLM may call more tools or finish
7. Verification triggers: cargo build → cargo test
8. TUI shows verification panel:
   ✓ Build: passed (3.2s)
   ✓ Test: passed (5/5 tests, 8.1s)
9. Agent responds: "Done. Added Result<T> return types and proper error propagation."
10. TUI shows final response
11. Telegram receives: "✅ Task complete: filesystem error handling. Build+tests passed."
12. User sees summary: files changed, verification status, next suggestions
```

---

## Acceptance Criteria

### AC-1: Tool Execution is Safe
- [ ] Every tool call goes through PermissionGuard.check_command() or check_path()
- [ ] Shell commands are risk-classified before execution
- [ ] Paths outside workspace are rejected
- [ ] Secrets in tool output are redacted before returning to LLM
- [ ] Test: unit test for permission check in call_tools()

### AC-2: Function Calling Works
- [ ] ChatRequest includes tool schemas when tools are registered
- [ ] OpenAI-compatible providers receive tool definitions
- [ ] LLM can generate tool_calls that map to registered tools
- [ ] Test: integration test with mock provider verifying tools field

### AC-3: Agent Understands the Repo
- [ ] On session start, repo is scanned for structure
- [ ] Languages, build system, test setup are detected
- [ ] ProjectSummary is injected into system prompt
- [ ] Agent references repo context in its responses
- [ ] Test: unit test for repo scanning, integration test for prompt injection

### AC-4: TUI Shows Real-Time Progress
- [ ] Tool call started events appear in TUI during loop execution
- [ ] Tool call completed events appear with duration
- [ ] User can see what's happening without waiting for loop to finish
- [ ] Test: manual verification (screenshot or manual test)

### AC-5: Verification Runs After Changes
- [ ] After file-modifying tool calls, verification triggers
- [ ] Build/test results appear in TUI
- [ ] Verification failure is visible and reported to user
- [ ] Test: unit test for verification trigger logic

### AC-6: Telegram Reports Status
- [ ] Session start/stop notifications sent to Telegram
- [ ] Tool execution events sent to Telegram (rate-limited)
- [ ] Verification results sent to Telegram
- [ ] /status command returns current state
- [ ] Test: unit test for event formatting, integration test with mock client

### AC-7: Session Has Memory
- [ ] Message history is persisted across loop iterations
- [ ] Messages survive loop restart (within same session)
- [ ] Test: unit test for MemoryStore read/write

---

## MVP Success Metrics

1. **Compiles**: `cargo check` passes with 0 errors
2. **Tests pass**: `cargo test` passes with no regressions
3. **End-to-end flow works**: User can type a goal, agent executes tools with permissions, verification runs, results visible
4. **Telegram works**: Status updates appear on phone during agent execution
5. **No fake features**: Every claimed feature has real implementation
6. **Docs match code**: README, ARCHITECTURE, FEATURE_MATRIX reflect reality
