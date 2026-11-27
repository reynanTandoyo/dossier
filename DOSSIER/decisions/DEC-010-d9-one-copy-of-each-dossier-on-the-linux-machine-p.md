<!-- dossier-format: 1 -->
# DEC-010 · D9: One copy of each dossier on the Linux machine; PCs use ssh

- kind: decision
- status: approved
- created: 2026-11-25T07:33:27+01:00
- author: reyna
- via: reyna - claude
- approved: 2026-11-25T07:33:27+01:00 by reyna
- valid-from: 2026-11-25T07:33:27+01:00
- code: src/remote/
- topic: remote
- related: DEC-043, ASM-001, ASM-002
- tags: server, ssh, remote, linux
- history: 2026-11-25T07:33:27+01:00 reyna via reyna - claude: created
- history: 2026-11-25T07:33:27+01:00 reyna via reyna - claude: approved
- history: 2026-11-25T20:40:54+01:00 reyna via reyna - claude: changed — field code
- history: 2026-11-25T20:45:12+01:00 reyna via reyna - claude: changed — field topic
- history: 2026-11-25T21:45:12+01:00 reyna via reyna - claude: changed — related DEC-043
- history: 2026-11-25T21:45:12+01:00 reyna via reyna - claude: changed — related ASM-001
- history: 2026-11-25T21:45:12+01:00 reyna via reyna - claude: changed — related ASM-002
- history: 2026-11-27T19:12:39+01:00 reyna via reyna - claude: changed — field tags

## Decision

One copy of each dossier, on the Linux machine, changed only by commands that run on that machine. PCs reach it through ssh (remote mode).

## Reason

The lock works only between programs on the same machine, and one copy needs no syncing.
