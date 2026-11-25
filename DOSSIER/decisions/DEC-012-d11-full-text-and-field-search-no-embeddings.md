<!-- dossier-format: 1 -->
# DEC-012 · D11: Full-text and field search, no embeddings

- kind: decision
- status: approved
- created: 2026-11-25T07:33:27+01:00
- author: reyna
- via: reyna - claude
- approved: 2026-11-25T07:33:27+01:00 by reyna
- valid-from: 2026-11-25T07:33:27+01:00
- code: src/select.rs, src/cmd/search.rs
- topic: search
- history: 2026-11-25T07:33:27+01:00 reyna via reyna - claude: created
- history: 2026-11-25T07:33:27+01:00 reyna via reyna - claude: approved
- history: 2026-11-25T20:40:54+01:00 reyna via reyna - claude: changed — field code
- history: 2026-11-25T20:45:12+01:00 reyna via reyna - claude: changed — field topic

## Decision

Search is full-text plus fields. No embeddings.

## Reason

A few thousand entries don't need them, and they would add a model to every write.
