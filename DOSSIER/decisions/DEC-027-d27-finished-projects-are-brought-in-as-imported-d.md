<!-- dossier-format: 1 -->
# DEC-027 · D27: Finished projects are brought in as imported drafts

- kind: decision
- status: approved
- created: 2026-11-25T07:33:28+01:00
- author: reyna
- via: reyna - claude
- approved: 2026-11-25T07:33:28+01:00 by reyna
- valid-from: 2026-11-25T07:33:28+01:00
- code: src/imports.rs
- topic: approval
- related: DEC-006
- tags: import, draft
- history: 2026-11-25T07:33:28+01:00 reyna via reyna - claude: created
- history: 2026-11-25T07:33:28+01:00 reyna via reyna - claude: approved
- history: 2026-11-25T20:40:55+01:00 reyna via reyna - claude: changed — field code
- history: 2026-11-25T20:45:13+01:00 reyna via reyna - claude: changed — field topic
- history: 2026-11-25T21:45:12+01:00 reyna via reyna - claude: changed — related DEC-006
- history: 2026-11-27T19:12:40+01:00 reyna via reyna - claude: changed — field tags

## Decision

Finished projects are brought in the same way new knowledge is: `dossier` prints the material, the LLM in the session drafts, a person approves. Imported entries are always drafts first and stay marked as imported.

## Reason

The dossier must not start empty, and what is reconstructed from an old folder is less certain than what was recorded at the time.
