# FeverCode Codebase Audit

**Date**: 2026-04-01
**Auditor**: Automated codebase audit

---

## Executive Summary

FeverCode is a **Rust workspace with 11 crates (~8,000 LOC)** implementing a terminal coding agent. The project has a solid foundation: working provider adapters (4 adapters, 348+ models via OpenRouter), a production-quality permission system, real CLI commands, and an iterative agent loop with LoopDriver. However, the system still lacks end-to-end integration between the agent loop, tools, and TUI.

---

## Language & Runtime

- **Language**: Rust (2024 edition)
- **MSRV**: 1.85 (pinned in rust-toolchain.toml)
- **Runtime**: Tokio async runtime
- **Build system**: Cargo workspace with `resolver = "2"`

## CLI Entry Point

- Binary: `fever` (crates/fever-cli/src/main.rs)
- Parser: clap derive macros
- Commands: `chat`, `providers`, `version` (with --local and --bump flags)
- Default (no subcommand): launches TUI

## AI Providers

- 4 adapter implementations: OpenAI-compatible (covers 10+ providers), Anthropic, Gemini, Ollama
- 13 env vars auto-discovered: OPENAI_API_KEY, OPENROUTER_API_KEY, ANTHROPIC_API_KEY, GEMINI_API_KEY, GROQ_API_KEY, TOGETHER_API_KEY, DEEPSEEK_API_KEY, MISTRAL_API_KEY, FIREWORKS_API_KEY, PERPLEXITY_API_KEY, MINIMAX_API_KEY, OLLAMA_BASE_URL, FEVER_ZAI_KEY
- Streaming support via SSE (OpenAI-compatible)
- Tool/function calling supported in provider models

## Agent Loop Architecture

- **LoopDriver** (fever-agent/src/loop_driver.rs): Iterative plan→execute→verify cycle
  - Max 20 iterations by default
  - LLM call → parse tool_calls → execute tools → feed results back → repeat
  - Termination: no tool_calls, finish_reason="stop", or max iterations
- **FeverAgent** (fever-agent/src/agent.rs): Implements Agent trait, wraps ProviderClient + ToolRegistry
- **RoleRegistry** (fever-agent/src/role.rs): 10 specialist roles with system prompts

## Configuration

- TOML-based in `~/.config/fevercode/config.toml`
- Sections: defaults, providers, ui, tools, search
- ConfigManager reads/writes/creates dirs

## Existing Features

1. **Provider chat**: Single-shot and streaming LLM calls
2. **Provider model fetching**: `fever providers --fetch` fetches live model catalogs
3. **Iterative agent loop**: LoopDriver with tool execution
4. **10 specialist roles**: coder, architect, debugger, planner, reviewer, researcher, tester, default, refactorer, doc_writer
5. **4 tools**: ShellTool, FilesystemTool, GrepTool, GitTool
6. **Security**: PermissionGuard (deny-by-default, path allowlisting, command risk classification, secret redaction)
7. **Verification**: OperationalVerifier (build, test, lint, format checks)
8. **Fighting mode**: SolutionArbiter with RuleBasedScorer for comparing solutions
9. **Requirements interrogation**: Confidence scoring, clarification questions
10. **Prompt improvement**: Restructure vague requests into engineering briefs
11. **Memory store**: SQLite-backed context and message storage
12. **Retry policy**: Exponential/linear/fixed backoff
13. **Event bus**: Pub/sub event system
14. **TUI**: ratatui-based with chat, plan, tasks, tool log, browser panels
15. **Local versioning**: .fever/local/version.json with bump commands

## Tests

- **33 tests** (all passing):
  - fever-agent: 7 fighting_mode, 6 prompt_improver, 14 requirements_interrogator, 5 operational_verifier, 1 loop_driver
  - fever-core: 16 permission tests
  - fever-cli: 1 local_version test
  - fever-providers: 3 integration tests (require env var)

## Issues Found

### Fixed During Audit
1. **Streaming SSE compilation error**: `futures::stream::unfold` closure wasn't `async move`, and `reqwest::Response` wasn't `Unpin`. Fixed with mpsc channel approach.

### Remaining Issues
1. **Dead code in main.rs**: Functions `list_providers`, `run_chat`, `handle_version_command`, `list_roles`, `show_config` are defined but never called (CLI inlines their logic)
2. **Duplicate ProviderConfig**: Defined in both fever-config/src/config.rs and fever-config/src/provider.rs with identical fields
3. **BrowserTool is all placeholder**: Returns "requires Chrome MCP" for every action
4. **Memory/Retry/Search/EventBus** exist but aren't wired into the agent loop
5. **Tools lack PermissionGuard integration**: Security exists but isn't enforced
6. **fever-release makes inaccurate claims**: Says "50+ specialist roles" (actually 10)

## Architecture Decisions

- Crate structure is clean and modular
- Type system is well-used (traits, enums, strong typing)
- Error types are thorough with thiserror
- Provider layer is genuinely functional
- Security/PermissionGuard is production-quality
