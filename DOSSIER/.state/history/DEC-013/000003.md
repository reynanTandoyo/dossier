<!-- dossier-format: 1 -->
# DEC-013 · D12: Every change is recorded in a journal; git is optional

- kind: decision
- status: approved
- created: 2026-11-25T07:33:27+01:00
- author: reyna
- via: reyna - claude
- approved: 2026-11-25T07:33:27+01:00 by reyna
- valid-from: 2026-11-25T07:33:27+01:00
- code: src/journal.rs
- topic: entries
- history: 2026-11-25T07:33:27+01:00 reyna via reyna - claude: created
- history: 2026-11-25T07:33:27+01:00 reyna via reyna - claude: approved
- history: 2026-11-25T20:40:54+01:00 reyna via reyna - claude: changed — field code
- history: 2026-11-25T20:45:12+01:00 reyna via reyna - claude: changed — field topic

## Decision

Every change is recorded in a journal in the dossier's state folder. git is optional.

## Reason

The project folders are not git repositories today, and the history must exist anyway.
