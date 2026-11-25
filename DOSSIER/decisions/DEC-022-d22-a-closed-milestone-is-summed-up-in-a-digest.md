<!-- dossier-format: 1 -->
# DEC-022 · D22: A closed milestone is summed up in a digest

- kind: decision
- status: approved
- created: 2026-11-25T07:33:27+01:00
- author: reyna
- via: reyna - claude
- approved: 2026-11-25T07:33:27+01:00 by reyna
- valid-from: 2026-11-25T07:33:27+01:00
- task: T-056
- code: src/digest.rs, src/cmd/digest.rs
- topic: entries
- history: 2026-11-25T07:33:27+01:00 reyna via reyna - claude: created
- history: 2026-11-25T07:33:27+01:00 reyna via reyna - claude: approved
- history: 2026-11-25T20:40:55+01:00 reyna via reyna - claude: changed — field code
- history: 2026-11-25T20:45:12+01:00 reyna via reyna - claude: changed — field topic

## Decision

When a `todo` milestone is closed, its decisions and findings are summed up in a digest. Afterwards `brief` shows the digest instead of the single entries.

## Reason

The context an LLM gets must stay small as a project grows.
