<!-- dossier-format: 1 -->
# FND-005 · In a .cmd script, %1 inside a quoted prompt ends the quotes early because %1 kee

- kind: finding
- status: approved
- created: 2026-11-25T07:34:44+01:00
- author: reyna
- via: reyna - claude
- approved: 2026-11-25T07:34:44+01:00 by reyna
- valid-from: 2026-11-25T07:34:44+01:00
- task: T-427
- software: Windows cmd
- topic: hooks
- related: FND-006, FND-004
- history: 2026-11-25T07:34:44+01:00 reyna via reyna - claude: created
- history: 2026-11-25T07:34:44+01:00 reyna via reyna - claude: approved
- history: 2026-11-25T20:45:14+01:00 reyna via reyna - claude: changed — field topic
- history: 2026-11-25T21:45:12+01:00 reyna via reyna - claude: changed — related FND-006
- history: 2026-11-25T21:45:12+01:00 reyna via reyna - claude: changed — related FND-004

## Observation

In a .cmd script, %1 inside a quoted prompt ends the quotes early because %1 keeps its own quotes; %~1 gives the path without them

## Evidence

T-427, tested with a stand-in claude and a path with spaces
