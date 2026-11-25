<!-- dossier-format: 1 -->
# DEC-044 · D44: The session review reads dossier's own session format

- kind: decision
- status: approved
- created: 2026-11-25T07:33:28+01:00
- author: reyna
- via: reyna - claude
- approved: 2026-11-25T07:33:28+01:00 by reyna
- valid-from: 2026-11-25T07:33:28+01:00
- task: T-337
- code: src/session.rs, src/claude.rs
- topic: hooks
- history: 2026-11-25T07:33:28+01:00 reyna via reyna - claude: created
- history: 2026-11-25T07:33:28+01:00 reyna via reyna - claude: approved
- history: 2026-11-25T20:40:57+01:00 reyna via reyna - claude: changed — field code
- history: 2026-11-25T20:45:13+01:00 reyna via reyna - claude: changed — field topic

## Decision

The session review reads a session file in `dossier`'s own format, which has a version number like the entry format. A transcript of Claude Code is first converted to it; only that converter knows Claude Code's format. (Owner, 2026-11-23.)

## Reason

Claude Code's transcript format is internal and changes between releases. With an own format the review works for every LLM, and a change at Claude Code touches one converter.
