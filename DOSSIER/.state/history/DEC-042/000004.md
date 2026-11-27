<!-- dossier-format: 1 -->
# DEC-042 · D42: An attempt is a line in an investigation, not a todo task

- kind: decision
- status: approved
- created: 2026-11-25T07:33:28+01:00
- author: reyna
- via: reyna - claude
- approved: 2026-11-25T07:33:28+01:00 by reyna
- valid-from: 2026-11-25T07:33:28+01:00
- task: T-286
- code: src/attempts.rs
- topic: entries, todo
- related: DEC-041
- history: 2026-11-25T07:33:28+01:00 reyna via reyna - claude: created
- history: 2026-11-25T07:33:28+01:00 reyna via reyna - claude: approved
- history: 2026-11-25T20:40:56+01:00 reyna via reyna - claude: changed — field code
- history: 2026-11-25T20:45:13+01:00 reyna via reyna - claude: changed — field topic
- history: 2026-11-25T21:45:13+01:00 reyna via reyna - claude: changed — related DEC-041

## Decision

An attempt is a line in an investigation, not a `todo` task. Only an attempt that is real work (long compute, another person, a plan) is a task; a person or the planner creates it with `todo`, and the investigation points to it. `dossier` never creates it (D38). (Owner, 2026-11-22.)

## Reason

Most attempts take minutes inside a task that is already claimed; a task for each would be too heavy to be written, and workers would be planning.
