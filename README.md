# dossier

`dossier` is a command-line program that keeps what a small engineering team learns and decides
while doing its projects. People and their LLMs write to it and read from it, so that neither has
to rely on memory.

**Status: v0.3 (phase 3)**

- [`docs/requirements.md`](docs/requirements.md): what the tool must do, and the six phases it is
  built in.
- [`docs/format.md`](docs/format.md): the exact format of the entry files.
- [`TODO.md`](TODO.md): the work plan for building the program.

## What it is for

Each project has a `DOSSIER` folder next to its tasks. The tasks are managed by
[`todo`](https://github.com/reynanTandoyo/todo) in `TODO.md`, and `dossier` manages the knowledge.
`dossier` writes entries, directly or as drafts that a person approves, and finds them again. It
prints the entries that matter for the session at hand, so that an LLM starts with the right
context.

### What version 0.1 does

- Records decisions, assumptions, findings (dead ends too) and lessons.
- Keeps the lessons an LLM writes as drafts until a person approves them.
- Lists and searches the entries.
- Prints `brief`, the context for a new session, and `status`, the state of the project.
- Keeps the history of every change. It shows and restores earlier versions.
- Runs from a Windows PC through ssh, after one `dossier connect`.
- Writes a backup archive with `pack`.

### What version 0.2 adds

Version 0.2 records the project's numbers, files and runs. So a result can be traced to what it
was made from. Each feature has one short example. The examples continue the
[first project](#a-first-project-in-ten-commands), and their dates are 20 and 21 November 2026.

First add ben, who works with the LLM `ben-claude`:

```sh
dossier people add ben --llm ben-claude
```

Then name who decides. Open `DOSSIER/dossier.toml` and add this line above `[tags]`:

```toml
deciders = ["anna"]
```

Now only anna decides. Without this line, everyone who is listed decides.

#### Key values

A key value is a number that results depend on, such as a massflow from a spec. It has a unit and
a source. Values marked `--key` come first in `brief` and `status`. A value comes into force when
a person sets it.

```sh
dossier value set design.massflow "12.1 kg/s" --source "Spec rev 2" --key
dossier value set design.pressure-ratio "1.85" --dimensionless --note "Design point"
dossier values
```

```
design.massflow  12.1 kg/s  (Spec rev 2, 2026-11-20)
design.pressure-ratio  1.85  (2026-11-20)
2 values
```

A value that an LLM sets waits for a person to approve it, unless the LLM names the person with
`--said-by`. A person at a terminal approves it:

```sh
dossier value set design.inlet-temperature "288 K" --agent ben-claude
dossier value approve design.inlet-temperature
```

```
design.inlet-temperature: proposed — 288 K (waiting for a person)
design.inlet-temperature: approved — 288 K
```

#### Files of a task

A task has files: a model, its results, a report. `attach` records each one with its role, such
as `model`, `result` or `report`. A result made from a model is marked `outdated` when the model
changes. A file over 1 GiB gets a short checksum: the first and last 16 MiB, and its size.

```sh
dossier attach T-042 sim/stage1.sim --role model --note "Stage 1 model, steady mixing plane"
dossier attach T-042 results/point07 --role result --from sim/stage1.sim --note "Speedline point 7"
dossier files --task T-042
```

```
model
  sim/stage1.sim  Stage 1 model, steady mixing plane  2026-11-19
result
  results/point07  Speedline point 7  2026-11-20
```

The date at the end of each line is the file's modification time (`modified` in format.md §17),
not the day the file was attached. The model was saved on 19 November and attached on 20
November, so its line shows 19 November.

On 21 November the mesh of the model was refined. The list keeps the date recorded at attach, and
marks the model as changed. Now `dossier files --task T-042` shows:

```
model
  sim/stage1.sim  Stage 1 model, steady mixing plane  2026-11-19  [changed since attached]
result
  results/point07  Speedline point 7  2026-11-20  [outdated]
```

`which` goes the other way. It shows the task and role of a file, and the results made from it:

```sh
dossier which sim/stage1.sim
```

```
T-042  model
made into: T-042 results/point07
```

`attach --moved` changes a path and keeps its role. `detach` takes a file off a task.

#### Simulation runs

`used` records one run of a simulation: the software and its version, the model file, the
parameters and the outputs. A parameter can name a key value, as `{design.massflow}` does here.
The outputs are attached to the task as results, made from the model.

```sh
dossier used ExampleSolver --version 2406 --task T-042 --model sim/stage1.sim --param backpressure=1.42 --param "massflow={design.massflow}" --output results/point07 --note "Speedline point 7"
```

```
USE-001: run recorded
```

The next run uses the changed model and a new backpressure. `used` records the model's new size,
time and checksum in the task's file list, so the model and the result are current again. No
`reattach` is needed after a run; `reattach` is for a file that changed outside a run:

```sh
dossier used ExampleSolver --version 2406 --task T-042 --model sim/stage1.sim --param backpressure=1.50 --param "massflow={design.massflow}" --output results/point07 --note "Speedline point 7, mesh 3.1 million cells"
dossier runs --task T-042
```

```
USE-002: run recorded
2026-11-21  anna  USE-002  ExampleSolver 2406  sim/stage1.sim
  changed: backpressure 1.42 -> 1.50
  outputs: results/point07
2026-11-20  anna  USE-001  ExampleSolver 2406  sim/stage1.sim
  outputs: results/point07
```

`dossier runs --compare USE-001 USE-002` lists each field and parameter that differs between two
runs, including the model's checksum.

#### Pictures

A picture goes into an entry with a caption. It can be a PNG, JPEG or SVG file of up to 2 MB, and
an entry can hold 10 pictures. The picture is copied into `DOSSIER/images/`.

```sh
dossier found "The speedline at point 7 is smooth" --evidence "results/point07"
dossier image add FND-002 plot.svg --caption "Speedline at point 7, steady mixing plane"
```

```
FND-002: added (approved)
FND-002: picture added: plot.svg
```

`dossier show FND-002` prints the entry with a *Pictures* section.

#### Proposals and deciders

With `deciders` set, a decision by anyone else is a proposal. A decider accepts or declines it at
a terminal. Only a decider's decision is approved at once. An LLM's decision names the person it
is for with `--said-by`, and it is a proposal unless that person is a decider. An LLM with nobody
to name uses `dossier propose`.

```sh
dossier decide "Run stage 1 at backpressure 1.50 as well" --reason "The stall margin is not known at this backpressure" --task T-042 --agent ben
dossier list --status proposed
dossier accept DEC-002
```

```
DEC-002: proposed (ben is not in deciders; a decider accepts it with: dossier accept DEC-002)
warning: related: DEC-001 Use the steady mixing-plane run for the speedline
DEC-002  decision  [proposed] Run stage 1 at backpressure 1.50 as well
1 entry
DEC-002: accepted
```

`accept` shows the proposal and asks for a yes or no first. `decline` turns a proposal down and
keeps the reason in the entry:

```sh
dossier decide "Keep the mesh at 3.1 million cells" --reason "The speedline is converged at that size" --said-by ben --agent ben-claude
dossier decline DEC-003 --reason "Already covered by the 3.1 million cell run"
```

```
DEC-003: proposed (ben is not in deciders; a decider accepts it with: dossier accept DEC-003)
DEC-003: declined
```

#### Confirming and refuting assumptions

An assumption is taken as true without proof. A person confirms it and says how it was checked. A
person at a terminal refutes it. Then everything that rests on it is marked and listed, and a
delivery based on it asks for a confirmation.

```sh
dossier assume "The rig reads the same mass flow as the model" --reason "The rig calibration is from last year" --affects FND-001
dossier confirm ASM-002 --by "measured on the rig, report 3"
dossier assume "The inlet pressure profile is flat" --reason "No inlet survey yet" --affects "T-042, design.massflow"
dossier refute ASM-003 --by "ben, from the inlet survey"
```

```
ASM-002: added (approved, open)
ASM-002: confirmed
ASM-003: added (approved, open)
ASM-003: refuted
USE-001: rests on ASM-003
DEC-002: rests on ASM-003
USE-002: rests on ASM-003
T-042 file sim\stage1.sim: rests on ASM-003
T-042 file results\point07: rests on ASM-003
values named: design.massflow (used by USE-001, USE-002)
```

`brief` shows the mark on each entry:

```
## Decisions in force
DEC-002 · Run stage 1 at backpressure 1.50 as well (ben, 2026-11-21) [rests on ASM-003]
...
```

When a person has checked that an entry does not depend on the wrong assumption, `cleared` removes
the mark. The note says why:

```sh
dossier cleared DEC-002 --assumption ASM-003 --note "The backpressure runs do not depend on the inlet profile"
```

```
DEC-002: cleared ASM-003
```

#### What was in force on a date

`--on <date>` works with `list`, `search`, `brief`, `status`, `values` and `files`. The date is
`YYYY-MM-DD`. The output shows what was in force on that day, not what is in force now.

```sh
dossier brief --on 2026-11-20
```

```
Project P-001
## Key values
design.massflow = 12.1 kg/s
design.inlet-temperature = 288 K
design.pressure-ratio = 1.85
## Decisions in force
DEC-001 · Use the steady mixing-plane run for the speedline (anna, 2026-11-20)
  Use the steady mixing-plane run for the speedline
  It converges at the design point
## Open assumptions
ASM-001 · The inlet total pressure is uniform
## Findings
FND-001 · The steady speedline stops at mass flow 12.1 kg/s
## Lessons
LES-001 · Steady mixing-plane runs overpredict the stall margin near the stability limit
```

#### The exchange with the customer

`received` records files that come from the customer. `delivered` records files that go out, and
it can name what they are based on: a task or a file. A record keeps the names and checksums of
its files. The files themselves are not copied into the dossier.

```sh
dossier received inlet-survey.txt --from "ben (the customer)" --task T-042 --note "Inlet survey from the customer"
dossier delivered results/point07/report.txt --to "ben (the customer)" --based-on T-042 --note "Speedline report, draft 1"
```

```
RCV-001: received 1 files from ben (the customer)
error: the delivery has warnings that a person at a terminal must confirm: T-042: sim/stage1.sim rests on a wrong assumption; T-042: results/point07 rests on a wrong assumption; T-042: USE-001 rests on a wrong assumption: ASM-003; T-042: USE-002 rests on a wrong assumption: ASM-003
```

This delivery rests on a refuted assumption, so a person at a terminal must confirm it. The
command shows the warnings and asks whether to deliver anyway. After a yes, it records the
delivery:

```
DLV-001: delivered 1 files to ben (the customer)
```

`dossier exchange` lists the records in time order:

```sh
dossier exchange
```

```
2026-11-20  RCV-001  from ben (the customer)
  inlet-survey.txt
2026-11-21  DLV-001  to ben (the customer)
  results/point07/report.txt
  based on T-042
2 records
```

#### The code map

A code project keeps a map of its code: what each folder is for, and where to start reading. Start
it with `init --code-project`. An entry can also point at code, with `code=<path>` or
`code=<path>::<name>`.

```sh
dossier init --code P-002 --code-project
dossier people add anna --llm anna-claude
dossier map set src/solver "Time loop and the solver core" --start src/solver/main.rs
dossier map set src/io "Reads the mesh and the boundary files"
dossier map
```

```
src/io  Reads the mesh and the boundary files
src/solver  Time loop and the solver core (start: src/solver/main.rs)
```

```sh
dossier found "The time loop writes the residuals every step" --evidence "src/solver/main.rs"
dossier set FND-001 --field "code=src/solver/main.rs::main"
dossier which src/solver/main.rs
```

```
not attached to any task
entries: FND-001 The time loop writes the residuals every step
```

`dossier map remove src/io` takes a folder out of the map. Its description stays in the file, with
its history.

#### Claude Code hooks and skill

Two Claude Code settings help. One starts each session with the brief. The other stops Claude from
editing the dossier by hand. Run these two commands in the project folder. Without `--yes`, each
one only says what it would change:

```sh
dossier hooks install --yes
dossier skill install --yes
```

```
hooks installed in .claude/settings.json
+ PreToolUse (Edit|Write|MultiEdit): dossier guard
+ SessionStart: dossier brief
.claude/skills/dossier/SKILL.md: skill written
```

The `SessionStart` hook runs `dossier brief` when a session starts. The `PreToolUse` hook runs
`dossier guard` before an edit. An edit to a file inside a `DOSSIER` folder is blocked, and the
message says to use the dossier commands. The skill file tells Claude how to use the commands.
`dossier hooks uninstall --yes` removes the two hooks.

### What version 0.3 adds

Version 0.3 shares what one project learned with the other projects of a team, without customer
data, and brings in the finished projects of the past. Each feature has one short example. The
examples start from new folders, so their IDs begin again. They use a team dossier, `TEAM`, and two
projects: `P-021`, which is running, and `P-009`, which is finished. Their dates are 22 and 23
November 2026.

#### The team dossier

The team dossier holds the lessons, how-tos, tools, glossary and usage index that all projects of
the team can read. Make it once, in a folder of its own:

```sh
dossier init --code TEAM --team-dossier
```

```
created the team dossier TEAM
```

Its members are the usernames of the people who may write to it. `init` makes the person who runs it
the first member. Here anna runs it, and `init` writes this line into `DOSSIER/dossier.toml`, above
`[tags]`:

```toml
members = ["anna"]
```

Then anna adds ben, and a plain name for someone who is not a user:

```sh
dossier team member add ben
dossier team name add "Dr. Weber"
```

```
added member ben
added name Dr. Weber
```

A project is linked to the team dossier with `team use`. Then a command with `--team` works on the
team dossier, and a command without it works on the project:

```sh
cd /projects/P-021
dossier init --code P-021
dossier people add anna --llm anna-claude
dossier people add ben --llm ben-claude
dossier team use ../TEAM
dossier team
```

```
team dossier: /projects/TEAM/DOSSIER
members: anna, ben
names: Dr. Weber
```

#### Names

A member's name is their operating-system username. An LLM is named `<person> - <anything>`, such as
`anna - claude opus 5.5 high`, and it needs no registration. A plain name can be an author or a
source, but it cannot write. `hooks install` names the LLM of Claude Code after the person, so the
entries that Claude Code writes are anna's:

```sh
dossier found "The outlet temperature is 2 K above the model" --evidence "results/point07.txt" --agent "anna - claude opus 5.5 high"
```

```
FND-002: added (approved)
```

The log names the LLM after the person. Its newest line, which `dossier log` prints first, is:

```
2026-11-22T09:00:00+01:00  anna via anna - claude opus 5.5 high  created  FND-002  The outlet temperature is 2 K above the model
```

A plain name is refused when it tries to write:

```sh
dossier found "A plain name tries to write" --evidence "results/point07.txt" --agent "Dr. Weber"
```

```
error: 'Dr. Weber' is a plain name in dossier.toml: it may be an author or a source, but it cannot write. Write as a member (an operating-system user) or as an LLM named <person> - <anything>
```

Anna runs `hooks install --yes` in the project folder, and Claude Code then writes as `anna - claude`:

```sh
dossier hooks install --yes
```

```
hooks installed in .claude/settings.json and DOSSIER_AGENT set in .claude/settings.local.json
+ PreToolUse (Edit|Write|MultiEdit): dossier guard
+ SessionStart: dossier brief
+ env DOSSIER_AGENT: anna - claude
```

#### Confidential terms

A project lists the names that identify its customer as confidential terms: the customer's name,
its products and machines, and the names of its people. Anyone in the project may add a term, also
an LLM. Only a person at a terminal removes one. Before anything is written from a project to the
team dossier, the whole text is checked against these terms, and the write is refused when a term
is found. A project with no terms must first say so, as a person, with
`dossier confidential none --yes`.

```sh
dossier confidential add "Example Customer"
dossier confidential list
```

```
Example Customer: added
Example Customer
1 term
```

`promote` runs the check before it copies an entry. LES-002 names the customer, so it is refused,
and the message gives each term with its line in the entry file:

```sh
dossier promote LES-002
```

```
error: confidential terms found: Example Customer (line 2); Example Customer (line 17)
```

#### Promoting and closing

`promote` copies an approved finding, lesson or how-to into the team dossier as a draft, with its
own team ID and a link to the project entry. A person approves the copy in the team dossier, with
`--team`. `approve` shows the copy in full and asks; the last lines are the result:

```sh
dossier promote LES-001
dossier approve LES-001 --team
```

```
LES-001: promoted as team:LES-001 (draft, waiting for approval in the team dossier)
LES-001: approved
```

At the end of a project, a person runs `dossier close` at a terminal. It shows each approved
finding, lesson and how-to that is not promoted yet, and asks for each one: `p` promotes it, `k`
keeps it in the project, and `l` decides later. Each answer is saved, so a second run asks only
about what is left:

```sh
dossier close
```

```
FND-001: promoted as team:LES-002
FND-002: left for later
LES-002: kept in the project
FND-003: kept in the project
FND-004: promoted as team:LES-003
2 promoted, 2 kept, 1 left for later
```

A promoted finding becomes a lesson, so FND-001 is LES-002 in the team dossier. That lesson cannot
be approved until it has *Valid for* and a confidence:

```sh
dossier approve LES-002 --team
```

```
LES-002: skipped
warning: LES-002 cannot be approved yet: missing section Valid for; missing field confidence
```

#### How-tos

A how-to says how a software is run for a kind of analysis: its purpose, steps, settings, traps and
what it was tested with. A person's how-to is approved at once, and an LLM's is a draft. `--team`
writes it to the team dossier, and `--review` sets the date to check it again:

```sh
dossier howto --software "ExampleSolver 2406" --title "Speedline with the steady mixing plane" --section "Purpose=Finds the speedline of a compressor stage with steady runs" --section "Steps=Run the steady case at each back pressure. Stop at the first point that does not converge." --section "Traps=Steady runs overpredict the stall margin near the stability limit." --machine "axial compressor" --analysis speedline --review 2027-06-01 --team
dossier list --team --kind howto --software "ExampleSolver 2406"
```

```
HOW-001: added (approved)
HOW-001  howto  Speedline with the steady mixing plane  — Finds the speedline of a compressor stage with steady runs
1 entry
```

In a project, `brief` prints the team's lessons and how-tos whose machine, analysis or software
tags match the project's. The tags are in `[tags]` of the project's `dossier.toml`. A project that
runs ExampleSolver 2406 has this line there:

```toml
software = ["ExampleSolver 2406"]
```

```sh
dossier brief
```

```
Project P-021
tags: software ExampleSolver 2406
...
## Team lessons and how-tos
team:HOW-001 · Speedline with the steady mixing plane
```

#### Tools

A tool is a script, a command, installed software or a git repository, with its purpose and how to
run it. For a script, the checksum of its file is recorded, so that a later change after approval
is marked. `tool update` records the checksum again after an intended change. A script's location
is relative to the folder that holds its dossier, so for a team tool the command runs in the team
folder, where the file `scripts/mesh_check.py` is:

```sh
cd /projects/TEAM
dossier tool add --title "Mesh check: cell count" --type script --location scripts/mesh_check.py --language Python --software "ExampleSolver 2406" --section "Purpose=Counts the cells of a mesh before a run"
```

```
TOOL-001: added (approved)
```

#### Where used

`used` records a run of a software in a project, as in version 0.2. With a team dossier, it also
adds one line to the team's usage index, without the title, the task or the files. `where-used`
reads that index:

```sh
cd /projects/P-021
dossier used ExampleSolver --version 2406 --task T-042 --output results/point07.txt --param "backpressure=1.42" --note "Speedline point 7"
dossier where-used ExampleSolver
```

```
USE-001: run recorded
2026-11-22  P-021  ExampleSolver 2406  -  -  anna
1 use
```

Each line gives the date, the project, the software and its version, the analysis and the machine,
and the person. The analysis and the machine are the project's tags when the run is recorded, and
`-` when there are none. `--analysis` and `--machine` keep only the uses whose analysis or machine
contains the text.

#### Glossary

The team dossier holds a glossary. Each term has a meaning, and it can have a symbol, a unit and
other spellings, such as a German word. `term` looks a word up in any of its spellings, and
`glossary` prints all the terms:

```sh
dossier term add "mass flow" "Mass of fluid per second through the machine" --symbol m-dot --unit kg/s --also "Massenstrom, mdot" --team
dossier term add "pressure ratio" "Outlet total pressure divided by inlet total pressure" --symbol PR --unit "-" --team
dossier term Massenstrom
dossier glossary --team
```

```
mass flow: added
pressure ratio: added
mass flow (m-dot, kg/s): Mass of fluid per second through the machine  [also: Massenstrom, mdot]  [team]
mass flow (m-dot, kg/s): Mass of fluid per second through the machine  [also: Massenstrom, mdot]  [team]
pressure ratio (PR, -): Outlet total pressure divided by inlet total pressure  [team]
2 terms
```

#### What is new

`status --since last` shows what was recorded since the person's last run, in the project and in the
team dossier. The first run lists everything recorded so far. Each run moves the person's marker to
now. `--keep` leaves the marker where it is, and an LLM's run never moves its person's marker.
`brief` ends with a line when there is news, as here, after a finding by ben on 23 November:

```
1 new entry since 2026-11-22: dossier status --since last
```

```sh
dossier status --since last
```

```
Project P-021
## Since 2026-11-22T11:00:00+01:00
## Project P-021
2026-11-23T10:00:00+01:00  ben  created  FND-003  Stage 2 rotor tip gap changes the stall margin
## Team
none
```

#### Review dates

An entry can have a review date, set with `--review`. When the date has passed, `brief`, `show` and
`check` mark the entry `[due for review]`. A person confirms that the entry still holds with
`reviewed`, and gives the next date:

```sh
cd /projects/TEAM
dossier howto --software "ExampleSolver 2406" --title "Mesh check before a speedline" --section "Purpose=Checks the mesh before the speedline runs" --section "Steps=Run the mesh check script, then the steady case." --review 2026-11-01
dossier check
dossier reviewed HOW-002 --next 2027-05-23
dossier check
```

```
HOW-002: added (approved)
error: DOSSIER/howtos/HOW-002-mesh-check-before-a-speedline.md:11: HOW-002 REVIEW due for review: review date 2026-11-01 has passed
DOSSIER: 1 problems
HOW-002: reviewed, next review 2027-05-23
DOSSIER: ok (0 problems)
```

#### Finished projects

A finished project is brought in from its own folder. `init --past` marks the folder as finished.
`review-folder` lists the documents in it, newest first, with the rules for drafting. Each document
is then marked as being reviewed with `review-document`, and the LLM drafts entries from it with
`--from-document`. A draft from a document is marked as imported, and a person approves it as usual.
`drafts --imported` lists these drafts by document. `reviewed-document` marks the document as done,
and `import status` shows how far the project has come:

```sh
cd /projects/P-009
dossier init --code P-009 --past
dossier people add anna --llm anna-claude
dossier review-folder
```

```
created DOSSIER for P-009
finished project: marked as past
next: dossier people add <person> --llm <name>
added anna with anna-claude
# Folder .

## Tree
reports/
  final-report.txt  93
scripts/
  old_check.py  19

## Documents
reports/final-report.txt  report  93  2026-11-22  [not reviewed]
scripts/old_check.py  script  19  2026-11-22  [not reviewed]

## Already reviewed
0 of 2 documents reviewed (done or nothing), 0 being reviewed.

## Import rules
...
```

```sh
dossier review-document reports/final-report.txt --agent anna-claude
dossier learn "The rig stalls above a tip Mach number of 1.1" --valid-for "Single-stage test rigs with a tip Mach number above 1.1" --evidence "Final report, page 1" --confidence measured --from-document reports/final-report.txt --agent anna-claude
dossier approve LES-001
dossier reviewed-document reports/final-report.txt
dossier import status
```

```
reports/final-report.txt: reviewing
import rules:
- Draft only what a later project would use.
...
draft with: --from-document reports/final-report.txt
LES-001: added (draft, waiting for approval)
LES-001: approved
reports/final-report.txt: done
Import status of P-009 (folder .)
Documents found: 2
  not reviewed: 1
  being reviewed: 0
  done: 1
  nothing to take: 0
Imported entries:
  drafts waiting for approval: 0
  approved: 1
  promoted to the team: 0
```

An imported entry is promoted like any other entry, after the project's confidential terms are
stated:

```sh
dossier confidential add "Example Customer"
dossier team use ../TEAM
dossier promote LES-001
```

```
Example Customer: added
LES-001: promoted as team:LES-004 (draft, waiting for approval in the team dossier)
```

### Not in version 0.3

These come in later phases. See [`docs/requirements.md`](docs/requirements.md), section 11.

- **Version 0.3.1:** text that looks like a password, key or access token is refused on every
  write.
- **Phase 4 (v0.4):** investigations: a problem, the attempts to solve it in order, and what
  solved it.
- **Phase 5 (v0.5):** copying large simulation files to a PC and back.
- **Phase 6 (v0.6):** milestone digests, session review, hooks after `todo` commands, and export.

## What it needs

- [`todo`](https://github.com/reynanTandoyo/todo) 0.8 or newer. `todo` keeps the tasks of a
  project in `TODO.md`. `brief --task`, `review-task` and `dossier todo` need it. `status` works
  without it: the tasks are one line, `tasks: todo is not installed`, and the task parts are left
  out. The other commands work without it.
- `ssh`, to use a dossier on a server from a Windows PC. Windows 10 and 11 have an OpenSSH client
  (`ssh`). Check it with `ssh -V`. The server must accept sign-in with a key.

## Install

Each release has two programs: `dossier-linux-x64` for the server and `dossier-windows-x64.exe`
for each PC. Download them from the
[release page](https://github.com/reynanTandoyo/dossier/releases). Each download has a `.sha256`
file with its checksum.

You do not need administrator rights. Copy the program into a folder that is on your `PATH`.

### On the server (Linux)

```sh
cd ~/Downloads   # the folder where you saved the two files
sha256sum -c dossier-linux-x64.sha256
mkdir -p ~/bin
cp dossier-linux-x64 ~/bin/dossier
chmod +x ~/bin/dossier
export PATH="$HOME/bin:$PATH"
dossier --version
```

Add the `export` line to `~/.bashrc` to keep it.

### On each PC (Windows)

In PowerShell:

```powershell
New-Item -ItemType Directory -Force -Path "$HOME\bin" | Out-Null
Copy-Item "$HOME\Downloads\dossier-windows-x64.exe" "$HOME\bin\dossier.exe"
$userPath = [Environment]::GetEnvironmentVariable("Path", "User")
[Environment]::SetEnvironmentVariable("Path", "$userPath;$HOME\bin", "User")
```

Open a new PowerShell window, then check the program:

```powershell
dossier --version
```

## A first project in ten commands

Run these on the server. The examples use a person, `anna`, and the LLM that works for anna, `anna-claude`. Use your
own names. Every name must be listed in the project before it can write.

Go to the project folder (create it first if it is new), and set your name:

```sh
cd /projects/P-001
export DOSSIER_AGENT=anna
```

Then run these ten commands:

```sh
dossier init --code P-001
dossier people add anna --llm anna-claude
dossier agents install --yes
dossier decide "Use the steady mixing-plane run for the speedline" --reason "It converges at the design point" --rejected "Transient run: too slow for this study"
dossier found "The steady speedline stops at mass flow 12.1 kg/s" --evidence "Run 07, speedline table"
dossier assume "The inlet total pressure is uniform" --reason "No inlet survey is available yet"
dossier learn "Steady mixing-plane runs overpredict the stall margin near the stability limit" --valid-for "Transonic front stages, tip Mach number above 1.1" --evidence "Speedline comparison in this project, section 4.2" --confidence simulated --agent anna-claude
dossier drafts
dossier approve LES-001
dossier brief
```

What each command does:

- `init` creates the `DOSSIER` folder and its settings.
- `people add` lists anna and the LLM `anna-claude`.
- `agents install` writes the rules for LLMs into `AGENTS.md`. See [For LLMs](#for-llms).
- `decide`, `found` and `assume` are approved at once, because anna wrote them.
- `learn` is run by anna's LLM (`--agent anna-claude`). A lesson from an LLM is a draft.
- `drafts` lists the drafts.
- `approve` shows a draft in full. Then it asks whether to approve, reject or skip it. Only a
  person at a terminal can approve. Without a terminal, it refuses.
- `brief` prints what a new session needs to know. The output looks like this:

```
Project P-001
## Decisions in force
DEC-001 · Use the steady mixing-plane run for the speedline (anna, 2026-11-21)
  Use the steady mixing-plane run for the speedline
  It converges at the design point
## Open assumptions
ASM-001 · The inlet total pressure is uniform
## Findings
FND-001 · The steady speedline stops at mass flow 12.1 kg/s
## Lessons
LES-001 · Steady mixing-plane runs overpredict the stall margin near the stability limit
```

### Options

These options work with every command:

- `--agent <name>`: the name that writes. It does the same as `DOSSIER_AGENT`. A name that the
  project does not list is refused, and the message lists the names it knows.
- `--dry-run`: for a write, shows what would be written, and changes nothing.
- `--json`: prints one JSON object instead of text. Scripts and LLMs can read it.

For example, to see a change before it is made:

```sh
dossier set FND-001 --section "Evidence=Run 07, speedline table, page 3" --dry-run
```

```
FND-001: changed
warning: dry run: would write DOSSIER/findings/FND-001-the-steady-speedline-stops-at-mass-flow-12-1-kg-s.md
```

### Exit codes

| Code | Meaning |
|---|---|
| `0` | Done. |
| `1` | Refused by a rule, or problems found (for example by `check`). Read the message. Do not repeat the command unchanged. |
| `2` | Wrong usage, such as a missing argument. |
| `3` | Busy: another command holds the lock, or a file changed while the command ran. Run the same command again. |
| `4` | A file or settings problem, such as no `DOSSIER` folder here. |
| `5` | A needed program or connection failed: `todo` is missing, or ssh could not reach the server. |

### For LLMs

- `dossier help agents` prints the rules that an LLM follows in a dossier project.
- `dossier agents install --yes` writes the same rules into `AGENTS.md` in the project folder,
  between two markers. Every other line stays as it is. `dossier agents check` says whether the
  rules are current.
- Entries are written in English, also when the session is in another language.

## How it works with todo

- Tasks live in `TODO.md`, and `todo` manages them. Knowledge lives in the `DOSSIER` folder, and
  `dossier` manages it. Neither program changes the other's files.
- An entry names its task with `--task`, such as `--task T-042`. `dossier` stores the name. It
  does not check the name against `TODO.md` when it writes the entry.
- `dossier brief --task T-042` prints what matters for one task; it needs `todo`. `dossier status`
  shows the tasks and the knowledge together; without `todo` it leaves the task parts out.
- `dossier todo <arguments>` runs `todo` in the project folder. For example,
  `dossier todo show T-042`.

```sh
dossier found "The speedline run finished in 3 hours" --task T-042
dossier brief --task T-042
dossier todo show T-042
```

## Remote mode: from a Windows PC

Team members work from their Windows PCs. The project stays on the server. A PC sends each command
to the server through ssh, and the output comes back as if the command had run locally.

### Sign in with a key, once

`dossier` runs ssh without asking any questions. So sign-in must work with a key, and without a
password. In PowerShell:

```powershell
ssh-keygen -t ed25519
type $HOME\.ssh\id_ed25519.pub | ssh anna@server "mkdir -p ~/.ssh && cat >> ~/.ssh/authorized_keys"
ssh anna@server "echo signed in"
```

- `ssh-keygen` creates the key. Press Enter at the passphrase prompts to use no passphrase.
- The second line copies the public key to the server. It asks for your server password once.
- The third line must print `signed in`, with no password question.

A key with a passphrase works only when the key is loaded into the ssh agent. Without the agent,
use a key with no passphrase.

### Connect a folder, once

```powershell
cd C:\Projects\P-001
dossier connect anna@server:/projects/P-001
```

The program reads the project code from the server, and writes a small pointer file in the folder.
From then on, every `dossier` command in that folder, or in a folder below it, runs on the server.
If the server cannot give the code, add it with `--code P-001`.

### Use the commands as if local

```powershell
$env:DOSSIER_AGENT = "anna"
dossier brief
dossier found 'Outlet pressure $p_2$ is 1.8 bar' --evidence "Report, section 2"
dossier status
dossier todo show T-042
```

PowerShell changes `$name` inside double quotes. Put text that contains `$` in single quotes.

### Other projects

```powershell
dossier projects
dossier --project P-001 brief
```

`projects` lists the projects that this PC is connected to. `--project` runs a command for one of
them, from any folder.

### When the server cannot be reached

If ssh cannot sign in, or the server does not answer, the command stops with exit code `5`:

```
error: could not reach the server; sign in once by hand with: ssh anna@server
```

Run `ssh anna@server` by hand, and check the key from the sign-in step above.

## Who and when

Every entry records who wrote it, the LLM that wrote it (if one did), and when. It also records who
approved it, and when. Every change adds a line to the entry's history, and a line to the journal.

```sh
dossier show DEC-001
dossier log
```

- `show` prints the entry with its history lines.
- `log` prints who did what, newest first. The first lines look like this:

```
2026-11-21T02:00:15+01:00  anna  created  FND-002  The speedline run finished in 3 hours
2026-11-21T02:00:15+01:00  anna  changed  FND-001  The steady speedline stops at mass flow 12.1 kg/s
2026-11-21T02:00:10+01:00  anna  approved  LES-001  Steady mixing-plane runs overpredict the stall margin near the stability limit
```

- `history FND-001` lists the events and the saved versions of an entry.
- `diff FND-001` shows what changed since the last saved version.
- `restore FND-001 1` brings saved version 1 back. Only a person at a terminal can run it.

## Backup

The dossier is one copy, on the server. There is no second copy and no git history. So the
server's backup must include the project folders. Each project folder holds its `DOSSIER` folder
and the files that the entries point to, such as simulation files and reports. The team dossier,
from version 0.3, is a folder on the server too, and the backup must include it.

`dossier pack` writes the whole dossier, with its history, into one archive. It does not include
the files that the entries point to. Use it as an extra copy:

```sh
dossier pack --to /backups/P-001-2026-11-21.tar
```

To make a copy every night, open the crontab with `crontab -e` and add this line. In a crontab,
write `%` as `\%`. Cron has a short `PATH`, so the line gives the full path to `dossier`. The
`/backups/dossier` folder must exist before the first run.

```
30 2 * * * cd /projects/P-001 && $HOME/bin/dossier pack --to /backups/dossier/P-001-$(date +\%F).tar
```

To restore an archive into an empty folder, run:

```sh
dossier unpack /backups/dossier/P-001-2026-11-21.tar --into /restore/P-001
```

This creates `/restore/P-001/DOSSIER`, and checks it.

## Windows and Linux

- Only the program on the server writes to the dossier. A PC uses it through `dossier connect`.
  `dossier` refuses to write to a network drive.
- Reading the files through a network share is fine. You can open an entry in an editor or in a
  file browser.
- Change entries with the `dossier` commands, or with `dossier edit <ID>`. If an approved lesson is
  changed by hand, `dossier` marks it `changed after approval`.
- The team shares one Linux group. The project folder must belong to that group:

  ```sh
  chgrp -R team /projects/P-001
  ```

  `init` sets the folders to group-writable, and new files keep the group of their folder.
  `dossier` makes every file it creates group-writable, so nobody needs to change their `umask`.

## Confidentiality

Versions 0.1 and 0.2 have one dossier per project, and nothing is shared between projects. So
customer names and customer data stay in the project's own folder. The exchange records name the
files and their checksums, but do not copy the files. From version 0.3, a team dossier keeps the
lessons, how-tos and tools that can be reused in other projects, without customer data. Before
anything is written from a project to the team dossier, it is checked against the project's
confidential terms; see [Confidential terms](#confidential-terms).

## Building from source

You need Rust 1.89 or newer. In the project folder, in PowerShell or in a Linux shell, run:

```sh
cargo build --release
```

The program is `target\release\dossier.exe` on Windows, and `target/release/dossier` on Linux.
Before you commit, run the three checks that CI runs. See [`AGENTS.md`](AGENTS.md) for the rules:

```sh
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
```
