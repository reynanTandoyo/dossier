<!-- dossier-format: 1 -->
# DEC-061 · D61: A link may have a type: refines or contradicts

- kind: decision
- status: approved
- created: 2026-11-27T20:10:17+01:00
- author: reyna
- via: reyna - claude
- approved: 2026-11-27T20:10:17+01:00 by reyna
- valid-from: 2026-11-27T20:10:17+01:00
- tags: link, contradiction, refines, status
- topic: entries, search
- code: src/cmd/link.rs, src/links.rs
- related: DEC-053, DEC-063
- history: 2026-11-27T20:10:17+01:00 reyna via reyna - claude: created
- history: 2026-11-27T20:10:17+01:00 reyna via reyna - claude: approved
- history: 2026-11-27T20:10:36+01:00 reyna via reyna - claude: changed — related DEC-053
- history: 2026-11-27T20:10:36+01:00 reyna via reyna - claude: changed — related DEC-063

## Decision

link takes --as refines or --as contradicts next to the plain related link. The fields are refines, refined-by and contradicts. Linked entries are shown with their type, contradictions first, still one level deep and at most three (D53). Two entries in force that contradict each other are listed by status under Unsettled.

## Reason

A plain related link does not say that one entry narrows another or that two pull against each other, and a contradiction that cannot be settled yet had no place except retiring one entry. Idea weighed from the study of the public toppingout logbook on 2026-11-27; only the idea is taken, no text and no code.

## Rejected

A free list of link types: every new type needs a meaning in search, brief and status. An unlink command: a contradiction ends when one entry is superseded or retired. Refusing a write that contradicts: only a person judges that.
