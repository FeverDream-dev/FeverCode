FeverCode Security Policy

Overview
FeverCode operates within a defined workspace. All writes and prompts are bounded to the project workspace unless explicitly approved.

Reporting vulnerabilities
If you discover a vulnerability, please report via security@fevercode.dev or through GitHub Security Advisories on this repository. Include steps to reproduce, impact, and proposed remediation. We will acknowledge and triage promptly.

Workspace-only write rule
All writes occur inside the FeverCode workspace. External writes require explicit, documented approval.

Approval modes
- None: changes require review before merging.
- Maintained: changes require maintainer approval before merge.
- Auto: changes can be merged automatically after tests pass.

Prompt injection defense
Prompts are sanitized and cannot cause arbitrary code execution. The system avoids evaluating or running code supplied in prompts.
