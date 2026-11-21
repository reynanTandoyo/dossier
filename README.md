# dossier

`dossier` is a command-line program that keeps what a small engineering team learns and decides
while doing its projects. People and their LLMs write to it and read from it, so that neither has
to rely on memory.

**Status: v0.1 (phase 1)**

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

### Not in version 0.1

These come in later phases. See [`docs/requirements.md`](docs/requirements.md), section 11.

- **Phase 2 (v0.2):** key values, the files of each task, simulation runs, pictures in entries,
  proposals for the project lead, and the exchange with the customer.
- **Phase 3 (v0.3):** the team dossier, promoting entries from one project to another, how-tos,
  scripts, the software list, the glossary, and finished projects.
- **Phase 4 (v0.4):** copying large simulation files to a PC and back.
- **Phase 5 (v0.5):** milestone digests, Claude Code hooks, and export.

## What it needs

- [`todo`](https://github.com/reynanTandoyo/todo) 0.8 or newer. `todo` keeps the tasks of a
  project in `TODO.md`. `brief --task`, `review-task`, `status` and `dossier todo` need it. The
  other commands work without it.
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

  `init` sets the folders to group-writable, and new files keep the group of their folder. Each
  person also needs `umask 002` in the shell, so that the files they create are group-writable.

## Confidentiality

Version 0.1 has one dossier per project, and no team dossier. Nothing is shared between projects.
So customer names and customer data stay in the project's own folder. The team dossier comes in
phase 3 (v0.3). It keeps the lessons that can be reused in other projects, without customer data.

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
