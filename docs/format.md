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

---

# Phase 2 (v0.2) additions

Everything below is still format version 1: the files of phase 1 keep their meaning, and new
fields, statuses and files are added. Every rule of §2, §3, §6, §7 and §13 holds for the new
files too. Every change of every file carries who and when (D29).

## 14. New statuses, events and fields of entries

- Statuses for decisions: `proposed` (behaves like `draft`) and `declined` (behaves like
  `rejected`). An accepted proposal becomes `approved`; its `approved` field names the person who
  accepted it.
- History events added to §6: `proposed`, `accepted`, `declined`, `confirmed`, `refuted`,
  `cleared`, `picture`.
- Fields added to §5, in this order after `rejected-reason`: `declined-reason` (free text),
  `confirmed` and `refuted` (assumptions: `<time> by <person>`), `rests-on` (comma-separated IDs
  of wrong assumptions this entry rests on), `code` (comma-separated pointers `path` or
  `path::name`).
- A `code` pointer's path may be a folder (R-CODE-6): it then covers every file below it. A
  trailing `/` and a leading `./` are ignored.
- An assumption that is confirmed gets `state: confirmed`, `confirmed`, and the `--by` text as
  its *Confirmation* section. A refuted one gets `state: wrong`, `refuted`, `valid-until`, and the
  `--by` text as *Confirmation*.
- The *Affects* section of an assumption holds items separated by commas or line breaks. The
  tool recognises a task ID (`T-042`), an entry ID (`FND-004`), a value name (`design.massflow`,
  a name of §17 that exists) and an attached path (§18). Anything else is plain text.

## 15. Usage records

A fifth kind of entry: `usage`, prefix `USE`, folder `usage/`.

| Field | Value |
|---|---|
| `software` | name, free text in phase 2 |
| `version` | the version used |
| `task` | the task (required for a usage record) |
| `analysis` | as in §5 |
| `model` | stored path of the simulation file that was run, if any |
| `model-checksum` | its checksum (§18) at the time of the run |
| `howto` | an ID, if one was followed |

Sections, in this order: `Parameters` (one `name = value` per line; a value may be `{name}` of a
key value, §17), `Outputs` (one stored path per line), `Deviations`, `Notes`. Required:
`software`, `version`, `task`. A usage record is approved at once, whoever writes it.

```markdown
<!-- dossier-format: 1 -->
# USE-003 · ExampleSolver 2406 run of speedline point 7

- kind: usage
- status: approved
- created: 2026-12-02T10:15:00+01:00
- author: anna
- via: anna-claude
- approved: 2026-12-02T10:15:00+01:00 by anna
- valid-from: 2026-12-02T10:15:00+01:00
- task: T-042
- software: ExampleSolver
- version: 2406
- model: sim/stage1.sim
- model-checksum: sha256:9f2c…
- history: 2026-12-02T10:15:00+01:00 anna via anna-claude: created
- history: 2026-12-02T10:15:00+01:00 anna via anna-claude: approved

## Parameters

backpressure = 1.42 bar
massflow = {design.massflow}

## Outputs

results/point07/
```

## 16. Pictures

- A picture of entry `<ID>` is copied to `DOSSIER/images/<ID>/<file>`, where `<file>` is the slug
  (§4) of the original name without its extension, then `.png`, `.jpg` or `.svg`. A second
  picture with the same name gets `-2`, `-3`, … before the extension.
- The entry gets a section `Pictures` (after the kind's own sections) with one line per picture:
  `![<caption>](../images/<ID>/<file>)`. The caption is one line and required.
- Allowed: PNG, JPEG and SVG, recognised by their first bytes, at most 2 MB each and 10 per
  entry.

## 17. Block files

`values.md`, `files/<task>.md`, `exchange.md` and `map.md` are block files:

1. the format line of §3;
2. a heading `# <title>`;
3. blocks: a line `## <key>`, then field lines `- <name>: <value>` as in §3, the `history` lines
   last in the block. Lines that are neither are kept as written.

A new block is written as `## <key>`, a blank line, its field lines, a blank line. A block is
never deleted: an item that ends gets a field saying so (`detached`, below) and keeps its
history, so that what was true on a date (R-LIFE-9) can be read back.

In snapshots and the journal the whole file is one item with a fixed ID: `VALUES`, `MAP`,
`EXCHANGE`, and `FILES-<task>` for `files/<task>.md`. The journal `detail` names the block
(`design.massflow: set — 12.4 kg/s`).

### values.md

Title `Values of <code>`. One block per value; the key is the name (lower-case letters, digits,
`.`, `_` and `-`, starting with a letter, such as `design.massflow`).

| Field | Value |
|---|---|
| `value` | the value in force: a number, a range or a short text |
| `unit` | its unit; left out for a dimensionless value, which has `dimensionless: yes` |
| `source` | where it comes from: an attached path, a document, a person |
| `note` | free text |
| `key` | `yes` for a value that `brief` and `status` print first |
| `valid-from` | the time the value in force was set |
| `proposed` | `<value>[ <unit>] by <name> at <time>`: a change waiting for a person |

History lines: `<time> <who>: set — <value>[ <unit>]` for every value that came into force,
`proposed — <value>[ <unit>]`, `rejected — <value>[ <unit>]`, `changed — note` (or `source`,
`key`). The value in force on a date is the newest `set` line on or before it.

```markdown
<!-- dossier-format: 1 -->
# Values of P-017

## design.massflow

- value: 12.4
- unit: kg/s
- source: docs/spec-rev3.pdf
- key: yes
- valid-from: 2026-12-01T09:00:00+01:00
- history: 2026-11-20T10:00:00+01:00 anna: set — 12.1 kg/s
- history: 2026-12-01T09:00:00+01:00 anna: set — 12.4 kg/s
```

### files/<task>.md

Title `Files of <task>`. One block per attached path; the key is the stored path (R-XOS-2: relative
to the project folder, with `/`).

| Field | Value |
|---|---|
| `role` | `description`, `geometry`, `mesh`, `model`, `setup`, `script`, `result`, `report`, `reference`, `other`, or a role of `roles` in the settings |
| `type` | `file` or `folder` |
| `note` | free text |
| `size` | bytes (files) |
| `files` | number of files (folders) |
| `modified` | the file's modification time, format of §7 |
| `checksum` | `sha256:<hex>`, or for files over 1 GiB `short-sha256:<hex>` of the first and last 16 MiB and the size (R-FILE-6) |
| `made-from` | comma-separated `<stored path>@<checksum>` |
| `moved-from` | the old stored path, after `attach --moved` |
| `detached` | `<time> by <name>`: the file no longer belongs to the task |
| `rests-on` | as in §14 |

History events: `attached`, `reattached`, `moved`, `detached`, `cleared`.

### exchange.md

Title `Exchange of <code>`. One block per record; the key is `RCV-<n>` (received) or `DLV-<n>`
(delivered), numbered as entry IDs (ids.json holds `RCV` and `DLV`).

| Field | Value |
|---|---|
| `date` | the date of the exchange, `YYYY-MM-DD` |
| `from` / `to` | who sent or received it |
| `files` | comma-separated `<stored path>@<checksum>` |
| `based-on` | comma-separated task IDs and `<stored path>@<checksum>` |
| `note` | free text |

History events: `received`, `delivered`, `changed`.

### map.md

Title `Map of <code>`. One block per described folder or package; the key is the stored path.
Fields `purpose`, `start` (where to begin reading), `detached`. History events: `set`,
`changed`, `detached`.

## 18. Checksums of files outside the dossier

`sha256:<64 hex digits>` over the whole file, or for a file over 1 GiB
`short-sha256:<64 hex digits>` over the first 16 MiB, the last 16 MiB and the size as eight bytes
little-endian.

## 19. dossier.toml additions

```toml
deciders = ["anna"]          # D28; empty or missing: everyone listed decides
roles = ["postprocessing"]   # added to the roles of §17
code_project = false         # true after init --code-project
```

Top-level keys, before `[tags]`. All three may be left out.

---

# Phase 3 (v0.3) additions

Still format version 1. Every rule of §2 to §19 holds; every change carries who and when (D29).

## 20. The team dossier

A team dossier is a `DOSSIER` folder like a project's, made by `init --team-dossier --code <code>`
in the folder that holds it (that folder is a git repository, D36). Its `dossier.toml` has
`kind = "team"`. It has the folders `lessons/`, `howtos/` and `tools/` (no decisions,
assumptions, findings or usage records, R-TEAM-1), and these block files:

| File | Title | One block per | Fields |
|---|---|---|---|
| `projects.md` | `Projects of <code>` | project code | `folder` (the project folder on the server), `history` |
| `glossary.md` | `Glossary of <code>` | term | `meaning`, `symbol`, `unit`, `also` (comma list of other spellings), `history` |
| `usage-index.md` | `Usage of <code>` | `<project code>:<USE-ID>` | `date`, `code`, `software`, `version`, `analysis`, `machine`, `howto`, `person` |

A project dossier may have a `glossary.md` of its own, same format. `.state/.gitignore` holds the
two lines `write.lock` and `holder.json`.

## 21. People and names (D34)

`dossier.toml`, top-level keys before `[tags]`, in project and team dossiers:

```toml
members = ["tony", "anna"]          # usernames (operating-system logins) of the people
names = ["Dr. Weber", "Tom Example"] # plain names: not users, never write
```

A name that writes is resolved in this order: `--agent`, `DOSSIER_AGENT`, `TODO_AGENT`, then the
operating-system user (`USERNAME`, then `USER`). It is a person when it equals a member or a
`[[people]]` name (case ignored). It is an LLM when it has the form `<person> - <anything>` (the
first ` - ` splits it) or is an LLM registered with `people add --llm`. A project with a team
dossier also knows the team's members and plain names. `--said-by` and an author may be a person
or a plain name. In history lines an LLM is written as `<person> via <full LLM name>`.

## 22. How-tos and tools

| Kind | Prefix | Folder | Sections, in this order | Required |
|---|---|---|---|---|
| howto | `HOW` | `howtos/` | `Purpose`, `Steps`, `Settings`, `Traps`, `Tested with`, `Open remarks` | field `software`, `Purpose`, `Steps` |
| tool | `TOOL` | `tools/` | `Purpose`, `How to run`, `Inputs`, `Outputs`, `Traps`, `Tested with` | `Purpose` |

Tool fields: `type` (`script`, `command`, `software`, `repo`), `location` (optional: a path, a
command name or a URL), `software`, `language`, `checksum` (`sha256:…` of a script's file when
its location is a file that exists). Lessons, how-tos and tools are *guarded*: an LLM writes them
as drafts, a person's approval records their checksum, and a hand change after approval is
marked (R-LIFE-6).

`Open remarks` of a team how-to holds one line per reported deviation (R-USE-6):
`- <project code> <YYYY-MM-DD>: <deviation>`. The program adds these lines itself; adding one
records the approval checksum again when the how-to was unchanged since approval, so a remark is
not a hand change.

## 23. New entry fields

In this order after `code` (§14): `source` (`<project code>:<ID>` on a promoted team entry),
`promoted` (`team:<ID>` on the project entry), `promote` (`no` when kept in the project at
`close`), `imported` (`yes`), `source-document` (`<stored path>[ p. <page>]`), `review`
(`YYYY-MM-DD`), `reviewed` (`<time> by <person>`), then for tools `type`, `location`,
`language`, `checksum`.

## 24. Confidential terms

`confidential.md` in a project dossier, title `Confidential terms of <code>`, one block per term
with `history` (`added`, `removed`) and, after removal, `removed: <time> by <person>`. A block
with the key `(no confidential terms)` and the field `stated: <time> by <person>` records that
a person stated there are none. Matching ignores case and treats spaces, hyphens and underscores
as the same, and also matches when they are left out (`NT-450`, `nt 450`, `NT_450` and `NT450`
all match). A false match only stops a write until the text is reworded; a missed one would leak.

## 25. Finished projects

`dossier.toml` gains `past = true`, and `describes = "<folder>"` when the dossier was made with
`--in` somewhere else. `imports.md`, title `Documents of <code>`, one block per document (stored
path) with `state` (`reviewing`, `done`, `nothing`) and `history`. Entries drafted from a
document carry `imported: yes` and `source-document`; they are always drafts at first and keep
`imported: yes` for good.

## 26. dossier.toml additions

```toml
kind = "project"        # or "team"; missing means project
team = "/projects/TEAM/DOSSIER"   # the team dossier of a project
past = true             # a finished project brought in later
describes = "/archive/P-003"      # the project folder it describes, with --in
```

All top-level, before `[tags]`, all optional.

---

# Additions of 2026-11-22

Still format version 1. Every rule of §2 to §26 holds; every change carries who and when (D29).

## 27. Text that looks like a credential (R-CONF-7)

A line of a `.md` file of the dossier looks like it holds a credential when one of these holds:

| Kind | Rule |
|---|---|
| `private key` | the line contains `-----BEGIN ` and, after it, `PRIVATE KEY-----` |
| `access token` | a word of the line (cut at spaces and at `"`, `'`, `` ` ``, `(`, `)`, `,`, `;`, `=`, `:`) starts with one of `ghp_`, `gho_`, `ghu_`, `ghs_`, `ghr_`, `github_pat_`, `glpat-`, `xoxb-`, `xoxp-`, `xoxa-`, `sk-` and has at least 20 more characters of letters, digits, `_` and `-`; or starts with `AKIA` or `ASIA` and is exactly 20 capital letters and digits; or starts with `AIza` and has 39 characters; or starts with `eyJ` and has three parts of at least 10 characters separated by two `.` |
| `password` | the line contains one of the names `password`, `passwd`, `pwd`, `secret`, `token`, `api key`, `api-key`, `api_key`, `apikey` (case ignored, not directly after a letter or digit), then optional spaces and one optional `"` or `'`, then `=` or `:`, then a value: the rest of the line, trimmed and without surrounding quotes, is one word of at least 4 characters |

A password value is no hit when it starts with `<`, `{`, `[`, `$` or `%`, or consists only of
`*`, `x`, `X` and `.` (a placeholder or a masked value). Field lines of the tool itself never
match, because no field name of §5 contains one of the names.

`.state/credential-ok.json` is a JSON array of strings, sorted: the SHA-256 (hex) of each line,
trimmed, that a person allowed with `--allow-credential-like`. A line whose hash is in the list is
no hit, in any file. The journal gets one record per file: event `allowed`, detail
`credential-like text, line 12` (several lines: `line 12, 14`).

## 28. Investigations (D41)

An eighth kind of entry: `investigation`, prefix `INV`, folder `investigations/`. Sections, in
this order: `Problem` (required), `Attempts`, `Solution`. Fields: `state` (`open`, `solved`,
`given up`), `task`, the tag fields, `closed` (`<time> by <person>`, while solved or given up)
and `given-up-reason`. An investigation is approved at once, whoever writes it, and exists only
in project dossiers. History events added to §6: `tried`, `solved`, `given up`.

The `Attempts` section holds one line per attempt, in the order they were recorded:

```
<n>. [<verdict>] <text>[ — did: <text>][ — result: <text>][ — run: <USE-ID>][ — task: <task>] (<time> <who>)
```

`<n>` counts from 1 and is never reused or changed. `<verdict>` is `failed`, `worked` or
`unclear`. `<who>` is written as in history lines (`anna` or `anna via anna - claude`). The parts
are separated by space, em dash (U+2014), space, and come in this order; a part that was not
given is left out. Each text is one line. Inside a text, `tried` writes an en dash (U+2013) in
place of the em dash of a separator form (` — did: `, ` — result: `, ` — run: `, ` — task: `),
so that the line reads back the same; every other em dash is kept as written. When reading, the last ` (` of the line starts the time
and who; a line of the section that does not have this form is kept as written and is no attempt.

```markdown
<!-- dossier-format: 1 -->
# INV-003 · Residual blows up after 40 iterations

- kind: investigation
- status: approved
- created: 2026-12-03T09:10:00+01:00
- author: anna
- via: anna - claude
- approved: 2026-12-03T09:10:00+01:00 by anna
- valid-from: 2026-12-03T09:10:00+01:00
- state: solved
- task: T-012
- closed: 2026-12-03T11:40:00+01:00 by anna
- history: 2026-12-03T09:10:00+01:00 anna via anna - claude: created
- history: 2026-12-03T09:10:00+01:00 anna via anna - claude: approved
- history: 2026-12-03T09:30:00+01:00 anna via anna - claude: tried — 1 failed
- history: 2026-12-03T10:15:00+01:00 anna via anna - claude: tried — 2 failed
- history: 2026-12-03T11:35:00+01:00 anna via anna - claude: tried — 3 worked
- history: 2026-12-03T11:40:00+01:00 anna via anna - claude: solved

## Problem

The density residual grows from iteration 40 on and the run stops with a negative pressure.

## Attempts

1. [failed] The CFL number is too high — did: CFL 5 to 1 — result: diverges at iteration 180 (2026-12-03T09:30:00+01:00 anna via anna - claude)
2. [failed] The mesh is too coarse at the leading edge — did: one refinement level more — result: same divergence (2026-12-03T10:15:00+01:00 anna via anna - claude)
3. [worked] The flux at the outlet has the wrong sign — did: corrected the sign in the outlet flux — result: converges in 900 iterations (2026-12-03T11:35:00+01:00 anna via anna - claude)

## Solution

The outlet flux had the wrong sign.

Attempts: 3
```

## 29. The hook after shell commands (R-AGENT-5)

`hooks install` adds to `.claude/settings.json`, next to the two entries of design.md §8:

```json
"PostToolUse": [
  { "matcher": "Bash|PowerShell",
    "hooks": [ { "type": "command", "command": "dossier after", "timeout": 30 } ] }
]
```

`dossier after` answers with nothing, or with one line of JSON that Claude Code adds to the
agent's context:
`{"hookSpecificOutput":{"hookEventName":"PostToolUse","additionalContext":"<text>"}}`.

---

# Phase 5 (v0.5) additions

## 30. Taken files

`taken.md` in a project dossier is a block file (§17), title `Taken files of <code>`, item ID
`TAKEN`. One block per file that was ever taken; the key is the stored path.

| Field | Value |
|---|---|
| `taken` | `<time> by <person>`, while the file is taken; removed when it is returned or abandoned |
| `task` | the task it was taken for |
| `size`, `modified`, `checksum` | of the file on the server when it was taken, and after `return --done` of the returned file (§18) |
| `forced` | `yes` while the take was made with `--force` over another person's |

History lines: `taken — <task> <checksum>`, `taken by force — <task> <checksum>, from <person>`,
`returned — <checksum>`, `abandoned`. The history is the log of the file's versions (R-FILE-5).

`dossier.toml` gains `take_days = 14` under `[limits]`: above this age a take is marked.

## 31. Versions between PC and server (R-XOS-9)

The PC sets two variables for every `dossier` command it runs on the server:
`DOSSIER_CLIENT_VERSION` (the program version, such as `0.5.0`) and `DOSSIER_CLIENT_FORMAT` (the
format version, `1`). A server that reads another format than its own refuses the command
(`TOO_NEW`) and says which side to update; another program version with the same format is a
warning. Without the variables (a command typed on the server) nothing is compared.

---

# Phase 6 (v0.6) additions

## 32. Digests (D22)

A ninth kind of entry: `digest`, prefix `DIG`, folder `digests/`. Section `Summary` (required, at
most 15 lines). Fields: `milestone` (the `todo` milestone it sums up) and `tasks` (comma-separated:
the milestone and every task below it when the digest was written), both required. A digest is
guarded like a lesson: an LLM writes it as a draft, a person approves it. A decision may carry
`standing: yes`: it binds the whole project and is never replaced by a digest.

## 33. The session review hook (R-DRAFT-9)

`hooks install --session-review "<command>"` adds to `.claude/settings.json`:

```json
"SessionEnd": [
  { "hooks": [ { "type": "command", "command": "dossier session-end --run '<command>'" } ] }
]
```

The material is written to `.claude/dossier-session-review.md` in the project folder, replaced at
the end of each session.

## 34. Session files (D44)

A session file holds what a person and an LLM said in one session, as plain text. It is what
`review-session` reads; any tool, or an LLM itself, can write one.

```markdown
<!-- dossier-session: 1 -->
# Session

- tool: claude code

## user

We go with the SST model because the test data is for attached flow.

## assistant

Recorded as DEC-012. The first run diverges at iteration 40.
```

1. **Format line**: `<!-- dossier-session: 1 -->`. A file without one is read as version 1. A
   higher number than the program knows is refused (`TOO_NEW`) with a message that says to update
   `dossier`. The number rises only when an older program could misread the file; new optional
   field lines do not raise it.
2. **Heading** `# Session`, optionally followed by more text on the line.
3. **Field lines** `- <key>: <value>`: `tool` (which program the session ran in), and any other
   key, which is kept and ignored.
4. **Turns**: a line `## user` or `## assistant`, then the text up to the next such line. A text
   line that itself starts with `## ` is written with one space in front. A `## ` heading with
   another name starts a block that is skipped.

`session convert --claude` and `review-session --claude` make this file from a transcript of
Claude Code; only `src/claude.rs` knows that format.

---

# Phase 7 (v1.0) additions

Still format version 1. Every rule of §2 to §34 holds; every change carries who and when (D29).

## 35. Topics (R-TOPIC-1 to R-TOPIC-3)

`dossier.toml` may hold a top-level key, written before the first table:

```toml
topics = ["solver", "mesh", "boundary conditions"]
```

A missing key is an empty list. A topic is one to three words in lower case, without commas.
An entry may carry the field `topic` with one or more topics, comma-separated, such as
`- topic: solver, mesh`. It is written directly after `tags` in the field order (§5). When the
list is not empty, every topic of an entry is on the list.

Version 0.6 refuses a `dossier.toml` that has `topics`, because it is an unknown key (§10). The
format number stays 1: a PC and a server on different program versions only get the warning of
R-XOS-9, and the refusal comes from the key, not from the number. An older program keeps an
entry's `topic` line as written (§3).

## 36. Entries shown in a session (R-CODE-8)

The folder `.state/shown/` holds one file per Claude Code session, `<session>.txt`, with one
entry ID per line: the entries that the PreToolUse hook `dossier guard` has shown in that session
(D51). The session name is used only when it consists of ASCII letters, digits and `-`. Files
older than 30 days are removed. `which <path> --binding --session <id>` leaves out the entries
listed in that session's file (R-CODE-8).
