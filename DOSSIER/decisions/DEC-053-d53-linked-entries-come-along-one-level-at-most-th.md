<!-- dossier-format: 1 -->
# DEC-053 · D53: Linked entries come along, one level, at most three

- kind: decision
- status: approved
- created: 2026-11-25T21:03:06+01:00
- author: reyna
- via: reyna - claude
- approved: 2026-11-25T21:03:06+01:00 by reyna
- valid-from: 2026-11-25T21:03:06+01:00
- topic: search
- code: src/links.rs
- related: DEC-051, DEC-061
- tags: link, search
- history: 2026-11-25T21:03:06+01:00 reyna via reyna - claude: created
- history: 2026-11-25T21:03:06+01:00 reyna via reyna - claude: approved
- history: 2026-11-25T21:45:13+01:00 reyna via reyna - claude: changed — related DEC-051
- history: 2026-11-27T19:12:41+01:00 reyna via reyna - claude: changed — field tags
- history: 2026-11-27T20:10:36+01:00 reyna via reyna - claude: changed — related DEC-061

## Decision

Wherever an LLM is given entries (search, brief --task, which --binding and the refusal of guard), each entry's related entries (link) are shown under it: one level deep, titles only, at most three, only entries with status approved.

## Reason

A link is the most precise relation in the dossier because someone made it on purpose; one level, titles and three at most keep the output short, and an old link must not feed a replaced rule.
