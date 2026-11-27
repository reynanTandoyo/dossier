<!-- dossier-format: 1 -->
# DEC-043 · D43: Each command from a PC makes its own ssh connection

- kind: decision
- status: approved
- created: 2026-11-25T07:33:28+01:00
- author: reyna
- via: reyna - claude
- approved: 2026-11-25T07:33:28+01:00 by reyna
- valid-from: 2026-11-25T07:33:28+01:00
- task: T-053
- code: src/remote/ssh.rs
- topic: remote
- related: FND-011, DEC-010
- history: 2026-11-25T07:33:28+01:00 reyna via reyna - claude: created
- history: 2026-11-25T07:33:28+01:00 reyna via reyna - claude: approved
- history: 2026-11-25T20:40:47+01:00 reyna via reyna - claude: changed — field code
- history: 2026-11-25T20:45:13+01:00 reyna via reyna - claude: changed — field topic
- history: 2026-11-25T21:45:10+01:00 reyna via reyna - claude: changed — related FND-011
- history: 2026-11-25T21:45:12+01:00 reyna via reyna - claude: changed — related DEC-010

## Decision

A command from a PC makes its own ssh connection; nothing is kept open in the background. Reading through the network share with a file browser or an editor stays possible at any time (R-XOS-1). R-REMOTE-7 is not built. (Owner, 2026-11-23.)

## Reason

The ssh program of Windows cannot share a connection, and a helper process on every PC would be more to install and to go wrong than the wait it saves.
