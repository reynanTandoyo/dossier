<!-- dossier-format: 1 -->
# DEC-057 · D57: An LLM carries out the decision of its person with --said-by; no terminal is needed

- kind: decision
- status: approved
- created: 2026-11-26T21:40:49+01:00
- author: reyna
- via: reyna - claude
- approved: 2026-11-26T21:40:49+01:00 by reyna
- valid-from: 2026-11-26T21:40:49+01:00
- standing: yes
- topic: approval
- supersedes: DEC-007
- code: src/names.rs, src/cmd/approve.rs
- tags: said-by, approval
- history: 2026-11-26T21:40:49+01:00 reyna via reyna - claude: created
- history: 2026-11-26T21:40:49+01:00 reyna via reyna - claude: approved
- history: 2026-11-27T19:12:41+01:00 reyna via reyna - claude: changed — field tags

## Decision

A person decides what is approved, rejected, accepted, declined, confirmed, refuted, retired, removed, restored, reopened, cleared and reviewed, and which confidential term is removed. The person need not type the command: an LLM carries the decision out with `--said-by <its person>` after the yes of that person in the session, without a terminal and without the question the command asks a person; the entry and the journal name both. Without `--said-by` an LLM is refused as before, and in a worker session always. (Owner, 2026-11-26.)

## Reason

The owner works through an LLM and does not want to type commands at a terminal; the same was decided for recording decisions (decide --said-by), topics (D52) and takeover (D54). The check that D5 needs is the yes of the person in the session, and the journal records who said it and which LLM ran the command.

## Rejected

Keeping the terminal for approvals only: every draft would wait until the owner opens a terminal.
