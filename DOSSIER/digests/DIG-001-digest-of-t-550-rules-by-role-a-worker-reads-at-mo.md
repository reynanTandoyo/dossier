<!-- dossier-format: 1 -->
# DIG-001 · Digest of T-550: Rules by role: a worker reads at most 15 lines and starts without the project brief

- kind: digest
- status: draft
- created: 2026-11-26T21:05:13+01:00
- author: reyna
- via: reyna - claude
- milestone: T-550
- tasks: T-550, T-551, T-552, T-553, T-554, T-555, T-556, T-557, T-558, T-559, T-560, T-561
- tags: agent-rules, worker, digest
- history: 2026-11-26T21:05:13+01:00 reyna via reyna - claude: created
- history: 2026-11-26T23:15:20+01:00 reyna via reyna - claude: approved
- history: 2026-11-27T19:12:38+01:00 reyna via reyna - claude: changed — field tags, back to draft

## Summary

Decided and still binding: D55, the rules for LLMs are split by role (common block of at most 8 lines, help worker of at most 7, help planner for the rest); D56, a worker session is marked by DOSSIER_ROLE=worker.
Built in version 1.3.0: the three rules texts with tests on their limits; brief prints one line in a worker session (--full prints the brief); the hook after todo commands is silent after todo done and todo stuck for a worker; agents install --yes replaces a 1.2 block and renews an existing skill file, and agents check covers both.
Measured: a worker reads 13 lines (about 280 tokens) instead of 38 (about 1,070); its session start is 1 line (about 30 tokens) instead of 60 (about 1,500). The planner reads 41 lines of rules and the unchanged brief.
Found: Haiku workers handled the refused first edit without help in 4 of 4 cases.
Not built: the program does not stop a worker from writing to the dossier; the rule is only in its text. A session is a planner session unless whoever starts it sets the variable.
Left over: one test name in tests/cmd_agents.rs still speaks of a code project block that leaves lines out; the block no longer differs by project.
