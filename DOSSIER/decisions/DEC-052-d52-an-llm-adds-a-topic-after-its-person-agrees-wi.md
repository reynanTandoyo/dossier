<!-- dossier-format: 1 -->
# DEC-052 · D52: An LLM adds a topic after its person agrees, with --said-by

- kind: decision
- status: approved
- created: 2026-11-25T20:45:06+01:00
- author: reyna
- via: reyna - claude
- approved: 2026-11-25T20:45:06+01:00 by reyna
- valid-from: 2026-11-25T20:45:06+01:00
- topic: entries
- supersedes: DEC-050
- code: src/cmd/topics.rs
- history: 2026-11-25T20:45:06+01:00 reyna via reyna - claude: created
- history: 2026-11-25T20:45:06+01:00 reyna via reyna - claude: approved

## Decision

A person decides each topic of a project. An LLM that finds no fitting topic asks its person; with the person's yes it adds the topic itself with topics add --said-by <person>, and the journal names both. Replaces the reading of D50 that only a person may run topics add.

## Reason

The owner wants an LLM to carry out an agreed topic, as it records an agreed decision with decide --said-by; the list stays short because every topic still needs a person's yes.
