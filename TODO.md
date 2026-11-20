<!-- todo-format: 3 -->
<!-- todo-tool: 0.8.0 -->
# dossier

<!-- todo:begin -->
# Working with TODO.md

- Never edit `TODO.md` or `TODO-archive.md` directly; always use `todo`.
- Use one unique agent name per agent: `--agent <name>` or `TODO_AGENT`.
- Exit code `1`: someone else has the task, or a rule refused the change; pick another task.
- Exit code `3`: the lock or lease is busy; run the same command again.
- Before you start a task, read the goals of the milestones above it (`todo show <id>` prints them).
- Start every commit message with the task ID, such as `T-012: short summary`.
- Workers: run `todo help worker`.
- Planner: run `todo help planner`.
<!-- todo:end -->

## Tasks

### T-001 · Core: one project remembers what was decided, assumed, found and learned
- status: open
- owner: —
- priority: P0 · type: milestone
- goal: One project folder has a dossier that records decisions, assumptions, findings, dead ends and lessons, lets a person approve an LLM's drafts, and gives every LLM brief and status at session start. Requirements: docs/requirements.md section 11, Phase 1 (v0.1).
- note (2026-11-20, planner): added

### T-002 · Write docs/format.md: the entry file format, version 1
- status: open
- owner: —
- priority: P0 · difficulty: 4 · type: docs
- milestone: T-001
- why: Every parser and writer task follows this document, so it must exist first and be exact.
- files: docs/format.md
- read: docs/requirements.md, ../todo/docs/format.md
- accept: format.md defines the format line, heading, field block, sections, line endings and file names for decision, assumption, finding and lesson, the settings file dossier.toml, and the state folder; it has one complete example per kind; rules for lenient reading and canonical writing are stated.
- reqs: R-ENTRY-1 to R-ENTRY-7, R-ENTRY-10, R-ENTRY-13 to R-ENTRY-15, R-ENTRY-19, R-ID-1 to R-ID-4, D2, D17
- note (2026-11-20, planner): added

### T-003 · Project skeleton: Cargo project, code rules, CI, AGENTS.md, licence
- status: open
- owner: —
- priority: P0 · difficulty: 3 · type: chore
- milestone: T-001
- why: Workers need a building project with the checks and rules in place before the first feature.
- files: Cargo.toml, .cargo/config.toml, .gitignore, .gitattributes, src/main.rs, src/lib.rs, .github/workflows/ci.yml, AGENTS.md, CLAUDE.md, LICENSE
- read: docs/requirements.md, ../todo/Cargo.toml, ../todo/AGENTS.md, ../todo/.github/workflows/ci.yml
- accept: cargo build, cargo fmt --check, cargo clippy -D warnings and cargo test pass on Windows; CI runs them on Windows and Linux; clippy denies unwrap, expect and panic; AGENTS.md holds the working rules, the checks and the todo block; licence is MIT like todo; dossier --version prints the version.
- verify: cargo build
- reqs: D16, NFR-1, NFR-2, NFR-5, NFR-6, REL-1
- note (2026-11-20, planner): added

### T-004 · Errors, exit codes, JSON envelope and pipe-safe output
- status: open
- owner: —
- priority: P0 · difficulty: 3 · type: feature
- milestone: T-001
- why: Every command reports results and failures the same way, so agents can branch on exit codes and parse one JSON object.
- depends: T-003
- files: src/error.rs, src/output.rs, tests/output.rs
- read: docs/requirements.md, ../todo/src/error.rs, ../todo/src/output.rs
- accept: Error type with code, message and exit codes 0 to 5; --json prints exactly one envelope dossier.<command>.v1 for success and failure; a closed pipe ends the program quietly; output is UTF-8 on a Windows console; tests cover each exit code.
- verify: cargo test --test output
- reqs: R-OUT-1 to R-OUT-4
- note (2026-11-20, planner): added

### T-005 · Find the project dossier, and the where command
- status: open
- owner: —
- priority: P0 · difficulty: 2 · type: feature
- milestone: T-001
- why: Every command must agree on which DOSSIER folder it works on.
- depends: T-004
- files: src/find.rs, src/cmd/where_cmd.rs, tests/find.rs
- read: docs/requirements.md, docs/format.md
- accept: The dossier is found by walking up from the current folder; --dossier names it directly; a missing dossier is exit 4 with a message naming init; where prints the dossier, the state folder and the project code; paths with spaces and non-ASCII characters work.
- verify: cargo test --test find
- reqs: R-PLACE-1, R-PLACE-7, R-PLACE-8, NFR-4
- note (2026-11-20, planner): added

### T-006 · Write lock and state folder, refusing network file systems
- status: open
- owner: —
- priority: P0 · difficulty: 3 · type: feature
- milestone: T-001
- why: Three people and their LLMs write at the same time; the lock is what keeps entries and ID counters intact.
- depends: T-005
- files: src/lock.rs, src/state.rs, tests/lock.rs
- read: docs/requirements.md, ../todo/src/lock.rs, ../todo/src/state.rs
- accept: A write takes an operating-system file lock in DOSSIER/.state, waits up to lock_wait_seconds and then exits 3 naming the holder; a killed holder does not block the next process; a dossier on a network drive, UNC path, cifs, smb or nfs mount is refused unless --allow-network-drive is given.
- verify: cargo test --test lock
- reqs: R-LOCK-1 to R-LOCK-3, TEST-4
- note (2026-11-20, planner): added

### T-007 · Atomic file replacement with a changed-file check
- status: open
- owner: —
- priority: P0 · difficulty: 3 · type: feature
- milestone: T-001
- why: A crash or a hand edit during a write must never leave half an entry.
- depends: T-006
- files: src/atomic.rs, tests/atomic.rs
- read: docs/requirements.md, ../todo/src/atomic.rs
- accept: A file is written to a temporary file in the same folder, flushed and renamed; a file that changed since it was read makes the command exit 3 without writing; tests kill the write at each step and find either the old or the new file.
- verify: cargo test --test atomic
- reqs: R-IO-1, R-IO-2
- note (2026-11-20, planner): added

### T-008 · Entry model: read and change an entry file keeping every other byte
- status: open
- owner: —
- priority: P0 · difficulty: 4 · type: feature
- milestone: T-001
- why: All commands read and write entries through this one model; it carries the promise that hand-written files are never damaged.
- depends: T-002, T-004
- files: src/entry/mod.rs, src/entry/parse.rs, src/entry/write.rs, tests/entry_model.rs, tests/fixtures/
- read: docs/format.md, docs/requirements.md, ../todo/src/lib.rs
- accept: Every example in format.md parses; fields, sections, unknown keys and unknown sections are kept as written; changing one field changes only that line, with CRLF, mixed endings and a missing final line ending preserved; a file that cannot be parsed is reported with file and line and skipped by readers; fixtures are invented and marked as such.
- verify: cargo test --test entry_model
- reqs: R-ENTRY-1, R-ENTRY-2, R-EDIT-1, R-EDIT-3, D17, TEST-1, TEST-3
- note (2026-11-20, planner): added

### T-009 · Project settings dossier.toml, including the people and their LLMs
- status: open
- owner: —
- priority: P0 · difficulty: 3 · type: feature
- milestone: T-001
- why: The project code, limits and the list of people and LLM names drive who may write and approve.
- depends: T-005
- files: src/config.rs, tests/config.rs
- read: docs/requirements.md, docs/format.md
- accept: dossier.toml holds the project code, default tags, people with their LLM names, and the limits brief_lines, status_lines, draft_days and lock_wait_seconds with their defaults; an unknown setting or wrong value is exit 4 naming file, line and setting; people add <person> --llm <name> adds one.
- verify: cargo test --test config
- reqs: R-CFG-1, R-CFG-3, R-CFG-4 (people kept in the project until phase 3)
- note (2026-11-20, planner): added

### T-010 · Who is writing: agent name, person and LLM
- status: open
- owner: —
- priority: P0 · difficulty: 2 · type: feature
- milestone: T-001
- why: Every entry must show whether a person or an LLM wrote it and for whom; approval depends on it.
- depends: T-009
- files: src/names.rs, tests/names.rs
- read: docs/requirements.md
- accept: The name comes from --agent, then DOSSIER_AGENT, then TODO_AGENT; a write without a name or with a name not in the settings is refused with the known names; the name resolves to a person and, for an LLM name, the person it belongs to.
- verify: cargo test --test names
- reqs: D13, R-CFG-4, R-LIFE-2
- note (2026-11-20, planner): added

### T-011 · init: create a project dossier
- status: open
- owner: —
- priority: P0 · difficulty: 2 · type: feature
- milestone: T-001
- why: The first command anyone runs in a project.
- depends: T-009, T-006
- files: src/cmd/init.rs, tests/cmd_init.rs
- read: docs/requirements.md, docs/format.md
- accept: init --code <code> creates DOSSIER with the kind folders, dossier.toml and the state folder; it refuses where a dossier exists; a team dossier path is optional in this phase.
- verify: cargo test --test cmd_init
- reqs: R-PLACE-2
- note (2026-11-20, planner): added

### T-012 · Entry IDs per kind, given out under the lock
- status: open
- owner: —
- priority: P0 · difficulty: 2 · type: feature
- milestone: T-001
- why: IDs are how people and LLMs cite entries; two writers must never get the same one.
- depends: T-006, T-008
- files: src/ids.rs, tests/ids.rs
- read: docs/requirements.md, docs/format.md
- accept: DEC, ASM, FND and LES numbers count separately, are the highest ever given plus one including removed entries, and are never reused; an ID is accepted with or without its place prefix; 20 processes adding at once all get different IDs.
- verify: cargo test --test ids
- reqs: R-ID-1 to R-ID-4, TEST-4
- note (2026-11-20, planner): added

### T-013 · add and the short forms decide, found, learn and assume
- status: open
- owner: —
- priority: P0 · difficulty: 4 · type: feature
- milestone: T-001
- why: Recording must be one short command, or nobody records anything.
- depends: T-012, T-007, T-010
- files: src/cmd/add.rs, src/cmd/shortforms.rs, src/entry/kinds.rs, tests/cmd_add.rs
- read: docs/requirements.md, docs/format.md
- accept: add <kind> creates an entry from options, --from or --stdin; decide, found, found --dead-end, learn, assume and note write the right kind with required fields and sections checked; a person's entry is approved at once, an LLM's lesson is a draft, an LLM's decision needs --said-by; text with quotes, backticks and $ survives; one line such as DEC-012: added is printed.
- verify: cargo test --test cmd_add
- reqs: R-WRITE-1 to R-WRITE-4, R-WRITE-7, R-WRITE-14, R-WRITE-16, R-ENTRY-3 to R-ENTRY-7, R-ENTRY-13, R-ENTRY-15, D5
- note (2026-11-20, planner): added
- note (2026-11-20, planner): The quick note is found --later (R-WRITE-7); there is no note command (T-059).

### T-014 · set, retitle, link and --dry-run
- status: open
- owner: —
- priority: P1 · difficulty: 3 · type: feature
- milestone: T-001
- why: Entries need correcting without opening an editor, and an LLM's change to approved knowledge must not stay approved.
- depends: T-013
- files: src/cmd/set.rs, src/cmd/link.rs, tests/cmd_set.rs
- read: docs/requirements.md
- accept: set changes a field or section and retitle renames the file but not the ID; link adds each entry to the other's related field; an LLM changing an approved lesson turns it into a draft; --dry-run on every write command prints the change and writes nothing.
- verify: cargo test --test cmd_set
- reqs: R-WRITE-8 to R-WRITE-10
- note (2026-11-20, planner): added

### T-015 · Show similar existing entries when a lesson is added
- status: open
- owner: —
- priority: P2 · difficulty: 2 · type: feature
- milestone: T-001
- why: Without it the same lesson is written five times in slightly different words.
- depends: T-013
- files: src/similar.rs, src/cmd/add.rs, tests/similar.rs
- read: docs/requirements.md
- accept: Adding a lesson prints up to five entries with the most similar title and tags, rejected ones marked; --update <ID> updates one of them in the same command.
- verify: cargo test --test similar
- reqs: R-WRITE-11
- note (2026-11-20, planner): added

### T-016 · Journal and snapshots: history, diff, restore, log
- status: open
- owner: —
- priority: P0 · difficulty: 3 · type: feature
- milestone: T-001
- why: There is no git in the project folders, so the tool's own history is the only undo.
- depends: T-013
- files: src/history.rs, src/cmd/history.rs, tests/history.rs
- read: docs/requirements.md, ../todo/src/history.rs
- accept: Every change adds a line to journal.jsonl with time, name, person, command and entry; the old file is saved as a snapshot before it is changed; history lists what happened to an entry or task, diff shows a change, restore brings a version back for a person at a terminal, log prints the journal.
- verify: cargo test --test history
- reqs: R-HIST-1 to R-HIST-3, R-READ-6
- note (2026-11-20, planner): added

### T-017 · show and list with filters
- status: open
- owner: —
- priority: P0 · difficulty: 3 · type: feature
- milestone: T-001
- why: Reading entries back is the point of writing them.
- depends: T-008, T-005
- files: src/cmd/show.rs, src/cmd/list.rs, tests/cmd_read.rs
- read: docs/requirements.md
- accept: show prints the entry as in the file with marks for draft, superseded, retired and changed after approval, and --json returns fields, sections, unknown keys and links both ways; list prints one line per entry with the filters kind, status, machine, analysis, software, tag, author, task, since and where, approved entries by default.
- verify: cargo test --test cmd_read
- reqs: R-READ-1, R-READ-2, R-READ-5
- note (2026-11-20, planner): added

### T-018 · search: full text over titles, fields and sections
- status: open
- owner: —
- priority: P0 · difficulty: 3 · type: feature
- milestone: T-001
- why: Retrieval by words is how a person finds what the team knew.
- depends: T-017
- files: src/cmd/search.rs, tests/cmd_search.rs
- read: docs/requirements.md
- accept: search finds entries containing all words, case ignored, title matches first, then tags, then text, newest first; results show ID, title and the matching line, --full whole entries; drafts appear only with --drafts and are marked; the list filters apply.
- verify: cargo test --test cmd_search
- reqs: R-READ-3, R-READ-4, R-DRAFT-6
- note (2026-11-20, planner): added

### T-019 · Drafts and approval: review-task, drafts, approve, reject
- status: open
- owner: —
- priority: P0 · difficulty: 4 · type: feature
- milestone: T-001
- why: This is the step that keeps wrong LLM conclusions out of the team's knowledge.
- depends: T-013, T-017
- files: src/cmd/review.rs, src/cmd/approve.rs, src/todo_bridge.rs, tests/cmd_drafts.rs
- read: docs/requirements.md
- accept: review-task prints the task from todo show --json, linked entries and the drafting rules, and changes nothing; drafts lists drafts with proposer, task and age, old ones first; approve runs only on a terminal, shows each draft, asks approve, edit, reject or skip, and refuses LLM names; reject keeps the entry hidden; todo missing gives exit 5 with a clear message; tests use a stand-in for todo.
- verify: cargo test --test cmd_drafts
- reqs: R-DRAFT-1 to R-DRAFT-6, R-DRAFT-8, D1, D6, D7, TEST-2, TEST-5
- note (2026-11-20, planner): added

### T-020 · Life of an entry: statuses, supersede, retire, remove, validity dates, approval checksum
- status: open
- owner: —
- priority: P0 · difficulty: 4 · type: feature
- milestone: T-001
- why: Knowledge goes out of date; the dossier must show what replaced it and never silently change what an entry says.
- depends: T-019
- files: src/cmd/life.rs, src/entry/status.rs, tests/cmd_life.rs
- read: docs/requirements.md, docs/format.md
- accept: The status table of R-LIFE-1 is enforced; supersede links both entries and readers show the new one; retire needs a reason; remove needs a person at a terminal and journals the removal without the text; valid-from and valid-until are written by the tool; an approved lesson changed by hand is marked changed after approval in show and check.
- verify: cargo test --test cmd_life
- reqs: R-LIFE-1 to R-LIFE-6, R-ENTRY-14, D20
- note (2026-11-20, planner): added

### T-021 · edit: open an entry in an editor under the lock
- status: open
- owner: —
- priority: P1 · difficulty: 2 · type: feature
- milestone: T-001
- why: Longer entries are easier to write in an editor than in options.
- depends: T-020
- files: src/cmd/edit.rs, tests/cmd_edit.rs
- read: docs/requirements.md, ../todo/src/lib.rs
- accept: edit opens the entry in VISUAL, EDITOR or the system default under the lock, checks the result when the editor closes, records the change, and treats an LLM's edit of an approved lesson as in set.
- verify: cargo test --test cmd_edit
- reqs: R-EDIT-2
- note (2026-11-20, planner): added

### T-022 · brief and brief --task: context for a session or a task
- status: open
- owner: —
- priority: P0 · difficulty: 4 · type: feature
- milestone: T-001
- why: An LLM that starts without the project's decisions will contradict them; this is the context manager.
- depends: T-018, T-020
- files: src/cmd/brief.rs, src/select.rs, tests/cmd_brief.rs
- read: docs/requirements.md
- accept: brief prints project code and tags, approved decisions in full newest first, finding titles and the count of drafts within brief_lines, shortening older decisions and saying how many were left out; brief --task prints entries linked to the task, decisions of its milestones and entries matching its tags and title words; drafts, rejected, superseded and retired entries never appear; the selection is the same on every run and holds with 500 decisions; brief works without todo, brief --task says it needs it.
- verify: cargo test --test cmd_brief
- reqs: R-CTX-1 to R-CTX-5, TEST-9
- note (2026-11-20, planner): added

### T-023 · status: the state of the whole project
- status: open
- owner: —
- priority: P0 · difficulty: 3 · type: feature
- milestone: T-001
- why: Three LLMs working for three people must see the same picture of progress, decisions and what is unsettled.
- depends: T-022
- files: src/cmd/status.rs, tests/cmd_status.rs
- read: docs/requirements.md
- accept: status prints milestone goals with progress, who is working on what, blocked tasks and questions, decisions in force, newest findings, drafts waiting and the next ready tasks within status_lines; task parts come from todo --json and are left out with one line when todo is missing; --since and --milestone limit it; summary is covered by status.
- verify: cargo test --test cmd_status
- reqs: R-CTX-9 to R-CTX-11, R-READ-8
- note (2026-11-20, planner): added
- note (2026-11-20, planner): There is no summary command; status covers R-READ-8 (T-059).

### T-024 · decide shows related decisions and needs --replaces or --besides from an LLM
- status: open
- owner: —
- priority: P1 · difficulty: 2 · type: feature
- milestone: T-001
- why: Two LLMs must not record contradicting decisions without anyone noticing.
- depends: T-013, T-020
- files: src/cmd/shortforms.rs, tests/cmd_decide.rs
- read: docs/requirements.md
- accept: decide prints decisions in force that share tags or title words before writing; --replaces supersedes the old one in the same step; an LLM without --replaces or --besides is refused with the list shown.
- verify: cargo test --test cmd_decide
- reqs: R-CTX-13
- note (2026-11-20, planner): added

### T-025 · check: report problems in a dossier
- status: open
- owner: —
- priority: P1 · difficulty: 3 · type: feature
- milestone: T-001
- why: Hand edits and time break entries; check is how the team notices.
- depends: T-020
- files: src/cmd/check.rs, tests/cmd_check.rs
- read: docs/requirements.md
- accept: check reports one line per problem as file:line: ID KIND message for unparsable files, missing required fields and sections, unknown statuses, IDs not matching file names or duplicated, broken links, entries changed after approval and lessons without Valid for; exit 0 when clean, 1 otherwise.
- verify: cargo test --test cmd_check
- reqs: R-CHECK-1
- note (2026-11-20, planner): added

### T-026 · Agent rules: help agents, agents install, agents check
- status: open
- owner: —
- priority: P0 · difficulty: 3 · type: feature
- milestone: T-001
- why: Every LLM learns the tool from this one text; if it is unclear, nothing gets recorded.
- depends: T-022, T-023, T-019
- files: src/agents/rules.md, src/agents/mod.rs, tests/cmd_agents.rs
- read: docs/requirements.md, ../todo/src/agents/
- accept: help agents prints at most 40 lines with the phase 1 rules: brief first, status before unasked work, work only on a claimed task, decisions in force bind, decide with --said-by, found, review-task after todo done, write in English, never approve; agents install writes a managed block into AGENTS.md below todo's block keeping every other line; agents check tells whether it is current.
- verify: cargo test --test cmd_agents
- reqs: R-AGENT-1, R-AGENT-2, R-CTX-12, R-ENTRY-19, D14, D25
- note (2026-11-20, planner): added

### T-027 · pack and unpack: one archive of a dossier
- status: open
- owner: —
- priority: P1 · difficulty: 3 · type: feature
- milestone: T-001
- why: One copy on a server with no git means the backup is the only protection.
- depends: T-006
- files: src/cmd/pack.rs, tests/cmd_pack.rs
- read: docs/requirements.md
- accept: pack --to <file> writes the whole dossier with its state folder into one archive under the lock and never includes files that entries point to; unpack restores into an empty folder and runs check; the archive format needs no dependency outside the approved list, or the commit message gives the reason for one.
- verify: cargo test --test cmd_pack
- reqs: R-BACKUP-1, R-BACKUP-2, D24
- note (2026-11-20, planner): added

### T-028 · Speed test with 2,000 entries
- status: open
- owner: —
- priority: P1 · difficulty: 2 · type: test
- milestone: T-001
- why: No index is kept, so the speed limits must be proven before the design is relied on.
- depends: T-022, T-018
- files: tests/perf.rs
- read: docs/requirements.md, ../todo/tests/
- accept: With 300 project entries, brief and search finish under 300 ms and a write under 200 ms, measured as medians after a warm-up; the test runs in CI.
- verify: cargo test --test perf
- reqs: NFR-3
- note (2026-11-20, planner): added

### T-029 · README: purpose, what it needs, first project, use over ssh, backup
- status: open
- owner: —
- priority: P1 · difficulty: 3 · type: docs
- milestone: T-001
- why: The repository is public and the team has to be able to start without the author.
- depends: T-026,T-027,T-063
- files: README.md
- read: docs/requirements.md
- accept: README explains what the tool is for, that it needs todo 0.8 or newer with a link, a first project in ten commands, how it works with todo, how to run it on the server from a PC with a plain ssh command, that the dossier is one copy that must be in the server's backup, and how to schedule pack; examples are in PowerShell and a Linux shell.
- reqs: REL-2, R-BACKUP-3, O-7
- note (2026-11-20, planner): added

### T-030 · Release 0.1.0: Linux and Windows programs
- status: open
- owner: —
- priority: P1 · difficulty: 3 · type: chore
- milestone: T-001
- why: Phase 1 is only finished when the team can install it on the server and try it on a real project.
- depends: T-029,T-028,T-025,T-016,T-014,T-021,T-024,T-015,T-011,T-063,T-064
- files: .github/workflows/release.yml, Cargo.toml, README.md
- read: docs/requirements.md, ../todo/.github/workflows/
- accept: A v0.1.0 tag builds one program for Linux x64 and one dossier.exe for Windows x64 with the C runtime linked in, with checksums; all checks pass on both systems; the program refuses a dossier written by a newer format with a message to update.
- reqs: NFR-1, NFR-2, NFR-7, NFR-8, REL-1
- note (2026-11-20, planner): added

### T-031 · Project facts: a result can be traced to what it was made from
- status: open
- owner: —
- priority: P1 · type: milestone
- goal: The project's key numbers, the files of each task, the simulation runs and the exchange with the customer are recorded, so that for a delivered report the dossier names the model, runs, values and assumptions behind every result. Requirements: docs/requirements.md section 11, Phase 2 (v0.2). Each task here is split when the phase starts.
- note (2026-11-20, planner): added

### T-032 · Key values: one sheet of named numbers with units
- status: open
- owner: —
- priority: P1 · difficulty: 5 · type: feature
- milestone: T-031
- why: One number written in five places is soon five numbers, and an LLM cannot tell which is current.
- depends: T-030
- accept: value set, value show and values work as in R-VALUE-1 to R-VALUE-6; brief and status print the key values.
- reqs: R-VALUE-1 to R-VALUE-6, D23
- note (2026-11-20, planner): added

### T-033 · Files of a task: attach, detach, reattach, files, which
- status: open
- owner: —
- priority: P1 · difficulty: 5 · type: feature
- milestone: T-031
- why: A reader must find the simulation file, geometry and results of a task without searching the disk, and see which results are outdated.
- depends: T-030
- accept: The file list of each task by role with made-from links works as in R-FILE-8 to R-FILE-16.
- reqs: R-FILE-8 to R-FILE-16, D18
- note (2026-11-20, planner): added
- note (2026-11-20, planner): Also covers R-XOS-5: warn about file names that cannot exist on Windows or differ only in case.

### T-034 · Software use and simulation runs: used, ran, runs
- status: open
- owner: —
- priority: P1 · difficulty: 5 · type: feature
- milestone: T-031
- why: Which software version and which parameters produced a result is asked in every review.
- depends: T-033
- accept: used, ran, runs and runs --compare work as in R-USE-2, R-USE-14 and R-USE-15, with software names kept in the project.
- reqs: R-USE-2, R-USE-14, R-USE-15, R-ENTRY-9
- note (2026-11-20, planner): added

### T-035 · Pictures in entries
- status: open
- owner: —
- priority: P1 · difficulty: 5 · type: feature
- milestone: T-031
- why: Engineering evidence is mostly plots, and a lesson whose plot was deleted has lost its evidence.
- depends: T-030
- accept: --image, image add and image remove copy small pictures into the dossier and show them as in R-ENTRY-16 and R-ENTRY-17.
- reqs: R-ENTRY-16, R-ENTRY-17, D26
- note (2026-11-20, planner): added

### T-036 · Proposed decisions: propose, accept, decline, and the deciders
- status: open
- owner: —
- priority: P1 · difficulty: 5 · type: feature
- milestone: T-031
- why: Three people and three LLMs must be able to suggest a direction without it becoming binding by accident.
- depends: T-030
- accept: propose, accept and decline work as in R-WRITE-12, and decide follows R-WRITE-13.
- reqs: R-WRITE-12, R-WRITE-13, D21
- note (2026-11-20, planner): added

### T-037 · Assumptions confirmed and refuted, and what rests on them
- status: open
- owner: —
- priority: P1 · difficulty: 5 · type: feature
- milestone: T-031
- why: Results built on a wrong assumption must be found again.
- depends: T-033
- accept: confirm, refute and cleared work as in R-WRITE-15, with marks shown in show, files, status and brief.
- reqs: R-WRITE-15
- note (2026-11-20, planner): added

### T-038 · What was in force on a date: --on
- status: open
- owner: —
- priority: P2 · difficulty: 5 · type: feature
- milestone: T-031
- why: What did we know and decide on the day the report went out must have an answer.
- depends: T-033
- accept: --on <date> works on the commands named in R-LIFE-9.
- reqs: R-LIFE-9
- note (2026-11-20, planner): added

### T-039 · Exchange with the customer: received, delivered, exchange
- status: open
- owner: —
- priority: P1 · difficulty: 5 · type: feature
- milestone: T-031
- why: What was received and delivered, when and in which version, is the first thing asked in a dispute.
- depends: T-033
- accept: received, delivered and exchange work as in R-EXCH-1 to R-EXCH-5, including the warning before delivering outdated results.
- reqs: R-EXCH-1 to R-EXCH-5
- note (2026-11-20, planner): added

### T-040 · Code projects: map of the code, pointers into code, init --code-project
- status: open
- owner: —
- priority: P2 · difficulty: 5 · type: feature
- milestone: T-031
- why: The owner's own software is the first test bed, and an LLM needs orientation in thousands of files before it searches.
- depends: T-030
- accept: map set, map, code pointers and init --code-project work as in R-CODE-1 to R-CODE-5.
- reqs: R-CODE-1 to R-CODE-5
- note (2026-11-20, planner): added

### T-041 · brief and status show values, open assumptions, proposals and dead ends; release 0.2.0
- status: open
- owner: —
- priority: P1 · difficulty: 5 · type: feature
- milestone: T-031
- why: What phase 2 records only helps when it reaches the LLM's context.
- depends: T-033
- accept: R-CTX-16 holds; format.md and the tests cover every phase 2 addition; version 0.2.0 is released.
- reqs: R-CTX-16
- note (2026-11-20, planner): added

### T-042 · Team: knowledge reaches the next project without customer data
- status: open
- owner: —
- priority: P1 · type: milestone
- goal: One team dossier holds lessons, how-tos, scripts, the software list and a glossary; entries are promoted from projects only after the confidentiality check; finished projects of the past are brought in. Requirements: docs/requirements.md section 11, Phase 3 (v0.3). Each task here is split when the phase starts.
- note (2026-11-20, planner): added

### T-043 · Team dossier: init --team-dossier, project codes, team settings
- status: open
- owner: —
- priority: P1 · difficulty: 5 · type: feature
- milestone: T-042
- why: Knowledge locked in one project folder is lost to the next project.
- depends: T-041
- accept: R-PLACE-3 to R-PLACE-6, R-TEAM-1, R-CFG-2 and R-CFG-4 hold; people move from the project settings to the team settings.
- reqs: R-PLACE-3 to R-PLACE-6, R-TEAM-1, R-CFG-2, R-CFG-4, D3
- note (2026-11-20, planner): added

### T-044 · Confidentiality: confidential terms and the check before any team write
- status: open
- owner: —
- priority: P0 · difficulty: 5 · type: feature
- milestone: T-042
- why: The team works for several customers under confidentiality agreements; this must exist before anything is promoted.
- depends: T-043
- accept: R-CONF-1 to R-CONF-6 and R-ENTRY-18 hold, with the tests of TEST-6.
- reqs: R-CONF-1 to R-CONF-6, R-ENTRY-18, D8, TEST-6
- note (2026-11-20, planner): added

### T-045 · Promotion and closing a project: promote, close
- status: open
- owner: —
- priority: P1 · difficulty: 5 · type: feature
- milestone: T-042
- why: Promotion is the only way project knowledge becomes team knowledge.
- depends: T-044
- accept: promote and close work as in R-TEAM-2 to R-TEAM-4 and R-TEAM-6; search and brief include the team dossier.
- reqs: R-TEAM-2 to R-TEAM-4, R-TEAM-6, R-WRITE-5, R-WRITE-6
- note (2026-11-20, planner): added

### T-046 · Software list, how-tos and the usage index: software, howto, howtos, where-used
- status: open
- owner: —
- priority: P1 · difficulty: 5 · type: feature
- milestone: T-042
- why: How and where a software was used for a task is asked at the start of every similar task.
- depends: T-044
- accept: R-USE-1, R-USE-3 to R-USE-7 and R-ENTRY-8 hold.
- reqs: R-USE-1, R-USE-3 to R-USE-7, R-ENTRY-8, R-ENTRY-9
- note (2026-11-20, planner): added

### T-047 · Scripts: script add, script update, scripts
- status: open
- owner: —
- priority: P1 · difficulty: 5 · type: feature
- milestone: T-042
- why: A script nobody can find or trust is rewritten.
- depends: T-044
- accept: R-USE-8 to R-USE-13 and R-ENTRY-12 hold.
- reqs: R-USE-8 to R-USE-13, R-ENTRY-12, D19
- note (2026-11-20, planner): added

### T-048 · Glossary: term add, term, glossary, and search through other spellings
- status: open
- owner: —
- priority: P2 · difficulty: 5 · type: feature
- milestone: T-042
- why: Every LLM should read the team's abbreviations the same way, and a German word should find the English entries.
- depends: T-043
- accept: R-TERM-1 to R-TERM-4 hold; check warns about entries that look German as in R-ENTRY-20.
- reqs: R-TERM-1 to R-TERM-4, R-ENTRY-20
- note (2026-11-20, planner): added

### T-049 · What is new: news, and the news line in brief
- status: open
- owner: —
- priority: P2 · difficulty: 5 · type: feature
- milestone: T-042
- why: Colleagues need to see what was added to the team dossier without reading everything.
- depends: T-043
- accept: R-READ-10 and R-READ-11 hold.
- reqs: R-READ-10, R-READ-11
- note (2026-11-20, planner): added

### T-050 · Review dates and how-tos not tested with the newest version
- status: open
- owner: —
- priority: P2 · difficulty: 5 · type: feature
- milestone: T-042
- why: Lessons and how-tos go out of date quietly.
- depends: T-043
- accept: R-LIFE-7 and R-LIFE-8 hold.
- reqs: R-LIFE-7, R-LIFE-8
- note (2026-11-20, planner): added

### T-051 · Bringing in finished projects: init --past, review-folder, review-document, import status
- status: open
- owner: —
- priority: P1 · difficulty: 5 · type: feature
- milestone: T-042
- why: The dossier must not start empty; years of finished projects hold most of what the team knows.
- depends: T-044
- accept: R-IMPORT-1 to R-IMPORT-9 hold; format.md and the tests cover every phase 3 addition; version 0.3.0 is released.
- reqs: R-IMPORT-1 to R-IMPORT-9, D27
- note (2026-11-20, planner): added

### T-052 · Working from the PC: the dossier is used from Windows as if it were local
- status: open
- owner: —
- priority: P2 · type: milestone
- goal: A folder on a PC is connected to a project on the server; every dossier and todo command runs there through ssh; large simulation files are taken, worked on locally and returned without overwriting a colleague's version. Requirements: docs/requirements.md section 11, Phase 4 (v0.4). Each task here is split when the phase starts.
- note (2026-11-20, planner): added

### T-053 · Remote mode: connection reuse and connection check
- status: open
- owner: —
- priority: P2 · difficulty: 5 · type: feature
- milestone: T-052
- why: Team members work on Windows PCs while the one copy of the dossier is on the Linux server.
- depends: T-051
- accept: R-REMOTE-1 to R-REMOTE-3 and R-REMOTE-5 to R-REMOTE-8 hold, with the tests of TEST-8.
- reqs: R-REMOTE-1 to R-REMOTE-3, R-REMOTE-5 to R-REMOTE-8, D9, TEST-8
- note (2026-11-20, planner): added
- note (2026-11-20, planner): Forwarding, connect and the project register moved to phase 1 (T-060). Left here: R-REMOTE-7 and R-REMOTE-8.

### T-054 · Large files: take, return, and the copy in remote mode; release 0.4.0
- status: open
- owner: —
- priority: P2 · difficulty: 5 · type: feature
- milestone: T-052
- why: Two people who copy the same simulation file out and back overwrite each other's work.
- depends: T-053
- accept: R-FILE-1 to R-FILE-7 and R-REMOTE-4 hold, with the tests of TEST-7; the manual test plan of TEST-10 was run on the real server; version 0.4.0 is released.
- reqs: R-FILE-1 to R-FILE-7, R-REMOTE-4, D15, TEST-7, TEST-10
- note (2026-11-20, planner): added

### T-055 · Growing projects: the context stays small and routine steps run by themselves
- status: open
- owner: —
- priority: P3 · type: milestone
- goal: In a long project brief still fits its limit and still holds what the LLM needs, and session review, hooks and export take routine work off the people. Requirements: docs/requirements.md section 11, Phase 5 (v0.5). Each task here is split when the phase starts.
- note (2026-11-20, planner): added

### T-056 · Milestone digests: digest, and digests in brief and status
- status: open
- owner: —
- priority: P3 · difficulty: 5 · type: feature
- milestone: T-055
- why: The context an LLM gets must stay small as a project grows.
- depends: T-054
- accept: R-CTX-14 and R-CTX-15 hold.
- reqs: R-CTX-14, R-CTX-15, D22
- note (2026-11-20, planner): added

### T-057 · Session review and Claude Code hooks and skill
- status: open
- owner: —
- priority: P3 · difficulty: 5 · type: feature
- milestone: T-055
- why: Capture only becomes a habit when the routine steps happen by themselves.
- depends: T-054
- accept: R-DRAFT-7, R-DRAFT-9, R-AGENT-3, R-AGENT-4, R-CTX-7 hold.
- reqs: R-DRAFT-7, R-DRAFT-9, R-AGENT-3, R-AGENT-4, R-CTX-7
- note (2026-11-20, planner): added

### T-058 · Extras: story, search --similar, brief --budget, check --fix, export, schema, unit warnings, team summary, readers' guide; release 0.5.0
- status: open
- owner: —
- priority: P3 · difficulty: 5 · type: feature
- milestone: T-055
- why: Smaller comforts that are worth having once the daily use is settled.
- depends: T-054
- accept: R-READ-7, R-READ-9, R-CTX-6, R-CTX-8, R-CHECK-2, R-OUT-5 to R-OUT-7, R-ENTRY-11, R-TEAM-5 and REL-4 hold; version 0.5.0 is released.
- reqs: R-READ-7, R-READ-9, R-CTX-6, R-CTX-8, R-CHECK-2, R-OUT-5 to R-OUT-7, R-ENTRY-11, R-TEAM-5, REL-4
- note (2026-11-20, planner): added

### T-063 · Run commands from the PC: connect, the per-user project register, forwarding through ssh, dossier todo
- status: open
- owner: —
- priority: P0 · difficulty: 4 · type: feature
- milestone: T-001
- why: The team works on Windows PCs from the first day while the one copy of the dossier is on the Linux server; people and LLMs must not have to know a server name or path.
- depends: T-004,T-005
- files: src/remote/mod.rs, src/remote/register.rs, src/cmd/connect.rs, src/cmd/projects.rs, tests/remote.rs
- read: docs/requirements.md
- accept: connect writes the pointer file and the register entry; projects lists the register and --project <code> works from any folder; every command in a connected folder runs on the server with output and exit code unchanged; quotes, backticks, dollar signs, umlauts and line breaks arrive unchanged and --stdin is passed on; approve, remove and close ask ssh for a terminal; ssh never prompts and fails with exit 5 within the time limit; dossier todo forwards todo the same way; brief and status name the project code and server folder first; tests use a stand-in for ssh.
- verify: cargo test --test remote
- reqs: R-REMOTE-1 to R-REMOTE-3, R-REMOTE-5, R-REMOTE-6, R-REMOTE-9 to R-REMOTE-12, TEST-8
- note (2026-11-20, planner): added

### T-064 · Windows and Linux together: paths, file names, line endings, group access, version match
- status: open
- owner: —
- priority: P0 · difficulty: 4 · type: feature
- milestone: T-001
- why: The dossier lives on Linux and is used from Windows; nothing may depend on which system a command was typed on, and one colleague's entry must be writable by another.
- depends: T-008,T-011,T-063
- files: src/xos.rs, src/paths.rs, tests/xos.rs
- read: docs/requirements.md, docs/format.md
- accept: Stored paths are relative with forward slashes and a path typed with backslashes stores the same line; created file names are lower-case ASCII, never a Windows reserved name, never differing only in case; CRLF files and a byte-order mark are changed keeping every other byte; times carry their UTC offset; the same commands give byte-identical files, checksums and search order on Windows and Linux in CI; on Linux new files and folders are group-writable and init reports the group; a format version mismatch between PC and server is refused.
- verify: cargo test --test xos
- reqs: R-XOS-1 to R-XOS-4, R-XOS-6 to R-XOS-10, TEST-11
- note (2026-11-20, planner): added

## Questions

### T-059 · Merge the overlapping commands before phase 1 is built? summary into status, note into found, used and ran into one record, howtos and scripts into list with a filter, news into status --since last. See docs/requirements.md section 11, last part.
- status: done
- owner: planner
- type: question
- about: T-001
- note (2026-11-20, planner): added
- note (2026-11-20, planner): answer — Yes: merged. summary into status, note into found --later, ran into used --model, howtos and scripts into list --kind, news into status --since last. Requirements updated (section 11, Merged commands).

### T-060 · Is typing a plain ssh command comfortable enough for phases 1 to 3, or must a first part of remote mode (R-REMOTE-1, 2, 5) move into phase 1? Open point O-7.
- status: done
- owner: planner
- type: question
- about: T-001
- note (2026-11-20, planner): added
- note (2026-11-20, planner): answer — Running commands from the PC moves into phase 1, with a per-user register of projects (code, server, folder) so nobody types a server or path. ssh signs in by key without a password on the team's network. File copying stays in phase 4. Requirements: R-REMOTE-9 to 12, O-7.

### T-061 · Does a lock taken through the Samba share see a lock taken on the Linux server? Needs a test on the real server (open point O-1); decides whether R-LOCK-3 refuses or only warns.
- status: open
- owner: —
- type: question
- about: T-006
- note (2026-11-20, planner): added

### T-062 · Is a command named dossier already installed on the team's server? Open point O-3.
- status: done
- owner: planner
- type: question
- about: T-001
- note (2026-11-20, planner): added
- note (2026-11-20, planner): answer — No: there is no command named dossier on the server (owner, 2026-11-20).
