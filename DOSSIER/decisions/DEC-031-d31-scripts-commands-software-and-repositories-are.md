<!-- dossier-format: 1 -->
# DEC-031 · D31: Scripts, commands, software and repositories are one kind: the tool

- kind: decision
- status: approved
- created: 2026-11-25T07:33:28+01:00
- author: reyna
- via: reyna - claude
- approved: 2026-11-25T07:33:28+01:00 by reyna
- valid-from: 2026-11-25T07:33:28+01:00
- supersedes: DEC-020
- code: src/cmd/tool.rs
- topic: entries
- related: DEC-037, DEC-005
- tags: tool
- history: 2026-11-25T07:33:28+01:00 reyna via reyna - claude: created
- history: 2026-11-25T07:33:28+01:00 reyna via reyna - claude: approved
- history: 2026-11-25T20:40:56+01:00 reyna via reyna - claude: changed — field code
- history: 2026-11-25T20:45:13+01:00 reyna via reyna - claude: changed — field topic
- history: 2026-11-25T21:45:13+01:00 reyna via reyna - claude: changed — related DEC-037
- history: 2026-11-25T21:45:13+01:00 reyna via reyna - claude: changed — related DEC-005
- history: 2026-11-27T19:12:40+01:00 reyna via reyna - claude: changed — field tags

## Decision

Scripts, commands on the server, installed software and git repositories are one entry kind, a *tool* (`TOOL-001`), with a `type` (`script`, `command`, `software`, `repo`) and a `location`. It replaces the separate software list, script entries and `script_root` of D19 and R-USE-1, R-USE-8 to R-USE-13, which are rewritten when phase 3 starts. The team standardises its tools in the meantime. (Owner, 2026-11-21.)

## Reason

The team has no single place or form for its tools yet; one flexible kind fits all of them.
