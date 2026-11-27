<!-- dossier-format: 1 -->
# DEC-016 · D15: Large files are never stored or copied by the dossier itself

- kind: decision
- status: approved
- created: 2026-11-25T07:33:27+01:00
- author: reyna
- via: reyna - claude
- approved: 2026-11-25T07:33:27+01:00 by reyna
- valid-from: 2026-11-25T07:33:27+01:00
- topic: files
- related: DEC-046, DEC-019, DEC-026
- history: 2026-11-25T07:33:27+01:00 reyna via reyna - claude: created
- history: 2026-11-25T07:33:27+01:00 reyna via reyna - claude: approved
- history: 2026-11-25T20:45:12+01:00 reyna via reyna - claude: changed — field topic
- history: 2026-11-25T21:45:12+01:00 reyna via reyna - claude: changed — related DEC-046
- history: 2026-11-25T21:45:12+01:00 reyna via reyna - claude: changed — related DEC-019
- history: 2026-11-25T21:45:12+01:00 reyna via reyna - claude: changed — related DEC-026

## Decision

Large simulation files are never stored or copied by the dossier itself. It records who took which file and checks before a file is returned.

## Reason

The files are gigabytes; the risk is overwriting a colleague's work, not losing history.
