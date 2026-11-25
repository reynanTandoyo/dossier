<!-- dossier-format: 1 -->
# FND-011 · Reusing one ssh connection from a Windows PC: the ssh program of Windows cannot

- kind: finding
- status: approved
- created: 2026-11-25T07:34:44+01:00
- author: reyna
- via: reyna - claude
- approved: 2026-11-25T07:34:44+01:00 by reyna
- valid-from: 2026-11-25T07:34:44+01:00
- outcome: dead end
- task: T-053
- code: src/remote/ssh.rs
- topic: remote
- related: DEC-043
- history: 2026-11-25T07:34:44+01:00 reyna via reyna - claude: created
- history: 2026-11-25T07:34:44+01:00 reyna via reyna - claude: approved
- history: 2026-11-25T20:40:57+01:00 reyna via reyna - claude: changed — field code
- history: 2026-11-25T20:45:14+01:00 reyna via reyna - claude: changed — field topic
- history: 2026-11-25T21:45:10+01:00 reyna via reyna - claude: changed — related DEC-043

## Observation

Reusing one ssh connection from a Windows PC: the ssh program of Windows cannot share a connection

## Evidence

D43, 2026-11-23
