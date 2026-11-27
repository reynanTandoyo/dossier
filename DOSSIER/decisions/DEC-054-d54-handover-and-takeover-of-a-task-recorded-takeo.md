<!-- dossier-format: 1 -->
# DEC-054 · D54: Handover and takeover of a task, recorded, takeover by an LLM only with --said-by

- kind: decision
- status: approved
- created: 2026-11-26T06:51:03+01:00
- author: reyna
- via: reyna - claude
- approved: 2026-11-26T06:51:03+01:00 by reyna
- valid-from: 2026-11-26T06:51:03+01:00
- topic: todo,files
- tags: handover, takeover, todo, said-by
- history: 2026-11-26T06:51:03+01:00 reyna via reyna - claude: created
- history: 2026-11-26T06:51:03+01:00 reyna via reyna - claude: approved
- history: 2026-11-27T19:12:41+01:00 reyna via reyna - claude: changed — field tags

## Decision

A task is handed over with handover <task> --note, which gives it back in todo with a note of what is done and what is next, once the writer has returned every file taken for it. Anyone in the team takes over a task at any time with takeover <task>; it is recorded who took over from whom, and the files the previous holder has not returned are named. An LLM may hand over its own task, and takes over only with --said-by <person>. The handover note is a todo note on the task.

## Reason

A colleague who changes team or falls ill must not leave a task locked or its progress lost. The task belongs to todo (D38), so the note is a todo note; a small team trusts each other, so a takeover is recorded rather than restricted.
