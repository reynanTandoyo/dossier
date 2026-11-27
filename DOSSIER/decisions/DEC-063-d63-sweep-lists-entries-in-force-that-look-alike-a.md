<!-- dossier-format: 1 -->
# DEC-063 · D63: sweep lists entries in force that look alike; a person decides what follows

- kind: decision
- status: approved
- created: 2026-11-27T20:10:32+01:00
- author: reyna
- via: reyna - claude
- approved: 2026-11-27T20:10:32+01:00 by reyna
- valid-from: 2026-11-27T20:10:32+01:00
- tags: sweep, duplicate, contradiction, link
- topic: search
- related: DEC-061, DEC-048
- history: 2026-11-27T20:10:32+01:00 reyna via reyna - claude: created
- history: 2026-11-27T20:10:32+01:00 reyna via reyna - claude: approved
- history: 2026-11-27T20:10:36+01:00 reyna via reyna - claude: changed — related DEC-061
- history: 2026-11-27T20:10:36+01:00 reyna via reyna - claude: changed — related DEC-048

## Decision

The new read-only command sweep lists the pairs of approved entries in force that look alike by word stems, tags and rare words (D48) and are not linked yet, best first, followed by rules for judging each pair. The LLM in the session reads the pairs and proposes to its person what follows: supersede, retire, link --as contradicts, link --as refines, link, or nothing. sweep changes nothing.

## Reason

Duplicates were caught only at the moment of writing (R-DRAFT-2, R-WRITE-11), so two entries that drifted together or apart were never looked at again. Listing pairs needs no model (D7), and judging them stays with a person (D5, D57). It also gives the pairs that the link-suggestion idea T-593 asked for. Idea weighed from the study of the public toppingout logbook on 2026-11-27; only the idea is taken, no text and no code.

## Rejected

Merging or superseding by itself: breaks D5. Detecting contradictions in the program: needs judgement, so the program lists pairs and the LLM in the session judges. A separate link --suggest command (T-593): the same pairs, so one command.
