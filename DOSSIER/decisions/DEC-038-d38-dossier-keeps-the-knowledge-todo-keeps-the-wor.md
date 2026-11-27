<!-- dossier-format: 1 -->
# DEC-038 · D38: dossier keeps the knowledge, todo keeps the work

- kind: decision
- status: approved
- created: 2026-11-25T07:33:28+01:00
- author: reyna
- via: reyna - claude
- approved: 2026-11-25T07:33:28+01:00 by reyna
- valid-from: 2026-11-25T07:33:28+01:00
- code: src/todo_bridge.rs
- topic: todo
- related: DEC-001, DEC-039
- tags: todo
- history: 2026-11-25T07:33:28+01:00 reyna via reyna - claude: created
- history: 2026-11-25T07:33:28+01:00 reyna via reyna - claude: approved
- history: 2026-11-25T20:40:56+01:00 reyna via reyna - claude: changed — field code
- history: 2026-11-25T20:45:13+01:00 reyna via reyna - claude: changed — field topic
- history: 2026-11-25T21:45:12+01:00 reyna via reyna - claude: changed — related DEC-001
- history: 2026-11-25T21:45:12+01:00 reyna via reyna - claude: changed — related DEC-039
- history: 2026-11-27T19:12:40+01:00 reyna via reyna - claude: changed — field tags

## Decision

`dossier` keeps the knowledge, `todo` keeps the work, side by side. `dossier` never creates, changes, claims or closes a task and never writes `TODO.md`; it reads tasks only through `todo --json`, and `dossier todo` only passes a command through to `todo` on the server. `dossier`'s register on the PC (`connect`, `projects`) holds the server connection for remote mode and is separate from `todo`'s project register (`todo project`, `todo overview`). Every later feature keeps to this. (Owner, 2026-11-21.)

## Reason

Two tools that each do one thing work well together; overlapping features would drift apart.
