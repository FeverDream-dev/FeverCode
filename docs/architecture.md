# FeverCode Architecture — Module Overview

Current status: Working. Core modules exist and interoperate in a safe workspace-limited scope.

Module overview
- Safety: Enforces workspace-only boundaries and risk classifications.
- Config: Provider configuration, workspace settings, and user-defined hooks.
- Workspace: Root directory confinement and project-scoped state.
- Providers: mod (core provider), openai_compat (OpenAI-compatible endpoint layer), external_cli (CLI transport for external services).
- Tools: files, shell, git_tools for file operations, command execution, and git interactions.
- Agents: Planners and builders that compose plans and execute steps.
- Patch: Differential patching system for code edits.
- MCP: Maintains a client for MCP transport.
- Approval: Workflow gates for action validation.
- TUI: Ratatui-based terminal UI for interaction and feedback.

Data flow
- Prompt -> Plan: The user prompt is transformed into a high-level plan by the planner.
- Plan -> Approval: The plan is sent to an approval gate to review actions.
- Approval -> Tools: Approved actions are executed by tool modules (files, shell, git_tools).
- Tools -> Checks: Post-action checks verify results and safety constraints.
- Checks -> Summary: A concise summary is presented back to the user.
