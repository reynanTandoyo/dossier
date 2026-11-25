<!-- dossier-format: 1 -->
# FND-004 · Rust escapes the quotes of an argument for cmd /C with backslashes, which cmd do

- kind: finding
- status: approved
- created: 2026-11-25T07:34:44+01:00
- author: reyna
- via: reyna - claude
- approved: 2026-11-25T07:34:44+01:00 by reyna
- valid-from: 2026-11-25T07:34:44+01:00
- task: T-387
- code: src/cmd/session_end.rs
- topic: hooks
- related: DEC-044, FND-005
- history: 2026-11-25T07:34:44+01:00 reyna via reyna - claude: created
- history: 2026-11-25T07:34:44+01:00 reyna via reyna - claude: approved
- history: 2026-11-25T20:40:57+01:00 reyna via reyna - claude: changed — field code
- history: 2026-11-25T20:45:14+01:00 reyna via reyna - claude: changed — field topic
- history: 2026-11-25T21:45:12+01:00 reyna via reyna - claude: changed — related DEC-044
- history: 2026-11-25T21:45:12+01:00 reyna via reyna - claude: changed — related FND-005

## Observation

Rust escapes the quotes of an argument for cmd /C with backslashes, which cmd does not read; a command line passed in an environment variable that cmd expands arrives whole

## Evidence

T-387 tests on Windows 11 and Fedora 44
