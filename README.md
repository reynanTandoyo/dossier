# dossier

`dossier` is a command-line program that keeps what a small engineering team learns and decides
while doing its projects. People and their LLMs write to it and read from it, so that neither has
to rely on memory.

**Status: v0.2 (phase 2)**

- [`docs/requirements.md`](docs/requirements.md): what the tool must do, and the five phases it is
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

### Not in version 0.2

These come in later phases. See [`docs/requirements.md`](docs/requirements.md), section 11.

- **Phase 3 (v0.3):** the team dossier, promoting entries from one project to another, how-tos,
  scripts, the software list, the glossary, what is new, and finished projects.
- **Phase 4 (v0.4):** copying large simulation files to a PC and back.
- **Phase 5 (v0.5):** milestone digests, session review, and export.

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
- `dossier brief --task T-042` prints what matters for one task. `dossier status` shows the tasks
  and the knowledge together. Both need `todo`.
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
and the files that the entries point to, such as simulation files and reports.

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

Versions 0.1 and 0.2 have one dossier per project, and no team dossier. Nothing is shared between
projects. So customer names and customer data stay in the project's own folder. The exchange
records name the files and their checksums, but do not copy the files. The team dossier comes in
phase 3 (v0.3). It keeps the lessons that can be reused in other projects,
without customer data.

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
