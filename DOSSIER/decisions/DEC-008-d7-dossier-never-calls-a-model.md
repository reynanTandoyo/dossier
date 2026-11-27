<!-- dossier-format: 1 -->
# DEC-008 · D7: dossier never calls a model

- kind: decision
- status: approved
- created: 2026-11-25T07:33:27+01:00
- author: reyna
- via: reyna - claude
- approved: 2026-11-25T07:33:27+01:00 by reyna
- valid-from: 2026-11-25T07:33:27+01:00
- topic: search
- related: DEC-012
- tags: model, search
- history: 2026-11-25T07:33:27+01:00 reyna via reyna - claude: created
- history: 2026-11-25T07:33:27+01:00 reyna via reyna - claude: approved
- history: 2026-11-25T20:45:12+01:00 reyna via reyna - claude: changed — field topic
- history: 2026-11-25T21:45:13+01:00 reyna via reyna - claude: changed — related DEC-012
- history: 2026-11-27T19:12:39+01:00 reyna via reyna - claude: changed — field tags

## Decision

`dossier` never calls a model, just like `todo`: the model calls the tool. Drafts are written by the LLM that is already in the session, using material and rules that `dossier` prints.

## Reason

No network code, no keys, no cost outside the session, and it works with every LLM.
