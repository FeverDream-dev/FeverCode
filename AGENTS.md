# FeverCode Agent Contract

This file teaches AI coding agents how to work in this repository.

## Product

Build `fever` / `fevercode`: a Rust-based full-screen terminal AI coding agent for macOS, Linux, and Windows.

## Non-negotiables

- Never write outside the detected workspace root.
- Never copy source code from OpenCode, Codex, Claude Code, Gemini CLI, Aider, Goose, Cline, OpenHands, or any other project.
- Use public tools only as product inspiration.
- Keep the project compiling after every meaningful change.
- Add tests for workspace boundary, config parsing, provider registry, safety policy, souls, and context economy.
- Prefer simple, reliable MVP code over magical abstractions.

## Souls

FeverCode uses SOULS.md as its agent constitution. Read it. Follow it.

The 6 built-in souls and their roles:
- **Ra** — Planner. Clarifies goals, creates plans, defines done criteria.
- **Thoth** — Architect. Designs modules, preserves consistency.
- **Ptah** — Builder. Implements patches, keeps diffs small.
- **Maat** — Checker. Runs fmt/clippy/tests, verifies docs claims.
- **Anubis** — Guardian. Enforces safety, protects workspace boundary.
- **Seshat** — Chronicler. Updates docs, maintains changelog.

Each soul has a config entry in `.fevercode/souls.toml` with allowed tools, risk level, escalation rules, and context budget.

CLI: `fever souls list`, `fever souls show ra`, `fever souls validate`, `fever souls init`

## Context economy

Agents must not flood the model context:
- Truncate tool output at 200 lines (configurable in souls.toml)
- Redact secrets from logs (API keys, tokens, private keys)
- Store full output in `.fevercode/session/` and show summary
- Log session events to `.fevercode/session/events.jsonl`
- Generate compact summaries with `fever context compact`

## UX identity

FeverCode should feel like an Egyptian portal of craft and knowledge:

- Ra = visibility, execution, energy, command.
- Thoth = planning, memory, architecture, documentation.
- Portal = the TUI shell where tasks are planned, approved, executed, and checked.

## Approval modes

- `ask`: default. Ask before file edits, shell commands, network calls, package installs, git writes.
- `auto`: approve safe file edits in workspace and safe read-only commands.
- `spray`: allow autonomous workspace edits and commands, but block anything outside workspace, destructive home-directory commands, credential exfiltration, and privileged system mutation.

## Implementation priorities

1. Hard workspace safety boundary.
2. Provider abstraction.
3. Streaming chat in TUI.
4. Patch preview and approval queue.
5. Doctor/check tool.
6. Agent registry with souls.
7. MCP server support.
8. Endless mode with budget, iteration, test, and checkpoint limits.
9. Context economy with secret redaction and session logging.
10. One-line installer for macOS, Linux, Windows via cargo-dist.
