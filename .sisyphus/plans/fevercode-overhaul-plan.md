# FeverCode Overhaul Plan

**Date**: 2026-04-05
**Scope**: Full UX/TUI overhaul + testable core + claw-code pattern porting
**Target**: `FeverDream-dev/FeverCode` (local Rust workspace)
**Reference**: `ultraworkers/claw-code` (Python + Rust hybrid terminal agent)

---

## DELIVERABLE 1: AUDIT & GAP ANALYSIS

### 1.1 FeverCode Current State (Target Repo)

**Workspace**: 12 crates, ~14,600 LOC, Rust edition 2024, ratatui + crossterm

| Crate | LOC | Status | Functional? |
|-------|-----|--------|-------------|
| fever-core | ~1,400 | Good abstractions, permission excellent, execution engine stub | Partial |
| fever-agent | ~2,600 | Real LoopDriver, roles, verifier, fighting mode, prompt improver | Yes |
| fever-providers | ~2,100 | 4 real adapters (OpenAI, Anthropic, Gemini, Ollama), streaming | Yes |
| fever-tools | ~600 | 4 working tools (Shell, Filesystem, Grep, Git), 1 placeholder (Browser) | Mostly |
| fever-tui | ~1,300 | Elm architecture, 4 screens (Home, Chat, Settings, Onboarding), command palette, slash commands | Yes |
| fever-cli | ~460 | 10 subcommands, provider/chat/version | Yes |
| fever-config | ~240 | Config read/write | Yes |
| fever-telegram | ~500 | Real service, rate limiting | Yes (standalone) |
| fever-onboard | ~1,100 | 21-question onboarding + scaffold generation | Yes |
| fever-search | ~400 | DuckDuckGo scaffold | No (not integrated) |
| fever-browser | ~250 | Data models + placeholder tool | No |
| fever-release | ~60 | Release note template (inaccurate claims) | Yes but misleading |

**Tests**: 122 passing + 3 ignored (require env vars)

**TUI Architecture**:
- Elm-style: `AppState.update(Message) -> Vec<Command>`
- 4 screens: Home (hero), Chat (messages + input), Settings (8 tabs), Onboarding
- Command palette (Ctrl+K) with basic fuzzy filter
- Slash commands: 21 commands, simple enum-based, no metadata
- Status bar: provider, model, theme, workspace, tokens, cost, elapsed
- Sidebar (Ctrl+B): navigation + quick commands + telemetry
- Help overlay (?): keyboard shortcuts
- Mouse: scroll (mapped to PageUp/Down), click (basic), no hover

**Slash Commands (current)**:
- Simple `enum SlashCommand` with 21 variants
- No aliases, no categories, no fuzzy search, no parameter hints
- Basic prefix-only matching (`name.starts_with(&query)`)
- No structured metadata (name, description only)

**Session Persistence**:
- JSON files in `~/.local/share/fevercode/sessions/`
- Basic save on quit, list in /session command
- No resume, no fork, no JSONL, no rotation

**Permission System**:
- PermissionGuard with scopes: ShellExec, FilesystemRead/Write/Delete, GitOperations
- Path allowlisting, command risk classification, secret redaction
- 16 tests — but NOT wired into tool execution

**Agent Loop**:
- LoopDriver: real iterative loop (LLM → parse tools → execute → feed back → repeat)
- Max 20 iterations, termination on stop/no tool calls
- LoopEvent for observability
- NOT connected to TUI's FeverAgentHandle (handle streams char-by-char, no loop events)

**Config**:
- TOML in `~/.config/fevercode/config.toml`
- Flat TOML parsing in AppState (no ConfigManager used by TUI)
- No cascade, no project-level config, no local overrides

### 1.2 Claw-Code Key Patterns (Reference Repo)

**Architecture**: Python + Rust hybrid. Rust workspace at `rust/crates/` with 8 crates.

**Key Patterns Worth Porting**:

1. **Slash Command Spec Table**: `const SLASH_COMMAND_SPECS` with ~90+ commands, each having `name`, `aliases: &[&str]`, `summary`, `argument_hint`, `resume_supported`. Compile-time, zero-allocation.

2. **Typed Command Enum**: `SlashCommand` with per-variant parsed args (`Session { action: Option<String>, target: Option<String> }`). Exact-match parsing with inline alias handling.

3. **Session JSONL Persistence**: Append-only JSONL with typed records (`session_meta`, `message`, `compaction`). Rotation at 256KB. Fork with lineage tracking. Much more robust than FeverCode's JSON dump.

4. **Generic Runtime**: `ConversationRuntime<C: ApiClient, T: ToolExecutor>` — makes the entire loop testable without live providers. FeverCode's FeverAgentHandle is not generic.

5. **Permission Hierarchy**: 5 levels (ReadOnly, WorkspaceWrite, DangerFullAccess, Prompt, Allow) + per-tool requirements + allow/deny/ask rules + hook overrides + workspace boundary checks + bash read-only heuristic.

6. **Config Cascade**: 5-file cascade (user legacy → user settings → project → project settings → local overrides) with deep merge.

7. **CLAUDE.md Discovery**: Upward-walking from CWD looking for instruction files, dedup by content hash, budget caps.

8. **Mock HTTP Server**: Full TCP mock server with scenario-driven responses (12 predefined scenarios).

9. **Telemetry Pipeline**: Structured events (HTTP traces, session traces, analytics) with MemorySink (tests) and JSONLSink (production).

10. **Auto-Compaction**: When cumulative input tokens exceed threshold, summarize older messages, preserve recent N.

### 1.3 Gap Analysis

| Area | FeverCode (Current) | Claw-Code (Reference) | Desired State | Gap Severity |
|------|---------------------|------------------------|---------------|--------------|
| **Slash Commands** | 21 commands, simple enum, no metadata, prefix-only match | 90+ commands, spec table + typed enum, aliases, categories, hints | 30+ commands with spec table, aliases, fuzzy search, categories, descriptions | **HIGH** |
| **Start Screen** | Basic hero + provider info + recent sessions + keybinding hints | N/A (REPL-focused) | Premium branded start screen with actions, diagnostics, mock mode, workspace status | **HIGH** |
| **Command Palette** | Basic fuzzy filter, shows commands only | N/A | Fuzzy search, categories, descriptions, actions beyond slash commands | **MEDIUM** |
| **Input Experience** | Single-line char-by-char, basic history | N/A | Better placeholder, multiline support, paste handling, slash awareness | **MEDIUM** |
| **Mouse Support** | Scroll + basic click | N/A | Hover highlighting, click-to-select in menus, session list clicking | **MEDIUM** |
| **Panel System** | Single chat view + sidebar | N/A | Chat, plan/tasks, tool log, diff panel, status bar improvements | **HIGH** |
| **Session Persistence** | JSON dump, no resume, no fork | JSONL append-only, rotation, fork with lineage | JSONL persistence, resume, fork, rotation | **HIGH** |
| **Permission Modes** | PermissionGuard exists but not wired | 5-level hierarchy + rules + enforcer | Visible permission mode, enforced at runtime, changeable via /permissions | **HIGH** |
| **Mock Provider** | Hardcoded echo response when no agent | Full TCP mock server with 12 scenarios | Deterministic mock provider for testing + demo mode | **HIGH** |
| **Runtime Loop** | LoopDriver exists, not connected to TUI | Generic ConversationRuntime<C,T> | LoopDriver connected to TUI with real-time tool event streaming | **HIGH** |
| **Config Hierarchy** | Flat TOML, no cascade | 5-file cascade with deep merge | Project-level config, local overrides, deep merge | **MEDIUM** |
| **Doctor/Diagnostics** | Basic 6-check in-app + CLI doctor | N/A | Comprehensive diagnostics: provider, workspace, git, permissions, session storage, tools | **MEDIUM** |
| **Diff Visibility** | None | N/A | Git diff surface, modified files list | **MEDIUM** |
| **Auto-Compaction** | None | Token-threshold compaction with summary | Context window management | **LOW** (later) |
| **CLAUDE.md Discovery** | None | Upward-walking instruction file discovery | .fevercode/instructions.md discovery | **LOW** (later) |
| **Telemetry** | None | Structured event pipeline | Session traces for debugging | **LOW** (later) |

### 1.4 Implementation Plan

#### PHASE 1: UX/TUI Overhaul (Deliverable 2)

**P1.1 — Slash Command Registry Overhaul**
- Create `SlashCommandSpec` struct with: name, aliases, summary, argument_hint, category, requires_provider, safe_in_mock
- Build `const SLASH_COMMAND_SPECS: &[SlashCommandSpec]` with 30+ commands
- Create typed `SlashCommand` enum with per-variant parsed args
- Implement fuzzy matching using `fuzzy-matcher` (already a dependency)
- Add categories: session, config, model, permissions, workspace, tools, diagnostics, help, appearance
- Render slash menu popup with: command name, aliases, description, parameter hints, category badge
- Support keyboard navigation (up/down/enter/tab/escape), mouse hover, mouse click selection, scroll

**P1.2 — Premium Start Screen**
- Redesign home screen with:
  - Branded hero area (logo + tagline + version)
  - Provider status indicator (configured / unconfigured / mock mode)
  - Workspace/repo status (detect git, show branch)
  - Quick actions: [Start Session] [Resume] [Configure Provider] [Mock Mode] [Doctor] [Help]
  - Recent sessions list (clickable, with timestamps)
  - Keyboard hints row at bottom
  - Clean visual hierarchy with dividers and spacing
- Make "no provider" state feel operational, not broken
- Add "Run in mock/demo mode" action when no provider is configured

**P1.3 — Command Palette Enhancement**
- Add categories to palette filter
- Show descriptions inline
- Add non-slash actions: "Resume session", "Open settings", "Toggle sidebar", "Switch theme", "Toggle mouse"
- Fuzzy search using `fuzzy-matcher`

**P1.4 — Input Experience**
- Better placeholder text: "Type a message, /command, or Ctrl+K for actions"
- Proper slash awareness: typing "/" immediately shows command menu
- Tab completion for slash commands
- Stable backspace behavior when slash menu is open
- Graceful long-input handling

**P1.5 — Mouse Support Enhancement**
- Click-to-select in slash command menu
- Click on home screen quick actions
- Click on recent sessions
- Hover highlighting where feasible
- Click on settings items

**P1.6 — Status Bar Improvements**
- Show permission mode indicator
- Show session ID (abbreviated)
- Show git branch if in repo
- Show provider connectivity status (connected/disconnected/mock)
- Compact but informative layout

**P1.7 — Panel Skeletons**
- Add Tool Activity panel (list of tool calls with status)
- Add diff panel (placeholder, wire to git diff)
- Make panels toggleable (future: keyboard shortcuts for panel switching)

#### PHASE 2: Real Testable Core (Deliverable 3)

**P2.1 — Mock Provider**
- Create `MockProvider` implementing `ProviderAdapter` trait
- Deterministic responses: scripted responses for common prompts
- Simulate tool calls with scripted results
- Simulate streaming (char-by-char with delay)
- Simulate multi-turn loops (tool call → result → continue)
- Enable `fever --mock` flag to launch in demo mode
- Tests: unit tests for mock responses, integration test for full loop with mock

**P2.2 — Runtime Loop Scaffold**
- Wire LoopDriver into FeverAgentHandle (currently bypassed)
- Stream LoopEvents to TUI during execution (currently tool events shown only after loop completes)
- Show real-time tool call status in TUI (started → running → completed/failed)
- Show iteration count and progress during multi-step execution
- Handle cancellation (Ctrl+C during loop)

**P2.3 — Session Persistence Overhaul**
- Migrate from JSON dump to JSONL append-only format
- Add session metadata: created_at, updated_at, message_count, workspace
- Add session resume: load messages from JSONL and populate TUI
- Add session fork (clone with new ID)
- Add rotation: keep sessions under 256KB
- Wire MemoryStore (currently exists but never instantiated)

**P2.4 — Permission Modes in TUI**
- Add permission mode to AppState: ReadOnly, WorkspaceWrite, DangerFullAccess
- Show current mode in status bar
- Add /permissions command to switch modes
- Gate destructive tool calls based on mode
- Wire PermissionGuard into tool execution (10 lines of glue, per existing audit)

**P2.5 — Enhanced Doctor**
- Check provider config (env vars + config.toml)
- Check workspace access (read/write permissions)
- Check git availability and repo status
- Check shell availability
- Check session storage health
- Check tool availability (shell, git, grep)
- Report all checks with pass/fail + actionable fix suggestions

**P2.6 — Diff Visibility**
- Add /diff command to show git diff of working tree
- Show modified files list with status
- Basic diff rendering in a panel or as system messages

#### PHASE 3: Port Claw-Code Patterns (Deliverable 4)

**P3.1 — Config Cascade**
- Add project-level config: `.fevercode/config.toml`
- Add local overrides: `.fevercode/config.local.toml` (gitignored)
- Implement deep merge: local > project > user > defaults
- Environment variable overrides

**P3.2 — Enhanced Permission System**
- Port 5-level PermissionMode hierarchy
- Add per-tool requirements (bash requires DangerFullAccess)
- Add allow/deny/ask rule syntax
- Add workspace boundary checking
- Add bash read-only heuristic

**P3.3 — Instruction File Discovery**
- Walk from CWD upward looking for `.fevercode/instructions.md`
- Load and include in system prompt
- Budget cap (4K per file, 12K total)

**P3.4 — Telemetry Skeleton**
- Create MemoryTelemetrySink for tests
- Create basic event types: turn_started, tool_execution_started/finished, turn_completed
- Wire into LoopDriver

#### PHASE 4: Polish (Deliverable 5)

**P4.1 — Dead Code Removal**
- Remove BrowserTool placeholder (or mark as clearly unimplemented)
- Fix fever-release inaccurate claims
- Remove unused ExecutionEngine::simulate_task
- Clean up duplicate ProviderConfig

**P4.2 — Naming & Copy Cleanup**
- Consistent "Fever Code" branding throughout
- Consistent tone in help text, error messages, empty states
- Remove mismatched placeholder language

**P4.3 — Documentation**
- Update README to match reality
- Add slash command reference
- Add mock/demo mode documentation
- Add development/testing guide
- Update ARCHITECTURE.md

**P4.4 — Tests**
- Unit tests for slash command registry (parsing, fuzzy matching, aliases)
- Unit tests for session persistence (JSONL read/write)
- Unit tests for permission logic
- Unit tests for diagnostics
- Mock provider integration tests
- TUI smoke tests where practical

**P4.5 — CI**
- Verify cargo fmt, cargo clippy, cargo test, cargo build --release
- Add feature-gated test path for mock mode

---

## Execution Order

1. **P1.1** — Slash command registry (foundation for everything else)
2. **P1.2** — Start screen redesign (first thing users see)
3. **P1.3** — Command palette enhancement
4. **P1.4** — Input experience
5. **P1.5** — Mouse support
6. **P1.6** — Status bar
7. **P1.7** — Panel skeletons
8. **P2.1** — Mock provider
9. **P2.2** — Runtime loop wiring
10. **P2.3** — Session persistence
11. **P2.4** — Permission modes
12. **P2.5** — Enhanced doctor
13. **P2.6** — Diff visibility
14. **P3.1–P3.4** — Claw-code pattern ports
15. **P4.1–P4.5** — Polish

Each step: implement → test → verify → move on.
