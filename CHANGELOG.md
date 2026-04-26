# FeverCode Changelog

All notable changes to FeverCode are documented here. The format follows [Keep a Changelog](https://keepachangelog.com/).

## [0.1.0] - 2026-04-26

### Added

- CLI with two binary names: `fever` and `fevercode`
- Subcommands: `init`, `doctor`, `plan`, `run`, `endless`, `providers`, `agents`, `version`
- Full-screen Ratatui TUI with chat input, sidebar, and slash commands
- Slash commands: `/help`, `/plan`, `/run`, `/spray`, `/ask`, `/auto`, `/mode`, `/doctor`, `/diff`, `/approve`, `/status`, `/model`, `/providers`, `/theme`, `/version`, `/clear`, `/exit`
- Workspace detection and `.fevercode/config.toml` configuration
- Safety system with three approval modes: `ask`, `auto`, `spray`
- Command risk classification (privileged, destructive, credential, network, shell-write, shell-read, safe)
- Workspace-only write boundary enforced at the safety layer
- Provider abstraction with streaming OpenAI-compatible HTTP client
- 5 pre-configured providers: Z.ai, OpenAI, Ollama Local, Ollama Cloud, Gemini CLI
- External CLI bridge provider type
- 6 built-in agent roles with system prompt templates (Ra, Thoth, Anubis, Ptah, Maat, Seshat)
- Tool system: read_file, write_file, list_files, search_text, run_shell, git_status, git_diff, git_checkpoint
- MCP stdio client with JSON-RPC transport and tool discovery
- Patch/diff system with unified diff rendering and approval queue
- 43 passing tests covering safety boundaries, config parsing, patch application, workspace detection
- GitHub Actions CI: fmt, clippy, test
- Egyptian portal themed TUI with gold/amber accents and mode-colored indicators

### Security

- Workspace-only write enforcement — FeverCode never writes outside the launch directory
- Command risk classification blocks destructive, privileged, and credential-exfiltration commands even in spray mode
- Path traversal prevention for `../`, absolute paths outside root, and symlink-style escapes

### Known Limitations

- Provider streaming requires API keys to be configured — no AI calls happen without a key
- MCP client is implemented but not wired into the TUI agent loop
- Agent roles serve as prompt templates — no multi-agent orchestration loop yet
- No release binaries or crates.io publication
