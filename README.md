# Fever Code

A terminal-first, open-source AI coding platform for Linux.

**Fever Code** is a CLI/TUI-first system that provides a single visible agent with 50+ internal specialist roles, supporting 30+ LLM providers through a clean abstraction layer. It includes Chrome MCP support, free web search, and a polished TUI with chat, plans, tasks, tool logs, and browser panels.

## Features

- **Single Visible Agent**: One agent with 50+ internal specialist role modules
- **30+ LLM Provider Support**: Clean abstraction layer supporting OpenAI, Anthropic, Google, Ollama, and many more
- **No-Paid-API Search**: DuckDuckGo-based search tool requiring no API keys
- **Chrome MCP Integration**: First-class browser debugging support
- **Polished TUI**: Chat, plans, tasks, tool logs, and browser panels
- **Linux-First**: Optimized for Linux with broad distribution support
- **Open Source**: MIT/Apache-2.0 dual licensed

## Quick Start

### Installation

One-line installer:

```bash
curl -fsSL https://github.com/FeverDream-dev/FeverCode/releases/latest/download/install.sh | bash
```

### From Source

```bash
git clone https://github.com/FeverDream-dev/FeverCode.git
cd FeverCode
cargo install --path .
```

### Usage

```bash
# Start the TUI
fever

# or
fever code

# List available roles
fever roles

# Show configuration
fever config

# Show version
fever version
```

## Architecture

Fever Code is built with Rust and organized into modular crates:

- `fever-core`: Core orchestration engine (tasks, events, memory, tools)
- `fever-agent`: Single visible agent with role system
- `fever-providers`: LLM provider abstraction layer
- `fever-tools`: System tools (shell, filesystem, git, grep)
- `fever-search`: No-paid-API search (DuckDuckGo)
- `fever-browser`: Chrome MCP integration
- `fever-tui`: Terminal user interface
- `fever-config`: Configuration management
- `fever-cli`: Command-line interface
- `fever-release`: Release and build management

See [ARCHITECTURE.md](ARCHITECTURE.md) for detailed technical documentation.

## Provider Support

Fever Code supports 30+ LLM providers:

**Native Adapters:**
- OpenAI, Anthropic, Google Gemini, Ollama (Local & Cloud)
- Together AI, Groq, Fireworks, Mistral, Cohere
- xAI, DeepSeek, Perplexity

**Cloud Providers:**
- Azure OpenAI, AWS Bedrock, Google Vertex AI
- Hugging Face Inference, Cloudflare Workers AI
- SambaNova, Cerebras, Replicate

**Other:**
- MiniMax, OpenRouter, Z.ai, Nebius, Baseten, Novita, AI21
- Moonshot/Kimi, Alibaba DashScope, Tencent Hunyuan
- Generic OpenAI-compatible endpoints

See [ARCHITECTURE.md](ARCHITECTURE.md#provider-system) for configuration details.

## Specialist Roles

The single agent can invoke 50+ internal specialist roles:

- **Core**: Researcher, Planner, Architect, Coder, Refactorer, Tester, Debugger, Reviewer
- **DevOps**: DevOps Engineer, CI Fixer, Release Manager, Container Specialist
- **Security**: Security Reviewer, Security Auditor, Crypto Specialist
- **Quality**: QA Specialist, Accessibility Specialist, i18n Specialist
- **Data**: Database Specialist, Data Engineer, ML Engineer
- **Specialized**: Browser Debugger, Performance Investigator, Memory Specialist, etc.

See `fever roles` for the full list.

## Configuration

Configuration is stored in `~/.config/fevercode/config.toml`:

```toml
[defaults]
provider = "openai"
model = "gpt-4o"
temperature = 0.7
max_tokens = 4096

[search]
engine = "duckduckgo"
searxng_url = null
max_results = 10
cache_enabled = true

[providers.openai]
enabled = true
api_key = "your-api-key"

[providers.ollama]
enabled = true
base_url = "http://localhost:11434"
```

See [ARCHITECTURE.md](ARCHITECTURE.md#configuration) for all options.

## Chrome MCP Setup

Chrome MCP provides browser debugging capabilities. To enable:

1. Install Chrome DevTools MCP server
2. Configure the connection in Fever Code
3. Browser panel shows DOM, console, network data

See [ARCHITECTURE.md](ARCHITECTURE.md#chrome-mcp) for setup instructions.

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

See [CONTRIBUTING.md](CONTRIBUTING.md) for contribution guidelines.

## Documentation

- [ARCHITECTURE.md](ARCHITECTURE.md) - Technical architecture
- [CONTRIBUTING.md](CONTRIBUTING.md) - Contribution guide
- [CHANGELOG.md](CHANGELOG.md) - Version history
- [ROADMAP.md](ROADMAP.md) - Future plans

## License

Dual licensed under MIT OR Apache-2.0. See [LICENSE](LICENSE) for details.

## Acknowledgments

Inspired by the ambition of OpenCode-class tools and Manus-style autonomy, but implemented as a CLI/TUI-first system for Linux with a single visible agent architecture.

Built with:
- Rust (portability, single-binary distribution)
- ratatui (TUI)
- crossterm (terminal handling)
- tokio (async runtime)
- serde (serialization)
- clap (CLI parsing)

---

**Fever Code** - Code like fever, ship like dream.
