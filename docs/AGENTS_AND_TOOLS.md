# Agents and Tooling Inspiration

FeverCode must be original code. Use these products as inspiration for patterns only.

## Core inspiration list

- OpenCode: terminal-first, desktop/IDE optional, provider-flexible coding agent.
- OpenAI Codex CLI: local terminal coding agent, Rust performance, reads/edits/runs code in selected directory.
- Claude Code: approval modes, hooks, permission controls, terminal-native agent workflows.
- Gemini CLI: ReAct loop, built-in tools, Google Search grounding, file operations, shell commands, web fetching, MCP support.
- Aider: terminal pair programming, local git repo editing, chat commands, model flexibility.
- Cline: plan/act feel, step-by-step tool use, user approval before commands, MCP extension.
- Goose: extensible open-source agent, CLI/desktop/API, custom distributions, provider/extensions architecture.
- OpenHands: software-development agents, planning, execution, cloud scaling, SDK concepts.
- SWE-agent: issue-to-patch loop and benchmark-style operational discipline.
- Continue, Roo Code, Qodo, Copilot coding agent, Devin, Replit Agent, Cursor, Windsurf: study UX, workflows, and failure modes.

## FeverCode built-in agents

### Ra Planner

Clarifies the user intent and creates a plan before writing.

### Thoth Architect

Builds the repo map, architecture notes, dependency graph, and context pack.

### Anubis Guardian

Enforces safety rules, workspace-only writes, permission prompts, secrets detection, and destructive command blocking.

### Ptah Builder

Applies patches, creates files, refactors, and implements code.

### Maat Checker

Runs tests, type checks, linting, smoke checks, and operational verification.

### Seshat Docs

Maintains docs, README, changelog, usage examples, and release notes.

## Tool categories to implement

- `read_file`, `list_files`, `search_text`, `repo_map`.
- `propose_patch`, `apply_patch`, `write_file`, `delete_file`.
- `run_shell`, `run_tests`, `run_lint`, `run_typecheck`.
- `git_status`, `git_diff`, `git_checkpoint`, `git_restore`.
- `doctor_check`, `provider_check`, `mcp_check`.
- `ask_user`, `approval_request`, `show_diff`.

## MCP plan

- Read `.fevercode/mcp.json`.
- Support stdio servers first.
- Expose MCP tools in the same approval queue as built-in tools.
- Show server, tool name, arguments, and risk level before execution.

## Expansion radar

Use curated directories like `awesome-cli-coding-agents` and GitHub topics to keep discovering new terminal agents, wrappers, harnesses, sandboxes, MCP servers, and prompt packs. Do not vendor their code. For each candidate, record:

- Name and link.
- License.
- Product idea worth learning from.
- What FeverCode should do differently.
- Whether it suggests a provider, agent role, MCP server, sandbox, or check tool.

Suggested categories for 100+ research expansion:

1. Terminal coding agents.
2. IDE autonomous agents.
3. SWE-bench agents.
4. Shell/browser automation agents.
5. MCP servers.
6. Sandbox and approval systems.
7. Patch/diff engines.
8. Repo-map/indexing tools.
9. Test-generation tools.
10. AI code review and CI agents.
