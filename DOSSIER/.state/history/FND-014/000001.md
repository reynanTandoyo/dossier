<!-- dossier-format: 1 -->
# FND-014 · Haiku workers handle the refused first edit of a file without help (4 of 4 cases)

- kind: finding
- status: approved
- created: 2026-11-26T21:05:13+01:00
- author: reyna
- via: reyna - claude
- approved: 2026-11-26T21:05:13+01:00 by reyna
- valid-from: 2026-11-26T21:05:13+01:00
- task: T-550
- topic: hooks
- code: src/cmd/guard.rs
- history: 2026-11-26T21:05:13+01:00 reyna via reyna - claude: created
- history: 2026-11-26T21:05:13+01:00 reyna via reyna - claude: approved

## Observation

Haiku workers handle the refused first edit without help: in 4 of the 7 worker tasks of T-550 (T-554, T-558, T-559, T-560) the first edit of a file was refused with the entries in force for it; each worker read or weighed the entries and made the same edit again. The refusal for Cargo.toml named DEC-017, which the version bump of a release does not touch.
