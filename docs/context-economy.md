# FeverCode Context Economy

FeverCode uses a disciplined context economy to balance reasoning depth with safety, privacy, and performance. Outputs are compressed, sensitive data is redacted, and sessions are logged for auditing and replay.

## Output truncation
- Default maximum: 200 lines per output. If a response would exceed this, it is truncated and a compact summary is presented along with a reference to the full output stored in the session log.
- The limit can be adjusted as part of the session configuration by operations.

## Secret redaction
- Any API keys, bearer tokens, private keys, or environment values found in prompts or tool outputs are redacted before being shown to the user.
- Redaction rules are defined in the configuration and applied uniformly across all agents and tools.

## Session event logging
- All session events are recorded in .fevercode/session/events.jsonl as JSON Lines for robust parsing and auditing.
- Each event contains a timestamp, event type, and relevant payload such as tool results or user actions.

## Session summaries
- Compact session summaries are written to .fevercode/session/latest.md for quick context recall.
- Full transcripts and events remain in events.jsonl for deeper analysis.

## Fever context stats and fever context compact
- Fever context stats provide high-level metrics about context usage, average output length, and redaction occurrences.
- Fever context compact creates a condensed representation of the current context suitable for cross-agent reasoning, without leaking sensitive content.
- These commands are designed to guide decisions without exposing raw transcripts.

## Design inspiration
- The approach is inspired by the idea of a context economy: compress outputs, redact secrets, and store concise summaries. The goal is to enable think-in-code style reasoning while preserving safety and provenance, without naming specific tools.
