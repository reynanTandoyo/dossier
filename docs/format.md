# dossier file format, version 1

This document fixes the files of a project dossier as built in phase 1 (v0.1): the four kinds
decision, assumption, finding and lesson, the settings file and the state folder. Later phases
add kinds and files and raise the version when a file changes its meaning.

## 1. Example

```markdown
<!-- dossier-format: 1 -->
# DEC-012 · Use transient runs for the last three speedline points

- kind: decision
- status: approved
- created: 2026-11-20T14:03:27+01:00
- author: anna
- via: anna-claude
- approved: 2026-11-20T14:03:27+01:00 by anna
- valid-from: 2026-11-20T14:03:27+01:00
- task: T-042
- machine: axial compressor
- analysis: steady RANS, speedline
- history: 2026-11-20T14:03:27+01:00 anna via anna-claude: created
- history: 2026-11-20T14:03:27+01:00 anna via anna-claude: approved

## Decision

The last three points of every speedline are run transient.

## Reason

Steady runs converged at mass flows where the transient run already showed rotating stall.

## Rejected

Steady runs with a finer mesh: same result, three times the cost.
```

An assumption, a finding that is a dead end, and a lesson drafted by an LLM and not yet approved:

```markdown
<!-- dossier-format: 1 -->
# ASM-003 · Inlet profile is uniform

- kind: assumption
- status: approved
- created: 2026-11-20T09:12:05+01:00
- author: ben
- approved: 2026-11-20T09:12:05+01:00 by ben
- valid-from: 2026-11-20T09:12:05+01:00
- state: open
- task: T-040
- history: 2026-11-20T09:12:05+01:00 ben: created
- history: 2026-11-20T09:12:05+01:00 ben: approved

## Assumption

The total pressure and temperature at the inlet are uniform over the span.

## Reason

No measured profile was delivered.

## Affects

T-042, T-043
```

```markdown
<!-- dossier-format: 1 -->
# FND-007 · Halving the time step did not remove the oscillation

- kind: finding
- status: approved
- created: 2026-11-21T16:40:11+01:00
- author: anna
- via: anna-claude
- approved: 2026-11-21T16:40:11+01:00 by anna
- valid-from: 2026-11-21T16:40:11+01:00
- outcome: dead end
- task: T-042
- history: 2026-11-21T16:40:11+01:00 anna via anna-claude: created
- history: 2026-11-21T16:40:11+01:00 anna via anna-claude: approved

## Observation

The mass flow still oscillated by 0.4 % with half the time step. The cause is not the time step.

## Evidence

Runs 12 and 13, monitor plots in results/monitors/.
```

```markdown
<!-- dossier-format: 1 -->
# LES-004 · Steady mixing-plane runs overpredict stall margin

- kind: lesson
- status: draft
- created: 2026-11-21T17:02:44+01:00
- author: anna
- via: anna-claude
- confidence: simulated
- task: T-042
- machine: axial compressor
- analysis: steady RANS, speedline
- history: 2026-11-21T17:02:44+01:00 anna via anna-claude: created

## Statement

Near the stability limit, steady runs with a mixing-plane interface converged at mass flows
where the transient run already showed rotating stall.

## Valid for

Transonic front stages, tip Mach number above 1.1. Not checked for subsonic rear stages.

## Evidence

Comparison of steady and transient speedlines, report section 4.2.
```

## 2. Text and lines

- Files are UTF-8. A byte-order mark at the start is accepted and kept.
- A line ends with `LF` or `CRLF`. Each line keeps its own ending. Lines the tool adds end with
  `LF`, or with `CRLF` when the line before them ends with `CRLF`.
- The tool changes whole lines and keeps every other byte.

## 3. Structure of an entry file

In this order:

1. **Format line**: `<!-- dossier-format: 1 -->`. A file without one is read as version 1. A
   higher number than the tool knows is refused (`TOO_NEW`).
2. **Heading**: `# <ID> · <title>`. The separator is space, middle dot (U+00B7), space. When
   reading, ` - ` and `: ` are accepted too. The tool writes ` · `.
3. **Field block**: lines `- <key>: <value>`. It starts at the first such line after the heading
   and ends at the first line that is not one. A key is lower-case letters, digits and `-`.
   Unknown keys are kept as written.
4. **Sections**: `## <Name>` followed by free text up to the next `## ` line or the end of the
   file. Unknown sections are kept as written. The body of a section is its text without the
   blank lines at its start and end.

Blank lines between the parts are kept. A new file has one blank line after the heading, after
the field block, after each section heading and after each section body.

## 4. IDs and file names

- An ID is a prefix and a number of at least three digits: `DEC-012`.

  | Kind | Prefix | Folder |
  |---|---|---|
  | decision | `DEC` | `decisions/` |
  | assumption | `ASM` | `assumptions/` |
  | finding | `FND` | `findings/` |
  | lesson | `LES` | `lessons/` |

- The file is `<folder>/<ID>-<slug>.md`. The slug is made from the title: lower case; every
  character that is not an ASCII letter or digit becomes `-`; several `-` become one; `-` at the
  start and end is removed; cut to 50 characters (and a trailing `-` removed again); an empty
  slug becomes `entry`.
- When reading, the ID in the heading counts. A file whose name starts with a different ID is a
  problem that `check` reports.

## 5. Fields

Written by the tool, in this order, each at most once (except `history`):

| Field | Value | Set by |
|---|---|---|
| `kind` | `decision`, `assumption`, `finding`, `lesson` | creation |
| `status` | `draft`, `approved`, `rejected`, `superseded`, `retired` | the tool |
| `created` | time (§7) | creation |
| `author` | the person | creation |
| `via` | the LLM that wrote it; left out when a person wrote it | creation |
| `approved` | `<time> by <person>`; left out until approved | approval |
| `valid-from` | time of approval | approval |
| `valid-until` | time it was superseded, retired or, for an assumption, found wrong | the tool |
| `state` | assumptions only: `open`, `confirmed`, `wrong` | creation, later commands |
| `confidence` | lessons only: `measured`, `simulated`, `experience`, `guess` | the writer |
| `outcome` | findings only, optional: `dead end` | the writer |
| `task` | the `todo` task it came from, such as `T-042` | the writer |
| `machine`, `analysis`, `software`, `tags` | comma-separated lists | the writer |
| `related` | comma-separated IDs | `link` |
| `supersedes`, `superseded-by` | one ID | `supersede` |
| `retired-reason`, `rejected-reason` | free text | `retire`, `reject` |
| any other key | kept as written | — |
| `history` | one line per event (§6); always last in the block | the tool |

A field with an empty value is not written. The options of the write commands can't set
`status`, `created`, `author`, `via`, `approved`, `valid-from`, `valid-until` or `history`.

## 6. History lines

`- history: <time> <person>[ via <llm>]: <event>[ — <detail>]`

Events: `created`, `approved`, `rejected`, `changed`, `superseded`, `retired`, `reopened`,
`restored`. The detail names what changed, for example `changed — field machine` or
`superseded — by DEC-019`. History lines are only ever added, never changed or removed.

## 7. Times

Local time to the second with UTC offset: `2026-11-20T14:03:27+01:00`. When the environment
variable `DOSSIER_NOW` holds a time in this form, the tool uses it instead of the clock; tests
rely on this.

## 8. Sections per kind

| Kind | Sections, in this order | Required |
|---|---|---|
| decision | `Decision`, `Reason`, `Rejected` | `Decision`, `Reason` |
| assumption | `Assumption`, `Reason`, `Affects`, `Confirmation` | `Assumption`, `Reason` |
| finding | `Observation`, `Evidence` | `Observation` |
| lesson | `Statement`, `Valid for`, `Evidence` | all three, and the field `confidence` |

A draft may lack required parts; it can't be approved until they are there.

## 9. The dossier folder

```
DOSSIER/
  dossier.toml
  decisions/  assumptions/  findings/  lessons/
  .state/
    write.lock        target of the operating-system lock; stays empty
    holder.json       who holds the lock: name, command, process ID, start time
    ids.json          highest number ever given out per prefix: {"DEC":12,"ASM":3}
    journal.jsonl     one JSON object per change (§11)
    approved.json     checksum of each approved lesson: {"LES-031":"<sha256 hex>"}
    history/<ID>/000001.md …   snapshots of an entry before each change
    history/<ID>/index.jsonl   one line per snapshot: {"n":1,"time":"…","name":"…","command":"…"}
```

## 10. dossier.toml

```toml
format = 1
code = "P-017"

[tags]
machine = ["axial compressor"]
analysis = ["steady RANS"]
software = []

[[people]]
name = "anna"
llms = ["anna-claude", "anna-codex"]

[limits]
brief_lines = 60
status_lines = 60
draft_days = 14
lock_wait_seconds = 30
ssh_timeout_seconds = 10
```

`format` and `code` are required. Everything else may be left out and then has the value shown
(`people` and the tag lists are empty). An unknown key is an error.

## 11. journal.jsonl

One line per change:

```json
{"time":"2026-11-20T14:03:27+01:00","name":"anna-claude","person":"anna","llm":true,"command":"decide","id":"DEC-012","event":"created","detail":""}
```

## 12. On a PC

- `.dossier-connect` in a connected folder: `server = "anna@sim01"`, `folder = "/projects/P-017"`,
  `code = "P-017"` (TOML).
- The per-user register `projects.toml` (`%APPDATA%\dossier\` on Windows,
  `$XDG_CONFIG_HOME/dossier/` or `~/.config/dossier/` elsewhere): one `[[project]]` table per
  project with `code`, `server`, `folder` and, if there is one, `local`.

## 13. Lenient reading, canonical writing

Read: any heading separator of §3, a missing format line, `CRLF`, a byte-order mark, fields in
any order, blank lines anywhere between the parts. Written by the tool: exactly what §1 shows. The
tool never rewrites a line it has no reason to change.
