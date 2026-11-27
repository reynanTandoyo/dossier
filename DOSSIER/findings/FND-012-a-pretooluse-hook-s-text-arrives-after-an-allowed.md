<!-- dossier-format: 1 -->
# FND-012 · A PreToolUse hook's text arrives after an allowed edit, a refusal's reason before it

- kind: finding
- status: approved
- created: 2026-11-25T20:10:46+01:00
- author: reyna
- via: reyna - claude
- approved: 2026-11-25T20:10:46+01:00 by reyna
- valid-from: 2026-11-25T20:10:46+01:00
- task: T-484
- code: src/cmd/guard.rs
- topic: hooks
- related: DEC-051
- tags: claude-code, hook, guard
- history: 2026-11-25T20:10:46+01:00 reyna via reyna - claude: created
- history: 2026-11-25T20:10:46+01:00 reyna via reyna - claude: approved
- history: 2026-11-25T20:45:14+01:00 reyna via reyna - claude: changed — field topic
- history: 2026-11-25T21:45:11+01:00 reyna via reyna - claude: changed — related DEC-051
- history: 2026-11-27T19:12:39+01:00 reyna via reyna - claude: changed — field tags

## Observation

A Claude Code PreToolUse hook reaches the model in two ways with different timing: additionalContext on an allowed edit arrives only together with the edit's result, after the file has changed; a refusal (exit code 2 or permissionDecision deny) with its reason reaches the model before the file changes, and the model reads it and retries. So 'before the edit' means refusing the first edit once.

## Evidence

Tests with Claude Code 2.1.331 and claude -p --model haiku on 2026-11-25, a scratch file with an invented rule; then end to end with dossier guard and DEC-001 of a scratch project: the first edit was refused, the model followed the rule and its retry went through.
