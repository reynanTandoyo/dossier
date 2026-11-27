<!-- dossier-format: 1 -->
# ASM-001 · A lock taken through the Samba share and one taken on the Linux machine may not

- kind: assumption
- status: approved
- created: 2026-11-25T07:34:44+01:00
- author: reyna
- via: reyna - claude
- approved: 2026-11-25T07:34:44+01:00 by reyna
- valid-from: 2026-11-25T07:34:44+01:00
- state: open
- code: src/lock.rs
- topic: remote
- related: DEC-010
- tags: samba, lock, server, remote
- settle-by: a test on the team server: take a lock over the Samba share while one is held on the Linux machine
- history: 2026-11-25T07:34:44+01:00 reyna via reyna - claude: created
- history: 2026-11-25T07:34:44+01:00 reyna via reyna - claude: approved
- history: 2026-11-25T20:40:57+01:00 reyna via reyna - claude: changed — field code
- history: 2026-11-25T20:45:13+01:00 reyna via reyna - claude: changed — field topic
- history: 2026-11-25T21:45:12+01:00 reyna via reyna - claude: changed — related DEC-010
- history: 2026-11-27T19:12:37+01:00 reyna via reyna - claude: changed — field tags
- history: 2026-11-27T21:27:15+01:00 reyna via reyna - claude: changed — field settle-by

## Assumption

A lock taken through the Samba share and one taken on the Linux machine may not see each other

## Reason

Not tested on the team server (open point O-1); R-LOCK-3 refuses writes on a network drive because of it
