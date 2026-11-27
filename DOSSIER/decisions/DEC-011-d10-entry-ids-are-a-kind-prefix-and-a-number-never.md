<!-- dossier-format: 1 -->
# DEC-011 · D10: Entry IDs are a kind prefix and a number, never reused

- kind: decision
- status: approved
- created: 2026-11-25T07:33:27+01:00
- author: reyna
- via: reyna - claude
- approved: 2026-11-25T07:33:27+01:00 by reyna
- valid-from: 2026-11-25T07:33:27+01:00
- code: src/ids.rs
- topic: entries
- related: DEC-021
- tags: entry-id
- history: 2026-11-25T07:33:27+01:00 reyna via reyna - claude: created
- history: 2026-11-25T07:33:27+01:00 reyna via reyna - claude: approved
- history: 2026-11-25T20:40:54+01:00 reyna via reyna - claude: changed — field code
- history: 2026-11-25T20:45:12+01:00 reyna via reyna - claude: changed — field topic
- history: 2026-11-25T21:45:13+01:00 reyna via reyna - claude: changed — related DEC-021
- history: 2026-11-27T19:12:39+01:00 reyna via reyna - claude: changed — field tags

## Decision

Entry IDs are a kind prefix and a number (`LES-031`), given out under the lock, never reused. From outside, an ID is written with its place: `team:LES-031`, `P-017:FND-004`.

## Reason

Short enough to say and cite. One copy with a lock makes numbers safe.
