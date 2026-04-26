# FeverCode Commands — Reference

Current status: Working. Core commands are implemented and documented. Some advanced flows (like patch/diff and approvals) are in experimental stages until wiring is complete.

Command reference
- fever init
- fever doctor
- fever plan
- fever run
- fever endless
- fever providers
- fever agents
- fever version
- fever patch
- fever diff
- fever approve

Flags and usage examples
- fever init [--config PATH]  - Initialize workspace and components
  Example: fever init --config fever.yaml
- fever doctor [--verbose] - Run health checks
  Example: fever doctor --verbose
- fever plan [--format json|markdown] - Generate the current plan
  Example: fever plan --format json
- fever run [--step N] - Execute plan steps
  Example: fever run --step 2
- fever endless providers [--watch] - Loop through providers endlessly
  Example: fever endless providers --watch
- fever providers [--detail] - List configured providers
- fever agents [--detail] - List all agents and roles
- fever version [--short] - Show version
- fever patch [--path PATH] [--old OLD] [--new NEW] - Apply code patch
  Example: fever patch --path src/main.rs --old "foo" --new "bar"
- fever diff [--path PATH] - Show changes to a file
- fever approve [--id ID] - Approve a pending action

Examples
- Start from a clean slate, initialize, diagnose, plan, and run the current plan:
  fever init --config fever.yaml
  fever doctor
  fever plan
  fever run

- Inspect configuration and providers:
  fever providers
  fever agents

Notes
- The MCP client exists but is not yet wired to the TUI; behavior described in mcp.md.
