<!-- dossier-format: 1 -->
# DEC-060 · D60: An assumption may name what would settle it (settle-by)

- kind: decision
- status: approved
- created: 2026-11-27T20:10:17+01:00
- author: reyna
- via: reyna - claude
- approved: 2026-11-27T20:10:17+01:00 by reyna
- valid-from: 2026-11-27T20:10:17+01:00
- tags: assumption, settle-by, status
- topic: entries
- code: src/cmd/assume.rs
- history: 2026-11-27T20:10:17+01:00 reyna via reyna - claude: created
- history: 2026-11-27T20:10:17+01:00 reyna via reyna - claude: approved

## Decision

An assumption may carry the field settle-by: the test, the measurement or the person that would confirm or refute it. assume takes --settle-by; brief and status print it on the line of each open assumption, and status counts the open assumptions that name nothing.

## Reason

Open assumptions pile up because nothing says how one gets settled: this project has five open since 2026-11-25 and none names its test. Idea weighed from the study of the public toppingout logbook on 2026-11-27; only the idea is taken, no text and no code.

## Rejected

A required settle-by: an assumption must stay a ten-second note. A due date: the review field already gives one.
