<!-- dossier-format: 1 -->
# DEC-051 · D51: Entries that bind a file are shown before its first edit

- kind: decision
- status: approved
- created: 2026-11-25T20:10:34+01:00
- author: reyna
- via: reyna - claude
- approved: 2026-11-25T20:10:34+01:00 by reyna
- valid-from: 2026-11-25T20:10:34+01:00
- task: T-484
- code: src/cmd/guard.rs, src/shown.rs
- topic: hooks
- history: 2026-11-25T20:10:34+01:00 reyna via reyna - claude: created
- history: 2026-11-25T20:10:34+01:00 reyna via reyna - claude: approved
- history: 2026-11-25T20:45:13+01:00 reyna via reyna - claude: changed — field topic

## Decision

An LLM is shown the entries in force that bind a file before it changes the file: its first edit of the file in a session is refused once, naming those entries, and its next edit goes through (R-CODE-8).

## Reason

Shown after the edit, a rule comes too late, and the LLM can at best repair its change. A test with Claude Code 2.1.331 on 2026-11-25 showed that the reason of a refused edit reaches the model before the file changes, while text added to an allowed edit only arrives with the edit's result.
