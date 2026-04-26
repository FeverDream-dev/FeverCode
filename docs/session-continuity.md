# FeverCode Session Continuity

FeverCode maintains session continuity via persistent event logs and compact summaries to allow agents to resume work after interruptions or context compaction.

## Key files
- .fevercode/session/events.jsonl — JSON Lines logging of session events (start, tool usage, edits, commands, compactions, stop).
- .fevercode/session/latest.md — Compact, human-friendly summary used to resume context quickly.

## Event types
- SessionStart
- BeforeTool
- AfterTool
- BeforeEdit
- AfterEdit
- BeforeCommand
- AfterCommand
- BeforeCompact
- AfterCompact
- SessionStop

## Summaries and resumption
- Summaries are generated after notable milestones or at regular intervals and written to latest.md.
- When resuming, agents load the latest.md as the starting context and replay events from events.jsonl to reconstruct the session flow.
- Context compaction may occur to shorten the active context; resumption uses latest.md to rehydrate the planning state and then applies the remaining events.
