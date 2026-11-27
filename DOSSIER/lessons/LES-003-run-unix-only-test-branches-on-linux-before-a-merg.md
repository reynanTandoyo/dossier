<!-- dossier-format: 1 -->
# LES-003 · Run Unix-only test branches on Linux before a merge, for example in WSL

- kind: lesson
- status: draft
- created: 2026-11-25T07:34:45+01:00
- author: reyna
- via: reyna - claude
- confidence: experience
- topic: release
- related: TOOL-004
- tags: linux, wsl, test
- history: 2026-11-25T07:34:45+01:00 reyna via reyna - claude: created
- history: 2026-11-25T20:45:14+01:00 reyna via reyna - claude: changed — field topic
- history: 2026-11-25T21:45:11+01:00 reyna via reyna - claude: changed — related TOOL-004
- history: 2026-11-27T19:12:38+01:00 reyna via reyna - claude: changed — field tags

## Statement

Run Unix-only test branches on Linux before a merge, for example in WSL

## Valid for

Tests with sh stand-ins or Unix-only cases, developed on Windows

## Evidence

T-313, T-315, T-321 and T-374 had branches that only run on Linux; all were run in FedoraLinux-44 before the merge
