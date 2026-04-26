# FeverCode / `fever`

**FeverCode** is a starter foundation for a full-screen terminal AI coding agent inspired by the mythic intelligence of **Ra** and **Thoth**: a portal that scans the current folder, plans carefully, asks before dangerous actions, and then crafts software with precision.

> Command goals: `fever` for daily use, `fevercode` as the explicit long name.

## What this starter already includes

- Rust CLI scaffold with two binary names: `fever` and `fevercode`.
- Full-screen terminal UI placeholder using Ratatui and Crossterm.
- Workspace-root detection and a hard rule that write operations must stay inside the current project folder.
- Approval modes:
  - `ask`: ask before edits and commands.
  - `auto`: allow low-risk edits inside the workspace.
  - `spray`: allow broad autonomous edits **only inside the workspace**.
- Provider config format for Z.ai, OpenAI/ChatGPT, Codex-compatible endpoints, Gemini CLI integration, Ollama, and Ollama Cloud/OpenAI-compatible endpoints.
- Agent and MCP registry examples.
- OpenCode mega-prompt to continue building the product.
- macOS/Linux install script foundation.

## Fast start for local development

```bash
cargo run --bin fever -- doctor
cargo run --bin fever -- init
cargo run --bin fever
```

## Planned one-line installer

Once you publish GitHub releases, the public installer can be:

```bash
curl -fsSL https://raw.githubusercontent.com/YOUR_ORG/fevercode/main/install.sh | sh
```

For now, `install.sh` is a safe placeholder that supports local Cargo install and future GitHub release binaries.

## Philosophy

FeverCode should compete with OpenCode, Codex CLI, Claude Code, Gemini CLI, Aider, Cline, Goose, and OpenHands by being:

1. **Vibe-coder friendly**: beautiful full-screen terminal, clear plans, strong defaults.
2. **More thoughtful**: asks better questions before coding, builds plans, validates outcomes.
3. **Agent-rich**: specialized agents for planning, repo mapping, coding, testing, docs, security, and release checks.
4. **Provider-flexible**: Z.ai GLM Coding Plan, OpenAI/ChatGPT/Codex-compatible APIs, Gemini CLI, Ollama local, and Ollama Cloud.
5. **Workspace-safe**: even the dangerous mode cannot write outside the folder where the user launched FeverCode.

## Commercial use and no-warranty notice

This starter includes a custom commercial-friendly license template in `LICENSE.md`. Review it with a lawyer before public launch. The intended policy is: users may use FeverCode commercially, but the software is provided as-is, without refunds, guarantees, or liability for damage caused by misuse.

## Next commands to implement

```bash
fever                # launch portal TUI
fever init           # create .fevercode config in current repo
fever plan "task"    # repo scan + plan only
fever run "task"     # plan, approve, edit, test
fever endless "goal" # autonomous loop with checkpoint/doctor guardrails
fever doctor         # validate install, tools, provider keys, git status
fever providers      # list configured providers
fever mcp list       # list MCP servers
fever agents list    # list internal agents
```

## Keep it original

Do not copy code from other projects. Study public tools for product patterns, then implement original code, prompts, UI, and workflows.
