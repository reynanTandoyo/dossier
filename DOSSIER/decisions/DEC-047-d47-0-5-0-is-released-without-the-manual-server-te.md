<!-- dossier-format: 1 -->
# DEC-047 · D47: 0.5.0 is released without the manual server test

- kind: decision
- status: approved
- created: 2026-11-25T07:33:28+01:00
- author: reyna
- via: reyna - claude
- approved: 2026-11-25T07:33:28+01:00 by reyna
- valid-from: 2026-11-25T07:33:28+01:00
- task: T-054
- topic: release
- related: ASM-005, DEC-045
- tags: release, take, server, test
- history: 2026-11-25T07:33:28+01:00 reyna via reyna - claude: created
- history: 2026-11-25T07:33:28+01:00 reyna via reyna - claude: approved
- history: 2026-11-25T20:45:13+01:00 reyna via reyna - claude: changed — field topic
- history: 2026-11-25T21:45:11+01:00 reyna via reyna - claude: changed — related ASM-005
- history: 2026-11-25T21:45:14+01:00 reyna via reyna - claude: changed — related DEC-045
- history: 2026-11-27T19:12:41+01:00 reyna via reyna - claude: changed — field tags

## Decision

0.5.0 is released without the manual test on the real server (TEST-10). It is run before `take` and `return` are used on real work. (Owner, 2026-11-23.)

## Reason

The team does not use the large-file feature yet; the automatic tests, and an end-to-end run with real ssh and scp against a test server in WSL, pass.
