<!-- dossier-format: 1 -->
# DEC-048 · D48: Finding by word stems and rare words

- kind: decision
- status: approved
- created: 2026-11-25T20:10:34+01:00
- author: reyna
- via: reyna - claude
- approved: 2026-11-25T20:10:34+01:00 by reyna
- valid-from: 2026-11-25T20:10:34+01:00
- task: T-431
- code: src/select.rs, src/cmd/search.rs
- topic: search
- related: DEC-012, DEC-049, DEC-063
- tags: search, stemming, ranking
- history: 2026-11-25T20:10:34+01:00 reyna via reyna - claude: created
- history: 2026-11-25T20:10:34+01:00 reyna via reyna - claude: approved
- history: 2026-11-25T20:45:13+01:00 reyna via reyna - claude: changed — field topic
- history: 2026-11-25T21:45:13+01:00 reyna via reyna - claude: changed — related DEC-012
- history: 2026-11-25T21:45:13+01:00 reyna via reyna - claude: changed — related DEC-049
- history: 2026-11-27T19:12:41+01:00 reyna via reyna - claude: changed — field tags
- history: 2026-11-27T20:10:36+01:00 reyna via reyna - claude: changed — related DEC-063

## Decision

Finding by word stems and rare words: wherever entries are chosen as related to a text, and in search, words are compared by their English word stem, and a word that few entries hold counts more than a common one (R-FIND-1 to R-FIND-4). This is still full-text search: no model is called and nothing is stored next to the entries, as D7 and D11 require.

## Reason

In the project's own dossier, search connection reuse found nothing although two entries are about it, and brief --task listed entries that shared only the word and with the task.
