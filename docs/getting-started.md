# FeverCode Getting Started — Vibe Edition

Purpose
- This document provides a quick, honest path to install FeverCode from source, run the initial setup, and understand the TUI and slash commands.

Current status: Working. Core features like cargo fmt/clippy/test pass, and the full-screen TUI with chat input and slash commands are functional.

Install from source
- Prerequisites:
  - Rust toolchain (1.70+)
  - Git
- Clone the repository into a working directory and navigate to the project
  - git clone https://github.com/FeverDream-dev/FeverCode fevercode_repo
  - cd fevercode_repo
- Build and verify
  - cargo fmt --all
  - cargo build --release
  - cargo test --tests
  - The build produces a binary (see docs/install.md for exact name in this repo).

First run
- Initialize the project workspace and components
  - fever init
- Sanity checks
  - fever doctor
- Check your preset
  - fever preset show
- Quick plan and run loop
  - fever plan
  - fever run
- Vibe code (creative one-shot mode)
  - fever vibe "build a REST API for user auth"

TUI overview
- FeverCode uses a full-screen Ratatui-based terminal UI.
- Features:
  - Chat input at the bottom for natural language prompts
  - Slash commands for quick actions
  - Real-time plan, approvals, and tool execution status
  - Preset display and llama3.2 warning indicator
- Slash commands are prefixed with a slash (/) and expose common workflows.

Slash commands (sample)
| Command | Description | Example |
| --- | --- | --- |
| /init | Re-initialize workspace components | /init |
| /doctor | Run health checks and diagnostic routines | /doctor |
| /plan | Generate and show current plan | /plan |
| /run | Execute the current plan | /run |
| /vibe | Switch to vibe coding mode (spray + creative preset) | /vibe |
| /providers | List configured providers | /providers |
| /agents | List active agents and roles | /agents |
| /preset | Show current preset for the default model | /preset |
| /model | Change active model | /model gpt-4o |

llama3.2 restriction
- llama3.2 is HARD-LOCKED to TestResearch preset.
- It is blocked from production coding in both CLI and TUI.
- Use llama3.2 for: tests, research, internet tech exploration, prototyping.
- Do NOT use llama3.2 for: production features, security code, release branches.

Troubleshooting
- If cargo fmt or cargo test fail, ensure the Rust toolchain matches the repository requirements and run cargo update.
- Ensure you are in the FeverCode project directory and that no other processes block the TUI.

Notes
- This documentation reflects the current state of FeverCode as of this write-up. Features labeled as experimental or planned are noted in other docs.
