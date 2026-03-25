# Architecture

This document provides a detailed technical overview of Fever Code's architecture, design decisions, and implementation details.

## Table of Contents

- [Overview](#overview)
- [Design Principles](#design-principles)
- [System Architecture](#system-architecture)
- [Crate Organization](#crate-organization)
- [Core Components](#core-components)
- [Provider System](#provider-system)
- [Agent Model](#agent-model)
- [Tooling System](#tooling-system)
- [Search System](#search-system)
- [Browser Integration](#browser-integration)
- [TUI Design](#tui-design)
- [Configuration](#configuration)
- [Data Flow](#data-flow)
- [Performance Considerations](#performance-considerations)
- [Security Considerations](#security-considerations)
- [Extensibility](#extensibility)

## Overview

Fever Code is a terminal-first AI coding platform built with Rust. It provides a single visible agent with 50+ internal specialist roles, supports 30+ LLM providers, and includes Chrome MCP integration and free web search.

### Key Design Goals

1. **Terminal-first**: Optimized for Linux terminal usage
2. **Single visible agent**: One agent, multiple internal role modes
3. **Modular architecture**: Clean separation of concerns
4. **Provider-agnostic**: Unified interface to multiple LLM providers
5. **Extensible tools**: Easy to add new tools
6. **Free search**: No API key required for basic search
7. **Chrome MCP**: First-class browser debugging support
8. **Polished TUI**: Professional terminal UI

## Design Principles

### 1. Simplicity Over Complexity

Prefer simple, well-tested solutions over complex abstractions. The single agent model reduces cognitive load compared to multi-agent swarms.

### 2. Composability

Small, focused components that can be composed together. Each crate has a single responsibility.

### 3. Performance

Use Rust's type system for compile-time guarantees, async/await for concurrency, and efficient data structures.

### 4. Portability

Single-binary distribution, minimal runtime dependencies, broad Linux support.

### 5. Extensibility

Plugin-like architecture for providers, tools, and roles. Easy to add new components without modifying core code.

## System Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                         fever-cli                              │
│                      (Entry Points)                             │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │   fever | fever code | fever-code                        │  │
│  └──────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                         fever-tui                               │
│                      (User Interface)                            │
│  ┌──────────┬──────────┬──────────┬──────────┬──────────────┐  │
│  │  Chat    │   Plan   │  Tasks   │Tool Log  │   Browser    │  │
│  └──────────┴──────────┴──────────┴──────────┴──────────────┘  │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                        fever-agent                              │
│                    (Single Visible Agent)                       │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │  FeverAgent with 50+ Specialist Roles                   │  │
│  │  - Researcher, Planner, Architect, Coder, Tester...     │  │
│  └──────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
                              │
              ┌───────────────┼───────────────┐
              ▼               ▼               ▼
┌──────────────────┐ ┌─────────────────┐ ┌──────────────────────┐
│ fever-providers │ │  fever-tools    │ │   fever-browser      │
│  (LLM APIs)     │ │ (Shell, File...) │ │  (Chrome MCP)        │
└──────────────────┘ └─────────────────┘ └──────────────────────┘
              │               │                    │
              └───────────────┼────────────────────┘
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                        fever-core                               │
│                   (Orchestration Engine)                        │
│  ┌────────┬────────┬────────┬────────┬────────┬─────────────┐ │
│  │  Task  │  Event │ Memory │  Retry │   Tool │ Execution   │ │
│  │  Graph │   Bus  │  Store │ Policy │ Router │  Engine     │ │
│  └────────┴────────┴────────┴────────┴────────┴─────────────┘ │
└─────────────────────────────────────────────────────────────────┘
                              │
              ┌───────────────┼───────────────┐
              ▼               ▼               ▼
┌──────────────────┐ ┌─────────────────┐ ┌──────────────────────┐
│  fever-config   │ │  fever-search   │ │   fever-release      │
│  (Config Mgmt)  │ │  (DuckDuckGo)   │ │  (Release/Build)     │
└──────────────────┘ └─────────────────┘ └──────────────────────┘
```

## Crate Organization

### fever-core

**Purpose**: Foundational orchestration engine

**Key Components**:
- `task`: Task graph, plan management, todo lists
- `event`: Event bus for pub/sub communication
- `execution`: Task execution engine with retry logic
- `memory`: In-memory context storage with SQLite backing
- `retry`: Exponential backoff, linear retry policies
- `tool`: Tool registry and execution dispatcher
- `agent`: Agent trait and messaging model

**Dependencies**: tokio, serde, rusqlite

### fever-agent

**Purpose**: Single visible agent with internal specialist roles

**Key Components**:
- `agent`: FeverAgent implementation of Agent trait
- `role`: RoleRegistry with 50+ SpecialistRole definitions

**Dependencies**: fever-core, fever-providers, fever-config

### fever-providers

**Purpose**: LLM provider abstraction layer

**Key Components**:
- `client`: ProviderClient with unified chat interface
- `adapter`: ProviderAdapter trait for implementing providers
- `models`: Request/response types (ChatRequest, ChatResponse)
- `error`: ProviderError types

**Supported Providers** (30+):
- OpenAI, Anthropic, Google Gemini, Ollama, Together AI
- Groq, Fireworks, Mistral, Cohere, xAI, DeepSeek, Perplexity
- Azure OpenAI, AWS Bedrock, Google Vertex AI
- Hugging Face, Cloudflare Workers, SambaNova, Cerebras
- Replicate, Nebius, Baseten, Novita, AI21, MiniMax, OpenRouter
- Moonshot/Kimi, Alibaba DashScope, Tencent Hunyuan
- Generic OpenAI-compatible

**Dependencies**: fever-core, reqwest

### fever-tools

**Purpose**: System interaction tools

**Key Components**:
- `shell`: Shell command execution
- `filesystem`: File read/write/list operations
- `grep`: Pattern searching with regex
- `git`: Git operations (status, log, commit, branch)

**Dependencies**: fever-core, walkdir, ignore, git2

### fever-search

**Purpose**: No-paid-API search capability

**Key Components**:
- `client`: SearchClient with caching
- `parser`: DuckDuckGo HTML parser, SearXNG parser
- `cache`: SQLite-based search result cache
- `result`: SearchResult types

**Dependencies**: fever-core, reqwest, scraper

### fever-browser

**Purpose**: Chrome MCP integration

**Key Components**:
- `client`: Browser client for Chrome MCP communication
- `tool`: BrowserTool for TUI integration
- `error`: BrowserError types

**Dependencies**: fever-core, tokio-tungstenite

### fever-tui

**Purpose**: Terminal user interface

**Key Components**:
- `app`: FeverTui main application loop
- `ui`: FeverUI with pane management
- `widgets`: Individual pane widgets (Chat, Plan, Tasks, ToolLog, Browser)

**Dependencies**: fever-core, fever-agent, ratatui, crossterm

### fever-config

**Purpose**: Configuration management

**Key Components**:
- `config`: Config struct, ConfigManager
- `provider`: ProviderCredentials types

**Dependencies**: fever-core, directories

### fever-cli

**Purpose**: Command-line interface entry points

**Key Components**:
- `main.rs`: CLI argument parsing and command routing

**Dependencies**: All other crates, clap

### fever-release

**Purpose**: Release and build management

**Key Components**:
- `lib.rs`: Release note generation

**Dependencies**: anyhow

## Core Components

### Task Graph and Execution Engine

The task graph manages dependencies and execution order:

```rust
pub struct Task {
    pub id: String,
    pub title: String,
    pub description: String,
    pub status: TaskStatus,
    pub dependencies: Vec<String>,
    // ...
}

pub struct Plan {
    pub id: String,
    pub title: String,
    pub tasks: Vec<Task>,
    // ...
}
```

Tasks can only start when all dependencies are completed.

### Event Bus

Pub/sub system for loose coupling:

```rust
pub enum Event {
    TaskQueued { task_id: String, title: String },
    TaskStarted { task_id: String, title: String },
    TaskCompleted { task_id: String, title: String },
    TaskFailed { task_id: String, title: String, error: String },
    ToolCalled { tool_name: String, args: serde_json::Value },
    ToolResult { tool_name: String, result: serde_json::Value },
    // ...
}
```

### Tool Registry

Dynamic tool registration and execution:

```rust
pub trait Tool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    async fn execute(&self, args: Value, context: &ExecutionContext) -> Result<Value>;
    fn schema(&self) -> ToolSchema;
}
```

## Provider System

### Provider Abstraction

Unified interface for all LLM providers:

```rust
#[async_trait]
pub trait ProviderAdapter: Send + Sync {
    fn name(&self) -> &str;
    fn capabilities(&self) -> &ProviderCapabilities;
    async fn chat(&self, request: &ChatRequest) -> ProviderResult<ChatResponse>;
    async fn chat_stream(&self, request: &ChatRequest)
        -> ProviderResult<Box<dyn Stream<Item = ProviderResult<StreamChunk>> + Send + Unpin>>;
    fn list_models(&self) -> Vec<String>;
    fn get_model_info(&self, model_id: &str) -> Option<ModelInfo>;
    fn is_configured(&self) -> bool;
}
```

### Provider Capabilities

Each provider declares capabilities:

```rust
pub struct ProviderCapabilities {
    pub supports_chat: bool,
    pub supports_tools: bool,
    pub supports_streaming: bool,
    pub supports_images: bool,
    pub supports_function_calling: bool,
    pub max_context_length: Option<u32>,
    pub supported_capabilities: Vec<ModelCapability>,
}
```

### Configuration

Provider configuration in `config.toml`:

```toml
[defaults]
provider = "openai"
model = "gpt-4o"
temperature = 0.7
max_tokens = 4096

[providers.openai]
enabled = true
api_key = "sk-..."

[providers.ollama]
enabled = true
base_url = "http://localhost:11434"
```

### Supported Providers

See README.md for the full list of 30+ providers.

## Agent Model

### Single Visible Agent

One agent with internal role modes, not multiple autonomous agents:

```rust
pub struct FeverAgent {
    provider: Arc<ProviderClient>,
    roles: RoleRegistry,
    config: AgentConfig,
    current_role: String,
}
```

### Specialist Roles

50+ internal specialist role profiles with:

- ID and name
- Description
- System prompt
- Capabilities list
- Tools list
- Temperature override

Example:

```rust
SpecialistRole::new(
    "coder".to_string(),
    "Coder".to_string(),
    "Code implementation and modification".to_string(),
    "You are a coding specialist. Your task is to write clean, efficient, well-documented code.".to_string(),
)
.with_capabilities(vec!["coding".to_string(), "debugging".to_string()])
.with_tools(vec!["filesystem".to_string(), "shell".to_string(), "grep".to_string()])
```

### Role Invocation

The agent can switch roles based on task context:

```rust
agent.set_role("coder")?;
let response = agent.chat(&messages, &context).await?;
```

## Tooling System

### Tool Interface

All tools implement the `Tool` trait:

```rust
#[async_trait]
pub trait Tool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    async fn execute(&self, args: Value, context: &ExecutionContext) -> Result<Value>;
    fn schema(&self) -> ToolSchema;
}
```

### Built-in Tools

- **shell**: Execute shell commands
- **filesystem**: Read, write, list files
- **grep**: Search for patterns
- **git**: Git operations
- **browser**: Chrome MCP integration
- **search**: Web search

### Tool Registration

```rust
let mut registry = ToolRegistry::new();
registry.register(Box::new(ShellTool::new()))?;
registry.register(Box::new(FilesystemTool::new()))?;
```

## Search System

### DuckDuckGo Parser

HTML parsing without API key:

```rust
pub struct DuckDuckGoParser;

impl SearchParser for DuckDuckGoParser {
    fn name(&self) -> &str {
        "duckduckgo"
    }

    fn parse_results(&self, html: &str, query: &str) -> SearchResult<SearchResults> {
        // Parse HTML and extract results
    }
}
```

### Caching

SQLite-based cache with TTL:

```rust
pub struct SearchCache {
    conn: Connection,
    ttl_hours: u64,
}
```

Cache entries are automatically pruned based on TTL.

## Browser Integration

### Chrome MCP

Chrome DevTools MCP for browser automation:

```rust
pub struct BrowserTool {
    enabled: bool,
}

impl Tool for BrowserTool {
    async fn execute(&self, args: Value, context: &ExecutionContext) -> Result<Value> {
        match action {
            "snapshot" => self.get_snapshot(args).await?,
            "navigate" => self.navigate(args).await?,
            "click" => self.click(args).await?,
            "screenshot" => self.screenshot(args).await?,
            "evaluate" => self.evaluate(args).await?,
        }
    }
}
```

## TUI Design

### Pane Layout

```
┌──────────────────────────────┬──────────────────────────────┐
│         Messages ●           │            Plan ●            │
├──────────────────────────────┼──────────────────────────────┤
│         Tasks ●              │         Tool Log ●           │
├──────────────────────────────┼──────────────────────────────┤
│         Browser ●            │                              │
└──────────────────────────────┴──────────────────────────────┘
│                Status: Fever Code v0.1.0 - Ready             │
└──────────────────────────────────────────────────────────────┘
```

### Keyboard Shortcuts

- `1-5`: Switch focus between panes
- `q`/`Esc`: Quit
- `Up`/`Down`: Scroll

## Configuration

### File Location

Configuration stored in `~/.config/fevercode/config.toml`.

### Configuration Structure

```toml
[defaults]
provider = "openai"
model = "gpt-4o"
temperature = 0.7
max_tokens = 4096

[ui]
theme = "dark"
auto_scroll = true
show_thinking = true

[tools]
browser_enabled = true
search_enabled = true
shell_enabled = true
git_enabled = true

[search]
engine = "duckduckgo"
searxng_url = null
max_results = 10
cache_enabled = true

[providers.openai]
enabled = true
api_key = "sk-..."
```

## Data Flow

### Typical Workflow

1. User enters request in TUI
2. FeverAgent selects appropriate role
3. Agent builds prompt with role-specific system message
4. ProviderAdapter sends request to LLM
5. LLM may request tool calls
6. ToolRegistry executes tools
7. Tool results returned to LLM
8. Final response displayed in TUI
9. Events published to EventBus
10. UI updates based on events

## Performance Considerations

### Async/Await

Extensive use of tokio for concurrent operations:

- Tool execution in parallel when possible
- Streaming responses (future)
- Non-blocking I/O

### Caching

- Search result cache with TTL
- Provider response caching (future)
- Session caching (future)

### Memory

- SQLite for persistent storage
- In-memory cache with limits
- Prompt summarization for long conversations (future)

## Security Considerations

### Tool Permissions

Each tool declares its capabilities:

```rust
impl Tool for ShellTool {
    fn schema(&self) -> ToolSchema {
        ToolSchema {
            name: "shell".to_string(),
            description: "Execute shell commands".to_string(),
            parameters: /* ... */,
        }
    }
}
```

### Role-Based Access

Different roles have access to different tools:

```rust
SpecialistRole::new(...)
.with_tools(vec!["filesystem".to_string(), "grep".to_string()])
```

### Configuration Security

- API keys stored in config file (user-controlled)
- No default credentials
- Encrypted storage option (future)

## Extensibility

### Adding a New Provider

1. Implement `ProviderAdapter` trait
2. Register with `ProviderClient`
3. Add to configuration schema

### Adding a New Tool

1. Implement `Tool` trait
2. Register with `ToolRegistry`
3. Add to role tool lists

### Adding a New Role

1. Create `SpecialistRole` with system prompt
2. Register with `RoleRegistry`
3. Configure tools and capabilities

### Adding a New Search Engine

1. Implement `SearchParser` trait
2. Register with `SearchClient`

## Testing

### Unit Tests

Each crate includes unit tests:

```bash
cargo test
```

### Integration Tests

End-to-end testing of workflows:

```bash
cargo test --test integration
```

### Manual Testing

TUI testing requires manual verification:

```bash
cargo run -- fever code
```

## Future Improvements

- Streaming responses
- Session persistence
- Plan export/import
- Multi-file editing
- Enhanced Chrome MCP integration
- Provider health monitoring
- Role composition
- Custom user roles
- RAG for codebase context
- Semantic code search

---

For questions or contributions, see [CONTRIBUTING.md](CONTRIBUTING.md) or open an issue on GitHub.
