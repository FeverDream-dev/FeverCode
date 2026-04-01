# Fever v2 — Product Design Document

> "A sacred machine terminal from a lost cold civilization."

---

## 1. Vision Summary

Fever v2 is a full-screen, terminal-native AI coding agent. Not a CLI with subcommands. Not a wrapper around `clap`. A **product** — the same class as OpenCode, Claude Code, and Gemini CLI — with a premium, immersive, keyboard-first experience rooted in cold Egyptian-inspired aesthetics.

The user types `fever` and enters a world: a full-screen interface with a branded startup, provider/model selection, workspace awareness, chat-driven coding, live tool visualization, settings screens, and slash commands. Everything stays in the terminal. Everything feels alive.

**The shift**: Fever v1 is a CLI that happens to have a TUI crate. Fever v2 is a TUI application that happens to have a CLI entry point.

---

## 2. Product Critique of Current Approach

### What exists today

| Crate | Lines | State |
|-------|-------|-------|
| fever-cli | ~280 | Subcommand router. `fever code` is a no-op. `run_tui()` is dead code. |
| fever-tui | ~790 | Bare ratatui proof-of-concept. 5 flat panes, no theme, no streaming, no settings. |
| fever-core | ~1,200 | Clean abstractions (Task, Plan, Tool, EventBus, Permission). Well-tested. |
| fever-providers | ~2,500 | 14 provider adapters. Streaming models exist but TUI doesn't consume them. |
| fever-agent | ~1,800 | LoopDriver, FightingMode, Verifier, PromptImprover. Zero TUI integration. |
| fever-tools | ~1,000 | Shell, filesystem, git, grep tools. Not wired to TUI. |
| fever-config | ~200 | TOML config loader. No TUI for editing. |
| fever-search | ~500 | DuckDuckGo search. Not exposed in UI. |
| fever-browser | ~200 | Chrome MCP placeholder. |

### Fundamental problems

1. **No actual product experience.** Launching `fever` shows a help menu. The TUI only renders if called programmatically — no user ever reaches it.
2. **Subcommand-first UX.** The CLI is the main entry point. In v2, the TUI IS the main entry point.
3. **Flat, static TUI.** Five equal panes crammed into a grid. No focus management, no modals, no navigation.
4. **No streaming.** The provider layer supports streaming but the TUI renders nothing incrementally.
5. **No provider/model selection.** Users must set env vars manually. No in-app configuration.
6. **No session persistence.** Every launch starts fresh.
7. **No visual identity.** Default colors, no branding, no startup screen, no personality.
8. **Disconnected backend.** All the agent logic (loop, fighting, verification) exists but nothing connects it to the UI.

### What to keep

- **fever-core** — solid abstractions, keep as-is
- **fever-providers** — adapter pattern is good, add streaming consumer
- **fever-agent** — loop driver, roles, verifiers — keep, wire to TUI
- **fever-tools** — keep, wire to TUI
- **fever-config** — extend, don't replace
- **fever-search** — keep
- **Workspace structure** — Cargo workspace with focused crates is correct

### What to replace

- **fever-cli** → becomes a thin binary that launches the TUI (single command: `fever`)
- **fever-tui** → complete rewrite from 790 lines to ~8,000+ lines

---

## 3. Fever v2 Design Principles

1. **TUI-first, always.** `fever` launches the interface. Period. No subcommand needed.
2. **Keyboard-first.** Every action has a keybinding. Mouse is optional augmentation.
3. **Streaming everything.** Responses stream in. Tool calls stream in. Status streams in.
4. **Visual restraint.** Egyptian motifs as texture, not decoration. Cold. Sacred. Luminous.
5. **Progressive disclosure.** Simple by default. Power features one keystroke away.
6. **Zero-config launch.** Works immediately with sensible defaults. Configuration is for power users.
7. **Session-aware.** Remembers workspace, provider, model, conversation history.
8. **Alive.** Subtle animations, responsive feedback, status that breathes.
9. **Production-grade.** Error recovery, graceful degradation, no panics in the UI layer.
10. **Modular.** Theme, provider, layout — all swappable without rewriting core.

---

## 4. Information Architecture

```
Fever v2
├── Shell (home screen)
│   ├── Logo + status
│   ├── Provider/model indicator
│   ├── Workspace summary
│   ├── Recent sessions
│   └── Quick actions
│
├── Chat Mode (primary surface)
│   ├── Message stream (user + assistant)
│   ├── Tool call cards (expandable)
│   ├── Reasoning panels (collapsible)
│   ├── Input bar (with mode indicator)
│   ├── Slash command handler
│   └── Context sidebar (files, git status)
│
├── Code Mode (agent workspace)
│   ├── Diff preview pane
│   ├── File edit list
│   ├── Command approval flow
│   ├── Progress tracker
│   └── Patch accept/reject UI
│
├── Settings (full-screen modal)
│   ├── Provider management
│   ├── Model selection
│   ├── API key configuration
│   ├── Temperature / tokens / system prompt
│   ├── Theme controls
│   ├── Animation intensity
│   ├── Role presets
│   └── Workspace permissions
│
├── Command Palette (overlay)
│   ├── Fuzzy search actions
│   ├── Model switching
│   ├── Role switching
│   ├── Settings access
│   └── Slash commands
│
└── Status Bar (persistent)
    ├── Provider + model
    ├── Token usage
    ├── Session info
    ├── Mode indicator
    └── Keybinding hints
```

---

## 5. Screen-by-Screen UX Spec

### 5.1 Startup / Logo Reveal

**Duration**: 1.2 seconds (skippable with any key)
**Sequence**:
1. Terminal clears to obsidian black
2. Fever glyph appears center-screen (compact mark, not full logo)
3. Cold cyan glow pulse (single pulse, not looping)
4. Glyph fades, app shell materializes

**ASCII concept**:
```
                    
          ◈         
       FEVER        
                    
   ◈ cold sacred code
   
   [detecting providers...]
   [loading workspace...]
```

### 5.2 Home / Shell Screen

**Purpose**: Landing after startup, or when user presses `<Esc>` from chat
**Layout**: Centered content, status bar at bottom

```
╭──────────────────────────────────────────────────────╮
│                                                      │
│              ◈  F E V E R                            │
│              cold sacred code                         │
│                                                      │
│   Provider: openai · gpt-4o                          │
│   Workspace: ~/projects/myapp                        │
│                                                      │
│   ─ ── ────── ── ─ ────── ── ─ ────── ── ─          │
│                                                      │
│   Recent                                             │
│   ├── auth-refactor (3h ago) · 12 messages           │
│   └── api-endpoints (yesterday) · 34 messages        │
│                                                      │
│   [Enter] chat    [/] commands    [S] settings       │
│   [Tab] code mode [P] providers  [R] roles           │
│                                                      │
╰──────────────────────────────────────────────────────╯
│ ◈ openai · gpt-4o │ 0 tokens │ ~/projects/myapp │ ? │
╰──────────────────────────────────────────────────────╯
```

### 5.3 Chat Mode

**Purpose**: Primary conversational interface with the AI
**Layout**: Full-width message area, input bar at bottom, optional sidebar

```
╭─ Chat ──────────────────────────────╮╭─ Context ────────╮
│                                      ││                  │
│  [user]                              ││  ~/myapp         │
│  Refactor the auth module to use     ││  ├── src/        │
│  JWT tokens instead of sessions      ││  │   auth.rs     │
│                                      ││  │   main.rs     │
│  [assistant]                         ││  ├── tests/      │
│  I'll analyze the current auth       ││  └── Cargo.toml  │
│  implementation and plan the JWT     ││                  │
│  migration.                          ││  git: main       │
│                                      ││  status: clean   │
│  ┌─ Tool: read_file ─────────────┐   ││                  │
│  │ ◈ Reading src/auth.rs         │   ││  Recent edits    │
│  │ (234 lines) ✓                 │   ││  (none yet)      │
│  └───────────────────────────────┘   ││                  │
│                                      ││                  │
│  The auth module uses cookie-based   │╰──────────────────╯
│  sessions. Here's my migration plan: │
│                                      │
│  1. Add jsonwebtoken dependency      │
│  2. Create JWT token generation      │
│  3. Replace session middleware       │
│  4. Update auth tests                │
│                                      │
│  ┌─ Tool: edit_file ─────────────┐   │
│  │ ◈ Editing Cargo.toml          │   │
│  │ + jsonwebtoken = "9.3"        │   │
│  │ ✓ applied                     │   │
│  └───────────────────────────────┘   │
│                                      │
╰──────────────────────────────────────╯
│ ◈ openai · gpt-4o │ 1,247 tokens │ ? for help │ > _   │
╰──────────────────────────────────────────────────────────╯
```

**Key interactions**:
- `Enter` sends message
- `Shift+Enter` or `\` for newline
- `/` at start of input triggers slash command mode
- `Tab` toggles context sidebar
- `Ctrl+P` opens command palette
- `Ctrl+S` opens settings
- `[` / `]` cycle through tool call cards
- `e` on a tool card expands/collapses it

### 5.4 Code Mode

**Purpose**: Agent-driven coding workspace with diff preview and approval flow
**Layout**: Left: diff/code view, Right: task progress + actions

```
╭─ Code ─ auth-refactor ──────────────╮╭─ Progress ───────╮
│                                      ││                  │
│  ◈ src/auth.rs                       ││  ✓ Add JWT dep   │
│                                      ││  ● Create token  │
│  - use cookie::Session;              ││    generation     │
│  + use jsonwebtoken::{encode,        ││  ○ Replace        │
│  +   decode, Header, Validation};    ││    middleware     │
│  +                                   ││  ○ Update tests  │
│  - fn validate_session(req: &        ││                  │
│  -   Request) -> Result<User> {      ││  Changed files   │
│  + fn validate_token(req: &          ││  ├── Cargo.toml  │
│  +   Request) -> Result<User> {      ││  ├── auth.rs     │
│  +   let token = extract_bearer      ││  └── auth_test.rs│
│  +     (req)?;                       ││                  │
│  +   let claims = decode(token,      │╰──────────────────╯
│  +     &KEY, &Validation::default    │
│  +   )?.claims;                      │╭─ Actions ────────╮
│                                      ││                  │
│                                      ││  [a] Accept all  │
│                                      ││  [r] Reject all  │
│                                      ││  [n] Next diff   │
│                                      ││  [p] Prev diff   │
│                                      ││  [e] Edit file   │
│                                      │╰──────────────────╯
╰──────────────────────────────────────╯
│ ◈ code mode │ 3/4 tasks │ [a]ccept [r]eject [n]ext │     │
╰──────────────────────────────────────────────────────────╯
```

### 5.5 Settings Screen

**Purpose**: Full-screen modal for all configuration
**Layout**: Tabbed interface with categories

```
╭─ Settings ────────────────────────────────────────────╮
│                                                        │
│  [Providers] [Models] [Behavior] [Theme] [Roles]       │
│  ═══════════                                           │
│                                                        │
│  Configured Providers                                  │
│                                                        │
│  ┌─ OpenAI ──────────────────────────── active ──┐    │
│  │  API Key: sk-••••••••••••••••                   │    │
│  │  Base URL: https://api.openai.com/v1            │    │
│  │  Models: gpt-4o, gpt-4o-mini, o1, o3-mini      │    │
│  │  Status: ● connected                            │    │
│  └─────────────────────────────────────────────────┘    │
│                                                        │
│  ┌─ Anthropic ────────────────────────────────────┐    │
│  │  API Key: (not set)                             │    │
│  │  Status: ○ not configured                       │    │
│  └─────────────────────────────────────────────────┘    │
│                                                        │
│  ┌─ Ollama ───────────────────────────────────────┐    │
│  │  Base URL: http://localhost:11434               │    │
│  │  Status: ● connected (12 models)               │    │
│  └─────────────────────────────────────────────────┘    │
│                                                        │
│  [+ Add Provider]                                      │
│                                                        │
│  [Esc] back    [Enter] save    [?] help                │
╰────────────────────────────────────────────────────────╯
```

### 5.6 Command Palette

**Purpose**: Fuzzy-search overlay for all actions
**Trigger**: `Ctrl+P` or `:` in normal mode

```
╭─ Command Palette ──────────────────────────────────────╮
│                                                        │
│  > switch model                                        │
│                                                        │
│  ◈ Switch to gpt-4o-mini                              │
│  ◈ Switch to claude-sonnet-4-20250514                  │
│  ◈ Switch to ollama:codellama                          │
│  ─ ── ────── ── ─                                      │
│  New chat session                                      │
│  Open settings                                         │
│  Change role to: architect                             │
│  Clear conversation                                    │
│  Export session                                        │
│                                                        │
╰────────────────────────────────────────────────────────╯
```

### 5.7 Provider Setup (First Launch)

**Purpose**: Onboarding flow when no providers are configured
**Layout**: Centered wizard

```
╭──────────────────────────────────────────────────────╮
│                                                      │
│              ◈  F E V E R                            │
│              cold sacred code                         │
│                                                      │
│   Welcome. Let's configure your first provider.      │
│                                                      │
│   ── ─── ── ─── ── ─── ── ─── ──                    │
│                                                      │
│   Choose a provider:                                 │
│                                                      │
│   [1] OpenAI (GPT-4o, o1, o3)                       │
│   [2] Anthropic (Claude)                             │
│   [3] Google (Gemini)                                │
│   [4] Ollama (local models)                          │
│   [5] OpenRouter (multi-provider)                    │
│   [6] Skip — configure later                         │
│                                                      │
│   > 1                                                │
│                                                      │
│   Enter your OpenAI API key:                         │
│   > sk-•••••••••••••••••••••••••••••                  │
│                                                      │
│   [Enter] confirm    [Esc] back                      │
│                                                      │
╰──────────────────────────────────────────────────────╯
```

---

## 6. TUI Component Map

```
fever-tui/
├── app.rs              ← Main application state machine
├── event.rs            ← Input handling, event routing
├── render.rs           ← Top-level frame composition
│
├── screens/
│   ├── mod.rs
│   ├── home.rs         ← Home/shell screen
│   ├── chat.rs         ← Chat mode screen
│   ├── code.rs         ← Code/agent mode screen
│   ├── settings.rs     ← Settings full-screen modal
│   └── onboarding.rs   ← First-launch wizard
│
├── components/
│   ├── mod.rs
│   ├── input_bar.rs    ← Multi-line input with mode indicator
│   ├── message.rs      ← Single message rendering (markdown-aware)
│   ├── tool_card.rs    ← Expandable tool call visualization
│   ├── diff_view.rs    ← Inline diff rendering
│   ├── status_bar.rs   ← Persistent bottom status bar
│   ├── sidebar.rs      ← Context panel (files, git, context)
│   ├── command_palette.rs ← Fuzzy-search overlay
│   ├── provider_card.rs ← Provider status/config card
│   ├── role_picker.rs  ← Role selection dropdown
│   ├── model_picker.rs ← Model selection dropdown
│   ├── progress.rs     ← Task progress tracker
│   └── logo.rs         ← Animated logo renderer
│
├── theme/
│   ├── mod.rs          ← Theme trait + current theme
│   ├── cold_sacred.rs  ← Default Egyptian cold theme
│   └── colors.rs       ← Color palette definitions
│
├── animation/
│   ├── mod.rs          ← Animation scheduler
│   ├── pulse.rs        ← Glow/pulse effects
│   ├── reveal.rs       ← Logo reveal sequence
│   └── transition.rs   ← Screen transition effects
│
├── slash/
│   ├── mod.rs          ← Slash command parser + router
│   ├── commands.rs     ← Command definitions
│   └── help.rs         ← Slash command help text
│
└── util/
    ├── mod.rs
    ├── text.rs         ← Text wrapping, truncation, markdown
    └── glyphs.rs       ← Egyptian glyph constants
```

---

## 7. Visual Design System

### Design tokens

```
Border radius:   none (terminal-native = sharp corners)
Border style:    plain lines (┌─┐│└─┘)
Border emphasis: double lines (╔═╗║╚═╝) only for primary focus
Dividers:        ─ ── ────── ── ─  (Egyptian stepped pattern)
Indicators:      ◈ (primary), ● (active), ○ (inactive), ◆ (accent)
Status:          ✓ (success), ✗ (fail), ● (running), ○ (pending)
Arrows:          → ◄ ► ▸ ▹
Decorative:      𓂀 𓁹 𓃭 (used sparingly, startup only)
```

### Border hierarchy

- **Level 0** (unfocused): `┌─┐│└─┘` dim gray
- **Level 1** (focused): `┌─┐│└─┘` icy cyan
- **Level 2** (modal/overlay): `╔═╗║╚═╝` icy cyan
- **Level 3** (tool card): `┌─┐│└─┘` with ◈ accent at corners

### Spacing system

- Panel padding: 1 char
- Section gap: 1 blank line
- Status bar height: 1 line
- Input bar height: 3 lines (border + input + border)

---

## 8. Color Palette

### "Cold Sacred" (default theme)

```
Name              Hex         Terminal         Usage
─────────────────────────────────────────────────────────
obsidian          #0a0a0f     Black            Background
shadow            #12121a     BrightBlack      Secondary bg
mist              #1a1a2e     #1a1a2e          Panel bg
frost             #e0e0e8     White            Primary text
silver            #a0a0b0     BrightWhite      Secondary text
ash               #606070     BrightBlack      Dimmed text
icy_cyan          #00e5ff     Cyan             Primary accent
sapphire          #4fc3f7     BrightCyan       Secondary accent
deep_blue         #1a237e     Blue             Info accent
moonlight         #c5cae9     BrightBlue       Highlight
aurora_green      #69f0ae     BrightGreen      Success
warm_amber        #ffab40     BrightYellow     Warning
blood_red         #ff1744     BrightRed        Error
sacred_gold       #ffd54f     Yellow           Rare accent (logo only)
```

### Terminal-safe fallback

For terminals without true color, map to 16-color:
- obsidian → Black
- icy_cyan → Cyan (bright)
- frost → White
- aurora_green → Green (bright)
- blood_red → Red (bright)
- warm_amber → Yellow (bright)
- silver → BrightWhite
- ash → BrightBlack

---

## 9. Typography / Glyph / Icon Direction

### Text styles

- **Primary**: Default monospace, frost color
- **Secondary**: Default monospace, silver color
- **Dimmed**: Default monospace, ash color
- **Bold**: `Modifier::BOLD` for titles, user messages, key labels
- **Emphasis**: icy_cyan bold for active/focused elements
- **Code blocks**: shadow background, frost text, 1-char padding

### Glyph system (restrained)

```
Primary mark:     ◈  (U+25C8 WHITE DIAMOND CONTAINING BLACK SMALL DIAMOND)
Active dot:       ●  (U+25CF BLACK CIRCLE)
Inactive dot:     ○  (U+25CB WHITE CIRCLE)
Accent:           ◆  (U+25C6 BLACK DIAMOND)
Arrow right:      →  (U+2192)
Separator:        ─  (U+2500)
Stepped divider:  ─ ── ────── ── ─
Section:          ═══ (U+2550, triple)

Hieroglyph accents (startup/onboarding ONLY):
  Eye of Horus:   𓂀  (U+13080)
  Eye of Ra:      𓁹  (U+13079)
  Ankh:           𓋹  (U+132F9)
```

**Rule**: Hieroglyphs appear ONLY in:
1. Startup logo reveal ( fades after 1.2s)
2. Onboarding screen header
3. Error states (as a subtle "sacred warning" motif)
4. NEVER in everyday chat, settings, or tool cards

---

## 10. Egyptian Motif System

### Design philosophy

The Egyptian visual language is **texture**, not **ornament**. It gives Fever a unique identity without making it look like a museum exhibit or a Halloween costume.

### Where motifs appear

| Element | Motif | Frequency |
|---------|-------|-----------|
| Stepped dividers | `─ ── ────── ── ─` | Home screen, between sections |
| Logo mark | ◈ with optional 𓂀 glyph | Startup, status bar |
| Loading animation | Eye of Horus rotating | During streaming waits |
| Tool card corners | ◈ at ┌ and ┐ | All tool call cards |
| Error borders | Double-line with ◆ | Error states |
| Progress dots | ● ○ ○ ○ pattern | Task progress |
| Settings tabs | ═══ underline | Active tab indicator |
| Onboarding | 𓋹 𓂀 𓁹 as section markers | First-launch only |

### Where motifs DO NOT appear

- Message content
- Code/diff rendering
- Input bar
- Status text
- Provider names or model names
- Help text

---

## 11. Logo Transformation Plan

### Current state

Fever has a text name "Fever Code" / "FeverCode" with no distinct visual mark.

### v2 Logo System

#### 1. Primary Mark: ◈ (The Cold Diamond)

```
    ╱╲
   ╱  ╲
  ╱ ◈  ╲
 ╱      ╲
╲      ╱
 ╲    ╱
  ╲  ╱
   ╲╱
```

Simplified for terminal: just `◈` — the white diamond containing a black diamond. Represents:
- Precision (diamond cut)
- Code (the inner structure)
- Cold (clarity, ice)

#### 2. Text Lockup

```
◈ FEVER
  cold sacred code
```

Rules:
- "FEVER" is always caps, always spaced letters in logo form
- "cold sacred code" is always lowercase, always subtitle
- Mark always precedes text
- Minimum spacing: 1 space between mark and text

#### 3. Startup Animation Sequence

```
Frame 1 (0.0s):   blank screen
Frame 2 (0.2s):       ◈
Frame 3 (0.5s):       ◈
                    FEVER
Frame 4 (0.8s):       ◈
                    FEVER
                 cold sacred code
Frame 5 (1.2s):  (fade to app shell)
```

#### 4. Compact Mark (status bar, tabs)

Just `◈` — single character, icy_cyan color.

#### 5. SVG Source Approach

Create an SVG with:
- Diamond shape (outer)
- Smaller diamond (inner)
- Icy cyan fill (#00e5ff)
- Export to PNG at various sizes for non-terminal use (website, social)

---

## 12. Motion / Animation Plan

### Animation system

A tick-based animation scheduler that runs on the existing 250ms tick loop. Animations are state machines with `start()`, `tick()`, `is_complete()` methods.

### Implemented animations

| Animation | Duration | Where | Purpose |
|-----------|----------|-------|---------|
| Logo reveal | 1.2s | Startup | Brand identity |
| Glow pulse | 0.5s | Focus indicators | Active state |
| Stream cursor | continuous | Chat | Typing indicator |
| Tool card expand | 0.25s | Chat | Tool call reveal |
| Status bar breathing | 4s cycle | Status bar | "Alive" feeling |
| Screen transition | 0.15s | Navigation | Mode switching |

### Animation intensity levels

```
0: None — all animations disabled (accessibility / slow terminals)
1: Minimal — stream cursor only
2: Normal — all above (default)
3: Extra — aurora gradient in status bar, particle effects in logo
```

### Graceful degradation

- Check `$TERM` for xterm-256color or truecolor support
- If 16-color only: disable gradients, use flat colors
- If no alternate screen: fall back to line-by-line output
- Animations that don't render correctly are automatically disabled

---

## 13. Rust Architecture Proposal

### Architecture: Elm-style (State → Message → Update → Render)

```
┌─────────────────────────────────────────┐
│                  App                     │
│                                         │
│  State ──────→ Update ──────→ Render    │
│    ▲                            │       │
│    │                            │       │
│    └──── Message ←─────────────┘       │
│                                         │
│  Events:                                │
│  - KeyPress(KeyEvent)                   │
│  - MouseClick(MouseEvent)               │
│  - Tick(Duration)                       │
│  - StreamChunk(String)                  │
│  - ToolCall(ToolEvent)                  │
│  - ProviderResponse(ChatResponse)       │
│  - ConfigChanged                        │
└─────────────────────────────────────────┘
```

### State machine

```rust
enum Screen {
    Home,
    Chat,
    Code,
    Settings(SettingsTab),
    Onboarding(OnboardingStep),
    CommandPalette,
}

struct AppState {
    screen: Screen,
    provider_state: ProviderState,    // active provider, model, health
    session: Session,                 // messages, tool calls, context
    workspace: WorkspaceInfo,         // git status, file tree
    config: FeverConfig,              // all settings
    theme: Theme,                     // current theme
    animations: AnimationState,       // active animations
    input: InputState,                // current input buffer, mode
    status: StatusBarState,           // what to show in status bar
}
```

### Message enum

```rust
enum Message {
    Key(KeyEvent),
    Mouse(MouseEvent),
    Tick,
    StreamChunk { content: String },
    StreamEnd,
    ToolCallStarted { tool: String, args: String },
    ToolCallCompleted { tool: String, result: String },
    ToolCallFailed { tool: String, error: String },
    ProviderConnected { name: String },
    ProviderError { name: String, error: String },
    SessionLoaded { session: Session },
    ConfigSaved,
    Navigate(Screen),
    InputChanged(String),
    InputSubmitted,
    SlashCommand(String),
}
```

### Update function

```rust
fn update(state: &mut AppState, msg: Message) -> Vec<Command> {
    match msg {
        Message::Key(key) => handle_key(state, key),
        Message::StreamChunk { content } => {
            state.session.append_streaming(content);
            vec![]
        }
        Message::ToolCallCompleted { tool, result } => {
            state.session.complete_tool_call(tool, result);
            vec![]
        }
        // ...
    }
}
```

### Async commands

The update function returns `Vec<Command>` — side effects that run asynchronously:

```rust
enum Command {
    SendMessage { provider: String, model: String, messages: Vec<ChatMessage> },
    ExecuteTool { tool: String, args: Value },
    SaveConfig,
    LoadSession { id: String },
    DetectProviders,
    None,
}
```

These run on a tokio task and emit Messages back into the event loop.

---

## 14. Recommended Crates / Libraries

### Core stack

| Crate | Version | Purpose |
|-------|---------|---------|
| `ratatui` | 0.28 | TUI framework (already in use) |
| `crossterm` | 0.28 | Terminal backend (already in use) |
| `tokio` | 1.42 | Async runtime (already in use) |
| `serde` / `serde_json` | 1.0 / 1.0 | Serialization (already in use) |
| `anyhow` / `thiserror` | 1.0 / 2.0 | Error handling (already in use) |
| `clap` | 4.5 | CLI argument parsing (minimal — just `fever` and flags) |

### New additions

| Crate | Purpose | Why |
|-------|---------|-----|
| `tui-textarea` | Multi-line text input | Battle-tested ratatui textarea with undo/redo, selection, scrolling |
| `ratatui-widgets` | Additional widgets | Throbber, calendar, etc. |
| `fuzzy-matcher` | Fuzzy search | For command palette and model/provider search |
| `syntect` | Syntax highlighting | For diff view and code rendering |
| `pulldown-cmark` | Markdown rendering | Parse markdown in assistant responses |
| `unicode-segmentation` | Unicode text handling | Proper grapheme-based text editing |
| `chrono` | Timestamps | Already in workspace |
| `dirs` | XDG directories | For config/data/session storage paths |
| `uuid` | Session/message IDs | Already in workspace |

### NOT recommended

| Crate | Why not |
|-------|---------|
| `cursive` | Different paradigm, not ratatui-compatible |
| `tuirealm` | Adds complexity without enough benefit for our component count |
| `termion` | Crossterm is already our backend and more feature-rich |
| `makepad` | GUI framework, not terminal |

---

## 15. Project Folder Structure

```
FeverCode/
├── Cargo.toml                    # workspace root
├── rust-toolchain.toml           # Rust 1.85
├── install.sh                    # installer
├── uninstall.sh                  # uninstaller
├── README.md
├── ARCHITECTURE.md
├── .github/workflows/ci.yml
│
├── crates/
│   ├── fever-cli/                # Binary entry point (thin)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       └── main.rs           # ~50 lines: parse --help/--version, launch TUI
│   │
│   ├── fever-tui/                # THE PRODUCT — full rewrite
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── app.rs            # AppState, Screen enum, update()
│   │       ├── event.rs          # Message enum, event loop
│   │       ├── render.rs         # Top-level render dispatch
│   │       │
│   │       ├── screens/
│   │       │   ├── mod.rs
│   │       │   ├── home.rs       # Home/shell screen
│   │       │   ├── chat.rs       # Chat mode
│   │       │   ├── code.rs       # Code/agent mode
│   │       │   ├── settings.rs   # Settings modal
│   │       │   └── onboarding.rs # First-launch wizard
│   │       │
│   │       ├── components/
│   │       │   ├── mod.rs
│   │       │   ├── input_bar.rs
│   │       │   ├── message.rs
│   │       │   ├── tool_card.rs
│   │       │   ├── diff_view.rs
│   │       │   ├── status_bar.rs
│   │       │   ├── sidebar.rs
│   │       │   ├── command_palette.rs
│   │       │   ├── provider_card.rs
│   │       │   ├── role_picker.rs
│   │       │   ├── model_picker.rs
│   │       │   ├── progress.rs
│   │       │   └── logo.rs
│   │       │
│   │       ├── theme/
│   │       │   ├── mod.rs
│   │       │   ├── cold_sacred.rs
│   │       │   └── colors.rs
│   │       │
│   │       ├── animation/
│   │       │   ├── mod.rs
│   │       │   ├── pulse.rs
│   │       │   ├── reveal.rs
│   │       │   └── transition.rs
│   │       │
│   │       ├── slash/
│   │       │   ├── mod.rs
│   │       │   ├── commands.rs
│   │       │   └── help.rs
│   │       │
│   │       └── util/
│   │           ├── mod.rs
│   │           ├── text.rs
│   │           └── glyphs.rs
│   │
│   ├── fever-core/               # KEEP — abstractions (extend, don't break)
│   ├── fever-providers/          # KEEP — provider adapters (add streaming consumer)
│   ├── fever-agent/              # KEEP — agent logic (wire to TUI)
│   ├── fever-tools/              # KEEP — tool implementations
│   ├── fever-config/             # KEEP — extend with new fields
│   ├── fever-search/             # KEEP
│   ├── fever-browser/            # KEEP
│   └── fever-release/            # KEEP
```

---

## 16. Config / Provider Model Design

### Config file: `~/.config/fevercode/config.toml`

```toml
[appearance]
theme = "cold_sacred"
animation_intensity = 2        # 0=none, 1=minimal, 2=normal, 3=extra

[defaults]
provider = "openai"
model = "gpt-4o"
role = "coder"
temperature = 0.7
max_tokens = 4096

[providers.openai]
enabled = true
api_key = "sk-..."             # or env:OPENAI_API_KEY
base_url = "https://api.openai.com/v1"
models = ["gpt-4o", "gpt-4o-mini", "o1", "o3-mini"]

[providers.anthropic]
enabled = false
api_key = ""                   # or env:ANTHROPIC_API_KEY

[providers.ollama]
enabled = true
base_url = "http://localhost:11434"

[providers.openrouter]
enabled = false
api_key = ""

[session]
save_on_exit = true
max_history = 100
workspace_aware = true

[agent]
auto_approve_safe_tools = true  # read, search, list
require_approval_for = ["shell", "write", "delete"]
streaming = true
show_reasoning = false
```

### Session storage: `~/.local/share/fevercode/sessions/`

```
sessions/
├── 2026-03-31_auth-refactor.json
├── 2026-03-30_api-endpoints.json
└── 2026-03-29_debug-imports.json
```

### Workspace config: `.fever/config.toml` (per-project)

```toml
[workspace]
context_files = ["src/**/*.rs", "Cargo.toml", "README.md"]
ignore_paths = ["target/", ".git/"]

[agent]
role = "coder"
system_prompt = "You are working on a Rust TUI project..."
auto_approve = ["read", "search"]
```

---

## 17. MVP Implementation Roadmap

### MVP scope (what "done" looks like for v2 alpha)

1. **Launches into TUI** — `fever` opens full-screen interface
2. **Startup animation** — Logo reveal, provider detection
3. **Onboarding** — First-launch provider setup wizard
4. **Home screen** — Shows provider, workspace, recent sessions, quick actions
5. **Chat mode** — Send messages, receive streaming responses, basic tool cards
6. **Status bar** — Provider, model, token count, keybinding hints
7. **Settings screen** — Provider CRUD, model selection, theme picker
8. **Session persistence** — Save/load conversations
9. **Slash commands** — `/model`, `/role`, `/clear`, `/help`, `/settings`
10. **Theming** — "Cold Sacred" theme with color fallback

### NOT in MVP (v2.1+)

- Code mode (diff preview, patch approval)
- Command palette (fuzzy search overlay)
- Context sidebar (file tree, git status)
- Role management screen
- Animation intensity controls
- Advanced keybinding customization
- Export sessions
- Multi-message undo

---

## 18. v2 Phased Rollout Plan

### Phase 1: Foundation (Week 1-2)

Replace fever-cli with TUI launcher. Rebuild fever-tui architecture.

- New AppState / Message / Command system
- Screen enum and navigation
- Event loop with async command dispatch
- Theme system
- Status bar component
- Input bar component

### Phase 2: Core Screens (Week 3-4)

- Home screen with provider detection
- Chat screen with streaming
- Settings screen (providers, models, theme)
- Onboarding wizard
- Session save/load

### Phase 3: Polish (Week 5-6)

- Startup animation
- Tool call visualization
- Slash commands
- Glyph system
- Markdown rendering in messages
- Color fallback for 16-color terminals

### Phase 4: Power Features (Week 7-8)

- Code mode with diff preview
- Command palette with fuzzy search
- Context sidebar
- Role management
- Animation intensity controls

---

## 19. Sample Terminal Mockups in ASCII

### Startup (Frame 3 of 5)

```
                    
                    
                    
                    
                        ◈
                      FEVER
                    
                    
                    
                    
```

### Home Screen

```
╭─────────────────────────────────────────────────────────╮
│                                                         │
│               ◈  F E V E R                              │
│               cold sacred code                           │
│                                                         │
│   Provider: openai · gpt-4o                             │
│   Workspace: ~/projects/myapp                           │
│                                                         │
│   ─ ── ────── ── ─ ────── ── ─                          │
│                                                         │
│   Recent                                                │
│   ├── auth-refactor (3h ago) · 12 messages              │
│   └── api-endpoints (yesterday) · 34 messages           │
│                                                         │
│   [Enter] chat    [/] commands    [S] ettings           │
│   [Tab] code      [P] providers   [R] oles              │
│                                                         │
╰─────────────────────────────────────────────────────────╯
│ ◈ openai · gpt-4o │ 0 tok │ ~/projects/myapp │ ? help  │
╰─────────────────────────────────────────────────────────╯
```

### Chat with Streaming

```
╭─ Chat ──────────────────────────────────────────────────╮
│                                                         │
│  [user]                                                 │
│  Refactor auth.rs to use JWT                            │
│                                                         │
│  [assistant]                                            │
│  I'll analyze the current auth module and plan the      │
│  migration to JWT tokens. Let me start by reading the   │
│  current implementation.                                │
│                                                         │
│  ┌─ ◈ Tool: read_file ─────────────────────────────┐   │
│  │  Reading: src/auth.rs                            │   │
│  │  234 lines · ✓ complete                          │   │
│  └──────────────────────────────────────────────────┘   │
│                                                         │
│  The auth module uses cookie-based sessions stored in   │
│  a HashMap. Here's my migration plan:                   │
│                                                         │
│  1. Add `jsonwebtoken` dependency                       │
│  2. Create `TokenClaims` struct                         │
│  3. Replace `validate_session` with `validate_token`    │
│  4. Update auth middleware                              │
│  5. Add token refresh endpoint                          │
│                                                         │
│  Shall I proceed with these changes?                    │
│                                                         │
│  ...▌                                                   │
│                                                         │
╰─────────────────────────────────────────────────────────╯
│ > Yes, start with steps 1-3.                            │
╰─────────────────────────────────────────────────────────╯
│ ◈ openai · gpt-4o │ 1,847 tok │ streaming... │ ? help  │
╰─────────────────────────────────────────────────────────╯
```

### Settings — Provider Tab

```
╭─ Settings ═══════════════════════════════════════════════╮
│                                                         │
│  [Providers]  Models  Behavior  Theme  Roles            │
│  ═══════════                                            │
│                                                         │
│  ┌─ OpenAI ─────────────────────────── ● active ───┐   │
│  │  API Key: sk-••••••••••••••••                    │   │
│  │  Base URL: https://api.openai.com/v1             │   │
│  │  Health: ● connected (4 models available)        │   │
│  │  Active model: gpt-4o                            │   │
│  └──────────────────────────────────────────────────┘   │
│                                                         │
│  ┌─ Ollama ────────────────────────────────────────┐   │
│  │  Base URL: http://localhost:11434                │   │
│  │  Health: ● connected (12 models available)       │   │
│  └──────────────────────────────────────────────────┘   │
│                                                         │
│  ┌─ Anthropic ─────────────────────────────────────┐   │
│  │  API Key: (not configured)                       │   │
│  │  Health: ○ not configured                        │   │
│  └──────────────────────────────────────────────────┘   │
│                                                         │
│  [+ Add Provider]                                       │
│                                                         │
╰─────────────────────────────────────────────────────────╯
│ [Esc] back    [Enter] save    [Tab] next section        │
╰─────────────────────────────────────────────────────────╯
```

---

## 20. Exact Files to Create / Change

### Files to CREATE in fever-tui

| File | Purpose | Estimated Lines |
|------|---------|-----------------|
| `src/app.rs` | AppState, Screen enum, update() | 400 |
| `src/event.rs` | Message enum, event loop | 200 |
| `src/render.rs` | Top-level render dispatch | 150 |
| `src/screens/mod.rs` | Screen trait | 30 |
| `src/screens/home.rs` | Home/shell screen | 250 |
| `src/screens/chat.rs` | Chat mode | 350 |
| `src/screens/code.rs` | Code/agent mode | 300 |
| `src/screens/settings.rs` | Settings modal | 400 |
| `src/screens/onboarding.rs` | First-launch wizard | 250 |
| `src/components/mod.rs` | Component trait | 20 |
| `src/components/input_bar.rs` | Multi-line input | 200 |
| `src/components/message.rs` | Message rendering | 150 |
| `src/components/tool_card.rs` | Tool call card | 120 |
| `src/components/diff_view.rs` | Diff rendering | 200 |
| `src/components/status_bar.rs` | Status bar | 100 |
| `src/components/sidebar.rs` | Context panel | 150 |
| `src/components/command_palette.rs` | Fuzzy search | 200 |
| `src/components/provider_card.rs` | Provider card | 100 |
| `src/components/role_picker.rs` | Role selector | 80 |
| `src/components/model_picker.rs` | Model selector | 80 |
| `src/components/progress.rs` | Task progress | 80 |
| `src/components/logo.rs` | Animated logo | 100 |
| `src/theme/mod.rs` | Theme trait + loader | 80 |
| `src/theme/cold_sacred.rs` | Default theme | 120 |
| `src/theme/colors.rs` | Color definitions | 60 |
| `src/animation/mod.rs` | Animation scheduler | 100 |
| `src/animation/pulse.rs` | Glow effects | 50 |
| `src/animation/reveal.rs` | Logo reveal | 80 |
| `src/animation/transition.rs` | Screen transitions | 60 |
| `src/slash/mod.rs` | Slash command router | 80 |
| `src/slash/commands.rs` | Command definitions | 120 |
| `src/slash/help.rs` | Help text | 60 |
| `src/util/mod.rs` | Utilities | 10 |
| `src/util/text.rs` | Text helpers | 80 |
| `src/util/glyphs.rs` | Glyph constants | 40 |
| **Total new TUI code** | | **~5,100** |

### Files to REPLACE

| File | Action |
|------|--------|
| `fever-cli/src/main.rs` | Rewrite: ~50 lines, just launches TUI |
| `fever-tui/src/lib.rs` | Rewrite: export new modules |
| `fever-tui/src/app.rs` | Replace with Elm architecture |
| `fever-tui/src/ui.rs` | Delete (replaced by screens/) |
| `fever-tui/src/widgets.rs` | Delete (replaced by components/) |

### Files to MODIFY (extend, don't break)

| File | Change |
|------|--------|
| `fever-tui/Cargo.toml` | Add tui-textarea, fuzzy-matcher, pulldown-cmark, dirs, syntect |
| `fever-config/src/config.rs` | Add appearance, session, agent fields |
| `fever-config/src/provider.rs` | Add health check, model list caching |
| `fever-providers/src/client.rs` | Add streaming consumer callback |
| `fever-core/src/lib.rs` | No changes needed |
| `Cargo.toml` (workspace) | Add new workspace deps |

### New files outside fever-tui

| File | Purpose |
|------|---------|
| `fever-tui/Cargo.toml` | Updated deps |
| `docs/FEVER_V2_DESIGN.md` | This document |

---

## 21. Final Prioritized Build Plan

### Sprint 1: "Hello Fever" (TUI launches, shows home screen)

**Files** (in order):
1. `fever-tui/src/theme/colors.rs` — Color palette
2. `fever-tui/src/theme/cold_sacred.rs` — Theme definition
3. `fever-tui/src/theme/mod.rs` — Theme trait
4. `fever-tui/src/util/glyphs.rs` — Glyph constants
5. `fever-tui/src/util/text.rs` — Text helpers
6. `fever-tui/src/util/mod.rs` — Re-exports
7. `fever-tui/src/components/status_bar.rs` — Status bar
8. `fever-tui/src/components/logo.rs` — Static logo (no animation yet)
9. `fever-tui/src/screens/home.rs` — Home screen
10. `fever-tui/src/app.rs` — AppState + update + basic event loop
11. `fever-tui/src/event.rs` — Message enum
12. `fever-tui/src/render.rs` — Render dispatch
13. `fever-tui/src/lib.rs` — New module exports
14. `fever-cli/src/main.rs` — Thin launcher
15. `fever-tui/Cargo.toml` — Updated deps

**Deliverable**: `fever` opens full-screen, shows home screen with logo and provider status.

### Sprint 2: "Talk to Me" (Chat with streaming)

**Files**:
1. `fever-tui/src/components/input_bar.rs`
2. `fever-tui/src/components/message.rs`
3. `fever-tui/src/components/tool_card.rs`
4. `fever-tui/src/screens/chat.rs`
5. `fever-tui/src/animation/mod.rs` + `pulse.rs`
6. Wire provider streaming to event loop

**Deliverable**: User can type messages, see streaming responses, see tool call cards.

### Sprint 3: "Make It Yours" (Settings + onboarding)

**Files**:
1. `fever-tui/src/components/provider_card.rs`
2. `fever-tui/src/components/model_picker.rs`
3. `fever-tui/src/screens/settings.rs`
4. `fever-tui/src/screens/onboarding.rs`
5. `fever-config/src/config.rs` — Extended fields
6. Session save/load

**Deliverable**: First-launch wizard, provider CRUD, model selection, session persistence.

### Sprint 4: "Feel Alive" (Animations + slash commands + polish)

**Files**:
1. `fever-tui/src/animation/reveal.rs`
2. `fever-tui/src/animation/transition.rs`
3. `fever-tui/src/slash/commands.rs`
4. `fever-tui/src/slash/help.rs`
5. `fever-tui/src/components/command_palette.rs` (MVP)
6. Color fallback system
7. Keybinding documentation

**Deliverable**: Startup animation, slash commands, command palette, graceful terminal degradation.

---

## Final Recommended v2 Feature Set

### Must-have (MVP)
- Full-screen TUI with branded startup
- Home screen with provider/workspace/session info
- Chat mode with streaming responses
- Tool call visualization (expandable cards)
- Status bar (provider, model, tokens, keybindings)
- Settings screen (provider CRUD, model selection)
- First-launch onboarding wizard
- Session save/load
- Slash commands (`/model`, `/role`, `/clear`, `/help`, `/settings`)
- "Cold Sacred" theme with 16-color fallback
- Keyboard-first (every action has a keybinding)

### Nice-to-have (v2.1)
- Code mode with diff preview
- Command palette with fuzzy search
- Context sidebar (file tree, git status)
- Role management screen
- Animation intensity slider
- Markdown rendering in messages
- Syntax-highlighted code blocks
- Export sessions to markdown

### Future (v2.2+)
- Plugin system
- Multi-panel layouts (user-configurable)
- Custom themes (user-editable TOML)
- Collaborative sessions
- MCP integration
- Voice input

---

## Ideal Crate Stack

```toml
# fever-tui/Cargo.toml additions
[dependencies]
# Core TUI
ratatui = { workspace = true }
crossterm = { workspace = true }
tui-textarea = "0.7"         # Multi-line input

# Async
tokio = { workspace = true }

# Serialization
serde = { workspace = true }
serde_json = { workspace = true }
toml = { workspace = true }

# Text rendering
pulldown-cmark = "0.12"      # Markdown → ratatui Spans
syntect = "5.2"              # Syntax highlighting
unicode-segmentation = "1.12" # Grapheme-aware text editing

# Search
fuzzy-matcher = "0.3"        # Command palette fuzzy search

# Utility
chrono = { workspace = true }
dirs = "6.0"                 # XDG paths
uuid = { workspace = true }
anyhow = { workspace = true }
thiserror = { workspace = true }
tracing = { workspace = true }
```

---

## Concrete MVP Scope

**One sentence**: Launch `fever`, see a branded home screen, configure your first provider in-app, start chatting with streaming responses and tool visualization, save your session.

**Concrete checklist**:
- [ ] `fever` opens alternate-screen TUI
- [ ] Startup logo (static, 1 frame — animation is polish)
- [ ] Home screen: provider status, workspace, quick actions
- [ ] Onboarding: if no provider configured, walk through setup
- [ ] Chat: type messages, see streaming responses
- [ ] Tool calls: rendered as cards (name, args, status)
- [ ] Settings: list providers, select model, set API keys
- [ ] Status bar: always visible, shows context
- [ ] Session: auto-save on exit, resume on relaunch
- [ ] Slash commands: `/help`, `/model`, `/clear`, `/settings`
- [ ] Theme: Cold Sacred with 16-color fallback
- [ ] No panics, no broken rendering on common terminals

---

## First Implementation Sprint — Files and Tasks

| # | File | Task | Hours |
|---|------|------|-------|
| 1 | `fever-tui/src/theme/colors.rs` | Define Cold Sacred palette as constants | 0.5 |
| 2 | `fever-tui/src/theme/cold_sacred.rs` | Theme struct implementing trait | 0.5 |
| 3 | `fever-tui/src/theme/mod.rs` | Theme trait, loader, fallback | 0.5 |
| 4 | `fever-tui/src/util/glyphs.rs` | Glyph constants (◈, ●, ○, dividers) | 0.5 |
| 5 | `fever-tui/src/util/text.rs` | Wrap, truncate, grapheme helpers | 1 |
| 6 | `fever-tui/src/util/mod.rs` | Re-exports | 0.1 |
| 7 | `fever-tui/src/components/status_bar.rs` | Render status bar | 1 |
| 8 | `fever-tui/src/components/logo.rs` | Static logo render | 0.5 |
| 9 | `fever-tui/src/components/input_bar.rs` | Text input with border | 2 |
| 10 | `fever-tui/src/event.rs` | Message enum, event loop skeleton | 2 |
| 11 | `fever-tui/src/screens/home.rs` | Home screen layout + render | 2 |
| 12 | `fever-tui/src/app.rs` | AppState, Screen, update(), run() | 3 |
| 13 | `fever-tui/src/render.rs` | Frame dispatch | 0.5 |
| 14 | `fever-tui/src/lib.rs` | Module exports | 0.1 |
| 15 | `fever-cli/src/main.rs` | Thin TUI launcher | 0.5 |
| 16 | `fever-tui/Cargo.toml` | Add new deps | 0.2 |
| 17 | `Cargo.toml` (workspace) | Add workspace deps | 0.2 |
| | | **Total** | **~15h** |

---

> **Fever v2**: Code like fever, ship like dream.
> ◈ cold sacred code
