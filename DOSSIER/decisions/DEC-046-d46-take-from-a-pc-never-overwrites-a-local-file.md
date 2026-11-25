<!-- dossier-format: 1 -->
# DEC-046 · D46: take from a PC never overwrites a local file

- kind: decision
- status: approved
- created: 2026-11-25T07:33:28+01:00
- author: reyna
- via: reyna - claude
- approved: 2026-11-25T07:33:28+01:00 by reyna
- valid-from: 2026-11-25T07:33:28+01:00
- task: T-407
- code: src/cmd/take.rs
- topic: files, remote
- history: 2026-11-25T07:33:28+01:00 reyna via reyna - claude: created
- history: 2026-11-25T07:33:28+01:00 reyna via reyna - claude: approved
- history: 2026-11-25T20:40:57+01:00 reyna via reyna - claude: changed — field code
- history: 2026-11-25T20:45:13+01:00 reyna via reyna - claude: changed — field topic

## Decision

`take` from a PC never overwrites a local file that already exists: it keeps the local copy and says that deleting it and running the same `take` copies the server's version. A copy goes to `<name>.dossier-part` first and is renamed only when complete, so a failed copy leaves no half file. (Owner, 2026-11-23.)

## Reason

A repeated `take` by mistake must not wipe local edits, and a retry after a failed copy still works.
