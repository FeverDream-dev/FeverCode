# FeverCode Agent Contract

This file teaches AI coding agents how to work in this repository.

## Product

Build `fever` / `fevercode`: a Rust-based full-screen terminal AI coding agent for macOS and Linux.

## Non-negotiables

- Never write outside the detected workspace root.
- Never copy source code from OpenCode, Codex, Claude Code, Gemini CLI, Aider, Goose, Cline, OpenHands, or any other project.
- Use public tools only as product inspiration.
- Keep the project compiling after every meaningful change.
- Add tests for workspace boundary, config parsing, provider registry, and safety policy.
- Prefer simple, reliable MVP code over magical abstractions.

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
6. Agent registry.
7. MCP server support.
8. Endless mode with budget, iteration, test, and checkpoint limits.
