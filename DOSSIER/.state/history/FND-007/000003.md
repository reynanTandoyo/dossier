<!-- dossier-format: 1 -->
# FND-007 · GitHub secret scanning reports an invented AWS-shaped test value as a leaked key

- kind: finding
- status: approved
- created: 2026-11-25T07:34:44+01:00
- author: reyna
- via: reyna - claude
- approved: 2026-11-25T07:34:44+01:00 by reyna
- valid-from: 2026-11-25T07:34:44+01:00
- task: T-411
- topic: release
- related: DEC-040
- history: 2026-11-25T07:34:44+01:00 reyna via reyna - claude: created
- history: 2026-11-25T07:34:44+01:00 reyna via reyna - claude: approved
- history: 2026-11-25T20:45:14+01:00 reyna via reyna - claude: changed — field topic
- history: 2026-11-25T21:45:11+01:00 reyna via reyna - claude: changed — related DEC-040

## Observation

GitHub secret scanning reports an invented AWS-shaped test value as a leaked key

## Evidence

GitHub alert #1 on src/secret.rs, 2026-11-23; T-411 builds the values at run time
