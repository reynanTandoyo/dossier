<!-- dossier-format: 1 -->
# DEC-037 · D37: A tool has a name and an optional location

- kind: decision
- status: approved
- created: 2026-11-25T07:33:28+01:00
- author: reyna
- via: reyna - claude
- approved: 2026-11-25T07:33:28+01:00 by reyna
- valid-from: 2026-11-25T07:33:28+01:00
- code: src/cmd/tool.rs
- topic: entries
- related: DEC-031
- history: 2026-11-25T07:33:28+01:00 reyna via reyna - claude: created
- history: 2026-11-25T07:33:28+01:00 reyna via reyna - claude: approved
- history: 2026-11-25T20:40:56+01:00 reyna via reyna - claude: changed — field code
- history: 2026-11-25T20:45:13+01:00 reyna via reyna - claude: changed — field topic
- history: 2026-11-25T21:45:13+01:00 reyna via reyna - claude: changed — related DEC-031

## Decision

A tool (D31) has a name, required, and a location, optional: a path, a command or a URL when one is known. Nothing is checked for a tool without a location. (Owner, 2026-11-21.)

## Reason

Many tools have no single place yet.
