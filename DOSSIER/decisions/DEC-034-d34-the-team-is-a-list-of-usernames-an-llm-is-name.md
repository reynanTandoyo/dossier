<!-- dossier-format: 1 -->
# DEC-034 · D34: The team is a list of usernames; an LLM is named <username> - <anything>

- kind: decision
- status: approved
- created: 2026-11-25T07:33:28+01:00
- author: reyna
- via: reyna - claude
- approved: 2026-11-25T07:33:28+01:00 by reyna
- valid-from: 2026-11-25T07:33:28+01:00
- supersedes: DEC-014
- code: src/names.rs
- topic: team
- related: DEC-028
- history: 2026-11-25T07:33:28+01:00 reyna via reyna - claude: created
- history: 2026-11-25T07:33:28+01:00 reyna via reyna - claude: approved
- history: 2026-11-25T20:40:56+01:00 reyna via reyna - claude: changed — field code
- history: 2026-11-25T20:45:13+01:00 reyna via reyna - claude: changed — field topic
- history: 2026-11-25T21:45:13+01:00 reyna via reyna - claude: changed — related DEC-028

## Decision

The team is a list of usernames (the operating-system login of each member), plus a separate list of plain names: people who are not users (an external colleague, a customer contact) and can be named as author, source or `--said-by`, but never write themselves. An LLM is named `<username> - <anything>`, for example `tony - claude opus 5.5 high`; LLM names are not registered. Without `--agent` or `DOSSIER_AGENT` the writer is the operating-system username. `hooks install` sets `DOSSIER_AGENT` to `<username> - claude` in the user's personal Claude Code settings (`.claude/settings.local.json`), so that Claude's writes are always marked as an LLM's. Names registered with `people add --llm` keep working. Replaces the registration of LLM names of D13. (Owner, 2026-11-21.)

## Reason

Simple to set up: nobody keeps a list of LLM names, and a person's own commands need no name at all.
