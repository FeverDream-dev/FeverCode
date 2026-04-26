# FeverCode Safety Model — Deeper View

Current status: Working. Workspace-only safety boundary is enforced. Command risk classification is defined, with spray mechanics available in experimental mode.

Modes of operation
- Ask: Prompt-driven decisions with human-in-the-loop input.
- Auto: Autonomous mode where the system executes plans with safety checks.
- Spray: Aggressive risk-blocking layer for high-risk operations. Status: Experimental.

Workspace-only rule
- All actions are scoped to the current workspace directory. Attempts to traverse outside the workspace are blocked.

Command risk classification
- High: Destructive or invasive actions (e.g., file deletion outside workspace, repo-wide changes).
- Medium: Potentially risky operations (e.g., patching files, creating new files in important paths).
- Low: Read-only operations or benign prompts.

Spray blocks (experimental mode)
- The Spray layer can halt high-risk commands even when in Auto or Ask mode.
- Attack surface examples: directory traversal, overwriting critical system files, mass deletes outside workspace.

Path traversal prevention
- All path interactions are validated against the workspace root. Absolute paths are resolved, and parent directories are checked to prevent .. traversal.

Troubleshooting
- If you see a spray-blocked action, review the origin of the command and adjust the plan to operate within the allowed workspace scope.
- Ensure the workspace boundary is correctly set for the current project directory.
