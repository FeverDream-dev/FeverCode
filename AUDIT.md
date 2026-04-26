# Truthful current-state audit

Last updated: Phase 2 milestone (SOULS, context economy, installer).

## What works

- CLI with 11 subcommands: init, doctor, plan, run, endless, providers, agents, version, souls (list/show/validate/init), context (stats/compact)
- TUI with full Ratatui interface, slash commands, mode switching
- Safety: workspace boundary, command risk classification, spray guards (43+ tests)
- Provider abstraction: 5 configured providers, streaming support
- Tools: read/write/search files, shell, git status/diff/checkpoint
- Agent roles: 6 built-in souls (Ra, Thoth, Ptah, Maat, Anubis, Seshat)
- SOULS.md: agent constitution file
- Souls config: machine-readable .fevercode/souls.toml
- Context economy: output limiting, secret redaction, session event logging
- Session files: .fevercode/session/latest.md and events.jsonl
- CI workflow with fmt/clippy/test

## What's placeholder / not wired

- MCP not connected to the TUI loop
- Agents are prompts with config, not live orchestration
- No browser/search/memory tools
- No release artifacts published yet (workflow exists, tag-triggered)
- fever context stats shows MVP message (no live token counting)
- fever context compact creates summary if session data exists

## Release / installer status

- cargo-dist config in Cargo.toml
- Release workflow: .github/workflows/release.yml (tag-triggered)
- Installer scripts: install.sh (curl|sh), install.ps1 (PowerShell)
- No releases published yet — requires `git tag v0.1.0 && git push origin v0.1.0`
- Windows support in release matrix but labeled "planned" until tested

## Docs status

- README: honest status table, source-only install, no overclaims
- docs/: getting-started, install, providers, safety, commands, architecture, agents, mcp, roadmap, souls, context-economy, session-continuity, releases
- GitHub Pages site with dark Egyptian portal aesthetic
- SOULS.md at repo root (usable by any coding agent)

## Safety status

- Workspace boundary enforced at path level
- Command risk classification: 8 risk levels
- Spray mode bounded to workspace-only edits
- Secret redaction in session logs
- Path traversal blocked (../, absolute outside root)

## Technical debt

- Dead code allowed via #[allow(dead_code)] on module declarations
- Single-crate architecture
- No integration tests (unit tests only)
- Event model is MVP (struct + JSONL, no async dispatch)
