<!-- dossier-format: 1 -->
# DEC-015 · D14: Agents learn the tool from one rules text; no MCP server

- kind: decision
- status: approved
- created: 2026-11-25T07:33:27+01:00
- author: reyna
- via: reyna - claude
- approved: 2026-11-25T07:33:27+01:00 by reyna
- valid-from: 2026-11-25T07:33:27+01:00
- code: src/agents/
- topic: hooks
- related: DEC-033
- history: 2026-11-25T07:33:27+01:00 reyna via reyna - claude: created
- history: 2026-11-25T07:33:27+01:00 reyna via reyna - claude: approved
- history: 2026-11-25T20:40:54+01:00 reyna via reyna - claude: changed — field code
- history: 2026-11-25T20:45:12+01:00 reyna via reyna - claude: changed — field topic
- history: 2026-11-25T21:45:12+01:00 reyna via reyna - claude: changed — related DEC-033

## Decision

Agents learn the tool from one rules text (`dossier help agents`, installed into `AGENTS.md`). Claude Code hooks are an extra. No MCP server.

## Reason

Same reasons as in `todo`: every agent can run a command, and hooks can only run commands.
