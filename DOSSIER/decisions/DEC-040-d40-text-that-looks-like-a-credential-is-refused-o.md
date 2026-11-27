<!-- dossier-format: 1 -->
# DEC-040 · D40: Text that looks like a credential is refused on every write

- kind: decision
- status: approved
- created: 2026-11-25T07:33:28+01:00
- author: reyna
- via: reyna - claude
- approved: 2026-11-25T07:33:28+01:00 by reyna
- valid-from: 2026-11-25T07:33:28+01:00
- task: T-284
- code: src/secret.rs
- topic: entries
- related: FND-007
- tags: credential
- history: 2026-11-25T07:33:28+01:00 reyna via reyna - claude: created
- history: 2026-11-25T07:33:28+01:00 reyna via reyna - claude: approved
- history: 2026-11-25T20:40:56+01:00 reyna via reyna - claude: changed — field code
- history: 2026-11-25T20:45:13+01:00 reyna via reyna - claude: changed — field topic
- history: 2026-11-25T21:45:11+01:00 reyna via reyna - claude: changed — related FND-007
- history: 2026-11-27T19:12:40+01:00 reyna via reyna - claude: changed — field tags

## Decision

Text that looks like a credential (a private key, an access token, a password) is refused on every write, in a project dossier and in the team dossier. (Owner, 2026-11-22.)

## Reason

Dossiers are committed to git (D36), where a credential stays in the history. How-tos about licence servers and ssh are where one would be written down. (Idea taken from the ECC project.)
