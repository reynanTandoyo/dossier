<!-- dossier-format: 1 -->
# LES-001 · Before a phase of level-1 tasks starts, look for edits that break files outside

- kind: lesson
- status: draft
- created: 2026-11-25T07:34:45+01:00
- author: reyna
- via: reyna - claude
- confidence: experience
- topic: todo
- related: TOOL-005
- tags: worker, orchestration, test, file-list
- history: 2026-11-25T07:34:45+01:00 reyna via reyna - claude: created
- history: 2026-11-25T20:45:14+01:00 reyna via reyna - claude: changed — field topic
- history: 2026-11-25T21:45:11+01:00 reyna via reyna - claude: changed — related TOOL-005
- history: 2026-11-27T19:12:38+01:00 reyna via reyna - claude: changed — field tags

## Statement

Before a phase of level-1 tasks starts, look for edits that break files outside a task's list: tests that list every command, and literals of a struct that gains a field

## Valid for

Planning todo tasks with a three-file limit in this repository

## Evidence

T-359, T-319 and T-352 stopped for a file outside their list; the same check before phase 6 fixed T-395 before it ran
