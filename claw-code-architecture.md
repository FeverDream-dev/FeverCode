# Claw-Code Exhaustive Architectural Breakdown

> **Repo**: [ultraworkers/claw-code](https://github.com/ultraworkers/claw-code)  
> **SHA**: `3df5dece39312ee48fbc2378895858dc18d5c1e9`  
> **Scale**: ~32,085 Rust LOC across 9 crates  
> **Binary**: `claw`  
> **Default model**: `claude-opus-4-6`  
> **Default permissions**: `danger-full-access`  
> **Analysis date**: 2026-04-05

---

## Table of Contents

1. [Workspace / Crate Structure](#1-workspace--crate-structure)
2. [TUI Architecture](#2-tui-architecture)
3. [Event Loop / Input Handling](#3-event-loop--input-handling)
4. [Slash Command System](#4-slash-command-system)
5. [Session / State Model](#5-session--state-model)
6. [Provider Abstraction](#6-provider-abstraction)
7. [Tool System](#7-tool-system)
8. [Permission / Trust Model](#8-permission--trust-model)
9. [Runtime / Agent Loop](#9-runtime--agent-loop)
10. [Config / First-Run](#10-config--first-run)
11. [Memory / Instructions](#11-memory--instructions)
12. [Status / Cost / Diff UX](#12-status--cost--diff-ux)
13. [Mock / Test Patterns](#13-mock--test-patterns)
14. [Keyboard / Mouse UX](#14-keyboard--mouse-ux)
15. [Command Palette](#15-command-palette)
16. [Home Screen / Start Screen](#16-home-screen--start-screen)
17. [Patterns Worth Porting](#17-patterns-worth-porting)

---

## 1. Workspace / Crate Structure

The Rust workspace lives at `rust/` and is organized into **9 crates**:

```
rust/
├── Cargo.toml                    # Workspace root
├── crates/
│   ├── rusty-claude-cli/         # Binary crate — the `claw` CLI (7,749 lines)
│   │   ├── src/
│   │   │   ├── main.rs           # Monolithic entry point, REPL, streaming, cost display
│   │   │   ├── app.rs            # SessionConfig, slash command dispatch (567 lines)
│   │   │   ├── args.rs           # CLI arg parsing with clap (108 lines)
│   │   │   ├── render.rs         # TerminalRenderer, Spinner, Markdown→ANSI (796 lines)
│   │   │   ├── input.rs          # LineEditor wrapping rustyline (330 lines)
│   │   │   └── init.rs           # Project init (CLAUDE.md generation) (433 lines)
│   │   └── tests/                # Integration tests
│   │       ├── mock_parity_harness.rs
│   │       ├── cli_flags_and_config_defaults.rs
│   │       └── resume_slash_commands.rs
│   ├── runtime/                  # Core agent loop & state (1,690 + 1,246 + 1,648 + 683 + 803 + ...)
│   │   ├── src/
│   │   │   ├── lib.rs            # Module index (159 lines)
│   │   │   ├── conversation.rs   # ConversationRuntime<C,T> — the agent loop (1,690 lines)
│   │   │   ├── session.rs        # Session persistence, JSONL (1,246 lines)
│   │   │   ├── config.rs         # ConfigLoader, 5-layer hierarchy (1,648 lines)
│   │   │   ├── permissions.rs    # PermissionPolicy, rules engine (683 lines)
│   │   │   ├── prompt.rs         # System prompt assembly (803 lines)
│   │   │   ├── bash.rs           # Bash tool implementation
│   │   │   ├── file_ops.rs       # File read/write/edit tools
│   │   │   ├── compact.rs        # Session compaction
│   │   │   ├── mcp_client.rs     # MCP stdio transport client
│   │   │   ├── mcp_manager.rs    # MCP server lifecycle manager
│   │   │   ├── mcp_protocol.rs   # JSON-RPC protocol types
│   │   │   └── ...
│   │   └── tests/
│   │       └── integration_tests.rs
│   ├── commands/                 # Slash command registry & parser (4,257 lines)
│   │   └── src/lib.rs
│   ├── tools/                    # Tool specs & execution dispatch (7,181 lines)
│   │   └── src/lib.rs
│   ├── api/                      # Provider clients (Anthropic, OpenAI compat)
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   └── providers/
│   │   │       ├── mod.rs        # ProviderKind, MODEL_REGISTRY, resolve_model_alias (218 lines)
│   │   │       ├── anthropic.rs  # AnthropicClient (streaming SSE)
│   │   │       └── openai_compat.rs  # OpenAiCompatClient (Grok, etc.)
│   ├── plugins/                  # Plugin system (3,361 lines)
│   │   └── src/lib.rs
│   ├── telemetry/                # Usage tracking & cost estimation (526 lines)
│   │   └── src/lib.rs
│   ├── compat_harness/           # Upstream TS test compatibility shim
│   │   └── src/lib.rs
│   └── mock-anthropic-service/   # Deterministic mock HTTP server (1,123 lines)
│       └── src/lib.rs
```

### Dependency Graph (simplified)

```
rusty-claude-cli
  ├── runtime
  │     ├── api
  │     ├── tools
  │     ├── permissions (internal module)
  │     └── session (internal module)
  ├── commands
  ├── tools
  │     └── runtime (for PermissionMode)
  ├── api
  ├── plugins
  ├── telemetry
  └── compat_harness

mock-anthropic-service
  └── api (reuses types)
```

**Key design observations**:
- The binary crate (`rusty-claude-cli`) is a **monolith** at 7,749 lines — all REPL logic, streaming display, cost formatting, and slash-command orchestration lives in `main.rs`
- `runtime` is the architectural core — it owns the agent loop, session state, config loading, permissions, and MCP management
- `tools` is the largest crate at 7,181 lines — it contains all 40 tool specs as static data plus a large dispatch function
- `commands` at 4,257 lines is a close second — 100+ slash commands, a parser, and per-command handlers

**Evidence** ([Cargo.toml](https://github.com/ultraworkers/claw-code/blob/3df5dece39312ee48fbc2378895858dc18d5c1e9/rust/Cargo.toml)):
```toml
[workspace]
members = [
    "crates/rusty-claude-cli",
    "crates/runtime",
    "crates/commands",
    "crates/tools",
    "crates/api",
    "crates/plugins",
    "crates/telemetry",
    "crates/compat_harness",
    "crates/mock-anthropic-service",
]
```

---

## 2. TUI Architecture

### Critical finding: There is NO full TUI framework

Claw-code does **not** use ratatui, crossterm's full-screen mode, or any widget framework. Instead, it uses a **line-oriented, prompt-centric** approach:

1. **crossterm** — used only for spinner animation (save/restore cursor position, clear line)
2. **rustyline** — handles line editing, history, tab completion
3. **pulldown-cmark + syntect** — renders Markdown to ANSI-colored text inline
4. **stdout** — all output goes to stdout as scrolling text

### Spinner ([render.rs L48-L97](https://github.com/ultraworkers/claw-code/blob/3df5dece39312ee48fbc2378895858dc18d5c1e9/rust/crates/rusty-claude-cli/src/render.rs#L48-L97))

```rust
pub struct Spinner {
    frame_index: usize,
}

impl Spinner {
    const FRAMES: [&str; 10] = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

    pub fn tick(&mut self, label: &str, theme: &ColorTheme, out: &mut impl Write) -> io::Result<()> {
        let frame = Self::FRAMES[self.frame_index % Self::FRAMES.len()];
        self.frame_index += 1;
        queue!(
            out,
            SavePosition,
            MoveToColumn(0),
            Clear(ClearType::CurrentLine),
            SetForegroundColor(theme.spinner_active),
            Print(format!("{frame} {label}")),
            ResetColor,
            RestorePosition
        )?;
        out.flush()
    }

    pub fn finish(&mut self, label: &str, theme: &ColorTheme, out: &mut impl Write) -> io::Result<()> {
        self.frame_index = 0;
        execute!(
            out,
            MoveToColumn(0),
            Clear(ClearType::CurrentLine),
            SetForegroundColor(theme.spinner_done),
            Print(format!("✔ {label}\n")),
            ResetColor
        )?;
        out.flush()
    }
}
```

**How it works**: On each `tick()`, the spinner saves cursor position, moves to column 0, clears the line, prints the next Braille frame + label, then restores cursor. On `finish()`, it clears and prints `✔ label`.

### TerminalRenderer ([render.rs L218-L297](https://github.com/ultraworkers/claw-code/blob/3df5dece39312ee48fbc2378895858dc18d5c1e9/rust/crates/rusty-claude-cli/src/render.rs#L218-L297))

```rust
pub struct TerminalRenderer {
    syntax_set: SyntaxSet,        // syntect syntax definitions
    syntax_theme: Theme,          // base16-ocean.dark color theme
    color_theme: ColorTheme,      // ANSI color mapping
}
```

**Renders Markdown → ANSI** using pulldown-cmark event streaming:
- **Headings**: Bold + cyan (`## Heading` → bold cyan text)
- **Paragraphs**: Normal text with `\n\n` spacing
- **Code blocks**: Syntect syntax highlighting with `╭─ lang` / `╰─` borders
- **Inline code**: Yellow background
- **Lists**: `• ` prefix with proper indentation
- **Tables**: Unicode box-drawing characters
- **Blockquotes**: `│ ` prefix with dimmed text
- **Links**: `[text](url)` format in blue

### ColorTheme

The `ColorTheme` struct maps semantic colors to `Color` values:
- `spinner_active` (cyan), `spinner_done` (green), `spinner_failed` (red)
- `heading`, `code_border`, `inline_code_bg`, etc.

**Porting note**: This is the simplest possible TUI — no screens, no panels, no layout engine. For a ratatui-based app, you'd replace the Spinner with a ratatui spinner widget and the line-oriented rendering with a scrollable paragraph widget. The Markdown renderer is worth porting as-is.

---

## 3. Event Loop / Input Handling

### LineEditor ([input.rs L101-L150](https://github.com/ultraworkers/claw-code/blob/3df5dece39312ee48fbc2378895858dc18d5c1e9/rust/crates/rusty-claude-cli/src/input.rs#L101-L150))

```rust
pub struct LineEditor {
    prompt: String,
    editor: Editor<SlashCommandHelper, DefaultHistory>,
}

impl LineEditor {
    pub fn new(prompt: impl Into<String>, completions: Vec<String>) -> Self {
        let config = Config::builder()
            .completion_type(CompletionType::List)
            .edit_mode(EditMode::Emacs)
            .build();
        let mut editor = Editor::<SlashCommandHelper, DefaultHistory>::with_config(config)
            .expect("rustyline editor should initialize");
        editor.set_helper(Some(SlashCommandHelper::new(completions)));
        editor.bind_sequence(KeyEvent(KeyCode::Char('J'), Modifiers::CTRL), Cmd::Newline);
        editor.bind_sequence(KeyEvent(KeyCode::Enter, Modifiers::SHIFT), Cmd::Newline);
        Self { prompt: prompt.into(), editor }
    }

    pub fn read_line(&mut self) -> io::Result<ReadOutcome> {
        if !io::stdin().is_terminal() || !io::stdout().is_terminal() {
            return self.read_line_fallback();
        }
        match self.editor.readline(&self.prompt) {
            Ok(line) => Ok(ReadOutcome::Submit(line)),
            Err(ReadlineError::Interrupted) => Ok(ReadOutcome::Interrupted),
            Err(ReadlineError::Eof) => Ok(ReadOutcome::Eof),
            Err(error) => Err(io::Error::new(io::ErrorKind::Other, error)),
        }
    }
}
```

### The main REPL loop (simplified, in main.rs)

The event loop is a **blocking loop** — not event-driven:

```
loop {
    1. editor.read_line() → get user input
    2. if starts with "/" → dispatch slash command
    3. else → runtime.run_turn(user_input, permission_prompter)
    4. during streaming: spinner.tick() on each chunk
    5. after completion: spinner.finish(), render Markdown response
    6. display cost summary
    7. push to history
    repeat
```

There is **no async event loop** in the main thread. The runtime uses `mpsc` channels to communicate between the streaming thread and the spinner display.

**Key input behaviors**:
- `Ctrl+J` or `Shift+Enter` → insert newline (multiline input)
- `Ctrl+C` → interrupt current operation or exit
- `Ctrl+D` (EOF) → exit
- `Tab` → cycle through slash command completions
- `Enter` → submit

**ReadOutcome enum**:
```rust
pub enum ReadOutcome {
    Submit(String),
    Interrupted,
    Eof,
}
```

**Porting note**: For ratatui, you'd replace rustyline with a ratatui text input widget. The blocking loop pattern works but you'd want a proper event loop with `crossterm::event::poll()` for mouse support and resize handling.

---

## 4. Slash Command System

### SlashCommandSpec ([commands/src/lib.rs L44-L50](https://github.com/ultraworkers/claw-code/blob/3df5dece39312ee48fbc2378895858dc18d5c1e9/rust/crates/commands/src/lib.rs#L44-L50))

```rust
pub struct SlashCommandSpec {
    pub name: &'static str,
    pub aliases: &'static [&'static str],
    pub summary: &'static str,
    pub argument_hint: Option<&'static str>,
    pub resume_supported: bool,
}
```

### Command Registry (static array, L52+)

All commands are defined in a static `SLASH_COMMAND_SPECS` array. Here are representative entries:

```rust
const SLASH_COMMAND_SPECS: &[SlashCommandSpec] = &[
    SlashCommandSpec { name: "help",       summary: "Show available slash commands", ... },
    SlashCommandSpec { name: "status",     summary: "Show current session status", ... },
    SlashCommandSpec { name: "compact",    summary: "Compact local session history", ... },
    SlashCommandSpec { name: "model",      summary: "Show or switch the active model",
                       argument_hint: Some("[model]"), ... },
    SlashCommandSpec { name: "permissions",summary: "Show or switch permission mode",
                       argument_hint: Some("[read-only|workspace-write|danger-full-access]"), ... },
    SlashCommandSpec { name: "clear",      summary: "Start a fresh local session", ... },
    SlashCommandSpec { name: "cost",       summary: "Show cumulative token usage", ... },
    SlashCommandSpec { name: "resume",     summary: "Load a saved session into REPL",
                       argument_hint: Some("<session-path>"), ... },
    SlashCommandSpec { name: "config",     summary: "Inspect Claude config files",
                       argument_hint: Some("[env|hooks|model|plugins]"), ... },
    // ... 90+ more commands
];
```

### SlashCommand Enum ([commands/src/lib.rs L1045](https://github.com/ultraworkers/claw-code/blob/3df5dece39312ee48fbc2378895858dc18d5c1e9/rust/crates/commands/src/lib.rs#L1045))

```rust
pub enum SlashCommand {
    Help, Status, Sandbox, Compact, Model(/* String */), Permissions(/* String */),
    Clear(/* bool */), Cost, Resume(/* String */), Config(/* String */),
    Doctor, Init, Quit, History, Diff, Undo, Redo,
    MCP(/* McpSubcommand */), Plugins(/* PluginSubcommand */),
    Agents(/* AgentSubcommand */), Skills(/* SkillSubcommand */),
    // ... 50+ more variants
}
```

### Parsing & Validation

The parser:
1. Strips leading `/`
2. Splits on first whitespace to get command name + args
3. Looks up in `SLASH_COMMAND_SPECS` by exact name match
4. On no match: computes **Levenshtein distance** to all known commands and suggests the closest
5. Validates arguments against `argument_hint`

**Evidence**: The `validate_slash_command_input()` function handles all validation and produces error messages with suggestions.

### Command Categories

| Category | Examples |
|----------|----------|
| Session | `/clear`, `/compact`, `/resume`, `/history`, `/undo`, `/redo` |
| Model | `/model`, `/cost` |
| Permissions | `/permissions`, `/sandbox` |
| Config | `/config`, `/doctor`, `/init` |
| MCP | `/mcp list`, `/mcp start`, `/mcp stop`, `/mcp status` |
| Plugins | `/plugins list`, `/plugins install`, `/plugins enable` |
| Agents | `/agents spawn`, `/agents list`, `/agents observe` |
| Skills | `/skills list`, `/skills install` |
| Tools | `/tools list`, `/tools search` |
| Workers | `/workers create`, `/workers list`, `/workers observe` |
| Teams | `/teams create`, `/teams delete` |
| Cron | `/cron create`, `/cron delete`, `/cron list` |
| Memory | `/memory show`, `/memory save` |
| Notes | `/notes add`, `/notes list`, `/notes search` |

---

## 5. Session / State Model

### Session struct ([session.rs L75-L84](https://github.com/ultraworkers/claw-code/blob/3df5dece39312ee48fbc2378895858dc18d5c1e9/rust/crates/runtime/src/session.rs#L75-L84))

```rust
pub struct Session {
    pub version: u32,
    pub session_id: String,
    pub created_at_ms: u64,
    pub updated_at_ms: u64,
    pub messages: Vec<ConversationMessage>,
    pub compaction: Option<SessionCompaction>,
    pub fork: Option<SessionFork>,
    persistence: Option<SessionPersistence>,
}
```

### ConversationMessage ([session.rs L47-L51](https://github.com/ultraworkers/claw-code/blob/3df5dece39312ee48fbc2378895858dc18d5c1e9/rust/crates/runtime/src/session.rs#L47-L51))

```rust
pub struct ConversationMessage {
    pub role: MessageRole,
    pub blocks: Vec<ContentBlock>,
    pub usage: Option<TokenUsage>,
}
```

### ContentBlock (discriminated union)

```rust
pub enum ContentBlock {
    Text { text: String },
    ToolUse { id: String, name: String, input: serde_json::Value },
    ToolResult { id: String, name: String, output: String, is_error: bool },
}
```

### Persistence Format: JSONL

Sessions are saved as **JSONL** (one JSON object per line) in `.claw/sessions/`:
- Each line is a JSON object with a `_type` discriminator
- File naming: `{session_id}.jsonl`
- **Rotation**: When a file exceeds 256KB, it's rotated. Up to 3 rotated files kept (`.1`, `.2`, `.3`)
- **Atomic writes**: Uses `write_atomic()` to prevent corruption

### Session References

- `latest` — symlink to the most recent session
- `last` — the previous session
- `recent` — list of recent sessions

### Compaction ([session.rs](https://github.com/ultraworkers/claw-code/blob/3df5dece39312ee48fbc2378895858dc18d5c1e9/rust/crates/runtime/src/session.rs))

```rust
pub struct SessionCompaction {
    pub count: u32,
    pub removed_message_count: usize,
    pub summary: String,
}
```

When the session exceeds the **auto-compaction threshold** (100K input tokens by default), older messages are summarized into a single `SessionCompaction` entry. The compaction count tracks how many times this has happened.

### Session Forking

```rust
pub struct SessionFork {
    pub parent_session_id: String,
    pub branch_name: Option<String>,
}
```

Sessions can be forked from other sessions, preserving provenance.

---

## 6. Provider Abstraction

### ProviderKind ([api/providers/mod.rs L29-L33](https://github.com/ultraworkers/claw-code/blob/3df5dece39312ee48fbc2378895858dc18d5c1e9/rust/crates/api/src/providers/mod.rs#L29-L33))

```rust
pub enum ProviderKind {
    Anthropic,
    Xai,
    OpenAi,
}
```

### Model Registry ([mod.rs L43-L116](https://github.com/ultraworkers/claw-code/blob/3df5dece39312ee48fbc2378895858dc18d5c1e9/rust/crates/api/src/providers/mod.rs#L43-L116))

A static `MODEL_REGISTRY` maps short aliases to full model names and provider metadata:

| Alias | Resolved Model | Provider | Auth Env Var |
|-------|---------------|----------|--------------|
| `opus` | `claude-opus-4-6` | Anthropic | `ANTHROPIC_API_KEY` |
| `sonnet` | `claude-sonnet-4-6` | Anthropic | `ANTHROPIC_API_KEY` |
| `haiku` | `claude-haiku-4-5-20251213` | Anthropic | `ANTHROPIC_API_KEY` |
| `grok` | `grok-3` | Xai | `XAI_API_KEY` |
| `grok-mini` | `grok-3-mini` | Xai | `XAI_API_KEY` |
| `grok-2` | `grok-2` | Xai | `XAI_API_KEY` |

### resolve_model_alias() ([mod.rs L118-L142](https://github.com/ultraworkers/claw-code/blob/3df5dece39312ee48fbc2378895858dc18d5c1e9/rust/crates/api/src/providers/mod.rs#L118-L142))

```rust
pub fn resolve_model_alias(model: &str) -> String {
    let trimmed = model.trim();
    let lower = trimmed.to_ascii_lowercase();
    MODEL_REGISTRY
        .iter()
        .find_map(|(alias, metadata)| {
            (*alias == lower).then_some(match metadata.provider {
                ProviderKind::Anthropic => match *alias {
                    "opus" => "claude-opus-4-6",
                    "sonnet" => "claude-sonnet-4-6",
                    "haiku" => "claude-haiku-4-5-20251213",
                    _ => trimmed,
                },
                // ...
            })
        })
        .map_or_else(|| trimmed.to_string(), ToOwned::to_owned)
}
```

### ApiClient Trait

```rust
pub trait ApiClient {
    fn stream(&self, request: ApiRequest) -> Result<Box<dyn Iterator<Item = ApiStreamEvent>>, RuntimeError>;
}
```

Two implementations:
- **`AnthropicClient`** — POSTs to `/v1/messages` with SSE streaming
- **`OpenAiCompatClient`** — POSTs to `/v1/chat/completions` with SSE streaming (for Grok/OpenAI)

### Auth Resolution

Auth sources resolved in order:
1. Explicit `--api-key` flag
2. Provider-specific env var (e.g. `ANTHROPIC_API_KEY`)
3. OAuth token (if configured)
4. Error if none found

---

## 7. Tool System

### ToolSpec ([tools/src/lib.rs L100-L105](https://github.com/ultraworkers/claw-code/blob/3df5dece39312ee48fbc2378895858dc18d5c1e9/rust/crates/tools/src/lib.rs#L100-L105))

```rust
pub struct ToolSpec {
    pub name: &'static str,
    pub description: &'static str,
    pub input_schema: Value,            // JSON Schema
    pub required_permission: PermissionMode,
}
```

### All 40 Built-in Tools

The `mvp_tool_specs()` function ([tools/src/lib.rs L384](https://github.com/ultraworkers/claw-code/blob/3df5dece39312ee48fbc2378895858dc18d5c1e9/rust/crates/tools/src/lib.rs#L384)) returns a static `Vec<ToolSpec>`:

| Tool | Permission | Description |
|------|-----------|-------------|
| `bash` | DangerFullAccess | Execute shell commands |
| `read_file` | ReadOnly | Read text files |
| `write_file` | WorkspaceWrite | Write text files |
| `edit_file` | WorkspaceWrite | Find-and-replace in files |
| `list_directory` | ReadOnly | List directory contents |
| `glob_search` | ReadOnly | Glob pattern file search |
| `grep_search` | ReadOnly | Regex content search |
| `web_fetch` | ReadOnly | Fetch URLs |
| `web_search` | ReadOnly | Web search |
| `todo_write` | WorkspaceWrite | Manage todo lists |
| `agent` | DangerFullAccess | Spawn sub-agent |
| `tool_search` | ReadOnly | Search available tools |
| `notebook_edit` | WorkspaceWrite | Edit Jupyter notebooks |
| `lsp_diagnostics` | ReadOnly | Language server diagnostics |
| `lsp_goto_definition` | ReadOnly | Go to definition |
| `lsp_references` | ReadOnly | Find references |
| `lsp_symbols` | ReadOnly | Document/workspace symbols |
| `mcp_tool` | DangerFullAccess | Bridge to MCP tools |
| `task_create` | WorkspaceWrite | Create background task |
| `task_get` | ReadOnly | Get task output |
| `task_list` | ReadOnly | List tasks |
| `task_stop` | WorkspaceWrite | Stop task |
| `worker_create` | WorkspaceWrite | Create worker |
| `worker_get` | ReadOnly | Get worker info |
| `worker_observe` | ReadOnly | Observe worker output |
| `team_create` | WorkspaceWrite | Create team |
| `team_delete` | WorkspaceWrite | Delete team |
| `cron_create` | WorkspaceWrite | Create cron job |
| `cron_delete` | WorkspaceWrite | Delete cron job |
| `cron_list` | ReadOnly | List cron jobs |
| `memory_create` | WorkspaceWrite | Create memory entries |
| `memory_read` | ReadOnly | Read memory |
| `memory_search` | ReadOnly | Search memory |
| `memory_update` | WorkspaceWrite | Update memory |
| `memory_delete` | WorkspaceWrite | Delete memory |
| `git_status` | ReadOnly | Git working tree status |
| `git_diff` | ReadOnly | Git diff |
| `git_log` | ReadOnly | Git log |
| `git_blame` | ReadOnly | Git blame |
| `fuzzy_search` | ReadOnly | Fuzzy file search |

### GlobalToolRegistry ([tools/src/lib.rs L108](https://github.com/ultraworkers/claw-code/blob/3df5dece39312ee48fbc2378895858dc18d5c1e9/rust/crates/tools/src/lib.rs#L108))

```rust
pub struct GlobalToolRegistry {
    plugin_tools: Vec<PluginTool>,
    runtime_tools: Vec<RuntimeToolDefinition>,
    enforcer: Option<PermissionEnforcer>,
}
```

The registry unifies three tool sources:
1. **Built-in** — from `mvp_tool_specs()`
2. **Plugin** — from the plugin manager
3. **Runtime** — from MCP servers

Tool name aliases are resolved:
```rust
("read", "read_file"),
("write", "write_file"),
("edit", "edit_file"),
("glob", "glob_search"),
("grep", "grep_search"),
```

### Execution Dispatch ([tools/src/lib.rs L338-L348](https://github.com/ultraworkers/claw-code/blob/3df5dece39312ee48fbc2378895858dc18d5c1e9/rust/crates/tools/src/lib.rs#L338-L348))

```rust
pub fn execute(&self, name: &str, input: &Value) -> Result<String, String> {
    if mvp_tool_specs().iter().any(|spec| spec.name == name) {
        return execute_tool_with_enforcer(self.enforcer.as_ref(), name, input);
    }
    self.plugin_tools
        .iter()
        .find(|tool| tool.definition().name == name)
        .ok_or_else(|| format!("unsupported tool: {name}"))?
        .execute(input)
        .map_err(|error| error.to_string())
}
```

### ToolSearch

The `search()` method ([L313-L332](https://github.com/ultraworkers/claw-code/blob/3df5dece39312ee48fbc2378895858dc18d5c1e9/rust/crates/tools/src/lib.rs#L313-L332)) provides fuzzy search over all tool names and descriptions, returning a `ToolSearchOutput` with match results and metadata about pending/degraded MCP servers.

---

## 8. Permission / Trust Model

### PermissionMode ([permissions.rs L9-L15](https://github.com/ultraworkers/claw-code/blob/3df5dece39312ee48fbc2378895858dc18d5c1e9/rust/crates/runtime/src/permissions.rs#L9-L15))

```rust
pub enum PermissionMode {
    ReadOnly,           // Can only read files/search
    WorkspaceWrite,     // Can write to workspace files
    DangerFullAccess,   // Can execute arbitrary commands
    Prompt,             // Ask user for each action
    Allow,              // Allow everything silently
}
```

These are **ordered** (`PartialOrd`/`Ord`): `ReadOnly < WorkspaceWrite < DangerFullAccess < Prompt < Allow`

### PermissionPolicy ([permissions.rs L99-L105](https://github.com/ultraworkers/claw-code/blob/3df5dece39312ee48fbc2378895858dc18d5c1e9/rust/crates/runtime/src/permissions.rs#L99-L105))

```rust
pub struct PermissionPolicy {
    active_mode: PermissionMode,
    tool_requirements: BTreeMap<String, PermissionMode>,
    allow_rules: Vec<PermissionRule>,
    deny_rules: Vec<PermissionRule>,
    ask_rules: Vec<PermissionRule>,
}
```

### Authorization Flow ([permissions.rs L164-L268](https://github.com/ultraworkers/claw-code/blob/3df5dece39312ee48fbc2378895858dc18d5c1e9/rust/crates/runtime/src/permissions.rs#L164-L268))

```
1. Check deny_rules    → if match: Deny
2. Check hook override → if Deny: Deny
3. Check allow_rules   → if match: Allow
4. Check ask_rules     → if match: Prompt user
5. Compare active_mode vs required_mode
   - If active >= required: Allow
   - Else: Prompt user (or Deny if no prompter)
```

### PermissionRule

Rules are pattern-matched against tool input JSON. A rule can match on:
- Tool name
- Input field patterns (e.g., file path globs, command prefixes)

### PermissionEnforcer

Wraps the `ToolExecutor` trait. Before executing, it calls `PermissionPolicy::authorize()`. If denied, returns an error string instead of executing.

### PermissionPrompter Trait

```rust
pub trait PermissionPrompter {
    fn decide(&mut self, request: &PermissionRequest) -> PermissionPromptDecision;
}
```

The CLI implements this to show an interactive prompt. In non-interactive mode (piped input), it always denies.

---

## 9. Runtime / Agent Loop

### ConversationRuntime ([conversation.rs L126-L139](https://github.com/ultraworkers/claw-code/blob/3df5dece39312ee48fbc2378895858dc18d5c1e9/rust/crates/runtime/src/conversation.rs#L126-L139))

```rust
pub struct ConversationRuntime<C, T> {
    session: Session,
    api_client: C,
    tool_executor: T,
    permission_policy: PermissionPolicy,
    system_prompt: Vec<String>,
    max_iterations: usize,
    usage_tracker: UsageTracker,
    hook_runner: HookRunner,
    auto_compaction_input_tokens_threshold: u32,
    hook_abort_signal: HookAbortSignal,
    hook_progress_reporter: Option<Box<dyn HookProgressReporter>>,
    session_tracer: Option<SessionTracer>,
}
```

Generic over `C: ApiClient` and `T: ToolExecutor` — this is the key design for testability.

### run_turn() — The Core Loop ([conversation.rs L296-L485](https://github.com/ultraworkers/claw-code/blob/3df5dece39312ee48fbc2378895858dc18d5c1e9/rust/crates/runtime/src/conversation.rs#L296-L485))

```
fn run_turn(user_input, prompter) -> Result<TurnSummary, RuntimeError> {
    1. Push user message to session
    2. LOOP:
       a. Build ApiRequest from system_prompt + session.messages
       b. Call api_client.stream(request) → get SSE events
       c. Build AssistantMessage from events (text + tool_use blocks)
       d. Record usage (input/output tokens)
       e. Push assistant message to session
       f. Extract pending_tool_uses from blocks
       g. If NO pending tools → BREAK
       h. For each pending tool:
          i.   Run PreToolUse hook → may modify input, deny, cancel
          ii.  Check permission policy (with hook context)
          iii. If allowed: execute tool → capture output
          iv.  Run PostToolUse hook (or PostToolUseFailure on error)
          v.   Push tool result to session
       i. LOOP again (model sees tool results, may call more tools)
    3. Maybe auto-compact if over threshold
    4. Return TurnSummary { assistant_messages, tool_results, iterations, usage }
}
```

### TurnSummary

```rust
pub struct TurnSummary {
    pub assistant_messages: Vec<ConversationMessage>,
    pub tool_results: Vec<ConversationMessage>,
    pub prompt_cache_events: Vec<PromptCacheEvent>,
    pub iterations: usize,
    pub usage: Option<TokenUsage>,
    pub auto_compaction: Option<CompactionResult>,
}
```

### Builder Pattern

The runtime uses a builder pattern for configuration:

```rust
let runtime = ConversationRuntime::new_with_features(
    session, api_client, tool_executor, permission_policy, system_prompt, &feature_config,
)
.with_max_iterations(50)
.with_auto_compaction_input_tokens_threshold(100_000)
.with_hook_abort_signal(signal)
.with_session_tracer(tracer);
```

---

## 10. Config / First-Run

### ConfigLoader ([config.rs L202-L293](https://github.com/ultraworkers/claw-code/blob/3df5dece39312ee48fbc2378895858dc18d5c1e9/rust/crates/runtime/src/config.rs#L202-L293))

```rust
pub struct ConfigLoader {
    cwd: PathBuf,
    config_home: PathBuf,
}
```

### 5-Layer Config Hierarchy ([config.rs L229-L256](https://github.com/ultraworkers/claw-code/blob/3df5dece39312ee48fbc2378895858dc18d5c1e9/rust/crates/runtime/src/config.rs#L229-L256))

```rust
fn discover(&self) -> Vec<ConfigEntry> {
    vec![
        ConfigEntry { source: User,   path: "~/.claw.json" },
        ConfigEntry { source: User,   path: "~/.config/claw/settings.json" },
        ConfigEntry { source: Project, path: "<repo>/.claw.json" },
        ConfigEntry { source: Project, path: "<repo>/.claw/settings.json" },
        ConfigEntry { source: Local,  path: "<repo>/.claw/settings.local.json" },
    ]
}
```

**Merge strategy**: Deep merge (later entries override earlier). MCP servers are merged with project-level taking precedence over user-level. All JSON keys are merged recursively.

### RuntimeFeatureConfig

After loading, the config is parsed into:

```rust
pub struct RuntimeFeatureConfig {
    pub hooks: HooksConfig,
    pub plugins: PluginConfig,
    pub mcp: McpConfigCollection,
    pub oauth: Option<OAuthConfig>,
    pub model: Option<String>,
    pub permission_mode: Option<ResolvedPermissionMode>,
    pub permission_rules: RuntimePermissionRuleConfig,
    pub sandbox: Option<SandboxConfig>,
}
```

### First-Run / Init ([init.rs](https://github.com/ultraworkers/claw-code/blob/3df5dece39312ee48fbc2378895858dc18d5c1e9/rust/crates/rusty-claude-cli/src/init.rs))

The `/init` command generates project scaffolding:
- `.claw/settings.json` — starter permissions config (`"defaultMode": "dontAsk"`)
- `.claude/CLAUDE.md` — project instructions (auto-generated from git context)
- Updates `.gitignore` with `.claude/settings.local.json` and `.claude/sessions/`

**InitStatus tracking**:
```rust
pub(crate) enum InitStatus {
    Created,   // New file created
    Updated,   // Existing file updated
    Skipped,   // Already exists, no changes needed
}
```

---

## 11. Memory / Instructions

### System Prompt Assembly ([prompt.rs](https://github.com/ultraworkers/claw-code/blob/3df5dece39312ee48fbc2378895858dc18d5c1e9/rust/crates/runtime/src/prompt.rs))

The `load_system_prompt()` function assembles the system prompt from multiple sources:

1. **Base prompt** — hardcoded instructions about tool usage, safety guidelines
2. **Project context** — auto-detected from the workspace (git info, file tree, language detection)
3. **CLAUDE.md** — user-authored instructions from:
   - `~/.claude/CLAUDE.md` (global)
   - `<repo>/CLAUDE.md` (project root)
   - `<repo>/.claude/CLAUDE.md` (project .claude dir)
   - `<repo>/**/CLAUDE.md` (nested — discovered recursively)
4. **Memory entries** — persisted notes from `/memory` commands

The final system prompt is a `Vec<String>` where each element is a separate system prompt block (for prompt caching optimization).

### ProjectContext

```rust
pub struct ProjectContext {
    pub cwd: PathBuf,
    pub git_branch: Option<String>,
    pub git_remote: Option<String>,
    pub detected_languages: Vec<String>,
    pub file_tree_summary: String,
}
```

### Memory Tools

The memory system provides CRUD over a key-value store:
- `memory_create(name, entity_type, observations)`
- `memory_read(names)` 
- `memory_search(query)`
- `memory_update(name, observations_to_add, observations_to_delete)`
- `memory_delete(names)`

---

## 12. Status / Cost / Diff UX

### Cost Display

After each turn, the CLI displays:
```
Cost: $0.0423 (12,450 input + 1,230 output tokens)
Cache: 8,200 tokens cached (saved $0.0082)
```

**Implementation**: The `UsageTracker` accumulates `TokenUsage` across iterations within a turn and across turns. Pricing is looked up from a `ModelPricing` struct:

```rust
pub struct ModelPricing {
    pub input_per_million: f64,
    pub output_per_million: f64,
    pub cache_read_per_million: f64,
}
```

The `format_usd()` function renders costs with proper precision.

### `/status` command output
```
Session: abc123
Model: claude-opus-4-6
Permission: danger-full-access
Turns: 7
Total cost: $1.2345
Context: 45,000 / 200,000 tokens
```

### `/cost` command output
```
Total: $1.2345
  Input tokens:  890,000  ($0.8900)
  Output tokens:  45,000  ($0.3450)
  Cache savings:          ($0.0105)
```

### Diff Display

When tools like `edit_file` or `write_file` modify files, the CLI shows a **unified diff** with ANSI colors:
- Green (`+`) for additions
- Red (`-`) for deletions
- Context lines in default color

### Spinner States During Turn

1. `⠋ Thinking...` — while streaming API response
2. `⠋ Running bash...` — while executing bash tool
3. `⠋ Reading file...` — while reading files
4. `✔ Done` — when turn completes
5. `✘ Error: ...` — on failure

---

## 13. Mock / Test Patterns

### MockAnthropicService ([mock-anthropic-service/src/lib.rs](https://github.com/ultraworkers/claw-code/blob/3df5dece39312ee48fbc2378895858dc18d5c1e9/rust/crates/mock-anthropic-service/src/lib.rs))

A full HTTP server that deterministically responds to `/v1/messages` requests:

```rust
pub struct MockAnthropicService {
    base_url: String,
    requests: Arc<Mutex<Vec<CapturedRequest>>>,
    shutdown: Option<oneshot::Sender<()>>,
    join_handle: JoinHandle<()>,
}
```

**How it works**:
1. Spawns a real TCP listener on `127.0.0.1:0` (random port)
2. Reads the user message from the request body
3. Looks for `PARITY_SCENARIO:<name>` prefix in the message
4. Returns a pre-scripted SSE response for that scenario
5. Captures all requests for later assertion

### Test Scenarios

10+ scripted scenarios including:
- **basic_chat** — simple text response
- **streaming_response** — chunked SSE with multiple ContentBlockDelta events
- **tool_use_bash** — model requests bash execution
- **tool_use_file_ops** — model reads/writes files
- **permission_denied** — model requests a tool that's denied
- **plugin_tool** — model uses a plugin-provided tool
- **multi_turn** — back-and-forth conversation
- **compaction** — triggers session compaction

### Integration Test Pattern

```rust
#[test]
fn test_basic_chat() {
    let mock = MockAnthropicService::spawn();
    let client = AnthropicClient::new(mock.base_url(), "test-key");
    let runtime = ConversationRuntime::new(
        Session::new(), client, registry, policy, prompt,
    );
    let result = runtime.run_turn("PARITY_SCENARIO:basic_chat", None);
    assert!(result.is_ok());
    // Assert captured requests, session state, etc.
}
```

### Compat Harness

The `compat_harness` crate provides a shim for running the upstream TypeScript test suite's scenarios against the Rust implementation, ensuring behavioral parity.

---

## 14. Keyboard / Mouse UX

### Keyboard Bindings

| Key | Action |
|-----|--------|
| `Enter` | Submit input |
| `Shift+Enter` | Insert newline |
| `Ctrl+J` | Insert newline (alternative) |
| `Ctrl+C` | Interrupt current operation / exit |
| `Ctrl+D` | Exit (EOF) |
| `Tab` | Cycle slash command completions |
| Up/Down arrows | Navigate history (rustyline default) |
| `Ctrl+R` | Reverse search history (rustyline default) |

**Implementation**: All bindings are configured via rustyline's `Config::builder()` and explicit `bind_sequence()` calls:

```rust
editor.bind_sequence(KeyEvent(KeyCode::Char('J'), Modifiers::CTRL), Cmd::Newline);
editor.bind_sequence(KeyEvent(KeyCode::Enter, Modifiers::SHIFT), Cmd::Newline);
```

### Mouse Support

**There is no mouse support**. The CLI is entirely keyboard-driven. No mouse scroll, click, or selection handling exists.

**Porting note**: This is a significant gap for a TUI app. For ratatui, you'd want mouse capture with `EnableMouseCapture`, scroll events on the output area, and click-to-select on the command palette.

---

## 15. Command Palette

**There is no command palette.** The command discovery mechanism is **tab completion** only:

1. User types `/`
2. Presses `Tab`
3. rustyline cycles through matching slash command names
4. After command name + space, Tab cycles through argument completions

The `SlashCommandHelper` (rustyline helper) manages completion state:

```rust
struct SlashCommandHelper {
    completions: Vec<String>,
    current_index: usize,
    current_line: String,
}
```

**Porting note**: A ratatui-based app could add a proper command palette — a fuzzy-searchable popup list triggered by `Ctrl+P` or `/`. This would be a significant UX improvement over tab cycling.

---

## 16. Home Screen / Start Screen

The home screen is **minimal** — just two informational lines:

```
Rusty Claude CLI interactive mode
Type /help for commands. Shift+Enter or Ctrl+J inserts a newline.
```

Plus a startup banner showing:
```
Model: claude-opus-4-6 | CWD: /path/to/project | Session: abc123
```

**No splash screen, no dashboard, no status panels.** The CLI immediately enters the input loop.

**Porting note**: For a ratatui app, you could create a rich home screen with:
- Session summary panel
- Model/permission status bar
- Recent sessions list
- Quick-start suggestions

---

## 17. Patterns Worth Porting

### High-Value Patterns

1. **ConversationRuntime<C, T> generic** — The two-trait generic (`ApiClient` + `ToolExecutor`) makes the entire agent loop testable with mocks. Port this pattern exactly.

2. **PermissionPolicy with rules engine** — The 3-list allow/deny/ask system with pattern matching on tool input is elegant and extensible. The authorization flow (deny → hook override → allow → ask → mode comparison) is well-structured.

3. **JSONL session persistence** — One-line-per-object is simple, append-friendly, and supports rotation. Much better than a single JSON blob.

4. **Builder pattern on runtime** — `with_max_iterations()`, `with_auto_compaction_threshold()`, etc. makes configuration readable.

5. **ToolSpec as static data** — Defining tools as `&'static str` specs with JSON Schema and permission levels is clean and zero-cost.

6. **Mock service pattern** — The `MockAnthropicService` with scenario-based responses is excellent for integration testing.

7. **Config deep-merge with sources** — The 5-layer hierarchy with deep merge and `ConfigEntry` provenance tracking is solid.

8. **Session compaction** — Auto-summarizing when context grows too large is critical for long-running sessions.

### Patterns to Improve

1. **Monolithic main.rs** (7,749 lines) — Break into modules: repl.rs, streaming.rs, display.rs, cost.rs
2. **No mouse support** — Essential for a modern TUI
3. **No command palette** — Tab completion alone is limiting
4. **No async main loop** — The blocking loop prevents concurrent UI updates during streaming
5. **Spinner hack** — SavePosition/RestorePosition is fragile; use ratatui's proper rendering
6. **No syntax highlighting in input** — Only output gets Markdown rendering

### Crate Organization Recommendation for ratatui App

```
your-app/
├── crates/
│   ├── app/              # Binary: main event loop, ratatui setup
│   ├── runtime/          # ConversationRuntime<C,T>, agent loop
│   ├── session/          # Session persistence, JSONL
│   ├── config/           # ConfigLoader, hierarchy
│   ├── permissions/      # PermissionPolicy, rules
│   ├── tools/            # ToolSpec, registry, execution
│   ├── commands/         # SlashCommandSpec, parser, handlers
│   ├── providers/        # ApiClient trait, Anthropic/OpenAI impls
│   ├── render/           # Markdown→ratatui rendering
│   ├── mock-server/      # Test infrastructure
│   └── ui/               # ratatui widgets (command palette, status bar, etc.)
```

---

## Quick Reference: Key Source Locations

| Component | File | Lines | Key Struct/Trait |
|-----------|------|-------|------------------|
| Entry point | `rusty-claude-cli/src/main.rs` | 7,749 | — |
| REPL | `rusty-claude-cli/src/main.rs` L~4000-7749 | ~3,700 | — |
| Markdown renderer | `rusty-claude-cli/src/render.rs` | 796 | `TerminalRenderer` |
| Spinner | `rusty-claude-cli/src/render.rs` L48-97 | 50 | `Spinner` |
| Line editor | `rusty-claude-cli/src/input.rs` | 330 | `LineEditor` |
| Project init | `rusty-claude-cli/src/init.rs` | 433 | `InitReport` |
| Agent loop | `runtime/src/conversation.rs` | 1,690 | `ConversationRuntime<C,T>` |
| Session | `runtime/src/session.rs` | 1,246 | `Session` |
| Config | `runtime/src/config.rs` | 1,648 | `ConfigLoader` |
| Permissions | `runtime/src/permissions.rs` | 683 | `PermissionPolicy` |
| System prompt | `runtime/src/prompt.rs` | 803 | `load_system_prompt()` |
| Slash commands | `commands/src/lib.rs` | 4,257 | `SlashCommandSpec`, `SlashCommand` |
| Tool specs | `tools/src/lib.rs` | 7,181 | `ToolSpec`, `GlobalToolRegistry` |
| Providers | `api/src/providers/mod.rs` | 218 | `ProviderKind`, `resolve_model_alias()` |
| Plugins | `plugins/src/lib.rs` | 3,361 | `PluginManager` |
| Mock server | `mock-anthropic-service/src/lib.rs` | 1,123 | `MockAnthropicService` |
