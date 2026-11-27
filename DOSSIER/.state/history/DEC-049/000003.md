<!-- dossier-format: 1 -->
# DEC-049 · D49: Word stems come from rust-stemmers

- kind: decision
- status: approved
- created: 2026-11-25T20:10:34+01:00
- author: reyna
- via: reyna - claude
- approved: 2026-11-25T20:10:34+01:00 by reyna
- valid-from: 2026-11-25T20:10:34+01:00
- task: T-482
- code: src/select.rs::stem
- topic: search
- related: DEC-048
- history: 2026-11-25T20:10:34+01:00 reyna via reyna - claude: created
- history: 2026-11-25T20:10:34+01:00 reyna via reyna - claude: approved
- history: 2026-11-25T20:45:13+01:00 reyna via reyna - claude: changed — field topic
- history: 2026-11-25T21:45:13+01:00 reyna via reyna - claude: changed — related DEC-048

## Decision

Word stems come from the library rust-stemmers (the Snowball English stemmer, MIT/BSD licence); dossier carries no stemmer of its own.

## Reason

A tested stemmer handles English endings correctly; an own one would be smaller but make more mistakes, and finding is what version 1.0 is for.
