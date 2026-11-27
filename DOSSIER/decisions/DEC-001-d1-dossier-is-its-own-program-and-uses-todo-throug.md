<!-- dossier-format: 1 -->
# DEC-001 · D1: dossier is its own program and uses todo through --json

- kind: decision
- status: approved
- created: 2026-11-25T07:33:08+01:00
- author: reyna
- via: reyna - claude
- approved: 2026-11-25T07:33:08+01:00 by reyna
- valid-from: 2026-11-25T07:33:08+01:00
- code: src/todo_bridge.rs
- topic: todo
- related: TOOL-001, DEC-038
- tags: todo, json
- history: 2026-11-25T07:33:08+01:00 reyna via reyna - claude: created
- history: 2026-11-25T07:33:08+01:00 reyna via reyna - claude: approved
- history: 2026-11-25T07:33:22+01:00 reyna via reyna - claude: changed — title
- history: 2026-11-25T20:40:53+01:00 reyna via reyna - claude: changed — field code
- history: 2026-11-25T20:45:12+01:00 reyna via reyna - claude: changed — field topic
- history: 2026-11-25T21:45:11+01:00 reyna via reyna - claude: changed — related TOOL-001
- history: 2026-11-25T21:45:12+01:00 reyna via reyna - claude: changed — related DEC-038
- history: 2026-11-27T19:12:38+01:00 reyna via reyna - claude: changed — field tags

## Decision

`dossier` is its own program. It uses `todo` (0.8 or newer) by running it with `--json`; `todo` knows nothing about `dossier`.

## Reason

`todo` edits one file under a lock; `dossier` manages many files and search. `todo` stays usable alone.
