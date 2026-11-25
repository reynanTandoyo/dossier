<!-- dossier-format: 1 -->
# DEC-019 · D18: The dossier points to files; each task has a file list with roles

- kind: decision
- status: approved
- created: 2026-11-25T07:33:27+01:00
- author: reyna
- via: reyna - claude
- approved: 2026-11-25T07:33:27+01:00 by reyna
- valid-from: 2026-11-25T07:33:27+01:00
- code: src/filelist.rs
- topic: files
- history: 2026-11-25T07:33:27+01:00 reyna via reyna - claude: created
- history: 2026-11-25T07:33:27+01:00 reyna via reyna - claude: approved
- history: 2026-11-25T20:40:55+01:00 reyna via reyna - claude: changed — field code
- history: 2026-11-25T20:45:12+01:00 reyna via reyna - claude: changed — field topic

## Decision

The dossier points to files; it doesn't hold them. Each task has a list of its files with a role for each (model, geometry, result, …), kept as paths relative to the project folder.

## Reason

A reader must find the simulation file, the geometry and the results of a task without searching the disk, and the files stay where the programs expect them.
