<!-- dossier-format: 1 -->
# TOOL-003 · Release workflow

- kind: tool
- status: draft
- created: 2026-11-25T07:34:45+01:00
- author: reyna
- via: reyna - claude
- type: command
- location: .github/workflows/release.yml
- code: .github/workflows/release.yml
- topic: release
- related: DEC-017, TOOL-002
- tags: release, github, ci
- history: 2026-11-25T07:34:45+01:00 reyna via reyna - claude: created
- history: 2026-11-25T20:40:58+01:00 reyna via reyna - claude: changed — field code
- history: 2026-11-25T20:45:14+01:00 reyna via reyna - claude: changed — field topic
- history: 2026-11-25T21:45:11+01:00 reyna via reyna - claude: changed — related DEC-017
- history: 2026-11-25T21:45:11+01:00 reyna via reyna - claude: changed — related TOOL-002
- history: 2026-11-27T19:12:38+01:00 reyna via reyna - claude: changed — field tags

## Purpose

Builds the Linux and Windows programs and publishes the GitHub release when a v* tag is pushed.
