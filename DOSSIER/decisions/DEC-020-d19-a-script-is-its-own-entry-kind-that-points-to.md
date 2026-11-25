<!-- dossier-format: 1 -->
# DEC-020 · D19: A script is its own entry kind that points to the script file

- kind: decision
- status: superseded
- created: 2026-11-25T07:33:27+01:00
- author: reyna
- via: reyna - claude
- approved: 2026-11-25T07:33:27+01:00 by reyna
- valid-from: 2026-11-25T07:33:27+01:00
- valid-until: 2026-11-25T07:33:28+01:00
- superseded-by: DEC-031
- history: 2026-11-25T07:33:27+01:00 reyna via reyna - claude: created
- history: 2026-11-25T07:33:27+01:00 reyna via reyna - claude: approved
- history: 2026-11-25T07:33:28+01:00 reyna via reyna - claude: superseded — by DEC-031

## Decision

A script (a Java macro, a Python script) is an entry of its own kind that points to the script file and says what it does, for which software, and what it needs. The file itself lives in the team's script folder or repository.

## Reason

Scripts are reused more than anything else, and a script nobody can find or trust is rewritten. Code belongs where it can be versioned and run.
