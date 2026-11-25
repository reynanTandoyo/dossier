<!-- dossier-format: 1 -->
# DEC-024 · D24: The server backup protects the dossier; pack makes one archive

- kind: decision
- status: approved
- created: 2026-11-25T07:33:27+01:00
- author: reyna
- via: reyna - claude
- approved: 2026-11-25T07:33:27+01:00 by reyna
- valid-from: 2026-11-25T07:33:27+01:00
- code: src/cmd/pack.rs
- topic: files
- history: 2026-11-25T07:33:27+01:00 reyna via reyna - claude: created
- history: 2026-11-25T07:33:27+01:00 reyna via reyna - claude: approved
- history: 2026-11-25T20:40:55+01:00 reyna via reyna - claude: changed — field code
- history: 2026-11-25T20:45:12+01:00 reyna via reyna - claude: changed — field topic

## Decision

The dossier depends on the server's backup. `dossier` offers a command that packs a dossier into one archive, and nothing more.

## Reason

One copy and no git (D9, D12) means the backup is the only protection against losing the disk.
