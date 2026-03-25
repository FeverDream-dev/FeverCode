# Changelog

All notable changes to Fever Code will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Planned
- Full Chrome MCP integration
- Streaming responses
- Additional provider adapters
- Provider health monitoring
- Session persistence
- Plan export/import
- More specialized roles

## [0.1.0] - 2025-03-25

### Added
- Initial release of Fever Code
- Core orchestration engine with task execution
- Single visible agent with 50+ internal specialist roles
- Provider abstraction layer supporting 30+ LLM providers
- No-paid-API search tool (DuckDuckGo HTML parsing)
- Chrome MCP integration framework
- Full-featured TUI with:
  - Chat/messages pane
  - Plan view with task status
  - Task/to-do list
  - Tool activity log
  - Browser panel (placeholder for Chrome MCP)
  - Status bar
- System tools: shell, filesystem, git, grep
- Configuration management (TOML)
- Search caching with TTL
- Modular crate architecture
- One-line installer script
- Comprehensive documentation

### Providers (Scaffolded)
- OpenAI
- Anthropic
- Google Gemini
- Ollama (Local & Cloud)
- Together AI
- Groq
- Fireworks
- Mistral
- Cohere
- xAI
- DeepSeek
- Perplexity
- Azure OpenAI
- AWS Bedrock
- Google Vertex AI
- Hugging Face Inference
- Cloudflare Workers AI
- SambaNova
- Cerebras
- Replicate
- And 11 more providers (generic OpenAI-compatible)

### Specialist Roles (50)
- Researcher, Planner, Architect, Coder, Refactorer, Tester, Debugger, Reviewer
- Doc Writer, Release Manager, CI Fixer, Dependency Auditor
- Performance Investigator, Security Reviewer, Browser Debugger
- Issue Triager, Repo Analyst, Shell Executor, Git Operator, UX Critic
- Prompt Optimizer, Migration Planner, API Designer, Database Specialist
- DevOps Engineer, Frontend Engineer, Backend Engineer, Mobile Developer
- Cloud Architect, Data Engineer, ML Engineer, Security Auditor
- Compliance Specialist, Accessibility Specialist, i18n Specialist
- Monitoring Specialist, Log Analyst, Network Specialist, Container Specialist
- Caching Specialist, Async Specialist, Error Handling Specialist
- Testing Strategist, Benchmark Specialist, Memory Specialist
- Crypto Specialist, JSON Specialist, XML Specialist, Regex Specialist
- CLI Specialist, Default role

### Documentation
- README.md with quick start guide
- ARCHITECTURE.md with technical details
- CONTRIBUTING.md with development guidelines
- CODE_OF_CONDUCT.md
- SECURITY.md
- ROADMAP.md
- CHANGELOG.md

### Known Limitations
- Chrome MCP requires manual setup and configuration
- Some providers need additional implementation
- Browser panel shows placeholder without Chrome MCP connection
- Streaming responses not yet implemented
- Session persistence not yet implemented

### Technical Notes
- Built with Rust 2024 edition
- Uses ratatui for TUI
- Tokio async runtime
- SQLite for storage
- Modular workspace structure with 10 crates
- Single-binary distribution possible

[Unreleased]: https://github.com/FeverDream-dev/FeverCode/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/FeverDream-dev/FeverCode/releases/tag/v0.1.0
