# Architecture

This document provides technical overview of Fever Code's architecture.

## Overview

Fever Code is a terminal coding agent built with Rust. It provides a focused system for working with local repositories, running tools, and managing coding tasks.

## Design Principles

1. **Terminal-first**: Optimized for Linux terminal usage
2. **Focused**: A tight core workflow, not a sprawling platform
3. **Practical**: Real tools that do real work
4. **Honest**: No fake claims about capabilities
5. **Extensible**: Clean interfaces for future growth

## System Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                         fever-cli                              │
│                      (Entry Points)                             │
│  fever | fever code | fever-code                           │
└─────────────────────────────────────────────────────────────────┘
                               │
                               ▼
┌─────────────────────────────────────────────────────────────────┐
│                         fever-tui                               │
│                      (User Interface)                            │
│  Chat Input │ Plan View │ Tasks │ Tool Log │ Status  │
└─────────────────────────────────────────────────────────────────┘
                               │
                               ▼
┌─────────────────────────────────────────────────────────────────┐
│                        fever-agent                              │
│                    (Coding Agent)                                 │
│  Agent with specialist role modes                          │
└─────────────────────────────────────────────────────────────────┘
                               │
               ┌───────────────┼───────────────┼───────────────────┐
               ▼               ▼               ▼                   ▼
┌──────────────────┐ ┌─────────────────┐ ┌──────────────────────┐
│ fever-providers │ │  fever-tools    │ │   fever-browser      │
│  (LLM APIs)     │ │ (Shell, File...) │ │  (Chrome MCP)       │
└──────────────────┘ └─────────────────┘ └──────────────────────┘
               │               │                    │
               └───────────────┼────────────────────┘
                               ▼
┌─────────────────────────────────────────────────────────────────┐
│                        fever-core                               │
│                   (Orchestration Engine)                        │
│  Task │ Event │ Memory │ Retry │ Tool │ Execution           │
│  Graph │  Bus  │ Store  │ Policy │ Router │  Engine             │
└─────────────────────────────────────────────────────────────────┘
                               │
               ┌───────────────┼───────────────┼───────────────────┐
               ▼               ▼               ▼                   ▼
┌──────────────────┐ ┌─────────────────┐ ┌──────────────────────┐
│  fever-config   │ │  fever-search   │ │   fever-release      │
│  (Config Mgmt)  │ │  (DuckDuckGo)   │ │  (Release Mgmt)     │
└──────────────────┘ └─────────────────┘ └──────────────────────┘
```

## Crate Organization

### fever-core
**Purpose**: Foundational orchestration engine

**Key Components**:
- `task`: Task graph, plan management, todo lists
- `event`: Event bus for pub/sub communication
- `execution`: Task execution engine
- `memory`: In-memory context storage
- `retry`: Exponential backoff, linear retry policies
- `tool`: Tool registry and execution dispatcher

**Dependencies**: tokio, serde, rusqlite

### fever-agent
**Purpose**: Coding agent with specialist role modes

**Key Components**:
- `agent`: FeverAgent implementation
- `role`: RoleRegistry with specialist role definitions

**Dependencies**: fever-core, fever-providers, fever-config

### fever-providers
**Purpose**: LLM provider abstraction layer

**Key Components**:
- `client`: ProviderClient with unified chat interface
- `adapter`: ProviderAdapter trait for implementing providers
- `models`: Request/response types (ChatRequest, ChatResponse)

**Current Providers**:
- **OpenAI**: Planned
- **Ollama**: Planned
- **Anthropic**: Planned

See [fever-providers/src/adapter.rs](crates/fever-providers/src/adapter.rs) for the trait definition.

### fever-tools
**Purpose**: System interaction tools

**Key Components**:
- `shell`: Shell command execution
- `filesystem`: File read/write/list operations
- `grep`: Pattern searching with regex
- `git`: Git operations (status, log, commit, branch)

**Dependencies**: fever-core, walkdir, ignore, regex

### fever-search
**Purpose**: No-API-key-required search

**Key Components**:
- `client`: SearchClient with caching
- `parser`: DuckDuckGo HTML parser
- `cache`: SQLite-based search result cache

**Dependencies**: fever-core, reqwest, scraper

### fever-browser
**Purpose**: Chrome MCP integration (planned)

**Status**: Framework in place, integration pending.

### fever-tui
**Purpose**: Terminal user interface

**Key Components**:
- `app`: FeverTui main application loop
- `ui`: Layout and pane management
- `widgets`: Individual pane implementations

**Dependencies**: fever-core, fever-agent, ratatui, crossterm

### fever-config
**Purpose**: Configuration management

**Key Components**:
- `config`: Config struct, ConfigManager
- `provider`: ProviderCredentials types

**Dependencies**: fever-core, directories

### fever-cli
**Purpose**: Command-line interface entry points

**Dependencies**: All other crates, clap

## Provider System

### Provider Abstraction

Unified interface for LLM providers:

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
api_key = "your-api-key"

[providers.ollama]
enabled = true
base_url = "http://localhost:11434"
```

## Agent Model

### Coding Agent

Aever Code uses a single coding agent with internal role modes:

```rust
pub struct FeverAgent {
    provider: Arc<ProviderClient>,
    roles: RoleRegistry,
    config: AgentConfig,
    current_role: String,
}
```

### Specialist Roles

Core specialist roles for different coding tasks:

- **Core**: Researcher, Planner, Architect, Coder, Refactorer, Tester, Debugger, Reviewer, DocWriter, Default

Roles can be selected based on task context:

## Tooling System

### Tool Interface

All tools implement the `Tool` trait

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

### Tool Registration

```rust
let mut registry = ToolRegistry::new();
registry.register(Box::new(ShellTool::new()))?;
registry.register(Box::new(FilesystemTool::new()))?;
```

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

[tools]
shell_enabled = true
git_enabled = true

[search]
engine = "duckduckgo"
max_results = 10

[providers.openai]
enabled = true
api_key = "sk-..."
```

## Development

```bash
# Clone repository
git clone https://github.com/FeverDream-dev/FeverCode.git
cd FeverCode

# Run tests
cargo test

# Format code
cargo fmt

# Run linter
cargo clippy

# Build release
cargo build --release
```

## Roadmap

### Near Term
- Complete OpenAI provider implementation
- Complete Ollama provider implementation
- Add chat input to TUI
- Wire tools into agent loop

### Medium Term
- Anthropic provider
- Streaming responses
- Chrome MCP integration
- Session persistence

### Future
- Additional providers (Gemini, Claude, etc.)
- Custom user roles
- RAG for codebase context
- Semantic code search
