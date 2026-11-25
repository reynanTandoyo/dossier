<!-- dossier-format: 1 -->
# DEC-041 · D41: Trial and error is recorded as an investigation

- kind: decision
- status: approved
- created: 2026-11-25T07:33:28+01:00
- author: reyna
- via: reyna - claude
- approved: 2026-11-25T07:33:28+01:00 by reyna
- valid-from: 2026-11-25T07:33:28+01:00
- task: T-286
- code: src/cmd/problem.rs
- topic: entries
- related: DEC-042
- history: 2026-11-25T07:33:28+01:00 reyna via reyna - claude: created
- history: 2026-11-25T07:33:28+01:00 reyna via reyna - claude: approved
- history: 2026-11-25T20:40:56+01:00 reyna via reyna - claude: changed — field code
- history: 2026-11-25T20:45:13+01:00 reyna via reyna - claude: changed — field topic
- history: 2026-11-25T21:45:13+01:00 reyna via reyna - claude: changed — related DEC-042

## Decision

Trial and error is recorded in an eighth kind of entry, the *investigation* (`INV-001`): one problem, the attempts in the order they were made, each with its hypothesis and its result, and what solved it. It serves people and LLMs alike. (Owner, 2026-11-22.)

## Reason

Single dead ends lose the order and the link to what finally worked, and the order is often the recipe: a change that failed first may work after another one.
