<!-- dossier-format: 1 -->
# DEC-017 · D16: Rust 1.89 and edition 2024; one program for Linux and one for Windows

- kind: decision
- status: approved
- created: 2026-11-25T07:33:27+01:00
- author: reyna
- via: reyna - claude
- approved: 2026-11-25T07:33:27+01:00 by reyna
- valid-from: 2026-11-25T07:33:27+01:00
- code: Cargo.toml
- topic: release
- history: 2026-11-25T07:33:27+01:00 reyna via reyna - claude: created
- history: 2026-11-25T07:33:27+01:00 reyna via reyna - claude: approved
- history: 2026-11-25T20:40:54+01:00 reyna via reyna - claude: changed — field code
- history: 2026-11-25T20:45:12+01:00 reyna via reyna - claude: changed — field topic

## Decision

Rust 1.89 or newer, edition 2024. One program for Linux x64 (the server, where the work is done) and one for Windows 10/11 x64 (remote mode, and local dossiers). Same code rules as `todo`.

## Reason

One toolchain for all the team's tools.
