<!-- dossier-format: 1 -->
# DEC-058 · D58: An LLM tags every entry it writes

- kind: decision
- status: approved
- created: 2026-11-27T18:50:48+01:00
- author: reyna
- via: reyna - claude
- approved: 2026-11-27T18:50:48+01:00 by reyna
- valid-from: 2026-11-27T18:50:48+01:00
- topic: entries
- code: src/agents/planner.md
- history: 2026-11-27T18:50:48+01:00 reyna via reyna - claude: created
- history: 2026-11-27T18:50:48+01:00 reyna via reyna - claude: approved

## Decision

An LLM gives every entry it writes two to five tags with --tags, reusing the tags already in use; the planner rules say so.

## Reason

Tags weigh three times in search and make an entry related on their own (R-FIND-2, R-FIND-3), but no rule asked for them, so the project's 82 entries had none (owner, 2026-11-27).
