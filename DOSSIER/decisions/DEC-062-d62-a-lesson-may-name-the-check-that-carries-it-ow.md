<!-- dossier-format: 1 -->
# DEC-062 · D62: A lesson may name the check that carries it, owed or checked

- kind: decision
- status: approved
- created: 2026-11-27T20:10:32+01:00
- author: reyna
- via: reyna - claude
- approved: 2026-11-27T20:10:32+01:00 by reyna
- valid-from: 2026-11-27T20:10:32+01:00
- tags: lesson, check, test, status
- topic: entries, approval
- code: src/cmd/learn.rs
- related: DEC-039
- history: 2026-11-27T20:10:32+01:00 reyna via reyna - claude: created
- history: 2026-11-27T20:10:32+01:00 reyna via reyna - claude: approved
- history: 2026-11-27T20:10:36+01:00 reyna via reyna - claude: changed — related DEC-039

## Decision

A lesson may carry the field check: the test, lint, hook or review step that catches the mistake the lesson is about, and check-state: owed (no such check yet, or not shown to catch it) or checked. learn takes --check and writes the state owed; the new command checked sets the state checked with --by, how it was shown that the check fails on the mistake, and is a person's decision carried out with --said-by (D57). status counts the lessons whose check is owed. dossier only points to the check; it never writes or runs one.

## Reason

A lesson in a brief is the weakest way to stop a mistake; a test or a hook stops it every time (the reason of D39). Naming the check, and showing which lessons still have none, turns lessons into checks over time. Idea weighed from the study of the public toppingout logbook on 2026-11-27; only the idea is taken, no text and no code.

## Rejected

dossier writing or running the check: code and tests belong to the project (D38 in spirit, D7). A check on every lesson: many lessons are judgement and have none. An LLM setting checked by itself: that the mistake is caught is a claim a person must stand for (D5).
