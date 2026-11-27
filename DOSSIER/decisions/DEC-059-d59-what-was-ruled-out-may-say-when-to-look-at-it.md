<!-- dossier-format: 1 -->
# DEC-059 · D59: What was ruled out may say when to look at it again (revisit-if)

- kind: decision
- status: approved
- created: 2026-11-27T20:10:17+01:00
- author: reyna
- via: reyna - claude
- approved: 2026-11-27T20:10:17+01:00 by reyna
- valid-from: 2026-11-27T20:10:17+01:00
- tags: revisit-if, decide, dead-end, ruled-out
- topic: entries
- code: src/cmd/decide.rs, src/cmd/propose.rs
- history: 2026-11-27T20:10:17+01:00 reyna via reyna - claude: created
- history: 2026-11-27T20:10:17+01:00 reyna via reyna - claude: approved

## Decision

A decision, a proposal and a dead-end finding may carry the field revisit-if: the condition under which what was ruled out is looked at again. decide and propose show the most related ruled-out entries (declined proposals, dead ends, decisions with a condition) with their condition, and search prints the condition under a result that has one.

## Reason

A Rejected section, a declined proposal and a dead end say why something was ruled out but not when that stops being true, and decide showed only the decisions in force, so a dead end or a declined proposal was not seen when the same question came back. Idea weighed from the study of the public toppingout logbook on 2026-11-27; only the idea is taken, no text and no code.

## Rejected

A required condition on every rejection: most have none worth writing. A condition per rejected option inside the Rejected section: one line per entry is enough and stays a plain field. Showing ruled-out entries in brief: R-CTX-4 keeps them out so the brief stays short.
