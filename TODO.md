<!-- todo-format: 3 -->
<!-- todo-tool: 1.0.0 -->
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
- status: done
- owner: coordinator
- priority: P0 · type: milestone
- goal: One project folder has a dossier that records decisions, assumptions, findings, dead ends and lessons, lets a person approve an LLM's drafts, and gives every LLM brief and status at session start. Requirements: docs/requirements.md section 11, Phase 1 (v0.1).
- by: Reynan Henry Tandoyo
- note (2026-11-20, planner): added
- note (2026-11-21, coordinator): done

### T-002 · Write docs/format.md: the entry file format, version 1
- status: done
- owner: planner
- priority: P0 · difficulty: 4 · type: docs
- milestone: T-001
- why: Every parser and writer task follows this document, so it must exist first and be exact.
- files: docs/format.md, docs/design.md, docs/requirements.md, AGENTS.md
- read: docs/requirements.md, ../todo/docs/format.md
- accept: format.md defines the format line, heading, field block, sections, line endings and file names for decision, assumption, finding and lesson, the settings file dossier.toml, and the state folder; it has one complete example per kind; rules for lenient reading and canonical writing are stated.
- commit: 38b117a
- reqs: R-ENTRY-1 to R-ENTRY-7, R-ENTRY-10, R-ENTRY-13 to R-ENTRY-15, R-ENTRY-19, R-ID-1 to R-ID-4, D2, D17
- by: Reynan Henry Tandoyo
- note (2026-11-20, planner): added
- note (2026-11-21, planner): done — format.md written for phase 1; design.md added so that every later task fills in one file.

### T-003 · Project skeleton: Cargo project, code rules, CI, AGENTS.md, licence
- status: dropped
- owner: —
- priority: P0 · difficulty: 3 · type: chore
- milestone: T-001
- why: Workers need a building project with the checks and rules in place before the first feature.
- depends: 
- files: Cargo.toml, .cargo/config.toml, .gitignore, .gitattributes, src/main.rs, src/lib.rs, .github/workflows/ci.yml, AGENTS.md, CLAUDE.md, LICENSE
- read: docs/requirements.md, ../todo/Cargo.toml, ../todo/AGENTS.md, ../todo/.github/workflows/ci.yml
- accept: cargo build, cargo fmt --check, cargo clippy -D warnings and cargo test pass on Windows; CI runs them on Windows and Linux; clippy denies unwrap, expect and panic; AGENTS.md holds the working rules, the checks and the todo block; licence is MIT like todo; dossier --version prints the version.
- verify: cargo build
- reqs: D16, NFR-1, NFR-2, NFR-5, NFR-6, REL-1
- note (2026-11-20, planner): added
- note (2026-11-21, planner): dropped — Replaced by the detailed v0.1 tasks under T-001 (one file per task, see docs/design.md).

### T-004 · Errors, exit codes, JSON envelope and pipe-safe output
- status: dropped
- owner: —
- priority: P0 · difficulty: 3 · type: feature
- milestone: T-001
- why: Every command reports results and failures the same way, so agents can branch on exit codes and parse one JSON object.
- depends: 
- files: src/error.rs, src/output.rs, tests/output.rs
- read: docs/requirements.md, ../todo/src/error.rs, ../todo/src/output.rs
- accept: Error type with code, message and exit codes 0 to 5; --json prints exactly one envelope dossier.<command>.v1 for success and failure; a closed pipe ends the program quietly; output is UTF-8 on a Windows console; tests cover each exit code.
- verify: cargo test --test output
- reqs: R-OUT-1 to R-OUT-4
- note (2026-11-20, planner): added
- note (2026-11-21, planner): dropped — Replaced by the detailed v0.1 tasks under T-001 (one file per task, see docs/design.md).

### T-005 · Find the project dossier, and the where command
- status: dropped
- owner: —
- priority: P0 · difficulty: 2 · type: feature
- milestone: T-001
- why: Every command must agree on which DOSSIER folder it works on.
- depends: 
- files: src/find.rs, src/cmd/where_cmd.rs, tests/find.rs
- read: docs/requirements.md, docs/format.md
- accept: The dossier is found by walking up from the current folder; --dossier names it directly; a missing dossier is exit 4 with a message naming init; where prints the dossier, the state folder and the project code; paths with spaces and non-ASCII characters work.
- verify: cargo test --test find
- reqs: R-PLACE-1, R-PLACE-7, R-PLACE-8, NFR-4
- note (2026-11-20, planner): added
- note (2026-11-21, planner): dropped — Replaced by the detailed v0.1 tasks under T-001 (one file per task, see docs/design.md).

### T-006 · Write lock and state folder, refusing network file systems
- status: dropped
- owner: —
- priority: P0 · difficulty: 3 · type: feature
- milestone: T-001
- why: Three people and their LLMs write at the same time; the lock is what keeps entries and ID counters intact.
- depends: 
- files: src/lock.rs, src/state.rs, tests/lock.rs
- read: docs/requirements.md, ../todo/src/lock.rs, ../todo/src/state.rs
- accept: A write takes an operating-system file lock in DOSSIER/.state, waits up to lock_wait_seconds and then exits 3 naming the holder; a killed holder does not block the next process; a dossier on a network drive, UNC path, cifs, smb or nfs mount is refused unless --allow-network-drive is given.
- verify: cargo test --test lock
- reqs: R-LOCK-1 to R-LOCK-3, TEST-4
- note (2026-11-20, planner): added
- note (2026-11-21, planner): dropped — Replaced by the detailed v0.1 tasks under T-001 (one file per task, see docs/design.md).

### T-007 · Atomic file replacement with a changed-file check
- status: dropped
- owner: —
- priority: P0 · difficulty: 3 · type: feature
- milestone: T-001
- why: A crash or a hand edit during a write must never leave half an entry.
- depends: 
- files: src/atomic.rs, tests/atomic.rs
- read: docs/requirements.md, ../todo/src/atomic.rs
- accept: A file is written to a temporary file in the same folder, flushed and renamed; a file that changed since it was read makes the command exit 3 without writing; tests kill the write at each step and find either the old or the new file.
- verify: cargo test --test atomic
- reqs: R-IO-1, R-IO-2
- note (2026-11-20, planner): added
- note (2026-11-21, planner): dropped — Replaced by the detailed v0.1 tasks under T-001 (one file per task, see docs/design.md).

### T-008 · Entry model: read and change an entry file keeping every other byte
- status: dropped
- owner: —
- priority: P0 · difficulty: 4 · type: feature
- milestone: T-001
- why: All commands read and write entries through this one model; it carries the promise that hand-written files are never damaged.
- depends: 
- files: src/entry/mod.rs, src/entry/parse.rs, src/entry/write.rs, tests/entry_model.rs, tests/fixtures/
- read: docs/format.md, docs/requirements.md, ../todo/src/lib.rs
- accept: Every example in format.md parses; fields, sections, unknown keys and unknown sections are kept as written; changing one field changes only that line, with CRLF, mixed endings and a missing final line ending preserved; a file that cannot be parsed is reported with file and line and skipped by readers; fixtures are invented and marked as such.
- verify: cargo test --test entry_model
- reqs: R-ENTRY-1, R-ENTRY-2, R-EDIT-1, R-EDIT-3, D17, TEST-1, TEST-3
- note (2026-11-20, planner): added
- note (2026-11-21, planner): dropped — Replaced by the detailed v0.1 tasks under T-001 (one file per task, see docs/design.md).

### T-009 · Project settings dossier.toml, including the people and their LLMs
- status: dropped
- owner: —
- priority: P0 · difficulty: 3 · type: feature
- milestone: T-001
- why: The project code, limits and the list of people and LLM names drive who may write and approve.
- depends: 
- files: src/config.rs, tests/config.rs
- read: docs/requirements.md, docs/format.md
- accept: dossier.toml holds the project code, default tags, people with their LLM names, and the limits brief_lines, status_lines, draft_days and lock_wait_seconds with their defaults; an unknown setting or wrong value is exit 4 naming file, line and setting; people add <person> --llm <name> adds one.
- verify: cargo test --test config
- reqs: R-CFG-1, R-CFG-3, R-CFG-4 (people kept in the project until phase 3)
- note (2026-11-20, planner): added
- note (2026-11-21, planner): dropped — Replaced by the detailed v0.1 tasks under T-001 (one file per task, see docs/design.md).

### T-010 · Who is writing: agent name, person and LLM
- status: dropped
- owner: —
- priority: P0 · difficulty: 2 · type: feature
- milestone: T-001
- why: Every entry must show whether a person or an LLM wrote it and for whom; approval depends on it.
- depends: 
- files: src/names.rs, tests/names.rs
- read: docs/requirements.md
- accept: The name comes from --agent, then DOSSIER_AGENT, then TODO_AGENT; a write without a name or with a name not in the settings is refused with the known names; the name resolves to a person and, for an LLM name, the person it belongs to.
- verify: cargo test --test names
- reqs: D13, R-CFG-4, R-LIFE-2
- note (2026-11-20, planner): added
- note (2026-11-21, planner): dropped — Replaced by the detailed v0.1 tasks under T-001 (one file per task, see docs/design.md).

### T-011 · init: create a project dossier
- status: dropped
- owner: —
- priority: P0 · difficulty: 2 · type: feature
- milestone: T-001
- why: The first command anyone runs in a project.
- depends: 
- files: src/cmd/init.rs, tests/cmd_init.rs
- read: docs/requirements.md, docs/format.md
- accept: init --code <code> creates DOSSIER with the kind folders, dossier.toml and the state folder; it refuses where a dossier exists; a team dossier path is optional in this phase.
- verify: cargo test --test cmd_init
- reqs: R-PLACE-2
- note (2026-11-20, planner): added
- note (2026-11-21, planner): dropped — Replaced by the detailed v0.1 tasks under T-001 (one file per task, see docs/design.md).

### T-012 · Entry IDs per kind, given out under the lock
- status: dropped
- owner: —
- priority: P0 · difficulty: 2 · type: feature
- milestone: T-001
- why: IDs are how people and LLMs cite entries; two writers must never get the same one.
- depends: 
- files: src/ids.rs, tests/ids.rs
- read: docs/requirements.md, docs/format.md
- accept: DEC, ASM, FND and LES numbers count separately, are the highest ever given plus one including removed entries, and are never reused; an ID is accepted with or without its place prefix; 20 processes adding at once all get different IDs.
- verify: cargo test --test ids
- reqs: R-ID-1 to R-ID-4, TEST-4
- note (2026-11-20, planner): added
- note (2026-11-21, planner): dropped — Replaced by the detailed v0.1 tasks under T-001 (one file per task, see docs/design.md).

### T-013 · add and the short forms decide, found, learn and assume
- status: dropped
- owner: —
- priority: P0 · difficulty: 4 · type: feature
- milestone: T-001
- why: Recording must be one short command, or nobody records anything.
- depends: 
- files: src/cmd/add.rs, src/cmd/shortforms.rs, src/entry/kinds.rs, tests/cmd_add.rs
- read: docs/requirements.md, docs/format.md
- accept: add <kind> creates an entry from options, --from or --stdin; decide, found, found --dead-end, learn, assume and note write the right kind with required fields and sections checked; a person's entry is approved at once, an LLM's lesson is a draft, an LLM's decision needs --said-by; text with quotes, backticks and $ survives; one line such as DEC-012: added is printed.
- verify: cargo test --test cmd_add
- reqs: R-WRITE-1 to R-WRITE-4, R-WRITE-7, R-WRITE-14, R-WRITE-16, R-ENTRY-3 to R-ENTRY-7, R-ENTRY-13, R-ENTRY-15, D5
- note (2026-11-20, planner): added
- note (2026-11-20, planner): The quick note is found --later (R-WRITE-7); there is no note command (T-059).
- note (2026-11-21, planner): dropped — Replaced by the detailed v0.1 tasks under T-001 (one file per task, see docs/design.md).

### T-014 · set, retitle, link and --dry-run
- status: dropped
- owner: —
- priority: P1 · difficulty: 3 · type: feature
- milestone: T-001
- why: Entries need correcting without opening an editor, and an LLM's change to approved knowledge must not stay approved.
- depends: 
- files: src/cmd/set.rs, src/cmd/link.rs, tests/cmd_set.rs
- read: docs/requirements.md
- accept: set changes a field or section and retitle renames the file but not the ID; link adds each entry to the other's related field; an LLM changing an approved lesson turns it into a draft; --dry-run on every write command prints the change and writes nothing.
- verify: cargo test --test cmd_set
- reqs: R-WRITE-8 to R-WRITE-10
- note (2026-11-20, planner): added
- note (2026-11-21, planner): dropped — Replaced by the detailed v0.1 tasks under T-001 (one file per task, see docs/design.md).

### T-015 · Show similar existing entries when a lesson is added
- status: dropped
- owner: —
- priority: P2 · difficulty: 2 · type: feature
- milestone: T-001
- why: Without it the same lesson is written five times in slightly different words.
- depends: 
- files: src/similar.rs, src/cmd/add.rs, tests/similar.rs
- read: docs/requirements.md
- accept: Adding a lesson prints up to five entries with the most similar title and tags, rejected ones marked; --update <ID> updates one of them in the same command.
- verify: cargo test --test similar
- reqs: R-WRITE-11
- note (2026-11-20, planner): added
- note (2026-11-21, planner): dropped — Replaced by the detailed v0.1 tasks under T-001 (one file per task, see docs/design.md).

### T-016 · Journal and snapshots: history, diff, restore, log
- status: dropped
- owner: —
- priority: P0 · difficulty: 3 · type: feature
- milestone: T-001
- why: There is no git in the project folders, so the tool's own history is the only undo.
- depends: 
- files: src/history.rs, src/cmd/history.rs, tests/history.rs
- read: docs/requirements.md, ../todo/src/history.rs
- accept: Every change adds a line to journal.jsonl with time, name, person, command and entry; the old file is saved as a snapshot before it is changed; history lists what happened to an entry or task, diff shows a change, restore brings a version back for a person at a terminal, log prints the journal.
- verify: cargo test --test history
- reqs: R-HIST-1 to R-HIST-3, R-READ-6
- note (2026-11-20, planner): added
- note (2026-11-21, planner): dropped — Replaced by the detailed v0.1 tasks under T-001 (one file per task, see docs/design.md).

### T-017 · show and list with filters
- status: dropped
- owner: —
- priority: P0 · difficulty: 3 · type: feature
- milestone: T-001
- why: Reading entries back is the point of writing them.
- depends: 
- files: src/cmd/show.rs, src/cmd/list.rs, tests/cmd_read.rs
- read: docs/requirements.md
- accept: show prints the entry as in the file with marks for draft, superseded, retired and changed after approval, and --json returns fields, sections, unknown keys and links both ways; list prints one line per entry with the filters kind, status, machine, analysis, software, tag, author, task, since and where, approved entries by default.
- verify: cargo test --test cmd_read
- reqs: R-READ-1, R-READ-2, R-READ-5
- note (2026-11-20, planner): added
- note (2026-11-21, planner): dropped — Replaced by the detailed v0.1 tasks under T-001 (one file per task, see docs/design.md).

### T-018 · search: full text over titles, fields and sections
- status: dropped
- owner: —
- priority: P0 · difficulty: 3 · type: feature
- milestone: T-001
- why: Retrieval by words is how a person finds what the team knew.
- depends: 
- files: src/cmd/search.rs, tests/cmd_search.rs
- read: docs/requirements.md
- accept: search finds entries containing all words, case ignored, title matches first, then tags, then text, newest first; results show ID, title and the matching line, --full whole entries; drafts appear only with --drafts and are marked; the list filters apply.
- verify: cargo test --test cmd_search
- reqs: R-READ-3, R-READ-4, R-DRAFT-6
- note (2026-11-20, planner): added
- note (2026-11-21, planner): dropped — Replaced by the detailed v0.1 tasks under T-001 (one file per task, see docs/design.md).

### T-019 · Drafts and approval: review-task, drafts, approve, reject
- status: dropped
- owner: —
- priority: P0 · difficulty: 4 · type: feature
- milestone: T-001
- why: This is the step that keeps wrong LLM conclusions out of the team's knowledge.
- depends: 
- files: src/cmd/review.rs, src/cmd/approve.rs, src/todo_bridge.rs, tests/cmd_drafts.rs
- read: docs/requirements.md
- accept: review-task prints the task from todo show --json, linked entries and the drafting rules, and changes nothing; drafts lists drafts with proposer, task and age, old ones first; approve runs only on a terminal, shows each draft, asks approve, edit, reject or skip, and refuses LLM names; reject keeps the entry hidden; todo missing gives exit 5 with a clear message; tests use a stand-in for todo.
- verify: cargo test --test cmd_drafts
- reqs: R-DRAFT-1 to R-DRAFT-6, R-DRAFT-8, D1, D6, D7, TEST-2, TEST-5
- note (2026-11-20, planner): added
- note (2026-11-21, planner): dropped — Replaced by the detailed v0.1 tasks under T-001 (one file per task, see docs/design.md).

### T-020 · Life of an entry: statuses, supersede, retire, remove, validity dates, approval checksum
- status: dropped
- owner: —
- priority: P0 · difficulty: 4 · type: feature
- milestone: T-001
- why: Knowledge goes out of date; the dossier must show what replaced it and never silently change what an entry says.
- depends: 
- files: src/cmd/life.rs, src/entry/status.rs, tests/cmd_life.rs
- read: docs/requirements.md, docs/format.md
- accept: The status table of R-LIFE-1 is enforced; supersede links both entries and readers show the new one; retire needs a reason; remove needs a person at a terminal and journals the removal without the text; valid-from and valid-until are written by the tool; an approved lesson changed by hand is marked changed after approval in show and check.
- verify: cargo test --test cmd_life
- reqs: R-LIFE-1 to R-LIFE-6, R-ENTRY-14, D20
- note (2026-11-20, planner): added
- note (2026-11-21, planner): dropped — Replaced by the detailed v0.1 tasks under T-001 (one file per task, see docs/design.md).

### T-021 · edit: open an entry in an editor under the lock
- status: dropped
- owner: —
- priority: P1 · difficulty: 2 · type: feature
- milestone: T-001
- why: Longer entries are easier to write in an editor than in options.
- depends: 
- files: src/cmd/edit.rs, tests/cmd_edit.rs
- read: docs/requirements.md, ../todo/src/lib.rs
- accept: edit opens the entry in VISUAL, EDITOR or the system default under the lock, checks the result when the editor closes, records the change, and treats an LLM's edit of an approved lesson as in set.
- verify: cargo test --test cmd_edit
- reqs: R-EDIT-2
- note (2026-11-20, planner): added
- note (2026-11-21, planner): dropped — Replaced by the detailed v0.1 tasks under T-001 (one file per task, see docs/design.md).

### T-022 · brief and brief --task: context for a session or a task
- status: dropped
- owner: —
- priority: P0 · difficulty: 4 · type: feature
- milestone: T-001
- why: An LLM that starts without the project's decisions will contradict them; this is the context manager.
- depends: 
- files: src/cmd/brief.rs, src/select.rs, tests/cmd_brief.rs
- read: docs/requirements.md
- accept: brief prints project code and tags, approved decisions in full newest first, finding titles and the count of drafts within brief_lines, shortening older decisions and saying how many were left out; brief --task prints entries linked to the task, decisions of its milestones and entries matching its tags and title words; drafts, rejected, superseded and retired entries never appear; the selection is the same on every run and holds with 500 decisions; brief works without todo, brief --task says it needs it.
- verify: cargo test --test cmd_brief
- reqs: R-CTX-1 to R-CTX-5, TEST-9
- note (2026-11-20, planner): added
- note (2026-11-21, planner): dropped — Replaced by the detailed v0.1 tasks under T-001 (one file per task, see docs/design.md).

### T-023 · status: the state of the whole project
- status: dropped
- owner: —
- priority: P0 · difficulty: 3 · type: feature
- milestone: T-001
- why: Three LLMs working for three people must see the same picture of progress, decisions and what is unsettled.
- depends: 
- files: src/cmd/status.rs, tests/cmd_status.rs
- read: docs/requirements.md
- accept: status prints milestone goals with progress, who is working on what, blocked tasks and questions, decisions in force, newest findings, drafts waiting and the next ready tasks within status_lines; task parts come from todo --json and are left out with one line when todo is missing; --since and --milestone limit it; summary is covered by status.
- verify: cargo test --test cmd_status
- reqs: R-CTX-9 to R-CTX-11, R-READ-8
- note (2026-11-20, planner): added
- note (2026-11-20, planner): There is no summary command; status covers R-READ-8 (T-059).
- note (2026-11-21, planner): dropped — Replaced by the detailed v0.1 tasks under T-001 (one file per task, see docs/design.md).

### T-024 · decide shows related decisions and needs --replaces or --besides from an LLM
- status: dropped
- owner: —
- priority: P1 · difficulty: 2 · type: feature
- milestone: T-001
- why: Two LLMs must not record contradicting decisions without anyone noticing.
- depends: 
- files: src/cmd/shortforms.rs, tests/cmd_decide.rs
- read: docs/requirements.md
- accept: decide prints decisions in force that share tags or title words before writing; --replaces supersedes the old one in the same step; an LLM without --replaces or --besides is refused with the list shown.
- verify: cargo test --test cmd_decide
- reqs: R-CTX-13
- note (2026-11-20, planner): added
- note (2026-11-21, planner): dropped — Replaced by the detailed v0.1 tasks under T-001 (one file per task, see docs/design.md).

### T-025 · check: report problems in a dossier
- status: dropped
- owner: —
- priority: P1 · difficulty: 3 · type: feature
- milestone: T-001
- why: Hand edits and time break entries; check is how the team notices.
- depends: 
- files: src/cmd/check.rs, tests/cmd_check.rs
- read: docs/requirements.md
- accept: check reports one line per problem as file:line: ID KIND message for unparsable files, missing required fields and sections, unknown statuses, IDs not matching file names or duplicated, broken links, entries changed after approval and lessons without Valid for; exit 0 when clean, 1 otherwise.
- verify: cargo test --test cmd_check
- reqs: R-CHECK-1
- note (2026-11-20, planner): added
- note (2026-11-21, planner): dropped — Replaced by the detailed v0.1 tasks under T-001 (one file per task, see docs/design.md).

### T-026 · Agent rules: help agents, agents install, agents check
- status: dropped
- owner: —
- priority: P0 · difficulty: 3 · type: feature
- milestone: T-001
- why: Every LLM learns the tool from this one text; if it is unclear, nothing gets recorded.
- depends: 
- files: src/agents/rules.md, src/agents/mod.rs, tests/cmd_agents.rs
- read: docs/requirements.md, ../todo/src/agents/
- accept: help agents prints at most 40 lines with the phase 1 rules: brief first, status before unasked work, work only on a claimed task, decisions in force bind, decide with --said-by, found, review-task after todo done, write in English, never approve; agents install writes a managed block into AGENTS.md below todo's block keeping every other line; agents check tells whether it is current.
- verify: cargo test --test cmd_agents
- reqs: R-AGENT-1, R-AGENT-2, R-CTX-12, R-ENTRY-19, D14, D25
- note (2026-11-20, planner): added
- note (2026-11-21, planner): dropped — Replaced by the detailed v0.1 tasks under T-001 (one file per task, see docs/design.md).

### T-027 · pack and unpack: one archive of a dossier
- status: dropped
- owner: —
- priority: P1 · difficulty: 3 · type: feature
- milestone: T-001
- why: One copy on a server with no git means the backup is the only protection.
- depends: 
- files: src/cmd/pack.rs, tests/cmd_pack.rs
- read: docs/requirements.md
- accept: pack --to <file> writes the whole dossier with its state folder into one archive under the lock and never includes files that entries point to; unpack restores into an empty folder and runs check; the archive format needs no dependency outside the approved list, or the commit message gives the reason for one.
- verify: cargo test --test cmd_pack
- reqs: R-BACKUP-1, R-BACKUP-2, D24
- note (2026-11-20, planner): added
- note (2026-11-21, planner): dropped — Replaced by the detailed v0.1 tasks under T-001 (one file per task, see docs/design.md).

### T-028 · Speed test with 2,000 entries
- status: dropped
- owner: —
- priority: P1 · difficulty: 2 · type: test
- milestone: T-001
- why: No index is kept, so the speed limits must be proven before the design is relied on.
- depends: 
- files: tests/perf.rs
- read: docs/requirements.md, ../todo/tests/
- accept: With 300 project entries, brief and search finish under 300 ms and a write under 200 ms, measured as medians after a warm-up; the test runs in CI.
- verify: cargo test --test perf
- reqs: NFR-3
- note (2026-11-20, planner): added
- note (2026-11-21, planner): dropped — Replaced by the detailed v0.1 tasks under T-001 (one file per task, see docs/design.md).

### T-029 · README: purpose, what it needs, first project, use over ssh, backup
- status: dropped
- owner: —
- priority: P1 · difficulty: 3 · type: docs
- milestone: T-001
- why: The repository is public and the team has to be able to start without the author.
- depends: 
- files: README.md
- read: docs/requirements.md
- accept: README explains what the tool is for, that it needs todo 0.8 or newer with a link, a first project in ten commands, how it works with todo, how to run it on the server from a PC with a plain ssh command, that the dossier is one copy that must be in the server's backup, and how to schedule pack; examples are in PowerShell and a Linux shell.
- reqs: REL-2, R-BACKUP-3, O-7
- note (2026-11-20, planner): added
- note (2026-11-21, planner): dropped — Replaced by the detailed v0.1 tasks under T-001 (one file per task, see docs/design.md).

### T-030 · Release 0.1.0: Linux and Windows programs
- status: done
- owner: coordinator
- priority: P1 · difficulty: 2 · type: chore
- milestone: T-001
- why: Phase 1 is only finished when the team can install it on the server and try it on a real project.
- final: Cleaning up warnings across the whole crate cannot be listed in advance.
- depends: T-065,T-070,T-083,T-095,T-108,T-119,T-130,T-139,T-150
- files: src/lib.rs, Cargo.toml, README.md, src/lock.rs, src/tx.rs
- read: docs/requirements.md, docs/design.md
- steps:
  1. Remove the line #![allow(dead_code, unused_variables)] from src/lib.rs.
  2. Run cargo clippy --all-targets --all-features -- -D warnings and fix every warning by deleting code that nothing uses or by using it; do not add allow attributes.
  3. Run cargo fmt --all and cargo test --all-features.
  4. Set the version in Cargo.toml to 0.1.0 if it is not.
  5. Check by hand that every command of docs/design.md section 5 works once in a temporary folder and note anything that fails with todo add.
  6. Tell the owner that the tag v0.1.0 can be pushed; do not push it yourself.
- accept: A v0.1.0 tag builds one program for Linux x64 and one dossier.exe for Windows x64 with the C runtime linked in, with checksums; all checks pass on both systems; the program refuses a dossier written by a newer format with a message to update.
- verify: cargo clippy --all-targets --all-features -- -D warnings
- verified: 2026-11-21
- commit: 760bbc4
- reqs: NFR-1, NFR-2, NFR-7, NFR-8, REL-1
- by: Reynan Henry Tandoyo
- note (2026-11-20, planner): added
- note (2026-11-21, coordinator): Removing the allow line left three never-read fields, in src/lock.rs (the guard's file) and in test helpers of src/lock.rs and src/tx.rs; they were renamed with a leading underscore, so both files are listed now.
- note (2026-11-21, coordinator): done

### T-031 · Project facts: a result can be traced to what it was made from
- status: done
- owner: coordinator
- priority: P1 · type: milestone
- goal: The project's key numbers, the files of each task, the simulation runs and the exchange with the customer are recorded, so that for a delivered report the dossier names the model, runs, values and assumptions behind every result. Requirements: docs/requirements.md section 11, Phase 2 (v0.2). Each task here is split when the phase starts.
- by: Reynan Henry Tandoyo
- note (2026-11-20, planner): added
- note (2026-11-21, coordinator): done

### T-032 · Key values: one sheet of named numbers with units
- status: done
- owner: coordinator
- priority: P1 · type: milestone
- milestone: T-031
- goal: value set, value show and values work as in R-VALUE-1 to R-VALUE-6; brief and status print the key values.
- why: One number written in five places is soon five numbers, and an LLM cannot tell which is current.
- depends: T-030
- accept: value set, value show and values work as in R-VALUE-1 to R-VALUE-6; brief and status print the key values.
- reqs: R-VALUE-1 to R-VALUE-6, D23
- by: Reynan Henry Tandoyo
- note (2026-11-20, planner): added
- note (2026-11-21, planner): split into T-175, T-176, T-177
- note (2026-11-21, coordinator): done

### T-175 · values.rs: the key values file
- status: done
- owner: haiku-t-175
- priority: P0 · difficulty: 1 · type: feature
- milestone: T-032
- why: The key numbers live in one sheet (R-VALUE-1 to R-VALUE-4).
- depends: T-161
- files: src/values.rs
- read: docs/design.md, docs/format.md
- steps:
  1. Implement every function of docs/design.md section 8 values.rs with the format of format.md section 17 (values.md).
  2. valid_name: starts with a lower-case letter; then lower-case letters, digits, '.', '_' and '-'.
  3. on(file, name, date): the newest history line of the block whose event is 'set' and whose time's date (first 10 characters) is on or before the date; return the text after ' — '.
  4. Tests: the values.md example of format.md parses; get and all; split('12.4 kg/s', false) and split('air, ideal gas', true) and split('12.4', false) fails; on() before, between and after the two set lines.
  5. Run cargo test --lib values::.
- accept: values.md is read, and the value in force on a date is known.
- verify: cargo test --lib values::
- verified: 2026-11-21
- commit: 422daaf
- reqs: R-VALUE-1, R-VALUE-3, R-VALUE-4, R-LIFE-9
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-21, haiku-t-175): done

### T-176 · value set, show, approve and reject
- status: done
- owner: haiku-t-176
- priority: P0 · difficulty: 1 · type: feature
- milestone: T-032
- why: One number, one place, one history (R-VALUE-1 to R-VALUE-3).
- depends: T-175, T-162, T-169
- files: src/cmd/value.rs, tests/cmd_value.rs
- read: docs/design.md, docs/format.md
- steps:
  1. Read docs/design.md section 9 for the command and the existing command files of the same kind (for example src/cmd/set.rs for a change, src/cmd/list.rs for a list) and copy their patterns: Ctx::load, ctx.writer(), tx::write, Output::with with JSON data.
  2. Args: a subcommand Set { name, value, --dimensionless, --source, --note, --key, --said-by }, Show { name }, Approve { name }, Reject { name }.
  3. Set: refuse an invalid name (BadValue); split the value with values::split. It comes into force at once when the writer is a person, when an LLM gives --said-by naming a person, or when --source is a stored path attached to some task (filelist::tasks and their keys); otherwise it is proposed (R-VALUE-2).
  4. In force: a new block gets value, unit (or dimensionless: yes), source, note, key: yes when --key, valid-from <time>, and history 'set — <value>[ <unit>]'; an existing block gets those fields set and the same history line; a pending proposed field is removed. Proposed: set the field proposed '<value>[ <unit>] by <who.name> at <time>' and history 'proposed — <value>[ <unit>]'; the value in force stays.
  5. After setting a value that changed, add warnings (R-VALUE-3): 'used by <ID>' for each usage record whose Parameters section contains '{<name>}', and 'named in <ID>' for each entry whose text contains the name.
  6. Write with tx.put_raw(<dossier>/values.md, old, text, Change{id: values::ITEM, event: 'set' or 'proposed', detail: '<name>: <value>[ <unit>]'}).
  7. Show: the fields and the history lines of the block, or Code::NotFound. Approve (a person at a terminal): the proposed value comes into force as above. Reject (a person): remove proposed, history 'rejected — <proposed value>'.
  8. Tests in the test file with 'mod common;' and common::Project::new() (people anna with LLM anna-claude, ben with ben-codex; fixed time 2026-11-20T10:00:00+01:00); a terminal rule is tested with project.command(agent, now).env("DOSSIER_ANSWERS", ...). Test: anna sets design.massflow '12.4 kg/s' and the file matches format.md; anna-claude's change is proposed and the old value stays; approve with DOSSIER_ANSWERS brings it in force; a value without unit fails; --dimensionless works.
  9. Run cargo test --test cmd_value.
- accept: Key values are set, shown, proposed and approved with who and when, and their history is kept.
- verify: cargo test --test cmd_value
- verified: 2026-11-21
- commit: c13fce9
- reqs: R-VALUE-1, R-VALUE-2, R-VALUE-3
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-21, haiku-t-176): done

### T-177 · values: the values in force
- status: done
- owner: haiku-t-177
- priority: P1 · difficulty: 1 · type: feature
- milestone: T-032
- why: An LLM takes numbers from values, never from memory (R-VALUE-4, R-LIFE-9).
- depends: T-176
- files: src/cmd/values.rs, tests/cmd_values.rs
- read: docs/design.md, docs/format.md
- steps:
  1. Read docs/design.md section 9 for the command and the existing command files of the same kind (for example src/cmd/set.rs for a change, src/cmd/list.rs for a list) and copy their patterns: Ctx::load, ctx.writer(), tx::write, Output::with with JSON data.
  2. Args: prefix: Option<String>, --on <date>. Print one line per value whose name starts with the prefix: '<name>  <value>[ <unit>]  (<source>, <valid-from date>)', key values first, then by name; a pending proposal adds '  [proposed: <text>]'. With --on use values::on and leave out values that did not exist then.
  3. End with '<n> values' ('1 value'). JSON: list of {name, value, unit, source, valid_from, key, proposed}.
  4. Tests in the test file with 'mod common;' and common::Project::new() (people anna with LLM anna-claude, ben with ben-codex; fixed time 2026-11-20T10:00:00+01:00); a terminal rule is tested with project.command(agent, now).env("DOSSIER_ANSWERS", ...). Test: two values, a prefix narrows; --on before the change shows the old value.
  5. Run cargo test --test cmd_values.
- accept: values lists the values in force, also on a past date.
- verify: cargo test --test cmd_values
- verified: 2026-11-21
- commit: 802cab7
- reqs: R-VALUE-4, R-LIFE-9
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-21, haiku-t-177): done

### T-033 · Files of a task: attach, detach, reattach, files, which
- status: done
- owner: coordinator
- priority: P1 · type: milestone
- milestone: T-031
- goal: The file list of each task by role with made-from links works as in R-FILE-8 to R-FILE-16.
- why: A reader must find the simulation file, geometry and results of a task without searching the disk, and see which results are outdated.
- depends: T-030
- accept: The file list of each task by role with made-from links works as in R-FILE-8 to R-FILE-16.
- reqs: R-FILE-8 to R-FILE-16, D18
- by: Reynan Henry Tandoyo
- note (2026-11-20, planner): added
- note (2026-11-20, planner): Also covers R-XOS-5: warn about file names that cannot exist on Windows or differ only in case.
- note (2026-11-21, planner): split into T-168, T-169, T-170, T-171, T-172, T-173, T-174
- note (2026-11-21, coordinator): done

### T-168 · filesum.rs: checksums, size and time of files outside the dossier
- status: done
- owner: haiku-t-168
- priority: P0 · difficulty: 1 · type: feature
- milestone: T-033
- why: Every attached file is recorded with size, time and checksum (R-FILE-8, R-FILE-6).
- depends: T-157
- files: src/filesum.rs
- read: docs/design.md, docs/format.md
- steps:
  1. Implement checksum, stamp and count_files as docs/design.md section 8 and format.md section 18 say. Read files in chunks of 1 MiB with sha2::Sha256 (never the whole file at once). Above SHORT_ABOVE hash the first 16 MiB, the last 16 MiB and the size as 8 bytes little-endian, and prefix 'short-sha256:'.
  2. stamp: size from metadata; modified as local time in the format of format.md section 7 (chrono::DateTime<chrono::Local> from the SystemTime, formatted %Y-%m-%dT%H:%M:%S%:z).
  3. Errors are Code::Io naming the path.
  4. Tests with tempfile: the checksum of a file holding 'abc' is sha256:ba7816bf...; stamp gives the size; count_files counts files in a nested folder. Test the short method by a private helper taking the limit as a parameter, called with a small limit.
  5. Run cargo test --lib filesum::.
- accept: Files get full or short checksums, size and time, without reading big files into memory.
- verify: cargo test --lib filesum::
- verified: 2026-11-21
- commit: b5e7e8d
- reqs: R-FILE-6, R-FILE-8
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-21, haiku-t-168): done

### T-169 · filelist.rs: the file list of a task
- status: done
- owner: haiku-t-169
- priority: P0 · difficulty: 1 · type: feature
- milestone: T-033
- why: The files of a task are found without searching the disk, and outdated results are seen (R-FILE-8 to R-FILE-11).
- depends: T-161, T-168
- files: src/filelist.rs
- read: docs/design.md, docs/format.md
- steps:
  1. Implement every function of docs/design.md section 8 filelist.rs with the format of format.md section 17 (files/<task>.md). load parses with BlockFile::parse; a missing file is BlockFile::empty("Files of <TASK>") and None.
  2. describe(root, stored): for a file the fields type file, size, modified, checksum (filesum); for a folder type folder, files (count), modified. A missing path is Code::NotFound '<stored> does not exist'.
  3. marks(root, file, key): 'detached' when the field detached is set; else 'missing' when the path is gone; 'changed since attached' when the size or checksum differs from now (compare size first and only hash when the size is equal); 'outdated' when any made-from entry 'path@checksum' differs from filesum::checksum of that path now (or the path is gone); 'rests on a wrong assumption' when rests-on is not empty.
  4. Tests with tempfile: describe a file and a folder; a block for a changed file is marked changed; a made-from file that changed makes outdated; tasks() lists files/T-001.md and files/T-002.md sorted.
  5. Run cargo test --lib filelist::.
- accept: File lists load, describe files and mark missing, changed and outdated ones.
- verify: cargo test --lib filelist::
- verified: 2026-11-21
- commit: 7442f22
- reqs: R-FILE-8 to R-FILE-12
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-21, haiku-t-169): done

### T-170 · attach: add files of a task, or move one
- status: done
- owner: haiku-t-170
- priority: P0 · difficulty: 1 · type: feature
- milestone: T-033
- why: A reader finds the files of a task without searching (R-FILE-8 to R-FILE-10, R-FILE-13).
- depends: T-169, T-162
- files: src/cmd/attach.rs, tests/cmd_attach.rs
- read: docs/design.md, docs/format.md
- steps:
  1. Read docs/design.md section 9 for the command and the existing command files of the same kind (for example src/cmd/set.rs for a change, src/cmd/list.rs for a list) and copy their patterns: Ctx::load, ctx.writer(), tx::write, Output::with with JSON data.
  2. Args: task: String, paths: Vec<String>, --role <role>, --note <text>, --from <path> (repeatable), --moved (flag: then paths are exactly <old> <new>).
  3. Each path is stored with crate::paths::stored(root, cwd, path) (outside the project: refused). The role must pass filelist::valid_role (else Code::BadValue listing the roles). Warn (Output::warn) when a file name is not paths::windows_safe or two paths in the list differ only in case (R-XOS-5).
  4. Inside tx::write: load the list; for a new path add_block with role, note, the describe fields, made-from ('<from stored>@<checksum>' joined with ', ') and history 'attached'; a detached path is attached again (remove detached, refresh the fields, history 'attached'); an attached one is refused with Code::Exists 'already attached; use dossier reattach'. --moved: the old block gets detached '<time> by <name>' and history 'moved — to <new>'; a new block for the new path copies role, note and made-from, adds moved-from and the describe fields, history 'moved — from <old>'. Write with tx.put_raw(filelist::path, old bytes, text, Change{id: filelist::item(task), event: 'attached', detail: '<path>'}).
  5. Tests in the test file with 'mod common;' and common::Project::new() (people anna with LLM anna-claude, ben with ben-codex; fixed time 2026-11-20T10:00:00+01:00); a terminal rule is tested with project.command(agent, now).env("DOSSIER_ANSWERS", ...). Test: attach two files with --role model and result --from the model; the list file has both blocks with checksums; a second attach is refused; --moved keeps the history; a path outside the project fails.
  6. Run cargo test --test cmd_attach.
- accept: attach records files with role, note, size, time, checksum and what they were made from; --moved keeps the history.
- verify: cargo test --test cmd_attach
- verified: 2026-11-21
- commit: 7b02872
- reqs: R-FILE-8, R-FILE-9, R-FILE-10, R-FILE-13, R-XOS-5
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-21, haiku-t-170): done

### T-171 · detach: a file no longer belongs to a task
- status: done
- owner: haiku-t-171
- priority: P1 · difficulty: 1 · type: feature
- milestone: T-033
- why: File lists change only through the tool and keep what was true on a date (R-FILE-13, R-LIFE-9).
- depends: T-170
- files: src/cmd/detach.rs, tests/cmd_detach.rs
- read: docs/design.md, docs/format.md
- steps:
  1. Read docs/design.md section 9 for the command and the existing command files of the same kind (for example src/cmd/set.rs for a change, src/cmd/list.rs for a list) and copy their patterns: Ctx::load, ctx.writer(), tx::write, Output::with with JSON data.
  2. Args: task: String, path: String. Inside tx::write: the block must exist and not be detached (else Code::NotFound); set detached '<time> by <who.name>' and history 'detached'; put_raw with Change{id: filelist::item(task), event 'detached', detail path}.
  3. Tests in the test file with 'mod common;' and common::Project::new() (people anna with LLM anna-claude, ben with ben-codex; fixed time 2026-11-20T10:00:00+01:00); a terminal rule is tested with project.command(agent, now).env("DOSSIER_ANSWERS", ...). Test: after attach and detach the block has detached and keeps its history; detaching again fails.
  4. Run cargo test --test cmd_detach.
- accept: detach marks a file as no longer part of the task and keeps its history.
- verify: cargo test --test cmd_detach
- verified: 2026-11-21
- commit: 33dabd8
- reqs: R-FILE-13
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-21, haiku-t-171): done

### T-172 · reattach: record a file's new size, time and checksum
- status: done
- owner: haiku-t-172
- priority: P1 · difficulty: 1 · type: feature
- milestone: T-033
- why: After an intended change the list must say the file is current again (R-FILE-13).
- depends: T-170
- files: src/cmd/reattach.rs, tests/cmd_reattach.rs
- read: docs/design.md, docs/format.md
- steps:
  1. Read docs/design.md section 9 for the command and the existing command files of the same kind (for example src/cmd/set.rs for a change, src/cmd/list.rs for a list) and copy their patterns: Ctx::load, ctx.writer(), tx::write, Output::with with JSON data.
  2. Args: task: String, path: String. Inside tx::write: the block must exist and not be detached; set the describe fields again; history 'reattached' with detail 'size <old> to <new>' when the size changed, else 'checksum changed' or 'unchanged'; put_raw.
  3. Tests in the test file with 'mod common;' and common::Project::new() (people anna with LLM anna-claude, ben with ben-codex; fixed time 2026-11-20T10:00:00+01:00); a terminal rule is tested with project.command(agent, now).env("DOSSIER_ANSWERS", ...). Test: change the file, files shows it changed, reattach, the mark is gone.
  4. Run cargo test --test cmd_reattach.
- accept: reattach records an intended change of a file.
- verify: cargo test --test cmd_reattach
- verified: 2026-11-21
- commit: 6f88cf5
- reqs: R-FILE-13
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-21, haiku-t-172): done

### T-173 · files: the file lists, grouped by role, with marks
- status: done
- owner: haiku-t-173
- priority: P1 · difficulty: 1 · type: feature
- milestone: T-033
- why: Readers see which files belong to a task and which results are outdated (R-FILE-11).
- depends: T-170
- files: src/cmd/files.rs, tests/cmd_files.rs
- read: docs/design.md, docs/format.md
- steps:
  1. Read docs/design.md section 9 for the command and the existing command files of the same kind (for example src/cmd/set.rs for a change, src/cmd/list.rs for a list) and copy their patterns: Ctx::load, ctx.writer(), tx::write, Output::with with JSON data.
  2. Args: --task <task>, --role <role>, words: Vec<String>, --on <date>, --all (also detached).
  3. With --task: that list, grouped by role in the order of filelist::ROLES then other roles, each line '<path>  <note>  <modified date>  [<marks>]'. Without --task: every list (filelist::tasks), each line prefixed with the task; --role and words (all words in path or note, case ignored) narrow it.
  4. Detached files are left out unless --all. --on <date>: a block counts when its first history line is on or before the date and it was not detached on or before it; marks are left out with --on.
  5. JSON: list of {task, path, role, note, modified, marks}.
  6. Tests in the test file with 'mod common;' and common::Project::new() (people anna with LLM anna-claude, ben with ben-codex; fixed time 2026-11-20T10:00:00+01:00); a terminal rule is tested with project.command(agent, now).env("DOSSIER_ANSWERS", ...). Test: two tasks with files; --task shows one; --role result finds the result; a changed file is marked; --on before the attach shows nothing.
  7. Run cargo test --test cmd_files.
- accept: files shows the files of a task or of the project with their marks, also as they were on a date.
- verify: cargo test --test cmd_files
- verified: 2026-11-21
- commit: 5ab519e
- reqs: R-FILE-11, R-LIFE-9
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-21, haiku-t-173): done

### T-174 · which: what a file belongs to
- status: done
- owner: haiku-t-174
- priority: P1 · difficulty: 1 · type: feature
- milestone: T-033
- why: The opposite question of files (R-FILE-15).
- depends: T-170
- files: src/cmd/which.rs, tests/cmd_which.rs
- read: docs/design.md, docs/format.md
- steps:
  1. Read docs/design.md section 9 for the command and the existing command files of the same kind (for example src/cmd/set.rs for a change, src/cmd/list.rs for a list) and copy their patterns: Ctx::load, ctx.writer(), tx::write, Output::with with JSON data.
  2. Args: path: String (stored with paths::stored). Print: each task whose list has it, with role and marks; 'made from: <paths>' from its made-from; 'made into: <task> <path>' for every block whose made-from names it; 'entries: <ID> <title>' for entries whose code field has a pointer whose path part equals it (R-CODE-2).
  3. Print 'not attached to any task' when nothing is found (exit 0). JSON with the same parts.
  4. Tests in the test file with 'mod common;' and common::Project::new() (people anna with LLM anna-claude, ben with ben-codex; fixed time 2026-11-20T10:00:00+01:00); a terminal rule is tested with project.command(agent, now).env("DOSSIER_ANSWERS", ...). Test: a result made from a model: which on the model shows 'made into' the result, which on the result shows 'made from'.
  5. Run cargo test --test cmd_which.
- accept: which answers which tasks a file belongs to, what it was made from and what was made from it.
- verify: cargo test --test cmd_which
- verified: 2026-11-21
- commit: 3ec4c43
- reqs: R-FILE-15, R-CODE-2
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-21, haiku-t-174): done

### T-034 · Software use and simulation runs: used, ran, runs
- status: done
- owner: coordinator
- priority: P1 · type: milestone
- milestone: T-031
- goal: used, ran, runs and runs --compare work as in R-USE-2, R-USE-14 and R-USE-15, with software names kept in the project.
- why: Which software version and which parameters produced a result is asked in every review.
- depends: T-033
- accept: used, ran, runs and runs --compare work as in R-USE-2, R-USE-14 and R-USE-15, with software names kept in the project.
- reqs: R-USE-2, R-USE-14, R-USE-15, R-ENTRY-9
- by: Reynan Henry Tandoyo
- note (2026-11-20, planner): added
- note (2026-11-21, planner): split into T-195, T-196
- note (2026-11-21, coordinator): done

### T-195 · used: record a run of a software
- status: done
- owner: haiku-t-195
- priority: P1 · difficulty: 1 · type: feature
- milestone: T-034
- why: Which software version and parameters produced a result is asked in every review (R-USE-2, R-USE-14).
- depends: T-186, T-169, T-162
- files: src/cmd/used.rs, tests/cmd_used.rs
- read: docs/design.md, docs/format.md
- steps:
  1. Read docs/design.md section 9 for the command and the existing command files of the same kind (for example src/cmd/set.rs for a change, src/cmd/list.rs for a list) and copy their patterns: Ctx::load, ctx.writer(), tx::write, Output::with with JSON data.
  2. Args: software (positional), --version (required), --task (required), --model <path>, --param name=value (repeatable), --output <path> (repeatable), --howto <ID>, --deviations, --note, --title (default '<software> <version> run for <task>').
  3. Inside one tx::write: create a usage entry (format.md section 15) with create::create: fields software, version, task (upper case), howto, model (stored path) and model-checksum (filesum::checksum); sections Parameters ('name = value' per line), Outputs (stored path per line), Deviations, Notes.
  4. Attach in the same transaction to the task's file list: the model with role model when it is not attached yet; each output with role result and made-from '<model>@<checksum>' (a new block with the describe fields and history 'attached', or for an attached output the describe fields and made-from refreshed with history 'reattached').
  5. Message '<USE-ID>: run recorded'. JSON {id}.
  6. Tests in the test file with 'mod common;' and common::Project::new() (people anna with LLM anna-claude, ben with ben-codex; fixed time 2026-11-20T10:00:00+01:00); a terminal rule is tested with project.command(agent, now).env("DOSSIER_ANSWERS", ...). Test: used ExampleSolver --version 2406 --task T-001 --model m.sim --param p=1 --output out.csv writes USE-001 with the checksum and attaches both files with made-from.
  7. Run cargo test --test cmd_used.
- accept: used records a run in one step: the usage record and the files it read and wrote.
- verify: cargo test --test cmd_used
- verified: 2026-11-21
- commit: e6bd1db
- reqs: R-USE-2, R-USE-14, R-ENTRY-9
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-21, haiku-t-195): done

### T-196 · runs: the recorded runs, and what differs between two
- status: done
- owner: haiku-t-196
- priority: P1 · difficulty: 1 · type: feature
- milestone: T-034
- why: R-USE-15.
- depends: T-195
- files: src/cmd/runs.rs, tests/cmd_runs.rs
- read: docs/design.md, docs/format.md
- steps:
  1. Read docs/design.md section 9 for the command and the existing command files of the same kind (for example src/cmd/set.rs for a change, src/cmd/list.rs for a list) and copy their patterns: Ctx::load, ctx.writer(), tx::write, Output::with with JSON data.
  2. Args: --task, --model, --software, --compare <ID> <ID> (num_args = 2).
  3. List approved usage entries, newest created first: '<date>  <author>  <ID>  <software> <version>  <model>', then '  changed: <name> <old> -> <new>' for each parameter that differs from the previous run (in time) with the same model, and '  outputs: <paths>'. Filters compare ignoring case.
  4. --compare: print each field and each parameter that differs between the two runs as '<name>: <a> | <b>'.
  5. Tests in the test file with 'mod common;' and common::Project::new() (people anna with LLM anna-claude, ben with ben-codex; fixed time 2026-11-20T10:00:00+01:00); a terminal rule is tested with project.command(agent, now).env("DOSSIER_ANSWERS", ...). Test: two runs of the same model with a changed parameter show the change; --compare lists it.
  6. Run cargo test --test cmd_runs.
- accept: runs lists the runs and what changed between them.
- verify: cargo test --test cmd_runs
- verified: 2026-11-21
- commit: 367ebff
- reqs: R-USE-15
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-21, haiku-t-196): done

### T-035 · Pictures in entries
- status: done
- owner: coordinator
- priority: P1 · type: milestone
- milestone: T-031
- goal: --image, image add and image remove copy small pictures into the dossier and show them as in R-ENTRY-16 and R-ENTRY-17.
- why: Engineering evidence is mostly plots, and a lesson whose plot was deleted has lost its evidence.
- depends: T-030
- accept: --image, image add and image remove copy small pictures into the dossier and show them as in R-ENTRY-16 and R-ENTRY-17.
- reqs: R-ENTRY-16, R-ENTRY-17, D26
- by: Reynan Henry Tandoyo
- note (2026-11-20, planner): added
- note (2026-11-21, planner): split into T-197, T-198
- note (2026-11-21, coordinator): done

### T-197 · picture.rs: picture type, name and line
- status: done
- owner: haiku-t-197
- priority: P1 · difficulty: 1 · type: feature
- milestone: T-035
- why: R-ENTRY-16.
- depends: T-157
- files: src/picture.rs
- read: docs/design.md, docs/format.md
- steps:
  1. Implement extension, file_name and line as docs/design.md section 8 and format.md section 16 say. PNG starts with the bytes 89 50 4E 47 0D 0A 1A 0A; JPEG with FF D8 FF; SVG is UTF-8 text whose first non-blank part starts with '<?xml' or '<svg' and contains '<svg'.
  2. file_name: paths::slug of the name without extension, '.' and ext; when it exists in folder, '-2', '-3', ... before the extension.
  3. Tests: the three types and a text file; file_name with an existing file; line output.
  4. Run cargo test --lib picture::.
- accept: Pictures are recognised by content and get safe names.
- verify: cargo test --lib picture::
- verified: 2026-11-21
- commit: 5f83ee5
- reqs: R-ENTRY-16
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-21, haiku-t-197): done

### T-198 · image add and image remove
- status: done
- owner: haiku-t-198
- priority: P1 · difficulty: 1 · type: feature
- milestone: T-035
- why: A lesson whose plot was deleted has lost its evidence (D26, R-ENTRY-16, R-ENTRY-17).
- depends: T-197, T-162
- files: src/cmd/image.rs, tests/cmd_image.rs
- read: docs/design.md, docs/format.md
- steps:
  1. Read docs/design.md section 9 for the command and the existing command files of the same kind (for example src/cmd/set.rs for a change, src/cmd/list.rs for a list) and copy their patterns: Ctx::load, ctx.writer(), tx::write, Output::with with JSON data.
  2. Args: a subcommand Add { id, file, --caption (required, one line, not blank) } and Remove { id, file }.
  3. Add: read the file; refuse a size above picture::MAX_BYTES, an unknown type and an 11th picture (count the lines of the Pictures section) with Code::BadValue. Inside tx::write: name = picture::file_name(<dossier>/images/<ID>, original name, ext); tx.put_raw(that path, None, bytes, Change{id: <ID>, event 'picture', detail 'added <name>'}); add the line picture::line to the Pictures section (set_section with the old body plus the line); history 'picture' 'added <name>'; tx.put the entry. The approved-lesson rule of src/cmd/set.rs applies.
  4. Remove: the line must exist; remove it (an empty section is removed), delete the file with tx.delete_raw, history 'picture' 'removed <name>'.
  5. Tests in the test file with 'mod common;' and common::Project::new() (people anna with LLM anna-claude, ben with ben-codex; fixed time 2026-11-20T10:00:00+01:00); a terminal rule is tested with project.command(agent, now).env("DOSSIER_ANSWERS", ...). Test: a small PNG written by the test (the 8 signature bytes plus some bytes) is added with a caption; show prints the line; remove deletes the file; a text file is refused.
  6. Run cargo test --test cmd_image.
- accept: Pictures are copied into the dossier and shown by any Markdown viewer; who added them and when is recorded.
- verify: cargo test --test cmd_image
- verified: 2026-11-21
- commit: 84b3e76
- reqs: R-ENTRY-16, R-ENTRY-17
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-21, haiku-t-198): done

### T-036 · Proposed decisions: propose, accept, decline, and the deciders
- status: done
- owner: coordinator
- priority: P1 · type: milestone
- milestone: T-031
- goal: propose, accept and decline work as in R-WRITE-12, and decide follows R-WRITE-13.
- why: Three people and three LLMs must be able to suggest a direction without it becoming binding by accident.
- depends: T-030
- accept: propose, accept and decline work as in R-WRITE-12, and decide follows R-WRITE-13.
- reqs: R-WRITE-12, R-WRITE-13, D21
- by: Reynan Henry Tandoyo
- note (2026-11-20, planner): added
- note (2026-11-21, planner): split into T-186, T-187, T-188, T-189, T-190
- note (2026-11-21, coordinator): done

### T-186 · create.rs: proposals for decisions, usage records approved at once
- status: done
- owner: haiku-t-186
- priority: P0 · difficulty: 1 · type: feature
- milestone: T-036
- why: A suggestion must not become binding by accident (D21, D28, R-WRITE-13).
- depends: T-157
- files: src/create.rs, tests/cmd_supersede.rs
- read: docs/design.md, docs/format.md
- steps:
  1. Implement the create.rs rule of docs/design.md section 7: a decision written with draft true, or whose author is not allowed to decide (names::may_decide(cfg, author)), gets status proposed: no approved and valid-from fields, history lines created and proposed, Change detail proposed. A usage record is approved at once like a finding (also for an LLM without said_by).
  2. Tests in its test module: with deciders = ["anna"], ben's decision is proposed and anna's approved; a decision with draft true is proposed; with no deciders everyone decides; anna-claude's usage record is approved.
  3. Run cargo test --lib create::.
- accept: Decisions by people who may not decide, and draft decisions, become proposals; usage records are approved at once.
- verify: cargo test --lib create::
- verified: 2026-11-21
- commit: c9d6cea
- reqs: R-WRITE-12, R-WRITE-13, D28
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-21, planner): forced set (owner was haiku-t-186)
- note (2026-11-21, planner): tests/cmd_supersede.rs added: add decision --draft now writes status proposed (design.md section 7).
- note (2026-11-21, haiku-t-186): done

### T-187 · propose: suggest a decision
- status: done
- owner: haiku-t-187
- priority: P1 · difficulty: 1 · type: feature
- milestone: T-036
- why: R-WRITE-12.
- depends: T-186, T-185
- files: src/cmd/propose.rs, tests/cmd_propose.rs, src/create.rs
- read: docs/design.md, docs/format.md
- steps:
  1. Read docs/design.md section 9 for the command and the existing command files of the same kind (for example src/cmd/set.rs for a change, src/cmd/list.rs for a list) and copy their patterns: Ctx::load, ctx.writer(), tx::write, Output::with with JSON data.
  2. Copy the argument handling of src/cmd/decide.rs without --said-by, --replaces and --besides: decision (positional), --reason (required), --rejected, --title, --task and the tag options. Create with create::create and New { draft: true, said_by: None, .. }; the result is a proposal.
  3. Message '<ID>: proposed; a decider accepts it with: dossier accept <ID>'. JSON {id, status}.
  4. Tests in the test file with 'mod common;' and common::Project::new() (people anna with LLM anna-claude, ben with ben-codex; fixed time 2026-11-20T10:00:00+01:00); a terminal rule is tested with project.command(agent, now).env("DOSSIER_ANSWERS", ...). Test: anna-claude proposes without --said-by and the file has status proposed and no approved line; list does not show it, list --drafts does.
  5. Run cargo test --test cmd_propose.
- accept: Anyone, also an LLM, can suggest a decision; it is not in force until accepted.
- verify: cargo test --test cmd_propose
- verified: 2026-11-21
- commit: ac24439
- reqs: R-WRITE-12
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-21, planner): forced set (owner was haiku-t-187)
- note (2026-11-21, planner): src/create.rs added: an LLM's draft decision without said_by must be a proposal, not NeedsPerson (R-WRITE-12; design.md section 7 corrected).
- note (2026-11-21, haiku-t-187): done

### T-188 · accept: a decider puts a proposal in force
- status: done
- owner: haiku-t-188
- priority: P1 · difficulty: 1 · type: feature
- milestone: T-036
- why: R-WRITE-12, D28.
- depends: T-187
- files: src/cmd/accept.rs, tests/cmd_accept.rs
- read: docs/design.md, docs/format.md
- steps:
  1. Read docs/design.md section 9 for the command and the existing command files of the same kind (for example src/cmd/set.rs for a change, src/cmd/list.rs for a list) and copy their patterns: Ctx::load, ctx.writer(), tx::write, Output::with with JSON data.
  2. Rules: a person, at a terminal, and names::may_decide(cfg, person) (else Code::Refused '<person> is not in deciders'). The entry must have status proposed (else Code::Refused).
  3. Ask with term::ask '<ID>: accept? (yes/no) ' after showing the entry text on standard error (as approve does); any answer but yes: 'nothing changed'. Then: status approved, approved '<time> by <person>', valid-from <time>, history 'accepted'; queue with tx.put.
  4. Tests in the test file with 'mod common;' and common::Project::new() (people anna with LLM anna-claude, ben with ben-codex; fixed time 2026-11-20T10:00:00+01:00); a terminal rule is tested with project.command(agent, now).env("DOSSIER_ANSWERS", ...). Test: with deciders = ["anna"] written into dossier.toml, ben's decision is a proposal; anna accepts it with DOSSIER_ANSWERS=yes; ben is refused; anna-claude is refused.
  5. Run cargo test --test cmd_accept.
- accept: Only a decider at a terminal accepts a proposal, with who and when.
- verify: cargo test --test cmd_accept
- verified: 2026-11-21
- commit: 00144a3
- reqs: R-WRITE-12, D28
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-21, haiku-t-188): done

### T-189 · decline: a decider turns a proposal down
- status: done
- owner: haiku-t-189
- priority: P1 · difficulty: 1 · type: feature
- milestone: T-036
- why: R-WRITE-12.
- depends: T-187
- files: src/cmd/decline.rs, tests/cmd_decline.rs
- read: docs/design.md, docs/format.md
- steps:
  1. Read docs/design.md section 9 for the command and the existing command files of the same kind (for example src/cmd/set.rs for a change, src/cmd/list.rs for a list) and copy their patterns: Ctx::load, ctx.writer(), tx::write, Output::with with JSON data.
  2. Rules as accept: a person, at a terminal, a decider; status must be proposed. --reason is required. Status declined, declined-reason, history 'declined' with the reason; tx.put.
  3. Tests in the test file with 'mod common;' and common::Project::new() (people anna with LLM anna-claude, ben with ben-codex; fixed time 2026-11-20T10:00:00+01:00); a terminal rule is tested with project.command(agent, now).env("DOSSIER_ANSWERS", ...). Test: a proposal is declined with its reason; list does not show it, list --status declined does.
  4. Run cargo test --test cmd_decline.
- accept: A decider declines a proposal with a reason; it is kept, hidden.
- verify: cargo test --test cmd_decline
- verified: 2026-11-21
- commit: 09f934f
- reqs: R-WRITE-12
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-21, haiku-t-189): done

### T-190 · decide: say when a decision became a proposal
- status: done
- owner: haiku-t-190
- priority: P1 · difficulty: 1 · type: feature
- milestone: T-036
- why: R-WRITE-13: the command says so.
- depends: T-186
- files: src/cmd/decide.rs, tests/cmd_decide.rs
- read: docs/design.md, docs/format.md
- steps:
  1. When the created entry has status proposed, the message is '<ID>: proposed (<author> is not in deciders; a decider accepts it with: dossier accept <ID>)' and the JSON status is proposed. Nothing else changes.
  2. Test in tests/cmd_decide.rs: with deciders = ["anna"] in dossier.toml, ben's decide prints that message and the file has status proposed.
  3. Run cargo test --test cmd_decide and cargo test --test cmd_decide_related.
- accept: decide tells the writer when the decision is only a proposal.
- verify: cargo test --test cmd_decide
- verified: 2026-11-21
- commit: 592dd30
- reqs: R-WRITE-13
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-21, haiku-t-190): done

### T-037 · Assumptions confirmed and refuted, and what rests on them
- status: done
- owner: coordinator
- priority: P1 · type: milestone
- milestone: T-031
- goal: confirm, refute and cleared work as in R-WRITE-15, with marks shown in show, files, status and brief.
- why: Results built on a wrong assumption must be found again.
- depends: T-033
- accept: confirm, refute and cleared work as in R-WRITE-15, with marks shown in show, files, status and brief.
- reqs: R-WRITE-15
- by: Reynan Henry Tandoyo
- note (2026-11-20, planner): added
- note (2026-11-21, planner): split into T-191, T-192, T-193, T-194
- note (2026-11-21, coordinator): done

### T-191 · affects.rs: the items an assumption affects
- status: done
- owner: haiku-t-191
- priority: P1 · difficulty: 1 · type: feature
- milestone: T-037
- why: refute must find what rests on a wrong assumption (R-WRITE-15).
- depends: T-157
- files: src/affects.rs
- read: docs/design.md, docs/format.md
- steps:
  1. Implement parse as docs/design.md section 8 and format.md section 14 say: split at commas and line breaks, trim, drop empty items. An item matching T-<digits> (any case) is Task (upper-cased); one for which Kind::of_id is Some and that is a prefix, '-' and digits is Entry (upper-cased); one equal to a name in values is Value; one equal to a stored path in paths (after turning backslashes into '/') is Path; anything else Text.
  2. Tests: 'T-042, FND-004\ndesign.massflow, sim/a.sim, the inlet' gives the five kinds.
  3. Run cargo test --lib affects::.
- accept: The Affects section is turned into tasks, entries, values, paths and text.
- verify: cargo test --lib affects::
- verified: 2026-11-21
- commit: 449d71b
- reqs: R-WRITE-15
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-21, haiku-t-191): done

### T-192 · confirm: an assumption turned out true
- status: done
- owner: haiku-t-192
- priority: P1 · difficulty: 1 · type: feature
- milestone: T-037
- why: R-WRITE-15.
- depends: T-157
- files: src/cmd/confirm.rs, tests/cmd_confirm.rs
- read: docs/design.md, docs/format.md
- steps:
  1. Read docs/design.md section 9 for the command and the existing command files of the same kind (for example src/cmd/set.rs for a change, src/cmd/list.rs for a list) and copy their patterns: Ctx::load, ctx.writer(), tx::write, Output::with with JSON data.
  2. Rules: a person. The entry must be an approved assumption with state open (else Code::Refused). --by <who and how> is required.
  3. state confirmed, confirmed '<time> by <person>', section Confirmation = the --by text, history 'confirmed' with the --by text; tx.put.
  4. Tests in the test file with 'mod common;' and common::Project::new() (people anna with LLM anna-claude, ben with ben-codex; fixed time 2026-11-20T10:00:00+01:00); a terminal rule is tested with project.command(agent, now).env("DOSSIER_ANSWERS", ...). Test: an open assumption is confirmed; confirming again is refused; anna-claude is refused.
  5. Run cargo test --test cmd_confirm.
- accept: A person confirms an assumption with who, when and how.
- verify: cargo test --test cmd_confirm
- verified: 2026-11-21
- commit: 886ab21
- reqs: R-WRITE-15
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-21, haiku-t-192): done

### T-193 · refute: an assumption was wrong; mark what rests on it
- status: done
- owner: haiku-t-193
- priority: P1 · difficulty: 1 · type: feature
- milestone: T-037
- why: Results built on a wrong assumption must be found again (R-WRITE-15).
- depends: T-191, T-175, T-169, T-162
- files: src/cmd/refute.rs, tests/cmd_refute.rs
- read: docs/design.md, docs/format.md
- steps:
  1. Read docs/design.md section 9 for the command and the existing command files of the same kind (for example src/cmd/set.rs for a change, src/cmd/list.rs for a list) and copy their patterns: Ctx::load, ctx.writer(), tx::write, Output::with with JSON data.
  2. Rules: a person at a terminal; an approved assumption with state open or confirmed; --by required.
  3. Inside one tx::write: the assumption gets state wrong, refuted '<time> by <person>', valid-until <time>, section Confirmation = --by, history 'refuted'. Then parse its Affects with affects::parse, giving the value names (values::load + all) and every attached stored path (filelist::tasks and their block keys).
  4. Mark with rests-on (add the assumption ID to the comma list, history 'changed' detail 'rests on <ASM>'): for Task items every approved entry whose task field is that task and every non-detached block of that task's file list; for Entry items that entry; for Path items every block with that key in any list and every block whose made-from names it. Entries via tx.put, file lists via tx.put_raw (one put per list file).
  5. Print the assumption line, then one line per marked entry and file, then 'values named: <name>' for Value items with the usage records whose Parameters contain '{<name>}', then 'not recognised: <text>' for Text items. JSON with the same lists.
  6. Tests in the test file with 'mod common;' and common::Project::new() (people anna with LLM anna-claude, ben with ben-codex; fixed time 2026-11-20T10:00:00+01:00); a terminal rule is tested with project.command(agent, now).env("DOSSIER_ANSWERS", ...). Test: an assumption affecting T-042 and a file; entries of T-042 and the file get rests-on; refute twice is refused; without a terminal it is refused.
  7. Run cargo test --test cmd_refute.
- accept: refute closes a wrong assumption and marks every entry and file that rests on it.
- verify: cargo test --test cmd_refute
- verified: 2026-11-21
- commit: f688a32
- reqs: R-WRITE-15
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-21, haiku-t-193): done

### T-194 · cleared: a person clears a rests-on mark
- status: done
- owner: haiku-t-194
- priority: P1 · difficulty: 1 · type: feature
- milestone: T-037
- why: R-WRITE-15.
- depends: T-193
- files: src/cmd/cleared.rs, tests/cmd_cleared.rs
- read: docs/design.md, docs/format.md
- steps:
  1. Read docs/design.md section 9 for the command and the existing command files of the same kind (for example src/cmd/set.rs for a change, src/cmd/list.rs for a list) and copy their patterns: Ctx::load, ctx.writer(), tx::write, Output::with with JSON data.
  2. Args: target (an entry ID or a path), --assumption <ID> (required), --note <text> (required). Rules: a person.
  3. An entry ID (Kind::of_id is Some): remove the assumption ID from its rests-on list (set_field with the rest; empty removes the line), history 'cleared' with detail '<ASM>: <note>'; tx.put. A path: in every file list whose block for that path has the ID in rests-on, the same on the block; tx.put_raw. Nothing found: Code::NotFound.
  4. Tests in the test file with 'mod common;' and common::Project::new() (people anna with LLM anna-claude, ben with ben-codex; fixed time 2026-11-20T10:00:00+01:00); a terminal rule is tested with project.command(agent, now).env("DOSSIER_ANSWERS", ...). Test: after refute, cleared on an entry and on a file removes the mark and adds the history line.
  5. Run cargo test --test cmd_cleared.
- accept: A person clears a mark with a note; the history keeps it.
- verify: cargo test --test cmd_cleared
- verified: 2026-11-21
- commit: 3422ac7
- reqs: R-WRITE-15
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-21, haiku-t-194): done

### T-038 · What was in force on a date: --on
- status: done
- owner: coordinator
- priority: P2 · type: milestone
- milestone: T-031
- goal: --on <date> works on the commands named in R-LIFE-9.
- why: What did we know and decide on the day the report went out must have an answer.
- depends: T-033
- accept: --on <date> works on the commands named in R-LIFE-9.
- reqs: R-LIFE-9
- by: Reynan Henry Tandoyo
- note (2026-11-20, planner): added
- note (2026-11-21, planner): split into T-185
- note (2026-11-21, coordinator): done

### T-185 · filter.rs: --on <date> and proposals
- status: done
- owner: haiku-t-185
- priority: P0 · difficulty: 1 · type: feature
- milestone: T-038
- why: What did we know and decide on the day the report went out (R-LIFE-9).
- depends: T-157
- files: src/filter.rs, tests/cmd_list.rs
- read: docs/design.md, docs/format.md
- steps:
  1. Add the option --on <date> (YYYY-MM-DD) to Filter and implement it as docs/design.md section 7 says: valid-from date on or before the date, valid-until empty or its date after the date; the status rule is not applied with --on.
  2. Without --status, --drafts and --all: proposed is treated like draft (only with --drafts), declined like rejected.
  3. Unit tests in filter.rs for --on and the proposal rule; one test in tests/cmd_list.rs: a decision superseded on a later date is listed with --on before that date and not after it (use run_at for the times).
  4. Run cargo test --lib filter:: and cargo test --test cmd_list.
- accept: list and search show what was in force on a date; proposals are hidden like drafts.
- verify: cargo test --lib filter::
- verified: 2026-11-21
- commit: e1b48cd
- reqs: R-LIFE-9, R-WRITE-12
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-21, haiku-t-185): done

### T-039 · Exchange with the customer: received, delivered, exchange
- status: done
- owner: coordinator
- priority: P1 · type: milestone
- milestone: T-031
- goal: received, delivered and exchange work as in R-EXCH-1 to R-EXCH-5, including the warning before delivering outdated results.
- why: What was received and delivered, when and in which version, is the first thing asked in a dispute.
- depends: T-033
- accept: received, delivered and exchange work as in R-EXCH-1 to R-EXCH-5, including the warning before delivering outdated results.
- reqs: R-EXCH-1 to R-EXCH-5
- by: Reynan Henry Tandoyo
- note (2026-11-20, planner): added
- note (2026-11-21, planner): split into T-180, T-181, T-182, T-183, T-184
- note (2026-11-21, coordinator): done

### T-180 · exchange.rs: the exchange file
- status: done
- owner: haiku-t-180
- priority: P1 · difficulty: 1 · type: feature
- milestone: T-039
- why: What came from and went to the customer is recorded in one file (R-EXCH-3).
- depends: T-161
- files: src/exchange.rs
- read: docs/design.md, docs/format.md
- steps:
  1. Implement load and highest as docs/design.md section 8 says, with the format of format.md section 17 (exchange.md). highest: the largest number among keys '<prefix>-<digits>'.
  2. Tests: an example with RCV-001, RCV-002 and DLV-001 gives highest RCV 2 and DLV 1; a missing file is empty.
  3. Run cargo test --lib exchange::.
- accept: exchange.md is read and its record numbers are known.
- verify: cargo test --lib exchange::
- verified: 2026-11-21
- commit: 172604d
- reqs: R-EXCH-3
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-21, haiku-t-180): done

### T-181 · ids.rs: next_prefix for record numbers
- status: done
- owner: haiku-t-181
- priority: P1 · difficulty: 1 · type: feature
- milestone: T-039
- why: Records must be citable by a number that never changes (R-ID-1, R-ID-3).
- depends: T-157
- files: src/ids.rs
- read: docs/design.md, docs/format.md
- steps:
  1. Implement next_prefix as docs/design.md section 7 says: read ids.json as next() does (a broken file is Code::Parse), the number is 1 + max(ids.json value for the prefix, on_disk), write ids.json, return '<PREFIX>-<number with at least three digits>'.
  2. Tests: the first RCV is RCV-001; with on_disk 4 it is RCV-005; RCV and DLV count separately and do not disturb DEC.
  3. Run cargo test --lib ids::.
- accept: Exchange records get numbers that are never reused.
- verify: cargo test --lib ids::
- verified: 2026-11-21
- commit: e523d52
- reqs: R-ID-1, R-ID-3
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-21, haiku-t-181): done

### T-182 · received: record what came from the customer
- status: done
- owner: haiku-t-182
- priority: P1 · difficulty: 1 · type: feature
- milestone: T-039
- why: What was received, when and in which version, is the first thing asked in a dispute (R-EXCH-1).
- depends: T-180, T-181, T-169, T-162
- files: src/cmd/received.rs, tests/cmd_received.rs
- read: docs/design.md, docs/format.md
- steps:
  1. Read docs/design.md section 9 for the command and the existing command files of the same kind (for example src/cmd/set.rs for a change, src/cmd/list.rs for a list) and copy their patterns: Ctx::load, ctx.writer(), tx::write, Output::with with JSON data.
  2. Args: paths: Vec<String> (at least one), --from <who> (required), --date <YYYY-MM-DD> (default the date of time::now()), --task <task>, --role <description or reference, default reference>, --note.
  3. Inside tx::write: the key is ids::next_prefix(state, "RCV", exchange::highest(file, "RCV")); add_block with date, from, files ('<stored>@<checksum>' joined with ', '), note, history 'received'; put_raw with Change{id: exchange::ITEM, event 'received', detail key}. With --task also attach each file to that task's list as the attach command does (new block with role, describe fields, history 'attached'; already attached: leave it), in the same transaction.
  4. Message '<RCV-ID>: received <n> files from <who>'.
  5. Tests in the test file with 'mod common;' and common::Project::new() (people anna with LLM anna-claude, ben with ben-codex; fixed time 2026-11-20T10:00:00+01:00); a terminal rule is tested with project.command(agent, now).env("DOSSIER_ANSWERS", ...). Test: received spec.pdf --from 'the customer' --task T-001 writes exchange.md with RCV-001 and attaches the file; a second record is RCV-002.
  6. Run cargo test --test cmd_received.
- accept: received records the files that came from the customer, with checksums, and can attach them to a task.
- verify: cargo test --test cmd_received
- verified: 2026-11-21
- commit: 8bc8762
- reqs: R-EXCH-1
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-21, haiku-t-182): done

### T-183 · delivered: record what went to the customer
- status: done
- owner: haiku-t-183
- priority: P1 · difficulty: 1 · type: feature
- milestone: T-039
- why: The exact results behind a report version must be named later (R-EXCH-2, R-EXCH-4).
- depends: T-182
- files: src/cmd/delivered.rs, tests/cmd_delivered.rs
- read: docs/design.md, docs/format.md
- steps:
  1. Read docs/design.md section 9 for the command and the existing command files of the same kind (for example src/cmd/set.rs for a change, src/cmd/list.rs for a list) and copy their patterns: Ctx::load, ctx.writer(), tx::write, Output::with with JSON data.
  2. Args: paths (at least one), --to <who> (required), --date, --based-on <task or path> (repeatable), --note.
  3. based-on: a task ID is kept as is; a path is stored with its checksum '<stored>@<checksum>'.
  4. Warnings (R-EXCH-4): for each based-on task, its attached files marked outdated or resting on a wrong assumption (filelist::marks), and its entries with a rests-on field; for each based-on path, its marks; and every approved assumption with state open whose Affects text names a based-on task or path. With warnings: print them to standard error and ask a person at a terminal 'deliver anyway? (yes/no) ' with term::ask before tx::write; no terminal or an LLM: Code::NeedsTerminal listing the warnings; any answer but yes: 'nothing changed'.
  5. Inside tx::write: key ids::next_prefix(state, "DLV", ...); add_block date, to, files, based-on, note, history 'delivered'; put_raw.
  6. Tests in the test file with 'mod common;' and common::Project::new() (people anna with LLM anna-claude, ben with ben-codex; fixed time 2026-11-20T10:00:00+01:00); a terminal rule is tested with project.command(agent, now).env("DOSSIER_ANSWERS", ...). Test: a delivery based on a task with clean files is written without a question; with an open assumption affecting the task, the run without DOSSIER_ANSWERS fails with exit 1 and with DOSSIER_ANSWERS=yes it is written.
  7. Run cargo test --test cmd_delivered.
- accept: delivered records what went to the customer and what it was based on, and stops on outdated or questionable results unless a person confirms.
- verify: cargo test --test cmd_delivered
- verified: 2026-11-21
- commit: 406f64e
- reqs: R-EXCH-2, R-EXCH-4
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-21, haiku-t-183): done

### T-184 · exchange: list what was received and delivered
- status: done
- owner: haiku-t-184
- priority: P1 · difficulty: 1 · type: feature
- milestone: T-039
- why: R-EXCH-3.
- depends: T-183
- files: src/cmd/exchange.rs, tests/cmd_exchange.rs
- read: docs/design.md, docs/format.md
- steps:
  1. Read docs/design.md section 9 for the command and the existing command files of the same kind (for example src/cmd/set.rs for a change, src/cmd/list.rs for a list) and copy their patterns: Ctx::load, ctx.writer(), tx::write, Output::with with JSON data.
  2. Args: --received, --delivered (flags). Print every record in time order (date, then key): '<date>  <key>  from <who>' or 'to <who>', then '  <path>' per file and '  based on <item>' per based-on item. End '<n> records'. JSON: list of {key, date, direction, who, files, based_on, note}.
  3. Tests in the test file with 'mod common;' and common::Project::new() (people anna with LLM anna-claude, ben with ben-codex; fixed time 2026-11-20T10:00:00+01:00); a terminal rule is tested with project.command(agent, now).env("DOSSIER_ANSWERS", ...). Test: one received and one delivered record in date order; --delivered shows one.
  4. Run cargo test --test cmd_exchange.
- accept: exchange lists the records in time order.
- verify: cargo test --test cmd_exchange
- verified: 2026-11-21
- commit: e2494e9
- reqs: R-EXCH-3
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-21, haiku-t-184): done

### T-040 · Code projects: map of the code, pointers into code, init --code-project
- status: done
- owner: coordinator
- priority: P2 · type: milestone
- milestone: T-031
- goal: map set, map, code pointers and init --code-project work as in R-CODE-1 to R-CODE-5.
- why: The owner's own software is the first test bed, and an LLM needs orientation in thousands of files before it searches.
- depends: T-030
- accept: map set, map, code pointers and init --code-project work as in R-CODE-1 to R-CODE-5.
- reqs: R-CODE-1 to R-CODE-5
- by: Reynan Henry Tandoyo
- note (2026-11-20, planner): added
- note (2026-11-21, planner): split into T-178, T-179
- note (2026-11-21, coordinator): done

### T-178 · mapfile.rs: the map of a code project
- status: done
- owner: haiku-t-178
- priority: P1 · difficulty: 1 · type: feature
- milestone: T-040
- why: An LLM needs orientation in a code base (R-CODE-1).
- depends: T-161
- files: src/mapfile.rs
- read: docs/design.md, docs/format.md
- steps:
  1. Implement load and lines as docs/design.md section 8 says, with the format of format.md section 17 (map.md).
  2. Tests: an example map with two blocks and one detached gives two lines sorted by path, with '(start: ...)' when start is set.
  3. Run cargo test --lib mapfile::.
- accept: map.md is read and printed as lines.
- verify: cargo test --lib mapfile::
- verified: 2026-11-21
- commit: f0d9e0b
- reqs: R-CODE-1
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-21, haiku-t-178): done

### T-179 · map: describe the parts of a code project
- status: done
- owner: haiku-t-179
- priority: P1 · difficulty: 1 · type: feature
- milestone: T-040
- why: brief prints the map first in a code project (R-CODE-1).
- depends: T-178, T-162
- files: src/cmd/map.rs, tests/cmd_map.rs
- read: docs/design.md, docs/format.md
- steps:
  1. Read docs/design.md section 9 for the command and the existing command files of the same kind (for example src/cmd/set.rs for a change, src/cmd/list.rs for a list) and copy their patterns: Ctx::load, ctx.writer(), tx::write, Output::with with JSON data.
  2. Args: an optional subcommand Set { path, purpose, --start <where> } and Remove { path }. Without a subcommand print mapfile::lines (or 'no map yet; add one with: dossier map set <path> "<what it is for>"').
  3. Set: the path is stored with paths::stored and must exist (else Code::NotFound); a new block gets purpose, start and history 'set'; an existing one gets the fields set and history 'changed'; a detached one is used again. Remove: set detached, history 'detached'. Write with tx.put_raw(<dossier>/map.md, ..., Change{id: mapfile::ITEM, event, detail: path}).
  4. Tests in the test file with 'mod common;' and common::Project::new() (people anna with LLM anna-claude, ben with ben-codex; fixed time 2026-11-20T10:00:00+01:00); a terminal rule is tested with project.command(agent, now).env("DOSSIER_ANSWERS", ...). Test: set two folders, print them sorted; change one; remove one and it is gone from the print.
  5. Run cargo test --test cmd_map.
- accept: map set, map remove and map print the map of a code project with who and when.
- verify: cargo test --test cmd_map
- verified: 2026-11-21
- commit: f357321
- reqs: R-CODE-1
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-21, haiku-t-179): done

### T-041 · brief and status show values, open assumptions, proposals and dead ends; release 0.2.0
- status: done
- owner: coordinator
- priority: P1 · type: milestone
- milestone: T-031
- goal: R-CTX-16 holds; format.md and the tests cover every phase 2 addition; version 0.2.0 is released.
- why: What phase 2 records only helps when it reaches the LLM's context.
- depends: T-033
- accept: R-CTX-16 holds; format.md and the tests cover every phase 2 addition; version 0.2.0 is released.
- reqs: R-CTX-16
- by: Reynan Henry Tandoyo
- note (2026-11-20, planner): added
- note (2026-11-21, planner): split into T-199, T-200, T-201, T-202, T-203, T-204, T-205, T-206
- note (2026-11-21, coordinator): done

### T-199 · show: marks for proposals, declined entries and rests-on
- status: done
- owner: haiku-t-199
- priority: P1 · difficulty: 1 · type: feature
- milestone: T-041
- why: R-WRITE-15: marked in show.
- depends: T-157
- files: src/cmd/show.rs, tests/cmd_show.rs
- read: docs/design.md, docs/format.md
- steps:
  1. Add the marks '[proposed]', '[declined]' and '[rests on <IDs>]' (from the rests-on field) to show's marks line and JSON marks.
  2. Tests: a proposal shows [proposed]; an entry with rests-on shows its mark.
  3. Run cargo test --test cmd_show.
- accept: show marks proposals, declined entries and entries resting on a wrong assumption.
- verify: cargo test --test cmd_show
- verified: 2026-11-21
- commit: 44d48eb
- reqs: R-WRITE-15, R-READ-1
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-21, haiku-t-199): done

### T-200 · brief: map, key values, proposals, rests-on marks and --on
- status: done
- owner: haiku-t-200
- priority: P1 · difficulty: 1 · type: feature
- milestone: T-041
- why: What phase 2 records only helps when it reaches the LLM's context (R-CTX-16, R-CODE-1, R-LIFE-9).
- depends: T-175, T-178, T-185
- files: src/cmd/brief.rs, tests/cmd_brief.rs
- read: docs/design.md, docs/format.md
- steps:
  1. When map.md has lines (mapfile::load + lines), print '## Map' first (after the project lines), at most 25 lines.
  2. Print '## Key values' with at most 15 values (values::all, key values first, then by name) as '<name> = <value>[ <unit>]'.
  3. Print '## Proposals waiting' with '<ID> · <title> (<author>)' for decisions with status proposed.
  4. Add ' [rests on <IDs>]' to the line of any shown entry with a rests-on field.
  5. Add --on <date>: entries in force on the date by the rule of filter.rs (valid-from and valid-until), values by values::on; proposals and the drafts line are left out with --on.
  6. Keep the line limit: the new parts count like the others and are cut after findings and lessons.
  7. Tests: a key value and a proposal appear; a map appears first; --on before a decision leaves it out.
  8. Run cargo test --test cmd_brief.
- accept: brief shows the map, the key values, proposals waiting and what rests on wrong assumptions, also on a past date.
- verify: cargo test --test cmd_brief
- verified: 2026-11-21
- commit: 165484f
- reqs: R-CTX-16, R-CODE-1, R-LIFE-9
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-21, haiku-t-200): done

### T-201 · status: key values, proposals, marks, last exchange and --on
- status: done
- owner: haiku-t-201
- priority: P1 · difficulty: 1 · type: feature
- milestone: T-041
- why: R-CTX-16, R-EXCH-5, R-LIFE-9.
- depends: T-175, T-180, T-169, T-185
- files: src/cmd/status.rs, tests/cmd_status.rs
- read: docs/design.md, docs/format.md
- steps:
  1. Add '## Key values' (at most 15, key values first) and, under '## Unsettled', '<n> proposals waiting' and '<n> entries and files rest on a wrong assumption' (entries with rests-on plus blocks with rests-on in all file lists).
  2. Add the last received and the last delivered record (exchange::load): '## Exchange' with 'received <date> <key> from <who>' and 'delivered <date> <key> to <who>' (R-EXCH-5).
  3. Add --on <date> for the decisions and findings parts (filter.rs rule) and the values (values::on).
  4. Keep status_lines. JSON gains values, proposals, rests_on, exchange.
  5. Tests: a key value, a proposal and an exchange record appear.
  6. Run cargo test --test cmd_status.
- accept: status shows key values, proposals, marks and the last exchange, also on a past date.
- verify: cargo test --test cmd_status
- verified: 2026-11-21
- commit: 037c109
- reqs: R-CTX-16, R-EXCH-5, R-LIFE-9
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-21, haiku-t-201): done

### T-202 · brief --task: files, dead ends, pictures and code pointers
- status: done
- owner: haiku-t-202
- priority: P1 · difficulty: 1 · type: feature
- milestone: T-041
- why: R-CTX-16, R-FILE-14, R-ENTRY-17, R-CODE-4.
- depends: T-169, T-185
- files: src/brief_task.rs
- read: docs/design.md, docs/format.md
- steps:
  1. Add '## Files' after the task's own entries: the task's file list (filelist::load), descriptions and models first, then the other roles, at most 15 lines, '<path> (<role>)' plus marks; detached left out.
  2. Add '## Dead ends': select::best over approved findings with outcome dead end, with the task's tags and title words, at most 5, not already listed.
  3. For each lesson and decision printed with its text, add one line per picture of its Pictures section: '  picture: images/<ID>/<file> — <caption>' (R-ENTRY-17).
  4. When the todo data has a files field (todo 0.8 file scopes), add '## Code' with entries whose code pointers lie in those paths (R-CODE-4).
  5. Tests in the module's test with the stand-in todo: a file list, a dead end and a picture line appear.
  6. Run cargo test --lib brief_task::.
- accept: brief --task shows the task's files, matching dead ends, pictures and code pointers.
- verify: cargo test --lib brief_task::
- verified: 2026-11-21
- commit: 2fe7a71
- reqs: R-CTX-16, R-FILE-14, R-ENTRY-17, R-CODE-4
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-21, haiku-t-202): done

### T-203 · check: block files and rests-on links
- status: done
- owner: haiku-t-203
- priority: P1 · difficulty: 1 · type: feature
- milestone: T-041
- why: R-CHECK-1.
- depends: T-161
- files: src/cmd/check.rs, tests/cmd_check.rs
- read: docs/design.md, docs/format.md
- steps:
  1. Report PARSE for values.md, map.md, exchange.md and every files/*.md that BlockFile::parse refuses, as '<path>:<line>: - PARSE <message>'.
  2. Report LINK for every ID in a rests-on field (entries and file blocks) that store::get does not find.
  3. Usage records are checked like the other kinds through rules::problems (nothing to add).
  4. Tests: a broken values.md and a rests-on naming ASM-999 are reported.
  5. Run cargo test --test cmd_check.
- accept: check covers the new files and links.
- verify: cargo test --test cmd_check
- verified: 2026-11-21
- commit: e086c4f
- reqs: R-CHECK-1
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-21, haiku-t-203): done

### T-204 · The rules text for LLMs: phase 2 commands
- status: done
- owner: haiku-t-204
- priority: P1 · difficulty: 1 · type: feature
- milestone: T-041
- why: R-AGENT-1.
- depends: T-157
- files: src/agents/rules.md
- read: docs/requirements.md, docs/design.md
- steps:
  1. Add these rules, keeping the file at most 40 lines: 'Take every number from `dossier values`, never from memory.'; 'When your person is not a decider, suggest with `dossier propose`; a decider accepts it.'; 'Attach the files a task reads and writes with `dossier attach <task> <path> --role <role>`.'; 'Record a simulation run with `dossier used <software> --version <v> --task <task> --model <file> --output <path>`.'; 'Never confirm or refute an assumption, never accept or decline a proposal: a person does that.'
  2. Run cargo test --test cmd_agents (help agents prints the file).
- accept: The rules tell an LLM how to use values, proposals, file lists and runs.
- verify: cargo test --test cmd_agents
- verified: 2026-11-21
- commit: 7eb8b71
- reqs: R-AGENT-1
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-21, haiku-t-204): done

### T-205 · README for v0.2
- status: done
- owner: haiku-t-205
- priority: P1 · difficulty: 1 · type: feature
- milestone: T-041
- why: The team must be able to use phase 2 without the author (REL-2).
- depends: T-165, T-167
- files: README.md
- read: docs/requirements.md, docs/design.md, docs/format.md
- steps:
  1. Change the status line to 'Status: v0.2 (phase 2)' and move the phase 2 items from 'Not in version 0.1' to a section 'What version 0.2 adds' with one short example each (run each one yourself in a temporary folder first): key values, files of a task, runs, pictures, proposals and deciders, confirming and refuting assumptions, --on, the customer exchange, the code map, and Claude Code hooks and skill (dossier hooks install --yes, dossier skill install --yes).
  2. Keep invented names only (anna, ben, P-001).
- accept: The README explains every phase 2 feature with a tested example.
- verify: git ls-files --error-unmatch README.md
- verified: 2026-11-21
- commit: 762a59f
- reqs: REL-2
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-21, haiku-t-205): done

### T-206 · Release 0.2.0
- status: done
- owner: haiku-t-206
- priority: P1 · difficulty: 2 · type: chore
- milestone: T-041
- why: Phase 2 ends with a release that is used on real work.
- final: Checking every command by hand cannot be listed in advance.
- depends: T-199, T-200, T-201, T-202, T-203, T-204, T-205, T-032, T-033, T-034, T-035, T-036, T-037, T-038, T-039, T-040, T-155, T-156, T-157, T-158, T-159, T-160, T-161, T-162, T-163, T-164, T-165, T-166, T-167, T-207, T-208, T-209
- files: Cargo.toml, Cargo.lock
- read: docs/requirements.md
- steps:
  1. Set the version in Cargo.toml to 0.2.0 and run cargo build so Cargo.lock follows.
  2. Run every check on Windows, cargo clippy --target x86_64-unknown-linux-gnu --all-targets --all-features -- -D warnings, and run every phase 2 command once by hand in a temporary folder; note anything that fails with todo add.
  3. Tell the owner that the tag v0.2.0 can be pushed after CI passes; do not push.
- accept: Version 0.2.0 builds and every check passes on both systems.
- verify: cargo build
- verified: 2026-11-21
- commit: 4fbcf4f
- reqs: REL-1
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-21, planner): forced set (owner was haiku-t-206)
- note (2026-11-21, haiku-t-206): done

### T-042 · Team: knowledge reaches the next project without customer data
- status: done
- owner: coordinator
- priority: P1 · type: milestone
- goal: One team dossier holds lessons, how-tos, scripts, the software list and a glossary; entries are promoted from projects only after the confidentiality check; finished projects of the past are brought in. Requirements: docs/requirements.md section 11, Phase 3 (v0.3). Each task here is split when the phase starts.
- summary: Lessons, how-tos and tools from one project reach the next project through a shared team dossier, without any customer data.
- by: Reynan Henry Tandoyo
- note (2026-11-20, planner): added
- note (2026-11-21, planner): Owner 2026-11-21 (D38): dossier keeps the knowledge, todo the work. No phase 3 feature creates, changes, claims or closes tasks or writes TODO.md; tasks are read only through todo --json.
- note (2026-11-21, planner): Milestone-level depends of T-044 to T-051 removed: each small task has its own dependencies, and the old chain made a cycle (T-043 -> T-226 -> T-230 -> T-043).
- note (2026-11-22, coordinator): done

### T-043 · Team dossier: init --team-dossier, project codes, team settings, team members
- status: done
- owner: coordinator
- priority: P1 · type: milestone
- milestone: T-042
- goal: R-PLACE-3 to R-PLACE-6, R-TEAM-1, R-CFG-2 and R-CFG-4 hold; people move from the project settings to the team settings.
- why: Knowledge locked in one project folder is lost to the next project.
- summary: The shared team dossier: its folder, its members (usernames) and a list of plain names, and who may write to it.
- depends: T-041
- accept: R-PLACE-3 to R-PLACE-6, R-TEAM-1, R-CFG-2 and R-CFG-4 hold; people move from the project settings to the team settings.
- reqs: R-PLACE-3 to R-PLACE-6, R-TEAM-1, R-TEAM-7, R-CFG-2, R-CFG-4, D3, D29
- by: Reynan Henry Tandoyo
- note (2026-11-20, planner): added
- note (2026-11-21, planner): Owner 2026-11-21: one team dossier path, given with init --team. Only members write to it; team member add <person> [--llm <name>] and team member remove <person> (a person at a terminal), each saved with who and when (R-TEAM-7, D29).
- note (2026-11-21, planner): Owner 2026-11-21 (D34): the team is a list of usernames (OS logins) plus a separate list of plain names (non-users: may be author, source or --said-by, never write). An LLM is '<username> - <anything>' with no registration; without --agent/DOSSIER_AGENT the writer is the OS username; hooks install also sets DOSSIER_AGENT='<username> - claude' in .claude/settings.local.json. names.rs, people and hooks change accordingly; people add --llm keeps working. (D36) the team dossier is a git repository and dossier commits its own changes there, never pushes.
- note (2026-11-21, planner): split into T-216, T-217, T-218, T-219, T-220, T-221, T-222, T-223, T-224, T-225, T-226, T-227, T-228, T-229
- note (2026-11-22, coordinator): done

### T-216 · names.rs: usernames, plain names and '<person> - <anything>' LLM names (D34)
- status: done
- owner: haiku-t-216
- priority: P0 · difficulty: 1 · type: feature
- milestone: T-043
- why: D34: simple names, no registration of LLM names.
- summary: Lets a person write without naming themselves, and any LLM write as '<person> - <name>'.
- depends: T-212
- files: src/names.rs, tests/common/mod.rs, src/ctx.rs
- read: docs/design.md, docs/format.md
- steps:
  1. names.rs: implement resolve, is_person and is_known as docs/design.md section 10 and format.md section 21 say. The name comes from flag, DOSSIER_AGENT, TODO_AGENT, USERNAME, USER (empty counts as unset). A person is a member (cfg.members) or a [[people]] name, case ignored; Who.name and Who.person are written as in the settings. '<person> - <rest>' (split at the first ' - ', both parts trimmed and not empty) with a known person is an LLM: Who.name is the whole name as given, Who.person the person as written. A registered LLM still works. Unknown: Code::UnknownName listing members, people and 'an LLM is named <person> - <anything>'.
  2. Keep the order: a name equal to a person is the person, also when it contains ' - '.
  3. tests/common/mod.rs: in command(), when agent is empty also env_remove USERNAME and USER, so a test without a name still has no name.
  4. src/ctx.rs: in its test 'a_missing_name_...' also remove USERNAME and USER under ENV_LOCK (and restore nothing else).
  5. names.rs tests: a member, a plain name is not a writer, 'tony - claude opus 5.5 high' for member tony, an unknown '<x> - y', the USERNAME fallback (hold crate::ENV_LOCK, remove the four other variables first).
  6. Run cargo test --lib names:: and cargo test --all-features.
- accept: Writers are members, LLMs are '<member> - <anything>', the OS user is the default, and plain names are known but never write.
- verify: cargo test --lib names::
- verified: 2026-11-21
- commit: 6ffe372
- reqs: D34, R-CFG-4
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-21, haiku-t-216): done

### T-217 · config.rs: set_list and set_value keep every other line
- status: done
- owner: haiku-t-217
- priority: P0 · difficulty: 1 · type: feature
- milestone: T-043
- why: team, member and init change dossier.toml without rewriting it.
- summary: Lets commands change single settings lines without touching the rest of dossier.toml.
- depends: T-212
- files: src/config.rs
- read: docs/design.md, docs/format.md
- steps:
  1. Implement set_list and set_value as docs/design.md section 10 says. A top-level key line is 'key = ...' before the first line starting with '['. set_list writes 'key = ["a", "b"]' with the TOML string quoting already used by add_person; set_value writes 'key = <value>' as given. Replace an existing line (keeping its ending) or insert the line before the first table (with the file's line ending). Check the result still loads (load) before writing; write with std::fs::write as add_person does.
  2. Tests: set_list members on default_text, twice; set_value team with a quoted path containing a space; the lines of [tags] and [limits] stay byte-identical; a CRLF file keeps CRLF.
  3. Run cargo test --lib config::.
- accept: Top-level settings are changed line by line.
- verify: cargo test --lib config::
- verified: 2026-11-21
- commit: c03923b
- reqs: D29, R-EDIT-1
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-21, haiku-t-217): done

### T-218 · ctx.rs: load the team dossier of a project, --team and team_ctx
- status: done
- owner: haiku-t-218
- priority: P0 · difficulty: 1 · type: feature
- milestone: T-043
- why: Every team command starts from the project it is run in (R-PLACE-5, R-PLACE-8).
- summary: Lets every command in a project reach its team dossier.
- depends: T-216, T-217
- files: src/ctx.rs
- read: docs/design.md, docs/format.md
- steps:
  1. When cfg.team is set (a project), resolve it: a relative path is taken from the project folder (root); when the folder exists and has dossier.toml, Ctx.team is that path; load its settings and add its members and names to cfg.members and cfg.names in memory (no duplicates, case ignored). A team path that does not exist leaves Ctx.team None (R-PLACE-8: the project still works).
  2. With g.team: when the current dossier is a project, the context becomes the team's: dossier, root, state and cfg of the team (with the project's members and names added), project = Some(the project DOSSIER), team = None; when the team cannot be reached: Code::NoDossier '<path>: the team dossier cannot be reached'; when the current dossier is itself a team dossier (cfg.kind == "team"), --team changes nothing.
  3. Resolve who after the settings are complete (so team members can write in a project).
  4. Implement team_ctx(): Code::NoDossier when Ctx.team is None ('this project has no team dossier; link one with: dossier team use <path>'); else a Ctx for the team dossier with the same who, json, dry_run, command and from_document, and project = Some(self.dossier).
  5. Tests with tempfile folders and config::default_text plus set_value/set_list (or written TOML): a project with a team path loads the team's members; --team switches dossier and sets project; a missing team path keeps working without team; team_ctx works and fails without team.
  6. Run cargo test --lib ctx::.
- accept: A project finds its team dossier; --team works on it; team members can write in the project.
- verify: cargo test --lib ctx::
- verified: 2026-11-22
- commit: bc9e081
- reqs: R-PLACE-5, R-PLACE-8, D34
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-22, haiku-t-218): done

### T-219 · gitcommit.rs: commit the team dossier's changes
- status: done
- owner: haiku-t-219
- priority: P1 · difficulty: 1 · type: feature
- milestone: T-043
- why: D36 and R-TEAM-6: the team dossier keeps its history in git.
- summary: Commits the team dossier's changes to git when it is a git repository.
- depends: T-212
- files: src/gitcommit.rs
- read: docs/design.md, docs/format.md
- steps:
  1. Implement commit as docs/design.md section 11 says with std::process::Command (program git, args -C <repo>). First 'git -C repo rev-parse --is-inside-work-tree' (not true, or git missing: Ok(false)). Then 'git -C repo add -- <path>', then 'git -C repo diff --cached --quiet -- <path>' (exit 0: nothing staged: Ok(false)), then 'git -C repo commit -q -m <message> -- <path>' (stdin null). A failing add or commit is Code::Io with git's stderr.
  2. Tests with a tempfile folder: not a repository gives Ok(false); after 'git init' and a configured user.name/user.email in that repo (git -C dir config ...), writing a file and commit gives Ok(true) and git log shows the message; a second commit with nothing new gives Ok(false). Skip the repository tests with an early return when git is not installed.
  3. Run cargo test --lib gitcommit::.
- accept: Changes in a git work tree are committed with a message; outside one nothing happens.
- verify: cargo test --lib gitcommit::
- verified: 2026-11-21
- commit: d2ef2c6
- reqs: D36, R-TEAM-6
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-21, haiku-t-219): done

### T-220 · tx.rs: commit the team dossier after a write
- status: done
- owner: haiku-t-220
- priority: P1 · difficulty: 1 · type: feature
- milestone: T-043
- why: D36: every team change is in git without anyone running git by hand.
- summary: Makes every change to the team dossier end up as a git commit.
- depends: T-219
- files: src/tx.rs
- read: docs/design.md, docs/format.md
- steps:
  1. After every queued change of a write is applied and journalled (not on a dry run): when ctx.cfg.kind == "team" and something was written, call crate::gitcommit::commit(<parent of ctx.dossier>, &ctx.dossier, "<command>: <the change IDs joined with ', '>"). An Err becomes a warning on the returned Output ('git: <message>'), never an error; Ok(true) or Ok(false) add nothing.
  2. Test: a team context (cfg.kind team) in a tempfile folder that is not a git repository writes normally and gives no warning.
  3. Run cargo test --lib tx::.
- accept: Writes to a team dossier are committed to git when it is a repository.
- verify: cargo test --lib tx::
- verified: 2026-11-22
- commit: 036f03b
- reqs: D36, R-TEAM-6
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-22, haiku-t-220): done

### T-221 · teamfiles.rs: projects.md and usage-index.md of the team dossier
- status: done
- owner: haiku-t-221
- priority: P1 · difficulty: 1 · type: feature
- milestone: T-043
- why: init --team registers projects there; used adds usage lines there.
- summary: Reads the team dossier's list of projects and its usage index.
- depends: T-212
- files: src/teamfiles.rs
- read: docs/design.md, docs/format.md
- steps:
  1. Implement projects and usage as docs/design.md section 11 says, like values::load: the file (projects.md, usage-index.md) in the team DOSSIER folder, parsed with BlockFile::parse; a missing file is BlockFile::empty("Projects of <code>") or ("Usage of <code>") and None. Errors name the file.
  2. Tests: missing files are empty with the right title; an example of each parses.
  3. Run cargo test --lib teamfiles::.
- accept: The team's project list and usage index are read.
- verify: cargo test --lib teamfiles::
- verified: 2026-11-22
- commit: af99af2
- reqs: R-PLACE-4, R-USE-3
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-22, haiku-t-221): done

### T-222 · init --team-dossier: create a team dossier
- status: done
- owner: haiku-t-222
- priority: P1 · difficulty: 1 · type: feature
- milestone: T-043
- why: R-PLACE-3.
- summary: Creates the shared team dossier.
- depends: T-217
- files: src/cmd/init.rs, tests/cmd_init.rs
- read: docs/design.md, docs/format.md
- steps:
  1. Add #[arg(long)] team_dossier: bool. With it: create DOSSIER with the folders lessons, howtos and tools only (plus .state), dossier.toml from config::default_text with the line 'kind = "team"' after the code line, and .state/.gitignore holding 'write.lock' and 'holder.json' (two lines, LF).
  2. The journal line says detail 'team dossier, code <code>'. The output's first line is 'created the team dossier <code>'.
  3. Tests: the folders, the kind line, the .gitignore and config::load(...).kind == "team"; --team-dossier with --code-project is refused (BadValue).
  4. Run cargo test --test cmd_init.
- accept: init --team-dossier creates a team dossier.
- verify: cargo test --test cmd_init
- verified: 2026-11-22
- commit: f5d21c8
- reqs: R-PLACE-3, R-TEAM-1
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-22, haiku-t-222): done

### T-223 · team: show the team settings and link a project (team use)
- status: done
- owner: haiku-t-223
- priority: P1 · difficulty: 1 · type: feature
- milestone: T-043
- why: R-PLACE-2: a project knows its team dossier.
- summary: Links a project to the team dossier and shows the team settings.
- depends: T-218, T-222
- files: src/cmd/team.rs, tests/cmd_team.rs
- read: docs/design.md, docs/format.md
- steps:
  1. Read docs/design.md section 12 and the existing command files of the same kind (for example src/cmd/set.rs for a change, src/cmd/list.rs for a list, src/cmd/value.rs for a block file) and copy their patterns: Ctx::load, ctx.writer(), tx::write, Output::with with JSON data.
  2. Args: an optional subcommand; this task builds Use { path } and the plain 'team' (no subcommand). 'team': print 'team dossier: <path or none>', 'members: <list>', 'names: <list>' of the current dossier's settings (after Ctx::load, which adds the team's). JSON {team, members, names}.
  3. team use <path>: the path must be a team dossier folder (dossier.toml with kind team; accept the DOSSIER folder or the folder holding it); store it with config::set_value(dossier, "team", <quoted absolute path with '/'>); append a journal line (time now, who = ctx.writer()? or names::unchecked when there is none, command team, id SETTINGS, event changed, detail 'team <path>').
  4. Tests in the test file with 'mod common;' and common::Project::new() (people anna with LLM anna-claude, ben with ben-codex; fixed time 2026-11-20T10:00:00+01:00); a terminal rule is tested with project.command(agent, now).env("DOSSIER_ANSWERS", ...). To get a team dossier in a test: in a second tempfile folder run 'dossier init --team-dossier --code TEAM' (as anna, with project.command and current_dir set to that folder), add anna as a member there, then in the project run 'dossier team use <path of that DOSSIER>'. Test: team use links the project and 'team' shows it; a folder that is not a team dossier is refused.
  5. Run cargo test --test cmd_team.
- accept: A project is linked to its team dossier and the settings are shown.
- verify: cargo test --test cmd_team
- verified: 2026-11-22
- commit: 0d690ae
- reqs: R-PLACE-2, R-PLACE-5
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-22, haiku-t-223): done
- note (2026-11-22, coordinator): commit: d678d91 (the done record names the work-plan commit; batch integration bookkeeping)

### T-224 · team member add and remove
- status: done
- owner: haiku-t-224
- priority: P1 · difficulty: 1 · type: feature
- milestone: T-043
- why: R-TEAM-7: only members write to the team dossier.
- summary: Adds and removes team members.
- depends: T-223, T-220
- files: src/cmd/team.rs, tests/cmd_team.rs
- read: docs/design.md, docs/format.md
- steps:
  1. Add the subcommand Member { action } with Add { username } and Remove { username } (R-TEAM-7). They work on the team dossier when the project has one (Ctx::load with --team semantics: use ctx.team_ctx() when ctx.cfg.kind is project and a team exists), else on the current dossier.
  2. Add: refuse an empty name or one with a space (BadValue) and an existing member (Exists); config::set_list(dossier, "members", old + new). Remove: a person at a terminal (NeedsPerson, NeedsTerminal), the member must exist (NotFound); set_list without it.
  3. Each change appends a journal line in that dossier: command team, id SETTINGS, event changed, detail 'member <name> added' or 'removed'. Under a team dossier that is a git repository, commit with crate::gitcommit::commit(...) as tx.rs does (warning on failure).
  4. Tests in the test file with 'mod common;' and common::Project::new() (people anna with LLM anna-claude, ben with ben-codex; fixed time 2026-11-20T10:00:00+01:00); a terminal rule is tested with project.command(agent, now).env("DOSSIER_ANSWERS", ...). To get a team dossier in a test: in a second tempfile folder run 'dossier init --team-dossier --code TEAM' (as anna, with project.command and current_dir set to that folder), add anna as a member there, then in the project run 'dossier team use <path of that DOSSIER>'. Test: member add tony, then tony can write in the project (dossier found ... --agent tony); member remove needs DOSSIER_ANSWERS and a person.
  5. Run cargo test --test cmd_team.
- accept: Team members are added and removed with who and when.
- verify: cargo test --test cmd_team
- verified: 2026-11-22
- commit: c3fa06b2e0ceeed0ecb33af670009e62da066a42
- reqs: R-TEAM-7, D29, D34
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-22, haiku-t-224): done

### T-225 · team name add and remove: plain names
- status: done
- owner: haiku-t-225
- priority: P1 · difficulty: 1 · type: feature
- milestone: T-043
- why: D34: plain names.
- summary: Keeps a list of names of people who are not users, such as a customer contact.
- depends: T-224
- files: src/cmd/team.rs, tests/cmd_team.rs
- read: docs/design.md, docs/format.md
- steps:
  1. Add the subcommand Name { action } with Add { name } and Remove { name }, working like member add and remove on the key 'names' (format.md section 21); a plain name may contain spaces; remove needs a person at a terminal.
  2. Journal detail 'name <name> added' or 'removed'.
  3. Tests in the test file with 'mod common;' and common::Project::new() (people anna with LLM anna-claude, ben with ben-codex; fixed time 2026-11-20T10:00:00+01:00); a terminal rule is tested with project.command(agent, now).env("DOSSIER_ANSWERS", ...). Test: name add "Dr. Weber" then decide ... --said-by "Dr. Weber" as anna-claude is accepted (names::is_known), and --agent "Dr. Weber" cannot write.
  4. Run cargo test --test cmd_team.
- accept: Plain names are kept for authors and sources who are not users.
- verify: cargo test --test cmd_team
- verified: 2026-11-22
- commit: bc927baba4f75aa5c3f39d90061fdf0fcdd6a97a
- reqs: D34
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-22, haiku-t-225): done

### T-226 · create.rs: --said-by and author accept plain names, and team writes are checked
- status: done
- owner: haiku-t-226
- priority: P0 · difficulty: 1 · type: feature
- milestone: T-043
- why: D34 and R-CONF-2: the check before every team write.
- summary: Lets a plain name be the source of a decision and blocks confidential terms on their way to the team dossier.
- depends: T-218, T-230
- files: src/create.rs
- read: docs/design.md, docs/format.md
- steps:
  1. said_by: accept names::is_known (a person or a plain name) instead of is_person; the author is that name as written in the settings.
  2. When tx.ctx().project is Some (a team write from a project), before anything is queued: let terms = conf::gate(project, &project code)? where the project code is read with config::load(project)?.code, and conf::check(&terms, &rendered text)? (design.md section 10).
  3. Tests in its test module: said_by a plain name; a team write with a term in the title is refused (build the confidential.md by hand with BlockFile text); a team write from a project without terms and without the none statement is refused.
  4. Run cargo test --lib create::.
- accept: Plain names can be the person a decision came from; nothing reaches the team dossier with a confidential term.
- verify: cargo test --lib create::
- verified: 2026-11-22
- commit: d886862
- reqs: D34, R-CONF-2, R-CONF-5, R-WRITE-6
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-22, haiku-t-226): done
- note (2026-11-22, coordinator): commit: e091c7b (the done record names the work-plan commit; batch integration bookkeeping)

### T-227 · init --with-team: link a new project and register its code
- status: done
- owner: haiku-t-227
- priority: P1 · difficulty: 1 · type: feature
- milestone: T-043
- why: R-PLACE-2 and R-PLACE-4: project codes are unique in the team.
- summary: Links a new project to the team dossier and registers its code there.
- depends: T-222, T-223, T-221
- files: src/cmd/init.rs, tests/cmd_init.rs
- read: docs/design.md, docs/format.md
- steps:
  1. Add #[arg(long = "with-team")] with_team: Option<String>: the path of the team dossier (the global --team flag is a bool, so init uses --with-team for the path). After the project dossier is created: store team = <path> as the team use command of src/cmd/team.rs does.
  2. Register the code in the team's projects.md (R-PLACE-4) through tx::write on the team context (Ctx::load with a Global whose dossier is the team DOSSIER path, and agent g.agent): a new block key <code> with folder = <the project folder, absolute, with '/'> and history 'registered'. A code already registered for another folder: refuse before anything is created (Code::Exists '<code> is already registered for <folder>'). The same folder: nothing to add.
  3. Tests in the test file with 'mod common;' and common::Project::new() (people anna with LLM anna-claude, ben with ben-codex; fixed time 2026-11-20T10:00:00+01:00); a terminal rule is tested with project.command(agent, now).env("DOSSIER_ANSWERS", ...). To get a team dossier in a test: in a second tempfile folder run 'dossier init --team-dossier --code TEAM' (as anna, with project.command and current_dir set to that folder), add anna as a member there, then in the project run 'dossier team use <path of that DOSSIER>'. Test: init --code P-002 with the team writes team = and a projects.md block; a second project with the same code in another folder is refused.
  4. Run cargo test --test cmd_init.
- accept: A new project is linked to its team dossier and its code registered there.
- verify: cargo test --test cmd_init
- verified: 2026-11-22
- commit: ce14fb5e8d630b5cbcb8deef4131640a54118eb6
- reqs: R-PLACE-2, R-PLACE-4
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-22, haiku-t-227): done

### T-228 · hooks.rs: DOSSIER_AGENT in .claude/settings.local.json
- status: done
- owner: haiku-t-228
- priority: P1 · difficulty: 1 · type: feature
- milestone: T-043
- why: D34: Claude's writes are always marked as an LLM's.
- summary: Sets the name Claude writes under in your personal Claude Code settings.
- depends: T-212
- files: src/hooks.rs
- read: docs/design.md, docs/format.md
- steps:
  1. Implement local_installed and local_uninstalled as docs/design.md section 10 says, like installed and uninstalled (serde_json, byte-order mark skipped, to_string_pretty and a final LF): set env.DOSSIER_AGENT to the given agent (replacing an older value), keep every other key; uninstall removes only env.DOSSIER_AGENT and an env object that became empty.
  2. Tests: into {}, into a file with other env keys, twice, uninstall.
  3. Run cargo test --lib hooks::.
- accept: The personal Claude Code settings carry the LLM name.
- verify: cargo test --lib hooks::
- verified: 2026-11-22
- commit: 16f941a
- reqs: D34
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-22, haiku-t-228): done

### T-229 · hooks install: also set the LLM name for Claude Code
- status: done
- owner: haiku-t-229
- priority: P1 · difficulty: 1 · type: feature
- milestone: T-043
- why: D34.
- summary: Makes Claude Code sessions write under '<you> - claude' automatically.
- depends: T-228, T-216
- files: src/cmd/hooks.rs, tests/cmd_hooks.rs
- read: docs/design.md, docs/format.md
- steps:
  1. hooks install --yes also writes .claude/settings.local.json with hooks::local_installed(existing, "<username> - claude"), where the username is the person of ctx.writer() when a name resolves, else names::unchecked(None).person; uninstall --yes also applies local_uninstalled. The output names both files. Without --yes it says both would change.
  2. Tests: install writes both files and settings.local.json has env.DOSSIER_AGENT 'anna - claude' when run as anna; uninstall removes it.
  3. Run cargo test --test cmd_hooks.
- accept: hooks install marks Claude Code's writes as an LLM's.
- verify: cargo test --test cmd_hooks
- verified: 2026-11-22
- commit: 28d49a2
- reqs: D34
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-22, haiku-t-229): done

### T-044 · Confidentiality: confidential terms and the check before any team write
- status: done
- owner: coordinator
- priority: P0 · type: milestone
- milestone: T-042
- goal: R-CONF-1 to R-CONF-6 and R-ENTRY-18 hold, with the tests of TEST-6.
- why: The team works for several customers under confidentiality agreements; this must exist before anything is promoted.
- summary: Each project lists its confidential terms; nothing is written to the team dossier while one of them appears in the text.
- accept: R-CONF-1 to R-CONF-6 and R-ENTRY-18 hold, with the tests of TEST-6.
- reqs: R-CONF-1 to R-CONF-6, R-ENTRY-18, D8, D32, TEST-6
- by: Reynan Henry Tandoyo
- note (2026-11-20, planner): added
- note (2026-11-21, planner): Owner 2026-11-21 (D32): anyone in the project adds a confidential term, also an LLM when its person tells it in the session (the rules text must say: add it at once with dossier confidential add); confidential remove needs a person at a terminal. No automatic terms; writing to the team dossier is refused until the project has at least one term or a person ran confidential none --yes.
- note (2026-11-21, planner): split into T-230, T-231, T-232, T-233
- note (2026-11-22, coordinator): done

### T-230 · conf.rs: confidential terms and the check
- status: done
- owner: haiku-t-230
- priority: P0 · difficulty: 1 · type: feature
- milestone: T-044
- why: R-CONF-1, R-CONF-2, R-CONF-5.
- summary: Finds a project's confidential terms in any text, however they are spelled.
- depends: T-212
- files: src/conf.rs
- read: docs/design.md, docs/format.md
- steps:
  1. Implement every function of docs/design.md section 11 conf.rs with format.md section 24. normalize: lower case, then every run of space, '-' and '_' becomes one space, trimmed. find: for each term and each line (1-based) of text, the normalized line contains the normalized term; return (term as written, line) pairs in line order.
  2. gate: load confidential.md of the project; terms() not empty: Ok(terms); stated_none: Ok(empty); else Code::Refused 'this project has no confidential terms yet; add them with: dossier confidential add "<term>", or let a person state there are none: dossier confidential none --yes'.
  3. check: Code::Refused listing '<term> (line <n>)' for each hit, joined with '; '.
  4. Tests: matching 'NT-450' against 'the nt 450 stage' and 'NT_450'; case; gate refuses an empty list and accepts the none statement; check names lines.
  5. Run cargo test --lib conf::.
- accept: Confidential terms are matched ignoring case, spaces, hyphens and underscores.
- verify: cargo test --lib conf::
- verified: 2026-11-22
- commit: da47711
- reqs: R-CONF-1, R-CONF-2, R-CONF-5
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-22, haiku-t-230): done

### T-231 · confidential add and list
- status: done
- owner: haiku-t-231
- priority: P0 · difficulty: 1 · type: feature
- milestone: T-044
- why: R-CONF-1, D32.
- summary: Lets anyone, also an LLM, add a confidential term of the project.
- depends: T-230, T-218
- files: src/cmd/confidential.rs, tests/cmd_confidential.rs
- read: docs/design.md, docs/format.md
- steps:
  1. Read docs/design.md section 12 and the existing command files of the same kind (for example src/cmd/set.rs for a change, src/cmd/list.rs for a list, src/cmd/value.rs for a block file) and copy their patterns: Ctx::load, ctx.writer(), tx::write, Output::with with JSON data.
  2. Args: a subcommand Add { term }, List (this task); later tasks add Remove, None and Scan. confidential.md is <DOSSIER>/confidential.md, loaded with conf::load.
  3. Add (anyone, also an LLM, D32): a term must not be blank; an existing term (normalized equal) is refused (Exists); a removed term is added again (remove the removed field). add_block(term, []) then add_history 'added'; tx.put_raw with Change{id: "CONFIDENTIAL", event 'added', detail term}.
  4. List: the terms, one per line, then 'no confidential terms stated by <who and when>' when stated, or '<n> terms'. JSON {terms, none_stated}.
  5. Tests in the test file with 'mod common;' and common::Project::new() (people anna with LLM anna-claude, ben with ben-codex; fixed time 2026-11-20T10:00:00+01:00); a terminal rule is tested with project.command(agent, now).env("DOSSIER_ANSWERS", ...). Test: anna-claude adds 'NT-450'; list shows it; adding 'nt 450' again is refused.
  6. Run cargo test --test cmd_confidential.
- accept: Anyone in the project adds confidential terms, with who and when.
- verify: cargo test --test cmd_confidential
- verified: 2026-11-22
- commit: 42f786a
- reqs: R-CONF-1, D32
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-22, haiku-t-231): done

### T-232 · confidential remove and confidential none
- status: done
- owner: haiku-t-232
- priority: P1 · difficulty: 1 · type: feature
- milestone: T-044
- why: D32, R-CONF-5.
- summary: Lets a person remove a term or state that the project has none.
- depends: T-231
- files: src/cmd/confidential.rs, tests/cmd_confidential.rs
- read: docs/design.md, docs/format.md
- steps:
  1. Add Remove { term } and None { --yes }. Both need a person at a terminal (NeedsPerson, NeedsTerminal) (D32).
  2. Remove: the term must exist and not be removed; set field removed '<time> by <person>', history 'removed'. None: needs --yes (Usage otherwise) and no terms in the list (Refused 'remove the terms first'); block conf::NONE with field stated '<time> by <person>' and history 'stated'.
  3. Tests in the test file with 'mod common;' and common::Project::new() (people anna with LLM anna-claude, ben with ben-codex; fixed time 2026-11-20T10:00:00+01:00); a terminal rule is tested with project.command(agent, now).env("DOSSIER_ANSWERS", ...). Test: remove with DOSSIER_ANSWERS as anna; anna-claude is refused; none --yes on an empty list; none with a term is refused.
  4. Run cargo test --test cmd_confidential.
- accept: Only a person removes a term or states there are none.
- verify: cargo test --test cmd_confidential
- verified: 2026-11-22
- commit: 6f5b55946a001c5c6a9c5a8cab95f2eab3b11bf7
- reqs: D32, R-CONF-5
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-22, haiku-t-232): done

### T-233 · confidential scan: look for the project's terms in the team dossier
- status: done
- owner: haiku-t-233
- priority: P1 · difficulty: 1 · type: feature
- milestone: T-044
- why: R-CONF-6.
- summary: Searches the team dossier for a project's confidential terms that were added late.
- depends: T-232
- files: src/cmd/confidential.rs, tests/cmd_confidential.rs
- read: docs/design.md, docs/format.md
- steps:
  1. Add Scan: for every entry and block file of the team dossier (ctx.team_ctx()?: store::all plus glossary.md and usage-index.md text), conf::find the project's terms; print '<team path>:<line>: <term>' per hit and '<n> matches'; exit 1 (Refused) when there are matches so scripts notice (R-CONF-6).
  2. Tests in the test file with 'mod common;' and common::Project::new() (people anna with LLM anna-claude, ben with ben-codex; fixed time 2026-11-20T10:00:00+01:00); a terminal rule is tested with project.command(agent, now).env("DOSSIER_ANSWERS", ...). To get a team dossier in a test: in a second tempfile folder run 'dossier init --team-dossier --code TEAM' (as anna, with project.command and current_dir set to that folder), add anna as a member there, then in the project run 'dossier team use <path of that DOSSIER>'. Test: a team lesson containing a term is found; a clean team dossier gives '0 matches'.
  3. Run cargo test --test cmd_confidential.
- accept: Terms added late are found in the team dossier.
- verify: cargo test --test cmd_confidential
- verified: 2026-11-22
- commit: ec9cde9de892047a1157896ae6419f9f51e17ec6
- reqs: R-CONF-6
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-22, haiku-t-233): done

### T-045 · Promotion and closing a project: promote, close
- status: done
- owner: coordinator
- priority: P1 · type: milestone
- milestone: T-042
- goal: promote and close work as in R-TEAM-2 to R-TEAM-4 and R-TEAM-6; search and brief include the team dossier.
- why: Promotion is the only way project knowledge becomes team knowledge.
- summary: A project's lesson or how-to is copied into the team dossier as a draft; at the end of a project, its knowledge is sorted out.
- accept: promote and close work as in R-TEAM-2 to R-TEAM-4 and R-TEAM-6; search and brief include the team dossier.
- reqs: R-TEAM-2 to R-TEAM-4, R-TEAM-6, R-WRITE-5, R-WRITE-6, D28, D30
- by: Reynan Henry Tandoyo
- note (2026-11-20, planner): added
- note (2026-11-21, planner): Owner 2026-11-21 (D30): only knowledge (entry text) is promoted to the team dossier; simulation files, CAD parts, spreadsheets and scripts are never copied, only pointed to from their project (small pictures are the exception, confirmed one by one, R-ENTRY-18). The team copy says source: <code>:<ID>, the project entry promoted: team:<ID>. close is run by a person in deciders (D28).
- note (2026-11-21, planner): split into T-234, T-235, T-236
- note (2026-11-22, coordinator): done

### T-234 · create.rs: imported drafts and approval checksums for every guarded kind
- status: done
- owner: haiku-t-234
- priority: P0 · difficulty: 1 · type: feature
- milestone: T-045
- why: R-IMPORT-5, R-LIFE-6, R-WRITE-5.
- summary: Makes entries from old documents drafts and gives how-tos and tools the same approval rules as lessons.
- depends: T-226
- files: src/create.rs
- read: docs/design.md, docs/format.md
- steps:
  1. When tx.ctx().from_document is Some: the entry is a draft whoever writes it, with the fields imported: yes and source-document: <stored path> (format.md section 23, R-IMPORT-3, R-IMPORT-5).
  2. approval::record for every approved entry of a guarded kind (approval::guarded), not only lessons. An LLM writing a how-to or a tool without said_by: draft, as for a lesson.
  3. Tests in its module: a decision with from_document is a draft with imported yes; an approved how-to by a person records its checksum; an LLM's tool is a draft.
  4. Run cargo test --lib create::.
- accept: Imported entries are drafts; how-tos and tools get the lesson rules.
- verify: cargo test --lib create::
- verified: 2026-11-22
- commit: 2bc1275133fe1edec6b628d163791e6ecaf7b763
- reqs: R-IMPORT-5, R-LIFE-6, R-WRITE-5
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-22, haiku-t-234): done

### T-235 · promote: copy a project entry into the team dossier
- status: done
- owner: haiku-t-235
- priority: P0 · difficulty: 2 · type: feature
- milestone: T-045
- why: R-TEAM-2, R-TEAM-3, R-ENTRY-18.
- summary: Copies a lesson or how-to from a project to the team dossier after the confidentiality check.
- final: Two dossiers, the confidentiality check and pictures interact; the order is given, details are settled in code.
- depends: T-234, T-231
- files: src/cmd/promote.rs, tests/cmd_promote.rs
- read: docs/design.md, docs/format.md
- steps:
  1. Read docs/design.md section 12 and the existing command files of the same kind (for example src/cmd/set.rs for a change, src/cmd/list.rs for a list, src/cmd/value.rs for a block file) and copy their patterns: Ctx::load, ctx.writer(), tx::write, Output::with with JSON data.
  2. Args: id, --as <lesson|howto> (default: the entry's own kind; a finding becomes a lesson; decisions, assumptions and usage records are refused: Refused 'only findings, lessons and how-tos are promoted'). Also make pub fn promote_one(ctx: &Ctx, id: &str, as_kind: Option<Kind>) -> Result<String> (the new team ID) for the close command.
  3. Check first: let terms = conf::gate(&ctx.dossier, &ctx.cfg.code)?; conf::check(&terms, <the entry text>)? (R-TEAM-3). Pictures (R-ENTRY-18): when the entry has a Pictures section, a person at a terminal must confirm each line with term::ask 'copy picture <file> (<caption>) to the team? (yes/no)'; an LLM or no terminal: Refused; a no leaves that picture out.
  4. Write the team copy through tx::write on ctx.team_ctx()?: create::create with the same title, the sections (a finding's Observation becomes Statement, Evidence stays), the tag fields, source '<code>:<ID>', draft true. Copy the confirmed pictures with put_raw into the team's images/<new ID>/. Then, in a tx::write on the project, set promoted 'team:<new ID>' on the project entry with history 'promoted' detail 'team:<new ID>'.
  5. Message '<ID>: promoted as team:<new ID> (draft, waiting for approval in the team dossier)'.
  6. Tests in the test file with 'mod common;' and common::Project::new() (people anna with LLM anna-claude, ben with ben-codex; fixed time 2026-11-20T10:00:00+01:00); a terminal rule is tested with project.command(agent, now).env("DOSSIER_ANSWERS", ...). To get a team dossier in a test: in a second tempfile folder run 'dossier init --team-dossier --code TEAM' (as anna, with project.command and current_dir set to that folder), add anna as a member there, then in the project run 'dossier team use <path of that DOSSIER>'. Add a confidential term first ('Nordwind'). Test: a lesson is promoted, the team copy is a draft with source P-001:LES-001, the project entry has promoted; a lesson containing 'Nordwind' is refused naming the line; a project with no terms is refused.
  7. Run cargo test --test cmd_promote.
- accept: Project knowledge reaches the team dossier as a checked draft, linked both ways.
- verify: cargo test --test cmd_promote
- verified: 2026-11-22
- commit: 57c50140d8526bb370135cf6b4a0f800245a34e3
- reqs: R-TEAM-2, R-TEAM-3, R-ENTRY-18
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-22, haiku-t-235): done

### T-236 · close: sort out a finished project's knowledge
- status: done
- owner: haiku-t-236
- priority: P1 · difficulty: 1 · type: feature
- milestone: T-045
- why: R-TEAM-4.
- summary: Goes through a finished project's knowledge and asks what goes to the team.
- depends: T-235
- files: src/cmd/close.rs, tests/cmd_close.rs
- read: docs/design.md, docs/format.md
- steps:
  1. Read docs/design.md section 12 and the existing command files of the same kind (for example src/cmd/set.rs for a change, src/cmd/list.rs for a list, src/cmd/value.rs for a block file) and copy their patterns: Ctx::load, ctx.writer(), tx::write, Output::with with JSON data.
  2. Rules: a person at a terminal who may decide (names::may_decide) (R-TEAM-4, D28).
  3. List every approved finding, lesson and how-to without a promoted field and without promote: no. For each: show its text on standard error and ask '<ID>: promote, keep in the project, or later? (p/k/l) '. p: crate::cmd::promote::promote_one(&ctx, id, None); k: set promote: no with history 'changed' detail 'kept in the project' (tx.put); l or anything else: nothing.
  4. End with '<n> promoted, <m> kept, <k> left for later'. A second run shows only what is left.
  5. Tests in the test file with 'mod common;' and common::Project::new() (people anna with LLM anna-claude, ben with ben-codex; fixed time 2026-11-20T10:00:00+01:00); a terminal rule is tested with project.command(agent, now).env("DOSSIER_ANSWERS", ...). To get a team dossier in a test: in a second tempfile folder run 'dossier init --team-dossier --code TEAM' (as anna, with project.command and current_dir set to that folder), add anna as a member there, then in the project run 'dossier team use <path of that DOSSIER>'. Test: with DOSSIER_ANSWERS 'k|l', the first entry gets promote: no and the second stays; a second run asks only for the second.
  6. Run cargo test --test cmd_close.
- accept: At the end of a project every piece of knowledge is promoted, kept or left for later, and the answers are kept.
- verify: cargo test --test cmd_close
- verified: 2026-11-22
- commit: fc625e2b5e1745dfd880810c0716861d5eea5716
- reqs: R-TEAM-4, D28
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-22, haiku-t-236): done

### T-046 · How-tos and the usage index: howto, where-used (software names come from tools, T-047)
- status: done
- owner: coordinator
- priority: P1 · type: milestone
- milestone: T-042
- goal: R-USE-1, R-USE-3 to R-USE-7 and R-ENTRY-8 hold.
- why: How and where a software was used for a task is asked at the start of every similar task.
- summary: How-tos say how a software is used for a task, and a team index shows where and with which version it was used.
- accept: R-USE-1, R-USE-3 to R-USE-7 and R-ENTRY-8 hold.
- reqs: R-USE-3 to R-USE-7, R-ENTRY-8, R-ENTRY-9, D31
- by: Reynan Henry Tandoyo
- note (2026-11-20, planner): added
- note (2026-11-21, planner): Owner 2026-11-21 (D31): there is no separate software list; software is a tool of type software (T-047). The usage index and how-tos name tools by ID or name.
- note (2026-11-21, planner): Owner 2026-11-21 (D35): software names stay free text for now; no refusal of unknown names.
- note (2026-11-21, planner): split into T-237, T-238, T-239, T-240
- note (2026-11-22, coordinator): done

### T-237 · howto: write a how-to
- status: done
- owner: haiku-t-237
- priority: P1 · difficulty: 1 · type: feature
- milestone: T-046
- why: R-ENTRY-8, R-WRITE-5, R-WRITE-6.
- summary: Records how a software is used for a task, in the project or for the whole team.
- depends: T-234
- files: src/cmd/howto.rs, tests/cmd_howto.rs
- read: docs/design.md, docs/format.md
- steps:
  1. Read docs/design.md section 12 and the existing command files of the same kind (for example src/cmd/set.rs for a change, src/cmd/list.rs for a list, src/cmd/value.rs for a block file) and copy their patterns: Ctx::load, ctx.writer(), tx::write, Output::with with JSON data.
  2. Copy src/cmd/add.rs (on main) for the options: --software (required), --title (required), --section 'Name=text' (repeatable), --from <file>, --stdin, --review <YYYY-MM-DD>, --task and the tag options. Kind::Howto; field software; field review when given (valid date, else BadValue).
  3. The global --team writes it into the team dossier (Ctx::load does that); create.rs checks the confidential terms. A person's how-to is approved, an LLM's a draft (R-WRITE-5).
  4. Tests in the test file with 'mod common;' and common::Project::new() (people anna with LLM anna-claude, ben with ben-codex; fixed time 2026-11-20T10:00:00+01:00); a terminal rule is tested with project.command(agent, now).env("DOSSIER_ANSWERS", ...). To get a team dossier in a test: in a second tempfile folder run 'dossier init --team-dossier --code TEAM' (as anna, with project.command and current_dir set to that folder), add anna as a member there, then in the project run 'dossier team use <path of that DOSSIER>'. Test: anna writes a how-to with Purpose and Steps (approved); anna-claude's is a draft; --team puts it into the team's howtos/ folder; a how-to without software fails.
  5. Run cargo test --test cmd_howto.
- accept: How-tos are written in the project or, with --team, in the team dossier.
- verify: cargo test --test cmd_howto
- verified: 2026-11-22
- commit: 56061b823677e1291039c5840ae914195acdb585
- reqs: R-ENTRY-8, R-WRITE-5, R-WRITE-6
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-22, haiku-t-237): done

### T-238 · used: add a line to the team's usage index
- status: done
- owner: haiku-t-238
- priority: P1 · difficulty: 1 · type: feature
- milestone: T-046
- why: R-USE-3.
- summary: Notes each software run in the team's index, without names of files or tasks.
- depends: T-221, T-218
- files: src/cmd/used.rs, tests/cmd_used.rs, src/ctx.rs
- read: docs/design.md, docs/format.md
- steps:
  1. After the run is recorded (same command, a second tx::write on ctx.team_ctx() when the project has a team dossier; none: nothing): add a block '<code>:<USE-ID>' to usage-index.md (teamfiles::usage) with date (first 10 characters of the time), code, software, version, analysis and machine (from the usage entry or the project's tags), howto, person (the writer's person); history 'used'. No title, task or file names (R-USE-3).
  2. A team that cannot be reached: a warning, the run stays recorded.
  3. Tests in the test file with 'mod common;' and common::Project::new() (people anna with LLM anna-claude, ben with ben-codex; fixed time 2026-11-20T10:00:00+01:00); a terminal rule is tested with project.command(agent, now).env("DOSSIER_ANSWERS", ...). To get a team dossier in a test: in a second tempfile folder run 'dossier init --team-dossier --code TEAM' (as anna, with project.command and current_dir set to that folder), add anna as a member there, then in the project run 'dossier team use <path of that DOSSIER>'. Test: with a team, used writes one usage-index block; without a team nothing changes.
  4. Run cargo test --test cmd_used.
- accept: Every run also leaves one anonymous line in the team's usage index.
- verify: cargo test --test cmd_used
- verified: 2026-11-22
- commit: 79baef7
- reqs: R-USE-3, R-USE-12
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-22, planner): forced set (owner was haiku-t-238)
- note (2026-11-22, haiku-t-238): done
- note (2026-11-22, coordinator): commit: 291cb08, a99b3b0 (the done record names the work-plan commit; batch integration bookkeeping)

### T-239 · where-used: where and how a software was used
- status: done
- owner: haiku-t-239
- priority: P1 · difficulty: 1 · type: feature
- milestone: T-046
- why: R-USE-4.
- summary: Shows where and how a software was used in earlier projects.
- depends: T-238
- files: src/cmd/where_used.rs, tests/cmd_where_used.rs
- read: docs/design.md, docs/format.md
- steps:
  1. Read docs/design.md section 12 and the existing command files of the same kind (for example src/cmd/set.rs for a change, src/cmd/list.rs for a list, src/cmd/value.rs for a block file) and copy their patterns: Ctx::load, ctx.writer(), tx::write, Output::with with JSON data.
  2. Args: software (positional), --analysis, --machine. Read the team's usage-index.md (ctx.team_ctx(), or the current dossier when it is a team dossier); keep blocks whose software equals (ignoring case), and analysis/machine contain the given text (ignoring case). Newest date first: '<date>  <code>  <software> <version>  <analysis>  <machine>  <person>[  howto <ID>]'. End '<n> uses'.
  3. Tests in the test file with 'mod common;' and common::Project::new() (people anna with LLM anna-claude, ben with ben-codex; fixed time 2026-11-20T10:00:00+01:00); a terminal rule is tested with project.command(agent, now).env("DOSSIER_ANSWERS", ...). To get a team dossier in a test: in a second tempfile folder run 'dossier init --team-dossier --code TEAM' (as anna, with project.command and current_dir set to that folder), add anna as a member there, then in the project run 'dossier team use <path of that DOSSIER>'. Test: two runs in two projects (write usage-index blocks directly) are listed newest first; --analysis narrows.
  4. Run cargo test --test cmd_where_used.
- accept: where-used answers how and where a software was used before.
- verify: cargo test --test cmd_where_used
- verified: 2026-11-22
- commit: 565716a1087f89386c2c64d09fa7f1c06d325db0
- reqs: R-USE-4
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-22, haiku-t-239): done

### T-240 · list: purpose and tested versions of how-tos and tools
- status: done
- owner: haiku-t-240
- priority: P1 · difficulty: 1 · type: feature
- milestone: T-046
- why: R-USE-5, R-USE-10.
- summary: Shows what each how-to and tool is for in the list.
- depends: T-212
- files: src/cmd/list.rs, tests/cmd_list.rs
- read: docs/design.md, docs/format.md
- steps:
  1. For how-tos and tools add after the title '  — <first line of Purpose>' and, when the Tested with section is not empty, '  (tested with <its first line>)' (R-USE-5, R-USE-10). Nothing changes for other kinds.
  2. Tests: a how-to and a tool show their purpose.
  3. Run cargo test --test cmd_list.
- accept: list --kind howto and list --kind tool show what each is for.
- verify: cargo test --test cmd_list
- verified: 2026-11-22
- commit: 44ad99af0ae5ae5769d99fd2d0fd0da03751a872
- reqs: R-USE-5, R-USE-10
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-22, haiku-t-240): done

### T-047 · Tools: one kind for scripts, commands, installed software and git repositories (TOOL-001)
- status: done
- owner: coordinator
- priority: P1 · type: milestone
- milestone: T-042
- goal: R-USE-8 to R-USE-13 and R-ENTRY-12 hold.
- why: A script nobody can find or trust is rewritten.
- summary: Scripts, commands, installed software and repositories are recorded as tools, with a name and, when known, a location.
- accept: R-USE-8 to R-USE-13 and R-ENTRY-12 hold.
- reqs: R-USE-8 to R-USE-13, R-USE-1, R-ENTRY-12, D19, D31
- by: Reynan Henry Tandoyo
- note (2026-11-20, planner): added
- note (2026-11-21, planner): Owner 2026-11-21 (D31): one entry kind tool (prefix TOOL) with type script | command | software | repo and a location (a path on the server, a command name on PATH, the install place, a git URL). Checks: a script file still matches its checksum, a command still exists on the server; software and repos are not checked. Sections Purpose, How to run, Inputs, Outputs, Traps, Tested with. Replaces the software list, script entries and script_root; R-USE-1 and R-USE-8 to R-USE-13 are rewritten when phase 3 starts. The team standardises its tools in the meantime.
- note (2026-11-21, planner): Owner 2026-11-21 (D37): a tool has a required name and an optional location; nothing is checked without a location. (D35) software names stay free text.
- note (2026-11-21, planner): split into T-241, T-242, T-243, T-244, T-245, T-246
- note (2026-11-22, coordinator): done

### T-241 · tool add: record a tool
- status: done
- owner: haiku-t-241
- priority: P1 · difficulty: 1 · type: feature
- milestone: T-047
- why: D31, D37.
- summary: Records a script, command, software or repository the team uses.
- depends: T-234
- files: src/cmd/tool.rs, tests/cmd_tool.rs
- read: docs/design.md, docs/format.md
- steps:
  1. Read docs/design.md section 12 and the existing command files of the same kind (for example src/cmd/set.rs for a change, src/cmd/list.rs for a list, src/cmd/value.rs for a block file) and copy their patterns: Ctx::load, ctx.writer(), tx::write, Output::with with JSON data.
  2. Args: a subcommand Add { --title, --type <script|command|software|repo>, --location, --software, --language, --section (repeatable), --from, --stdin, --review, tag options } (Update comes later).
  3. Kind::Tool with fields type, location, software, language (format.md section 22). When the type is script and the location is a path inside the project or an absolute path that exists as a file: field checksum = filesum::checksum. A location is optional (D37).
  4. Tests in the test file with 'mod common;' and common::Project::new() (people anna with LLM anna-claude, ben with ben-codex; fixed time 2026-11-20T10:00:00+01:00); a terminal rule is tested with project.command(agent, now).env("DOSSIER_ANSWERS", ...). Test: a tool with only a name and Purpose; a script tool whose file exists gets a checksum; a bad type fails.
  5. Run cargo test --test cmd_tool.
- accept: Tools are recorded with a name and, when known, a location.
- verify: cargo test --test cmd_tool
- verified: 2026-11-22
- commit: 46cf2dbb4a63f6473c3da54b56fe6f780f32d4af
- reqs: D31, D37, R-USE-8
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-22, haiku-t-241): done

### T-242 · tool update: record a script's checksum again
- status: done
- owner: haiku-t-242
- priority: P1 · difficulty: 1 · type: feature
- milestone: T-047
- why: R-USE-11.
- summary: Records the new checksum of a script after an intended change.
- depends: T-241
- files: src/cmd/tool.rs, tests/cmd_tool.rs
- read: docs/design.md, docs/format.md
- steps:
  1. Add Update { id }: the entry must be a tool with a location that is a file; set checksum to filesum::checksum now; history 'changed' detail 'checksum'; the rule of approval::guarded applies (an LLM's change of an approved tool makes it a draft, as src/cmd/set.rs does).
  2. Tests in the test file with 'mod common;' and common::Project::new() (people anna with LLM anna-claude, ben with ben-codex; fixed time 2026-11-20T10:00:00+01:00); a terminal rule is tested with project.command(agent, now).env("DOSSIER_ANSWERS", ...). Test: change the script file, update, the checksum changes.
  3. Run cargo test --test cmd_tool.
- accept: A script's new version is recorded on purpose.
- verify: cargo test --test cmd_tool
- verified: 2026-11-22
- commit: f79d5637aa15d7c5703bc13c7c2fd72a466dddba
- reqs: R-USE-11
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-22, haiku-t-242): done

### T-243 · set and edit: the approval rule for how-tos and tools
- status: done
- owner: haiku-t-243
- priority: P1 · difficulty: 1 · type: feature
- milestone: T-047
- why: R-WRITE-8, R-LIFE-6.
- summary: Gives how-tos and tools the lesson rule in set and edit.
- depends: T-212
- files: src/cmd/set.rs, src/cmd/edit.rs, tests/cmd_set.rs
- read: docs/design.md, docs/format.md
- steps:
  1. Replace each check 'kind is Lesson' that decides the approval rule (back to draft for an LLM, approval::record or forget) with approval::guarded(kind) in set.rs and edit.rs.
  2. Test in tests/cmd_set.rs: an LLM's set on an approved how-to makes it a draft.
  3. Run cargo test --test cmd_set and cargo test --test cmd_edit.
- accept: set and edit treat how-tos and tools like lessons.
- verify: cargo test --test cmd_set
- verified: 2026-11-22
- commit: a4d52df
- reqs: R-WRITE-8
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-22, haiku-t-243): done

### T-244 · retitle and image: the approval rule for how-tos and tools
- status: done
- owner: haiku-t-244
- priority: P1 · difficulty: 1 · type: feature
- milestone: T-047
- why: R-WRITE-8, R-LIFE-6.
- summary: Gives how-tos and tools the lesson rule in retitle and image.
- depends: T-212
- files: src/cmd/retitle.rs, src/cmd/image.rs
- read: docs/design.md, docs/format.md
- steps:
  1. Replace each check 'kind is Lesson' that decides the approval rule with approval::guarded(kind) in retitle.rs and image.rs.
  2. Run cargo test --test cmd_retitle and cargo test --test cmd_image.
- accept: retitle and image treat how-tos and tools like lessons.
- verify: cargo test --test cmd_retitle
- verified: 2026-11-22
- commit: 16c420c
- reqs: R-WRITE-8
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-22, haiku-t-244): done

### T-245 · approve and reopen: the approval rule for how-tos and tools
- status: done
- owner: haiku-t-245
- priority: P1 · difficulty: 1 · type: feature
- milestone: T-047
- why: R-LIFE-6.
- summary: Records the approval checksum of how-tos and tools too.
- depends: T-212
- files: src/cmd/approve.rs, src/cmd/reopen.rs
- read: docs/design.md, docs/format.md
- steps:
  1. Replace each check 'kind is Lesson' that decides approval::record or forget with approval::guarded(kind) in approve.rs and reopen.rs.
  2. Run cargo test --test cmd_approve and cargo test --test cmd_reopen.
- accept: approve and reopen record checksums of how-tos and tools.
- verify: cargo test --test cmd_approve
- verified: 2026-11-22
- commit: e18dcc8
- reqs: R-LIFE-6
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-22, haiku-t-245): done

### T-246 · retire and restore: the approval rule for how-tos and tools
- status: done
- owner: haiku-t-246
- priority: P1 · difficulty: 1 · type: feature
- milestone: T-047
- why: R-LIFE-6.
- summary: Handles how-tos and tools like lessons when retired or restored.
- depends: T-212
- files: src/cmd/retire.rs, src/cmd/restore.rs
- read: docs/design.md, docs/format.md
- steps:
  1. Replace each check 'kind is Lesson' that decides approval::record or forget with approval::guarded(kind) in retire.rs and restore.rs.
  2. Run cargo test --test cmd_retire and cargo test --test cmd_restore.
- accept: retire and restore handle how-tos and tools like lessons.
- verify: cargo test --test cmd_retire
- verified: 2026-11-22
- commit: 8feb99b
- reqs: R-LIFE-6
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-22, haiku-t-246): done

### T-048 · Glossary: term add, term, glossary, and search through other spellings
- status: done
- owner: coordinator
- priority: P2 · type: milestone
- milestone: T-042
- goal: R-TERM-1 to R-TERM-4 hold; check warns about entries that look German as in R-ENTRY-20.
- why: Every LLM should read the team's abbreviations the same way, and a German word should find the English entries.
- summary: A team glossary of terms, symbols and abbreviations; a German word finds the English entries.
- accept: R-TERM-1 to R-TERM-4 hold; check warns about entries that look German as in R-ENTRY-20.
- reqs: R-TERM-1 to R-TERM-4, R-ENTRY-20
- by: Reynan Henry Tandoyo
- note (2026-11-20, planner): added
- note (2026-11-21, planner): split into T-247, T-248, T-249, T-250, T-251, T-252, T-253
- note (2026-11-22, coordinator): done

### T-247 · glossary.rs: terms and their other spellings
- status: done
- owner: haiku-t-247
- priority: P1 · difficulty: 1 · type: feature
- milestone: T-048
- why: R-TERM-1, R-TERM-4.
- summary: Reads the glossary and finds every spelling of a word.
- depends: T-212
- files: src/glossary.rs
- read: docs/design.md, docs/format.md
- steps:
  1. Implement load, terms and spellings as docs/design.md section 11 and format.md section 20 say (glossary.md, title 'Glossary of <code>'). also is a comma list, trimmed. spellings(terms, word): for each term whose term or any also equals word (ignoring case), all of its spellings (term and also), no duplicates.
  2. Tests: a glossary with 'mass flow' also 'Massenstrom, mdot'; spellings('Massenstrom') gives all three.
  3. Run cargo test --lib glossary::.
- accept: Glossary terms are read and a German word finds its English term.
- verify: cargo test --lib glossary::
- verified: 2026-11-21
- commit: 47daf61
- reqs: R-TERM-1, R-TERM-4
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-21, haiku-t-247): done

### T-248 · term: add a term or look one up
- status: done
- owner: haiku-t-248
- priority: P1 · difficulty: 1 · type: feature
- milestone: T-048
- why: R-TERM-1.
- summary: Adds glossary terms and looks them up.
- depends: T-247, T-226
- files: src/cmd/term.rs, tests/cmd_term.rs
- read: docs/design.md, docs/format.md
- steps:
  1. Read docs/design.md section 12 and the existing command files of the same kind (for example src/cmd/set.rs for a change, src/cmd/list.rs for a list, src/cmd/value.rs for a block file) and copy their patterns: Ctx::load, ctx.writer(), tx::write, Output::with with JSON data.
  2. Args: an optional subcommand Add { term, meaning, --symbol, --unit, --also <comma list> } and otherwise a positional word (use #[command(args_conflicts_with_subcommands = true)]).
  3. Add: the current dossier's glossary.md (the team's with --team; a team write from a project is checked with conf::gate and conf::check on the term, meaning and also first); a new block with meaning, symbol, unit, also and history 'added', or Exists. Lookup: terms of the project and the team whose term, also or meaning contains the word (ignoring case): '<term> (<symbol>, <unit>): <meaning>  [also: ...]', marked '[team]' for team terms.
  4. Tests in the test file with 'mod common;' and common::Project::new() (people anna with LLM anna-claude, ben with ben-codex; fixed time 2026-11-20T10:00:00+01:00); a terminal rule is tested with project.command(agent, now).env("DOSSIER_ANSWERS", ...). To get a team dossier in a test: in a second tempfile folder run 'dossier init --team-dossier --code TEAM' (as anna, with project.command and current_dir set to that folder), add anna as a member there, then in the project run 'dossier team use <path of that DOSSIER>'. Test: add in the project and with --team; lookup by an other spelling finds it.
  5. Run cargo test --test cmd_term.
- accept: Terms are added and looked up in the project and the team glossary.
- verify: cargo test --test cmd_term
- verified: 2026-11-22
- commit: cbe5de8530dabf48c739e1c11d9c9cc1fa5f93d1
- reqs: R-TERM-1
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-22, haiku-t-248): done

### T-249 · glossary: every term of the project and the team
- status: done
- owner: haiku-t-249
- priority: P1 · difficulty: 1 · type: feature
- milestone: T-048
- why: R-TERM-1.
- summary: Prints the whole glossary.
- depends: T-247, T-218
- files: src/cmd/glossary.rs, tests/cmd_glossary.rs
- read: docs/design.md, docs/format.md
- steps:
  1. Read docs/design.md section 12 and the existing command files of the same kind (for example src/cmd/set.rs for a change, src/cmd/list.rs for a list, src/cmd/value.rs for a block file) and copy their patterns: Ctx::load, ctx.writer(), tx::write, Output::with with JSON data.
  2. Print every term of the project's glossary.md and the team's, sorted by term (ignoring case), team terms marked '[team]', in the line format of the term command; end '<n> terms'.
  3. Tests in the test file with 'mod common;' and common::Project::new() (people anna with LLM anna-claude, ben with ben-codex; fixed time 2026-11-20T10:00:00+01:00); a terminal rule is tested with project.command(agent, now).env("DOSSIER_ANSWERS", ...). Test: two project terms are sorted.
  4. Run cargo test --test cmd_glossary.
- accept: glossary prints all terms.
- verify: cargo test --test cmd_glossary
- verified: 2026-11-22
- commit: e189ca0
- reqs: R-TERM-1
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-22, haiku-t-249): done
- note (2026-11-22, coordinator): commit: 07b1211 (the done record names the work-plan commit; batch integration bookkeeping)

### T-250 · search: the project and the team dossier together
- status: done
- owner: haiku-t-250
- priority: P1 · difficulty: 1 · type: feature
- milestone: T-048
- why: R-READ-3.
- summary: Searches the project and the team dossier at once.
- depends: T-218
- files: src/cmd/search.rs, tests/cmd_search.rs
- read: docs/design.md, docs/format.md
- steps:
  1. When the project has a team dossier (ctx.team), search its entries too; mark each team result with 'team:' before its ID (R-READ-3); the ordering rules stay, team results sorted in with the others. JSON gains place ('project' or 'team').
  2. Tests: a team lesson is found from the project with the team: mark.
  3. Run cargo test --test cmd_search.
- accept: search finds team knowledge from inside a project.
- verify: cargo test --test cmd_search
- verified: 2026-11-22
- commit: b5faadb
- reqs: R-READ-3
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-22, haiku-t-250): done
- note (2026-11-22, coordinator): commit: cf1d3b3 (the done record names the work-plan commit; batch integration bookkeeping)

### T-251 · search: find a term's other spellings
- status: done
- owner: haiku-t-251
- priority: P1 · difficulty: 1 · type: feature
- milestone: T-048
- why: R-TERM-4, D25.
- summary: Lets a German word find entries written in English.
- depends: T-250, T-247
- files: src/cmd/search.rs, tests/cmd_search.rs
- read: docs/design.md, docs/format.md
- steps:
  1. Each search word also matches when the text contains one of its spellings from the project's and the team's glossary (glossary::spellings): a word counts as present when any of its spellings is (R-TERM-4).
  2. Tests: with 'mass flow' also 'Massenstrom', searching 'Massenstrom' finds an entry that says 'mass flow'.
  3. Run cargo test --test cmd_search.
- accept: A German word finds the English entries.
- verify: cargo test --test cmd_search
- verified: 2026-11-22
- commit: 2f2ad8a547077afb78040c028676accb5ae095f3
- reqs: R-TERM-4
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-22, haiku-t-251): done

### T-252 · brief --task: team lessons and how-tos
- status: done
- owner: haiku-t-252
- priority: P1 · difficulty: 1 · type: feature
- milestone: T-048
- why: R-CTX-2.
- summary: Shows the team's lessons and how-tos that fit the task.
- depends: T-218
- files: src/brief_task.rs
- read: docs/design.md, docs/format.md
- steps:
  1. In 'Related', also score the team dossier's approved lessons and how-tos (ctx.team: store::all of it) with the same query; mark them 'team:' before the ID; lessons with Statement and Valid for, how-tos by title (R-CTX-2).
  2. Test in the module with a team folder built in the test.
  3. Run cargo test --lib brief_task::.
- accept: brief --task shows how the team did this kind of work before.
- verify: cargo test --lib brief_task::
- verified: 2026-11-22
- commit: 4d208f8
- reqs: R-CTX-2
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-22, haiku-t-252): done
- note (2026-11-22, coordinator): commit: f694d47 (the done record names the work-plan commit; batch integration bookkeeping)

### T-253 · brief --task: glossary terms and the last uses of the software
- status: done
- owner: haiku-t-253
- priority: P1 · difficulty: 1 · type: feature
- milestone: T-048
- why: R-TERM-2, R-CTX-2.
- summary: Adds the task's glossary terms and earlier uses of its software to the task context.
- depends: T-252, T-221, T-247
- files: src/brief_task.rs
- read: docs/design.md, docs/format.md
- steps:
  1. Add '## Terms': glossary terms (project and team) whose term or other spelling appears in the task's title, fields or notes, at most 10 lines (R-TERM-2).
  2. Add '## Earlier uses': the last three usage-index blocks (teamfiles::usage) with the task's software and analysis, newest first (R-CTX-2).
  3. Tests in the module.
  4. Run cargo test --lib brief_task::.
- accept: brief --task explains the terms of the task and shows the last uses of its software.
- verify: cargo test --lib brief_task::
- verified: 2026-11-22
- commit: 317d3dc15979d483ec37d42bdccebc8c7fb606e2
- reqs: R-TERM-2, R-CTX-2
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-22, haiku-t-253): done

### T-049 · What is new: news, and the news line in brief
- status: done
- owner: coordinator
- priority: P2 · type: milestone
- milestone: T-042
- goal: R-READ-10 and R-READ-11 hold.
- why: Colleagues need to see what was added to the team dossier without reading everything.
- summary: Each person sees what was added or approved since they last looked, in the project and the team dossier.
- accept: R-READ-10 and R-READ-11 hold.
- reqs: R-READ-10, R-READ-11
- by: Reynan Henry Tandoyo
- note (2026-11-20, planner): added
- note (2026-11-21, planner): split into T-254, T-255
- note (2026-11-22, coordinator): done

### T-254 · status --since last: the team dossier too, grouped by place
- status: done
- owner: haiku-t-254
- priority: P1 · difficulty: 1 · type: feature
- milestone: T-049
- why: R-READ-10.
- summary: Shows what changed in the project and the team since you last looked.
- depends: T-218
- files: src/cmd/status.rs, tests/cmd_status.rs
- read: docs/design.md, docs/format.md
- steps:
  1. With --since last or a date, add the team dossier's journal records (when the project has a team): print the project's lines under '## Project <code>' and the team's under '## Team', each line '<time>  <who>  <event>  <ID>  <title>' (R-READ-10), only events approved, superseded, retired, accepted and created-approved; at most 20 lines with a last line counting the rest.
  2. Tests: a team lesson approved after the marker appears under Team.
  3. Run cargo test --test cmd_status.
- accept: status --since last shows what is new in the project and in the team.
- verify: cargo test --test cmd_status
- verified: 2026-11-22
- commit: f6915ea6e2afde6333f37b5344712f6ba1122352
- reqs: R-READ-10
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-22, haiku-t-254): done

### T-255 · brief: the news line and the team's lessons and how-tos
- status: done
- owner: haiku-t-255
- priority: P1 · difficulty: 1 · type: feature
- milestone: T-049
- why: R-CTX-1, R-READ-11.
- summary: Shows the team's matching lessons and a line when there is news.
- depends: T-218
- files: src/cmd/brief.rs, tests/cmd_brief.rs
- read: docs/design.md, docs/format.md
- steps:
  1. Add '## Team lessons and how-tos': the team's approved lessons and how-tos whose machine, analysis or software tags match the project's tags (cfg.tags), as '<team:ID> · <title>', at most 10 (R-CTX-1).
  2. Last line: when entries were approved in the project or the team after the person's marker (.state/seen-<person>.txt, as status uses), '<n> new entries since <date>: dossier status --since last' (R-READ-11); nothing when there are none.
  3. Tests: a matching team lesson is listed; the news line appears after an approval and not before.
  4. Run cargo test --test cmd_brief.
- accept: brief shows the team's matching knowledge and says when there is news.
- verify: cargo test --test cmd_brief
- verified: 2026-11-22
- commit: dd494cc1522dd42add5b87723a8ce4c40e236924
- reqs: R-CTX-1, R-READ-11
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-22, haiku-t-255): done

### T-050 · Review dates and how-tos not tested with the newest version
- status: done
- owner: coordinator
- priority: P2 · type: milestone
- milestone: T-042
- goal: R-LIFE-7 and R-LIFE-8 hold.
- why: Lessons and how-tos go out of date quietly.
- summary: Lessons and how-tos can carry a date to be checked again; how-tos not tested with the newest version are marked.
- accept: R-LIFE-7 and R-LIFE-8 hold.
- reqs: R-LIFE-7, R-LIFE-8
- by: Reynan Henry Tandoyo
- note (2026-11-20, planner): added
- note (2026-11-21, planner): Owner 2026-11-21 (D33): no default review date; it is set per entry with --review <date>.
- note (2026-11-21, planner): split into T-256, T-257, T-258, T-259, T-260, T-261
- note (2026-11-22, coordinator): done

### T-256 · reviewed: a person confirms an entry still holds
- status: done
- owner: haiku-t-256
- priority: P1 · difficulty: 1 · type: feature
- milestone: T-050
- why: R-LIFE-7, D33.
- summary: Lets a person confirm that an entry still holds.
- depends: T-212
- files: src/cmd/reviewed.rs, tests/cmd_reviewed.rs
- read: docs/design.md, docs/format.md
- steps:
  1. Read docs/design.md section 12 and the existing command files of the same kind (for example src/cmd/set.rs for a change, src/cmd/list.rs for a list, src/cmd/value.rs for a block file) and copy their patterns: Ctx::load, ctx.writer(), tx::write, Output::with with JSON data.
  2. Args: id, --next <YYYY-MM-DD>. Rules: a person at a terminal. Set reviewed '<time> by <person>'; set review to --next when given, else remove it; history 'reviewed' (with detail 'next <date>' when given).
  3. Tests in the test file with 'mod common;' and common::Project::new() (people anna with LLM anna-claude, ben with ben-codex; fixed time 2026-11-20T10:00:00+01:00); a terminal rule is tested with project.command(agent, now).env("DOSSIER_ANSWERS", ...). Test: an entry with a past review date is reviewed and the field is gone; --next sets a new date.
  4. Run cargo test --test cmd_reviewed.
- accept: A person confirms an entry and sets the next review date.
- verify: cargo test --test cmd_reviewed
- verified: 2026-11-22
- commit: 5ffd37e
- reqs: R-LIFE-7
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-22, haiku-t-256): done
- note (2026-11-22, coordinator): commit: 876cda4 (the done record names the work-plan commit; batch integration bookkeeping)

### T-257 · learn: --review <date>
- status: done
- owner: haiku-t-257
- priority: P1 · difficulty: 1 · type: feature
- milestone: T-050
- why: R-LIFE-7, D33.
- summary: Lets a lesson carry the date it should be checked again.
- depends: T-212
- files: src/cmd/learn.rs, tests/cmd_learn.rs
- read: docs/design.md, docs/format.md
- steps:
  1. Add --review <YYYY-MM-DD> (a valid date, else BadValue); it becomes the field review of the lesson. No default (D33).
  2. Test: learn --review 2027-06-01 writes the field.
  3. Run cargo test --test cmd_learn.
- accept: A lesson can carry a review date.
- verify: cargo test --test cmd_learn
- verified: 2026-11-22
- commit: a58be95
- reqs: R-LIFE-7
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-22, haiku-t-257): done
- note (2026-11-22, coordinator): commit: 4bb9b3c (the done record names the work-plan commit; batch integration bookkeeping)

### T-258 · show: due for review, imported, file changed since approval
- status: done
- owner: haiku-t-258
- priority: P1 · difficulty: 1 · type: feature
- milestone: T-050
- why: R-LIFE-7, R-IMPORT-5, R-USE-11.
- summary: Shows when an entry is due for review, was imported, or its tool file changed.
- depends: T-212
- files: src/cmd/show.rs, tests/cmd_show.rs
- read: docs/design.md, docs/format.md
- steps:
  1. Marks: '[due for review]' when the review date is before today (time::date of time::now()); '[imported]' for imported: yes; '[file changed since approval]' for a tool whose location file's checksum differs from its checksum field; '[changed after approval]' now for every guarded kind (approval::guarded).
  2. Tests for each mark.
  3. Run cargo test --test cmd_show.
- accept: show marks entries due for review, imported ones and changed tool files.
- verify: cargo test --test cmd_show
- verified: 2026-11-22
- commit: afbedb06774f0d76307eb9b6b07bfe171eb58e49
- reqs: R-LIFE-7, R-IMPORT-5, R-USE-11
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-22, haiku-t-258): done

### T-259 · list and search: the same marks
- status: done
- owner: haiku-t-259
- priority: P1 · difficulty: 1 · type: feature
- milestone: T-050
- why: R-LIFE-7, R-IMPORT-5.
- summary: Shows the same marks in lists and search results.
- depends: T-258, T-240, T-251
- files: src/cmd/list.rs, src/cmd/search.rs, tests/cmd_list.rs, tests/cmd_search.rs
- read: docs/design.md, docs/format.md
- steps:
  1. Add the marks of the show command ('[due for review]', '[imported]', '[file changed since approval]', '[changed after approval]' for guarded kinds) to list and search lines, copying show.rs's logic.
  2. Run cargo test --test cmd_list and cargo test --test cmd_search.
- accept: list and search carry the new marks.
- verify: cargo test --test cmd_list
- verified: 2026-11-22
- commit: 94056a4ec3e69b543729ccd2dfa3bd5ca1545cf8
- reqs: R-LIFE-7, R-IMPORT-5
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-22, haiku-t-259): done

### T-260 · check: due for review and changed tool files
- status: done
- owner: haiku-t-260
- priority: P1 · difficulty: 1 · type: feature
- milestone: T-050
- why: R-LIFE-7, R-USE-11, R-CHECK-1.
- summary: Reports entries due for review and changed tool files.
- depends: T-212
- files: src/cmd/check.rs, tests/cmd_check.rs
- read: docs/design.md, docs/format.md
- steps:
  1. Report REVIEW for an entry whose review date has passed and TOOL for a tool whose location file changed since its checksum or is gone; CHANGED now for every guarded kind.
  2. Tests for each.
  3. Run cargo test --test cmd_check.
- accept: check reports entries due for review and changed tool files.
- verify: cargo test --test cmd_check
- verified: 2026-11-22
- commit: a531b4bcb51100a2a54b61647ca7c3c8f6a6cd5d
- reqs: R-LIFE-7, R-USE-11
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-22, haiku-t-260): done

### T-261 · brief and status: guarded kinds and due for review
- status: done
- owner: haiku-t-261
- priority: P1 · difficulty: 1 · type: feature
- milestone: T-050
- why: R-LIFE-6, R-LIFE-7, R-CTX-4.
- summary: Marks changed and overdue how-tos and tools in brief and status.
- depends: T-255, T-254
- files: src/cmd/brief.rs, src/cmd/status.rs, tests/cmd_brief.rs, tests/cmd_status.rs
- read: docs/design.md, docs/format.md
- steps:
  1. In brief, the '[changed after approval]' mark and in status the 'changed by hand' count use approval::guarded instead of lessons only; both add ' [due for review]' to shown entries whose review date has passed.
  2. Run cargo test --test cmd_brief and cargo test --test cmd_status.
- accept: brief and status treat how-tos and tools like lessons and show review marks.
- verify: cargo test --test cmd_brief
- verified: 2026-11-22
- commit: 25f7f2f0339230c587d620c963c292f1786c81c1
- reqs: R-LIFE-6, R-LIFE-7
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-22, haiku-t-261): done

### T-051 · Bringing in finished projects: init --past, review-folder, review-document, import status
- status: done
- owner: coordinator
- priority: P1 · type: milestone
- milestone: T-042
- goal: R-IMPORT-1 to R-IMPORT-9 hold; format.md and the tests cover every phase 3 addition; version 0.3.0 is released.
- why: The dossier must not start empty; years of finished projects hold most of what the team knows.
- summary: Knowledge from finished projects is brought in: the tool prints the material, the LLM drafts, a person approves.
- accept: R-IMPORT-1 to R-IMPORT-9 hold; format.md and the tests cover every phase 3 addition; version 0.3.0 is released.
- reqs: R-IMPORT-1 to R-IMPORT-9, D27
- by: Reynan Henry Tandoyo
- note (2026-11-20, planner): added
- note (2026-11-21, planner): split into T-262, T-263, T-264, T-265, T-266, T-267, T-268
- note (2026-11-22, coordinator): done

### T-262 · init --past and --in: a dossier for a finished project
- status: done
- owner: haiku-t-262
- priority: P1 · difficulty: 1 · type: feature
- milestone: T-051
- why: R-IMPORT-1.
- summary: Creates a dossier for a project that is already finished.
- depends: T-227
- files: src/cmd/init.rs, tests/cmd_init.rs
- read: docs/design.md, docs/format.md
- steps:
  1. Add --past (flag) and --in <folder>. --past writes 'past = true' after the code line. --in <folder> creates the DOSSIER in that folder instead of the current one and writes describes = "<the current folder, absolute, with '/'>" (R-IMPORT-1). --in without --past: BadValue.
  2. Tests: --past sets past; --in puts DOSSIER elsewhere with describes.
  3. Run cargo test --test cmd_init.
- accept: Finished projects get a dossier, also when their folder cannot be written to.
- verify: cargo test --test cmd_init
- verified: 2026-11-22
- commit: e127d166aaecbf8d9510d59d9b6a36063d6281b5
- reqs: R-IMPORT-1
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-22, haiku-t-262): done

### T-263 · imports.rs: which documents were reviewed
- status: done
- owner: haiku-t-263
- priority: P1 · difficulty: 1 · type: feature
- milestone: T-051
- why: R-IMPORT-3.
- summary: Reads which old documents were already reviewed.
- depends: T-212
- files: src/imports.rs
- read: docs/design.md, docs/format.md
- steps:
  1. Implement load and state as docs/design.md section 11 and format.md section 25 say (imports.md, title 'Documents of <code>').
  2. Tests: a missing file is empty; state of a reviewed document.
  3. Run cargo test --lib imports::.
- accept: The review state of old documents is read.
- verify: cargo test --lib imports::
- verified: 2026-11-22
- commit: 4527409
- reqs: R-IMPORT-3
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-22, haiku-t-263): done

### T-264 · folderscan.rs: the tree and the documents of an old folder
- status: done
- owner: haiku-t-264
- priority: P1 · difficulty: 1 · type: feature
- milestone: T-051
- why: R-IMPORT-2.
- summary: Lists an old project folder and its documents without opening them.
- depends: T-212
- files: src/folderscan.rs
- read: docs/design.md, docs/format.md
- steps:
  1. Implement tree and documents as docs/design.md section 11 says. Never read file contents; never follow symbolic links; skip folders named DOSSIER, .git and target. tree: names sorted, folders first, two spaces of indent per level, sizes in bytes. documents: kind from the extension list (report, presentation, spreadsheet, script), stored paths relative to root with '/', newest modified first, then path.
  2. Tests with a tempfile tree.
  3. Run cargo test --lib folderscan::.
- accept: An old project folder is listed without opening a file.
- verify: cargo test --lib folderscan::
- verified: 2026-11-22
- commit: b4f6333
- reqs: R-IMPORT-2
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-22, haiku-t-264): done

### T-265 · review-folder: the material for drafting from an old folder
- status: done
- owner: haiku-t-265
- priority: P1 · difficulty: 1 · type: feature
- milestone: T-051
- why: R-IMPORT-2, R-IMPORT-4, R-IMPORT-9.
- summary: Prints the material an LLM needs to draft entries from an old project folder.
- depends: T-264, T-263
- files: src/cmd/review_folder.rs, tests/cmd_review_folder.rs
- read: docs/design.md, docs/format.md
- steps:
  1. Read docs/design.md section 12 and the existing command files of the same kind (for example src/cmd/set.rs for a change, src/cmd/list.rs for a list, src/cmd/value.rs for a block file) and copy their patterns: Ctx::load, ctx.writer(), tx::write, Output::with with JSON data.
  2. Args: path (default: describes of the settings, else the project folder), --depth (default 3). Print '# Folder <path>', '## Tree' (folderscan::tree), '## Documents' (each '<stored>  <kind>  <size>  <modified date>  [<state from imports.md>]'), '## Already reviewed' (count), '## Import rules' (R-IMPORT-4 as a numbered list, one rule per line), and the example commands 'dossier review-document <file>' and 'dossier learn ... --from-document <file>'. Changes nothing.
  3. Tests in the test file with 'mod common;' and common::Project::new() (people anna with LLM anna-claude, ben with ben-codex; fixed time 2026-11-20T10:00:00+01:00); a terminal rule is tested with project.command(agent, now).env("DOSSIER_ANSWERS", ...). Test: a folder with a report and a script lists both.
  4. Run cargo test --test cmd_review_folder.
- accept: review-folder prints what an LLM needs to draft from an old project.
- verify: cargo test --test cmd_review_folder
- verified: 2026-11-22
- commit: fb7bef9
- reqs: R-IMPORT-2, R-IMPORT-4
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-22, haiku-t-265): done

### T-266 · review-document and reviewed-document
- status: done
- owner: haiku-t-266
- priority: P1 · difficulty: 1 · type: feature
- milestone: T-051
- why: R-IMPORT-3.
- summary: Marks an old document as under review and then as done.
- depends: T-263
- files: src/cmd/review_document.rs, src/cmd/reviewed_document.rs, tests/cmd_review_document.rs
- read: docs/design.md, docs/format.md
- steps:
  1. Read docs/design.md section 12 and the existing command files of the same kind (for example src/cmd/set.rs for a change, src/cmd/list.rs for a list, src/cmd/value.rs for a block file) and copy their patterns: Ctx::load, ctx.writer(), tx::write, Output::with with JSON data.
  2. review-document <file>: store the path; in imports.md add the block (or set state) state reviewing, history 'reviewing'; print the import rules (as review-folder) and 'draft with: --from-document <file>'.
  3. reviewed-document <file> [--nothing]: the block must exist; state done, or nothing with --nothing; history 'reviewed' or 'nothing'.
  4. Tests in the test file with 'mod common;' and common::Project::new() (people anna with LLM anna-claude, ben with ben-codex; fixed time 2026-11-20T10:00:00+01:00); a terminal rule is tested with project.command(agent, now).env("DOSSIER_ANSWERS", ...). Test: review then reviewed sets done; --nothing sets nothing.
  5. Run cargo test --test cmd_review_document.
- accept: Documents are marked as being reviewed and as done.
- verify: cargo test --test cmd_review_document
- verified: 2026-11-22
- commit: 627e5e1
- reqs: R-IMPORT-3
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-22, haiku-t-266): done
- note (2026-11-22, coordinator): commit: 7827eb6 (the done record names the work-plan commit; batch integration bookkeeping)

### T-267 · import status: progress of bringing in a finished project
- status: done
- owner: haiku-t-267
- priority: P1 · difficulty: 1 · type: feature
- milestone: T-051
- why: R-IMPORT-7.
- summary: Shows how far a finished project has been brought in.
- depends: T-264, T-263, T-234
- files: src/cmd/import.rs, tests/cmd_import.rs
- read: docs/design.md, docs/format.md
- steps:
  1. Read docs/design.md section 12 and the existing command files of the same kind (for example src/cmd/set.rs for a change, src/cmd/list.rs for a list, src/cmd/value.rs for a block file) and copy their patterns: Ctx::load, ctx.writer(), tx::write, Output::with with JSON data.
  2. Args: a subcommand Status. Print for the current dossier: documents found (folderscan::documents of describes or the project folder), reviewing, done, nothing (imports.md), imported drafts waiting, imported entries approved, imported entries promoted (promoted field). JSON with the same numbers.
  3. Tests in the test file with 'mod common;' and common::Project::new() (people anna with LLM anna-claude, ben with ben-codex; fixed time 2026-11-20T10:00:00+01:00); a terminal rule is tested with project.command(agent, now).env("DOSSIER_ANSWERS", ...). Test: one document done and one imported draft are counted.
  4. Run cargo test --test cmd_import.
- accept: import status shows how far an old project has been brought in.
- verify: cargo test --test cmd_import
- verified: 2026-11-22
- commit: f93e8e9e877121a178141f4070148f67f06fd7f5
- reqs: R-IMPORT-7
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-22, haiku-t-267): done

### T-268 · drafts --imported: imported drafts by document
- status: done
- owner: haiku-t-268
- priority: P1 · difficulty: 1 · type: feature
- milestone: T-051
- why: R-IMPORT-6.
- summary: Lists drafts from old documents apart, grouped by document.
- depends: T-234
- files: src/cmd/drafts.rs, tests/cmd_drafts.rs
- read: docs/design.md, docs/format.md
- steps:
  1. Add --imported: only drafts with imported: yes, grouped under '## <source-document>' headings, oldest first inside each; without --imported, imported drafts are marked '[imported]' (R-IMPORT-6).
  2. Tests: two imported drafts from two documents are grouped.
  3. Run cargo test --test cmd_drafts.
- accept: Imported drafts are listed apart, by document.
- verify: cargo test --test cmd_drafts
- verified: 2026-11-22
- commit: 1cda0f7c0e70bfadebc25eb5a9642cdc99e100fc
- reqs: R-IMPORT-6
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-22, haiku-t-268): done

### T-052 · Working from the PC: the dossier is used from Windows as if it were local
- status: done
- owner: planner
- priority: P2 · type: milestone
- goal: A folder on a PC is connected to a project on the server; every dossier and todo command runs there through ssh; large simulation files are taken, worked on locally and returned without overwriting a colleague's version. Requirements: docs/requirements.md section 11, Phase 5 (v0.5; phase 4 before D45). Each task here is split when the phase starts.
- summary: Team members use the dossier from their Windows PCs as if it were local, and large files are copied out and back safely.
- by: Reynan Henry Tandoyo
- note (2026-11-20, planner): added
- note (2026-11-23, planner): Owner, 2026-11-23 (D45): moved from phase 4 to phase 5; it is released as 0.5.0 (T-328), after 0.4.0.
- note (2026-11-23, planner): done — From a Windows PC every command runs on the server with versions and subfolder checked, and large files are taken and returned safely; version 0.5.0.

### T-053 · Remote mode: connection reuse and connection check
- status: done
- owner: planner
- priority: P2 · type: milestone
- milestone: T-052
- goal: R-REMOTE-8 holds, and the two phase-1 gaps are closed: PC and server compare versions (R-XOS-9), and a path typed below the connected folder means the same path on the server (R-XOS-3). R-REMOTE-7 (keeping a connection open) is not built, by the owner's decision D43.
- why: Team members work on Windows PCs while the one copy of the dossier is on the Linux server.
- summary: Commands from a PC reuse the ssh connection where possible, and connect --check tests that the server is ready.
- depends: T-051
- accept: R-REMOTE-1 to R-REMOTE-3 and R-REMOTE-5 to R-REMOTE-8 hold, with the tests of TEST-8.
- reqs: R-REMOTE-1 to R-REMOTE-3, R-REMOTE-5 to R-REMOTE-8, D9, TEST-8
- by: Reynan Henry Tandoyo
- note (2026-11-20, planner): added
- note (2026-11-20, planner): Forwarding, connect and the project register moved to phase 1 (T-060). Left here: R-REMOTE-7 and R-REMOTE-8.
- note (2026-11-23, planner): split into T-311, T-312, T-313, T-314, T-315, T-316
- note (2026-11-23, planner): done — From a PC, every command checks the versions and runs in the matching subfolder, and connect --check tests the connection, dossier and todo on the server.

### T-311 · ssh.rs: variables and a subfolder for the remote command, and captured output
- status: done
- owner: haiku-T-311
- priority: P0 · difficulty: 1 · type: feature
- milestone: T-053
- why: Team members work on Windows PCs while the one copy of the dossier is on the Linux server.
- summary: The building blocks for a more exact connection from the PC to the server.
- files: src/remote/ssh.rs
- read: docs/design.md, docs/format.md, docs/requirements.md
- steps:
  1. Add argv_with and capture exactly as in docs/design.md section 16. `argv` keeps its signature and result: it calls argv_with with sub None and env [("DOSSIER_AGENT", agent)] when an agent is given, else an empty list.
  2. argv_with: the last element is one string. Without sub: `cd <quoted folder> && ` as today. With sub: `{ cd <quoted folder/sub> 2>/dev/null || cd <quoted folder>; } && `. Then each variable as `NAME=<quoted value> `, then the quoted program and arguments.
  3. capture runs the same program as run (DOSSIER_SSH or ssh) with the same arguments, standard input null, and returns (exit code, standard output, standard error); a program that cannot be started is Code::Ssh as in run. Nothing is added to the arguments: each command makes its own connection (D43).
  4. Unit tests in ssh.rs: argv gives exactly what it gave before (keep the existing tests green); argv_with with a sub and two variables gives the string of step 2; a folder and a sub with spaces and a quote are quoted.
  5. Run cargo test --lib remote::ssh:: and cargo test --test cmd_projects --test cmd_todo
- accept: The ssh command can carry variables and a subfolder, and its output can be captured.
- verify: cargo test --lib remote::ssh:: && cargo test --test cmd_projects
- verified: 2026-11-23
- commit: 04c0cb4
- reqs: R-XOS-3, R-XOS-9, D43
- by: Reynan Henry Tandoyo
- note (2026-11-23, planner): added
- note (2026-11-23, haiku-T-311): done

### T-312 · remote/version.rs: compare the versions of PC and server
- status: done
- owner: haiku-T-312
- priority: P0 · difficulty: 1 · type: feature
- milestone: T-053
- why: Team members work on Windows PCs while the one copy of the dossier is on the Linux server.
- summary: A PC and a server with different versions are noticed before a command half-works.
- files: src/remote/version.rs, src/remote/mod.rs
- read: docs/design.md, docs/format.md, docs/requirements.md
- steps:
  1. Create src/remote/version.rs with FORMAT, CLIENT_VERSION, CLIENT_FORMAT and check exactly as in docs/design.md section 16, and add `pub mod version;` to src/remote/mod.rs.
  2. check: both None: Ok(None). client_format given and not equal to FORMAT as text: Err(Code::TooNew) with `the PC uses dossier format <a>, the server format <b>: update dossier on the <PC or server>` (the side with the lower number; a value that is no number: the PC). Formats equal and client_version given and different from env!("CARGO_PKG_VERSION"): Ok(Some(`dossier on the PC is <a>, on the server <b>`)). Otherwise Ok(None).
  3. Unit tests: each of the four outcomes, and a format that is not a number.
  4. Run cargo test --lib remote::version::
- accept: The server can tell whether the PC that sent a command reads the same format and has the same program version.
- verify: cargo test --lib remote::version::
- verified: 2026-11-23
- commit: 5c7268d
- reqs: R-XOS-9, NFR-7
- by: Reynan Henry Tandoyo
- note (2026-11-23, planner): added
- note (2026-11-23, haiku-T-312): done

### T-313 · forward.rs: send the versions and the subfolder with every command
- status: done
- owner: haiku-T-313
- priority: P1 · difficulty: 1 · type: feature
- milestone: T-053
- why: Team members work on Windows PCs while the one copy of the dossier is on the Linux server.
- summary: A path typed in a subfolder on the PC means the same path on the server, and the server learns the PC's version.
- depends: T-311, T-312
- files: src/remote/forward.rs, tests/remote_forward.rs
- read: docs/design.md, docs/format.md, docs/requirements.md
- steps:
  1. In `maybe`, build the ssh arguments with ssh::argv_with instead of ssh::argv. env: DOSSIER_AGENT when a name is known (as today), and, when the program is dossier (not todo), DOSSIER_CLIENT_VERSION = env!("CARGO_PKG_VERSION") and DOSSIER_CLIENT_FORMAT = remote::version::FORMAT as text (use the constants of remote::version for the names).
  2. sub: when the target comes from a pointer file, the current folder relative to the folder that holds the pointer, written with `/`; None when they are the same folder or the target comes from --project. pointer::find already returns the folder it found the pointer in.
  3. tests/remote_forward.rs (new): write an ssh stand-in that appends its arguments to a file, as tests/cmd_projects.rs does; connect the project folder with `dossier connect --code P-001 anna@server:/projects/p1` and run `dossier list` in it and in a subfolder `sim/run 1`. The recorded command contains DOSSIER_CLIENT_VERSION= and DOSSIER_CLIENT_FORMAT='1'; from the subfolder it contains `cd '/projects/p1/sim/run 1' 2>/dev/null || cd '/projects/p1'`; `dossier todo list` carries no DOSSIER_CLIENT variables.
  4. Run cargo test --test remote_forward and cargo test --lib remote::forward::
- accept: Every command sent from a PC tells the server its version and runs in the same folder below the project as on the PC.
- verify: cargo test --test remote_forward
- verified: 2026-11-23
- commit: f58af55
- reqs: R-XOS-3, R-XOS-9, R-REMOTE-2
- by: Reynan Henry Tandoyo
- note (2026-11-23, planner): added
- note (2026-11-23, haiku-T-313): done

### T-314 · cli.rs: refuse a PC with another format, warn about another version
- status: done
- owner: haiku-T-314
- priority: P1 · difficulty: 1 · type: feature
- milestone: T-053
- why: Team members work on Windows PCs while the one copy of the dossier is on the Linux server.
- summary: Mixing an old PC program with a new server program is caught at once.
- depends: T-312
- files: src/cli.rs, tests/remote_version.rs
- read: docs/design.md, docs/format.md, docs/requirements.md
- steps:
  1. In main_exit, after the arguments are parsed and before the command runs: remote::version::check(the value of DOSSIER_CLIENT_VERSION, the value of DOSSIER_CLIENT_FORMAT) (empty counts as not set). Err: the result of the command is that error, the command does not run. Ok(Some(warning)): after the command ran, add the warning to its Output with Output::warn (an error result stays as it is).
  2. tests/remote_version.rs (new, uses tests/common): `dossier where` with DOSSIER_CLIENT_FORMAT=2 fails with exit 4 and names the format; with --json the error code is TOO_NEW; with DOSSIER_CLIENT_FORMAT=1 and DOSSIER_CLIENT_VERSION=0.0.1 it succeeds and standard error has a `warning:` line naming both versions; without the variables there is no warning.
  3. Run cargo test --test remote_version
- accept: A command from a PC with another format version is refused with a message that says which side to update; another program version is a warning.
- verify: cargo test --test remote_version
- verified: 2026-11-23
- commit: 9a8bad1
- reqs: R-XOS-9
- by: Reynan Henry Tandoyo
- note (2026-11-23, planner): added
- note (2026-11-23, haiku-T-314): done

### T-315 · connect --check: the connection, dossier and todo on the server, and the folder
- status: done
- owner: haiku-T-315
- priority: P1 · difficulty: 1 · type: feature
- milestone: T-053
- why: Team members work on Windows PCs while the one copy of the dossier is on the Linux server.
- summary: One command tells a team member why the connection to the server does not work.
- depends: T-311
- files: src/cmd/connect.rs, tests/cmd_connect.rs
- read: docs/design.md, docs/format.md, docs/requirements.md
- steps:
  1. Rewrite `check` in src/cmd/connect.rs as docs/design.md section 16 says. Three runs with ssh::capture and ssh::argv (program and arguments): `dossier --version`, `todo --version`, `dossier where --json`, each with the folder of the pointer.
  2. First run: exit 255, or ssh could not start: Code::Ssh with `could not reach <server>: <the last non-empty line of ssh's standard error>; sign in once by hand with ssh <server>`. Another exit code that is not 0: Code::Ssh `dossier is not installed on <server>, or not on its PATH`. Exit 0: line `ssh: ok` and line `dossier on the server: <version>` (the last word of the output); when it differs from env!("CARGO_PKG_VERSION") add a warning `this PC has dossier <a>, the server <b>`.
  3. Second run: not exit 0: Code::TodoMissing `todo is not installed on <server>, or not on its PATH`. Else line `todo on the server: <version>`.
  4. Third run: not exit 0 or no code in the answer: Code::NoDossier `<folder> on <server> has no dossier; run dossier init there`. Else line `dossier folder: ok (<code>)`, with the existing warning when the code differs from the pointer's.
  5. JSON data: {server, folder, code, dossier, todo}.
  6. tests/cmd_connect.rs: with an ssh stand-in that answers the three commands (it can look for `--version` and `where` in its last argument), --check prints the four lines; a stand-in that exits 255 and writes `Permission denied` to standard error gives exit 5 and that text; a stand-in that fails only for todo gives exit 5 and names todo; one that fails for `where` gives exit 4. Keep the existing tests green.
  7. Run cargo test --test cmd_connect
- accept: connect --check says what works and what does not: the connection, dossier and todo on the server with their versions, and the project folder.
- verify: cargo test --test cmd_connect
- verified: 2026-11-23
- commit: 973f929
- reqs: R-REMOTE-8
- by: Reynan Henry Tandoyo
- note (2026-11-23, planner): added
- note (2026-11-23, haiku-T-315): done

### T-316 · README: connect --check, versions and subfolders
- status: done
- owner: haiku-T-316
- priority: P1 · difficulty: 1 · type: docs
- milestone: T-053
- why: Team members work on Windows PCs while the one copy of the dossier is on the Linux server.
- summary: Team members can read how the connection from the PC behaves and how to check it.
- depends: T-313, T-314, T-315
- files: README.md
- read: docs/design.md, docs/format.md, docs/requirements.md
- steps:
  1. In `## Remote mode: from a Windows PC`: describe `dossier connect --check` with its four lines and the three failures; say that a command typed in a folder below the connected folder runs in the same folder below the project on the server; say that PC and server compare versions with every command (another format is refused, another version is a warning).
  2. Add a short paragraph: each command makes its own ssh connection, nothing runs in the background on the PC, and a command typically takes a second or so; the project folder can be read at any time through the network share with a file browser or an editor, and only writing goes through the commands.
  3. Run cargo test --test smoke (the README is not compiled; this only confirms nothing else changed).
- accept: The README explains the connection check, the version comparison and subfolders, and says plainly how a command reaches the server.
- verify: cargo test --test smoke
- verified: 2026-11-23
- commit: b730828
- reqs: R-REMOTE-8, REL-2, D43
- by: Reynan Henry Tandoyo
- note (2026-11-23, planner): added
- note (2026-11-23, haiku-T-316): done

### T-054 · Large files: take, return, and the copy in remote mode; release 0.5.0
- status: done
- owner: planner
- priority: P2 · type: milestone
- milestone: T-052
- goal: R-FILE-1 to R-FILE-7 and R-REMOTE-4 hold: a large file is taken, worked on locally and returned without overwriting a colleague's version, also from a PC; version 0.5.0 is ready to tag.
- why: Two people who copy the same simulation file out and back overwrite each other's work.
- summary: A large simulation file is taken to a PC and returned without overwriting a colleague's newer version.
- depends: T-053
- accept: R-FILE-1 to R-FILE-7 and R-REMOTE-4 hold, with the tests of TEST-7; the manual test plan of TEST-10 was run on the real server; version 0.5.0 is released.
- reqs: R-FILE-1 to R-FILE-7, R-REMOTE-4, D15, TEST-7, TEST-10
- by: Reynan Henry Tandoyo
- note (2026-11-20, planner): added
- note (2026-11-23, planner): split into T-318, T-319, T-320, T-321, T-322, T-323, T-324, T-325, T-326, T-327, T-328
- note (2026-11-23, planner): For TEST-10: a file returned from a PC gets the umask of the server's sftp-server (often 0644). If colleagues must replace each other's files, run sftp-server with a group umask (Subsystem sftp ... -u 002) or use folder ACLs. Checked end to end on 2026-11-23 against OpenSSH 10.2 in WSL: take, retake (kept), return --done from a subfolder, paths with spaces.
- note (2026-11-23, planner): done — take and return work on the server and from a PC; released as 0.5.0 without the manual server test, which is the idea for TEST-10 (D47).

### T-318 · v0.5 skeleton: stubs for take, return, taken.rs and remote/transfer.rs
- status: done
- owner: planner
- priority: P0 · type: milestone
- milestone: T-054
- goal: The commands take and return and the modules taken.rs and remote/transfer.rs exist as stubs that compile.
- why: The simulation files are gigabytes; the risk is overwriting a colleague's work when two people copy the same file to their PCs and back.
- summary: Empty places for the new commands and modules, so that each can be built on its own.
- accept: The two commands and the two modules exist as stubs that compile, and the overview lists take and return.
- reqs: R-FILE-1, R-FILE-2, R-FILE-3
- by: Reynan Henry Tandoyo
- note (2026-11-23, planner): added
- note (2026-11-23, planner): split into T-366, T-367, T-368, T-369, T-370
- note (2026-11-23, planner): done — take and return exist as commands that answer 'not built yet', with taken.rs and remote/transfer.rs as stubs.

### T-366 · Stub files: take and return
- status: done
- owner: haiku-T-366
- priority: P0 · difficulty: 1 · type: chore
- milestone: T-318
- why: The simulation files are gigabytes; the risk is overwriting a colleague's work when two people copy the same file to their PCs and back.
- summary: Empty files for the two new commands.
- files: src/cmd/take.rs, src/cmd/return_cmd.rs, src/cmd/mod.rs
- read: docs/design.md, docs/format.md, docs/requirements.md
- steps:
  1. Create both files, each with `#[derive(clap::Args, Debug)] pub struct Args {}` and `pub fn run(_args: &Args, _g: &Global) -> Result<Output>` returning Err(Error::new(Code::NotImplemented, "<command> is not built yet")), with the same `use` lines as src/cmd/guard.rs.
  2. Check them with rustfmt --check --edition 2024 src/cmd/take.rs src/cmd/return_cmd.rs
  3. src/cmd/mod.rs: `pub mod return_cmd;` and `pub mod take;` at their alphabetical places. These modules exist from this task on, so they are compiled and checked; the register task adds them to cli.rs and help.rs.
  4. Run cargo build (it must still pass).
- accept: The two stub files exist and are formatted.
- verify: rustfmt --check --edition 2024 src/cmd/take.rs src/cmd/return_cmd.rs && cargo build
- verified: 2026-11-23
- commit: cf02380
- reqs: R-FILE-1, R-FILE-2
- by: Reynan Henry Tandoyo
- note (2026-11-23, planner): added
- note (2026-11-23, haiku-T-366): done

### T-367 · Stub module: taken.rs
- status: done
- owner: haiku-T-367
- priority: P0 · difficulty: 1 · type: chore
- milestone: T-318
- why: The simulation files are gigabytes; the risk is overwriting a colleague's work when two people copy the same file to their PCs and back.
- summary: An empty place for the list of taken files.
- files: src/taken.rs, src/lib.rs
- read: docs/design.md, docs/format.md, docs/requirements.md
- steps:
  1. Create src/taken.rs with the items of docs/design.md section 17 for taken.rs (FILE, ITEM, load, Take, current, all, log). Bodies: load returns Err(Error::new(Code::NotImplemented, "not built yet")); current returns None; all and log return empty lists.
  2. src/lib.rs: `pub mod taken;` at its alphabetical place.
  3. Run cargo build
- accept: The module exists with its final signatures and compiles.
- verify: cargo build
- verified: 2026-11-23
- commit: 7e09a18
- reqs: R-FILE-1
- by: Reynan Henry Tandoyo
- note (2026-11-23, planner): added
- note (2026-11-23, haiku-T-367): done

### T-368 · Stub module: remote/transfer.rs
- status: done
- owner: haiku-T-368
- priority: P0 · difficulty: 1 · type: chore
- milestone: T-318
- why: The simulation files are gigabytes; the risk is overwriting a colleague's work when two people copy the same file to their PCs and back.
- summary: An empty place for copying files between PC and server.
- files: src/remote/transfer.rs, src/remote/mod.rs
- read: docs/design.md, docs/format.md, docs/requirements.md
- steps:
  1. Create src/remote/transfer.rs with the items of docs/design.md section 17 for remote/transfer.rs (program, argv, remote_path, maybe). Bodies: program returns "scp"; argv returns an empty list; remote_path returns an empty text; maybe returns None.
  2. src/remote/mod.rs: `pub mod transfer;`.
  3. Run cargo build
- accept: The module exists with its final signatures and compiles.
- verify: cargo build
- verified: 2026-11-23
- commit: 450b155
- reqs: R-REMOTE-4
- by: Reynan Henry Tandoyo
- note (2026-11-23, planner): added
- note (2026-11-23, haiku-T-368): done

### T-369 · Register take and return
- status: done
- owner: haiku-T-369
- priority: P0 · difficulty: 1 · type: chore
- milestone: T-318
- why: The simulation files are gigabytes; the risk is overwriting a colleague's work when two people copy the same file to their PCs and back.
- summary: The two new commands become part of the program.
- depends: T-366
- files: src/cli.rs, src/cmd/help.rs, tests/cmd_agents.rs
- read: docs/design.md, docs/format.md, docs/requirements.md
- steps:
  1. src/cli.rs: variants `Take(cmd::take::Args)` and `Return(cmd::return_cmd::Args)` in the Command enum and two arms in the match of main_exit with the names "take" and "return".
  2. src/cmd/help.rs: ("take", "Records who took a file") and ("return", "Checks or ends a take") in COMMANDS, and the array length raised by two.
  3. tests/cmd_agents.rs: help_without_a_topic_lists_every_command_in_order compares the whole list that `dossier help` prints; when the new commands now appear in it, add them to the expected list at the place where `dossier help` prints them, and change nothing else in that file.
  4. Run cargo build and cargo test --test smoke; `dossier take` must answer `take is not built yet` with exit 2.
- accept: take and return are part of the program and the overview.
- verify: cargo build && cargo test --test smoke
- verified: 2026-11-23
- commit: cd04af1
- reqs: R-FILE-1, R-FILE-2
- by: Reynan Henry Tandoyo
- note (2026-11-23, planner): added
- note (2026-11-23, planner): Added tests/cmd_agents.rs: its help overview test lists every command (found in T-359).
- note (2026-11-23, planner): src/cmd/mod.rs moved to T-366, so this task stays at 3 files (level 1).
- note (2026-11-23, haiku-T-369): done

### T-370 · ctx.rs: take and return are writing commands
- status: done
- owner: haiku-T-370
- priority: P0 · difficulty: 1 · type: chore
- milestone: T-318
- why: The simulation files are gigabytes; the risk is overwriting a colleague's work when two people copy the same file to their PCs and back.
- summary: The new commands follow the same name rule as every command that writes.
- files: src/ctx.rs
- read: docs/design.md, docs/format.md, docs/requirements.md
- steps:
  1. Add "take" and "return" to the WRITING list and raise its length by two.
  2. Run cargo test --lib ctx::
- accept: take and return refuse a name that is not in the settings.
- verify: cargo test --lib ctx::
- verified: 2026-11-23
- commit: 01e6ab2
- reqs: R-CFG-4
- by: Reynan Henry Tandoyo
- note (2026-11-23, planner): added
- note (2026-11-23, haiku-T-370): done

### T-319 · config.rs: take_days
- status: done
- owner: haiku-T-319
- priority: P1 · difficulty: 1 · type: feature
- milestone: T-054
- why: The simulation files are gigabytes; the risk is overwriting a colleague's work when two people copy the same file to their PCs and back.
- summary: A setting for when a file that was taken and not returned is marked as old.
- depends: T-318
- files: src/config.rs, src/filelist.rs, src/names.rs
- read: docs/design.md, docs/format.md, docs/requirements.md
- steps:
  1. Config gains `pub take_days: i64`; LimitsFile gains `take_days` with the default 14; `parse` copies it like draft_days. A value below 1 is Code::Config naming the file and the key, as the other limits do.
  2. Unit tests: missing gives 14; `take_days = 30` under [limits] gives 30; an unknown key under [limits] is still an error.
  3. Run cargo test --lib config::
- accept: The settings know after how many days a take counts as old.
- verify: cargo test --lib config::
- verified: 2026-11-23
- commit: ac5981f
- reqs: R-FILE-7, R-CFG-2
- by: Reynan Henry Tandoyo
- note (2026-11-23, planner): added
- note (2026-11-23, planner): forced set (owner was haiku-T-319)
- note (2026-11-23, planner): src/filelist.rs and src/names.rs build a Config in their tests and need take_days: 14 there; added to the files. The below-1 check is new for take_days and stays.
- note (2026-11-23, haiku-T-319): done

### T-320 · taken.rs: the file of taken files
- status: done
- owner: haiku-T-320
- priority: P0 · difficulty: 1 · type: feature
- milestone: T-054
- why: The simulation files are gigabytes; the risk is overwriting a colleague's work when two people copy the same file to their PCs and back.
- summary: The part that reads the list of files that are taken.
- depends: T-318
- files: src/taken.rs
- read: docs/design.md, docs/format.md, docs/requirements.md
- steps:
  1. Fill src/taken.rs as docs/design.md section 17 and docs/format.md section 30 say. load: like values::load in src/values.rs, with the file DOSSIER/taken.md and the title `Taken files of <code>`.
  2. current(file, path): Some(Take) when the block of that stored path has a `taken` field: person and time from `<time> by <person>`, task, size (parsed, 0 when missing), modified, checksum, forced (`forced: yes`). all: current for every key, in file order. log: the block's history lines (BlockFile::history).
  3. Unit tests with a BlockFile built by add_block and set_field: a block with `taken` is current, one without is not; all lists only the taken ones; log returns the history lines in order.
  4. Run cargo test --lib taken::
- accept: The program reads who holds which file and the log of each file's versions from one readable file.
- verify: cargo test --lib taken::
- verified: 2026-11-23
- commit: 6915748
- reqs: R-FILE-1, R-FILE-5
- by: Reynan Henry Tandoyo
- note (2026-11-23, planner): added
- note (2026-11-23, haiku-T-320): done

### T-321 · take: record who took a file
- status: done
- owner: haiku-T-321
- priority: P1 · difficulty: 1 · type: feature
- milestone: T-054
- why: The simulation files are gigabytes; the risk is overwriting a colleague's work when two people copy the same file to their PCs and back.
- summary: The command that says: I am working on this file now.
- depends: T-320
- files: src/cmd/take.rs, tests/cmd_take.rs
- read: docs/design.md, docs/format.md, docs/requirements.md
- steps:
  1. Args: `pub files: Vec<String>` (required), `--task <task>` (required), `--force`. Follow src/cmd/attach.rs for turning typed paths into stored paths (paths::stored with the current folder) and for writing a block file through tx.put_raw, and the table of docs/design.md section 17.
  2. Inside tx::write: taken::load; for each stored path the file must exist and be a regular file (Code::NotFound / Code::BadValue naming the path). filesum::stamp and filesum::checksum of the file.
  3. taken::current(path): held by another person and no --force: Code::Refused `<path> is taken by <person> for <task> since <time>; ask them, or add --force`, and nothing is written for any file. Held by the caller's person: change nothing for this file, report it as `already taken by you`. Otherwise: add the block when missing (BlockFile::add_block), set `taken` = `<time> by <person>`, `task`, `size`, `modified`, `checksum`, and `forced: yes` only with --force over another person's take (else remove `forced`); history `taken` with detail `<task> <checksum>`, or `taken by force` with detail `<task> <checksum>, from <person>`.
  4. One tx.put_raw for taken.md with Change { id: taken::ITEM, event: "taken", detail: the paths joined with `, ` }. Warnings (Output::warn): a file name for which paths::windows_safe is false (`<path> cannot be copied to a Windows PC`), and two given paths that differ only in upper and lower case.
  5. Output: `taken for <task>: <n> file(s)`, then one line per file `  <path>  <size> bytes  <checksum>` (or `  <path>  already taken by you`). JSON data {task, files: [{path, size, modified, checksum, already}]}.
  6. tests/cmd_take.rs (new): anna takes a file and taken.md holds the block with all fields and a history line; ben's take of the same file fails with exit 1 naming anna, the task and the time, and taken.md is unchanged; ben with --force takes it and the block has `forced: yes`; anna taking it twice succeeds and changes nothing; a missing file is refused; a path with a space and a non-ASCII letter works; an LLM (`anna - claude`) takes for its person.
  7. Run cargo test --test cmd_take
- accept: A file is recorded as taken with who, for which task, when, and its size, time and checksum; a second person is refused.
- verify: cargo test --test cmd_take
- verified: 2026-11-23
- commit: a6b68a8
- reqs: R-FILE-1, R-XOS-5
- by: Reynan Henry Tandoyo
- note (2026-11-23, planner): added
- note (2026-11-23, haiku-T-321): done

### T-322 · return: check a taken file, end the take, or abandon it
- status: done
- owner: haiku-T-322
- priority: P1 · difficulty: 1 · type: feature
- milestone: T-054
- why: The simulation files are gigabytes; the risk is overwriting a colleague's work when two people copy the same file to their PCs and back.
- summary: The command that protects a colleague's version before a file is copied back, and that says: I am done with this file.
- depends: T-321
- files: src/cmd/return_cmd.rs, tests/cmd_return.rs
- read: docs/design.md, docs/format.md, docs/requirements.md
- steps:
  1. Args: `pub files: Vec<String>` (required), `--check`, `--done`, `--abandon` (exactly one, Code::Usage otherwise), `--force`. Follow the table of docs/design.md section 17.
  2. Every file must be taken now (taken::current; Code::Refused `<path> is not taken`). For --done and --abandon the caller's person must be the one who took it, unless --force (Code::Refused naming the holder).
  3. --check (writes nothing, no tx): for each file compare filesum::stamp and filesum::checksum of the file on disk with the recorded size, modified and checksum. All equal: Output `unchanged: <n> file(s)`, exit 0. Otherwise Code::Refused listing each changed file with what differs (`size`, `time`, `content`); a missing file counts as changed.
  4. --done, inside tx::write: record the file's current size, modified and checksum, remove `taken` and `forced` (BlockFile::set_field with an empty value), history `returned` with the new checksum as detail. --abandon: remove `taken` and `forced`, history `abandoned`. One tx.put_raw with event `returned` or `abandoned`.
  5. Output `returned: <n> file(s)` or `abandoned: <n> file(s)`, one line per path. JSON data {event, files: [{path, checksum}]}.
  6. tests/cmd_return.rs (new): --check after take is exit 0; after the file's content changed it is exit 1 and names the file and `content`; --done records the new checksum and the block has no `taken` field; a second --done is refused; ben cannot return anna's file without --force; --abandon ends the take and keeps the old checksum; two modes at once are exit 2.
  7. Run cargo test --test cmd_return
- accept: return tells whether a file on the server is still the one that was taken, and ends a take with or without a new version.
- verify: cargo test --test cmd_return
- verified: 2026-11-23
- commit: 5ab975a
- reqs: R-FILE-2, R-FILE-3
- by: Reynan Henry Tandoyo
- note (2026-11-23, planner): added
- note (2026-11-23, haiku-T-322): done

### T-323 · files: the taken files, their age, and the log of one file
- status: done
- owner: haiku-T-323
- priority: P1 · difficulty: 1 · type: feature
- milestone: T-054
- why: The simulation files are gigabytes; the risk is overwriting a colleague's work when two people copy the same file to their PCs and back.
- summary: A look at who is working on which large file right now, and the history of one file.
- depends: T-322, T-319
- files: src/cmd/files.rs, tests/cmd_files.rs
- read: docs/design.md, docs/format.md, docs/requirements.md
- steps:
  1. Args gain `--taken` (bool) and `--log <file>`. `--log`: the stored path of the typed file (paths::stored), then taken::log; print one history line per row, or `nothing recorded for <path>`; JSON {path, log}.
  2. `--taken`: taken::all; one line per file `<path>  <person>  <task>  since <date>`, with `  [taken <n> days]` when time::days_between(the take's time, time::now()) is above ctx.cfg.take_days; `no files are taken` when empty; JSON {taken: [{path, person, task, since, days}]}.
  3. In the normal listing: a row whose stored path is taken now gets the mark `taken by <person>` among its marks.
  4. tests/cmd_files.rs: a taken file appears with `--taken` and, when attached to a task, with the mark in `files --task`; with DOSSIER_NOW moved 20 days on (Project::run_at) it has `[taken 20 days]`; `files --log <file>` shows `taken` and, after `return --done`, `returned`.
  5. Run cargo test --test cmd_files
- accept: files shows which files are taken, by whom and since when, marks old takes, and lists every recorded version of a file.
- verify: cargo test --test cmd_files
- verified: 2026-11-23
- commit: 0ecd628
- reqs: R-FILE-5, R-FILE-7, R-FILE-11
- by: Reynan Henry Tandoyo
- note (2026-11-23, planner): added
- note (2026-11-23, haiku-T-323): done

### T-324 · status: the taken files under Working now
- status: done
- owner: haiku-T-324
- priority: P1 · difficulty: 1 · type: feature
- milestone: T-054
- why: The simulation files are gigabytes; the risk is overwriting a colleague's work when two people copy the same file to their PCs and back.
- summary: Whoever asks where the project stands also sees who has which simulation file.
- depends: T-321
- files: src/cmd/status.rs, tests/cmd_status.rs
- read: docs/design.md, docs/format.md, docs/requirements.md
- steps:
  1. In project_screen, where `## Working now` is built from the claimed tasks: add one line per file of taken::all: `<path> · taken by <person> for <task> since <date>`. With --milestone keep only the files whose task is in scope, as the other task lines are. The section is printed when there is a claimed task or a taken file, also without todo.
  2. JSON of status gains `taken`: [{path, person, task}].
  3. tests/cmd_status.rs: a taken file is listed under `## Working now`; after `return --done` it is gone.
  4. Run cargo test --test cmd_status
- accept: status shows the files that are taken next to the tasks that are claimed.
- verify: cargo test --test cmd_status
- verified: 2026-11-23
- commit: 7bebcdf
- reqs: R-CTX-9, R-FILE-5
- by: Reynan Henry Tandoyo
- note (2026-11-23, planner): added
- note (2026-11-23, haiku-T-324): done

### T-325 · remote/transfer.rs: the scp arguments and the steps of take and return from a PC
- status: done
- owner: planner
- priority: P1 · type: milestone
- milestone: T-054
- goal: From a connected folder, take records the take on the server and then copies the files to the PC; return checks first, copies back, and only then ends the take.
- why: The simulation files are gigabytes; the risk is overwriting a colleague's work when two people copy the same file to their PCs and back.
- summary: The part that copies large files between the PC and the server in the safe order.
- depends: T-322, T-311
- accept: From a connected folder, take records the take on the server and then copies the files to the PC; return checks first, copies back, and only then ends the take.
- reqs: R-REMOTE-4, R-FILE-4
- by: Reynan Henry Tandoyo
- note (2026-11-23, planner): added
- note (2026-11-23, planner): split into T-371, T-372, T-373
- note (2026-11-23, planner): done — take and return copy files between a connected PC folder and the server with scp in SFTP mode, safely and retryably.

### T-371 · transfer.rs: the scp program, its arguments and the server path
- status: done
- owner: haiku-T-371
- priority: P0 · difficulty: 1 · type: feature
- milestone: T-325
- why: The simulation files are gigabytes; the risk is overwriting a colleague's work when two people copy the same file to their PCs and back.
- summary: The small pieces needed to copy a file with scp.
- depends: T-322, T-311
- files: src/remote/transfer.rs, tests/common/mod.rs
- read: docs/design.md, docs/format.md, docs/requirements.md
- steps:
  1. program: the value of DOSSIER_SCP when it is set and not empty, else `scp`. argv(source, target, timeout): ["-o", "BatchMode=yes", "-o", "ConnectTimeout=<timeout>", "-p", source, target]. remote_path(server, folder, stored): `<server>:` followed by ssh::quote of `<folder>/<stored>`.
  2. A private helper `files_of(args: &[String]) -> Vec<String>`: the words of args that do not start with `-` and are not the word after `--task` or `--agent`.
  3. tests/common/mod.rs, in `command`: also set DOSSIER_SCP to stand_in("scp"), the same way DOSSIER_SSH is set.
  4. Unit tests in transfer.rs: argv for timeout 10; remote_path for a folder and a file name with a space; files_of for `a.sim --task T-1 sub/b.sim --force` gives the two paths.
  5. Run cargo test --lib remote::transfer::
- accept: The arguments for scp and the path of a file on the server are built correctly, also with spaces.
- verify: cargo test --lib remote::transfer::
- verified: 2026-11-23
- commit: 1477074
- reqs: R-REMOTE-4, NFR-2
- by: Reynan Henry Tandoyo
- note (2026-11-23, planner): added
- note (2026-11-23, haiku-T-371): done
- note (2026-11-23, planner): Its remote_path shell-quoted the path, which SFTP-mode scp reads literally; fixed by T-405 (found with a real sshd in WSL).

### T-372 · transfer.rs: take from a PC
- status: done
- owner: haiku-T-372
- priority: P1 · difficulty: 1 · type: feature
- milestone: T-325
- why: The simulation files are gigabytes; the risk is overwriting a colleague's work when two people copy the same file to their PCs and back.
- summary: Taking a simulation file from the PC: the server notes it, then the file is copied down.
- depends: T-371, T-405
- files: src/remote/transfer.rs
- read: docs/design.md, docs/format.md, docs/requirements.md
- steps:
  1. In `maybe`: when command is `take`, call a private `take(args, base, pointer, agent) -> u8` and return Some of it. Every other case still returns None.
  2. take: build the ssh arguments with ssh::argv_with(&pointer.server, &pointer.folder, None, env, "dossier", ["take", args…, "--json"], false, 10), where env holds DOSSIER_AGENT when agent is Some; run them with ssh::capture.
  3. Exit code not 0: print to standard error `error: <message>`, where the message is `error.message` of the JSON answer on standard output, or else the last line of ssh's standard error; return that exit code. A failure of ssh::capture itself: print `error: <its message>` and return 5.
  4. Exit 0: for each object of `data.files` of the answer take its `path`; create the parent folders of base joined with the path; run the scp program (transfer::program) with argv(remote_path(server, folder, path), the local path as text, 10). Not exit 0: print `error: could not copy <path> from the server; the take is recorded, run the same command again to copy it` and return 5.
  5. All copied: print one line per file `<path>: taken and copied` and return 0.
  6. Unit test: maybe returns None for `files`, `return --check` and `return --abandon` and Some for `take` (use DOSSIER_SSH pointing at a file that does not exist, hold crate::ENV_LOCK, and expect Some(5)).
  7. Run cargo test --lib remote::transfer::
- accept: take from a connected folder records the take on the server first and copies each file afterwards; a failed copy says to run the command again.
- verify: cargo test --lib remote::transfer::
- verified: 2026-11-23
- commit: d57cc79
- reqs: R-REMOTE-4, R-FILE-4
- by: Reynan Henry Tandoyo
- note (2026-11-23, planner): added
- note (2026-11-23, haiku-T-372): done

### T-373 · transfer.rs: return --done from a PC
- status: done
- owner: haiku-T-373
- priority: P1 · difficulty: 1 · type: feature
- milestone: T-325
- why: The simulation files are gigabytes; the risk is overwriting a colleague's work when two people copy the same file to their PCs and back.
- summary: Returning a simulation file from the PC can't overwrite a colleague's version and can't leave half a file.
- depends: T-372, T-407
- files: src/remote/transfer.rs
- read: docs/design.md, docs/format.md, docs/requirements.md
- steps:
  1. src/remote/transfer.rs: add `pub fn remote_target(server: &str, folder: &str, stored: &str) -> String` with the doc comment of docs/design.md section 17: `format!("{server}:{folder}/{stored}")`, unquoted and not escaped. Unit test: `remote_target("anna@server", "/srv/p", "it's [2]*.sim.dossier-part")` is `"anna@server:/srv/p/it's [2]*.sim.dossier-part"`.
  2. In `maybe`: when command is `return` and args contain `--done`, call a private `give_back(args, base, pointer, agent) -> u8` and return Some of it.
  3. A private helper `remote(pointer, sub: Option<&str>, agent, program, args) -> Result<(u8, String, String)>`: ssh::capture of ssh::argv_with(server, folder, sub, crate::remote::forward::remote_variables(program, agent), program, args, false, TIMEOUT_SECONDS). After each call print the captured standard output to standard output and standard error to standard error.
  4. give_back: first `let sub = std::env::current_dir().ok().and_then(|current| crate::remote::forward::sub_folder(&current, base));` and the files = files_of(args). Then in this order: (a) remote(pointer, sub, agent, "dossier", ["return", files…, "--check"]); not 0: return its exit code. (b) each local file (the current folder joined with the path as typed) must exist, else print `error: <path> is not in this folder` and return 1. (c) for each file: the stored path is `<sub>/<path>` (just `<path>` without sub, `\` turned into `/`); scp with argv(the local path, remote_target(server, folder, `<stored>.dossier-part`), TIMEOUT_SECONDS), then remote(pointer, sub, agent, "mv", ["-f", "--", `<path>.dossier-part`, `<path>`]); a failure of either prints `error: could not copy <path> to the server; the take is unchanged, run the same command again` and returns 5. (d) remote(pointer, sub, agent, "dossier", ["return", files…, "--done"]) and return its exit code.
  5. Unit test: maybe returns Some for `return a.sim --done` (with DOSSIER_SSH pointing at a file that does not exist, hold crate::ENV_LOCK, expect Some(5)).
  6. Run cargo test --lib remote::transfer::
- accept: return --done from a connected folder checks on the server first, copies the file up under a temporary name, moves it into place, and only then ends the take.
- verify: cargo test --lib remote::transfer::
- verified: 2026-11-23
- commit: 715646a
- reqs: R-REMOTE-4, R-FILE-2, R-FILE-3
- by: Reynan Henry Tandoyo
- note (2026-11-23, planner): added
- note (2026-11-23, planner): Steps rewritten: the commands run in the subfolder with the version variables, and the upload target is remote_target (not escaped; SFTP does not glob a target, tested 2026-11-23).
- note (2026-11-23, haiku-T-373): done

### T-326 · forward.rs: take and return in a connected folder go through transfer
- status: done
- owner: planner
- priority: P1 · type: milestone
- milestone: T-054
- goal: In a connected folder take and return --done go through transfer and copy the files in the right order; a failed copy leaves the take as it was.
- why: The simulation files are gigabytes; the risk is overwriting a colleague's work when two people copy the same file to their PCs and back.
- summary: From the PC, taking and returning a simulation file is one command each.
- depends: T-325, T-313
- accept: In a connected folder take and return copy the files in the right order, and a failed copy leaves the take as it was.
- reqs: R-REMOTE-4
- by: Reynan Henry Tandoyo
- note (2026-11-23, planner): added
- note (2026-11-23, planner): split into T-374, T-375
- note (2026-11-23, planner): done — take and return in a connected folder copy the files through scp; tested with stand-ins on Windows and Linux.

### T-374 · forward.rs: hand take and return to transfer
- status: done
- owner: haiku-T-374
- priority: P1 · difficulty: 1 · type: feature
- milestone: T-326
- why: The simulation files are gigabytes; the risk is overwriting a colleague's work when two people copy the same file to their PCs and back.
- summary: Typing take or return on the PC starts the copy steps.
- depends: T-325, T-313, T-408
- files: src/remote/forward.rs
- read: docs/design.md, docs/format.md, docs/requirements.md
- steps:
  1. In `maybe`, after the target is known and before the generic forwarding: when the target comes from a pointer file and the command is `take` or `return`, call transfer::maybe(command, the arguments after the command, the folder that holds the pointer, the pointer, the agent name) and return its answer when it is Some.
  2. When the target comes from --project (no pointer) and the command is `take`, or `return` with `--done`: print `error: take and return --done copy files and need a connected folder: dossier connect` to standard error and return Some(2).
  3. Add a unit test for the small helper that decides this (a function taking the command and the arguments and returning whether files are copied).
  4. Run cargo test --lib remote::forward::
- accept: forward.rs sends take and return --done of a connected folder through transfer, and refuses them without a connected folder.
- verify: cargo test --lib remote::forward::
- verified: 2026-11-23
- commit: c6144fb
- reqs: R-REMOTE-4
- by: Reynan Henry Tandoyo
- note (2026-11-23, planner): added
- note (2026-11-23, haiku-T-374): done

### T-375 · Test: take and return from a connected folder with stand-ins for ssh and scp
- status: done
- owner: haiku-T-375
- priority: P1 · difficulty: 1 · type: feature
- milestone: T-326
- why: The simulation files are gigabytes; the risk is overwriting a colleague's work when two people copy the same file to their PCs and back.
- summary: A test proves that files are only copied when the server agreed.
- depends: T-374, T-409
- files: tests/remote_transfer.rs
- read: docs/design.md, docs/format.md, docs/requirements.md
- steps:
  1. Create tests/remote_transfer.rs using tests/common. Write stand-ins with Project::write_stand_in: `scp` appends its arguments to scp-calls.txt, writes the text `copied` to its last argument (the target; for a download it is a local path ending in `.dossier-part`) and exits 0 (model: the ssh stand-in of tests/cmd_projects.rs). `ssh` appends its arguments to ssh-calls.txt, prints one fixed line and exits 0 (model: tests/cmd_review_task.rs prints a JSON envelope); the line is {"schema":"dossier.take.v1","ok":true,"data":{"task":"T-1","files":[{"path":"sim/a.sim"}]},"warnings":[]}.
  2. Connect the project folder: `dossier connect --code P-001 anna@server:/projects/p1`. Do not create sim/a.sim locally.
  3. Test take: `dossier take sim/a.sim --task T-1` exits 0; ssh-calls.txt has one line containing `take` and `DOSSIER_CLIENT_VERSION`; scp-calls.txt has one line that contains `-s`, whose source contains `anna@server:` and `/projects/p1/sim/a.sim` and whose target ends with `a.sim.dossier-part`; afterwards sim/a.sim exists with the text `copied` and no `.dossier-part` file is left.
  4. Test a kept local file (D46): create sim/a.sim with the text `mine`, run the same take: it exits 0, standard output contains `kept your local copy`, scp-calls.txt does not exist, and sim/a.sim still reads `mine`.
  5. Test failed copy: with an scp stand-in that exits 1 and writes nothing, the take exits 5, standard error contains `run the same command again`, and neither sim/a.sim nor sim/a.sim.dossier-part exists.
  6. Test return: `dossier return sim/a.sim --done` with the ssh stand-in exiting 1 exits 1 and scp-calls.txt does not exist; `dossier return sim/a.sim --check` calls ssh once and scp never.
  7. Run cargo test --test remote_transfer on this system; the stand-ins must have a Windows text and a Unix text.
- accept: The order of the steps of take and return from a PC is pinned by a test that needs no real server.
- verify: cargo test --test remote_transfer
- verified: 2026-11-23
- commit: 7092764
- reqs: R-REMOTE-4, TEST-8
- by: Reynan Henry Tandoyo
- note (2026-11-23, planner): added
- note (2026-11-23, haiku-T-375): done

### T-327 · README: large files
- status: done
- owner: haiku-T-327
- priority: P1 · difficulty: 1 · type: docs
- milestone: T-054
- why: The simulation files are gigabytes; the risk is overwriting a colleague's work when two people copy the same file to their PCs and back.
- summary: Team members can read how to work on a large file locally without overwriting a colleague's version.
- depends: T-326, T-323
- files: README.md
- read: docs/design.md, docs/format.md, docs/requirements.md
- steps:
  1. Add a section `### Large files: take and return` under `## Remote mode: from a Windows PC`: the three steps (take, work locally, return --done), what a refused take looks like, `return --check`, `--abandon`, `--force`, `files --taken` and `files --log`, and what to do when a copy failed (run the same command again). State plainly that dossier never stores the files: it records who has them and checks before they come back. Say that a take never overwrites a file that is already in the folder (D46): it keeps the local copy and says so; delete the local file and run the same take to get the server's version.
  2. README.md, in the large-files part: one sentence that take and return from a PC need scp from OpenSSH 8.7 or newer (it is run with -s, the SFTP protocol); Windows 11 has it, and an older scp stops with an unknown-option error. Check the version with `ssh -V`.
  3. Run cargo test --test smoke
- accept: The README explains how to take a simulation file to a PC and return it safely.
- verify: cargo test --test smoke
- verified: 2026-11-23
- commit: dd513cd
- reqs: R-FILE-1 to R-FILE-5, REL-2
- by: Reynan Henry Tandoyo
- note (2026-11-23, planner): added
- note (2026-11-23, haiku-T-327): done

### T-328 · Release 0.5.0
- status: done
- owner: haiku-T-328
- priority: P1 · difficulty: 1 · type: chore
- milestone: T-054
- why: The simulation files are gigabytes; the risk is overwriting a colleague's work when two people copy the same file to their PCs and back.
- summary: The version number and the README are ready for the owner to tag release 0.5.0.
- depends: T-402, T-327, T-324, T-316, T-410, T-411
- files: Cargo.toml, Cargo.lock, README.md
- read: docs/design.md, docs/format.md, docs/requirements.md
- steps:
  1. Cargo.toml: version = "0.5.0"; run cargo build so that Cargo.lock follows (only the version line of the dossier package may change).
  2. README.md: set the status line at the top to `**Status: v0.5 (phase 5)**`; add `### What version 0.5 adds` directly after the `### What version 0.4 adds` section: a short summary with links to the sections that already describe it, `## Remote mode: from a Windows PC`, `### Check the connection` and `### Large files: take and return`: connect --check; the PC's versions compared with the server's; commands typed in a subfolder of a connected folder run in the same subfolder on the server; one ssh connection per command (D43, there is no connection reuse); take and return of large files, also from a PC, where a take never overwrites a local file (D46) and a copy goes through a part file; the taken files in files and status. Keep it short; do not repeat the long sections. Then rename `### Not in version 0.4` to `### Not in version 0.5` and remove its phase 5 line (what is left: phase 6).
  3. Run cargo fmt --all -- --check, cargo clippy --all-targets --all-features -- -D warnings and cargo test --all-features; all three must pass. Do not tag and do not push: the owner does that.
- accept: Version 0.5.0 builds, passes every check, and the README says what it adds.
- verify: cargo fmt --all -- --check && cargo clippy --all-targets --all-features -- -D warnings && cargo test --all-features
- verified: 2026-11-23
- commit: 98592bd
- reqs: REL-1, REL-2
- by: Reynan Henry Tandoyo
- note (2026-11-23, planner): added
- note (2026-11-23, planner): Steps updated: no connection reuse (D43); the 0.5 summary links the sections T-316 and T-327 wrote.
- note (2026-11-23, haiku-T-328): done

### T-055 · Growing projects: the context stays small and routine steps run by themselves
- status: open
- owner: —
- priority: P3 · type: milestone
- goal: In a long project brief still fits its limit and still holds what the LLM needs, and session review, hooks and export take routine work off the people. Requirements: docs/requirements.md section 11, Phase 6 (v0.6; phase 5 before D45). Each task here is split when the phase starts.
- summary: In a long project the context for an LLM stays small, and routine steps happen by themselves.
- note (2026-11-20, planner): added
- note (2026-11-23, planner): Owner, 2026-11-23 (D45): moved from phase 5 to phase 6; it is released as 0.6.0 (T-355), after 0.5.0.

### T-056 · Milestone digests: digest, and digests in brief and status
- status: open
- owner: —
- priority: P3 · type: milestone
- milestone: T-055
- goal: R-CTX-14 and R-CTX-15 hold: a closed milestone is summed up in a digest that a person approves, and brief and status show the digest in place of the entries it covers.
- why: The context an LLM gets must stay small as a project grows.
- summary: When a milestone is closed its decisions and findings are summed up, and brief shows the summary instead.
- depends: T-054
- accept: R-CTX-14 and R-CTX-15 hold.
- reqs: R-CTX-14, R-CTX-15, D22
- note (2026-11-20, planner): added
- note (2026-11-23, planner): split into T-329, T-330, T-331, T-332, T-333, T-334, T-335

### T-329 · Entry kinds: the digest (DIG)
- status: done
- owner: haiku-T-329
- priority: P0 · difficulty: 1 · type: feature
- milestone: T-056
- why: The context an LLM gets must stay small as a project grows (D22).
- summary: A new kind of entry that sums up a closed milestone.
- files: src/entry/mod.rs, src/entry/rules.rs, src/entry/new.rs
- read: docs/design.md, docs/format.md, docs/requirements.md
- steps:
  1. src/entry/mod.rs: add `Digest` as the last variant of Kind: all() gains it last (its array length rises by one); name `digest`, prefix `DIG`, folder `digests`. Update the tests of this file that list or count the kinds.
  2. src/entry/rules.rs: required_sections(Digest) = ["Summary"]; section_order = ["Summary"]; `problems` reports `missing field milestone` and `missing field tasks` for a digest without them, and `Summary has more than 15 lines` when it has. Update the per-kind test arrays of this file.
  3. src/entry/new.rs: FIELD_ORDER gains `milestone`, `tasks` and `standing`, in this order directly after `task` (raise the array length by three).
  4. Tests: Kind::parse("DIG") gives Digest; a digest Draft with milestone, tasks and a three-line Summary has no problems; one with 16 lines has one.
  5. Run cargo test --lib entry:: and cargo build; where the compiler asks for a match arm for Digest, add the arm that treats it like a lesson.
- accept: The program knows the digest as a kind of entry with its folder, section and fields.
- verify: cargo test --lib entry::
- verified: 2026-11-23
- commit: b3ef2ae
- reqs: R-CTX-14, D22
- by: Reynan Henry Tandoyo
- note (2026-11-23, planner): added
- note (2026-11-23, haiku-T-329): done

### T-330 · digest.rs: which digest covers which entry
- status: done
- owner: haiku-T-330
- priority: P0 · difficulty: 1 · type: feature
- milestone: T-056
- why: The context an LLM gets must stay small as a project grows (D22).
- summary: The rule that decides which single entries a milestone summary stands for.
- depends: T-329
- files: src/digest.rs, src/approval.rs, src/lib.rs
- read: docs/design.md, docs/format.md, docs/requirements.md
- steps:
  1. Create src/digest.rs with in_force, covering and RULES exactly as in docs/design.md section 18, and add `pub mod digest;` to src/lib.rs. src/approval.rs: `guarded` is also true for Kind::Digest (update its test).
  2. in_force: the items of kind Digest with status approved; with a day, those whose valid-from date is on or before it and whose valid-until is empty or after it (as src/filter.rs does for --on); newest `created` first.
  3. covering: the first digest whose `tasks` list (Entry::list) contains the entry's `task`, case ignored; None when the entry has no task, is a decision with `standing: yes`, or is an assumption or investigation with `state: open`.
  4. RULES, five texts: `At most 15 lines.`, `What was decided and still binds.`, `What was found.`, `What was assumed and is still open.`, `What did not work.`
  5. Unit tests: a decision of a task in the list is covered; one with standing: yes is not; an open assumption is not; a confirmed one is; a digest that is a draft covers nothing.
  6. Run cargo test --lib digest:: and cargo test --lib approval::
- accept: The program can tell which entries an approved digest replaces in brief and status.
- verify: cargo test --lib digest::
- verified: 2026-11-24
- commit: ec0a1fe
- reqs: R-CTX-15
- by: Reynan Henry Tandoyo
- note (2026-11-23, planner): added
- note (2026-11-24, haiku-T-330): done

### T-331 · digest: the material for summing up a milestone, and writing the digest
- status: open
- owner: —
- priority: P1 · type: milestone
- milestone: T-056
- goal: digest prints what a closed milestone decided, found, assumed and tried, with the rules for a summary, and stores the summary as an entry that a person approves.
- why: The context an LLM gets must stay small as a project grows (D22).
- summary: The command that turns a finished milestone into fifteen lines.
- depends: T-330
- accept: digest prints what a closed milestone decided, found, assumed and tried, with the rules for a summary, and stores the summary the LLM writes as an entry that a person approves.
- reqs: R-CTX-14, D7
- note (2026-11-23, planner): added
- note (2026-11-23, planner): split into T-376, T-377, T-378, T-379

### T-376 · digest: the command file and its place in the command list
- status: done
- owner: haiku-T-376
- priority: P0 · difficulty: 1 · type: chore
- milestone: T-331
- why: The context an LLM gets must stay small as a project grows (D22).
- summary: An empty place for the new command.
- depends: T-330
- files: src/cmd/digest.rs, src/cmd/mod.rs, src/cli.rs
- read: docs/design.md, docs/format.md, docs/requirements.md
- steps:
  1. Create src/cmd/digest.rs with `#[derive(clap::Args, Debug)] pub struct Args {}` and `pub fn run(_args: &Args, _g: &Global) -> Result<Output>` returning Err(Error::new(Code::NotImplemented, "<command> is not built yet")), with the same `use` lines as src/cmd/guard.rs.
  2. src/cmd/mod.rs: `pub mod digest;` at its alphabetical place. src/cli.rs: `Digest(cmd::digest::Args),` in the Command enum and the arm with the name "digest" in the match of main_exit.
  3. Run cargo build and cargo test --test smoke
- accept: `dossier digest` exists and answers that it is not built yet.
- verify: cargo build && cargo test --test smoke
- verified: 2026-11-24
- commit: 53dc69f
- reqs: R-CTX-14
- by: Reynan Henry Tandoyo
- note (2026-11-23, planner): added
- note (2026-11-24, haiku-T-376): done

### T-377 · digest: in the help overview and among the writing commands
- status: done
- owner: haiku-T-377
- priority: P1 · difficulty: 1 · type: chore
- milestone: T-331
- why: The context an LLM gets must stay small as a project grows (D22).
- summary: The new command is listed.
- depends: T-376
- files: src/cmd/help.rs, src/ctx.rs, tests/cmd_agents.rs
- read: docs/design.md, docs/format.md, docs/requirements.md
- steps:
  1. src/cmd/help.rs: ("digest", "Sums up a closed milestone") in COMMANDS, length raised by one.
  2. src/ctx.rs: "digest" in WRITING, length raised by one.
  3. tests/cmd_agents.rs: help_without_a_topic_lists_every_command_in_order compares the whole list that `dossier help` prints; when the new commands now appear in it, add them to the expected list at the place where `dossier help` prints them, and change nothing else in that file.
  4. Run cargo test --lib ctx:: and cargo test --test smoke
- accept: The overview lists digest, and it follows the name rule of writing commands.
- verify: cargo test --lib ctx:: && cargo test --test smoke
- verified: 2026-11-24
- commit: 6ef109f
- reqs: R-CTX-14
- by: Reynan Henry Tandoyo
- note (2026-11-23, planner): added
- note (2026-11-23, planner): Added tests/cmd_agents.rs: its help overview test lists every command (found in T-359).
- note (2026-11-24, haiku-T-377): done

### T-378 · digest <milestone>: the material for the summary
- status: done
- owner: haiku-T-378
- priority: P1 · difficulty: 1 · type: feature
- milestone: T-331
- why: The context an LLM gets must stay small as a project grows (D22).
- summary: Everything an LLM needs to sum up a finished milestone in fifteen lines.
- depends: T-376
- files: src/cmd/digest.rs, tests/cmd_digest.rs
- read: docs/design.md, docs/format.md, docs/requirements.md
- steps:
  1. Args: `pub milestone: String`, `--write`, `--stdin`, `--title <t>`. `--write` without `--stdin` is Code::Usage. This task builds the form without --write; with --write return Code::NotImplemented for now.
  2. Read todo: todo_bridge::json(&ctx.root, &["show", ms]) for the title (`title`), the goal (`goal`, else `fields.goal`) and the status; todo_bridge::json(&ctx.root, &["list", "--milestone", ms, "--status", "all"]) for `tasks[]` with id, title and status. The task list of the milestone is its own ID and those IDs. Put both reads in one private function that the next task uses too.
  3. Print: `# Digest of <ms> · <title>`; `goal: <goal>`; `## Tasks` with one line `<id> · <title> (<status>)` each; then the approved entries whose `task` is in the list (case ignored), under `## Decisions`, `## Assumptions` (with ` [<state>]`), `## Findings`, `## Dead ends` (findings with outcome `dead end`, not repeated under Findings) and `## Investigations` (with ` [<state>]`): one line `<ID> · <title>` each, followed by the first line of its first section indented by two spaces. A heading without entries is left out. Then `## Rules for the summary` with digest::RULES numbered, and the line `Write it with: dossier digest <ms> --write --stdin`.
  4. It changes nothing. JSON data {milestone, title, goal, tasks, entries: [ids], rules}.
  5. tests/cmd_digest.rs (new): a todo stand-in that answers both calls (its text can test the first argument for `show` or `list`; model: tests/cmd_status.rs); the material lists the tasks, a decision and a dead end of the milestone and the five rules; an entry of another task is not listed; the journal is unchanged afterwards.
  6. Run cargo test --test cmd_digest
- accept: digest <milestone> prints the milestone's goal, tasks, decisions, assumptions, findings, dead ends and investigations, and the rules for the summary.
- verify: cargo test --test cmd_digest
- verified: 2026-11-24
- commit: c6048f0
- reqs: R-CTX-14, D7
- by: Reynan Henry Tandoyo
- note (2026-11-23, planner): added
- note (2026-11-24, haiku-T-378): done

### T-379 · digest --write: store the summary as an entry
- status: done
- owner: haiku-T-379
- priority: P1 · difficulty: 1 · type: feature
- milestone: T-331
- why: The context an LLM gets must stay small as a project grows (D22).
- summary: The summary of a milestone becomes an entry that a person approves.
- depends: T-378, T-377
- files: src/cmd/digest.rs, tests/cmd_digest.rs
- read: docs/design.md, docs/format.md, docs/requirements.md
- steps:
  1. With --write --stdin: read todo as the material form does; the milestone's status must be `done`, else Code::Refused `<ms> is not finished in todo`.
  2. Read standard input as the Summary (trimmed). Empty, or more than 15 lines: Code::BadValue saying so.
  3. tx::write with create::create: New { kind: Kind::Digest, title: --title or `Digest of <ms>: <milestone title>`, fields: [("milestone", ms in upper case), ("tasks", the task list joined with `, `)], sections: [("Summary", text)], said_by: None, draft: false }. An LLM's digest becomes a draft by itself because the kind is guarded.
  4. Output: first line as other write commands print it (`DIG-001: added (approved)` or `DIG-001: added (draft, waiting for approval)`). When an approved digest with the same `milestone` exists already, add the warning `<old ID> already sums up <ms>: replace it with dossier supersede <old ID> --by <new ID>`.
  5. tests/cmd_digest.rs: as anna, --write creates an approved DIG-001 with milestone, tasks and Summary; as `anna - claude` it is a draft; 16 lines are exit 1 and nothing is written; a milestone whose status is `open` is refused; a second digest of the same milestone gives the warning.
  6. Run cargo test --test cmd_digest
- accept: digest --write stores at most 15 lines as a digest entry of the milestone; an LLM's is a draft until a person approves it.
- verify: cargo test --test cmd_digest
- verified: 2026-11-24
- commit: a32a365
- reqs: R-CTX-14, D5
- by: Reynan Henry Tandoyo
- note (2026-11-23, planner): added
- note (2026-11-24, haiku-T-379): done

### T-332 · decide --standing: a decision that binds the whole project
- status: done
- owner: haiku-T-332
- priority: P1 · difficulty: 1 · type: feature
- milestone: T-056
- why: The context an LLM gets must stay small as a project grows (D22).
- summary: Decisions that hold for the whole project stay visible one by one.
- depends: T-329
- files: src/cmd/decide.rs, tests/cmd_decide.rs
- read: docs/design.md, docs/format.md, docs/requirements.md
- steps:
  1. Args gain `--standing` (bool); with it the new decision gets the field `standing` = `yes`.
  2. tests/cmd_decide.rs: a decision made with --standing has the field; one without has not; `set DEC-001 --field standing=yes` adds it to an existing one (this works already; the test pins it).
  3. Run cargo test --test cmd_decide
- accept: A decision can be marked as standing, so that no digest ever hides it.
- verify: cargo test --test cmd_decide
- verified: 2026-11-23
- commit: d520534
- reqs: R-CTX-15
- by: Reynan Henry Tandoyo
- note (2026-11-23, planner): added
- note (2026-11-23, haiku-T-332): done

### T-333 · brief: a digest in place of the entries it covers
- status: done
- owner: haiku-T-333
- priority: P1 · difficulty: 1 · type: feature
- milestone: T-056
- why: The context an LLM gets must stay small as a project grows (D22).
- summary: In a long project the session start stays short: finished milestones appear as their summaries.
- depends: T-330, T-332
- files: src/cmd/brief.rs, tests/cmd_brief.rs
- read: docs/design.md, docs/format.md, docs/requirements.md
- steps:
  1. In Brief::collect: digests = digest::in_force(items, day). Leave out of the decisions and of the findings every entry for which digest::covering(&digests, entry) is Some, and count them. Open assumptions are never left out.
  2. Print, before `## Decisions in force`, a section `## Digests`: for each digest `<DIG-ID> · <milestone> · <title>` and its Summary lines indented by two spaces; after the section one line `<n> entries are summed up in digests: dossier list --task <task>` when n > 0 (docs/design.md section 18). cut_one never cuts a digest line. Brief::data gains `digests` (IDs) and `summed_up` (the count).
  3. tests/cmd_brief.rs: with an approved digest whose tasks list names T-010, a decision and a finding of T-010 are not printed and the digest's summary is; a decision of T-010 with standing: yes is still printed in full; an open assumption of T-010 is still printed; a draft digest changes nothing; with 500 decisions of which 480 are covered the brief keeps its limit.
  4. Run cargo test --test cmd_brief
- accept: Once a milestone has an approved digest, the session brief shows the digest instead of that milestone's single decisions and findings.
- verify: cargo test --test cmd_brief
- verified: 2026-11-24
- commit: 9c8c7b8
- reqs: R-CTX-15, TEST-9
- by: Reynan Henry Tandoyo
- note (2026-11-23, planner): added
- note (2026-11-24, haiku-T-333): done

### T-334 · status: a digest in place of the entries it covers
- status: done
- owner: haiku-T-334
- priority: P1 · difficulty: 1 · type: feature
- milestone: T-056
- why: The context an LLM gets must stay small as a project grows (D22).
- summary: The state of a long project stays readable on one screen.
- depends: T-330
- files: src/cmd/status.rs, tests/cmd_status.rs
- read: docs/design.md, docs/format.md, docs/requirements.md
- steps:
  1. In project_screen: digests = digest::in_force(all, the day). Leave out of `## Decisions in force` and `## Newest findings` every entry that digest::covering covers. Add a section `## Digests` before the decisions: one line per digest `<DIG-ID> · <milestone> · <title>`, and the line `<n> entries are summed up in digests: dossier show <DIG-ID>` when n > 0. With --milestone <ms> keep only the digest whose `milestone` is <ms>.
  2. JSON gains `digests` (IDs).
  3. tests/cmd_status.rs: a covered decision is not listed and the digest is; a standing decision stays.
  4. Run cargo test --test cmd_status
- accept: status shows finished milestones as their digests.
- verify: cargo test --test cmd_status
- verified: 2026-11-24
- commit: ca3e992
- reqs: R-CTX-15
- by: Reynan Henry Tandoyo
- note (2026-11-23, planner): added
- note (2026-11-24, haiku-T-334): done

### T-335 · Agent rules and README: digests
- status: open
- owner: —
- priority: P1 · difficulty: 1 · type: docs
- milestone: T-056
- why: The context an LLM gets must stay small as a project grows (D22).
- summary: LLMs and people learn when and how a milestone is summed up.
- depends: T-331
- files: src/agents/rules.md, README.md, AGENTS.md
- read: docs/design.md, docs/format.md, docs/requirements.md
- steps:
  1. src/agents/rules.md: add one line: `- When a milestone is closed in todo, sum it up: run `dossier digest <milestone>`, then write at most 15 lines with `dossier digest <milestone> --write --stdin`; a person approves it.` The file stays at 40 lines or fewer.
  2. README.md: a short section on digests under the newest version's additions: the two forms of the command, that brief and status then show the digest, `decide --standing`, and that open assumptions are always shown singly.
  3. If AGENTS.md of this repository holds a dossier rules block, bring it up to date with cargo run -- agents install --yes.
  4. Run cargo test --lib agents:: and cargo test --test cmd_agents --test cmd_skill; update an expected rules text where a test compares it.
- accept: The agent rules tell an LLM to sum up a closed milestone, and the README explains digests.
- verify: cargo test --lib agents:: && cargo test --test cmd_agents --test cmd_skill
- reqs: R-CTX-14, R-AGENT-1
- note (2026-11-23, planner): added

### T-057 · Session review (R-DRAFT-7, R-DRAFT-9)
- status: open
- owner: —
- priority: P3 · type: milestone
- milestone: T-055
- goal: R-DRAFT-7 and R-DRAFT-9 hold: the decisive parts of a session are printed as material for drafting from a session file in dossier's own versioned format (D44), a Claude Code transcript is converted to it, and a hook can start the team's own review command when a session ends.
- why: Capture only becomes a habit when the routine steps happen by themselves.
- summary: At the end of a session the material for drafting lessons is prepared, so drafts are waiting the next morning.
- depends: T-054
- accept: R-DRAFT-7, R-DRAFT-9, R-AGENT-3, R-AGENT-4, R-CTX-7 hold.
- reqs: R-DRAFT-7, R-DRAFT-9, R-AGENT-3, R-AGENT-4, R-CTX-7
- note (2026-11-20, planner): added
- note (2026-11-21, planner): Claude Code hooks and the skill (R-AGENT-3, R-AGENT-4, R-CTX-7) moved to phase 2 by the owner (D33): T-163 to T-167. What stays here is the session review (R-DRAFT-7, R-DRAFT-9).
- note (2026-11-23, planner): split into T-336, T-337, T-338, T-339, T-340

### T-336 · session.rs and claude.rs: dossier's own session format, and the converter from a Claude Code transcript
- status: done
- owner: haiku-T-336
- priority: P0 · difficulty: 1 · type: feature
- milestone: T-057
- why: Capture only becomes a habit when the routine steps happen by themselves.
- summary: The session review gets its own versioned file format, and one small converter that reads Claude Code's transcript.
- files: src/session.rs, src/claude.rs, src/lib.rs
- read: docs/design.md, docs/format.md, docs/requirements.md
- steps:
  1. Create src/session.rs and src/claude.rs with exactly the items of docs/design.md section 19 and add `pub mod claude;` and `pub mod session;` to src/lib.rs.
  2. session::render and session::parse follow docs/format.md section 34: the format line `<!-- dossier-session: 1 -->`, `# Session`, `- tool: <tool>`, then `## user` / `## assistant` blocks with one empty line after the heading and after the text. A text line that starts with `## ` is written with one space in front and read back without it. parse: a format line with a number above session::VERSION is Code::TooNew `this session file has format <n>; update dossier`; no format line reads as version 1; a `## ` heading with another name starts a block that is skipped; a turn with empty text is left out.
  3. session::CUES, sixteen lower-case words or phrases: decide, decided, decision, we will, let's use, instead of, because, assume, result, converge, diverge, failed, works, wrong, fixed, turned out. session::relevant: a turn counts when its text, in lower case, contains a cue; take each such turn and the turn before it, each once, in order, as `<role>: <text>` with the text cut to 1500 characters; while the total length is above `limit`, drop the oldest.
  4. claude::turns: for each line parse JSON and skip a line that does not parse. Keep objects whose `type` is `user` or `assistant`. The text is `message.content` when it is a string, or, when it is an array, the `text` values of its items whose `type` is `text`, joined with a line break. Skip a turn whose text is empty. Nothing else is read.
  5. Unit tests. session: parse(render("claude code", turns)) gives the turns back, also for a text with a line starting with `## `; the example of format.md section 34 parses to two turns; a file with `<!-- dossier-session: 2 -->` is TooNew; relevant keeps a turn with `we will` and the turn before it, drops one without a cue, and keeps only the newest with a small limit. claude: a user line with a string content and an assistant line with an array of a text item and a tool_use item give two turns; a line that is not JSON and a line of type `summary` are skipped.
  6. Run cargo test --lib session:: and cargo test --lib claude::
- accept: A session file is written and read back exactly, a newer version is refused with a message to update, and a Claude Code transcript converts to turns without ever failing on an unknown line.
- verify: cargo test --lib session:: && cargo test --lib claude::
- verified: 2026-11-23
- commit: ec8c0ac
- reqs: R-DRAFT-7, D44
- by: Reynan Henry Tandoyo
- note (2026-11-23, planner): added
- note (2026-11-23, haiku-T-336): done

### T-337 · review-session and session convert: the material of a whole session for drafting
- status: done
- owner: planner
- priority: P1 · type: milestone
- milestone: T-057
- goal: review-session prints the decisive parts of a session file and the drafting rules and changes nothing; with --claude it takes a Claude Code transcript; session convert writes the session file; all of them run on the machine of the session.
- why: Capture only becomes a habit when the routine steps happen by themselves.
- summary: After a session, the LLM gets the material to propose lessons from the whole conversation.
- depends: T-336
- accept: review-session prints the decisive parts of a session file and the drafting rules, cut to a size limit, and changes nothing; with --claude it takes a Claude Code transcript, and session convert writes the session file.
- reqs: R-DRAFT-7, D7, D44
- by: Reynan Henry Tandoyo
- note (2026-11-23, planner): added
- note (2026-11-23, planner): split into T-380, T-381, T-382, T-383, T-384
- note (2026-11-24, planner): done — review-session prints the decisive turns of a session with the drafting rules, and session convert --claude makes a session file from a Claude Code transcript.

### T-380 · Stub files: review-session and session
- status: done
- owner: haiku-T-380
- priority: P0 · difficulty: 1 · type: chore
- milestone: T-337
- why: Capture only becomes a habit when the routine steps happen by themselves.
- summary: Empty files for the two new commands.
- files: src/cmd/review_session.rs, src/cmd/session.rs, src/cmd/mod.rs
- read: docs/design.md, docs/format.md, docs/requirements.md
- steps:
  1. Create both files, each with `#[derive(clap::Args, Debug)] pub struct Args {}` and `pub fn run(_args: &Args, _g: &Global) -> Result<Output>` returning Err(Error::new(Code::NotImplemented, "<command> is not built yet")), with the same `use` lines as src/cmd/guard.rs.
  2. Check them with rustfmt --check --edition 2024 src/cmd/review_session.rs src/cmd/session.rs
  3. src/cmd/mod.rs: `pub mod review_session;` and `pub mod session;` at their alphabetical places. These modules exist from this task on, so they are compiled and checked; the register task adds them to cli.rs and help.rs.
  4. Run cargo build (it must still pass).
- accept: The two stub files exist and are formatted.
- verify: rustfmt --check --edition 2024 src/cmd/review_session.rs src/cmd/session.rs && cargo build
- verified: 2026-11-24
- commit: 031b715
- reqs: R-DRAFT-7
- by: Reynan Henry Tandoyo
- note (2026-11-23, planner): added
- note (2026-11-24, haiku-T-380): done

### T-381 · Register review-session and session
- status: done
- owner: haiku-T-381
- priority: P0 · difficulty: 1 · type: chore
- milestone: T-337
- why: Capture only becomes a habit when the routine steps happen by themselves.
- summary: The two new commands become part of the program.
- depends: T-380
- files: src/cli.rs, src/cmd/help.rs, tests/cmd_agents.rs
- read: docs/design.md, docs/format.md, docs/requirements.md
- steps:
  1. src/cli.rs: `ReviewSession(cmd::review_session::Args)` and `Session(cmd::session::Args)` in the Command enum, and two arms in the match of main_exit with the names "review-session" and "session".
  2. src/cmd/help.rs: ("review-session", "Material from a session file") and ("session", "Converts a transcript") in COMMANDS, length raised by two.
  3. tests/cmd_agents.rs: help_without_a_topic_lists_every_command_in_order compares the whole list that `dossier help` prints; when the new commands now appear in it, add them to the expected list at the place where `dossier help` prints them, and change nothing else in that file.
  4. Run cargo build and cargo test --test smoke
- accept: The two commands are part of the program and the overview.
- verify: cargo build && cargo test --test smoke
- verified: 2026-11-24
- commit: e043e16
- reqs: R-DRAFT-7
- by: Reynan Henry Tandoyo
- note (2026-11-23, planner): added
- note (2026-11-23, planner): Added tests/cmd_agents.rs: its help overview test lists every command (found in T-359).
- note (2026-11-23, planner): src/cmd/mod.rs moved to T-380, so this task stays at 3 files (level 1).
- note (2026-11-24, haiku-T-381): done

### T-382 · review-session and session run on the PC; the drafting rules become public
- status: done
- owner: haiku-T-382
- priority: P0 · difficulty: 1 · type: chore
- milestone: T-337
- why: Capture only becomes a habit when the routine steps happen by themselves.
- summary: The session commands stay on the machine where the session file is.
- files: src/remote/forward.rs, src/cmd/review_task.rs
- read: docs/design.md, docs/format.md, docs/requirements.md
- steps:
  1. src/remote/forward.rs: add "review-session" and "session" to LOCAL_COMMANDS and raise its length by two; extend the comment above it: they read a file of this machine, which the server could not read.
  2. src/cmd/review_task.rs: make the RULES constant `pub`.
  3. Run cargo test --lib remote::forward:: and cargo test --test cmd_review_task
- accept: In a connected folder the two commands are not sent to the server, and the drafting rules can be used by another command.
- verify: cargo test --lib remote::forward:: && cargo test --test cmd_review_task
- verified: 2026-11-24
- commit: 296a9bf
- reqs: R-DRAFT-7, D44
- by: Reynan Henry Tandoyo
- note (2026-11-23, planner): added
- note (2026-11-24, haiku-T-382): done

### T-383 · review-session: the material of a whole session
- status: done
- owner: haiku-T-383
- priority: P1 · difficulty: 1 · type: feature
- milestone: T-337
- why: Capture only becomes a habit when the routine steps happen by themselves.
- summary: After a session, the LLM gets the material to propose lessons from the whole conversation.
- depends: T-381, T-382, T-336
- files: src/cmd/review_session.rs, tests/cmd_review_session.rs
- read: docs/design.md, docs/format.md, docs/requirements.md
- steps:
  1. Args: `pub file: std::path::PathBuf`, `--claude`, `--limit <characters>` (default 20000). run needs no dossier and no name: do not call Ctx::load.
  2. Read the file: missing is Code::NotFound naming it; bytes that are not UTF-8 are read lossily. The turns are claude::turns(&text) with --claude, else session::parse(&text)?. Then session::relevant(&turns, limit).
  3. Output: `# Review of the session`, `<n> of <m> turns`, an empty line, the relevant turns separated by empty lines, then `## Drafting rules` with review_task::RULES numbered `1. …` as review-task prints them. No relevant turn: the single line `nothing in this session looks like a decision or a result`. JSON data {turns: m, kept: n, material: [..], rules: [..]}.
  4. tests/cmd_review_session.rs (new): a session file written by the test (the example of docs/format.md section 34 plus one turn without a cue) gives the material with the cue turns and the rules; a Claude Code transcript of three JSON lines gives the same material with --claude; a session file with `<!-- dossier-session: 2 -->` is exit 4; an empty file gives the single line; a missing file is exit 1; --limit 50 keeps only the newest turn; in a folder that has no dossier (a fresh tempfile::tempdir, the program run there with assert_cmd) the command still works.
  5. Run cargo test --test cmd_review_session
- accept: review-session prints the parts of a session where something was decided or a result was discussed, and the drafting rules, cut to a size limit.
- verify: cargo test --test cmd_review_session
- verified: 2026-11-24
- commit: fd90951
- reqs: R-DRAFT-7, D7, D44
- by: Reynan Henry Tandoyo
- note (2026-11-23, planner): added
- note (2026-11-24, haiku-T-383): done

### T-384 · session convert --claude: make a session file from a Claude Code transcript
- status: done
- owner: haiku-T-384
- priority: P1 · difficulty: 1 · type: feature
- milestone: T-337
- why: Capture only becomes a habit when the routine steps happen by themselves.
- summary: The one place where Claude Code's transcript is turned into dossier's own format.
- depends: T-381, T-382, T-336
- files: src/cmd/session.rs, tests/cmd_session.rs
- read: docs/design.md, docs/format.md, docs/requirements.md
- steps:
  1. Args: one subcommand `Convert { claude: Option<PathBuf> (--claude <transcript>), to: Option<PathBuf> (--to <file>) }`. Without --claude: Code::Usage `name the tool the transcript comes from: --claude <transcript>`. The command needs no dossier and no name.
  2. Read the transcript (missing: Code::NotFound; not UTF-8: lossily). text = session::render("claude code", &claude::turns(&transcript)).
  3. Without --to: the text is the Output. With --to: write it with crate::atomic::replace(path, bytes, None) and print `<n> turns written to <file>`. When no turn could be read, add the warning `no turn could be read from <file>; the transcript format may have changed`. JSON data {turns: n, file}.
  4. tests/cmd_session.rs (new): a transcript of three JSON lines converts to a text that starts with `<!-- dossier-session: 1 -->` and holds both turns; --to writes the file and `review-session <that file>` reads it; a transcript with no readable line gives the warning and exit 0; `session convert` without --claude is exit 2.
  5. Run cargo test --test cmd_session
- accept: A Claude Code transcript is converted to dossier's own session file, and a transcript that can't be read gives a warning and an empty session.
- verify: cargo test --test cmd_session
- verified: 2026-11-24
- commit: 744c635
- reqs: R-DRAFT-7, D44
- by: Reynan Henry Tandoyo
- note (2026-11-23, planner): added
- note (2026-11-24, haiku-T-384): done

### T-338 · hooks.rs: the SessionEnd hook for the session review
- status: done
- owner: haiku-T-338
- priority: P1 · difficulty: 1 · type: feature
- milestone: T-057
- why: Capture only becomes a habit when the routine steps happen by themselves.
- summary: The settings entry that lets the session review start by itself.
- depends: T-336
- files: src/hooks.rs
- read: docs/design.md, docs/format.md, docs/requirements.md
- steps:
  1. Add SESSION_END and review_installed as in docs/design.md section 19: it adds, once, a SessionEnd entry without a matcher whose command is `dossier session-end --run '<command>'` (a single quote inside the command is written as '\''); an existing SessionEnd entry whose command starts with SESSION_END is replaced, so that a new command takes the place of the old one. Other keys are kept.
  2. `is_dossier` also accepts a command that starts with SESSION_END, so that `uninstalled` removes the entry.
  3. Unit tests: installing into an empty object, twice with the same command (one entry), again with another command (still one entry, the new command), next to other SessionEnd hooks (kept); uninstalling removes only the dossier entry.
  4. Run cargo test --lib hooks::
- accept: The settings of Claude Code can get a hook that starts the session review when a session ends, and uninstall removes it.
- verify: cargo test --lib hooks::
- verified: 2026-11-24
- commit: 5346b2b
- reqs: R-DRAFT-9
- by: Reynan Henry Tandoyo
- note (2026-11-23, planner): added
- note (2026-11-24, haiku-T-338): done

### T-339 · session-end and hooks install --session-review: start the team's review command when a session ends
- status: open
- owner: —
- priority: P1 · type: milestone
- milestone: T-057
- goal: With the hook installed, the end of a session prepares the review material and starts the team's own command on it.
- why: Capture only becomes a habit when the routine steps happen by themselves.
- summary: The routine step after a session happens without anyone starting it.
- depends: T-337, T-338
- accept: With the hook installed, the end of a session prepares the review material and starts the team's own command on it, so that drafts are waiting the next morning.
- reqs: R-DRAFT-9, D7, D44
- note (2026-11-23, planner): added
- note (2026-11-23, planner): split into T-385, T-386, T-387, T-388

### T-385 · session-end: the command file and its place in the command list
- status: done
- owner: haiku-T-385
- priority: P0 · difficulty: 1 · type: chore
- milestone: T-339
- why: Capture only becomes a habit when the routine steps happen by themselves.
- summary: An empty place for the command behind the session-end hook.
- files: src/cmd/session_end.rs, src/cmd/mod.rs, src/cli.rs
- read: docs/design.md, docs/format.md, docs/requirements.md
- steps:
  1. Create src/cmd/session_end.rs with `#[derive(clap::Args, Debug)] pub struct Args { #[arg(long)] pub run: String }` and a `run` function that returns Ok(Output::text("")), with the `use` lines of src/cmd/guard.rs.
  2. src/cmd/mod.rs: `pub mod session_end;`. src/cli.rs: `SessionEnd(cmd::session_end::Args),` in the Command enum and the arm with the name "session-end" in the match of main_exit.
  3. Run cargo build and cargo test --test smoke
- accept: `dossier session-end --run x` exists and prints nothing.
- verify: cargo build && cargo test --test smoke
- verified: 2026-11-24
- commit: d00d572
- reqs: R-DRAFT-9
- by: Reynan Henry Tandoyo
- note (2026-11-23, planner): added
- note (2026-11-24, haiku-T-385): done

### T-386 · session-end: in the help overview, and run on the PC
- status: done
- owner: haiku-T-386
- priority: P1 · difficulty: 1 · type: chore
- milestone: T-339
- why: Capture only becomes a habit when the routine steps happen by themselves.
- summary: The hook command is listed and stays on the PC.
- depends: T-385
- files: src/cmd/help.rs, src/remote/forward.rs, tests/cmd_agents.rs
- read: docs/design.md, docs/format.md, docs/requirements.md
- steps:
  1. src/cmd/help.rs: ("session-end", "Starts the session review") in COMMANDS, length raised by one.
  2. src/remote/forward.rs: "session-end" in LOCAL_COMMANDS, length raised by one.
  3. tests/cmd_agents.rs: help_without_a_topic_lists_every_command_in_order compares the whole list that `dossier help` prints; when the new commands now appear in it, add them to the expected list at the place where `dossier help` prints them, and change nothing else in that file.
  4. Run cargo test --lib remote::forward:: and cargo test --test smoke
- accept: The overview lists session-end, and it runs on the PC.
- verify: cargo test --lib remote::forward:: && cargo test --test smoke
- verified: 2026-11-24
- commit: 21a1cf5
- reqs: R-DRAFT-9
- by: Reynan Henry Tandoyo
- note (2026-11-23, planner): added
- note (2026-11-23, planner): Added tests/cmd_agents.rs: its help overview test lists every command (found in T-359).
- note (2026-11-24, haiku-T-386): done

### T-387 · session-end: write the material and start the review command
- status: done
- owner: haiku-T-387
- priority: P1 · difficulty: 1 · type: feature
- milestone: T-339
- why: Capture only becomes a habit when the routine steps happen by themselves.
- summary: The routine step after a session happens without anyone starting it.
- depends: T-385, T-337
- files: src/cmd/session_end.rs, tests/cmd_session_end.rs
- read: docs/design.md, docs/format.md, docs/requirements.md
- steps:
  1. run never fails: every error ends as Ok(Output::text("")). Read the payload from standard input, parse it as JSON, take the string `transcript_path`; anything missing: nothing to do.
  2. Get the material by running the program itself (std::env::current_exe) with ["review-session", "--claude", <transcript_path>], standard input null, output captured. Not exit 0, or the output starts with `nothing in this session`: nothing to do.
  3. Write the material to `.claude/dossier-session-review.md` in the current folder (create `.claude` when it is missing; replace an older file).
  4. Start the review command without waiting for it, with standard input, output and error null, and do not call wait on the child. On Windows: program `cmd`, arguments `/C`, then the text `<command> "<path>"`. Elsewhere: program `sh`, arguments `-c`, `<command> "$1"`, `sh`, `<path>`. The path is the full path of the file of step 3. Return at once: Claude Code gives this hook about 1.5 seconds.
  5. tests/cmd_session_end.rs (new): a payload naming a transcript with a cue turn writes .claude/dossier-session-review.md containing `# Review of the session`; a review command that copies its last argument's file to marker.md is started (poll up to five seconds for marker.md); a payload without transcript_path, a transcript without a cue turn, and text that is not JSON: exit 0 and no file is written.
  6. Run cargo test --test cmd_session_end
- accept: When a session ends, the review material is written to a file and the team's command is started on it, without holding up Claude Code.
- verify: cargo test --test cmd_session_end
- verified: 2026-11-24
- commit: e91a8a6
- reqs: R-DRAFT-9, D7
- by: Reynan Henry Tandoyo
- note (2026-11-23, planner): added
- note (2026-11-24, haiku-T-387): done

### T-388 · hooks install --session-review: add the session-end hook
- status: done
- owner: haiku-T-388
- priority: P1 · difficulty: 1 · type: feature
- milestone: T-339
- why: Capture only becomes a habit when the routine steps happen by themselves.
- summary: One option switches the automatic session review on.
- depends: T-338
- files: src/cmd/hooks.rs, tests/cmd_hooks.rs
- read: docs/design.md, docs/format.md, docs/requirements.md
- steps:
  1. Action::Install gains `#[arg(long)] session_review: Option<String>`. When it is given, the new settings text also goes through hooks::review_installed(text, command), after hooks::installed.
  2. The listing of what changes shows the new line through the existing hook_lines: `+ SessionEnd: dossier session-end --run '<command>'`. Nothing else changes: without the option the output is as before.
  3. tests/cmd_hooks.rs: `hooks install --yes --session-review "my-review"` writes a SessionEnd entry whose command is `dossier session-end --run 'my-review'`; a second install with another command leaves one SessionEnd entry with the new command; `hooks uninstall --yes` removes it; install without the option adds no SessionEnd entry.
  4. Run cargo test --test cmd_hooks
- accept: hooks install --session-review adds the hook that starts the session review, and uninstall removes it.
- verify: cargo test --test cmd_hooks
- verified: 2026-11-24
- commit: 47aea05
- reqs: R-DRAFT-9
- by: Reynan Henry Tandoyo
- note (2026-11-23, planner): added
- note (2026-11-24, haiku-T-388): done

### T-340 · README: the session review
- status: open
- owner: —
- priority: P1 · difficulty: 1 · type: docs
- milestone: T-057
- why: Capture only becomes a habit when the routine steps happen by themselves.
- summary: Team members can read how to let lessons be drafted after each session.
- depends: T-339
- files: README.md
- read: docs/design.md, docs/format.md, docs/requirements.md
- steps:
  1. Add a section on the session review under the newest version's additions: `dossier review-session <session file>` and what it prints; the session file format in five lines with the example of docs/format.md section 34, and that any tool or LLM can write one; `--claude` and `dossier session convert --claude <transcript>` for Claude Code; `dossier hooks install --session-review "<command>"` with one example command (`claude -p "Read the file given as the last argument and propose lessons with dossier learn"`); that dossier only starts the command and never calls a model itself; that the material is in .claude/dossier-session-review.md; and that Claude Code's own transcript format may change, in which case only the converter is updated and a warning says that no turn could be read.
  2. README.md, the session review part: say that the review command given to `hooks install --session-review` runs on Windows in cmd and elsewhere in sh, so it is written in that shell's syntax (on Windows double quotes, not single quotes), and that dossier adds the path of the material as its last argument and does not wait for it.
  3. Run cargo test --test smoke
- accept: The README explains the session review and how to switch it on.
- verify: cargo test --test smoke
- reqs: R-DRAFT-7, R-DRAFT-9, REL-2
- note (2026-11-23, planner): added

### T-058 · Extras: story, search --similar, brief --budget, check --fix, export, schema, unit warnings, team summary, readers' guide; release 0.6.0
- status: open
- owner: —
- priority: P3 · type: milestone
- milestone: T-055
- goal: R-READ-7, R-READ-9, R-CTX-6, R-CHECK-2, R-OUT-5 to R-OUT-7, R-ENTRY-11, R-TEAM-5 and REL-4 hold; version 0.6.0 is ready to tag. R-CTX-8 is not built: it needs a change in todo, and R-AGENT-5 gives the same result for Claude Code.
- why: Smaller comforts that are worth having once the daily use is settled.
- summary: Smaller comforts: the story of decisions, similar entries, a longer brief, repairs, export to Markdown and more.
- depends: T-054
- accept: R-READ-7, R-READ-9, R-CTX-6, R-CTX-8, R-CHECK-2, R-OUT-5 to R-OUT-7, R-ENTRY-11, R-TEAM-5 and REL-4 hold; version 0.6.0 is released.
- reqs: R-READ-7, R-READ-9, R-CTX-6, R-CTX-8, R-CHECK-2, R-OUT-5 to R-OUT-7, R-ENTRY-11, R-TEAM-5, REL-4
- note (2026-11-20, planner): added
- note (2026-11-21, planner): Owner 2026-11-21 (D33): export writes Markdown only.
- note (2026-11-23, planner): split into T-341, T-342, T-343, T-344, T-345, T-346, T-347, T-348, T-349, T-350, T-351, T-352, T-353, T-354, T-355

### T-341 · v0.6 skeleton: stubs for story, export, schema and examples.rs
- status: done
- owner: planner
- priority: P0 · type: milestone
- milestone: T-058
- goal: The commands story, export and schema and the module examples.rs exist as stubs that compile.
- why: Smaller comforts that are worth having once the daily use is settled.
- summary: Empty places for the new commands, so that each can be built on its own.
- accept: The three commands and the examples module exist as stubs that compile.
- reqs: R-READ-7, R-OUT-6, R-OUT-7, R-OUT-5
- by: Reynan Henry Tandoyo
- note (2026-11-23, planner): added
- note (2026-11-23, planner): split into T-389, T-390, T-391
- note (2026-11-24, planner): done — story, export and schema exist as commands that answer 'not built yet', with examples.rs as a stub.

### T-389 · Stub files: story, export and schema
- status: done
- owner: haiku-T-389
- priority: P0 · difficulty: 1 · type: chore
- milestone: T-341
- why: Smaller comforts that are worth having once the daily use is settled.
- summary: Empty files for three new commands.
- files: src/cmd/story.rs, src/cmd/export.rs, src/cmd/schema.rs
- read: docs/design.md, docs/format.md, docs/requirements.md
- steps:
  1. Create the three files, each with `#[derive(clap::Args, Debug)] pub struct Args {}` and `pub fn run(_args: &Args, _g: &Global) -> Result<Output>` returning Err(Error::new(Code::NotImplemented, "<command> is not built yet")), with the same `use` lines as src/cmd/guard.rs.
  2. Check them with rustfmt --check --edition 2024 src/cmd/story.rs src/cmd/export.rs src/cmd/schema.rs
  3. Run cargo build (it must still pass).
- accept: The three stub files exist and are formatted.
- verify: rustfmt --check --edition 2024 src/cmd/story.rs src/cmd/export.rs src/cmd/schema.rs && cargo build
- verified: 2026-11-23
- commit: fe9890f
- reqs: R-READ-7, R-OUT-6, R-OUT-7
- by: Reynan Henry Tandoyo
- note (2026-11-23, planner): added
- note (2026-11-23, haiku-T-389): done

### T-390 · Stub module: examples.rs
- status: done
- owner: haiku-T-390
- priority: P0 · difficulty: 1 · type: chore
- milestone: T-341
- why: Smaller comforts that are worth having once the daily use is settled.
- summary: An empty place for the examples of each command.
- depends: T-389
- files: src/examples.rs, src/lib.rs, src/cmd/mod.rs
- read: docs/design.md, docs/format.md, docs/requirements.md
- steps:
  1. Create src/examples.rs with `pub fn of(_command: &str) -> Option<[&'static str; 3]> { None }` and the doc comment of docs/design.md section 20.
  2. src/lib.rs: `pub mod examples;` at its alphabetical place.
  3. src/cmd/mod.rs: `pub mod export;`, `pub mod schema;`, `pub mod story;` at their alphabetical places. These modules exist from this task on, so they are compiled and checked; the register task adds them to cli.rs and help.rs.
  4. Run cargo build
- accept: The module exists with its final signature and compiles.
- verify: cargo build
- verified: 2026-11-24
- commit: f1a55ef
- reqs: R-OUT-5
- by: Reynan Henry Tandoyo
- note (2026-11-23, planner): added
- note (2026-11-24, haiku-T-390): done

### T-391 · Register story, export and schema
- status: done
- owner: haiku-T-391
- priority: P0 · difficulty: 1 · type: chore
- milestone: T-341
- why: Smaller comforts that are worth having once the daily use is settled.
- summary: The three new commands become part of the program.
- depends: T-389, T-390
- files: src/cli.rs, src/cmd/help.rs, tests/cmd_agents.rs
- read: docs/design.md, docs/format.md, docs/requirements.md
- steps:
  1. src/cli.rs: `Story(cmd::story::Args)`, `Export(cmd::export::Args)`, `Schema(cmd::schema::Args)` in the Command enum and three arms in the match of main_exit with the names "story", "export", "schema".
  2. src/cmd/help.rs: ("story", "Decisions in the order made"), ("export", "Writes the project as Markdown"), ("schema", "Describes commands and JSON") in COMMANDS, length raised by three; make the COMMANDS constant `pub`.
  3. tests/cmd_agents.rs: help_without_a_topic_lists_every_command_in_order compares the whole list that `dossier help` prints; when the new commands now appear in it, add them to the expected list at the place where `dossier help` prints them, and change nothing else in that file.
  4. Run cargo build and cargo test --test smoke
- accept: The three commands are part of the program and the overview, and the command list can be read by other modules.
- verify: cargo build && cargo test --test smoke
- verified: 2026-11-24
- commit: 0e4ff1d
- reqs: R-READ-7, R-OUT-6, R-OUT-7
- by: Reynan Henry Tandoyo
- note (2026-11-23, planner): added
- note (2026-11-23, planner): Added tests/cmd_agents.rs: its help overview test lists every command (found in T-359).
- note (2026-11-23, planner): src/cmd/mod.rs moved to T-390, so this task stays at 3 files (level 1).
- note (2026-11-24, haiku-T-391): done

### T-342 · story: the decisions in the order they were made
- status: done
- owner: haiku-T-342
- priority: P1 · difficulty: 1 · type: feature
- milestone: T-058
- why: Smaller comforts that are worth having once the daily use is settled.
- summary: Someone who opens the project after two years reads its decisions as one story.
- depends: T-341
- files: src/cmd/story.rs, tests/cmd_story.rs
- read: docs/design.md, docs/format.md, docs/requirements.md
- steps:
  1. Args: `--milestone <task>`. run: Ctx::load(g, "story"); store::all; keep kind Decision with status approved or superseded, oldest `created` first, then by ID. With --milestone: keep those whose `task` is the milestone or one of the IDs of todo_bridge::json(&ctx.root, &["list", "--milestone", ms, "--status", "all"]) `tasks[]`; todo missing: Code::TodoMissing `story --milestone needs todo`.
  2. Output per decision ({D}): `<date>  <ID> · <title>`, with ` [superseded by <ID>]` when it is; then the Reason section indented by two spaces; then `  task: <task>` when it has one; an empty line between decisions. None: `no decisions yet`. JSON data {decisions: [{id, title, date, reason, task, superseded_by}]}.
  3. tests/cmd_story.rs (new): three decisions made at three times (Project::run_at) come out oldest first with their reasons; a superseded one is marked; a draft proposal is not listed; --milestone with a todo stand-in keeps only the decisions of its tasks.
  4. Run cargo test --test cmd_story
- accept: story answers why the project was done this way: every decision in order, with its reason and its task.
- verify: cargo test --test cmd_story
- verified: 2026-11-24
- commit: 23ee2d2
- reqs: R-READ-7
- by: Reynan Henry Tandoyo
- note (2026-11-23, planner): added
- note (2026-11-24, haiku-T-342): done

### T-343 · export: the project's knowledge as one Markdown document
- status: done
- owner: haiku-T-343
- priority: P1 · difficulty: 1 · type: feature
- milestone: T-058
- why: Smaller comforts that are worth having once the daily use is settled.
- summary: The project's knowledge can be handed over or put into a report as one document.
- depends: T-341
- files: src/cmd/export.rs, tests/cmd_export.rs
- read: docs/design.md, docs/format.md, docs/requirements.md
- steps:
  1. Args: `--milestone <task>`, `--to <file>`. run: Ctx::load(g, "export"); the approved entries of kind Decision, Finding, Investigation and Usage, oldest `created` first; with --milestone limited as `story` does it (the same todo call).
  2. The document: `# <project code>: decisions, findings and runs`, a line `exported <date>`, then per entry `## <ID> · <title>`, a line `<kind>, <date>, <author>[, task <task>]`, and its sections as `### <name>` with their text. For a usage record the line also names software and version.
  3. Without --to the document is the Output text. With --to: write it to that file with crate::atomic::replace (the file is outside the dossier; an existing file is replaced) and print `exported <n> entries to <file>`. JSON data {entries: n, file}. The command changes nothing in the dossier and needs no name.
  4. tests/cmd_export.rs (new): a decision, a finding and a usage record appear in time order with their sections; a draft lesson does not; --to writes the file and prints the count; the dossier's journal is unchanged afterwards.
  5. Run cargo test --test cmd_export
- accept: export writes the decisions, findings, investigations and usage records in order as one Markdown document, the basis for a report or a handover.
- verify: cargo test --test cmd_export
- verified: 2026-11-24
- commit: 569c96a
- reqs: R-OUT-7, D33
- by: Reynan Henry Tandoyo
- note (2026-11-23, planner): added
- note (2026-11-24, haiku-T-343): done

### T-344 · schema: a machine-readable description of the commands
- status: done
- owner: haiku-T-344
- priority: P1 · difficulty: 1 · type: feature
- milestone: T-058
- why: Smaller comforts that are worth having once the daily use is settled.
- summary: A program or an LLM can ask the tool which commands and answers exist.
- depends: T-341
- files: src/cmd/schema.rs, tests/cmd_schema.rs
- read: docs/design.md, docs/format.md, docs/requirements.md
- steps:
  1. run needs no dossier. Build one JSON value: `commands`: for each entry of cmd::help::COMMANDS {name, description, schema: `dossier.<name>.v1`}; `envelope`: {ok: `bool`, data: `object`, warnings: `array of strings`, error: {code: `string`, message: `string`}}; `exit_codes`: {"0": "done", "1": "refused by a rule, or problems found", "2": "wrong usage", "3": "busy, try again", "4": "file or settings problem", "5": "a needed program or connection failed"}; `error_codes`: every variant of error::Code with its name (Error::new(code, "").name()) and its exit code.
  2. Output: the JSON pretty-printed as text, and the same value as data, so that `dossier schema --json` wraps it in the envelope.
  3. tests/cmd_schema.rs (new): the output parses as JSON; `commands` holds `brief` with schema `dossier.brief.v1` and has as many entries as the help overview has lines; `error_codes` holds NO_DOSSIER with exit 4.
  4. Run cargo test --test cmd_schema
- accept: schema prints every command, the JSON envelope, the exit codes and the error codes as data.
- verify: cargo test --test cmd_schema
- verified: 2026-11-24
- commit: 51e617e
- reqs: R-OUT-6
- by: Reynan Henry Tandoyo
- note (2026-11-23, planner): added
- note (2026-11-24, haiku-T-344): done

### T-345 · search --similar: entries that share the most with one entry
- status: done
- owner: haiku-T-345
- priority: P1 · difficulty: 1 · type: feature
- milestone: T-058
- why: Smaller comforts that are worth having once the daily use is settled.
- summary: From one lesson, the reader finds the related ones.
- depends: T-341
- files: src/cmd/search.rs, tests/cmd_search.rs
- read: docs/design.md, docs/format.md, docs/requirements.md
- steps:
  1. Args: `--similar <ID>`; `words` is no longer required when it is given (clap `required_unless_present = "similar"`), and both together are Code::Usage.
  2. With --similar: store::get the entry; Query { tags: its machine, analysis, software and tags values, words: select::words(its title) }; over the entries of the project and the team that pass the filter options, without the entry itself: select::best, limit 10. Print them as search prints results, the matched line being the title; then the count line. JSON as for search.
  3. tests/cmd_search.rs: of three lessons, the one sharing a tag and a title word with LES-001 comes first, one sharing nothing is not listed, LES-001 itself is not listed; --similar with words is exit 2; an unknown ID is exit 1.
  4. Run cargo test --test cmd_search
- accept: search --similar lists the entries that share the most tags and title words with a given entry.
- verify: cargo test --test cmd_search
- verified: 2026-11-24
- commit: d730aba
- reqs: R-READ-9
- by: Reynan Henry Tandoyo
- note (2026-11-23, planner): added
- note (2026-11-24, haiku-T-345): done

### T-346 · brief --budget: another line limit for one call
- status: done
- owner: haiku-T-346
- priority: P1 · difficulty: 1 · type: feature
- milestone: T-058
- why: Smaller comforts that are worth having once the daily use is settled.
- summary: An LLM with a large context can ask for a longer brief, one with a small context for a shorter one.
- depends: T-341
- files: src/cmd/brief.rs, tests/cmd_brief.rs
- read: docs/design.md, docs/format.md, docs/requirements.md
- steps:
  1. Args gain `--budget <lines>` (a number of at least 5; below: Code::Usage). Where for_project and for_task take the limit from ctx.cfg.brief_lines, use the budget when it is given.
  2. tests/cmd_brief.rs: with 30 decisions, `brief --budget 10` prints at most 10 lines and ends with the left-out line; `brief --budget 200` prints them all in full; `brief --task T-001 --budget 8` prints at most 8 lines; --budget 2 is exit 2.
  3. Run cargo test --test cmd_brief
- accept: The limit of brief can be changed for one call.
- verify: cargo test --test cmd_brief
- verified: 2026-11-24
- commit: acc6cf4
- reqs: R-CTX-6
- by: Reynan Henry Tandoyo
- note (2026-11-23, planner): added
- note (2026-11-24, haiku-T-346): done

### T-347 · brief --task --json: why each entry was chosen
- status: open
- owner: —
- priority: P1 · type: milestone
- milestone: T-058
- goal: brief --task --json says for every entry why it was chosen: linked, milestone, tag match or word match.
- why: Smaller comforts that are worth having once the daily use is settled.
- summary: A person can check why the tool showed an entry for a task.
- depends: T-346
- accept: brief --task --json says for every entry why it was chosen: linked, milestone, tag match or word match.
- reqs: R-CTX-6, TEST-9
- note (2026-11-23, planner): added
- note (2026-11-23, planner): split into T-392, T-393, T-394

### T-392 · select.rs: why an entry matched
- status: done
- owner: haiku-T-392
- priority: P0 · difficulty: 1 · type: feature
- milestone: T-347
- why: Smaller comforts that are worth having once the daily use is settled.
- summary: The tool can say which rule made it show an entry.
- files: src/select.rs
- read: docs/design.md, docs/format.md, docs/requirements.md
- steps:
  1. Add `pub fn reason(item: &Item, query: &Query) -> Option<&'static str>` with the doc comment of docs/design.md section 20: Some("tag match") when a query tag is among the entry's tag fields (the same comparison `score` uses), else Some("word match") when a query word is in the title or the sections, else None.
  2. Unit tests next to those of `score`: an entry sharing a tag gives `tag match` (also when a word matches too); one sharing only a title word gives `word match`; one sharing nothing gives None, and score is 0 for it.
  3. Run cargo test --lib select::
- accept: select::reason tells whether an entry was chosen for a tag or for a word.
- verify: cargo test --lib select::
- verified: 2026-11-24
- commit: 48736c4
- reqs: R-CTX-6
- by: Reynan Henry Tandoyo
- note (2026-11-23, planner): added
- note (2026-11-24, haiku-T-392): done

### T-393 · brief_task.rs: the entries chosen for a task, each with its reason
- status: claimed
- owner: haiku-T-393
- claimed: 2026-11-24T00:56+01:00
- session: 04041e34-c8d2-4889-bbd5-daa3a4d9a285
- priority: P1 · difficulty: 1 · type: feature
- milestone: T-347
- why: Smaller comforts that are worth having once the daily use is settled.
- summary: The reasons behind the selection of brief --task, as data.
- depends: T-392
- files: src/brief_task.rs
- read: docs/design.md, docs/format.md, docs/requirements.md
- steps:
  1. Add `pub fn chosen(ctx: &Ctx, task: &str) -> Result<Vec<(String, &'static str)>>`. Do not change `lines` or any function it calls.
  2. It reads the same data `lines` reads at its start (the todo task with About::read, the approved entries of the project and, when there is one, the team's) and returns, in this order and each ID once: the entries for which task_is(item, task) holds, with `linked`; the decisions whose task is one of milestone_ids, with `milestone`; then, over the remaining entries, select::best with the same Query `lines` builds (the task's tags and the words of its title) and limit DEAD_ENDS_LIMIT + RELATED_LIMIT, each with select::reason (entries for which reason is None are left out).
  3. Unit test at the end of the file, in the style of the existing ones: a linked finding has `linked`, a milestone decision `milestone`, a lesson sharing only a title word `word match`, one sharing a tag `tag match`; an entry sharing nothing is absent; calling it twice gives the same list; every ID it returns appears in the text of `lines` for the same task.
  4. Run cargo test --lib brief_task::
- accept: For a task, the program can list the entries it would show and the reason for each, in a fixed order.
- verify: cargo test --lib brief_task::
- reqs: R-CTX-6, TEST-9
- note (2026-11-23, planner): added

### T-394 · brief --task --json: the reasons in the data
- status: open
- owner: —
- priority: P1 · difficulty: 1 · type: feature
- milestone: T-347
- why: Smaller comforts that are worth having once the daily use is settled.
- summary: A person can check why the tool showed an entry for a task.
- depends: T-393, T-346
- files: src/cmd/brief.rs, tests/cmd_brief.rs
- read: docs/design.md, docs/format.md, docs/requirements.md
- steps:
  1. In for_task: the JSON data gains `items`: one object {id, reason} per pair of brief_task::chosen(ctx, task)?. The text output does not change.
  2. tests/cmd_brief.rs: `brief --task T-001 --json` with a linked finding and a lesson that shares a title word has both in data.items with `linked` and `word match`.
  3. Run cargo test --test cmd_brief
- accept: brief --task --json carries, for each entry, the reason it was chosen.
- verify: cargo test --test cmd_brief
- reqs: R-CTX-6
- note (2026-11-23, planner): added

### T-348 · check --fix: repair what is safe
- status: open
- owner: —
- priority: P1 · type: milestone
- milestone: T-058
- goal: check --fix repairs file names, missing format lines and the order of the tool's own fields, and nothing else.
- why: Smaller comforts that are worth having once the daily use is settled.
- summary: Small damage from editing files by hand is repaired by one command.
- depends: T-341
- accept: check --fix repairs file names, missing format lines and the order of the tool's fields, and nothing else.
- reqs: R-CHECK-2, R-EDIT-1
- note (2026-11-23, planner): added
- note (2026-11-23, planner): split into T-395, T-396, T-397

### T-395 · check --fix: rename files whose name does not match
- status: done
- owner: haiku-T-395
- priority: P1 · difficulty: 1 · type: feature
- milestone: T-348
- why: Smaller comforts that are worth having once the daily use is settled.
- summary: A file renamed by mistake is put right by one command.
- depends: T-341
- files: src/cmd/check.rs, tests/cmd_check.rs, src/cmd/unpack.rs
- read: docs/design.md, docs/format.md, docs/requirements.md
- steps:
  1. Args gain `#[arg(long)] pub fix: bool`. With --fix, before the checks run, call a new private `fix(ctx: &Ctx) -> Result<Vec<String>>` that does its repairs inside one tx::write (so check --fix needs a name; without --fix nothing changes) and returns one line per repair. Print each as `fixed: <line>` before the normal check output; on a dry run tx reports them as warnings.
  2. First repair: for every entry file that parses and whose ID appears only once, when its path is not store::path_for(dossier, kind, id, title): rename it with tx.rename(&item, new_path, item.entry.text(), Change { id, event: "fixed", detail: "file name" }); line `<old file> renamed to <new file>`. Skip it when the new path exists already.
  3. tests/cmd_check.rs: a file renamed by hand to `FND-001-wrong.md` is renamed back by `check --fix` and the dossier is ok afterwards; with --dry-run nothing changes; `check --fix` without a name is exit 2; plain `check` still only reports.
  4. src/cmd/unpack.rs: the check::Args literal (near line 53, `check::run(&check::Args {}, ...)`) gains `fix: false`; nothing else in that file changes.
  5. Run cargo test --test cmd_check
- accept: check --fix gives every entry file the name that its ID and title ask for.
- verify: cargo test --test cmd_check
- verified: 2026-11-24
- commit: e341509
- reqs: R-CHECK-2
- by: Reynan Henry Tandoyo
- note (2026-11-23, planner): added
- note (2026-11-23, planner): src/cmd/unpack.rs builds check::Args, so adding --fix needs fix: false there (found before the phase started).
- note (2026-11-24, haiku-T-395): done

### T-396 · check --fix: add a missing format line
- status: done
- owner: haiku-T-396
- priority: P1 · difficulty: 1 · type: feature
- milestone: T-348
- why: Smaller comforts that are worth having once the daily use is settled.
- summary: An entry saved without its first line is repaired.
- depends: T-395
- files: src/cmd/check.rs, tests/cmd_check.rs
- read: docs/design.md, docs/format.md, docs/requirements.md
- steps:
  1. In `fix`: for every entry file that parses and whose first line (after an optional byte-order mark) is not `<!-- dossier-format: 1 -->`: put that line in front, ending as the line that follows it ends (LF or CRLF), keeping a byte-order mark at the very start; tx.put with Change { id, event: "fixed", detail: "format line" }; line `<file>: format line added`. When the same file is also renamed, do both in the one tx.rename.
  2. tests/cmd_check.rs: an entry whose format line was removed by hand gets it back and every other byte is the same as before the removal; an entry saved with CRLF endings gets a CRLF format line.
  3. Run cargo test --test cmd_check
- accept: check --fix adds the format line to an entry that lost it, and keeps every other byte.
- verify: cargo test --test cmd_check
- verified: 2026-11-24
- commit: 260b61d
- reqs: R-CHECK-2, R-EDIT-1
- by: Reynan Henry Tandoyo
- note (2026-11-23, planner): added
- note (2026-11-24, haiku-T-396): done

### T-397 · check --fix: put the tool's own fields in order
- status: open
- owner: —
- priority: P1 · difficulty: 1 · type: feature
- milestone: T-348
- why: Smaller comforts that are worth having once the daily use is settled.
- summary: Fields that were moved while editing by hand are put back in their usual order.
- depends: T-396
- files: src/cmd/check.rs, src/entry/new.rs, tests/cmd_check.rs
- read: docs/design.md, docs/format.md, docs/requirements.md
- steps:
  1. src/entry/new.rs: make FIELD_ORDER `pub`.
  2. In `fix`: for every entry file that parses, look at the lines of its field block. Take the positions of the lines whose key is in FIELD_ORDER (the `history` lines and unknown keys are not touched and keep their positions). When these lines are not in the order of FIELD_ORDER, sort them by their index in FIELD_ORDER and write them back into the same positions, each line byte for byte as it was. tx.put with detail `field order`; line `<file>: fields put in order`.
  3. tests/cmd_check.rs: an entry whose `created` line was moved by hand below `author` is put back in order; an unknown key between two known fields stays at its position; the history lines are unchanged; an entry that is already in order is not written (the journal gets no line for it).
  4. Run cargo test --test cmd_check
- accept: check --fix puts the tool's own field lines back in their order without touching unknown keys or history lines.
- verify: cargo test --test cmd_check
- reqs: R-CHECK-2, R-EDIT-1
- note (2026-11-23, planner): added

### T-349 · check: warn about numbers without a unit
- status: open
- owner: —
- priority: P1 · difficulty: 1 · type: feature
- milestone: T-058
- why: Smaller comforts that are worth having once the daily use is settled.
- summary: A setting written without its unit is noticed before someone reuses it wrongly.
- depends: T-348
- files: src/cmd/check.rs, tests/cmd_check.rs
- read: docs/design.md, docs/format.md, docs/requirements.md
- steps:
  1. Add a function that returns warnings (the `should` list that already holds the TERM reports) of kind `UNIT`: for every approved lesson's Statement section and every approved how-to's Settings section, each line of the form `<name>: <value>` or `<name> = <value>` whose value, trimmed, is only a number (digits with an optional sign, one `.` or `,`, an optional exponent such as `1e-5`). Message: `a number without a unit: "<the line>"; write the unit, or say that it is dimensionless`. The line number is the line in the file.
  2. A warning does not change the exit code, like TERM.
  3. tests/cmd_check.rs: a how-to with `CFL: 5` in Settings gives one UNIT warning and exit 0; `time step: 1e-5 s` and `scheme: second order` give none; the same line in a draft gives none.
  4. Run cargo test --test cmd_check
- accept: check warns about a bare number in the Settings of a how-to or the Statement of a lesson.
- verify: cargo test --test cmd_check
- reqs: R-ENTRY-11
- note (2026-11-23, planner): added

### T-350 · team summary: the state of the team dossier
- status: claimed
- owner: haiku-T-350
- claimed: 2026-11-24T00:55+01:00
- session: 04041e34-c8d2-4889-bbd5-daa3a4d9a285
- priority: P1 · difficulty: 1 · type: feature
- milestone: T-058
- why: Smaller comforts that are worth having once the daily use is settled.
- summary: Team members see what is new in the shared knowledge and what waits for approval.
- depends: T-341
- files: src/cmd/team.rs, tests/cmd_team.rs
- read: docs/design.md, docs/format.md, docs/requirements.md
- steps:
  1. Action gains `Summary`. It works in a team dossier and in a project that has one (ctx.team_ctx()); no team dossier: the existing refusal.
  2. Output, at most 40 lines: `team dossier: <path>`; `## Entries` with one line per kind that has entries: `<kind>: <n> approved, <n> drafts, <n> other`; `## Drafts waiting` (oldest first: `<ID> · <title> (<n> days)`, at most 10, then `<n> more: dossier drafts --team`); `## Due for review` (entries whose `review` date is before today, at most 10); `## Newest` (the ten newest approved entries: `<date>  <ID> · <title>`). An empty part is left out. JSON data {counts, drafts, due, newest}.
  3. tests/cmd_team.rs: a team dossier with two approved lessons and one draft how-to shows the counts, the draft with its age and the newest two; `team` without a subcommand still prints the settings as before.
  4. Run cargo test --test cmd_team
- accept: team summary shows the team dossier at a glance: entries per kind and status, drafts waiting, entries due for review, the newest ten.
- verify: cargo test --test cmd_team
- reqs: R-TEAM-5
- note (2026-11-23, planner): added

### T-351 · help <command>: a description and three examples, first group
- status: claimed
- owner: haiku-T-351
- claimed: 2026-11-24T00:56+01:00
- session: 04041e34-c8d2-4889-bbd5-daa3a4d9a285
- priority: P1 · difficulty: 1 · type: feature
- milestone: T-058
- why: Smaller comforts that are worth having once the daily use is settled.
- summary: A team member on Windows sees how a command is typed in PowerShell.
- depends: T-341
- files: src/examples.rs, src/cmd/help.rs, tests/cmd_help.rs
- read: docs/design.md, docs/format.md, docs/requirements.md
- steps:
  1. src/cmd/help.rs: `help <topic>` accepts, besides `agents`, every name of COMMANDS: it prints `<name>: <description>`, then `Examples (PowerShell):` with two lines and `Example (Linux shell):` with one, from examples::of(name); a command without examples prints only the first line. An unknown topic stays Code::Usage. JSON data {command, description, examples}.
  2. src/examples.rs: `of` returns three real, runnable command lines (two PowerShell, one Linux shell) for these commands: init, where, add, decide, found, learn, assume, propose, accept, decline, confirm, refute, cleared, set, retitle, link, edit, show, list, search, history, diff, restore, log, brief, status. Use invented data only (project P-017, tasks like T-042, an axial compressor; no real names). A PowerShell example with quotes inside a text uses single quotes around it; the Linux one may use --stdin with a here-document where a text has quotes.
  3. tests/cmd_help.rs (new): `help decide` prints the description and three example lines that start with `dossier decide`; `help nosuch` is exit 2; a unit test in examples.rs checks that every command of the group has three non-empty examples that start with `dossier <name>`.
  4. src/cmd/help.rs: the doc comment of COMMANDS still names only design.md sections 5, 9 and 12; make it say that the list holds every command of all phases, in the order of the help overview.
  5. Run cargo test --test cmd_help and cargo test --lib examples::
- accept: help <command> shows what a command does with two PowerShell examples and one for a Linux shell, for the first group of commands.
- verify: cargo test --test cmd_help && cargo test --lib examples::
- reqs: R-OUT-5
- note (2026-11-23, planner): added

### T-352 · help <command>: examples, second group
- status: open
- owner: —
- priority: P1 · difficulty: 1 · type: feature
- milestone: T-058
- why: Smaller comforts that are worth having once the daily use is settled.
- summary: More commands show how they are typed in PowerShell.
- depends: T-351
- files: src/examples.rs
- read: docs/design.md, docs/format.md, docs/requirements.md
- steps:
  1. Add three examples each (two PowerShell, one Linux shell, as in the first group) for: review-task, drafts, approve, reject, supersede, retire, remove, reopen, check, agents, pack, unpack, connect, projects, todo, value, values, attach, detach, reattach, files, which, used, runs, image, received, delivered, exchange, map, hooks, skill.
  2. Extend the unit test to this group.
  3. Run cargo test --lib examples::
- accept: The second group of commands has its three examples.
- verify: cargo test --lib examples::
- reqs: R-OUT-5
- note (2026-11-23, planner): added

### T-353 · help <command>: examples, third group, and none missing
- status: open
- owner: —
- priority: P1 · difficulty: 1 · type: feature
- milestone: T-058
- why: Smaller comforts that are worth having once the daily use is settled.
- summary: No command is left without an example.
- depends: T-352
- files: src/examples.rs
- read: docs/design.md, docs/format.md, docs/requirements.md
- steps:
  1. Add three examples each for every command of cmd::help::COMMANDS that has none yet, except the commands a person never types (guard, after, session-end): among them team, confidential, promote, close, howto, tool, where-used, term, glossary, reviewed, review-folder, review-document, reviewed-document, import, and the commands added since (for example problem, tried, solved, gave-up, take, return, digest, review-session, story, export, schema).
  2. Replace the group test by one that goes through COMMANDS: every command except guard, after and session-end has three non-empty examples starting with `dossier <name>`.
  3. Run cargo test --lib examples::
- accept: Every command a person types has two PowerShell examples and one for a Linux shell.
- verify: cargo test --lib examples::
- reqs: R-OUT-5
- note (2026-11-23, planner): added

### T-354 · A one-page guide for team members who only read and approve
- status: done
- owner: haiku-T-354
- priority: P1 · difficulty: 1 · type: docs
- milestone: T-058
- why: Smaller comforts that are worth having once the daily use is settled.
- summary: Colleagues who only approve drafts and look things up need one page, not the whole README.
- files: docs/guide-for-readers.md, README.md
- read: docs/design.md, docs/format.md, docs/requirements.md
- steps:
  1. Write docs/guide-for-readers.md, one page (at most 60 lines), in plain English for an engineer who never writes entries: what the dossier is in three sentences; `dossier drafts` (what waits for me); `dossier approve <ID>` (what the four answers approve, edit, reject, skip do, and what to look at before approving a lesson: the conditions under Valid for, the evidence, the confidence); `dossier search <words>` and `dossier show <ID>` with the marks a reader meets ([draft], [changed after approval], [due for review], [imported]). One example each, with invented data. No other command.
  2. README.md: add one line with a link to the guide in the section for people who use the tool (near `### For LLMs`).
  3. Run cargo test --test smoke
- accept: A team member who only reads and approves has one page that explains the four commands they need.
- verify: cargo test --test smoke
- verified: 2026-11-23
- commit: 936d50d
- reqs: REL-4
- by: Reynan Henry Tandoyo
- note (2026-11-23, planner): added
- note (2026-11-23, haiku-T-354): done

### T-355 · Release 0.6.0
- status: open
- owner: —
- priority: P1 · difficulty: 1 · type: chore
- milestone: T-058
- why: Smaller comforts that are worth having once the daily use is settled.
- summary: The version number and the README are ready for the owner to tag release 0.6.0.
- depends: T-328, T-342, T-343, T-344, T-345, T-347, T-349, T-350, T-353, T-354, T-335, T-340, T-294, T-309, T-413, T-414, T-416, T-417, T-418, T-419, T-421
- files: Cargo.toml, Cargo.lock, README.md
- read: docs/design.md, docs/format.md, docs/requirements.md
- steps:
  1. Cargo.toml: version = "0.6.0"; run cargo build so that Cargo.lock follows.
  2. README.md: set the status line at the top to `**Status: v0.6 (phase 6)**`; add `### What version 0.6 adds` (digests, session review, the hook after todo commands, story, search --similar, brief --budget and reasons, check --fix and unit warnings, export, schema, team summary, help with examples, the readers' guide) and remove or rewrite the `Not in version …` section so that it names only what is still not built (R-CTX-8: todo brief including the dossier's brief, which needs a change in todo).
  3. Run cargo fmt --all -- --check, cargo clippy --all-targets --all-features -- -D warnings and cargo test --all-features; all three must pass. Do not tag and do not push: the owner does that.
- accept: Version 0.6.0 builds, passes every check, and the README says what it adds.
- verify: cargo fmt --all -- --check && cargo clippy --all-targets --all-features -- -D warnings && cargo test --all-features
- reqs: REL-1, REL-2
- note (2026-11-23, planner): added

### T-063 · Run commands from the PC: connect, the per-user project register, forwarding through ssh, dossier todo
- status: dropped
- owner: —
- priority: P0 · difficulty: 4 · type: feature
- milestone: T-001
- why: The team works on Windows PCs from the first day while the one copy of the dossier is on the Linux server; people and LLMs must not have to know a server name or path.
- depends: 
- files: src/remote/mod.rs, src/remote/register.rs, src/cmd/connect.rs, src/cmd/projects.rs, tests/remote.rs
- read: docs/requirements.md
- accept: connect writes the pointer file and the register entry; projects lists the register and --project <code> works from any folder; every command in a connected folder runs on the server with output and exit code unchanged; quotes, backticks, dollar signs, umlauts and line breaks arrive unchanged and --stdin is passed on; approve, remove and close ask ssh for a terminal; ssh never prompts and fails with exit 5 within the time limit; dossier todo forwards todo the same way; brief and status name the project code and server folder first; tests use a stand-in for ssh.
- verify: cargo test --test remote
- reqs: R-REMOTE-1 to R-REMOTE-3, R-REMOTE-5, R-REMOTE-6, R-REMOTE-9 to R-REMOTE-12, TEST-8
- note (2026-11-20, planner): added
- note (2026-11-21, planner): dropped — Replaced by the detailed v0.1 tasks under T-001 (one file per task, see docs/design.md).

### T-064 · Windows and Linux together: paths, file names, line endings, group access, version match
- status: dropped
- owner: —
- priority: P0 · difficulty: 4 · type: feature
- milestone: T-001
- why: The dossier lives on Linux and is used from Windows; nothing may depend on which system a command was typed on, and one colleague's entry must be writable by another.
- depends: 
- files: src/xos.rs, src/paths.rs, tests/xos.rs
- read: docs/requirements.md, docs/format.md
- accept: Stored paths are relative with forward slashes and a path typed with backslashes stores the same line; created file names are lower-case ASCII, never a Windows reserved name, never differing only in case; CRLF files and a byte-order mark are changed keeping every other byte; times carry their UTC offset; the same commands give byte-identical files, checksums and search order on Windows and Linux in CI; on Linux new files and folders are group-writable and init reports the group; a format version mismatch between PC and server is refused.
- verify: cargo test --test xos
- reqs: R-XOS-1 to R-XOS-4, R-XOS-6 to R-XOS-10, TEST-11
- note (2026-11-20, planner): added
- note (2026-11-21, planner): dropped — Replaced by the detailed v0.1 tasks under T-001 (one file per task, see docs/design.md).

### T-065 · Skeleton: the program builds and every file exists
- status: done
- owner: coordinator
- priority: P0 · type: milestone
- milestone: T-001
- goal: cargo build, fmt, clippy and test pass; every library module and every command of docs/design.md exists as a file that compiles, so that every later task fills in exactly one file.
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-21, coordinator): done

### T-066 · Cargo project with the approved dependencies
- status: done
- owner: haiku-t066
- priority: P0 · difficulty: 1 · type: chore
- milestone: T-065
- why: Nothing can be built or tested before the Cargo project exists.
- files: Cargo.toml, Cargo.lock, src/main.rs
- read: docs/design.md, docs/requirements.md
- steps:
  1. Create Cargo.toml: package name dossier, version 0.1.0, edition 2024, rust-version 1.89, license MIT, publish false; one [[bin]] named dossier at src/main.rs. Do not add a [lib] section; the library skeleton task adds it.
  2. Add dependencies with these features: chrono (default-features false, features clock), clap (default-features false, features std, derive, help, usage, error-context, suggestions), serde (feature derive), serde_json, sha2 (default-features false), toml (default-features false, features std, serde, parse). Use cargo add so current versions are written.
  3. Add dev-dependencies assert_cmd, predicates and tempfile with cargo add --dev.
  4. Add windows-sys for Windows only: cargo add windows-sys --target 'cfg(windows)' --features Win32_Storage_FileSystem.
  5. Add [lints.clippy] with unwrap_used, expect_used and panic set to deny, and [profile.release] with lto true, codegen-units 1, strip true, panic abort.
  6. Create src/main.rs containing only fn main() {}.
  7. Run cargo build and fix any error. Commit Cargo.lock together with the other two files.
- accept: cargo build succeeds; Cargo.toml lists exactly the dependencies of the steps; Cargo.lock is committed.
- verify: cargo build
- verified: 2026-11-21
- commit: fef3ca9
- reqs: D16, NFR-1, NFR-5, NFR-6
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-21, haiku-t066): done

### T-067 · Library skeleton: lib.rs and every library module with its types and signatures
- status: done
- owner: haiku-t067
- priority: P0 · difficulty: 2 · type: chore
- milestone: T-065
- why: With every type and signature in place, each later task fills in one file without touching a shared one.
- final: Mechanical but touches every library file at once; all signatures are copied from docs/design.md.
- depends: T-066
- files: Cargo.toml, clippy.toml, src/lib.rs, src/error.rs, src/output.rs, src/time.rs, src/hash.rs, src/paths.rs, src/find.rs, src/config.rs, src/names.rs, src/lock.rs, src/atomic.rs, src/term.rs, src/ctx.rs, src/entry/mod.rs, src/entry/edit.rs, src/entry/new.rs, src/entry/rules.rs, src/store.rs, src/ids.rs, src/journal.rs, src/snapshot.rs, src/approval.rs, src/tx.rs, src/create.rs, src/filter.rs, src/select.rs, src/todo_bridge.rs, src/brief_task.rs, src/tar.rs, src/remote/mod.rs, src/remote/pointer.rs, src/remote/register.rs, src/remote/ssh.rs, src/remote/forward.rs, src/agents/mod.rs, src/agents/rules.md
- read: docs/design.md
- steps:
  1. Add the [lib] section (name dossier, path src/lib.rs) to Cargo.toml.
  2. Create clippy.toml with three lines: allow-unwrap-in-tests = true, allow-expect-in-tests = true, allow-panic-in-tests = true. Without it the deny lints in Cargo.toml reject unwrap, expect and panic in unit tests and in tests/, which AGENTS.md allows.
  3. Create src/lib.rs with one pub mod line per module of docs/design.md section 2 (cli and cmd are added by the next task).
  4. In src/lib.rs also add, for unit tests only: #[cfg(test)] pub(crate) static ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(()); with the comment: every unit test that sets or removes an environment variable, or changes the current folder, holds this lock for its whole run (ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner())), because unit tests run in parallel threads of one process.
  5. For every library module of design.md section 3, create its file with every type, constant and function signature exactly as written there.
  6. Give every function a body that compiles and does nothing useful: return Err(Error::new(Code::NotImplemented, '<module>::<function>')) where it returns a Result, and a plain default value otherwise (empty String, empty Vec, false, None, 0). Never use todo!, unimplemented!, unwrap, expect or panic.
  7. Implement only error.rs fully enough for the others to compile: the Code enum, Error, Result, Error::new; exit and name may return 1 and an empty string for now.
  8. Make output::emit work in the simplest way: on Ok write the text with a final line break to standard output, on Err write 'error: <message>' to standard error, both with std::io::Write ignoring write errors (no println!); return 0 or the error's exit(). The output task replaces it.
  9. In src/cli.rs is not part of this task: where a signature needs crate::cli::Global (ctx.rs), declare a temporary pub struct Global in ctx.rs with the six fields of design.md section 4 and a comment that the next task moves it.
  10. Create src/agents/rules.md with one line: rules are written by a later task.
  11. Put #![allow(dead_code, unused_variables)] at the top of lib.rs with a comment that the release task removes it.
  12. Run cargo build and cargo clippy --all-targets -- -D warnings and fix every error.
- accept: Every file of design.md section 3 exists with the listed signatures; lib.rs has the test-only ENV_LOCK; emit prints text or error and returns the exit code; clippy.toml allows unwrap, expect and panic in tests; cargo build and cargo clippy --all-targets -- -D warnings succeed.
- verify: cargo clippy --all-targets -- -D warnings
- verified: 2026-11-21
- commit: 20a8bd2
- reqs: D16, NFR-5
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-21, planner): clippy.toml added: the deny lints of T-066 otherwise fail cargo clippy --all-targets on test code (checked in a throwaway crate). It sits here, not in T-066, because a level-1 task may list at most 3 files.
- note (2026-11-21, planner): Simple emit moved from T-068 to T-067 (output.rs is T-067's file; T-068 did not list it and could run beside T-072). Smoke test uses an unknown command for exit 2, because dossier where stops returning 2 once T-109 is built.
- note (2026-11-21, planner): ENV_LOCK added: unit tests of time, names, term, todo_bridge, brief_task, register and forward set environment variables or the current folder and would race in parallel threads.
- note (2026-11-21, haiku-t067): done

### T-068 · Command skeleton: cli.rs, main.rs and a stub file for every command
- status: done
- owner: haiku-t-068
- priority: P0 · difficulty: 2 · type: chore
- milestone: T-065
- why: After this task a command task only fills in its own file; cli.rs never changes again.
- final: Mechanical but touches every command file at once; the layout is fixed in docs/design.md section 4.
- depends: T-067
- files: src/main.rs, src/lib.rs, src/cli.rs, src/ctx.rs, src/cmd/mod.rs, src/cmd/init.rs, src/cmd/where_cmd.rs, src/cmd/people.rs, src/cmd/add.rs, src/cmd/decide.rs, src/cmd/found.rs, src/cmd/learn.rs, src/cmd/assume.rs, src/cmd/set.rs, src/cmd/retitle.rs, src/cmd/link.rs, src/cmd/edit.rs, src/cmd/show.rs, src/cmd/list.rs, src/cmd/search.rs, src/cmd/history.rs, src/cmd/diff.rs, src/cmd/restore.rs, src/cmd/log.rs, src/cmd/review_task.rs, src/cmd/drafts.rs, src/cmd/approve.rs, src/cmd/reject.rs, src/cmd/supersede.rs, src/cmd/retire.rs, src/cmd/remove.rs, src/cmd/reopen.rs, src/cmd/brief.rs, src/cmd/status.rs, src/cmd/check.rs, src/cmd/agents.rs, src/cmd/help.rs, src/cmd/pack.rs, src/cmd/unpack.rs, src/cmd/connect.rs, src/cmd/projects.rs, src/cmd/todo.rs, tests/smoke.rs
- read: docs/design.md
- steps:
  1. For every command of design.md section 5 create src/cmd/<name>.rs (where is where_cmd.rs, review-task is review_task.rs) containing: #[derive(clap::Args, Debug)] pub struct Args {} and pub fn run(_args: &Args, _g: &crate::cli::Global) -> crate::error::Result<crate::output::Output> returning Err(Error::new(Code::NotImplemented, '<command> is not built yet')).
  2. Create src/cmd/mod.rs with one pub mod line per command file.
  3. Create src/cli.rs as design.md section 4 says: Global (move it here from ctx.rs and make ctx.rs use crate::cli::Global), a clap Parser named Cli with name dossier, version, disable_help_subcommand true, the flattened Global and a Command subcommand enum with one variant per command file holding that file's Args; the variant names give the command names of section 5 (Where gives where, ReviewTask gives review-task).
  4. In cli.rs write pub fn main_exit() -> u8: collect std::env::args, call crate::remote::forward::maybe(&args) and return its code when it is Some, else parse with Cli::try_parse_from (on error print the clap error and return 2, or 0 for --help and --version), match the command to its run function, and return crate::output::emit(<command name>, global.json, &result).
  5. Add pub mod cli; and pub mod cmd; to src/lib.rs.
  6. Make src/main.rs: fn main() -> std::process::ExitCode { std::process::ExitCode::from(dossier::cli::main_exit()) }.
  7. Create tests/smoke.rs: one test runs the program with --version and asserts success and that the output starts with dossier; a second test runs dossier no-such-command and asserts exit code 2 (a parse error, which stays 2 after the commands are built).
  8. Run cargo build, cargo clippy --all-targets -- -D warnings and cargo test --test smoke.
- accept: dossier --version prints the version; every command of design.md section 5 is accepted by the parser and fails with its not built yet message; an unknown command exits with 2; clippy is clean.
- verify: cargo test --test smoke
- verified: 2026-11-21
- commit: 2f7efdf
- reqs: D16, R-OUT-2
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-21, planner): Simple emit moved from T-068 to T-067 (output.rs is T-067's file; T-068 did not list it and could run beside T-072). Smoke test uses an unknown command for exit 2, because dossier where stops returning 2 once T-109 is built.
- note (2026-11-21, haiku-t-068): done

### T-069 · CI workflow, MIT licence and CLAUDE.md
- status: done
- owner: haiku-t069
- priority: P1 · difficulty: 1 · type: chore
- milestone: T-065
- why: The checks must run on Windows and Linux on every push, and a public repository needs a licence.
- depends: T-066
- files: .github/workflows/ci.yml, LICENSE, CLAUDE.md
- read: AGENTS.md, docs/requirements.md
- steps:
  1. Create .github/workflows/ci.yml: on push and pull_request; a job with a matrix of windows-latest and ubuntu-latest; steps: checkout, install stable Rust with rustfmt and clippy, cargo fmt --all -- --check, cargo clippy --all-targets --all-features -- -D warnings, cargo test --all-features.
  2. Create LICENSE with the standard MIT licence text, year 2026, copyright holder: the dossier authors.
  3. Create CLAUDE.md with the single line @AGENTS.md.
- accept: The three files exist; ci.yml is valid YAML with both systems and the three checks.
- verify: git ls-files --error-unmatch .github/workflows/ci.yml LICENSE CLAUDE.md
- verified: 2026-11-21
- commit: c197408
- reqs: REL-1, NFR-2
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-21, haiku-t069): done

### T-070 · Foundation: errors, output, time, paths, settings, names, lock, safe writing
- status: done
- owner: coordinator
- priority: P0 · type: milestone
- milestone: T-001
- goal: The small library modules that every command uses work as docs/design.md section 3 describes, each tested in its own file.
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-21, coordinator): done

### T-071 · error.rs: exit codes and code names
- status: done
- owner: haiku-t-071
- priority: P0 · difficulty: 1 · type: feature
- milestone: T-070
- why: Agents branch on the exit code and the code name of every failure.
- depends: T-067
- files: src/error.rs
- read: docs/design.md
- steps:
  1. Open src/error.rs. Keep the Code enum, Error, Result and Error::new.
  2. Implement Error::exit with the table of design.md section 3 error.rs.
  3. Implement Error::name: the variant name in upper case with underscores between words, for example NoDossier gives NO_DOSSIER; write it as one match with a string per variant.
  4. Implement From<std::io::Error> for Error giving Code::Io and the error text.
  5. Implement std::fmt::Display for Error printing the message.
  6. Add #[cfg(test)] mod tests: every variant has the exit code of the table (list all 20), three name checks, and the io conversion.
  7. Run cargo test --lib error::.
- accept: All 20 codes map to the exit codes of the design table; names are SCREAMING_SNAKE_CASE.
- verify: cargo test --lib error::
- verified: 2026-11-21
- commit: cc8f6d1
- reqs: R-OUT-2, R-OUT-3
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-21, haiku-t-071): done

### T-072 · output.rs: text, JSON envelope, closed pipe
- status: done
- owner: haiku-t-072
- priority: P0 · difficulty: 1 · type: feature
- milestone: T-070
- why: Every command prints through this one function, so text and JSON output are the same everywhere.
- depends: T-071
- files: src/output.rs
- read: docs/design.md
- steps:
  1. Implement Output::text, Output::with and Output::warn as in design.md.
  2. Write a private fn render(command, json, result) -> (String for stdout, String for stderr, u8 exit code) that builds the texts described under output.rs in design.md; for JSON use serde_json::json! and serde_json::to_string, one line.
  3. Implement emit: call render, write both strings with std::io::Write on locked stdout and stderr, ignore every write error (a closed pipe must not panic), return the exit code.
  4. An empty text prints nothing on standard output, not even a line break.
  5. Add tests on render: success text gets a final line break; an empty text gives an empty string; warnings go to stderr as warning: <text>; an error gives error: <message> and its exit code; JSON success and JSON failure have exactly the keys of design.md and parse with serde_json.
  6. Run cargo test --lib output::.
- accept: render produces the two shapes of design.md; emit never panics when writing fails.
- verify: cargo test --lib output::
- verified: 2026-11-21
- commit: eb0d99a
- reqs: R-OUT-1, R-OUT-2, R-OUT-4
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-21, haiku-t-072): done

### T-073 · time.rs: timestamps to the second with UTC offset
- status: done
- owner: haiku-t-073
- priority: P0 · difficulty: 1 · type: feature
- milestone: T-070
- why: Everything recorded carries when it happened; tests need a clock they can fix.
- depends: T-067
- files: src/time.rs
- read: docs/design.md, docs/format.md
- steps:
  1. Implement valid(stamp): true when chrono::DateTime::parse_from_rfc3339 accepts it and it is 25 characters long.
  2. Implement now(): if the environment variable DOSSIER_NOW is set and valid, return it; else chrono::Local::now() formatted with %Y-%m-%dT%H:%M:%S%:z.
  3. Implement date(stamp): the first 10 characters, or the whole string when it is shorter.
  4. Implement days_between(earlier, later): parse both with parse_from_rfc3339 and return the whole days between them; 0 when one does not parse.
  5. Add tests: valid accepts 2026-11-20T14:03:27+01:00 and refuses a date without time; now() returns DOSSIER_NOW when set (set and remove the variable inside one test that holds crate::ENV_LOCK; in edition 2024 set_var and remove_var need an unsafe block); date and days_between on fixed values.
  6. Run cargo test --lib time::.
- accept: now() has the form of format.md section 7 and obeys DOSSIER_NOW.
- verify: cargo test --lib time::
- verified: 2026-11-21
- commit: a831f28
- reqs: R-HIST-4, R-XOS-7
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-21, planner): Test holds crate::ENV_LOCK (see T-067): unit tests run in parallel threads and share the environment.
- note (2026-11-21, haiku-t-073): done

### T-074 · hash.rs: SHA-256 as hex
- status: done
- owner: haiku-t-074
- priority: P0 · difficulty: 1 · type: feature
- milestone: T-070
- why: Checksums tell whether a file changed since it was read or approved.
- depends: T-067
- files: src/hash.rs
- read: docs/design.md
- steps:
  1. Implement sha256_hex with the sha2 crate: hash the bytes and write each byte as two lower-case hex digits.
  2. Add tests: the empty input gives e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855; abc gives ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad.
  3. Run cargo test --lib hash::.
- accept: The two known values match.
- verify: cargo test --lib hash::
- verified: 2026-11-21
- commit: 17cb156
- reqs: R-IO-2, R-LIFE-6
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-21, haiku-t-074): done

### T-075 · paths.rs: slugs, Windows-safe names, stored paths
- status: done
- owner: haiku-t-075
- priority: P0 · difficulty: 1 · type: feature
- milestone: T-070
- why: File names and stored paths must be the same whether a command was typed on Windows or Linux.
- depends: T-071
- files: src/paths.rs
- read: docs/design.md, docs/format.md, docs/requirements.md
- steps:
  1. Implement slug(title) exactly as format.md section 4 describes.
  2. Implement windows_safe(name): false when the name is empty, ends with a dot or space, contains one of < > : double quote | ? * or a character below 32, or when its part before the first dot, in lower case, is con, prn, aux, nul, com1 to com9 or lpt1 to lpt9.
  3. Implement stored(root, cwd, typed): replace every backslash with /; if the result is not absolute join it to cwd; resolve . and .. parts without touching the disk; if the result does not start with root return Code::BadValue with the message 'the path is outside the project folder'; return the part after root with / separators and no leading /.
  4. Treat a Windows drive path such as C:/x as absolute on every system when comparing with a root written the same way.
  5. Implement shown(stored): on Windows replace / with backslash, elsewhere return it unchanged.
  6. Add tests: slug for a title with umlauts, punctuation, 80 characters and an empty title; windows_safe for con.md, a name with a colon, a trailing dot and a normal name; stored for a relative path, a path with backslashes, a path with .. that stays inside and one that leaves the root.
  7. Run cargo test --lib paths::.
- accept: slug follows format.md section 4; a path typed with backslashes and one typed with slashes store the same text; paths outside the project are refused.
- verify: cargo test --lib paths::
- verified: 2026-11-21
- commit: 1b51023
- reqs: R-ENTRY-1, R-XOS-2, R-XOS-4
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-21, haiku-t-075): done

### T-076 · find.rs: locate the DOSSIER folder
- status: done
- owner: haiku-t-076
- priority: P0 · difficulty: 1 · type: feature
- milestone: T-070
- why: Every command must agree on which dossier it works on.
- depends: T-071
- files: src/find.rs
- read: docs/design.md
- steps:
  1. Implement find(start, explicit): when explicit is Some, return it if it is a folder that contains dossier.toml, else Code::NoDossier naming the path.
  2. Otherwise look for a folder named DOSSIER containing dossier.toml in start, then in each parent of start, and return the first.
  3. When none is found return Code::NoDossier with the message 'no DOSSIER folder found here or above; create one with: dossier init --code <code>'.
  4. Add tests with tempfile: found in the start folder, found two levels up, not found, explicit path accepted and refused, a folder name with a space and an umlaut.
  5. Run cargo test --lib find::.
- accept: The dossier is found from any subfolder; a missing one gives NO_DOSSIER with the hint.
- verify: cargo test --lib find::
- verified: 2026-11-21
- commit: d4bba35
- reqs: R-PLACE-1, R-PLACE-8, NFR-4
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-21, haiku-t-076): done

### T-077 · config.rs: read dossier.toml, default text, add a person
- status: done
- owner: haiku-t-077
- priority: P0 · difficulty: 1 · type: feature
- milestone: T-070
- why: The project code, the limits and the list of people and their LLMs drive who may write and approve.
- depends: T-071
- files: src/config.rs
- read: docs/design.md, docs/format.md
- steps:
  1. Define private serde structs with #[serde(deny_unknown_fields)] matching format.md section 10: top level format, code, tags, people, limits; every part except format and code has #[serde(default)].
  2. Implement load(dossier): read dossier/dossier.toml, parse with toml::from_str, and build Config with the defaults of format.md section 10 for missing limits; a read error is Code::Io; a parse error is Code::Config with the message '<path>: <toml error text>' (the toml error names the line and the key); format greater than 1 is Code::TooNew with the message 'this dossier needs a newer dossier program'.
  3. Implement default_text(code): the text of format.md section 10 with that code, empty tag lists, no people, and the [limits] table.
  4. Implement add_person(dossier, person, llms): read the file as text; if a [[people]] table with that name exists, add the missing LLM names to its llms line; else append a blank line and a new [[people]] table with name and llms; write it back with std::fs::write; keep every other line unchanged.
  5. Add tests with tempfile: load of default_text gives the defaults; an unknown key gives Code::Config; format = 2 gives Code::TooNew; add_person twice with different LLMs gives one person with both; the lines before stay byte-identical.
  6. Run cargo test --lib config::.
- accept: dossier.toml of format.md section 10 loads; unknown keys are errors naming the file; add_person keeps other lines.
- verify: cargo test --lib config::
- verified: 2026-11-21
- commit: 2847923
- reqs: R-CFG-1, R-CFG-3, R-CFG-4, NFR-7
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-21, haiku-t-077): done

### T-078 · names.rs: who is writing
- status: done
- owner: haiku-t-078
- priority: P0 · difficulty: 1 · type: feature
- milestone: T-070
- why: Each entry must show whether a person or an LLM wrote it, and for whom.
- depends: T-077
- files: src/names.rs
- read: docs/design.md
- steps:
  1. Implement is_person(cfg, name): true when a person has that name, ignoring case.
  2. Implement resolve(cfg, flag): take the name from flag, else from the environment variable DOSSIER_AGENT, else TODO_AGENT; an empty value counts as missing; none gives Code::Usage with the message 'this command needs a name: --agent <name> or DOSSIER_AGENT'.
  3. If the name is a person return Who with name and person equal and is_llm false; if it is in a person's llms return Who with that person and is_llm true; else Code::UnknownName with the message listing every known person and LLM and the hint 'add one with: dossier people add <person> --llm <name>'.
  4. Implement label(who): the person, or '<person> via <name>' for an LLM.
  5. Add tests with a Config built in the test: person, LLM, unknown name, no name, no name and the environment fallback together in one test that holds crate::ENV_LOCK: first remove DOSSIER_AGENT and TODO_AGENT (a worker's shell may set TODO_AGENT), then set and remove them as needed (set_var and remove_var need an unsafe block in edition 2024), and label.
  6. Run cargo test --lib names::.
- accept: A name resolves to its person; unknown names are refused with the known names listed.
- verify: cargo test --lib names::
- verified: 2026-11-21
- commit: a88c949
- reqs: D13, R-CFG-4, R-LIFE-2
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-21, planner): Test holds crate::ENV_LOCK (see T-067): unit tests run in parallel threads and share the environment.
- note (2026-11-21, haiku-t-078): done

### T-079 · lock.rs: the write lock and the network drive test
- status: done
- owner: haiku-t-079
- priority: P0 · difficulty: 2 · type: feature
- milestone: T-070
- why: Three people and their LLMs write at the same time; the lock keeps entries and ID counters intact.
- depends: T-071
- files: src/lock.rs
- read: docs/design.md, docs/requirements.md
- steps:
  1. Implement acquire(state, wait_seconds, name, command): create the state folder if missing; open or create write.lock; call File::try_lock in a loop every 50 ms until it succeeds or wait_seconds passed.
  2. On success write holder.json next to it with name, command, the process ID and time::now(), and return a Guard holding the file; implement Drop for Guard to delete holder.json (ignore errors); the operating system releases the lock when the file is closed.
  3. On timeout return Code::Busy with the message 'the dossier is locked by <name> (<command>) since <time>; try again', read from holder.json when it exists.
  4. Implement is_network(path) for Linux: find the longest mount point in /proc/mounts that is a prefix of the canonical path and return true when its type is cifs, smb3, smbfs, nfs or nfs4.
  5. Implement is_network(path) for Windows with #[cfg(windows)]: true when the path starts with two backslashes or //, else call GetDriveTypeW from windows-sys on the drive root (such as C: plus backslash, as a wide string) and return true for DRIVE_REMOTE.
  6. Add tests: acquire then a second acquire with wait 0 in the same process fails with Code::Busy; after the guard is dropped it succeeds; holder.json exists while held; is_network is false for a tempfile folder.
  7. Run cargo test --lib lock::.
- accept: A held lock makes a second writer fail with BUSY naming the holder; the lock is free after the guard is dropped.
- verify: cargo test --lib lock::
- verified: 2026-11-21
- commit: 5904ff6
- final: The Windows part needs one call into the operating system; it cannot be written down more exactly without writing the code.
- reqs: R-LOCK-1 to R-LOCK-3, R-XOS-1
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-21, haiku-t-079): done

### T-080 · atomic.rs: replace a file in one step
- status: done
- owner: haiku-t-080
- priority: P0 · difficulty: 1 · type: feature
- milestone: T-070
- why: A crash or a hand edit during a write must never leave half an entry.
- depends: T-074
- files: src/atomic.rs
- read: docs/design.md
- steps:
  1. Implement seen(path): None when the file does not exist; else read it and return its length and hash::sha256_hex.
  2. Implement replace(path, bytes, expected): when expected is Some, compare it with seen(path) and return Code::Changed with the message '<path> was changed by someone else; run the command again' when they differ.
  3. Create the parent folder if missing; write bytes to '<file name>.tmp-<process id>' in the same folder; call sync_all on it; rename it over the path; on Windows retry the rename up to 10 times with 50 ms pauses when it fails with a permission error.
  4. If anything fails after the temporary file was created, delete it and return Code::Io.
  5. Add tests with tempfile: a new file is written; an existing file is replaced; expected None refuses an existing file; a changed file gives Code::Changed and the old content stays; no .tmp- file is left behind.
  6. Run cargo test --lib atomic::.
- accept: A file is either the old or the new version, never half; a file changed since it was read is not overwritten.
- verify: cargo test --lib atomic::
- verified: 2026-11-21
- commit: 5d4c398
- reqs: R-IO-1, R-IO-2
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-21, haiku-t-080): done

### T-081 · term.rs: is a person at a terminal
- status: done
- owner: haiku-t-081
- priority: P0 · difficulty: 1 · type: feature
- milestone: T-070
- why: Approval must be given by a person at a terminal, and tests need a stand-in for one.
- depends: T-071
- files: src/term.rs
- read: docs/design.md
- steps:
  1. Keep a private static std::sync::Mutex<Option<Vec<String>>> holding the answers of DOSSIER_ANSWERS, filled on first use by splitting the variable at |.
  2. Implement is_terminal(): true when DOSSIER_ANSWERS is set; else std::io::IsTerminal on stdin and stdout are both true.
  3. Implement ask(prompt): when DOSSIER_ANSWERS is set, remove and return the first answer, or Code::NeedsTerminal with the message 'no more answers' when none is left.
  4. Otherwise, when is_terminal() is false return Code::NeedsTerminal with the message 'this command must be run by a person at a terminal'; else write the prompt to stderr, read one line from stdin and return it trimmed.
  5. Add one test, holding crate::ENV_LOCK, that sets DOSSIER_ANSWERS to a|b and checks is_terminal and two asks and the error on the third.
  6. Run cargo test --lib term::.
- accept: Without a terminal ask fails with NEEDS_TERMINAL; DOSSIER_ANSWERS supplies answers in tests.
- verify: cargo test --lib term::
- verified: 2026-11-21
- commit: c4680d1
- reqs: D6, R-DRAFT-4
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-21, planner): Test holds crate::ENV_LOCK (see T-067): unit tests run in parallel threads and share the environment.
- note (2026-11-21, haiku-t-081): done

### T-082 · ctx.rs: what every command starts with
- status: done
- owner: haiku-t-082
- priority: P0 · difficulty: 1 · type: feature
- milestone: T-070
- why: One place finds the dossier, loads the settings and resolves who is writing.
- depends: T-076, T-077, T-078, T-068
- files: src/ctx.rs
- read: docs/design.md
- steps:
  1. Implement Ctx::load(g, command): take the current folder from std::env::current_dir; call find::find(&cwd, g.dossier.as_deref()); root is the parent of the dossier folder; state is dossier/.state.
  2. Load the settings with config::load.
  3. Resolve who: call names::resolve(&cfg, g.agent.as_deref()); when it fails with Code::Usage (no name given) store None; pass any other error on.
  4. Fill json and dry_run from g and command from the argument.
  5. Implement writer(): the Who, or Code::Usage with the message 'this command needs a name: --agent <name> or DOSSIER_AGENT'.
  6. Add a test that creates a temporary DOSSIER with config::default_text plus a person, builds a Global with dossier set to it and an agent, and checks root, state, cfg.code and who; and one with an unknown agent that fails with Code::UnknownName.
  7. Run cargo test --lib ctx::.
- accept: Ctx::load returns the folders, settings and writer; an unknown name is an error, a missing name is not.
- verify: cargo test --lib ctx::
- verified: 2026-11-21
- commit: 9f585d9
- reqs: R-PLACE-1, D13
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-21, haiku-t-082): done

### T-083 · Entries: files are read and changed without damage, with who and when
- status: done
- owner: coordinator
- priority: P0 · type: milestone
- milestone: T-001
- goal: An entry file of docs/format.md can be parsed, changed line by line, created, validated, stored under a new ID, snapshotted and journalled; every change carries the person, the LLM and the time.
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-21, coordinator): done

### T-084 · entry/mod.rs: parse an entry and read its parts
- status: done
- owner: haiku-t-084
- priority: P0 · difficulty: 2 · type: feature
- milestone: T-083
- why: Every command reads entries through this model; it must keep every byte of a hand-written file.
- depends: T-071
- files: src/entry/mod.rs
- read: docs/design.md, docs/format.md
- steps:
  1. Implement the methods of Kind and Status as design.md lists them, with the names, prefixes and folders of format.md sections 4 and 5.
  2. Implement Entry::parse(text): split the text into lines that keep their ending (split after each LF); a text without a heading line starting with '# ' is Code::Parse with the message 'line <n>: no heading'; a format line with a number above 1 is Code::TooNew.
  3. Implement text(): all lines joined; it must equal the input exactly.
  4. Write private helpers: heading_index (first line starting with '# ', a byte-order mark before the format line is skipped), field_range (from the first line after the heading that matches '- key: value' to the first line after it that does not), and section_ranges (each line starting with '## ' up to the next one).
  5. A field line is: '- ', then a key of lower-case letters, digits and -, then ':', then the value trimmed; a line '- key:' with nothing after the colon has an empty value.
  6. Implement id() and title() from the heading, accepting the separators ' · ', ' - ' and ': ' (format.md section 3); the ID is the first word, upper-cased.
  7. Implement field, fields, list, section, sections, kind, status and history as design.md describes.
  8. Add tests: the example of format.md section 1 parses and text() equals the input; the same with CRLF endings; with a byte-order mark; with no final line ending; with an unknown key and an unknown section (both returned); each heading separator; a file without heading fails.
  9. Run cargo test --lib entry::tests.
- accept: Entry::parse(text).text() equals text for every valid file; fields, sections, ID and title are read as format.md describes.
- verify: cargo test --lib entry::tests
- verified: 2026-11-21
- commit: 683699c
- final: Parsing needs a few small decisions about edge cases that are easier to settle in code than in steps.
- reqs: R-ENTRY-1, R-ENTRY-2, R-EDIT-3, R-XOS-6, D17
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-21, haiku-t-084): done

### T-085 · entry/edit.rs: change fields, title, sections and add history lines
- status: done
- owner: haiku-t-085
- priority: P0 · difficulty: 2 · type: feature
- milestone: T-083
- why: The tool must change only the lines it has to and keep every other byte.
- depends: T-084, T-078
- files: src/entry/edit.rs
- read: docs/design.md, docs/format.md
- steps:
  1. Write a private helper ending_for(index): the line ending of the line before index (LF or CRLF), LF when there is none.
  2. Implement set_field: if a line with the key exists replace only its value and keep its ending; else insert '- key: value' before the first history line, or at the end of the field block when there is none; an empty value removes the field instead.
  3. Implement remove_field: delete the first line with that key (never a history line).
  4. Implement set_title: rewrite the heading as '# <ID> · <title>' keeping its ending.
  5. Implement set_section: replace the lines between the section heading and the next section heading with: one blank line, the body lines, one blank line; when the section does not exist append a blank line (if the file does not end with one), '## <name>', a blank line and the body at the end; an empty body removes the section and its heading.
  6. Implement add_history: append '- history: <time> <names::label(who)>: <event>' plus ' — <detail>' when detail is not empty, as the last line of the field block.
  7. Add tests on the example of format.md section 1: each operation changes exactly the expected lines (compare the full text); the same with a CRLF file, where new lines are CRLF; history lines are never changed by set_field('history', ...), which must do nothing.
  8. Run cargo test --lib entry::edit::.
- accept: Each change touches only its own lines; line endings are kept; history lines are only ever added.
- verify: cargo test --lib entry::edit::
- verified: 2026-11-21
- commit: 34c7371
- final: Line-level editing has edge cases that are easier to settle in code than in steps.
- reqs: R-EDIT-1, R-HIST-5, R-XOS-6
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-21, haiku-t-085): done

### T-086 · entry/new.rs: the text of a new entry file
- status: done
- owner: haiku-t-086
- priority: P0 · difficulty: 1 · type: feature
- milestone: T-083
- why: Every new entry must look exactly like the example in format.md, whichever command creates it.
- depends: T-084
- files: src/entry/new.rs
- read: docs/design.md, docs/format.md
- steps:
  1. Implement render(draft): start with the format line and the heading '# <id> · <title>', then a blank line.
  2. Write the fields: first those whose key is in the order list of format.md section 5 (kind first, history last), in that order; then every other key in the order given; skip empty values; history values are written one line each, last.
  3. Always write kind from draft.kind even when it is not in draft.fields.
  4. After a blank line write each section of draft.sections that has a non-empty body, in the order of format.md section 8 for that kind, unknown section names after them: '## <name>', blank line, body, blank line.
  5. All lines end with LF; the file ends with exactly one LF.
  6. Add tests: a decision renders to the exact text of format.md section 1 (build the Draft from those values); Entry::parse of every rendered text succeeds and returns the same fields and sections; a section with an empty body is left out.
  7. Run cargo test --lib entry::new::.
- accept: render gives the canonical text of format.md section 1, and it parses back to the same fields and sections.
- verify: cargo test --lib entry::new::
- verified: 2026-11-21
- commit: 9050524
- reqs: R-ENTRY-1 to R-ENTRY-3
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-21, haiku-t-086): done

### T-087 · entry/rules.rs: what an entry of each kind needs
- status: done
- owner: haiku-t-087
- priority: P0 · difficulty: 1 · type: feature
- milestone: T-083
- why: A lesson without its conditions must never be approved.
- depends: T-084
- files: src/entry/rules.rs
- read: docs/design.md, docs/format.md
- steps:
  1. Implement section_order and required_sections with the table of format.md section 8.
  2. Implement problems(entry): one Problem 'missing section <name>' for each required section that is missing or empty.
  3. For a lesson add 'missing field confidence' when the field is missing, or 'confidence must be measured, simulated, experience or guess' when it has another value.
  4. For an assumption add 'state must be open, confirmed or wrong' when the field is missing or has another value.
  5. For a finding with an outcome field other than 'dead end' add 'outcome must be dead end'.
  6. When the entry has no kind field return a single Problem 'missing field kind'.
  7. Add tests: a complete entry of each kind has no problems; a lesson without Valid for, without confidence and with a wrong confidence; an assumption without state; a finding with a wrong outcome.
  8. Run cargo test --lib entry::rules::.
- accept: problems lists exactly what is missing for approval per kind.
- verify: cargo test --lib entry::rules::
- verified: 2026-11-21
- commit: 9e391d2
- reqs: R-ENTRY-5 to R-ENTRY-7, R-ENTRY-13, R-ENTRY-15
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-21, haiku-t-087): done

### T-088 · store.rs: read all entries, find one by ID, name a new file
- status: done
- owner: haiku-t-088
- priority: P0 · difficulty: 1 · type: feature
- milestone: T-083
- why: Commands need one way to load entries and to know where a new one goes.
- depends: T-084, T-075
- files: src/store.rs
- read: docs/design.md, docs/format.md
- steps:
  1. Implement path_for(dossier, kind, id, title): dossier / kind.folder() / '<id>-<paths::slug(title)>.md'.
  2. Implement all(dossier): for each Kind::all() read every file ending in .md in its folder (a missing folder is empty); parse each; a file that is not UTF-8 or fails to parse becomes a warning '<path>: <error message>' and is skipped; sort the items by the number in the ID, then by prefix.
  3. Implement get(dossier, id): strip a place prefix (everything up to and including the last ':'), upper-case the rest, and return the item of all() with that ID, or Code::NotFound with the message 'no entry <id>'.
  4. Add tests with tempfile: two entries in different folders are returned in order; a broken file gives a warning and the others still load; get accepts dec-001 and P-001:DEC-001; an unknown ID is NotFound.
  5. Run cargo test --lib store::.
- accept: All entries load in ID order; broken files are skipped with a warning; an entry is found by ID in any case, with or without a place prefix.
- verify: cargo test --lib store::
- verified: 2026-11-21
- commit: 6dabfee
- reqs: R-ID-2, R-EDIT-3, R-ENTRY-1
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-21, haiku-t-088): done

### T-089 · ids.rs: the next ID of a kind
- status: done
- owner: haiku-t-089
- priority: P0 · difficulty: 1 · type: feature
- milestone: T-083
- why: Two writers must never get the same ID, and an ID is never used twice.
- depends: T-088
- files: src/ids.rs
- read: docs/design.md, docs/format.md
- steps:
  1. Implement next(dossier, state, kind): read state/ids.json as a JSON object of prefix to number (a missing or unreadable file is an empty object).
  2. Find the highest number among the entries of store::all whose ID has this kind's prefix.
  3. The new number is 1 plus the larger of the two; store it under the prefix and write ids.json with atomic::replace (no expected value).
  4. Return the ID as prefix, '-', and the number with at least three digits (DEC-001, DEC-1000).
  5. Add tests with tempfile: the first ID is DEC-001; the next DEC-002; with an entry DEC-007 on disk the next is DEC-008; after ids.json says 9 and the entries are deleted the next is DEC-010; kinds count separately.
  6. Run cargo test --lib ids::.
- accept: IDs count per kind, never repeat, and survive deleted entries.
- verify: cargo test --lib ids::
- verified: 2026-11-21
- commit: f921799
- reqs: R-ID-1, R-ID-3
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-21, haiku-t-089): done

### T-090 · journal.rs: one line per change
- status: done
- owner: haiku-t-090
- priority: P0 · difficulty: 1 · type: feature
- milestone: T-083
- why: The journal answers who did what when for the whole dossier.
- depends: T-071
- files: src/journal.rs
- read: docs/design.md, docs/format.md
- steps:
  1. Implement append(state, record): create the state folder if missing; open journal.jsonl with append and create; write serde_json::to_string(record) and one LF.
  2. Implement read(state): a missing file is an empty list; parse each non-empty line with serde_json; skip lines that fail to parse.
  3. Add tests with tempfile: two appended records come back in order with equal fields; a garbage line in between is skipped; a missing file gives an empty list.
  4. Run cargo test --lib journal::.
- accept: Records are appended one JSON line each and read back oldest first.
- verify: cargo test --lib journal::
- verified: 2026-11-21
- commit: 1984cd3
- reqs: R-HIST-1, R-HIST-4
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-21, haiku-t-090): done

### T-091 · snapshot.rs: keep the old version of an entry
- status: done
- owner: haiku-t-091
- priority: P0 · difficulty: 1 · type: feature
- milestone: T-083
- why: There is no git in the project folders, so snapshots are the only undo.
- depends: T-071
- files: src/snapshot.rs
- read: docs/design.md, docs/format.md
- steps:
  1. Implement list(state, id): read state/history/<ID>/index.jsonl (missing gives an empty list), one Snap per line in file order.
  2. Implement save: n is 1 plus the highest n of list; write the bytes to state/history/<ID>/<n as six digits>.md, creating folders; append the index line with n, time, name and command; return n.
  3. Implement load(state, id, n): read that file, or Code::NotFound with the message 'no snapshot <n> of <id>'.
  4. Use the upper-case ID as the folder name.
  5. Add tests with tempfile: two saves give 1 and 2; list returns both with their data; load returns the bytes; load of a missing number is NotFound.
  6. Run cargo test --lib snapshot::.
- accept: Each save gets the next number; snapshots are listed and loaded by number.
- verify: cargo test --lib snapshot::
- verified: 2026-11-21
- commit: a94aba3
- reqs: R-HIST-2
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-21, haiku-t-091): done

### T-092 · approval.rs: notice an approved entry that was changed by hand
- status: done
- owner: haiku-t-092
- priority: P0 · difficulty: 1 · type: feature
- milestone: T-083
- why: Approved knowledge that someone edited without the tool must be marked until a person looks at it again.
- depends: T-074, T-084
- files: src/approval.rs
- read: docs/design.md, docs/format.md
- steps:
  1. Write a private fn stable(bytes) -> Vec<u8>: the text without the lines that start with '- history:', '- status:', '- approved:', '- valid-from:' or '- valid-until:', and with every CRLF turned into LF.
  2. Implement record(state, id, bytes): read state/approved.json as a JSON object (missing is empty), set the upper-case ID to hash::sha256_hex(stable(bytes)), write it with atomic::replace.
  3. Implement forget(state, id): remove the ID from the object and write it.
  4. Implement mark(state, id, bytes): NotRecorded when the ID is not in the object, Unchanged when the hash matches, Changed otherwise.
  5. Add tests with tempfile: record then mark is Unchanged; adding a history line or changing the status line keeps it Unchanged; changing a word in a section gives Changed; forget gives NotRecorded.
  6. Run cargo test --lib approval::.
- accept: A change to the content is noticed; history, status and approval lines do not count as a change.
- verify: cargo test --lib approval::
- verified: 2026-11-21
- commit: cf1db65
- reqs: R-LIFE-6
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-21, haiku-t-092): done

### T-093 · tx.rs: every write goes through one transaction
- status: done
- owner: haiku-t-093
- priority: P0 · difficulty: 2 · type: feature
- milestone: T-083
- why: Lock, snapshot, safe replacement and journal line must happen for every change, in every command, the same way.
- depends: T-082, T-079, T-080, T-090, T-091, T-088, T-072, T-073
- files: src/tx.rs
- read: docs/design.md, docs/requirements.md
- steps:
  1. Define Tx with a reference to the Ctx, the time taken once from time::now(), and a private Vec of pending operations: Put{path, old raw bytes or None, text, change}, Delete{path, old bytes, change}, Rename{old path, old bytes, new path, text, change}.
  2. Implement ctx(), time(), put, delete and rename so that they only push onto the list.
  3. Implement write(ctx, f): first call ctx.writer()? to get the Who; if lock::is_network(&ctx.dossier) is true and the program arguments (std::env::args) do not contain --allow-network-drive, return Code::NetworkDrive with the message 'the dossier is on a network drive; run the command on the server, see dossier connect'.
  4. If ctx.dry_run is false, acquire the lock with lock::acquire(&ctx.state, ctx.cfg.lock_wait_seconds, &who.name, &ctx.command); then create the Tx and call f.
  5. If f fails, return its error and write nothing.
  6. If ctx.dry_run is true, add to the Output one warning per operation 'dry run: would write <path>' (or 'would delete', 'would rename') and return it.
  7. Otherwise, for each operation in order: when there are old bytes, snapshot::save them under the change's ID; then atomic::replace the file with the new text, expecting the old bytes (Some(seen of old) or None for a new file); for Delete remove the file; for Rename write the new path and remove the old one; then journal::append a Record with the time, who.name, who.person, who.is_llm, ctx.command and the change's id, event and detail.
  8. Add tests with a temporary dossier and a Ctx built in the test: a put of a new file creates it and one journal line; a put over an existing file makes snapshot 1 with the old bytes; an error in f writes nothing; dry_run writes nothing and returns a warning; a writer of None gives Code::Usage.
  9. Run cargo test --lib tx::.
- accept: A change is written only under the lock, with a snapshot of the old file and a journal line carrying person, LLM and time; dry run and failures write nothing.
- verify: cargo test --lib tx::
- verified: 2026-11-21
- commit: 1fa449c
- final: It ties five modules together; the order of the steps is given but the borrowing details are settled in code.
- reqs: R-LOCK-1, R-LOCK-3, R-IO-1, R-IO-2, R-HIST-1, R-HIST-2, R-HIST-4, R-WRITE-10
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-21, haiku-t-093): done

### T-094 · create.rs: create one entry with author, approval and history
- status: done
- owner: haiku-t-094
- priority: P0 · difficulty: 2 · type: feature
- milestone: T-083
- why: Who wrote an entry, whether it is approved and when must be decided in one place for every kind and every command.
- depends: T-093, T-089, T-086, T-087, T-092
- files: src/create.rs
- read: docs/design.md, docs/format.md, docs/requirements.md
- steps:
  1. Implement create(tx, new) following the rules in the comment of design.md create.rs.
  2. Work out author, via and whether it is approved: writer is tx.ctx().writer()?; with said_by, check names::is_person and fail with Code::UnknownName otherwise; an LLM writing a decision without said_by fails with Code::NeedsPerson and the message 'a decision needs the person who made it: --said-by <person>'.
  3. An entry that would be approved but has rules::problems (build the Entry from the rendered text) fails with Code::MissingField listing the problems; a draft may have problems.
  4. Get the ID with ids::next; build the fields in this order: status, created (tx.time()), author, via, approved ('<time> by <author>') and valid-from when approved, then new.fields, then history lines 'created' and, when approved, 'approved', each formatted as format.md section 6 with the writer's label.
  5. Render with entry::new::render, queue it with tx.put(store::path_for(...), None, text, Change{id, event 'created', detail 'approved' or 'draft'}).
  6. When the entry is an approved lesson and not a dry run, call approval::record with the text.
  7. Return the ID and the text.
  8. Add tests with a temporary dossier (people anna with LLM anna-claude): a person's decision is approved with author anna and no via; an LLM's decision without said_by fails; with said_by it is approved with author anna and via anna-claude; an LLM's lesson is a draft; an approved lesson missing Valid for fails; the history lines carry the time of the transaction (compare with tx.time() inside the closure; do not set DOSSIER_NOW here, the time tests change it).
  9. Run cargo test --lib create::.
- accept: Author, LLM, status, approval time and history lines follow design.md for people and LLMs; incomplete entries cannot be approved.
- verify: cargo test --lib create::
- verified: 2026-11-21
- commit: 61f549a
- final: The approval rules have several cases that interact; they are listed, but their order is settled in code.
- reqs: D5, R-WRITE-2 to R-WRITE-4, R-LIFE-2, R-ENTRY-14, R-HIST-4, R-HIST-5
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-21, planner): Test compares with tx.time() instead of setting DOSSIER_NOW, which the time tests change.
- note (2026-11-21, haiku-t-094): done

### T-095 · Recording: one short command writes an entry
- status: done
- owner: coordinator
- priority: P0 · type: milestone
- milestone: T-001
- goal: init, people, add, decide, found, learn, assume, set, retitle, link and edit work as docs/design.md section 5 describes, each in its own file with its own test.
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-21, coordinator): done

### T-096 · init: create a project dossier
- status: done
- owner: haiku-t-096
- priority: P0 · difficulty: 1 · type: feature
- milestone: T-095
- why: The first command anyone runs in a project.
- depends: T-068, T-077, T-072
- files: src/cmd/init.rs, tests/cmd_init.rs
- read: docs/design.md, docs/format.md, docs/requirements.md
- steps:
  1. Give Args one required option: #[arg(long)] code: String.
  2. In run: the target is <current folder>/DOSSIER; if it exists return Code::Exists with the message 'DOSSIER already exists here'.
  3. Refuse a code that is empty or contains characters other than letters, digits and - with Code::BadValue.
  4. Create DOSSIER, the four kind folders of format.md section 4 and .state; write dossier.toml with config::default_text(code).
  5. On Linux (#[cfg(unix)]) set the mode of every created folder to 0o2775 and of dossier.toml to 0o664 with std::os::unix::fs::PermissionsExt, and add to the output the line 'group: files are writable by the group of this folder'.
  6. Return Output::text with 'created DOSSIER for <code>' and a second line 'next: dossier people add <person> --llm <name>'.
  7. Create tests/cmd_init.rs using assert_cmd directly in a tempfile folder (not tests/common): init creates the folders and a dossier.toml that contains the code; a second init fails with exit code 1; a code with a space fails.
  8. Run cargo test --test cmd_init.
- accept: init --code P-001 creates DOSSIER with its folders, settings and state folder; a second init is refused.
- verify: cargo test --test cmd_init
- verified: 2026-11-21
- commit: ec8defe
- reqs: R-PLACE-2, R-XOS-8
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-21, haiku-t-096): done

### T-097 · people add: register a person and their LLMs
- status: done
- owner: haiku-t-097
- priority: P0 · difficulty: 1 · type: feature
- milestone: T-095
- why: Only known names may write, and the tool must know which names are LLMs.
- depends: T-068, T-082
- files: src/cmd/people.rs, tests/cmd_people.rs
- read: docs/design.md, docs/format.md
- steps:
  1. Give Args a subcommand enum with Add { person: String, #[arg(long)] llm: Vec<String> } and List.
  2. In run for Add: find the dossier with find::find from the current folder (do not use Ctx::load, because no name is needed), refuse an empty person name or one containing a space with Code::BadValue, call config::add_person, and return 'added <person>' plus the LLM names.
  3. In run for List: load the settings and print one line per person: '<name>: <llms joined by comma>'.
  4. Create tests/cmd_people.rs with assert_cmd in a tempfile folder after dossier init: add a person with one LLM, add a second LLM later, list shows one person with both; dossier.toml still loads.
  5. Run cargo test --test cmd_people.
- accept: people add writes the person and LLM names into dossier.toml and keeps the rest of the file.
- verify: cargo test --test cmd_people
- verified: 2026-11-21
- commit: 9e5dda2
- reqs: R-CFG-4
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-21, haiku-t-097): done

### T-098 · tests/common/mod.rs: the shared test project
- status: done
- owner: haiku-t-098
- priority: P0 · difficulty: 1 · type: test
- milestone: T-095
- why: Every command test needs the same temporary project, so tests stay short and alike.
- depends: T-096, T-097
- files: tests/common/mod.rs, tests/common_check.rs
- read: docs/design.md
- steps:
  1. Create tests/common/mod.rs with #![allow(dead_code)] and the Project type of design.md section 6.
  2. Project::new: make a tempfile::TempDir; run the program (assert_cmd::Command::cargo_bin('dossier')) in it with init --code P-001, then people add anna --llm anna-claude and people add ben --llm ben-codex; assert each succeeds.
  3. command(agent, now): an assert_cmd::Command for the program (cargo_bin dossier), not yet run, with current_dir the project folder and the environment: DOSSIER_NOW = now, DOSSIER_REGISTER = <project>/.test/projects.toml, DOSSIER_TODO = stand_in('todo'), DOSSIER_SSH = stand_in('ssh'), DOSSIER_AGENT = agent when it is not empty and removed otherwise, TODO_AGENT removed. run(agent, args) is run_at(agent, '2026-11-20T10:00:00+01:00', args); run_at is command(agent, now).args(args).assert().
  4. stdout: run, assert success, return standard output as a String.
  5. read and write: files relative to the project folder; entry_file: the first file in DOSSIER/*/ whose name starts with the ID followed by -; panic with a clear message when there is none (tests may panic).
  6. stand_in(program): <project>/.test/<program>.cmd on Windows, <project>/.test/<program> elsewhere; nothing is there until a test writes it. write_stand_in(program, windows, unix): create .test, write the windows text (Windows) or the unix text (elsewhere, then mode 0o755) to stand_in(program), return the path.
  7. Create tests/common_check.rs with mod common; and two tests: Project::new() then stdout('', ['people', 'list']) contains anna; write_stand_in('todo', ...) returns a path that exists.
  8. Run cargo test --test common_check.
- accept: A test gets a ready project with two people in three lines, can add variables before running a command, and can put a stand-in todo or ssh where the program looks for it.
- verify: cargo test --test common_check
- verified: 2026-11-21
- commit: d988917
- reqs: TEST-1, TEST-2
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-21, planner): Added command, stand_in and write_stand_in (design.md section 6): run returns a finished Assert, so tests that need DOSSIER_ANSWERS, EDITOR or a stand-in todo had no way to set them.
- note (2026-11-21, haiku-t-098): done

### T-099 · add: create an entry of any kind from options, a file or standard input
- status: done
- owner: haiku-t-099
- priority: P0 · difficulty: 1 · type: feature
- milestone: T-095
- why: The general way to write an entry; the short forms are built the same way.
- depends: T-094, T-098
- files: src/cmd/add.rs, tests/cmd_add.rs
- read: docs/design.md, docs/format.md
- steps:
  1. Give Args: kind: String (positional), --title (required), --field key=value (repeatable), --section 'Name=text' (repeatable), --from <file>, --stdin, --task, --said-by, --draft.
  2. In run: load Ctx::load(g, 'add'); parse the kind with Kind::parse or fail with Code::BadValue listing the four kinds.
  3. Refuse a --field whose key is status, created, author, via, approved, valid-from, valid-until or history with Code::BadValue 'this field is written by the tool'.
  4. When --from or --stdin is given, read the text and split it into sections at lines starting with '## ' (text before the first heading is ignored); these sections come before the --section ones.
  5. Add the field task when --task is given.
  6. Call tx::write(&ctx, |tx| { let (id, _) = create::create(tx, &New{...})?; Ok(Output::with(format!('{id}: added ({status})'), json!({'id': id}))) }) where status is 'approved' or 'draft, waiting for approval'; read the status back from the created text.
  7. Create tests/cmd_add.rs with mod common: anna adds a finding with a section and the file exists with status approved; anna-claude adds a lesson and it is a draft; a text with a double quote, a backtick and a dollar sign given by --stdin arrives unchanged; an unknown kind fails with exit 1; --field status=approved fails; without a name the exit code is 2.
  8. Run cargo test --test cmd_add.
- accept: add writes an entry of any kind; tool-owned fields cannot be set; special characters survive.
- verify: cargo test --test cmd_add
- verified: 2026-11-21
- commit: ee2bd1f
- reqs: R-WRITE-1, R-HIST-4
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-21, haiku-t-099): done

### T-100 · decide: record a decision with its reason
- status: done
- owner: haiku-t-100
- priority: P0 · difficulty: 1 · type: feature
- milestone: T-095
- why: Decisions made in a session must be recorded at once, with who decided.
- depends: T-094, T-098
- files: src/cmd/decide.rs, tests/cmd_decide.rs
- read: docs/design.md, docs/format.md
- steps:
  1. Give Args: decision: String (positional), --reason (required), --rejected, --title, --task, --said-by, --replaces, --besides (flag), and the tag options --machine --analysis --software --tags.
  2. In run: Ctx::load(g, 'decide'); the title is --title or the first 80 characters of the decision (cut at a character boundary).
  3. Build New with kind Decision, sections Decision, Reason and Rejected, the tag fields that were given, task, said_by.
  4. Call tx::write and create::create as in the add command and print '<ID>: added (approved)'.
  5. Leave --replaces and --besides unused for now with a comment: the task 'decide: related decisions' implements them.
  6. Create tests/cmd_decide.rs with mod common: anna decides and the file has the sections, author anna and two history lines with the fixed time; anna-claude without --said-by fails with exit 1 and names --said-by; with --said-by anna it is approved with via anna-claude; --said-by nobody fails.
  7. Run cargo test --test cmd_decide.
- accept: decide writes an approved decision showing who decided, through which LLM, and when.
- verify: cargo test --test cmd_decide
- verified: 2026-11-21
- commit: 8e20f3d
- reqs: R-WRITE-2, R-ENTRY-5, R-HIST-5
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-21, haiku-t-100): done

### T-101 · found: record an observation, a dead end or a quick note
- status: done
- owner: haiku-t-101
- priority: P0 · difficulty: 1 · type: feature
- milestone: T-095
- why: Findings and dead ends are the facts of a project; a thought must cost ten seconds to save.
- depends: T-094, T-098
- files: src/cmd/found.rs, tests/cmd_found.rs
- read: docs/design.md, docs/format.md
- steps:
  1. Give Args: observation: String (positional), --evidence, --dead-end (flag), --later (flag), --title, --task, and the tag options.
  2. In run: Ctx::load(g, 'found'); title is --title or the first 80 characters of the observation.
  3. Build New with kind Finding, sections Observation and Evidence, field outcome = 'dead end' when --dead-end, task, the tag fields; with --later set draft true and add inbox to the tags field.
  4. Call tx::write and create::create; print '<ID>: added (approved)' or '<ID>: added (draft, sort it out later)'.
  5. Create tests/cmd_found.rs with mod common: anna-claude records a finding and it is approved with via anna-claude; --dead-end writes outcome: dead end; --later gives status draft and tags containing inbox.
  6. Run cargo test --test cmd_found.
- accept: found writes approved findings by people and LLMs; --dead-end and --later behave as described.
- verify: cargo test --test cmd_found
- verified: 2026-11-21
- commit: 1ac6e66
- reqs: R-WRITE-3, R-WRITE-7, R-WRITE-16, R-ENTRY-7, R-ENTRY-15
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-21, haiku-t-101): done

### T-102 · learn: record a lesson with its conditions and evidence
- status: done
- owner: haiku-t-102
- priority: P0 · difficulty: 1 · type: feature
- milestone: T-095
- why: A lesson is what reaches later projects, so it must carry where it holds and how sure it is.
- depends: T-094, T-098
- files: src/cmd/learn.rs, tests/cmd_learn.rs
- read: docs/design.md, docs/format.md
- steps:
  1. Give Args: statement: String (positional), --valid-for (required), --evidence (required), --confidence (required), --title, --task, --said-by, --update, and the tag options.
  2. In run: Ctx::load(g, 'learn'); refuse a confidence other than measured, simulated, experience or guess with Code::BadValue listing the four.
  3. Build New with kind Lesson, sections Statement, Valid for and Evidence, field confidence, task, tag fields, said_by.
  4. Call tx::write and create::create; print '<ID>: added (approved)' for a person and '<ID>: added (draft, waiting for approval)' for an LLM.
  5. Leave --update unused with a comment: the task 'learn: similar entries' implements it.
  6. Create tests/cmd_learn.rs with mod common: anna's lesson is approved; anna-claude's lesson is a draft with author anna and via anna-claude; a wrong confidence fails with exit 1.
  7. Run cargo test --test cmd_learn.
- accept: A person's lesson is approved, an LLM's is a draft; confidence is checked.
- verify: cargo test --test cmd_learn
- verified: 2026-11-21
- commit: 4c04083
- reqs: R-WRITE-4, R-ENTRY-6, D5
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-21, haiku-t-102): done

### T-103 · assume: record an assumption
- status: done
- owner: haiku-t-103
- priority: P0 · difficulty: 1 · type: feature
- milestone: T-095
- why: What had to be assumed must be visible until someone confirms it.
- depends: T-094, T-098
- files: src/cmd/assume.rs, tests/cmd_assume.rs
- read: docs/design.md, docs/format.md
- steps:
  1. Give Args: assumption: String (positional), --reason (required), --affects, --title, --task, and the tag options.
  2. In run: Ctx::load(g, 'assume'); title is --title or the first 80 characters.
  3. Build New with kind Assumption, field state = open, sections Assumption, Reason and Affects, task, tag fields.
  4. Call tx::write and create::create; print '<ID>: added (approved, open)'.
  5. Create tests/cmd_assume.rs with mod common: anna-claude records an assumption; the file has state: open, status: approved and the three sections.
  6. Run cargo test --test cmd_assume.
- accept: assume writes an approved assumption with state open.
- verify: cargo test --test cmd_assume
- verified: 2026-11-21
- commit: 8920a5a
- reqs: R-WRITE-14, R-ENTRY-13
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-21, haiku-t-103): done

### T-104 · set: change fields and sections of an entry
- status: done
- owner: haiku-t-104
- priority: P1 · difficulty: 1 · type: feature
- milestone: T-095
- why: Entries need correcting without an editor, and an LLM's change to approved knowledge must not stay approved.
- depends: T-093, T-085, T-092, T-098, T-099, T-102
- files: src/cmd/set.rs, tests/cmd_set.rs
- read: docs/design.md, docs/format.md, docs/requirements.md
- steps:
  1. Give Args: id: String, --field key=value (repeatable), --section 'Name=text' (repeatable).
  2. In run: Ctx::load(g, 'set'); refuse the tool-owned field keys as in the add command; refuse when nothing is given with Code::Usage.
  3. Inside tx::write: load the item with store::get; clone its entry; apply each set_field and set_section; add a history line 'changed' with the detail listing what was changed ('field machine, section Reason').
  4. If the writer is an LLM and the entry is an approved lesson: set status to draft, remove the fields approved and valid-from, add the detail 'back to draft', and call approval::forget.
  5. If the writer is a person and the entry is an approved lesson, call approval::record with the new text.
  6. Queue with tx.put(path, Some(&item), text, Change{event 'changed'}) and print '<ID>: changed' or '<ID>: changed (back to draft, waiting for approval)'.
  7. Create tests/cmd_set.rs with mod common: anna changes a field of a finding and only that line plus one history line differ; anna-claude changes an approved lesson and it becomes a draft; --field history=x fails; snapshot 1 exists in DOSSIER/.state/history/<ID>/.
  8. Run cargo test --test cmd_set.
- accept: set changes only the named parts, adds a history line, and turns an approved lesson into a draft when an LLM changed it.
- verify: cargo test --test cmd_set
- verified: 2026-11-21
- commit: 97d099c
- reqs: R-WRITE-8, R-EDIT-1, R-HIST-5
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-21, haiku-t-104): done

### T-105 · retitle: change a title and rename the file
- status: done
- owner: haiku-t-105
- priority: P1 · difficulty: 1 · type: feature
- milestone: T-095
- why: Titles are what people search and read; fixing one must keep the ID.
- depends: T-093, T-085, T-098, T-099
- files: src/cmd/retitle.rs, tests/cmd_retitle.rs
- read: docs/design.md, docs/format.md
- steps:
  1. Give Args: id: String, title: String.
  2. In run: Ctx::load(g, 'retitle'); inside tx::write load the item, clone the entry, set_title, add a history line 'changed' with detail 'title'.
  3. The new path is store::path_for with the entry's kind, ID and new title; when it equals the old path use tx.put, else tx.rename.
  4. Print '<ID>: retitled'.
  5. Create tests/cmd_retitle.rs with mod common: after retitle the old file is gone, a file with the new slug exists, the ID in the heading is unchanged and the history has a changed line.
  6. Run cargo test --test cmd_retitle.
- accept: retitle renames the file to the new slug and keeps the ID.
- verify: cargo test --test cmd_retitle
- verified: 2026-11-21
- commit: bffe643
- reqs: R-WRITE-8, R-ID-3
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-21, haiku-t-105): done

### T-106 · link: relate two entries
- status: done
- owner: haiku-t-106
- priority: P1 · difficulty: 1 · type: feature
- milestone: T-095
- why: Related entries should lead to each other.
- depends: T-093, T-085, T-098, T-099
- files: src/cmd/link.rs, tests/cmd_link.rs
- read: docs/design.md, docs/format.md
- steps:
  1. Give Args: a: String, b: String.
  2. In run: Ctx::load(g, 'link'); refuse two equal IDs with Code::BadValue.
  3. Inside tx::write load both items; for each, add the other's ID to the list of the related field if it is not there (comma and space separated) and add a history line 'changed' with detail 'related <other ID>'; queue both with tx.put.
  4. When both already name each other, queue nothing and print 'already linked'; else print '<A> and <B>: linked'.
  5. Create tests/cmd_link.rs with mod common: after link both files have the other's ID in related; a second link changes nothing; linking an entry to itself fails.
  6. Run cargo test --test cmd_link.
- accept: link adds each entry to the other's related field once.
- verify: cargo test --test cmd_link
- verified: 2026-11-21
- commit: 98b54cd
- reqs: R-WRITE-9
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-21, haiku-t-106): done

### T-107 · edit: open an entry in an editor under the lock
- status: done
- owner: haiku-t-107
- priority: P2 · difficulty: 1 · type: feature
- milestone: T-095
- why: Longer entries are easier to write in an editor than in options.
- depends: T-104
- files: src/cmd/edit.rs, tests/cmd_edit.rs
- read: docs/design.md, docs/requirements.md
- steps:
  1. Give Args: id: String.
  2. In run: Ctx::load(g, 'edit'); the editor is the environment variable VISUAL, else EDITOR, else notepad on Windows and vi elsewhere.
  3. Inside tx::write: load the item; write its text to a temporary file named '<ID>-<process id>.md' (tests run in parallel) in std::env::temp_dir(); run the editor on it and wait; read the file back and delete it.
  4. If the text is unchanged print 'no change' and queue nothing.
  5. Parse the new text with Entry::parse (an error is returned and nothing is written); refuse a changed ID, or changed status, author, via, created, approved or history lines, with Code::Refused 'these lines are written by the tool'.
  6. Add a history line 'changed' with detail 'edited'; apply the same rule as set for an approved lesson changed by an LLM or a person; queue with tx.put; print '<ID>: changed'.
  7. Create tests/cmd_edit.rs with mod common: write an editor with project.write_stand_in('editor', ...) that appends a line to the file it is given, and run with project.command(...) plus the variable EDITOR set to that path; the entry gains that line and a history line; an editor that changes the status line is refused.
  8. Run cargo test --test cmd_edit.
- accept: edit applies an editor's change under the lock and refuses changes to tool-owned lines.
- verify: cargo test --test cmd_edit
- verified: 2026-11-21
- commit: 5236d87
- reqs: R-EDIT-2
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-21, planner): Test uses Project::command or write_stand_in from tests/common (T-098).
- note (2026-11-21, planner): Temporary file named with the process ID: parallel tests editing the same ID would share one file.
- note (2026-11-21, haiku-t-107): done

### T-108 · Reading: find and read what was recorded, and who did what when
- status: done
- owner: coordinator
- priority: P0 · type: milestone
- milestone: T-001
- goal: where, show, list, search, history, diff, restore and log work as docs/design.md section 5 describes.
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-21, coordinator): done

### T-109 · where: the paths in use
- status: done
- owner: haiku-t-109
- priority: P1 · difficulty: 1 · type: feature
- milestone: T-108
- why: People and LLMs need to see which dossier a command would work on.
- depends: T-082, T-098
- files: src/cmd/where_cmd.rs, tests/cmd_where.rs
- read: docs/design.md
- steps:
  1. Args stays empty.
  2. In run: Ctx::load(g, 'where'); print four lines: 'project: <root>', 'dossier: <dossier>', 'state: <state>', 'code: <cfg.code>'; put the same four values into the JSON data.
  3. Create tests/cmd_where.rs with mod common: the output contains the code P-001 and the word DOSSIER; run in a subfolder of the project gives the same dossier line; in an empty temporary folder the exit code is 4.
  4. Run cargo test --test cmd_where.
- accept: where prints the project folder, dossier, state folder and code from any subfolder.
- verify: cargo test --test cmd_where
- verified: 2026-11-21
- commit: 70e5ad7
- reqs: R-PLACE-7
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-21, haiku-t-109): done

### T-110 · filter.rs: the filter options shared by list and search
- status: done
- owner: haiku-t-110
- priority: P0 · difficulty: 1 · type: feature
- milestone: T-108
- why: list and search must filter the same way.
- depends: T-088
- files: src/filter.rs
- read: docs/design.md, docs/format.md
- steps:
  1. Define Filter with clap options: --kind, --status, --machine, --analysis, --software, --tag, --author, --task, --since, --where (repeatable, key=value), --drafts (flag), --all (flag); every option is optional.
  2. Implement matches(item): kind compares with Kind::parse; status compares names; machine, analysis, software and tag match when any element of the entry's list (tag uses the tags field) equals the value ignoring case; author and task compare the field ignoring case; since matches when the created date (first 10 characters) is not before the value; each --where matches when the field equals the value ignoring case.
  3. Status rule when --status is not given: only approved entries match; with --drafts also drafts; with --all every status.
  4. Add tests with entries built by entry::new::render: each option alone, the default status rule, --drafts, --all, and two options together.
  5. Run cargo test --lib filter::.
- accept: Each filter option narrows the entries as described; approved only by default.
- verify: cargo test --lib filter::
- verified: 2026-11-21
- commit: 57935bd
- reqs: R-READ-2, R-DRAFT-6
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-21, haiku-t-110): done

### T-111 · show: print one entry with its marks
- status: done
- owner: haiku-t-111
- priority: P0 · difficulty: 1 · type: feature
- milestone: T-108
- why: Reading an entry back, with who wrote and approved it and when, is the point of writing it.
- depends: T-088, T-092, T-082, T-098, T-099
- files: src/cmd/show.rs, tests/cmd_show.rs
- read: docs/design.md, docs/format.md
- steps:
  1. Give Args: id: String.
  2. In run: Ctx::load(g, 'show'); store::get the item.
  3. Build the marks of design.md section 5 from the status, the superseded-by field and approval::mark (Changed gives '[changed after approval]'); when there are marks print them as a first line.
  4. Then print the file text unchanged.
  5. For JSON return an object with id, title, kind, status, marks (list), fields (object; history as a list of strings), sections (object) and path (relative to the project folder with /).
  6. Create tests/cmd_show.rs with mod common: show of an approved finding prints the file text; a draft lesson starts with [draft]; --json has the keys above and the history list has one line for a draft and two for an approved entry; an approved lesson whose file the test changes by hand shows [changed after approval]; an unknown ID gives exit 1.
  7. Run cargo test --test cmd_show.
- accept: show prints the entry as stored, with marks, and --json returns every field, section and history line.
- verify: cargo test --test cmd_show
- verified: 2026-11-21
- commit: d4e2938
- reqs: R-READ-1, R-LIFE-6, R-HIST-6
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-21, haiku-t-111): done

### T-112 · list: one line per entry
- status: done
- owner: haiku-t-112
- priority: P0 · difficulty: 1 · type: feature
- milestone: T-108
- why: An overview of what is recorded, narrowed by kind, tag, author or task.
- depends: T-110, T-082, T-098, T-099, T-102
- files: src/cmd/list.rs, tests/cmd_list.rs
- read: docs/design.md
- steps:
  1. Give Args: #[command(flatten)] filter: crate::filter::Filter and --wide (flag).
  2. In run: Ctx::load(g, 'list'); store::all; keep the items the filter matches; pass the warnings of store::all on to the Output.
  3. Print per entry: ID, kind name, a mark for a status other than approved (as in design.md section 5), and the title; with --wide add 'created <time> by <author>' and, when approved, 'approved <approved field>'.
  4. End with a line '<n> entries'.
  5. JSON data: a list of objects with id, kind, status, title, author, created, approved.
  6. Create tests/cmd_list.rs with mod common: with one approved finding and one draft lesson, list shows one entry, list --drafts two, list --kind lesson --all one; --wide contains the fixed time and the author; a broken .md file placed in DOSSIER/findings gives a warning and the list still works.
  7. Run cargo test --test cmd_list.
- accept: list shows approved entries by default, obeys the filters, and --wide shows who and when.
- verify: cargo test --test cmd_list
- verified: 2026-11-21
- commit: 5435d47
- reqs: R-READ-2, R-READ-5, R-HIST-6
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-21, haiku-t-112): done

### T-113 · select.rs: score entries against tags and words
- status: done
- owner: haiku-t-113
- priority: P0 · difficulty: 1 · type: feature
- milestone: T-108
- why: brief, search and the similar-entry hint must rank entries the same way.
- depends: T-088
- files: src/select.rs
- read: docs/design.md
- steps:
  1. Implement words(text) as design.md says; remove duplicates and keep the first order.
  2. Implement score(item, query) with the points of design.md: title words via words(title); tag fields are machine, analysis, software and tags, compared ignoring case; section text via words of all section bodies joined.
  3. Implement best(items, query, limit): score each, drop zeros, sort by score descending, then created descending, then ID ascending, cut to limit.
  4. Add tests with entries built by entry::new::render: a title match outranks a text match; a tag match scores 3; the order is the same when the input order is reversed; limit cuts the list; an empty query gives nothing.
  5. Run cargo test --lib select::.
- accept: Scores follow the design; the order is deterministic.
- verify: cargo test --lib select::
- verified: 2026-11-21
- commit: ac58ced
- reqs: R-READ-3, R-CTX-2, TEST-9
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-21, haiku-t-113): done

### T-114 · search: find entries by words
- status: done
- owner: haiku-t-114
- priority: P0 · difficulty: 1 · type: feature
- milestone: T-108
- why: Retrieval by words is how a person finds what the team knew.
- depends: T-110, T-113, T-082, T-098, T-099, T-102
- files: src/cmd/search.rs, tests/cmd_search.rs
- read: docs/design.md
- steps:
  1. Give Args: words: Vec<String> (at least one), --full (flag), #[command(flatten)] filter.
  2. In run: Ctx::load(g, 'search'); store::all; keep items the filter matches and whose whole text (lower case) contains every word (lower case).
  3. Order: entries with every word in the title first, then entries with a word in a tag field, then the rest; inside each group newest created first, then ID.
  4. Print per result: ID, mark for a non-approved status, title, and the first line of the file that contains the first word (trimmed); with --full print the whole file text instead, results separated by a line of three dashes.
  5. End with '<n> matches'. JSON data: list of id, title, status, line.
  6. Create tests/cmd_search.rs with mod common: a word in a title is found; two words must both be present; a draft is found only with --drafts and is marked [draft]; --kind narrows; no match prints '0 matches' with exit 0.
  7. Run cargo test --test cmd_search.
- accept: search finds entries containing all words, title matches first, drafts only on request.
- verify: cargo test --test cmd_search
- verified: 2026-11-21
- commit: deaf6a3
- reqs: R-READ-3, R-READ-4, R-DRAFT-6
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-21, haiku-t-114): done

### T-115 · history: what happened to an entry or a task
- status: done
- owner: haiku-t-115
- priority: P1 · difficulty: 1 · type: feature
- milestone: T-108
- why: Why an entry looks the way it does must be answerable from its history.
- depends: T-090, T-091, T-082, T-098, T-104
- files: src/cmd/history.rs, tests/cmd_history.rs
- read: docs/design.md, docs/format.md
- steps:
  1. Give Args: target: String.
  2. In run: Ctx::load(g, 'history'); if the target looks like an entry ID (Kind::of_id is Some) use store::get and print its history lines one per line, then a blank line and 'snapshots:' with one line per snapshot::list item: 'n <n>  <time>  <name>  <command>'.
  3. Otherwise treat it as a task: print one line per entry whose task field equals it (ignoring case): '<created>  <author>  <ID>  <title>', oldest first, or 'nothing recorded for <task>'.
  4. JSON data: for an entry {id, history: [...], snapshots: [...]}; for a task {task, entries: [...]}.
  5. Create tests/cmd_history.rs with mod common: after add and set, history shows created, approved and changed lines and one snapshot; history T-042 lists the entries added with --task T-042.
  6. Run cargo test --test cmd_history.
- accept: history shows an entry's events and snapshots, or the entries of a task in time order.
- verify: cargo test --test cmd_history
- verified: 2026-11-21
- commit: e181a08
- reqs: R-READ-6, R-HIST-2
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-21, haiku-t-115): done

### T-116 · diff: what changed since a snapshot
- status: done
- owner: haiku-t-116
- priority: P2 · difficulty: 1 · type: feature
- milestone: T-108
- why: Before restoring or approving again, a person needs to see what changed.
- depends: T-091, T-082, T-098, T-104
- files: src/cmd/diff.rs, tests/cmd_diff.rs
- read: docs/design.md
- steps:
  1. Give Args: id: String, n: Option<u32>.
  2. In run: Ctx::load(g, 'diff'); load the item; take the snapshot number n, or the highest of snapshot::list, or fail with Code::NotFound 'no snapshots of <ID>' when there is none.
  3. Compare the snapshot text and the current text line by line with a simple method: print lines only in the snapshot with a leading '- ', lines only in the current file with '+ ', in the order of the files; lines present in both are not printed.
  4. Print 'no difference' when nothing differs.
  5. Create tests/cmd_diff.rs with mod common: after set --field machine=x, diff shows a + line with machine and the history line; diff on an entry without snapshots gives exit 1.
  6. Run cargo test --test cmd_diff.
- accept: diff shows the lines removed and added since a snapshot.
- verify: cargo test --test cmd_diff
- verified: 2026-11-21
- commit: 2b48082
- reqs: R-HIST-2
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-21, haiku-t-116): done

### T-117 · restore: bring a snapshot back
- status: done
- owner: haiku-t-117
- priority: P2 · difficulty: 1 · type: feature
- milestone: T-108
- why: A wrong change must be undoable by a person.
- depends: T-093, T-091, T-081, T-085, T-098, T-104
- files: src/cmd/restore.rs, tests/cmd_restore.rs
- read: docs/design.md
- steps:
  1. Give Args: id: String, n: u32.
  2. In run: Ctx::load(g, 'restore'); the writer must be a person (Code::NeedsPerson otherwise) and term::is_terminal() must be true (Code::NeedsTerminal otherwise).
  3. Ask with term::ask 'restore <ID> to snapshot <n>? (yes/no) ' and stop with 'nothing changed' unless the answer is yes.
  4. Inside tx::write: load the item and the snapshot bytes; parse the snapshot text; keep the history lines of the current entry by copying into the restored entry every history line the snapshot does not have; add a history line 'restored' with detail 'snapshot <n>'; queue with tx.put.
  5. Print '<ID>: restored to snapshot <n>'.
  6. Create tests/cmd_restore.rs with mod common, running restore through project.command(...) with the variable DOSSIER_ANSWERS=yes added: after set and restore 1 the field has its old value and the history has changed and restored lines; run as anna-claude fails with exit 1.
  7. Run cargo test --test cmd_restore.
- accept: A person at a terminal can bring back a snapshot; the history keeps every earlier line and gains a restored line.
- verify: cargo test --test cmd_restore
- verified: 2026-11-21
- commit: f692be2
- reqs: R-HIST-2, R-HIST-5
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-21, planner): Test uses Project::command or write_stand_in from tests/common (T-098).
- note (2026-11-21, haiku-t-117): done

### T-118 · log: who did what when
- status: done
- owner: haiku-t-118
- priority: P0 · difficulty: 1 · type: feature
- milestone: T-108
- why: The owner wants to see what was decided by whom and when, across the whole dossier.
- depends: T-090, T-082, T-098, T-099, T-100, T-101
- files: src/cmd/log.rs, tests/cmd_log.rs
- read: docs/design.md, docs/format.md
- steps:
  1. Give Args: --kind, --person, --event, --since <date>, --until <date>, --limit (default 50).
  2. In run: Ctx::load(g, 'log'); journal::read; build a map of ID to title from store::all.
  3. Keep records that match: kind by the prefix of the record's ID (Kind::parse of the option, compared with Kind::of_id); person ignoring case; event exact; since and until compare the date part (first 10 characters), both inclusive.
  4. Print newest first, at most limit lines: '<time>  <person>[ via <name> when llm]  <event>  <ID>  <title or (removed)>'.
  5. End with '<n> events' and, when more matched than the limit, '(<m> more; use --limit)'.
  6. JSON data: the list of records with the title added.
  7. Create tests/cmd_log.rs with mod common, using run_at with different times: after a decision by anna and a finding by ben-codex, log shows two lines newest first with times, persons and 'via ben-codex'; --person anna shows one; --kind finding shows one; --since with a later date shows none.
  8. Run cargo test --test cmd_log.
- accept: log prints each event with time, person, LLM, entry and title, newest first, and can be narrowed by kind, person, event and date.
- verify: cargo test --test cmd_log
- verified: 2026-11-21
- commit: 9aea7dc
- reqs: R-HIST-3, R-HIST-4, R-HIST-6
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-21, planner): Depends also on the commands its test uses to set up entries.
- note (2026-11-21, haiku-t-118): done

### T-119 · Approval and life: a person decides what counts as knowledge
- status: done
- owner: coordinator
- priority: P0 · type: milestone
- milestone: T-001
- goal: review-task, drafts, approve, reject, supersede, retire, remove, reopen and check work as docs/design.md section 5 describes; an LLM can never approve.
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-21, coordinator): done

### T-120 · todo_bridge.rs: read tasks from todo
- status: done
- owner: haiku-t-120
- priority: P0 · difficulty: 1 · type: feature
- milestone: T-119
- why: dossier reads tasks through todo --json and shares no code with it.
- depends: T-071
- files: src/todo_bridge.rs
- read: docs/design.md, docs/requirements.md
- steps:
  1. Write a private fn program() -> String: the environment variable DOSSIER_TODO, or 'todo'.
  2. Implement available(): run the program with --version and return true when it starts and exits with 0.
  3. Implement json(root, args): run the program with the args plus --json in the folder root, capturing standard output; when the program cannot be started return Code::TodoMissing with the message 'todo is not installed or not on PATH; dossier needs todo 0.8 or newer'.
  4. Parse standard output as JSON; when ok is true return the data value; when ok is false return Code::Refused with error.message; when it is not JSON return Code::TodoMissing with the first 200 characters of the output.
  5. Add tests that write a stand-in program into a temporary folder (a .cmd file on Windows, a shell script with the executable bit elsewhere) printing a fixed JSON envelope, point DOSSIER_TODO at it inside one test that holds crate::ENV_LOCK and covers all cases, and check: data is returned; an ok:false envelope gives Code::Refused; a missing program gives Code::TodoMissing.
  6. Run cargo test --lib todo_bridge::.
- accept: json returns the data of todo's envelope and turns a missing todo into TODO_MISSING.
- verify: cargo test --lib todo_bridge::
- verified: 2026-11-21
- commit: c43d59d
- reqs: D1, TEST-2
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-21, planner): Test holds crate::ENV_LOCK (see T-067): unit tests run in parallel threads and share the environment.
- note (2026-11-21, haiku-t-120): done

### T-121 · review-task: the material for drafting lessons
- status: done
- owner: haiku-t-121
- priority: P0 · difficulty: 1 · type: feature
- milestone: T-119
- why: The LLM in the session drafts lessons from this text; the tool itself never calls a model.
- depends: T-120, T-082, T-098, T-101
- files: src/cmd/review_task.rs, tests/cmd_review_task.rs
- read: docs/design.md, docs/requirements.md
- steps:
  1. Give Args: task: String.
  2. In run: Ctx::load(g, 'review-task'); call todo_bridge::json(&ctx.root, ['show', &task]) and pass an error on.
  3. Print a heading '# Review of <task>', then the task data: its title and every string field of the returned object as 'key: value' lines, and its notes, each as '<date> <agent>: <text>' (notes is a list of objects with date, agent and text).
  4. Print '## Entries recorded for this task' and one line per entry whose task field equals the task: ID, kind, status mark, title; or 'none'.
  5. Print '## Drafting rules' and the rules of requirement R-DRAFT-2 as a numbered list (copy them from docs/requirements.md, one rule per line), then the two example commands: dossier learn ... and dossier found --dead-end ....
  6. Change nothing on disk.
  7. Create tests/cmd_review_task.rs with mod common: write a stand-in todo with project.write_stand_in('todo', ...) that prints a todo.show.v1 envelope; the output contains the task title from the stand-in, an entry added with --task, and the words Drafting rules; without the stand-in the exit code is 5.
  8. Run cargo test --test cmd_review_task.
- accept: review-task prints the task, its entries and the drafting rules, changes nothing, and fails with exit 5 when todo is missing.
- verify: cargo test --test cmd_review_task
- verified: 2026-11-21
- commit: aa95973
- reqs: R-DRAFT-1, R-DRAFT-2, D7
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-21, planner): Test uses Project::command or write_stand_in from tests/common (T-098).
- note (2026-11-21, haiku-t-121): done

### T-122 · drafts: what is waiting for approval
- status: done
- owner: haiku-t-122
- priority: P0 · difficulty: 1 · type: feature
- milestone: T-119
- why: A person must see in one list what their LLM proposed.
- depends: T-088, T-073, T-082, T-098, T-102
- files: src/cmd/drafts.rs, tests/cmd_drafts.rs
- read: docs/design.md
- steps:
  1. Give Args: --mine (flag).
  2. In run: Ctx::load(g, 'drafts'); take the entries with status draft; with --mine keep those whose author is the writer's person (ctx.writer()?).
  3. Sort oldest created first; print per draft: ID, kind, title, 'by <author>' plus ' via <via>' when present, 'task <task>' when present, and the age in days from time::days_between(created, time::now()); mark drafts older than cfg.draft_days with '(old)'.
  4. End with '<n> drafts'. JSON data: list of id, kind, title, author, via, task, created, age_days.
  5. Create tests/cmd_drafts.rs with mod common: two lessons by anna-claude and ben-codex at different times (run_at) are listed oldest first; --mine as anna shows one; a draft created 20 days before the run time is marked (old).
  6. Run cargo test --test cmd_drafts.
- accept: drafts lists waiting entries oldest first with proposer, task and age.
- verify: cargo test --test cmd_drafts
- verified: 2026-11-21
- commit: 5197207
- reqs: R-DRAFT-3, R-DRAFT-8
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-21, haiku-t-122): done

### T-123 · approve: a person at a terminal approves drafts
- status: done
- owner: haiku-t-123
- priority: P0 · difficulty: 1 · type: feature
- milestone: T-119
- why: This is the step that keeps wrong LLM conclusions out of the team's knowledge.
- depends: T-093, T-081, T-087, T-092, T-085, T-098, T-102, T-099
- files: src/cmd/approve.rs, tests/cmd_approve.rs
- read: docs/design.md, docs/format.md, docs/requirements.md
- steps:
  1. Give Args: ids: Vec<String> (at least one).
  2. In run: Ctx::load(g, 'approve'); the writer must be a person, else Code::NeedsPerson with the message 'only a person can approve; <name> is an LLM'; term::is_terminal() must be true, else Code::NeedsTerminal.
  3. Inside tx::write, for each ID: load the item; skip with a warning when its status is not draft; print its whole text to standard error; when rules::problems is not empty print them and skip with a warning 'cannot be approved yet'.
  4. Ask with term::ask '<ID>: approve, reject or skip? (a/r/s) '; a or approve: set status approved, set approved '<time> by <person>', set valid-from, add a history line 'approved', queue with tx.put, and for a lesson call approval::record; r or reject: set status rejected, add a history line 'rejected', queue; anything else: skip.
  5. Return an Output with one line per ID: '<ID>: approved', '<ID>: rejected' or '<ID>: skipped'.
  6. Create tests/cmd_approve.rs with mod common (run approve through project.command(...) with DOSSIER_ANSWERS added): anna approves a draft lesson with answer a and the file has status approved, an approved line with the fixed time and a history line 'anna: approved'; answer r rejects; anna-claude gets exit 1; without DOSSIER_ANSWERS (no terminal in tests) anna gets exit 1 with NEEDS_TERMINAL in --json; a lesson without Valid for is skipped.
  7. Run cargo test --test cmd_approve.
- accept: Only a person at a terminal approves; the entry records who approved and when; incomplete drafts are not approved.
- verify: cargo test --test cmd_approve
- verified: 2026-11-21
- commit: d2dbb58
- reqs: R-DRAFT-4, D5, D6, R-HIST-5, TEST-5
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-21, planner): Test uses Project::command or write_stand_in from tests/common (T-098).
- note (2026-11-21, planner): Depends also on the commands its test uses to set up entries.
- note (2026-11-21, haiku-t-123): done

### T-124 · reject: turn a draft down
- status: done
- owner: haiku-t-124
- priority: P1 · difficulty: 1 · type: feature
- milestone: T-119
- why: A rejected proposal must be kept so the same one is recognised next time.
- depends: T-093, T-085, T-098, T-102
- files: src/cmd/reject.rs, tests/cmd_reject.rs
- read: docs/design.md, docs/format.md
- steps:
  1. Give Args: id: String, --reason.
  2. In run: Ctx::load(g, 'reject'); the writer must be a person (Code::NeedsPerson).
  3. Inside tx::write: load the item; refuse a status other than draft with Code::Refused 'only a draft can be rejected'; set status rejected; set the field rejected-reason when a reason is given; add a history line 'rejected' with the reason as detail; queue with tx.put.
  4. Print '<ID>: rejected'.
  5. Create tests/cmd_reject.rs with mod common: anna rejects a draft and the file has status rejected and the reason; rejecting an approved entry fails; anna-claude fails.
  6. Run cargo test --test cmd_reject.
- accept: A person can reject a draft with a reason; the entry is kept with status rejected.
- verify: cargo test --test cmd_reject
- verified: 2026-11-21
- commit: 1574827
- reqs: R-DRAFT-5
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-21, haiku-t-124): done

### T-125 · supersede: replace an entry with a newer one
- status: done
- owner: haiku-t-125
- priority: P0 · difficulty: 1 · type: feature
- milestone: T-119
- why: Knowledge is never overwritten; the old entry is closed with a date and points to its replacement.
- depends: T-093, T-085, T-098, T-100, T-099
- files: src/cmd/supersede.rs, tests/cmd_supersede.rs
- read: docs/design.md, docs/format.md
- steps:
  1. Give Args: old: String, --by (required).
  2. In run: Ctx::load(g, 'supersede'); the writer must be a person (Code::NeedsPerson).
  3. Inside tx::write: load both items; refuse when the old entry is not approved or the new one is not approved (Code::Refused naming which), or when they are the same entry, or of different kinds.
  4. Old entry: status superseded, superseded-by <new ID>, valid-until <time>, history line 'superseded' with detail 'by <new ID>'. New entry: supersedes <old ID>, history line 'changed' with detail 'supersedes <old ID>'. Queue both.
  5. Print '<old ID>: superseded by <new ID>'.
  6. Create tests/cmd_supersede.rs with mod common: after two decisions and supersede, the old file has status superseded, superseded-by and valid-until with the fixed time, the new one has supersedes; superseding with a draft fails; anna-claude fails.
  7. Run cargo test --test cmd_supersede.
- accept: The old entry is closed with a date and both entries point to each other.
- verify: cargo test --test cmd_supersede
- verified: 2026-11-21
- commit: 2d2440f
- reqs: R-LIFE-4, R-ENTRY-14, D20
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-21, planner): Depends also on the commands its test uses to set up entries.
- note (2026-11-21, haiku-t-125): done

### T-126 · retire: end an entry without a replacement
- status: done
- owner: haiku-t-126
- priority: P1 · difficulty: 1 · type: feature
- milestone: T-119
- why: An entry that no longer holds must stop being shown as knowledge.
- depends: T-093, T-085, T-098, T-101
- files: src/cmd/retire.rs, tests/cmd_retire.rs
- read: docs/design.md, docs/format.md
- steps:
  1. Give Args: id: String, --reason (required).
  2. In run: Ctx::load(g, 'retire'); the writer must be a person (Code::NeedsPerson).
  3. Inside tx::write: load the item; refuse a status other than approved with Code::Refused; set status retired, retired-reason, valid-until <time>, history line 'retired' with the reason as detail; queue.
  4. Print '<ID>: retired'.
  5. Create tests/cmd_retire.rs with mod common: after retire the file has status retired, the reason and valid-until; retiring a draft fails; anna-claude fails.
  6. Run cargo test --test cmd_retire.
- accept: A person can retire an approved entry with a reason; it is closed with a date.
- verify: cargo test --test cmd_retire
- verified: 2026-11-21
- commit: c2b1c22
- reqs: R-LIFE-5, R-ENTRY-14
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-21, haiku-t-126): done

### T-127 · remove: delete an entry for good
- status: done
- owner: haiku-t-127
- priority: P2 · difficulty: 1 · type: feature
- milestone: T-119
- why: Customer data entered by mistake must be removable, by a person only.
- depends: T-093, T-081, T-092, T-098, T-101
- files: src/cmd/remove.rs, tests/cmd_remove.rs
- read: docs/design.md, docs/requirements.md
- steps:
  1. Give Args: id: String, --yes (flag).
  2. In run: Ctx::load(g, 'remove'); the writer must be a person (Code::NeedsPerson); term::is_terminal() must be true (Code::NeedsTerminal); without --yes fail with Code::Usage 'remove deletes the entry for good; add --yes'.
  3. Ask with term::ask 'type the ID to delete <ID> for good: ' and stop with 'nothing changed' unless the answer equals the ID ignoring case.
  4. Inside tx::write: load the item; queue tx.delete with Change{event 'removed', detail ''} (the journal line holds no text of the entry); call approval::forget.
  5. After the write, delete the folder DOSSIER/.state/history/<ID> if it exists, so that no copy of the text stays.
  6. Print '<ID>: removed'.
  7. Create tests/cmd_remove.rs with mod common (run remove through project.command(...) with DOSSIER_ANSWERS set to the ID): the file is gone, the journal has a removed line, the snapshot folder is gone; the next added finding gets a higher number, not the removed one; anna-claude fails.
  8. Run cargo test --test cmd_remove.
- accept: Only a person at a terminal can remove an entry; no copy of its text remains; its ID is not reused.
- verify: cargo test --test cmd_remove
- verified: 2026-11-21
- commit: 97ee4b7
- reqs: R-LIFE-3, R-ID-3
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-21, planner): Test uses Project::command or write_stand_in from tests/common (T-098).
- note (2026-11-21, haiku-t-127): done

### T-128 · reopen: back to draft, or a retired entry back in force
- status: done
- owner: haiku-t-128
- priority: P2 · difficulty: 1 · type: feature
- milestone: T-119
- why: A rejection or an approval can turn out to be wrong.
- depends: T-093, T-085, T-092, T-098, T-102, T-101, T-124, T-126
- files: src/cmd/reopen.rs, tests/cmd_reopen.rs
- read: docs/design.md, docs/requirements.md
- steps:
  1. Give Args: id: String.
  2. In run: Ctx::load(g, 'reopen'); inside tx::write load the item and act on its status.
  3. approved or rejected: set status draft; remove the fields approved, valid-from and rejected-reason; call approval::forget; history line 'reopened' with detail 'back to draft'.
  4. retired: the writer must be a person (Code::NeedsPerson); set status approved; remove valid-until and retired-reason; history line 'reopened' with detail 'in force again'; for a lesson call approval::record.
  5. draft or superseded: fail with Code::Refused 'a <status> entry cannot be reopened'.
  6. Queue with tx.put and print '<ID>: reopened (<new status>)'.
  7. Create tests/cmd_reopen.rs with mod common: a rejected lesson becomes a draft; a retired finding becomes approved again when anna runs it and fails for anna-claude; a draft fails.
  8. Run cargo test --test cmd_reopen.
- accept: The status table of requirement R-LIFE-1 holds for reopen.
- verify: cargo test --test cmd_reopen
- verified: 2026-11-21
- commit: 9106d8e
- reqs: R-LIFE-1
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-21, planner): Depends also on the commands its test uses to set up entries.
- note (2026-11-21, haiku-t-128): done

### T-129 · check: report problems in a dossier
- status: done
- owner: haiku-t-129
- priority: P1 · difficulty: 1 · type: feature
- milestone: T-119
- why: Hand edits and time break entries; check is how the team notices.
- depends: T-088, T-087, T-092, T-075, T-082, T-098, T-102
- files: src/cmd/check.rs, tests/cmd_check.rs
- read: docs/design.md, docs/format.md, docs/requirements.md
- steps:
  1. Args stays empty.
  2. In run: Ctx::load(g, 'check'); collect findings as lines '<path relative to the project with />:<line>: <ID> <KIND> <message>' (line 1 when unknown).
  3. PARSE: one per warning of store::all. NAME: the file name does not start with '<ID>-', or is not paths::windows_safe. DUPLICATE: an ID that appears in two files. STATUS: a missing or unknown status. INCOMPLETE: each rules::problems of an approved entry. LINK: an ID in related, supersedes or superseded-by that store::get does not find. CHANGED: an approved lesson whose approval::mark is Changed. TIME: created or approved does not start with a time that time::valid accepts. WHO: an entry without author or without any history line.
  4. Print the lines sorted; end with 'DOSSIER: ok (0 problems)' or 'DOSSIER: <n> problems'.
  5. When there are no problems return Output::text with the ok line. When there are problems return Err(Error::new(Code::Refused, <all lines joined by line breaks>)), so the exit code is 1 with and without --json.
  6. Create tests/cmd_check.rs with mod common: a fresh project with two entries is ok; a file renamed to another ID gives NAME; a related field naming DEC-999 gives LINK; an approved lesson edited by hand gives CHANGED; an entry with the author line deleted gives WHO.
  7. Run cargo test --test cmd_check.
- accept: check lists each kind of problem on its own line and says ok when there is none.
- verify: cargo test --test cmd_check
- verified: 2026-11-21
- commit: 98de25c
- reqs: R-CHECK-1, R-HIST-4, R-XOS-4
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-21, haiku-t-129): done

### T-130 · Context: every LLM starts with the project's decisions and state
- status: done
- owner: coordinator
- priority: P0 · type: milestone
- milestone: T-001
- goal: brief, brief --task, status, the conflict check of decide, the similar-entry hint of learn and the agent rules work as docs/design.md describes.
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-21, coordinator): done

### T-131 · brief: what an LLM should know at session start
- status: done
- owner: haiku-t-131
- priority: P0 · difficulty: 1 · type: feature
- milestone: T-130
- why: An LLM that starts without the project's decisions will contradict them.
- depends: T-088, T-082, T-098, T-100, T-101, T-103, T-102, T-125
- files: src/cmd/brief.rs, tests/cmd_brief.rs
- read: docs/design.md, docs/requirements.md
- steps:
  1. Give Args: --task <task>.
  2. In run: Ctx::load(g, 'brief'); when --task is given return the lines of crate::brief_task::lines(&ctx, task, ctx.cfg.brief_lines) joined (that module is written by its own task).
  3. Otherwise build the lines: 'Project <code>' and one line with the tags of the settings; '## Decisions in force' then every approved decision newest created first, each as '<ID> · <title> (<author>, <date>)' followed by the bodies of its Decision and Reason sections on following lines indented by two spaces; '## Open assumptions' with ID and title of approved assumptions whose state is open; '## Findings' with ID and title of approved findings, dead ends marked '(dead end)'; '## Lessons' with ID and title of approved lessons; a last line '<n> drafts are waiting: dossier drafts' when n > 0.
  4. Never include draft, rejected, superseded or retired entries.
  5. Keep to cfg.brief_lines: if the text is longer, shorten decisions from the oldest upwards to their title line only; if still longer, cut the Findings and Lessons lists and end with '<m> entries left out: dossier list'.
  6. JSON data: {code, decisions: [...ids], assumptions, findings, lessons, drafts: n}.
  7. Create tests/cmd_brief.rs with mod common: the brief contains a decision's reason, an open assumption, a finding title and the drafts line; a draft lesson's title and a superseded decision's title do not appear; with 70 decisions the output has at most 60 lines and ends with the left out line; two runs give identical output.
  8. Run cargo test --test cmd_brief.
- accept: brief shows decisions in full, open assumptions, findings and lessons by title, never unapproved entries, within the line limit and the same on every run.
- verify: cargo test --test cmd_brief
- verified: 2026-11-21
- commit: b0a9f93
- reqs: R-CTX-1, R-CTX-4, R-CTX-5, TEST-9
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-21, planner): Depends also on the commands its test uses to set up entries.
- note (2026-11-21, haiku-t-131): done

### T-132 · brief_task.rs: what matters for one task
- status: done
- owner: haiku-t-132
- priority: P0 · difficulty: 1 · type: feature
- milestone: T-130
- why: Before a task, the LLM needs the entries that concern exactly this task.
- depends: T-120, T-113, T-082
- files: src/brief_task.rs
- read: docs/design.md, docs/requirements.md
- steps:
  1. Implement lines(ctx, task, limit): call todo_bridge::json(&ctx.root, ['show', task]); on Code::TodoMissing return that error with the message 'brief --task needs todo'.
  2. From the data read the strings title, machine, analysis, software (look first at the top level, then inside an object named fields; missing ones are empty) and the list milestones if present (each item's id), else the single string milestone.
  3. Line 1: 'Task <task> Â· <title>'.
  4. '## Recorded for this task': approved entries whose task field equals the task, as '<ID> Â· <kind> Â· <title>'.
  5. '## Decisions of its milestones': approved decisions whose task field equals one of the milestone IDs, each with its Decision section body indented.
  6. '## Related': select::best over the other approved entries with Query{tags: the task's machine, analysis and software values split at commas, words: select::words(title)}, limit 10; lessons with their Statement and Valid for bodies indented, other kinds as one line.
  7. Leave a heading out when it has no lines; cut the list to limit lines and add '<m> more left out' when cut.
  8. Add tests with a temporary dossier, a Ctx built in the test and a stand-in todo (as in the todo_bridge task, DOSSIER_TODO set inside one test that holds crate::ENV_LOCK) whose show output has title 'Speedline of the compressor' and machine 'axial compressor': an entry with task equal to the task is listed first; a lesson tagged axial compressor appears under Related with its Valid for text; a draft never appears; without todo the error is TodoMissing.
  9. Run cargo test --lib brief_task::.
- accept: The lines contain the task's own entries, its milestones' decisions and the best-matching approved entries; without todo it fails clearly.
- verify: cargo test --lib brief_task::
- verified: 2026-11-21
- commit: 353d894
- reqs: R-CTX-2, R-CTX-3, R-CTX-5
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-21, planner): Test holds crate::ENV_LOCK (see T-067): unit tests run in parallel threads and share the environment.
- note (2026-11-21, haiku-t-132): done

### T-133 · status: the state of the whole project
- status: done
- owner: haiku-t-133
- priority: P0 · difficulty: 1 · type: feature
- milestone: T-130
- why: Three LLMs working for three people must see the same picture of progress, decisions and what is unsettled.
- depends: T-120, T-090, T-088, T-082, T-098, T-100, T-101, T-102
- files: src/cmd/status.rs, tests/cmd_status.rs
- read: docs/design.md, docs/requirements.md
- steps:
  1. Give Args: --since <date or the word last>, --milestone <task>.
  2. In run: Ctx::load(g, 'status'); line 1: 'Project <code>'.
  3. Task part: when todo_bridge::available() is false print 'tasks: todo is not installed' and go on; else call todo_bridge::json(&ctx.root, ['summary']) and print '## Tasks', one line per item of data.milestones as '<id> · <title> · <progress.done>/<progress.total> done', and one line from data.counts: 'open <n>, claimed <n>, blocked <n>, done <n>'; at most 20 lines.
  4. '## Decisions in force': approved decisions newest first as '<ID> Â· <title> (<author>, <date>)'.
  5. '## Newest findings': the five newest approved findings in the same form.
  6. '## Unsettled': one line each when not zero: '<n> drafts waiting', '<n> open assumptions', '<n> approved lessons changed by hand' (approval::mark Changed).
  7. With --milestone keep only entries whose task field equals it.
  8. With --since: print instead, under '## Since <date>', one line per journal record on or after the date as in the log command; the word last uses the time stored in DOSSIER/.state/seen-<person>.txt (missing: everything) and afterwards writes time::now() into that file when the writer is a person.
  9. Keep to cfg.status_lines by cutting the longest list and adding '<m> more: dossier list'.
  10. Create tests/cmd_status.rs with mod common: without todo the output has the 'todo is not installed' line and still lists a decision and the drafts count; with a stand-in todo written by project.write_stand_in('todo', ...) that prints a summary envelope with one milestone, the Tasks part shows that milestone; --since last run twice by anna shows the events the first time and none the second; an LLM's --since last does not move the marker.
  11. Run cargo test --test cmd_status.
- accept: status combines the task state from todo with decisions, findings and what is unsettled, works without todo, and --since last shows only what is new for that person.
- verify: cargo test --test cmd_status
- verified: 2026-11-21
- commit: 2679a78
- reqs: R-CTX-9 to R-CTX-11, R-READ-8, R-READ-10
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-21, planner): Test uses Project::command or write_stand_in from tests/common (T-098).
- note (2026-11-21, planner): Tasks part reads data.milestones and data.counts: todo summary --json has no text field.
- note (2026-11-21, haiku-t-133): done

### T-134 · decide: show related decisions, and --replaces or --besides for an LLM
- status: done
- owner: haiku-t-134
- priority: P1 · difficulty: 1 · type: feature
- milestone: T-130
- why: Two LLMs must not record contradicting decisions without anyone noticing.
- depends: T-100, T-113, T-125
- files: src/cmd/decide.rs, tests/cmd_decide_related.rs
- read: docs/design.md, docs/requirements.md
- steps:
  1. In run of decide, before creating: collect the approved decisions and call select::best with Query{tags: the given machine, analysis, software and tags values, words: select::words(title plus decision)}, limit 5; these are the related decisions.
  2. When the writer is an LLM, there are related decisions, and neither --replaces nor --besides is given: fail with Code::Refused and a message that lists the related decisions (ID and title) and says 'state --replaces <ID> or --besides'.
  3. With --replaces <ID>: inside the same tx::write, after creating the new decision, apply to the old one what the supersede command does (status superseded, superseded-by, valid-until, history line) and add supersedes to the new entry before it is written; the old entry must be an approved decision.
  4. Otherwise, when there are related decisions, add them to the Output as warnings 'related: <ID> <title>'.
  5. Create tests/cmd_decide_related.rs with mod common: anna-claude with --said-by anna and a second decision whose title shares words with the first is refused and the message names the first ID; with --besides it is written; with --replaces the first decision is superseded and the new one has supersedes; a person without flags gets a warning and the entry is written.
  6. Run cargo test --test cmd_decide_related and cargo test --test cmd_decide.
- accept: An LLM cannot record a decision next to related ones without saying whether it replaces them; --replaces supersedes in the same step.
- verify: cargo test --test cmd_decide_related
- verified: 2026-11-21
- commit: 405cb80
- reqs: R-CTX-13
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-21, haiku-t-134): done

### T-135 · learn: show similar entries, and --update
- status: done
- owner: haiku-t-135
- priority: P2 · difficulty: 1 · type: feature
- milestone: T-130
- why: Without it the same lesson is written five times in slightly different words.
- depends: T-102, T-113, T-104
- files: src/cmd/learn.rs, tests/cmd_learn_similar.rs
- read: docs/design.md, docs/requirements.md
- steps:
  1. In run of learn, collect the lessons of every status except superseded and retired and call select::best with Query{tags: the given tag values, words: select::words(title plus statement)}, limit 5.
  2. Without --update: create the lesson as before and add one warning per similar lesson: 'similar: <ID> <title>' plus ' [rejected]' or ' [draft]' for those statuses.
  3. With --update <ID>: do not create an entry; inside tx::write load that lesson, set the sections Statement, Valid for and Evidence and the field confidence to the given values, add a history line 'changed' with detail 'updated', apply the same approved-lesson rule as the set command (an LLM's change makes it a draft; a person's change records the approval checksum again), queue, and print '<ID>: updated'.
  4. Create tests/cmd_learn_similar.rs with mod common: a second lesson with a similar title prints a similar warning naming the first ID; --update changes the first lesson and creates no new file; an LLM's --update of an approved lesson makes it a draft.
  5. Run cargo test --test cmd_learn_similar and cargo test --test cmd_learn.
- accept: learn names similar existing lessons and can update one instead of adding a new one.
- verify: cargo test --test cmd_learn_similar
- verified: 2026-11-21
- commit: c78ee7f
- reqs: R-WRITE-11
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-21, haiku-t-135): done

### T-136 · The rules text for LLMs
- status: done
- owner: haiku-t-136
- priority: P0 · difficulty: 1 · type: docs
- milestone: T-130
- why: Every LLM learns the tool from this one text; if it is unclear, nothing gets recorded.
- depends: T-067
- files: src/agents/rules.md
- read: docs/requirements.md, docs/design.md
- steps:
  1. Replace the content of src/agents/rules.md with a heading '# Working with the dossier' and a list of rules, at most 40 lines in all, each one line, imperative, with the command in backticks.
  2. Rules to include, in this order: use one name per agent with --agent or DOSSIER_AGENT; start a session with dossier brief; before a task run dossier brief --task <id>; before work no task asks for, run dossier status; work only on a task you have claimed in todo; decisions in force bind you: if your task contradicts one, stop and ask your person, never work around it; when your person decides something, record it at once with dossier decide ... --reason ... --said-by <person>; record what you observe with dossier found, and what was tried and failed with dossier found --dead-end; record what you had to assume with dossier assume ... --reason ...; after todo done run dossier review-task <id> and propose lessons with dossier learn, or none if nothing was learned; search before proposing: dossier search <words>; write every entry in English; never edit files under DOSSIER/ directly; never approve, reject, retire or remove: a person does that; exit code 1 means a rule refused the change, 3 means busy, run it again.
  3. End the file with a single line break and use LF line endings.
- accept: rules.md has at most 40 lines and contains every rule listed.
- verify: cargo build
- verified: 2026-11-21
- commit: ee7e8ed
- reqs: R-AGENT-1, R-CTX-12, R-ENTRY-19, D25
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-21, haiku-t-136): done

### T-137 · agents/mod.rs: the managed block in AGENTS.md
- status: done
- owner: haiku-t-137
- priority: P0 · difficulty: 1 · type: feature
- milestone: T-130
- why: The rules must be installable into AGENTS.md without touching any other line.
- depends: T-067
- files: src/agents/mod.rs
- read: docs/design.md
- steps:
  1. Implement block(): BEGIN, a line break, RULES with CRLF turned into LF and trailing line breaks trimmed, a line break, END, a line break.
  2. Implement current(existing): true when existing with CRLF turned into LF contains block() with its final line break trimmed.
  3. Implement installed(existing): if existing contains BEGIN and END, replace everything from BEGIN to END (inclusive) with the block without its final line break; else if it contains the line '<!-- todo:end -->', insert a blank line and the block after that line; else append a blank line (when the text is not empty and does not end with a blank line) and the block. Keep every other byte.
  4. Add tests: installing into an empty text; into a text with a todo block (the dossier block comes after it); installing twice gives the same text; current is true after installing and false before; a text with CRLF endings keeps its other lines unchanged.
  5. Run cargo test --lib agents::.
- accept: The block is added once, after todo's block when there is one, and replaced in place later; other lines stay unchanged.
- verify: cargo test --lib agents::
- verified: 2026-11-21
- commit: 445ac20
- reqs: R-AGENT-2
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-21, haiku-t-137): done

### T-138 · agents install, agents check and help agents
- status: done
- owner: haiku-t-138
- priority: P0 · difficulty: 1 · type: feature
- milestone: T-130
- why: The commands that put the rules in front of every LLM.
- depends: T-137, T-136, T-068
- files: src/cmd/agents.rs, src/cmd/help.rs, tests/cmd_agents.rs
- read: docs/design.md
- steps:
  1. In src/cmd/agents.rs give Args a subcommand enum: Install { #[arg(long)] yes: bool } and Check.
  2. Install: the file is AGENTS.md in the current folder; read it (missing is empty); compute agents::installed; without --yes print 'would write the rules block to AGENTS.md; add --yes' and change nothing; with --yes write the file and print 'AGENTS.md: rules block written'.
  3. Check: print 'AGENTS.md: rules block is current' when agents::current, else return Code::Refused with the message 'AGENTS.md: rules block is missing or out of date; run: dossier agents install --yes'.
  4. In src/cmd/help.rs give Args an optional positional topic: Option<String>; with topic agents print agents::RULES; with no topic print a short overview: one line per command of design.md section 5 with five words of description; any other topic is Code::Usage.
  5. Create tests/cmd_agents.rs with assert_cmd in a tempfile folder (no dossier needed): help agents prints the heading of the rules; agents check fails with exit 1 before install and succeeds after agents install --yes; a line written into AGENTS.md before install is still there afterwards.
  6. Run cargo test --test cmd_agents.
- accept: agents install writes the block, agents check reports whether it is current, help agents prints the rules.
- verify: cargo test --test cmd_agents
- verified: 2026-11-21
- commit: 0345539
- reqs: R-AGENT-1, R-AGENT-2, D14
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-21, haiku-t-138): done

### T-139 · From the PC: commands run on the server, and Windows and Linux agree
- status: done
- owner: coordinator
- priority: P0 · type: milestone
- milestone: T-001
- goal: A folder on a PC is connected to a project on the server, the project is noted in a per-user register, and every dossier and todo command typed on the PC runs on the server; the same commands give the same files on Windows and Linux.
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-21, coordinator): done

### T-140 · remote/pointer.rs: the .dossier-connect file
- status: done
- owner: haiku-t-140
- priority: P0 · difficulty: 1 · type: feature
- milestone: T-139
- why: A connected folder must remember which server project it belongs to.
- depends: T-071
- files: src/remote/pointer.rs
- read: docs/design.md, docs/format.md
- steps:
  1. Implement write(folder, pointer): write the file .dossier-connect in the folder as three TOML lines server, folder and code, each a quoted string, with LF endings.
  2. Implement find(start): look for .dossier-connect in start and then in each parent; parse it with the toml crate into Pointer (a private serde struct with deny_unknown_fields); return the folder it is in and the Pointer; a file that does not parse counts as not found.
  3. Add tests with tempfile: write then find from a subfolder returns the same values and the folder; no file gives None; a broken file gives None.
  4. Run cargo test --lib remote::pointer::.
- accept: The pointer file is written and found from any subfolder.
- verify: cargo test --lib remote::pointer::
- verified: 2026-11-21
- commit: 2576723
- reqs: R-REMOTE-1
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-21, haiku-t-140): done

### T-141 · remote/register.rs: the per-user list of projects
- status: done
- owner: haiku-t-141
- priority: P0 · difficulty: 1 · type: feature
- milestone: T-139
- why: A person or an LLM must never have to know or type a server name or path.
- depends: T-071
- files: src/remote/register.rs
- read: docs/design.md, docs/format.md
- steps:
  1. Implement file(): the environment variable DOSSIER_REGISTER if set; else on Windows %APPDATA%/dossier/projects.toml; else $XDG_CONFIG_HOME/dossier/projects.toml, or $HOME/.config/dossier/projects.toml; a missing variable is Code::Config with the message naming it.
  2. Implement load(): a missing file is an empty list; parse the TOML with a private serde struct holding project: Vec<...> (default empty); a parse error is Code::Config naming the file.
  3. Implement put(project): load, remove the one with the same code (ignoring case), add the new one, sort by code, and write the whole file (create folders) as [[project]] tables with code, server, folder and, when present, local; write through atomic::replace.
  4. Implement remove(code): load, remove it, write, and return whether one was removed.
  5. Add tests that set DOSSIER_REGISTER to a path in a tempfile folder inside one test function that holds crate::ENV_LOCK: put two projects, load returns them sorted; put with the same code replaces; remove returns true then false.
  6. Run cargo test --lib remote::register::.
- accept: Projects are stored per user with code, server, folder and local folder, and survive being rewritten.
- verify: cargo test --lib remote::register::
- verified: 2026-11-21
- commit: 553f104
- reqs: R-REMOTE-9, R-REMOTE-10
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-21, planner): Test holds crate::ENV_LOCK (see T-067): unit tests run in parallel threads and share the environment.
- note (2026-11-21, haiku-t-141): done

### T-142 · remote/ssh.rs: build and run the ssh command
- status: done
- owner: haiku-t-142
- priority: P0 · difficulty: 1 · type: feature
- milestone: T-139
- why: Text with quotes, backticks and dollar signs must arrive on the server unchanged, and ssh must never ask a question.
- depends: T-071
- files: src/remote/ssh.rs
- read: docs/design.md, docs/requirements.md
- steps:
  1. Implement quote(arg): a single quote, the argument with every single quote replaced by the four characters quote backslash quote quote, and a closing single quote; an empty argument gives two single quotes.
  2. Implement argv as design.md describes: ['-o', 'BatchMode=yes', '-o', 'ConnectTimeout=<timeout>'], then '-t' when terminal, then the server, then one string built as: 'cd ' + quote(folder) + ' && ' + (when agent is Some: 'DOSSIER_AGENT=' + quote(agent) + ' ') + quote(program) + for each argument ' ' + quote(argument).
  3. Implement run(argv): the program is the environment variable DOSSIER_SSH or 'ssh'; start it with the arguments and inherited standard input, output and error; wait; when it cannot be started return Code::Ssh 'ssh is not installed or not on PATH'; when its exit code is 255 return Code::Ssh 'could not reach the server; sign in once by hand with: ssh <server>'; else return the exit code as u8.
  4. Add tests: quote for a plain word, a word with a space, with a single quote, with a dollar sign and a backtick, and an empty string; argv for a command with two arguments gives exactly the expected list, with and without agent and terminal; a text with a line break stays one argument.
  5. Run cargo test --lib remote::ssh::.
- accept: Every argument is quoted so a POSIX shell passes it on unchanged; ssh runs without prompts and its failures become SSH errors.
- verify: cargo test --lib remote::ssh::
- verified: 2026-11-21
- commit: 269e93c
- reqs: R-REMOTE-2, R-REMOTE-6, R-REMOTE-11, R-XOS-10
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-21, haiku-t-142): done

### T-143 · remote/forward.rs: run a command on the server
- status: done
- owner: haiku-t-143
- priority: P0 · difficulty: 1 · type: feature
- milestone: T-139
- why: From a PC, dossier must behave as if the project were local.
- depends: T-140, T-141, T-142, T-081
- files: src/remote/forward.rs
- read: docs/design.md, docs/requirements.md
- steps:
  1. Implement maybe(args): args are the program arguments including the program name at index 0; the command word is the first argument after index 0 that does not start with - and is not the value of --dossier, --agent or --project.
  2. Return None when the command word is connect, projects or help, or when there is none.
  3. Find the target: if --project <code> is among the arguments, look it up in register::load (unknown code: print 'error: no project <code> in the register; see dossier projects' to standard error and return Some(1)) and remove the option and its value from the arguments; else use pointer::find from the current folder; if neither gives a target return None.
  4. The agent is the value of --agent if present, else the environment variable DOSSIER_AGENT, else TODO_AGENT.
  5. The remote program is 'dossier'; when the command word is todo, the remote program is 'todo' and the word todo is removed from the arguments.
  6. terminal is true when the command word is approve, remove, restore or edit and term::is_terminal() is true.
  7. Build ssh::argv with timeout 10 and run it; on Ok(code) return Some(code); on an error print 'error: <message>' to standard error and return Some(5).
  8. Add one test function that holds crate::ENV_LOCK (it changes the current folder and DOSSIER_SSH; set the current folder back at the end), with a tempfile folder: no pointer and no --project gives None; with a pointer and DOSSIER_SSH pointing at a stand-in program that writes its arguments to a file and exits with 7, maybe returns Some(7) and the file contains BatchMode=yes, the server and the quoted command; the command connect gives None even with a pointer; todo list forwards the program todo.
  9. Run cargo test --lib remote::forward::.
- accept: In a connected folder, or with --project, a command is run on the server with its output and exit code passed through; connect, projects and help stay local.
- verify: cargo test --lib remote::forward::
- verified: 2026-11-21
- commit: 47fa687
- reqs: R-REMOTE-1 to R-REMOTE-3, R-REMOTE-5, R-REMOTE-10, R-REMOTE-11, TEST-8
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-21, planner): Test holds crate::ENV_LOCK (see T-067): unit tests run in parallel threads and share the environment.
- note (2026-11-21, haiku-t-143): done

### T-144 · connect: tie a PC folder to a project on the server
- status: done
- owner: haiku-t-144
- priority: P0 · difficulty: 1 · type: feature
- milestone: T-139
- why: One command per project, once, and nobody types a server or path again.
- depends: T-140, T-141, T-142, T-068
- files: src/cmd/connect.rs, tests/cmd_connect.rs
- read: docs/design.md, docs/requirements.md
- steps:
  1. Give Args: target: Option<String> (user@server:folder), --code, --no-folder (flag), --check (flag).
  2. With a target: split it at the first ':' into server and folder; both must be non-empty and the folder must start with / (Code::BadValue with an example otherwise).
  3. Get the project code: --code if given; else run ssh::argv(server, folder, None, 'dossier', ['where', '--json'], false, 10) with std::process::Command capturing standard output (program DOSSIER_SSH or ssh) and read data.code from the JSON; when that fails return Code::Ssh with the message 'could not read the project code from the server; give it with --code'.
  4. Unless --no-folder: write the pointer with pointer::write into the current folder.
  5. Call register::put with code, server, folder and local = the current folder (None with --no-folder).
  6. Print 'connected <code> -> <server>:<folder>'.
  7. With --check and no target: find the pointer, run the same where --json call, and print 'ok: <code> on <server>' or return the Ssh error.
  8. Create tests/cmd_connect.rs with assert_cmd in a tempfile folder, DOSSIER_REGISTER pointing into it: connect anna@sim01:/projects/P-017 --code P-017 creates .dossier-connect and a register file containing P-017 and sim01; a target without ':' fails with exit 1; --no-folder writes the register but no pointer file.
  9. Run cargo test --test cmd_connect.
- accept: connect writes the pointer file and the register entry; a bad target is refused.
- verify: cargo test --test cmd_connect
- verified: 2026-11-21
- commit: da36767
- reqs: R-REMOTE-1, R-REMOTE-9, R-REMOTE-8
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-21, haiku-t-144): done

### T-145 · projects: list and remove register entries
- status: done
- owner: haiku-t-145
- priority: P1 · difficulty: 1 · type: feature
- milestone: T-139
- why: People and LLMs need to see which projects this PC knows and where they are.
- depends: T-141, T-068, T-144
- files: src/cmd/projects.rs, tests/cmd_projects.rs
- read: docs/design.md
- steps:
  1. Give Args an optional subcommand enum with Remove { code: String }.
  2. Without a subcommand: register::load and print one line per project: '<code>  <server>:<folder>  <local folder or ->'; 'no projects; add one with: dossier connect <user@server>:<folder>' when empty. JSON data: the list.
  3. Remove: call register::remove; print 'removed <code>' or return Code::NotFound 'no project <code>'.
  4. Create tests/cmd_projects.rs with assert_cmd, DOSSIER_REGISTER in a tempfile folder: after two connect --code ... --no-folder calls, projects lists both codes; remove one, the list has one; removing it again gives exit 1.
  5. Run cargo test --test cmd_projects.
- accept: projects lists the register and removes entries.
- verify: cargo test --test cmd_projects
- verified: 2026-11-21
- commit: 7170a59
- reqs: R-REMOTE-10
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-21, planner): Depends also on the commands its test uses to set up entries.
- note (2026-11-21, haiku-t-145): done

### T-146 · todo: run todo in the project folder
- status: done
- owner: haiku-t-146
- priority: P1 · difficulty: 1 · type: feature
- milestone: T-139
- why: The task list is reached through the same command from a PC and on the server.
- depends: T-082, T-098
- files: src/cmd/todo.rs, tests/cmd_todo.rs
- read: docs/design.md
- steps:
  1. Give Args: #[arg(trailing_var_arg = true, allow_hyphen_values = true)] args: Vec<String>.
  2. In run: find the project folder with find::find from the current folder and take its parent (no name is needed).
  3. Run the program DOSSIER_TODO or todo with the arguments in that folder, with inherited standard input, output and error; when it cannot be started return Code::TodoMissing.
  4. When it exits with 0 return Output::text(''); otherwise end the process with the same exit code: std::process::exit(code).
  5. Create tests/cmd_todo.rs with mod common: with a stand-in todo written by project.write_stand_in('todo', ...) that prints its arguments and exits with 3, dossier todo list --wide prints 'list --wide' and exits with 3; without a stand-in the exit code is 5.
  6. Run cargo test --test cmd_todo.
- accept: dossier todo passes arguments, output and exit code through to todo, run in the project folder.
- verify: cargo test --test cmd_todo
- verified: 2026-11-21
- commit: 743b853
- reqs: R-REMOTE-5
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-21, planner): Test uses Project::command or write_stand_in from tests/common (T-098).
- note (2026-11-21, haiku-t-146): done

### T-147 · tar.rs: a small archive writer and reader
- status: done
- owner: haiku-t-147
- priority: P1 · difficulty: 2 · type: feature
- milestone: T-139
- why: A backup archive without a new dependency.
- depends: T-071
- files: src/tar.rs
- read: docs/design.md
- steps:
  1. Implement pack(folder, out): walk the folder recursively in sorted order; for each regular file write a 512-byte ustar header (name relative to the folder with /, mode 0644, size in octal, mtime 0, typeflag 0, magic ustar, checksum) followed by the content padded with zero bytes to a multiple of 512; names longer than 100 bytes use the ustar prefix field (up to 155 bytes) and longer ones are Code::BadValue; end with two zero blocks; return the number of files.
  2. Never follow or store symbolic links; skip them.
  3. Implement unpack(archive, into): refuse an existing non-empty folder with Code::Exists; read block by block; for each header with a non-empty name, join prefix and name, refuse a name that is absolute or contains .. with Code::BadValue, create folders, write size bytes; stop at a zero block; return the number of files.
  4. Add tests with tempfile: pack then unpack a folder with a nested file, an empty file, a file of 1000 bytes and a file whose name has a space and an umlaut gives identical contents; packing the same folder twice gives byte-identical archives; unpack into a non-empty folder fails.
  5. Run cargo test --lib tar::.
- accept: A folder packed and unpacked is identical; archives are byte-identical for the same input.
- verify: cargo test --lib tar::
- verified: 2026-11-21
- commit: ab26f73
- final: The header layout is fixed by the tar format; the worker has to look up the field offsets.
- reqs: R-BACKUP-1, R-BACKUP-2, NFR-6
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-21, haiku-t-147): done

### T-148 · pack and unpack: one archive of a dossier
- status: done
- owner: haiku-t-148
- priority: P1 · difficulty: 1 · type: feature
- milestone: T-139
- why: One copy on a server with no git means the backup is the only protection.
- depends: T-147, T-079, T-082, T-098, T-101
- files: src/cmd/pack.rs, src/cmd/unpack.rs, tests/cmd_pack.rs
- read: docs/design.md, docs/requirements.md
- steps:
  1. In pack.rs give Args: --to <file> (required). In run: Ctx::load(g, 'pack'); refuse an existing target file with Code::Exists; take the lock with lock::acquire using the name 'pack' when no writer is given; call tar::pack(&ctx.dossier, &to); print 'packed <n> files into <file>'.
  2. In unpack.rs give Args: file: PathBuf (positional), --into <folder> (required). In run: call tar::unpack(&file, &into.join('DOSSIER')); print 'unpacked <n> files into <folder>/DOSSIER' and a second line 'check it with: dossier check' (run from that folder).
  3. Create tests/cmd_pack.rs with mod common: after two entries, pack --to backup.tar creates the file; unpack backup.tar --into a new folder gives a DOSSIER whose dossier.toml and entry files equal the originals byte for byte, including .state/journal.jsonl; packing to an existing file fails.
  4. Run cargo test --test cmd_pack.
- accept: pack writes the whole dossier with its state folder into one archive under the lock; unpack restores it identically.
- verify: cargo test --test cmd_pack
- verified: 2026-11-21
- commit: 274fb49
- reqs: R-BACKUP-1, R-BACKUP-2, D24
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-21, haiku-t-148): done

### T-149 · Test: Windows and Linux write the same files
- status: done
- owner: haiku-t-149
- priority: P0 · difficulty: 1 · type: test
- milestone: T-139
- why: The dossier lives on Linux and is used from Windows; nothing may depend on where a command was typed.
- depends: T-104, T-123, T-118, T-114, T-100, T-101, T-103
- files: tests/xos.rs
- read: docs/design.md, docs/format.md, docs/requirements.md
- steps:
  1. Create tests/xos.rs with mod common.
  2. Test 1, identical bytes: run a fixed sequence (decide, found, assume, learn as anna, set on the finding) with fixed times via run_at; compute a SHA-256 over the names and contents of all files under DOSSIER except .state/holder.json and .state/write.lock; compare it with a constant in the test. To get the constant, run the test once, copy the printed value into the test, and commit; CI on the other system proves it is the same there.
  3. Test 2, line endings: rewrite an entry file with CRLF endings and a byte-order mark, run set --field machine=x, and check that every other line still ends with CRLF, the mark is still there, and the new history line ends with CRLF.
  4. Test 3, names: every file under DOSSIER has a lower-case name after its ID and passes a small windows-safe check written in the test (no reserved name, none of the forbidden characters).
  5. Test 4, times: every created, approved and history value in the files matches the pattern of four digits, dash, two digits, dash, two digits, T, time, and a UTC offset.
  6. Test 5, search order: the output of search for a common word is the same text as a constant in the test.
  7. Run cargo test --test xos.
- accept: The same commands give byte-identical dossier files and the same search order on Windows and Linux; CRLF files keep their endings.
- verify: cargo test --test xos
- verified: 2026-11-21
- commit: ad40269
- reqs: R-XOS-4, R-XOS-6, R-XOS-7, TEST-11, TEST-3
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-21, planner): Depends also on the commands its test uses to set up entries.
- note (2026-11-21, haiku-t-149): done

### T-150 · Ship v0.1: proven fast enough, documented, released
- status: done
- owner: coordinator
- priority: P1 · type: milestone
- milestone: T-001
- goal: The speed limits are proven, the README lets the team start without the author, and a v0.1.0 tag builds the Linux and Windows programs.
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-21, coordinator): done

### T-151 · Test: speed with 300 entries
- status: done
- owner: haiku-t-151
- priority: P1 · difficulty: 1 · type: test
- milestone: T-150
- why: No index is kept, so the speed limits must be proven before the design is relied on.
- depends: T-131, T-114, T-101
- files: tests/perf.rs
- read: docs/requirements.md, docs/design.md, docs/format.md
- steps:
  1. Create tests/perf.rs with mod common.
  2. Write 300 entry files directly into the project's DOSSIER folders (150 findings, 100 decisions, 50 lessons) with a loop, using the canonical text of format.md section 1 with different IDs, titles and a paragraph of text each.
  3. Measure with std::time::Instant, five runs each after one warm-up run, and take the median: dossier brief, dossier search compressor, and dossier found 'x' (a write).
  4. Assert brief and search medians under 300 ms and the write median under 200 ms; when the environment variable CI is set, allow three times these limits (shared runners are slow) and print the measured values.
  5. Run cargo test --test perf --release.
- accept: With 300 entries, brief and search take under 300 ms and a write under 200 ms on a normal PC.
- verify: cargo test --test perf --release
- verified: 2026-11-21
- commit: 06f12f9
- reqs: NFR-3
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-21, haiku-t-151): done

### T-152 · README for v0.1
- status: done
- owner: haiku-t-152
- priority: P1 · difficulty: 1 · type: docs
- milestone: T-150
- why: The repository is public and the team has to be able to start without the author.
- depends: T-138, T-148, T-144, T-133
- files: README.md
- read: docs/requirements.md, docs/design.md
- steps:
  1. Rewrite README.md with these sections in this order.
  2. What it is for: three sentences from requirements section 1.
  3. What it needs: todo 0.8 or newer with the link https://github.com/reynanTandoyo/todo, and ssh for use from a PC.
  4. Install: copy the program into a folder on PATH, on the server (Linux) and on each PC (Windows); no administrator rights.
  5. A first project in ten commands, as a code block run on the server: dossier init --code P-001; dossier people add anna --llm anna-claude; dossier agents install --yes; dossier decide ... --reason ...; dossier found ...; dossier assume ... --reason ...; dossier learn ... (as the LLM); dossier drafts; dossier approve LES-001; dossier brief.
  6. How it works with todo: tasks live in TODO.md, knowledge in DOSSIER/; entries name their task with --task; dossier status shows both; dossier todo ... runs todo.
  7. From a Windows PC: dossier connect user@server:/projects/P-001 once, then every command as if local; dossier projects; ssh must sign in by key without a password; show the examples in PowerShell.
  8. Who and when: every entry and every change carries the person, the LLM and the time; dossier log and dossier show.
  9. Backup: the dossier is one copy on the server; include the project folders in the server's backup; dossier pack --to <file> as an extra, with a cron line as an example.
  10. Windows and Linux: write only through the tool; reading the files through a network share is fine; the Linux group must be shared by the team.
  11. Building from source: cargo build --release.
- accept: README has every section above with working example commands in PowerShell and a Linux shell.
- verify: git ls-files --error-unmatch README.md
- verified: 2026-11-21
- commit: 34cd824
- reqs: REL-2, R-BACKUP-3, R-XOS-8
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-21, haiku-t-152): done

### T-153 · Release workflow for a version tag
- status: done
- owner: haiku-t153
- priority: P1 · difficulty: 1 · type: chore
- milestone: T-150
- why: The team installs the program from a release, on the server and on each PC.
- depends: T-069
- files: .github/workflows/release.yml, .cargo/config.toml
- read: docs/requirements.md
- steps:
  1. Create .cargo/config.toml with [target.x86_64-pc-windows-msvc] rustflags = ['-C', 'target-feature=+crt-static'], so the Windows program runs without Visual C++ installed.
  2. Create .github/workflows/release.yml triggered on push of tags matching v*.
  3. Job check: fail when the tag without the leading v differs from the version in Cargo.toml.
  4. Job build with a matrix: ubuntu-latest building target x86_64-unknown-linux-musl (install the target with rustup and the package musl-tools) producing the file dossier, and windows-latest building x86_64-pc-windows-msvc producing dossier.exe; each runs cargo build --release --target <target>.
  5. Rename the outputs to dossier-linux-x64 and dossier-windows-x64.exe, write a .sha256 file next to each, and upload them as artifacts.
  6. Job release: download the artifacts and create a GitHub release for the tag with the four files, using the gh CLI (gh release create) and permissions contents: write.
  7. Run cargo build to check that the project still builds with the new config file.
- accept: The workflow builds a static Linux program and a Windows program with checksums and publishes them for a v* tag.
- verify: cargo build
- verified: 2026-11-21
- commit: bd69078
- reqs: NFR-1, REL-1
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-21, haiku-t153): done

### T-154 · projects --check: whether each server answers
- status: done
- owner: haiku-t-154
- priority: P1 · difficulty: 1 · type: feature
- milestone: T-150
- why: R-REMOTE-10 asks projects to show whether the server answers; T-145 left it out.
- files: src/cmd/projects.rs, tests/cmd_projects.rs
- read: docs/requirements.md, src/remote/ssh.rs, src/cmd/connect.rs
- steps:
  1. Add a flag --check to the projects Args (list mode only).
  2. With --check, for each project run the ssh program (DOSSIER_SSH or ssh) with the arguments -o BatchMode=yes -o ConnectTimeout=<ssh_timeout> <server> true, stdin null, stdout and stderr captured, one project after another; the timeout is 10 seconds.
  3. Add to each line '  answers' when the exit code is 0, else '  no answer'; without --check print nothing extra. JSON: add "answers": true/false per project only with --check.
  4. Tests in tests/cmd_projects.rs: write a stand-in ssh (a .cmd on Windows, a shell script with mode 755 elsewhere) that exits 0 for one server name and 255 for another, point DOSSIER_SSH at it, connect two projects with --code and --no-folder, and check that projects --check marks one 'answers' and the other 'no answer'; projects without --check never runs ssh (point DOSSIER_SSH at a path that does not exist and expect success).
  5. Run cargo test --test cmd_projects.
- accept: projects --check shows for each project whether its server answers; plain projects runs no ssh.
- verify: cargo test --test cmd_projects
- verified: 2026-11-21
- commit: 9599cd5
- reqs: R-REMOTE-10
- by: Reynan Henry Tandoyo
- note (2026-11-21, coordinator): added
- note (2026-11-21, haiku-t-154): done

### T-155 · v0.2 skeleton: the usage kind, proposed and declined, new settings, register time
- status: done
- owner: haiku-t-155
- priority: P0 · difficulty: 2 · type: feature
- milestone: T-031
- why: New enum variants break every exhaustive match at once, so they come in one step before the phase 2 tasks.
- final: Adding enum variants touches every exhaustive match at once.
- files: src/entry/mod.rs, src/entry/new.rs, src/entry/rules.rs, src/cmd/list.rs, src/cmd/search.rs, src/config.rs, src/names.rs, src/remote/register.rs, src/cmd/connect.rs, src/cmd/show.rs, src/cmd/review_task.rs
- read: docs/design.md, docs/format.md
- steps:
  1. entry/mod.rs: add Kind::Usage (name usage, prefix USE, folder usage) as the last variant; Kind::all() returns [Kind; 5] in the order Decision, Assumption, Finding, Lesson, Usage. Add Status::Proposed (name proposed) and Status::Declined (name declined). Extend the tests.
  2. entry/rules.rs: for usage, section_order is Parameters, Outputs, Deviations, Notes and required_sections is empty; problems gives 'missing field software', 'missing field version' and 'missing field task' when one is missing or empty. Tests.
  3. entry/new.rs: the section order for usage as above; a section named Pictures always comes after every other section, for every kind. In the known field order put version, model, model-checksum and howto right after software, and declined-reason, confirmed, refuted, rests-on and code right after rejected-reason (format.md sections 14 and 15). Tests.
  4. cmd/list.rs and cmd/search.rs: the status mark is '[proposed]' and '[declined]' for those statuses; when the field rests-on is not empty add the mark '[rests on <value>]'. Keep their tests passing.
  5. config.rs: optional top-level keys deciders (list of strings), roles (list of strings) and code_project (bool), default empty, empty, false (format.md section 19); Config gains pub deciders, pub roles, pub code_project. Unknown keys stay errors. Tests.
  6. names.rs: add may_decide and unchecked exactly as docs/design.md section 7 says, with tests (unchecked holds crate::ENV_LOCK when it changes variables). Add the new Config fields to every Config literal in its tests.
  7. remote/register.rs: Project gains pub added: String, written as the line added = "<time>" after local; an entry without it reads as an empty string. cmd/connect.rs sets added to crate::time::now(). Fix every Project literal.
  8. Run cargo build, cargo clippy --all-targets --all-features -- -D warnings, cargo clippy --target x86_64-unknown-linux-gnu --all-targets --all-features -- -D warnings and cargo test --all-features; fix every error inside these files.
- accept: The usage kind and the two new statuses exist everywhere; the new settings load; register entries carry their time; every check passes.
- verify: cargo test --all-features
- verified: 2026-11-21
- commit: 05e8a10
- reqs: D28, D29, R-ENTRY-9
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-21, planner): forced set (owner was haiku-t-155)
- note (2026-11-21, planner): src/cmd/show.rs and src/cmd/review_task.rs added: they match Option<Status> exhaustively and need arms for proposed and declined (found by the worker).
- note (2026-11-21, haiku-t-155): done

### T-156 · v0.2 skeleton: the new library modules as stubs
- status: done
- owner: haiku-t-156
- priority: P0 · difficulty: 2 · type: feature
- milestone: T-031
- why: With every type in place, each later task fills in one file.
- final: Mechanical, but touches every new library file at once.
- depends: T-155
- files: src/lib.rs, src/blockfile.rs, src/filesum.rs, src/values.rs, src/filelist.rs, src/exchange.rs, src/mapfile.rs, src/affects.rs, src/picture.rs, src/hooks.rs, src/tx.rs, src/ids.rs, src/agents/mod.rs
- read: docs/design.md, docs/format.md
- steps:
  1. Add one pub mod line to src/lib.rs for each module of docs/design.md section 8.
  2. Create each module of section 8 with every type, constant and function signature exactly as written there, with their doc comments.
  3. Add the stubs of section 7 to the existing files: Tx::put_raw and Tx::delete_raw in tx.rs, ids::next_prefix in ids.rs, agents::skill in agents/mod.rs. Do not change anything else in those files.
  4. Bodies: return Err(Error::new(Code::NotImplemented, "<module>::<function>")) where the result is a Result, else a plain default (empty String or Vec, None, false, 0, BlockFile { lines: Vec::new() }). Name unused parameters with a leading underscore. Never todo!, unimplemented!, unwrap, expect or panic, and no allow attributes.
  5. Run cargo clippy --all-targets --all-features -- -D warnings and the same with --target x86_64-unknown-linux-gnu, and cargo test --all-features.
- accept: Every module of design.md section 8 and the stubs of section 7 exist with their exact signatures; every check passes.
- verify: cargo clippy --all-targets --all-features -- -D warnings
- verified: 2026-11-21
- commit: adfd05c
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-21, haiku-t-156): done

### T-157 · v0.2 skeleton: command stubs, help overview and the forwarding lists
- status: done
- owner: haiku-t-157
- priority: P0 · difficulty: 2 · type: feature
- milestone: T-031
- why: After this, a command task only fills in its own file.
- final: Mechanical, but touches every new command file at once.
- depends: T-156
- files: src/cli.rs, src/cmd/mod.rs, src/cmd/help.rs, src/remote/forward.rs, src/cmd/propose.rs, src/cmd/accept.rs, src/cmd/decline.rs, src/cmd/confirm.rs, src/cmd/refute.rs, src/cmd/cleared.rs, src/cmd/value.rs, src/cmd/values.rs, src/cmd/attach.rs, src/cmd/detach.rs, src/cmd/reattach.rs, src/cmd/files.rs, src/cmd/which.rs, src/cmd/used.rs, src/cmd/runs.rs, src/cmd/image.rs, src/cmd/received.rs, src/cmd/delivered.rs, src/cmd/exchange.rs, src/cmd/map.rs, src/cmd/hooks.rs, src/cmd/guard.rs, src/cmd/skill.rs, tests/cmd_agents.rs
- read: docs/design.md, docs/format.md
- steps:
  1. For every new command of docs/design.md section 9 create src/cmd/<name>.rs as the phase 1 stubs were: #[derive(clap::Args, Debug)] pub struct Args {} and pub fn run(_args: &Args, _g: &crate::cli::Global) -> crate::error::Result<crate::output::Output> returning Err(Error::new(Code::NotImplemented, "<name> is not built yet")).
  2. Add one pub mod line per file to src/cmd/mod.rs and one Command variant per file to src/cli.rs, with its dispatch line; the command names are those of section 9.
  3. src/cmd/help.rs: add each new command to the overview in the order of section 9, with a description of at most five words.
  4. src/remote/forward.rs: hooks, guard and skill run locally like connect; accept, decline, confirm, refute, cleared and value ask for a terminal like approve (design.md section 7). Extend its test for one local and one terminal command.
  5. Run cargo clippy --all-targets --all-features -- -D warnings and cargo test --all-features.
- accept: Every command of design.md section 9 is accepted by the parser and says it is not built yet; help lists them; forwarding knows the local and terminal ones.
- verify: cargo test --all-features
- verified: 2026-11-21
- commit: 5d996f7
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-21, planner): forced set (owner was haiku-t-157)
- note (2026-11-21, planner): tests/cmd_agents.rs added: its help test lists every command and must get the 23 new ones (found by the worker).
- note (2026-11-21, haiku-t-157): done

### T-158 · init: journal line, the usage folder and --code-project
- status: done
- owner: haiku-t-158
- priority: P1 · difficulty: 1 · type: feature
- milestone: T-031
- why: Every change is saved with who and when, also the creation of a dossier (D29).
- depends: T-157
- files: src/cmd/init.rs, tests/cmd_init.rs, tests/common/mod.rs, tests/cmd_found.rs, tests/cmd_log.rs, tests/xos.rs, src/cmd/log.rs
- read: docs/design.md, docs/format.md, docs/requirements.md
- steps:
  1. After the dossier is created, append one journal line with crate::journal::append: time crate::time::now(), name and person from crate::names::unchecked(g.agent.as_deref()), llm false, command init, id DOSSIER, event created, detail 'code <code>' (D29).
  2. The usage folder is created like the other kind folders (Kind::all now has five kinds).
  3. Add #[arg(long)] code_project: bool. With it, dossier.toml gets the line 'code_project = true' right after the code line, and the output gets a line 'code project: map and code pointers are on'.
  4. Tests: the journal has one line with event created and the name given by DOSSIER_AGENT; without a name it has the operating-system user or unknown; DOSSIER/usage exists; with --code-project config::load gives code_project true.
  5. Run cargo test --test cmd_init.
- accept: init records who created the dossier and when, creates the usage folder, and --code-project marks a code project.
- verify: cargo test --test cmd_init
- verified: 2026-11-21
- commit: 77bd528
- reqs: D29, R-CODE-5
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-21, planner): forced set (owner was haiku-t-158)
- note (2026-11-21, planner): Test files added: the init journal line changes log output and the xos digest; tests/common runs init and people add as anna so the journal is the same on every machine; log shows the detail of records that are not entries.
- note (2026-11-21, haiku-t-158): done

### T-159 · people add: journal line
- status: done
- owner: haiku-t-159
- priority: P1 · difficulty: 1 · type: feature
- milestone: T-031
- why: Every change is saved with who and when, also settings changes (D29).
- depends: T-157
- files: src/cmd/people.rs, tests/cmd_people.rs, tests/cmd_log.rs, tests/xos.rs
- read: docs/design.md, docs/format.md, docs/requirements.md
- steps:
  1. After config::add_person succeeds (not with --dry-run), append one journal line: time crate::time::now(), name and person from crate::names::unchecked(g.agent.as_deref()), llm false, command people, id SETTINGS, event changed, detail 'person <name>' plus ' with <llm1>, <llm2>' when LLMs were given (D29). The state folder is <dossier>/.state.
  2. Tests: after people add anna --llm anna-claude the journal has a line with id SETTINGS and that detail; a dry run adds no line.
  3. Run cargo test --test cmd_people.
- accept: Adding a person is saved with who and when.
- verify: cargo test --test cmd_people
- verified: 2026-11-21
- commit: 66f54e4
- reqs: D29
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-21, planner): tests/cmd_log.rs and tests/xos.rs added: the people journal lines change log output and the xos digest.
- note (2026-11-21, haiku-t-159): done

### T-160 · agents install: journal line
- status: done
- owner: haiku-t-160
- priority: P1 · difficulty: 1 · type: feature
- milestone: T-031
- why: Every change is saved with who and when (D29).
- depends: T-157
- files: src/cmd/agents.rs, tests/cmd_agents.rs
- read: docs/design.md, docs/format.md, docs/requirements.md
- steps:
  1. After agents install --yes writes AGENTS.md: when crate::find::find(&current folder, g.dossier) finds a dossier, append one journal line: time now, name and person from crate::names::unchecked(g.agent.as_deref()), llm false, command agents, id AGENTS, event changed, detail 'rules block written'. Without a dossier write no journal line and no error.
  2. Tests: in a project made with init, the journal gets that line; in a folder without a dossier install still works and no .state folder appears.
  3. Run cargo test --test cmd_agents.
- accept: Writing the rules block is saved with who and when when a dossier is there.
- verify: cargo test --test cmd_agents
- verified: 2026-11-21
- commit: 1841a46
- reqs: D29
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-21, haiku-t-160): done

### T-161 · blockfile.rs: read and change block files line by line
- status: done
- owner: haiku-t-161
- priority: P0 · difficulty: 2 · type: feature
- milestone: T-031
- why: values.md, the file lists, exchange.md and map.md all use this module.
- final: Line-level editing has edge cases that are easier to settle in code than in steps.
- depends: T-157
- files: src/blockfile.rs
- read: docs/design.md, docs/format.md
- steps:
  1. Implement BlockFile as docs/design.md section 8 and format.md section 17 describe. Reuse the line handling of src/entry/mod.rs and src/entry/edit.rs (copy what you need; their helpers are private): lines keep their endings, a byte-order mark is kept, the format line is checked (above 1: Code::TooNew), a text without a line starting with '# ' is Code::Parse 'line 1: no heading'.
  2. A block starts at a line '## <key>' and ends before the next '## ' line or at the end. Its field lines are the '- name: value' lines in it; history lines are '- history: ...'.
  3. set_field replaces the first line with that name (keeping its ending), or inserts '- <name>: <value>' before the block's first history line, else after its last field line, else right after the blank line under the heading line of the block. An empty value removes the line. The name history changes nothing.
  4. add_block appends (when the text does not end with a blank line, first a blank line) '## <key>', a blank line, the fields with history pairs last, and a blank line. add_history appends '- history: <time> <names::label(who)>: <event>[ — <detail>]' after the block's last field line. New lines end as format.md section 2 says.
  5. Tests: the values.md example of format.md section 17 parses and text() equals it; keys, field, fields and history; each change touches only its own lines (compare whole texts); a CRLF file gets CRLF lines; add_block on BlockFile::empty gives the expected text; a block key with spaces works.
  6. Run cargo test --lib blockfile::.
- accept: Block files are read and changed line by line, keeping every other byte.
- verify: cargo test --lib blockfile::
- verified: 2026-11-21
- commit: c7bccb8
- reqs: R-EDIT-1, D29
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-21, haiku-t-161): done

### T-162 · tx.rs: put_raw and delete_raw for files that are not entries
- status: done
- owner: haiku-t-162
- priority: P0 · difficulty: 1 · type: feature
- milestone: T-031
- why: Every write goes through tx, also for values, file lists, exchange records and pictures.
- depends: T-157
- files: src/tx.rs
- read: docs/design.md, docs/format.md
- steps:
  1. Implement Tx::put_raw and Tx::delete_raw as docs/design.md section 7 says, as new queued operations next to the existing ones.
  2. In write(): check them with the existing check step (the file must still be the old bytes, or not exist for a new file); write with atomic::replace; save the old bytes as a snapshot under change.id only when the path ends with .md; append the journal record as for the other operations. Dry run: 'dry run: would write <path>' and 'dry run: would delete <path>'.
  3. Tests in the existing test module: a new block file is written with one journal line; a second put_raw over it makes snapshot 1; a picture (bytes, path ending .png) is written and no snapshot is made when it is deleted; a changed file gives Code::Changed; dry run writes nothing.
  4. Run cargo test --lib tx::.
- accept: Files other than entries are written and removed through the transaction, with snapshot and journal line.
- verify: cargo test --lib tx::
- verified: 2026-11-21
- commit: 2b2a525
- reqs: R-LOCK-1, R-IO-1, R-HIST-1, D29
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-21, haiku-t-162): done

### T-163 · hooks.rs: Claude Code hooks in settings.json and the guard check
- status: done
- owner: haiku-t-163
- priority: P1 · difficulty: 1 · type: feature
- milestone: T-031
- why: Claude Code hooks make brief and the edit guard part of every session (D33).
- depends: T-157
- files: src/hooks.rs
- read: docs/design.md, docs/format.md
- steps:
  1. Implement installed, uninstalled and guard as docs/design.md section 8 says. Use serde_json::Value. The SessionStart entry is {"hooks": [{"type": "command", "command": BRIEF}]} under hooks.SessionStart; the PreToolUse entry is {"matcher": "Edit|Write|MultiEdit", "hooks": [{"type": "command", "command": GUARD}]} under hooks.PreToolUse. Add each only when no entry with that command exists; keep every other key and entry.
  2. uninstalled removes only entries whose command is BRIEF or GUARD, and empty arrays or objects that became empty because of that.
  3. guard: parse the payload; take tool_input.file_path (or tool_input.path); when it is inside a folder named DOSSIER (any path component equal to DOSSIER) return Some('<path> is inside DOSSIER: change entries with the dossier commands, for example dossier set or dossier edit'); otherwise, and for anything that does not parse, None.
  4. Return to_string_pretty plus a final LF; an empty or blank text counts as {}; a text that is not a JSON object is Code::Parse.
  5. Tests: installing into {} and into a file with other hooks; installing twice gives the same text; uninstall after install gives back the other entries; guard blocks DOSSIER/decisions/DEC-001-x.md and allows src/main.rs.
  6. Run cargo test --lib hooks::.
- accept: settings.json gets the two dossier hooks once and loses only them; the guard spots edits inside DOSSIER.
- verify: cargo test --lib hooks::
- verified: 2026-11-21
- commit: 19a4773
- reqs: R-AGENT-3, R-CTX-7, D33
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-21, haiku-t-163): done

### T-164 · agents/mod.rs: the Claude Code skill text
- status: done
- owner: haiku-t-164
- priority: P1 · difficulty: 1 · type: feature
- milestone: T-031
- why: The skill gives Claude Code the same rules as AGENTS.md (D33).
- depends: T-157
- files: src/agents/mod.rs
- read: docs/design.md, docs/format.md
- steps:
  1. Implement skill(): the lines '---', 'name: dossier', 'description: Record and read the project's decisions, assumptions, findings and lessons with the dossier command.', '---', a blank line, then RULES with CRLF turned into LF and a final LF.
  2. Test: skill() starts with '---\nname: dossier\n' and contains the first line of RULES.
  3. Run cargo test --lib agents::.
- accept: The skill text is the rules with a front matter.
- verify: cargo test --lib agents::
- verified: 2026-11-21
- commit: 8f07873
- reqs: R-AGENT-4, D33
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-21, haiku-t-164): done

### T-165 · hooks install and uninstall
- status: done
- owner: haiku-t-165
- priority: P1 · difficulty: 1 · type: feature
- milestone: T-031
- why: The hooks run brief at session start and guard against direct edits (D33).
- depends: T-163
- files: src/cmd/hooks.rs, tests/cmd_hooks.rs
- read: docs/design.md, docs/format.md
- steps:
  1. Args: a subcommand Install { #[arg(long)] yes: bool } and Uninstall { #[arg(long)] yes: bool }. No dossier and no name are needed.
  2. The file is .claude/settings.json in the current folder (missing is empty). Without --yes print what would change and write nothing. With --yes: copy the old file to .claude/settings.json.bak when it exists, then write hooks::installed (or uninstalled) with crate::atomic::replace(path, bytes, None), creating .claude.
  3. When crate::find::find finds a dossier from the current folder, append a journal line as the agents command does: command hooks, id HOOKS, event changed, detail 'hooks installed' or 'hooks removed' (D29).
  4. Tests with assert_cmd in a tempfile folder: install --yes writes both hooks; a second install changes nothing; an existing other hook is kept and a .bak is made; uninstall --yes removes only the dossier hooks.
  5. Run cargo test --test cmd_hooks.
- accept: hooks install writes the SessionStart and PreToolUse hooks into .claude/settings.json and uninstall removes them.
- verify: cargo test --test cmd_hooks
- verified: 2026-11-21
- commit: 7a2329a
- reqs: R-AGENT-3, R-CTX-7, D33
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-21, haiku-t-165): done

### T-166 · guard: the PreToolUse hook that blocks direct edits of DOSSIER
- status: done
- owner: haiku-t-166
- priority: P1 · difficulty: 1 · type: feature
- milestone: T-031
- why: LLMs must use the commands, never edit entry files directly (R-AGENT-3).
- depends: T-163
- files: src/cmd/guard.rs, tests/cmd_guard.rs
- read: docs/design.md, docs/format.md
- steps:
  1. Read all of standard input as text. Call hooks::guard. Some(message): return Err(Error::new(Code::Usage, message)) so the exit code is 2, which Claude Code treats as 'block'. None: return Ok(Output::text("")) (exit 0, nothing printed).
  2. Tests with assert_cmd write_stdin: a payload editing DOSSIER/findings/FND-001-x.md exits 2 and stderr names DOSSIER; one editing README.md exits 0; garbage exits 0.
  3. Run cargo test --test cmd_guard.
- accept: guard blocks Claude Code edits inside DOSSIER with exit code 2 and allows everything else.
- verify: cargo test --test cmd_guard
- verified: 2026-11-21
- commit: 999a931
- reqs: R-AGENT-3
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-21, haiku-t-166): done

### T-167 · skill install
- status: done
- owner: haiku-t-167
- priority: P1 · difficulty: 1 · type: feature
- milestone: T-031
- why: The skill gives Claude Code the rules with examples (R-AGENT-4).
- depends: T-164
- files: src/cmd/skill.rs, tests/cmd_skill.rs
- read: docs/design.md, docs/format.md
- steps:
  1. Args: a subcommand Install { #[arg(long)] yes: bool }. Without --yes print 'would write .claude/skills/dossier/SKILL.md; add --yes'. With --yes write crate::agents::skill() to .claude/skills/dossier/SKILL.md in the current folder with crate::atomic::replace(path, bytes, None).
  2. When a dossier is found from the current folder, a journal line: command skill, id SKILL, event changed, detail 'skill written' (D29).
  3. Tests in a tempfile folder: install --yes writes the file starting with '---'; without --yes nothing is written.
  4. Run cargo test --test cmd_skill.
- accept: skill install writes the Claude Code skill.
- verify: cargo test --test cmd_skill
- verified: 2026-11-21
- commit: b38d972
- reqs: R-AGENT-4
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-21, haiku-t-167): done

### T-207 · brief: list what rests on a wrong assumption
- status: done
- owner: haiku-t-207
- priority: P1 · difficulty: 1 · type: feature
- milestone: T-041
- why: R-CTX-16: brief must print anything marked rests on a wrong assumption; status counts them but brief did not show them (found in the end-to-end run).
- files: src/cmd/brief.rs, tests/cmd_brief.rs
- read: docs/requirements.md, docs/design.md, docs/format.md
- steps:
  1. Add a part '## Rests on a wrong assumption' to brief (without --task), after the proposals: one line per approved entry whose rests-on field is not empty, '<ID> · <title> [rests on <IDs>]', and one line per non-detached block in any file list (filelist::tasks and load) whose rests-on field is not empty, '<task> <path> [rests on <IDs>]'. Leave the part out when it is empty. It is cut with the proposals when the line limit is reached.
  2. With --on, leave the part out (marks are about now).
  3. Tests in tests/cmd_brief.rs: after an assumption affecting T-001 and an attached file is refuted (assume, attach and refute are on main; refute needs DOSSIER_ANSWERS=yes through project.command), brief lists the entry and the file under the new heading; a brief without such marks has no such heading.
  4. Run cargo test --test cmd_brief.
- accept: brief lists every entry and file that rests on a wrong assumption.
- verify: cargo test --test cmd_brief
- verified: 2026-11-21
- commit: 38b4966
- reqs: R-CTX-16
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-21, haiku-t-207): done

### T-208 · status: map, open assumptions and what rests on a wrong assumption as lists
- status: done
- owner: haiku-t-208
- priority: P1 · difficulty: 1 · type: feature
- milestone: T-041
- why: R-CTX-16 and design.md section 9: status must print these, not only count them (found by the release check).
- files: src/cmd/status.rs, tests/cmd_status.rs
- read: docs/requirements.md, docs/design.md, docs/format.md
- steps:
  1. Add '## Map' to status (not with --on, not with --since), right after the project lines, with mapfile::load and mapfile::lines, at most 10 lines; leave it out when there are none.
  2. Under the decisions part, add '## Open assumptions' listing each approved assumption with state open as '<ID> · <title>', and '## Rests on a wrong assumption' listing each approved entry with a rests-on field ('<ID> · <title> [rests on <IDs>]') and each non-detached file block with one ('<task> <path> [rests on <IDs>]'); copy the way src/cmd/brief.rs (on main) builds these lists. Keep the counts in Unsettled. Leave each part out when empty; keep status_lines (cut these lists before decisions).
  3. JSON gains map, open_assumptions and rests (lists).
  4. Tests in tests/cmd_status.rs: a map line, an open assumption and a refuted assumption's marked entry and file appear in the lists.
  5. Run cargo test --test cmd_status.
- accept: status lists the map, the open assumptions and everything that rests on a wrong assumption.
- verify: cargo test --test cmd_status
- verified: 2026-11-21
- commit: 218a876
- reqs: R-CTX-16
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-21, haiku-t-208): done

### T-209 · README: three corrections found by the release check
- status: done
- owner: haiku-t-209
- priority: P1 · difficulty: 1 · type: docs
- milestone: T-041
- why: The README must match what the program does (REL-2).
- files: README.md
- read: docs/format.md, docs/design.md
- steps:
  1. Fix three things in README.md, running each changed example again with target/debug/dossier.exe in a temporary folder and copying the real output:
  2. (1) The runs example: dossier used refreshes an attached model that changed since (its file-list block gets the new checksum and a reattached line), so remove the separate reattach step after the run and say that used does this.
  3. (2) The files example: the date column is the file's modification time (format.md section 17, modified), not the attach date; make the text and the shown output say so.
  4. (3) 'What it needs': status works without todo and leaves the task parts out with one line; only brief --task, review-task and dossier todo need todo.
  5. Change nothing else.
- accept: The runs, files and requirements parts of the README match the program.
- verify: git ls-files --error-unmatch README.md
- verified: 2026-11-21
- commit: 0c877e9
- reqs: REL-2
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-21, haiku-t-209): done

### T-210 · v0.3 skeleton: how-to and tool kinds, team settings, Ctx and Global fields, guarded kinds
- status: done
- owner: haiku-t-210
- priority: P0 · difficulty: 2 · type: feature
- milestone: T-042
- why: New enum variants and struct fields break every literal and exhaustive match at once.
- summary: Adds the how-to and tool kinds and the new settings everywhere at once, so later tasks each change one file.
- final: New enum variants and struct fields touch every literal at once.
- files: src/entry/mod.rs, src/entry/new.rs, src/entry/rules.rs, src/config.rs, src/cli.rs, src/ctx.rs, src/tx.rs, src/create.rs, src/brief_task.rs, src/approval.rs, src/cmd/show.rs, src/cmd/review_task.rs, src/cmd/list.rs, src/cmd/search.rs, src/names.rs, src/filelist.rs
- read: docs/design.md, docs/format.md
- steps:
  1. entry/mod.rs: add Kind::Howto (name howto, prefix HOW, folder howtos) and Kind::Tool (name tool, prefix TOOL, folder tools) after Usage; Kind::all() returns [Kind; 7]. Extend its tests.
  2. entry/rules.rs and entry/new.rs: the sections, required parts and field order of format.md sections 22 and 23. Tests.
  3. approval.rs: add pub fn guarded(kind: Kind) -> bool, true for Lesson, Howto and Tool. Test.
  4. config.rs: parse the optional top-level keys of format.md sections 21 and 26 (kind, default "project"; team; members; names; past; describes) into the new Config fields of design.md section 10; add set_list and set_value as stubs returning Err(Code::NotImplemented). Add the new fields to every Config literal in this crate's tests (grep for 'Config {').
  5. cli.rs: Global gains --team (bool) and --from-document <file> (Option<String>), both global.
  6. ctx.rs: Ctx gains team: Option<PathBuf>, from_document: Option<String>, project: Option<PathBuf> (design.md section 10); Ctx::load fills from_document from g (as given) and sets team and project to None for now (a later task loads the team). team_ctx is a stub returning Err(Code::NotImplemented). Add the new fields to every Ctx literal in tests (tx.rs, create.rs, brief_task.rs, ctx.rs).
  7. Every exhaustive match on Kind or Status in the listed files gets the new variants (cargo build lists them).
  8. Run cargo build, cargo clippy --all-targets --all-features -- -D warnings, the same with --target x86_64-unknown-linux-gnu, and cargo test --all-features; fix every error inside these files.
- accept: The two new kinds, the new settings and the new Ctx and Global fields exist everywhere; every check passes.
- verify: cargo test --all-features
- verified: 2026-11-21
- commit: e7d121e
- reqs: D31, D34, D37, R-ENTRY-8, R-ENTRY-12
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-21, planner): forced set (owner was haiku-t-210)
- note (2026-11-21, planner): src/names.rs and src/filelist.rs added: their test modules build Config literals that need the new fields (found by the worker).
- note (2026-11-21, haiku-t-210): done

### T-211 · v0.3 skeleton: the new library modules as stubs
- status: done
- owner: haiku-t-211
- priority: P0 · difficulty: 2 · type: feature
- milestone: T-042
- why: With every type in place, each later task fills in one file.
- summary: Creates empty versions of the new library parts so each can be written on its own.
- final: Mechanical, but touches every new library file at once.
- depends: T-210
- files: src/lib.rs, src/conf.rs, src/gitcommit.rs, src/teamfiles.rs, src/glossary.rs, src/imports.rs, src/folderscan.rs, src/hooks.rs
- read: docs/design.md, docs/format.md
- steps:
  1. Add one pub mod line to src/lib.rs per module of docs/design.md section 11.
  2. Create each module of section 11 with every type, constant and signature exactly as written there, with their doc comments.
  3. Add the two hooks.rs functions of section 10 (local_installed, local_uninstalled) as stubs; change nothing else in hooks.rs.
  4. Bodies: Err(Error::new(Code::NotImplemented, "<module>::<function>")) for a Result, else a plain default; unused parameters get a leading underscore. No unwrap, expect, panic or allow attributes.
  5. Run both clippy commands (Windows and --target x86_64-unknown-linux-gnu) and cargo test --all-features.
- accept: Every module of design.md section 11 exists with its exact signatures; every check passes.
- verify: cargo clippy --all-targets --all-features -- -D warnings
- verified: 2026-11-21
- commit: 0e04f46
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-21, haiku-t-211): done

### T-212 · v0.3 skeleton: command stubs, help overview and forwarding lists
- status: done
- owner: haiku-t-212
- priority: P0 · difficulty: 2 · type: feature
- milestone: T-042
- why: After this, a command task only fills in its own file.
- summary: Creates empty versions of the new commands so each can be written on its own.
- final: Mechanical, but touches every new command file at once.
- depends: T-211
- files: src/cli.rs, src/cmd/mod.rs, src/cmd/help.rs, src/remote/forward.rs, tests/cmd_agents.rs, src/cmd/team.rs, src/cmd/confidential.rs, src/cmd/promote.rs, src/cmd/close.rs, src/cmd/howto.rs, src/cmd/tool.rs, src/cmd/where_used.rs, src/cmd/term.rs, src/cmd/glossary.rs, src/cmd/reviewed.rs, src/cmd/review_folder.rs, src/cmd/review_document.rs, src/cmd/reviewed_document.rs, src/cmd/import.rs
- read: docs/design.md, docs/format.md
- steps:
  1. For every new command of docs/design.md section 12 create src/cmd/<name>.rs as the earlier stubs (Args {} and run returning Err(Code::NotImplemented, "<command> is not built yet")); where-used is where_used.rs, review-folder review_folder.rs, review-document review_document.rs, reviewed-document reviewed_document.rs.
  2. Add the mod lines to src/cmd/mod.rs, the Command variants and dispatch lines to src/cli.rs (names as in section 12), and each command to the help overview of src/cmd/help.rs; add the 14 names to the expected list of the help test in tests/cmd_agents.rs.
  3. src/remote/forward.rs: close and reviewed ask ssh for a terminal like approve; extend its test.
  4. Run cargo clippy --all-targets --all-features -- -D warnings and cargo test --all-features.
- accept: Every command of design.md section 12 is accepted by the parser and says it is not built yet.
- verify: cargo test --all-features
- verified: 2026-11-21
- commit: 26f6552
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-21, haiku-t-212): done

### T-213 · The rules text for LLMs: phase 3
- status: done
- owner: haiku-t-213
- priority: P1 · difficulty: 1 · type: feature
- milestone: T-042
- why: R-AGENT-1.
- summary: Teaches LLMs the phase 3 rules.
- depends: T-212
- files: src/agents/rules.md
- read: docs/requirements.md, docs/design.md
- steps:
  1. Add, keeping the file at most 40 lines: 'Write under your own name: <your person> - <your model>, for example `tony - claude opus 5.5 high` (DOSSIER_AGENT).'; 'When your person says a name or term is confidential, record it at once: `dossier confidential add "<term>"`.'; 'Never write a confidential term into the team dossier; propose team knowledge with `dossier promote <ID>`.'; 'Before writing a script, look for a tool: `dossier list --kind tool`.'; 'When drafting from an old document, add `--from-document <file>`.'
  2. Run cargo test --test cmd_agents.
- accept: The rules tell an LLM about names, confidential terms, promotion, tools and imports.
- verify: cargo test --test cmd_agents
- verified: 2026-11-22
- commit: e315467530c675cd2f74ffee384c98758d9c1e9a
- reqs: R-AGENT-1
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-22, haiku-t-213): done

### T-214 · README for v0.3
- status: done
- owner: haiku-t-214
- priority: P1 · difficulty: 1 · type: docs
- milestone: T-042
- why: REL-2.
- summary: Documents the phase 3 features with tested examples.
- depends: T-236, T-248, T-267
- files: README.md
- read: docs/requirements.md, docs/design.md, docs/format.md
- steps:
  1. Status line 'Status: v0.3 (phase 3)'. Move the phase 3 items from 'Not in version 0.2' to 'What version 0.3 adds', with one example each, each run yourself first in temporary folders: the team dossier (init --team-dossier, team use, team member add, team name add), names (your username; an LLM as '<you> - <model>'; hooks install sets it for Claude Code), confidential terms, promote and close, how-tos, tools, where-used, the glossary, news (status --since last), review dates, and bringing in finished projects (init --past, review-folder, review-document, --from-document, import status). Invented names only.
- accept: The README explains phase 3 with tested examples.
- verify: git ls-files --error-unmatch README.md
- verified: 2026-11-22
- commit: a99175439c0cb3ee63ccfb45dcb6a45d56fb22d1
- reqs: REL-2
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-22, haiku-t-214): done

### T-215 · Release 0.3.0
- status: done
- owner: haiku-t-215
- priority: P1 · difficulty: 2 · type: chore
- milestone: T-042
- why: Phase 3 ends with a release used on real work.
- summary: Prepares the 0.3.0 release.
- final: Checking every command by hand cannot be listed in advance.
- depends: T-213,T-214,T-279,T-043,T-044,T-045,T-046,T-047,T-048,T-049,T-050,T-051,T-210,T-211,T-212
- files: Cargo.toml, Cargo.lock
- read: docs/requirements.md
- steps:
  1. Set the version to 0.3.0 and run cargo build so Cargo.lock follows.
  2. Run every check on Windows and cargo clippy --target x86_64-unknown-linux-gnu --all-targets --all-features -- -D warnings; run each phase 3 command once by hand in temporary folders; list every problem in the report without fixing it.
  3. Do not push or tag.
- accept: Version 0.3.0 builds and every check passes.
- verify: cargo build
- verified: 2026-11-22
- commit: ec8d43f8a869945d7d7f211a57fcf399d458c758
- reqs: REL-1
- by: Reynan Henry Tandoyo
- note (2026-11-21, planner): added
- note (2026-11-22, haiku-t-215): done

### T-269 · show team:<ID>: read an entry of the team dossier
- status: done
- owner: haiku-t-269
- priority: P1 · difficulty: 1 · type: feature
- milestone: T-043
- why: R-ID-2: commands accept an ID with its place; search marks team results team:<ID>, so show must find them (found by the T-250 worker).
- summary: Lets you open a team dossier entry from inside a project.
- depends: T-218, T-258
- files: src/cmd/show.rs, tests/cmd_show.rs
- read: docs/requirements.md, docs/design.md
- steps:
  1. When the ID starts with 'team:' (any case) and the project has a team dossier (ctx.team), read the entry from the team dossier with store::get(team, id) and show it the same way, marking the first line with 'team:' before the ID in the JSON id field; without a team dossier: Code::NoDossier naming 'dossier team use <path>'.
  2. Other place prefixes ('<code>:') keep working as now (store::get strips them).
  3. Tests in tests/cmd_show.rs: with a team (init --team-dossier in a second folder, add anna to its members with dossier::config::set_list, link with dossier::config::set_value(project, "team", ...)), a team lesson written with 'dossier learn ... --team' is shown with 'show team:LES-001'; without a team, team:LES-001 gives exit 4.
  4. Run cargo test --test cmd_show.
- accept: show team:<ID> shows the team dossier's entry from a project.
- verify: cargo test --test cmd_show
- verified: 2026-11-22
- commit: 25ecdde4bd5c48071f51a316f481ba756fb233b7
- reqs: R-ID-2
- by: Reynan Henry Tandoyo
- note (2026-11-22, planner): added
- note (2026-11-22, haiku-t-269): done

### T-270 · brief --task: a promoted entry is listed once
- status: done
- owner: haiku-t-270
- priority: P1 · difficulty: 1 · type: feature
- milestone: T-048
- why: R-CTX-2, R-TEAM-4: a lesson promoted to the team must not appear twice in Related.
- summary: In the Related part of brief --task, a team entry copied from one of this project's entries is left out.
- depends: T-253,T-252
- files: src/brief_task.rs
- read: docs/design.md, docs/format.md
- steps:
  1. In Related (team lessons and how-tos, T-252), leave out a team entry whose source field is '<this project code>:<ID>' (case-insensitive code) of a project entry already listed.
  2. Unit test in the module: a project lesson LES-001 and a team lesson with source 'P-001:LES-001' give one line, the project's.
  3. Run cargo test --lib brief_task::.
- accept: A promoted lesson appears once in brief --task.
- verify: cargo test --lib brief_task::
- verified: 2026-11-22
- commit: 6265bc2b7e70314df8a219ee9f079b9f5ceab946
- by: Reynan Henry Tandoyo
- note (2026-11-22, coordinator): added
- note (2026-11-22, haiku-t-270): done

### T-271 · init --dry-run: create nothing
- status: done
- owner: haiku-t-271
- priority: P2 · difficulty: 1 · type: bug
- why: R-WRITE: --dry-run never writes; two workers found that plain init --dry-run still creates DOSSIER (only --with-team honours it).
- summary: init with --dry-run prints what it would create and creates nothing.
- files: src/cmd/init.rs, tests/cmd_init.rs
- steps:
  1. With the global --dry-run, init (without --with-team as well) prints the folders and files it would create as warnings and writes nothing.
  2. Test: init --dry-run in an empty temporary folder leaves it empty and exits 0.
  3. Run cargo test --test cmd_init.
- accept: init --dry-run leaves the folder untouched.
- verify: cargo test --test cmd_init
- verified: 2026-11-22
- commit: eb875af6889feb993d703382cbcb97dc5ed9e427
- by: Reynan Henry Tandoyo
- note (2026-11-22, coordinator): added
- note (2026-11-22, haiku-t-271): done

### T-272 · show: a how-to not tested with the newest version
- status: done
- owner: haiku-t-272
- priority: P1 · difficulty: 1 · type: feature
- milestone: T-050
- why: R-LIFE-8: a how-to goes out of date quietly when the software moves on.
- summary: A how-to is marked 'not tested with <version>' when the software was used in a newer version than the one it was tested with.
- files: src/cmd/show.rs, tests/cmd_show.rs
- read: docs/requirements.md, docs/format.md, src/cmd/used.rs, src/teamfiles.rs
- steps:
  1. The versions a how-to was tested with: each line of its 'Tested with' section; the version of a line is its last word that contains a digit. The newest is the highest by comparing the dot- or dash-separated parts as numbers where both are numbers, else as text.
  2. The newest use of the how-to's software (field software, ignoring case): the usage entries of the dossier (Kind::Usage, fields software, version, and the entry's created time) and, when the project has a team dossier (ctx.team), the blocks of the team's usage-index.md (teamfiles::usage: software, version, date); the newest by time or date wins.
  3. When the newest use has a version and it is higher than the newest tested version (or the how-to has no tested version at all), show adds the mark '[not tested with <version>]' after the other marks. Put the rule in one pub(crate) function in show.rs, not_tested_with(ctx: &Ctx, entry: &Entry) -> Option<String>, so that list can call it.
  4. Tests: a how-to tested with 2410 and a usage record of the same software 2502 is marked; tested with 2502 is not; another software's record does not count.
  5. Run cargo test --test cmd_show.
- accept: show marks a how-to whose software was used in a newer version than it was tested with.
- verify: cargo test --test cmd_show
- verified: 2026-11-22
- commit: 27212f60ac3b3a14e9d0e615336697c938712e21
- by: Reynan Henry Tandoyo
- note (2026-11-22, coordinator): added
- note (2026-11-22, haiku-t-272): done

### T-273 · list: a how-to not tested with the newest version
- status: done
- owner: haiku-t-273
- priority: P1 · difficulty: 1 · type: feature
- milestone: T-050
- why: R-LIFE-8.
- summary: list marks a how-to 'not tested with <version>' as show does.
- depends: T-272
- files: src/cmd/list.rs, tests/cmd_list.rs
- read: src/cmd/show.rs
- steps:
  1. Add the mark '[not tested with <version>]' to a how-to's line in list, from crate::cmd::show::not_tested_with (T-272).
  2. Test: a how-to tested with 2410 and a usage record of the same software 2502 is marked in list.
  3. Run cargo test --test cmd_list.
- accept: list shows the same mark as show.
- verify: cargo test --test cmd_list
- verified: 2026-11-22
- commit: 520741977d8c672135b298b30cbe3822e53b355a
- by: Reynan Henry Tandoyo
- note (2026-11-22, coordinator): added
- note (2026-11-22, haiku-t-273): done

### T-274 · entry: how-tos may have an Open remarks section
- status: done
- owner: haiku-t-274
- priority: P1 · difficulty: 1 · type: feature
- milestone: T-046
- why: R-USE-6, format.md section 22: how-tos get Open remarks after Tested with.
- summary: A how-to may carry the section Open remarks, last.
- files: src/entry/new.rs, src/entry/rules.rs
- read: docs/format.md
- steps:
  1. Add "Open remarks" after "Tested with" to the how-to sections in src/entry/new.rs and src/entry/rules.rs (format.md section 22), and update the tests in both files that list the sections.
  2. Run cargo test --lib entry::.
- accept: A how-to with an Open remarks section is valid and the section sorts last.
- verify: cargo test --lib entry::
- verified: 2026-11-22
- commit: f6fbcc84fe2054110c2c02d7611c794d4bfd572b
- by: Reynan Henry Tandoyo
- note (2026-11-22, coordinator): added
- note (2026-11-22, haiku-t-274): done

### T-275 · used: a deviation adds an open remark to the team how-to
- status: done
- owner: haiku-t-275
- priority: P1 · difficulty: 1 · type: feature
- milestone: T-046
- why: R-USE-6: the owner of a how-to sees what to update.
- summary: A usage record with deviations that names a team how-to adds a line to that how-to's Open remarks in the team dossier.
- depends: T-274
- files: src/cmd/used.rs, tests/cmd_used.rs
- read: docs/format.md, docs/requirements.md, src/approval.rs, src/conf.rs
- steps:
  1. When used is given --howto <ID> and --deviations <text>, and the how-to is the team's (the ID starts with 'team:' in any case, or the project has no entry with that ID and the team dossier has a how-to with it), and the project has a team dossier: check the deviation with conf::gate and conf::check first (a refusal refuses the whole command before anything is written).
  2. In a tx::write on ctx.team_ctx(): add the line '- <project code> <YYYY-MM-DD>: <deviation>' at the end of the how-to's Open remarks section (create the section, last, when missing), with a history line 'remark' and detail the project code; when approval::mark gave Unchanged before the change, approval::record the new bytes so the remark is not a hand change (format.md section 22).
  3. Tests: a team how-to (approved by anna with --team after 'dossier confidential add "NT-999"') gets the line, stays unchanged for show (no [changed after approval]); a deviation holding NT-999 is refused; without --deviations nothing is added.
  4. Run cargo test --test cmd_used.
- accept: A deviation reaches the team how-to as one open remark, after the confidentiality check.
- verify: cargo test --test cmd_used
- verified: 2026-11-22
- commit: a30175f7e692d97e782808c656d9079b6d952d01
- by: Reynan Henry Tandoyo
- note (2026-11-22, coordinator): added
- note (2026-11-22, haiku-t-275): done

### T-276 · check: abbreviations missing from the glossary
- status: done
- owner: haiku-t-276
- priority: P1 · difficulty: 1 · type: feature
- milestone: T-048
- why: R-TERM-3.
- summary: check lists abbreviations of two to six capital letters that appear in at least three entries and are in no glossary.
- files: src/cmd/check.rs, tests/cmd_check.rs
- read: docs/requirements.md, src/glossary.rs
- steps:
  1. Split the title and section text of every entry into words at characters that are not letters or digits. An abbreviation is a word of 2 to 6 ASCII capital letters that is not an entry ID prefix (DEC, FND, LES, ASM, HOW, TOOL, USE and every other Kind prefix) and not the project code.
  2. Count the entries each abbreviation appears in; when it is 3 or more and it is not a term, symbol or other spelling (ignoring case) of the project's or the team's glossary (glossary::load, glossary::terms), report it with the code TERM: '<ABBR> appears in <n> entries and is in no glossary; add it with: dossier term add <ABBR> "<meaning>"', at the first entry's file and line where it appears. A warning, like the other SHOULD reports.
  3. Tests: CFD in three entries and no glossary is reported; in two entries it is not; with 'term add CFD ...' it is not; LES-001 references are not abbreviations.
  4. Run cargo test --test cmd_check.
- accept: check reports a frequent abbreviation that no glossary explains.
- verify: cargo test --test cmd_check
- verified: 2026-11-22
- commit: 187a00f97584e7fabb2ba0cde9c23d2de062470c
- by: Reynan Henry Tandoyo
- note (2026-11-22, coordinator): added
- note (2026-11-22, haiku-t-276): done

### T-277 · status --since: no settings rows, --keep, LLMs never move the marker
- status: done
- owner: haiku-t-277
- priority: P1 · difficulty: 1 · type: bug
- milestone: T-049
- why: R-READ-10, R-CTX-10: the README run showed settings records (DOSSIER, SETTINGS, AGENTS, HOOKS, SKILL) as '(removed)' lines in the project part, and --keep is missing.
- summary: status --since lists entry changes only; --keep leaves the marker; an LLM never moves its person's marker.
- files: src/cmd/status.rs, tests/cmd_status.rs
- read: docs/requirements.md, docs/format.md
- steps:
  1. In the project part of status --since, leave out journal records whose ID is not an entry ID (the settings IDs DOSSIER, SETTINGS, AGENTS, HOOKS, SKILL and anything without an entry prefix; format.md section 11).
  2. Add --keep: with --since last the marker is not moved. --since <date> never moves it (check it stays so).
  3. When the writer is an LLM ('<person> - <model>' or a registered LLM), --since last reads its person's marker and never moves it (R-READ-10).
  4. Tests for each; update existing tests only where they expected settings rows.
  5. Run cargo test --test cmd_status.
- accept: status --since shows no settings rows; --keep and LLM runs leave the marker.
- verify: cargo test --test cmd_status
- verified: 2026-11-22
- commit: 98e800ebcb5acedf7c93eba52c893100bd3a62ac
- by: Reynan Henry Tandoyo
- note (2026-11-22, coordinator): added
- note (2026-11-22, haiku-t-277): done

### T-278 · init --team-dossier: the person who creates it is the first member
- status: done
- owner: haiku-t-278
- priority: P1 · difficulty: 1 · type: bug
- milestone: T-043
- why: R-TEAM-7: only members write to the team dossier and team member add needs a member, so a new team dossier could only get its first member by editing dossier.toml by hand.
- summary: A new team dossier lists its creator as its first member.
- files: src/cmd/init.rs, tests/cmd_init.rs
- read: docs/format.md, src/names.rs
- steps:
  1. In init --team-dossier, write members = ["<name>"] where <name> is the writer's name (--agent, DOSSIER_AGENT, TODO_AGENT, then USERNAME or USER; the part before the first ' - ' when it has that form). No name: members stays empty and a warning says to add one with 'team member add'.
  2. Tests: init --team-dossier as anna gives members ["anna"]; as 'anna - claude' also ["anna"]; after it, 'team member add ben' by anna in the team dossier works.
  3. Run cargo test --test cmd_init.
- accept: A new team dossier can be used without editing dossier.toml.
- verify: cargo test --test cmd_init
- verified: 2026-11-22
- commit: 4b6391f
- by: Reynan Henry Tandoyo
- note (2026-11-22, coordinator): added
- note (2026-11-22, haiku-t-278): done

### T-279 · README: the first team member and status --keep
- status: done
- owner: haiku-t-279
- priority: P1 · difficulty: 1 · type: docs
- milestone: T-042
- why: T-278 made the creator of a team dossier its first member and T-277 added --keep; the README still says to edit dossier.toml by hand.
- summary: The README matches init --team-dossier and status --since after T-277 and T-278.
- files: README.md
- steps:
  1. In 'What version 0.3 adds', replace the paragraph and toml block that say to add the first member by hand: init --team-dossier makes the person who runs it the first member (show the members line it writes); keep 'team member add ben' and 'team name add'.
  2. In 'What is new', add one sentence: --keep leaves the marker where it is, and an LLM's run never moves its person's marker.
  3. Run each changed example yourself with target/debug/dossier.exe in a temporary folder outside the repository and paste the real output.
  4. Run git ls-files --error-unmatch README.md.
- accept: The README examples match the program.
- verify: git ls-files --error-unmatch README.md
- verified: 2026-11-22
- commit: f1780cb53aab551344e34702d57ac0e022807b79
- by: Reynan Henry Tandoyo
- note (2026-11-22, coordinator): added
- note (2026-11-22, haiku-t-279): done

### T-280 · team member remove: the last member stays
- status: done
- owner: haiku-t-280
- priority: P1 · difficulty: 1 · type: bug
- milestone: T-042
- why: R-TEAM-7: found at the 0.3.0 release run; removing the last member leaves a team dossier nobody can write to, and team member add then fails.
- summary: team member remove refuses to remove the last member.
- files: src/cmd/team.rs, tests/cmd_team.rs
- steps:
  1. In team member remove, when the name is the only member left, refuse with Code::Refused: '<name> is the last member; add another member first: dossier team member add <username>'. Nothing is written.
  2. Test: in a team dossier whose only member is anna, 'team member remove anna' (a person at a terminal) is refused and members stays ["anna"].
  3. Run cargo test --test cmd_team.
- accept: A team dossier always keeps at least one member.
- verify: cargo test --test cmd_team
- verified: 2026-11-22
- commit: 5e2c9126b2eb326707d74591668a8f712f79288f
- by: Reynan Henry Tandoyo
- note (2026-11-22, coordinator): added
- note (2026-11-22, haiku-t-280): done

### T-281 · tool add: the help says Purpose is required
- status: done
- owner: haiku-t-281
- priority: P1 · difficulty: 1 · type: docs
- milestone: T-042
- why: Found at the 0.3.0 release run: tool add without a Purpose section fails, and --help does not say it is needed (format.md section 22).
- summary: tool add --help names Purpose as required.
- files: src/cmd/tool.rs
- steps:
  1. In the doc comments of tool add (the clap help), say that a Purpose section is required: --section 'Purpose=<what the tool is for>', --from or --stdin. Change nothing else.
  2. Run cargo test --test cmd_tool.
- accept: dossier tool add --help names the required Purpose section.
- verify: cargo test --test cmd_tool
- verified: 2026-11-22
- commit: 5b8809c306647771bf97eb2a4d90f6f163f4c890
- by: Reynan Henry Tandoyo
- note (2026-11-22, coordinator): added
- note (2026-11-22, haiku-t-281): done

### T-282 · CI: actions that run on Node.js 24
- status: done
- owner: coordinator
- priority: P2 · difficulty: 1 · type: chore
- why: GitHub warns that actions/checkout@v4 and the artifact actions v4 run on Node.js 20, which is being retired.
- summary: The CI and release workflows use action versions that run on Node.js 24.
- files: .github/workflows/ci.yml, .github/workflows/release.yml
- steps:
  1. actions/checkout@v4 -> @v7 (node24) in both workflows.
  2. actions/upload-artifact@v4 -> @v7 and actions/download-artifact@v4 -> @v8 (node24) in release.yml; upload by name (zipped) and download by pattern with merge-multiple are unchanged by their breaking changes.
  3. Push and see the CI run pass without the Node.js 20 warning.
- accept: CI runs without the Node.js 20 deprecation warning.
- verify: git ls-files --error-unmatch .github/workflows/ci.yml
- verified: 2026-11-22
- commit: 448eeba
- by: Reynan Henry Tandoyo
- note (2026-11-22, coordinator): added
- note (2026-11-22, coordinator): done

### T-283 · Hooks after todo commands: brief, review material and reminders appear by themselves
- status: open
- owner: —
- priority: P3 · type: milestone
- milestone: T-055
- goal: R-AGENT-5 holds: after a task is claimed, finished or given back with todo, a Claude Code hook puts the dossier's context for that moment in front of the agent.
- why: A rule that an agent must remember is followed only some of the time; a hook runs every time (D39).
- summary: When an LLM claims, finishes or gives back a task, the dossier's context for that moment is shown to it automatically, so it no longer depends on the LLM remembering the rules.
- accept: R-AGENT-5 holds.
- reqs: R-AGENT-5
- note (2026-11-22, planner): added
- note (2026-11-22, planner): Split when its turn comes: needs a design in docs/design.md first (a hidden command like guard, and a change to hooks.rs). Check against Claude Code's documentation how the output of a hook that runs after a shell command reaches the agent's context. Depends on nothing in phase 4, so it may be built before it.
- note (2026-11-22, planner): split into T-293, T-294

### T-293 · hooks.rs: the hook after shell commands, and reading a todo command from its payload
- status: done
- owner: haiku-T-293
- priority: P0 · difficulty: 1 · type: feature
- milestone: T-283
- why: A rule that an agent must remember is followed only some of the time; a hook runs every time (D39).
- summary: The settings of Claude Code get a hook that runs after every shell command, and the part that recognises a todo command in it.
- files: src/hooks.rs, tests/cmd_hooks.rs, README.md
- read: docs/design.md, docs/format.md, docs/requirements.md
- steps:
  1. src/hooks.rs: add AFTER, SHELL_TOOLS, AFTER_TIMEOUT, TodoEvent, todo_event and context exactly as in docs/design.md section 14.
  2. `installed` also adds, once, the PostToolUse entry of docs/format.md section 29 (matcher SHELL_TOOLS, command AFTER, "timeout": AFTER_TIMEOUT). `add_once` builds entries without a timeout today: give it the timeout as one more Option<u64> argument and pass None for the two existing hooks. `is_dossier` also accepts AFTER, so that `uninstalled` removes the entry.
  3. todo_event: parse the payload with serde_json after `without_mark`; anything that is not an object with a string `tool_input.command` is None. Follow the rules and the table of design.md section 14: cut the command at `&&`, `||`, `;`, `|` and line breaks; cut each part into words at spaces and remove one pair of surrounding `"` or `'` from a word; a part counts when a word is `todo` (compare the text after the last `/` or `\`, without a trailing `.exe`, case ignored) or is `dossier` (same comparison) directly followed by `todo`; look only at the words after it. A part with `--dry-run`, `--help` or `-h` is no event. A task ID is ASCII letters, `-`, ASCII digits and nothing else; return it in upper case. For `next` with `--claim` take the first word of `tool_response.stdout` (cut at spaces and line breaks) that is a task ID; no such word: None.
  4. context: serde_json::json!({"hookSpecificOutput": {"hookEventName": "PostToolUse", "additionalContext": text}}).to_string().
  5. Unit tests in src/hooks.rs: installing adds the three entries and twice gives the same text; uninstalling gives back the other entries; todo_event for `todo done T-012 --commit HEAD --agent a` (Done), `cd /x && todo stuck t-7 --note "no data"` (Stuck T-7), `todo claim T-3` (Claimed), `dossier todo done T-9` (Done), `C:\\tools\\todo.exe done T-1` (Done), `todo next --claim --brief --agent a` with stdout `T-053 · P2 · claimed · a · Title` (Claimed T-053), and None for `todo next --brief`, `todo done T-1 --dry-run`, `todo list`, `echo todo done`, `git commit -m "todo done T-1"`, a payload without a command, and text that is not JSON; context gives one line that parses back to the text.
  6. tests/cmd_hooks.rs and README.md: `hooks install` now lists a third line `+ PostToolUse (Bash|PowerShell): dossier after`; update every expected output and the two sample outputs in the README (search for `+ SessionStart: dossier brief`), and add two sentences to the README paragraph below them: after a task is claimed, finished or given back with todo, the hook shows the agent the task's brief, the review material, or a reminder.
  7. Run cargo test --lib hooks:: and cargo test --test cmd_hooks
- accept: hooks install adds the PostToolUse hook and uninstall removes it; todo_event tells a claimed, finished or given-back task from the payload and nothing else.
- verify: cargo test --lib hooks:: && cargo test --test cmd_hooks
- verified: 2026-11-24
- commit: 9445a7e
- reqs: R-AGENT-5, D39
- by: Reynan Henry Tandoyo
- note (2026-11-22, planner): added
- note (2026-11-24, haiku-T-293): done

### T-294 · after: show the agent what the dossier has after a todo command
- status: open
- owner: —
- priority: P1 · type: milestone
- milestone: T-283
- goal: The command `after` exists, runs locally, and after a claimed, finished or given-back task prints the context of docs/design.md section 14; it never fails.
- why: A rule that an agent must remember is followed only some of the time; a hook runs every time (D39).
- summary: The command behind the hook: it turns a claimed, finished or given-back task into the text the agent should read at that moment.
- depends: T-293
- accept: After a claim the agent's context gets brief --task, after done the review material and the usage reminder, after stuck the reminder; every other command gives nothing, and the command never fails.
- reqs: R-AGENT-5, D39, D38
- note (2026-11-22, planner): added
- note (2026-11-23, planner): split into T-398, T-399, T-400

### T-398 · after: the command file and its place in the command list
- status: open
- owner: —
- priority: P0 · difficulty: 1 · type: chore
- milestone: T-294
- why: A rule that an agent must remember is followed only some of the time; a hook runs every time (D39).
- summary: An empty place for the command behind the new hook.
- files: src/cmd/after.rs, src/cmd/mod.rs, src/cli.rs
- read: docs/design.md, docs/format.md, docs/requirements.md
- steps:
  1. Create src/cmd/after.rs with `#[derive(clap::Args, Debug)] pub struct Args {}` and `pub fn run(_args: &Args, _g: &Global) -> Result<Output>` that returns Ok(Output::text("")), with the `use` lines of src/cmd/guard.rs.
  2. src/cmd/mod.rs: add `pub mod after;` in alphabetical order. src/cli.rs: add `After(cmd::after::Args),` to the Command enum and `Command::After(a) => ("after", cmd::after::run(a, g)),` to the match in main_exit.
  3. Run cargo build and cargo test --test smoke
- accept: `dossier after` exists and prints nothing.
- verify: cargo build && cargo test --test smoke
- reqs: R-AGENT-5
- note (2026-11-23, planner): added

### T-399 · after: in the help overview, and run on the PC
- status: open
- owner: —
- priority: P1 · difficulty: 1 · type: chore
- milestone: T-294
- why: A rule that an agent must remember is followed only some of the time; a hook runs every time (D39).
- summary: The hook command is listed and stays on the PC, where Claude Code runs.
- depends: T-398
- files: src/cmd/help.rs, src/remote/forward.rs, tests/cmd_agents.rs
- read: docs/design.md, docs/format.md, docs/requirements.md
- steps:
  1. src/cmd/help.rs: add ("after", "Adds context after todo commands") to COMMANDS, next to guard, and raise the array length in its type by one.
  2. src/remote/forward.rs: add "after" to LOCAL_COMMANDS and raise its length by one; extend the comment above it: `after` reads the payload of the Claude Code on the PC.
  3. tests/cmd_agents.rs: help_without_a_topic_lists_every_command_in_order compares the whole list that `dossier help` prints; when the new commands now appear in it, add them to the expected list at the place where `dossier help` prints them, and change nothing else in that file.
  4. Run cargo test --lib remote::forward:: and cargo test --test smoke
- accept: The overview lists `after`, and in a connected folder it runs on the PC and is not sent to the server.
- verify: cargo test --lib remote::forward:: && cargo test --test smoke
- reqs: R-AGENT-5
- note (2026-11-23, planner): added
- note (2026-11-23, planner): Added tests/cmd_agents.rs: its help overview test lists every command (found in T-359).

### T-400 · after: the text for a claimed, finished or given-back task
- status: open
- owner: —
- priority: P1 · difficulty: 1 · type: feature
- milestone: T-294
- why: A rule that an agent must remember is followed only some of the time; a hook runs every time (D39).
- summary: The command turns a claimed, finished or given-back task into the text the agent should read at that moment.
- depends: T-398, T-293
- files: src/cmd/after.rs, tests/cmd_after.rs
- read: docs/design.md, docs/format.md, docs/requirements.md
- steps:
  1. run: read standard input to a string (a read error counts as empty); hooks::todo_event(&payload); None: return Ok(Output::text("")). This command never returns an error.
  2. A private helper `own(args: &[&str]) -> Option<String>`: runs std::env::current_exe() with these arguments in the current folder, standard input null, output captured; Some(standard output as text) only when it started and ended with exit 0.
  3. Build the text as the table of docs/design.md section 14 says. Claimed(id): own(["brief", "--task", id]). Done(id): own(["review-task", id]); then own(["list", "--kind", "usage", "--task", id, "--all", "--json"]) and, when it answered and the `data` of its JSON holds no entry, the line `No usage record for <id>: if software was used, record it with dossier used <software> --version <v> --task <id>`. Stuck(id): the fixed text of the table. Parts are joined with one empty line.
  4. Text empty: Ok(Output::text("")); otherwise Ok(Output::text(hooks::context(&text))).
  5. tests/cmd_after.rs (new): write a todo stand-in as tests/cmd_review_task.rs does, record one finding for T-001, and pipe payloads with `.write_stdin` as tests/cmd_guard.rs does. `todo next --claim` with a tool_response.stdout naming T-001: exit 0 and the output is one JSON line whose additionalContext contains the finding's title. `todo done T-001`: additionalContext contains `# Review of T-001` and `No usage record for T-001`; after `dossier used` recorded one for T-001 that line is gone. `todo stuck T-001`: contains `--dead-end`. `git status`, an empty payload and text that is not JSON: exit 0 and empty output. With the todo stand-in missing, `todo done T-001` still exits 0.
  6. Run cargo test --test cmd_after
- accept: After a claim the agent's context gets brief --task, after done the review material and the usage reminder, after stuck the reminder; every other command gives nothing.
- verify: cargo test --test cmd_after
- reqs: R-AGENT-5, D39, D38
- note (2026-11-23, planner): added

### T-284 · Credentials are never stored in a dossier
- status: done
- owner: planner
- priority: P2 · type: milestone
- goal: No write to a project dossier or the team dossier stores a private key, an access token or a password; check finds one that was added by hand; version 0.3.1 is ready to tag. Requirements: docs/requirements.md R-CONF-7, decisions D40 and D45. Released alone as 0.3.1, before phase 4, because it protects dossiers that are already in use.
- summary: A password, key or access token can't be written into a dossier, where it would stay in the git history.
- by: Reynan Henry Tandoyo
- note (2026-11-22, planner): added
- note (2026-11-23, planner): Owner, 2026-11-23 (D45): released alone as 0.3.1; T-401 prepares the release.
- note (2026-11-23, planner): done — Version 0.3.1 refuses text that looks like a credential on every write and reports one added by hand in check.

### T-285 · Refuse text that looks like a credential on every write, and report it in check
- status: done
- owner: planner
- priority: P2 · type: milestone
- milestone: T-284
- goal: R-CONF-7 holds: no write stores a line that looks like a credential, a person at a terminal can allow a false alarm, and check finds one added by hand.
- why: Dossiers are committed to git, where a credential stays in the history; how-tos about licence servers and ssh are where one would be written down (D40).
- summary: Every command that writes text refuses a private key, an access token or a password, and names the line without printing the value. A person at a terminal can override a false alarm.
- accept: R-CONF-7 holds.
- reqs: R-CONF-7
- by: Reynan Henry Tandoyo
- note (2026-11-22, planner): added
- note (2026-11-22, planner): split into T-288, T-289, T-290, T-291, T-292
- note (2026-11-23, planner): done — Every write refuses text that looks like a credential, a person at a terminal can allow a false alarm, and check reports one added by hand.

### T-288 · secret.rs: find text that looks like a credential
- status: done
- owner: haiku-T-288
- priority: P0 · difficulty: 1 · type: feature
- milestone: T-285
- why: Dossiers are committed to git, where a credential stays in the history (D40).
- summary: The part that recognises a private key, an access token or a password in a text.
- files: src/secret.rs, src/lib.rs
- read: docs/design.md, docs/format.md, docs/requirements.md
- steps:
  1. Create src/secret.rs with exactly the items of docs/design.md section 13 (FILE, Hit, find, added, allowed, allow, listed) and add `pub mod secret;` to src/lib.rs in alphabetical order.
  2. find: go through the lines of the text (1-based) and apply the three rules of docs/format.md section 27 in the order private key, access token, password; a line gives at most one Hit, with the first kind that matches. Write the matching by hand with str methods; add no dependency.
  3. added: the hits of find(new) whose trimmed line text is not the text of a hit of find(old) and whose hash (crate::hash::sha256_hex of the trimmed line's bytes) is not in `allowed`.
  4. allowed and allow: the file is <state>/credential-ok.json, a JSON array of strings; a missing file is an empty list; a file that is not such an array is Code::Parse naming the file. allow adds the hashes of the hits, each once, sorted, and writes with crate::atomic::replace(path, bytes, None).
  5. listed: `<file>: line 12 (password); line 14 (private key)`; it never contains the line text.
  6. Unit tests at the end of the file: every row of the table in format.md section 27 with one line that matches and one that does not; the placeholder values (`password: <your password>`, `password = ****`, `token: $TOKEN`) are no hits; `password: ask the administrator` is no hit (several words) and `password: required` is one; `- status: approved` and `- tags: token-ring` are no hits; added ignores a hit line that old already had and one whose hash is allowed; allow then allowed gives the hashes back, in a temporary folder.
  7. Run cargo test --lib secret::
- accept: secret::find recognises the three kinds of format.md section 27, and added leaves out what was there before or was allowed.
- verify: cargo test --lib secret::
- verified: 2026-11-23
- commit: 18fdbe9
- reqs: R-CONF-7, D40
- by: Reynan Henry Tandoyo
- note (2026-11-22, planner): added
- note (2026-11-23, haiku-T-288): done

### T-289 · tx.rs: refuse a write that adds text looking like a credential
- status: done
- owner: haiku-T-289
- priority: P0 · difficulty: 1 · type: feature
- milestone: T-285
- why: Dossiers are committed to git, where a credential stays in the history (D40).
- summary: Every command that writes refuses a private key, an access token or a password, and names the line without printing the value.
- depends: T-288
- files: src/tx.rs, src/cli.rs, tests/secret.rs
- read: docs/design.md, docs/format.md, docs/requirements.md
- steps:
  1. src/cli.rs: add `#[arg(long, global = true)] pub allow_credential_like: bool,` to Global, after allow_network_drive.
  2. src/tx.rs, in `write`, directly after `let output = f(&mut tx)?;` and before the `if ctx.dry_run` branch: for every queued change whose action is Put, Rename or PutRaw, whose path ends in `.md` and whose new bytes are valid UTF-8, call secret::added(old text, new text, &secret::allowed(&ctx.state)?). The old text is the queued `old` bytes when they are UTF-8, else None. Collect the hits per file; the file is shown relative to ctx.root with `/`.
  3. No hits: go on as before. Hits, and the program was not started with --allow-credential-like (read it from std::env::args_os like allowed_on_network does for --allow-network-drive): return Code::Refused with the message of docs/design.md section 13, the files joined with `; `.
  4. Hits with the flag: when who.is_llm return Code::NeedsPerson (`only a person can allow text that looks like a credential; <name> is an LLM`); when !term::is_terminal() return Code::NeedsTerminal (`this command must be run by a person at a terminal`). On a dry run return the refusal of step 3 when the flag is missing, and allow nothing when it is given.
  5. A person at a terminal with the flag: after `apply` succeeded, call secret::allow(&ctx.state, &hits) and journal::append one Record per file: time tx.time, name and person of who, command ctx.command, id the change's id, event `allowed`, detail `credential-like text, line 12` (several: `line 12, 14`).
  6. tests/secret.rs (new, uses tests/common): `found` with `password: hunter2secret` in the observation fails with exit 1, the message names `password` and not `hunter2secret`, and no finding file exists; the same with --dry-run; as anna with DOSSIER_ANSWERS set and --allow-credential-like it is written, .state/credential-ok.json exists and the journal has an `allowed` record; as `anna - claude` with the flag it fails with NEEDS_PERSON in --json; after allowing, `set FND-001 --field tags=x` on that entry still works (the old line is no new hit); `value set` and `attach --note` with a token are refused too (block files).
  7. Run cargo test --test secret and cargo test --lib tx::
- accept: No command writes a new line that looks like a credential into a .md file of a dossier, unless a person at a terminal allows it; the journal records that.
- verify: cargo test --test secret
- verified: 2026-11-23
- commit: da803c4
- reqs: R-CONF-7, D40
- by: Reynan Henry Tandoyo
- note (2026-11-22, planner): added
- note (2026-11-23, haiku-T-289): done

### T-290 · forward.rs: --allow-credential-like asks ssh for a terminal
- status: done
- owner: haiku-T-290
- priority: P1 · difficulty: 1 · type: feature
- milestone: T-285
- why: Dossiers are committed to git, where a credential stays in the history (D40).
- summary: From a PC, allowing a false alarm works like the other commands that need a person at a terminal.
- depends: T-289
- files: src/remote/forward.rs
- read: docs/design.md, docs/format.md, docs/requirements.md
- steps:
  1. In `maybe`, the terminal flag becomes: (TERMINAL_COMMANDS.contains(&command) || the forwarded arguments contain `--allow-credential-like`) && term::is_terminal().
  2. Add a unit test next to the existing ones of this file that shows the argument is found in the forwarded arguments (test the small helper you write for it, not ssh).
  3. Run cargo test --lib remote::forward::
- accept: A command typed on the PC with --allow-credential-like gets a terminal on the server, so the person there is asked.
- verify: cargo test --lib remote::forward::
- verified: 2026-11-23
- commit: 78b77ab
- reqs: R-CONF-7, R-REMOTE-3
- by: Reynan Henry Tandoyo
- note (2026-11-22, planner): added
- note (2026-11-23, haiku-T-290): done

### T-291 · check: report text that looks like a credential (SECRET)
- status: done
- owner: haiku-T-291
- priority: P1 · difficulty: 1 · type: feature
- milestone: T-285
- why: Dossiers are committed to git, where a credential stays in the history (D40).
- summary: The check command also finds credentials that someone typed into a file with an editor.
- depends: T-289
- files: src/cmd/check.rs, tests/cmd_check.rs
- read: docs/design.md, docs/format.md, docs/requirements.md
- steps:
  1. Add a function that returns Vec<Found> of kind `SECRET`: for every entry file (the items of store::all) and every other `.md` file directly in the dossier folder or in its `files/` folder, run secret::find on the text and leave out the hits whose hash (hash::sha256_hex of the trimmed line) is in secret::allowed(&ctx.state)?. One Found per hit: the file relative to the project root, the hit's line, the entry ID (or `-` for a block file), `SECRET`, and the message of docs/design.md section 13.
  2. Extend `found` with it in `run`, next to the other checks.
  3. tests/cmd_check.rs: an entry file to which a line `password: hunter2secret` was added by hand (Project::write) gives one SECRET line with that line number and exit 1, and the output does not contain `hunter2secret`; the same line allowed first (write it with `found … --allow-credential-like` as anna with DOSSIER_ANSWERS) is not reported.
  4. Run cargo test --test cmd_check
- accept: check finds a credential that was added by hand and stays quiet about a line a person allowed.
- verify: cargo test --test cmd_check
- verified: 2026-11-23
- commit: 451c0fe
- reqs: R-CONF-7, R-CHECK-1
- by: Reynan Henry Tandoyo
- note (2026-11-22, planner): added
- note (2026-11-23, haiku-T-291): done

### T-292 · Agent rules and README: never write a credential into the dossier
- status: done
- owner: haiku-T-292
- priority: P1 · difficulty: 1 · type: docs
- milestone: T-285
- why: Dossiers are committed to git, where a credential stays in the history (D40).
- summary: LLMs and people are told not to write credentials, and what the tool does when they try.
- depends: T-289
- files: src/agents/rules.md, README.md, AGENTS.md
- read: docs/design.md, docs/format.md, docs/requirements.md
- steps:
  1. src/agents/rules.md: add one line after the line about confidential terms being written to the team dossier: `- Never write a password, key or token into an entry; write where it is kept. The dossier refuses text that looks like one.`
  2. README.md, section `## Confidentiality`: add a short paragraph: what is refused (a private key, an access token, a password after `password:` or `=`), that the message names the line and never the value, that a person at a terminal adds --allow-credential-like for a false alarm, and that `check` finds one added by hand.
  3. If AGENTS.md of this repository holds a dossier rules block (between `<!-- dossier:begin -->` and `<!-- dossier:end -->`), bring it up to date by running the built program: cargo run -- agents install --yes; otherwise leave AGENTS.md alone.
  4. Run cargo test --lib agents:: and cargo test --test cmd_agents --test cmd_skill; where a test compares the whole rules text, update its expected text.
- accept: The agent rules and the README say that credentials do not belong in a dossier and what happens when one is written.
- verify: cargo test --lib agents:: && cargo test --test cmd_agents --test cmd_skill
- verified: 2026-11-23
- commit: 90d070d
- reqs: R-CONF-7, R-AGENT-1
- by: Reynan Henry Tandoyo
- note (2026-11-22, planner): added
- note (2026-11-23, haiku-T-292): done

### T-286 · Trial and error is recorded: a problem, the attempts in order, what solved it
- status: done
- owner: planner
- priority: P2 · type: milestone
- goal: A person or an LLM records a problem, each attempt to solve it with its hypothesis and result, and what solved it, in ten seconds per attempt; the next person or LLM on a similar task sees what was tried and what worked. Attempts are lines in the dossier, not tasks; dossier never creates a task. Requirements: docs/requirements.md section 5.11h (R-INV-1 to 10), section 11 Phase 4 (v0.4), decisions D41, D42 and D45; version 0.4.0 is ready to tag.
- summary: Hypotheses, experiments and their results are tracked, so that nobody tests again what already failed and the way to the solution can be reused.
- by: Reynan Henry Tandoyo
- note (2026-11-22, planner): added
- note (2026-11-23, planner): Owner, 2026-11-23 (D45): this is now phase 4, release 0.4.0 (T-402). T-309 moved to T-283, because it needs the hook of T-294, which comes in 0.6.
- note (2026-11-23, planner): Owner, 2026-11-23: replace, do not refuse; only the four separator forms, so plain em dashes stay.
- note (2026-11-23, planner): The note above about replacing belongs to the task on attempt texts with a separator form, added below T-287; it was written here by a planner mistake.
- note (2026-11-23, planner): done — Version 0.4.0 records trial and error as investigations: a problem, the attempts in order, and what solved it.

### T-287 · Investigations: problem, tried, solved, gave-up, and their place in show, brief, status and search
- status: done
- owner: planner
- priority: P2 · type: milestone
- milestone: T-286
- goal: R-INV-1 to R-INV-10 hold: a problem, its attempts in order and its solution are recorded in ten seconds per attempt, shown at session start and before similar tasks, and promoted to the team as a how-to or lesson.
- why: Single dead ends lose the order of the attempts and the link to what finally worked (D41).
- summary: A new kind of entry holds a problem, each attempt with its hypothesis, result and verdict, and the solution. Open ones are shown at session start; solved ones are shown to whoever starts a similar task.
- accept: R-INV-1 to R-INV-10 hold.
- reqs: R-INV-1, R-INV-2, R-INV-3, R-INV-4, R-INV-5, R-INV-6, R-INV-7, R-INV-8, R-INV-9, R-INV-10
- by: Reynan Henry Tandoyo
- note (2026-11-22, planner): added
- note (2026-11-22, planner): Split when its turn comes: needs the form of an attempt in docs/format.md and a design in docs/design.md first. R-INV-9 (the hook part) needs T-283; the rest depends on nothing open.
- note (2026-11-23, planner): split into T-295, T-296, T-297, T-298, T-299, T-300, T-301, T-302, T-303, T-304, T-305, T-306, T-307, T-308, T-309, T-310
- note (2026-11-23, planner): done — Investigations are recorded with problem, tried, solved and gave-up, and appear in list, show, search, brief, status, history, review-task and promote.

### T-295 · Entry kinds: the investigation (INV)
- status: done
- owner: haiku-T-295
- priority: P0 · difficulty: 1 · type: feature
- milestone: T-287
- why: Single dead ends lose the order of the attempts and the link to what finally worked (D41).
- summary: A new kind of entry for trial and error: a problem, its attempts and its solution.
- files: src/entry/mod.rs, src/entry/rules.rs, src/entry/new.rs
- read: docs/design.md, docs/format.md, docs/requirements.md
- steps:
  1. src/entry/mod.rs: add `Investigation` as the last variant of Kind: all() returns [Kind; 8] with it last; name `investigation`, prefix `INV`, folder `investigations`. Update the tests of this file that list or count the kinds.
  2. src/entry/rules.rs: required_sections(Investigation) = ["Problem"]; section_order = ["Problem", "Attempts", "Solution"]; a constant INVESTIGATION_STATES = ["open", "solved", "given up"]; `problems` reports `state must be open, solved or given up` when an investigation's `state` is missing or another value. Update the test arrays of this file that have one element per kind.
  3. src/entry/new.rs: FIELD_ORDER gains `closed` and `given-up-reason`, in this order directly after `refuted` (raise the array length by two); the known sections of an investigation are those of section_order.
  4. Tests: Kind::parse("INV") and Kind::of_id("P-017:INV-003") give Investigation; a rendered investigation Draft with state open and a Problem section has no problems; one with state `closed` has one; the fields come out in the order `state`, `task`, …, `closed`.
  5. Run cargo test --lib entry:: and cargo build (other files must still compile; where the compiler asks for a match arm for Investigation, add the arm that treats it like a finding).
- accept: The program knows the investigation as a kind of entry with its folder, sections and states.
- verify: cargo test --lib entry::
- verified: 2026-11-23
- commit: 5b17b4b
- reqs: R-INV-1, D41
- by: Reynan Henry Tandoyo
- note (2026-11-23, planner): added
- note (2026-11-23, haiku-T-295): done

### T-296 · attempts.rs: the attempt lines of an investigation
- status: done
- owner: haiku-T-296
- priority: P0 · difficulty: 1 · type: feature
- milestone: T-287
- why: Single dead ends lose the order of the attempts and the link to what finally worked (D41).
- summary: The part that writes and reads the numbered attempt lines inside an investigation.
- depends: T-295
- files: src/attempts.rs, src/lib.rs
- read: docs/design.md, docs/format.md, docs/requirements.md
- steps:
  1. Create src/attempts.rs with exactly the items of docs/design.md section 15 (SECTION, VERDICTS, Attempt, parse, line, next_number, summary, of) and add `pub mod attempts;` to src/lib.rs in alphabetical order.
  2. The line format is fixed in docs/format.md section 28: `<n>. [<verdict>] <text>[ — did: …][ — result: …][ — run: …][ — task: …] (<time> <who>)`. parse: for each line, read the number before `. `, the verdict in `[ ]` (one of VERDICTS), split off the last ` (` … `)` as time (up to the first space) and who (the rest), then cut the middle at ` — did: `, ` — result: `, ` — run: `, ` — task: ` in this order, each only when present. A line that does not fit is skipped.
  3. line writes the parts in that order and leaves out a part that is None. next_number is 1 + the highest number parse finds, or 1. summary is `<n>. <verdict>: <text>` plus ` — <result>` when there is one. of(entry) is parse of the entry's Attempts section when entry.kind() is Kind::Investigation, else empty.
  4. Unit tests: parse(line(a)) == a for an attempt with every part and for one with only text and verdict; the three attempt lines of the example in docs/format.md section 28 parse to numbers 1, 2, 3 with verdicts failed, failed, worked; a line of free text between them is skipped and next_number is still 4; who `anna via anna - claude` survives the round trip; summary of attempt 1 of the example is `1. failed: The CFL number is too high — diverges at iteration 180`.
  5. Run cargo test --lib attempts::
- accept: Attempt lines are written and read back exactly, and lines that are no attempts are left alone.
- verify: cargo test --lib attempts::
- verified: 2026-11-23
- commit: 850b694
- reqs: R-INV-3
- by: Reynan Henry Tandoyo
- note (2026-11-23, planner): added
- note (2026-11-23, haiku-T-296): done

### T-297 · Investigations skeleton: command stubs for problem, tried, solved and gave-up
- status: done
- owner: planner
- priority: P0 · type: milestone
- milestone: T-287
- goal: The commands problem, tried, solved and gave-up exist as stubs, are listed in the overview and count as writing commands.
- why: Single dead ends lose the order of the attempts and the link to what finally worked (D41).
- summary: Empty places for the four new commands, so that each can be built on its own.
- depends: T-295
- accept: The four commands exist, are listed in the overview and answer that they are not built yet.
- reqs: R-INV-2, R-INV-3, R-INV-4
- by: Reynan Henry Tandoyo
- note (2026-11-23, planner): added
- note (2026-11-23, planner): split into T-357, T-358, T-359, T-360
- note (2026-11-23, planner): done — problem, tried, solved and gave-up exist as commands that answer 'not built yet'.

### T-357 · Stub files: problem and tried
- status: done
- owner: haiku-T-357
- priority: P0 · difficulty: 1 · type: chore
- milestone: T-297
- why: Single dead ends lose the order of the attempts and the link to what finally worked (D41).
- summary: Empty files for two of the new commands.
- files: src/cmd/problem.rs, src/cmd/tried.rs
- read: docs/design.md, docs/format.md, docs/requirements.md
- steps:
  1. Create both files, each with `#[derive(clap::Args, Debug)] pub struct Args {}` and `pub fn run(_args: &Args, _g: &Global) -> Result<Output>` returning Err(Error::new(Code::NotImplemented, "<command> is not built yet")), with the same `use` lines as src/cmd/guard.rs.
  2. The files are not part of the module tree yet (the next task registers them): check them with rustfmt --check --edition 2024 src/cmd/problem.rs src/cmd/tried.rs
  3. Run cargo build (it must still pass).
- accept: The two stub files exist and are formatted.
- verify: rustfmt --check --edition 2024 src/cmd/problem.rs src/cmd/tried.rs && cargo build
- verified: 2026-11-23
- commit: ae39ee2
- reqs: R-INV-2, R-INV-3
- by: Reynan Henry Tandoyo
- note (2026-11-23, planner): added
- note (2026-11-23, haiku-T-357): done

### T-358 · Stub files: solved and gave-up
- status: done
- owner: haiku-T-358
- priority: P0 · difficulty: 1 · type: chore
- milestone: T-297
- why: Single dead ends lose the order of the attempts and the link to what finally worked (D41).
- summary: Empty files for the other two new commands.
- files: src/cmd/solved.rs, src/cmd/gave_up.rs
- read: docs/design.md, docs/format.md, docs/requirements.md
- steps:
  1. Create both files, each with `#[derive(clap::Args, Debug)] pub struct Args {}` and `pub fn run(_args: &Args, _g: &Global) -> Result<Output>` returning Err(Error::new(Code::NotImplemented, "<command> is not built yet")), with the same `use` lines as src/cmd/guard.rs.
  2. Check them with rustfmt --check --edition 2024 src/cmd/solved.rs src/cmd/gave_up.rs
  3. Run cargo build (it must still pass).
- accept: The two stub files exist and are formatted.
- verify: rustfmt --check --edition 2024 src/cmd/solved.rs src/cmd/gave_up.rs && cargo build
- verified: 2026-11-23
- commit: bc95291
- reqs: R-INV-4
- by: Reynan Henry Tandoyo
- note (2026-11-23, planner): added
- note (2026-11-23, haiku-T-358): done

### T-359 · Register problem, tried, solved and gave-up
- status: done
- owner: haiku-T-359
- priority: P0 · difficulty: 1 · type: chore
- milestone: T-297
- why: Single dead ends lose the order of the attempts and the link to what finally worked (D41).
- summary: The four new commands become part of the program.
- depends: T-357, T-358
- files: src/cmd/mod.rs, src/cli.rs, src/cmd/help.rs, tests/cmd_agents.rs
- read: docs/design.md, docs/format.md, docs/requirements.md
- steps:
  1. src/cmd/mod.rs: `pub mod gave_up;`, `pub mod problem;`, `pub mod solved;`, `pub mod tried;`, each at its alphabetical place.
  2. src/cli.rs: variants `Problem(cmd::problem::Args)`, `Tried(cmd::tried::Args)`, `Solved(cmd::solved::Args)`, `GaveUp(cmd::gave_up::Args)` in the Command enum (clap names the last one `gave-up`), and four arms in the match of main_exit with the names "problem", "tried", "solved", "gave-up".
  3. src/cmd/help.rs: four more entries in COMMANDS and the array length raised by four: ("problem", "Opens an investigation"), ("tried", "Adds an attempt"), ("solved", "Closes an investigation as solved"), ("gave-up", "Closes an investigation unsolved").
  4. Run cargo build and cargo test --test smoke; `dossier problem` must answer `problem is not built yet` with exit 2.
- accept: The four commands are part of the program and the overview, and answer that they are not built yet.
- verify: cargo build && cargo test --test smoke
- verified: 2026-11-23
- commit: 7d89571
- reqs: R-INV-2, R-INV-3, R-INV-4
- by: Reynan Henry Tandoyo
- note (2026-11-23, planner): added
- note (2026-11-23, planner): The help overview test in tests/cmd_agents.rs lists every command, so registering new commands changes its expected list; the file was added to this task.
- note (2026-11-23, planner): forced set (owner was haiku-T-359)
- note (2026-11-23, haiku-T-359): done

### T-360 · ctx.rs: problem, tried, solved and gave-up are writing commands
- status: done
- owner: haiku-T-360
- priority: P0 · difficulty: 1 · type: chore
- milestone: T-297
- why: Single dead ends lose the order of the attempts and the link to what finally worked (D41).
- summary: The new commands follow the same name rule as every command that writes.
- files: src/ctx.rs
- read: docs/design.md, docs/format.md, docs/requirements.md
- steps:
  1. Add "problem", "tried", "solved" and "gave-up" to the WRITING list and raise its length by four.
  2. Run cargo test --lib ctx::
- accept: The four commands refuse a name that is not in the settings, like the other writing commands.
- verify: cargo test --lib ctx::
- verified: 2026-11-23
- commit: 4672a0f
- reqs: R-CFG-4
- by: Reynan Henry Tandoyo
- note (2026-11-23, planner): added
- note (2026-11-23, haiku-T-360): done

### T-298 · problem: open an investigation
- status: done
- owner: haiku-T-298
- priority: P1 · difficulty: 1 · type: feature
- milestone: T-287
- why: Single dead ends lose the order of the attempts and the link to what finally worked (D41).
- summary: The command that records a problem and starts the list of attempts.
- depends: T-297
- files: src/cmd/problem.rs, tests/cmd_problem.rs
- read: docs/design.md, docs/format.md, docs/requirements.md
- steps:
  1. Args as in the table of docs/design.md section 15: the positional text, --observed, --title, --task and the tag options (--machine, --analysis, --software, --tags), as src/cmd/found.rs has them. Use src/cmd/found.rs as the model for the whole file.
  2. run: Ctx::load(g, "problem"); in a team dossier (ctx.cfg.kind == "team") return Code::Refused `investigations belong to a project, not to the team dossier; promote a solved one with dossier promote <ID> --as howto`. The text and --observed must be one line and must not start with `## ` (Code::BadValue).
  3. Before writing, collect the similar investigations: store::all, keep kind Investigation and status approved, select::best with Query { tags: the given tag values, words: select::words(the title) }, limit 5.
  4. tx::write with create::create: New { kind: Kind::Investigation, title: --title or the first 80 characters of the text, fields: state=open, task (trimmed, upper case) and the tags, sections: Problem = the text, followed by an empty line and `Observed: <text>` when --observed is given, said_by: None, draft: false }.
  5. Output: first line `<ID>: added (open)`, then one line per similar investigation `similar: <ID> [<state>] <title>`. JSON data: {id, state, similar: [ids]}.
  6. tests/cmd_problem.rs (new): a person and an LLM (`anna - claude`) each open one and both are approved at once with state open; the file is in DOSSIER/investigations/ and has the Problem section; --task t-12 is stored as T-12; a second problem with the same words lists the first as `similar:`; a text with a line break is refused with exit 1 and nothing is written; a dossier whose investigations folder does not exist yet (remove it in the test) still accepts the command.
  7. Run cargo test --test cmd_problem
- accept: problem writes an approved investigation with state open and shows similar ones.
- verify: cargo test --test cmd_problem
- verified: 2026-11-23
- commit: d80a13e
- reqs: R-INV-1, R-INV-2
- by: Reynan Henry Tandoyo
- note (2026-11-23, planner): added
- note (2026-11-23, haiku-T-298): done

### T-299 · tried: add one attempt to an investigation
- status: done
- owner: haiku-T-299
- priority: P1 · difficulty: 1 · type: feature
- milestone: T-287
- why: Single dead ends lose the order of the attempts and the link to what finally worked (D41).
- summary: The command that records one hypothesis and what came of it.
- depends: T-298, T-296
- files: src/cmd/tried.rs, tests/cmd_tried.rs
- read: docs/design.md, docs/format.md, docs/requirements.md
- steps:
  1. Args as in the table of docs/design.md section 15: id, text, --failed, --worked, --unclear (bool each), --did, --result, --run, --task. Exactly one of the three verdicts must be given (Code::Usage otherwise). Every text must be one line and not start with `## ` (Code::BadValue). Use src/cmd/confirm.rs as the model for changing an entry inside tx::write.
  2. Inside tx::write: store::get the entry; it must be kind Investigation with status approved (Code::Refused naming what it is). --run must be an entry of kind Usage (store::get, Code::NotFound when missing; Code::Refused when another kind); store its ID in upper case.
  3. Build attempts::Attempt { n: attempts::next_number(the Attempts body or ""), verdict, text, did, result, run, task (upper case), time: tx.time(), who: names::label style as history lines write it (`anna` or `anna via anna - claude`; use the same helper add_history uses or build `<person> via <name>` for an LLM) } and append attempts::line(&a) to the Attempts section the way src/cmd/used.rs write_remark appends to Open remarks (section missing: the line alone).
  4. When the state is not `open`: entry.set_field("state", "open"), remove the fields `closed` and `given-up-reason`, and add a history line `reopened`. Always add the history line `tried` with detail `<n> <verdict>`. tx.put with Change { id, event: "tried", detail: "<n> <verdict>" }.
  5. Output `<ID>: attempt <n> recorded (<verdict>)`, with `, opened again` appended when the state was changed. JSON data {id, attempt: n, verdict, state: "open"}.
  6. tests/cmd_tried.rs (new): three attempts get the numbers 1, 2, 3 and stand in the file in that order with every other byte unchanged (compare the text before the Attempts section); an attempt with --did, --result and --task has all parts in the line of format.md section 28; no verdict and two verdicts are exit 2; --run naming a finding is refused; an LLM may record an attempt and the line ends with `via`; tried on FND-001 is refused; `search <a word of the attempt>` finds the investigation and its result line starts with the attempt number.
  7. Run cargo test --test cmd_tried
- accept: Each attempt is one numbered line with its verdict, who and when, added in ten seconds, and a new attempt opens a closed investigation again.
- verify: cargo test --test cmd_tried
- verified: 2026-11-23
- commit: c785499
- reqs: R-INV-3, R-INV-4, R-INV-5
- by: Reynan Henry Tandoyo
- note (2026-11-23, planner): added
- note (2026-11-23, haiku-T-299): done

### T-300 · solved and gave-up: close an investigation
- status: done
- owner: haiku-T-300
- priority: P1 · difficulty: 1 · type: feature
- milestone: T-287
- why: Single dead ends lose the order of the attempts and the link to what finally worked (D41).
- summary: The two commands that end an investigation: solved, or given up.
- depends: T-299
- files: src/cmd/solved.rs, src/cmd/gave_up.rs, tests/cmd_solved.rs
- read: docs/design.md, docs/format.md, docs/requirements.md
- steps:
  1. solved: Args id, text, `--attempt <n>` (repeatable). gave-up: Args id, `--reason <text>` (required). Texts are one line and do not start with `## ` (Code::BadValue). Both follow src/cmd/confirm.rs and the table of docs/design.md section 15.
  2. Both, inside tx::write: the entry must be an approved investigation with state `open` (Code::Refused saying which state it has).
  3. solved: every --attempt number must be one of attempts::of(&entry) (Code::NotFound `no attempt <n> in <ID>`). Set state=solved, closed=`<time> by <person>`, section Solution = the text, followed by an empty line and `Attempts: 2, 3` when numbers were given; history `solved`; Change event `solved`. Output `<ID>: solved`.
  4. gave-up: set state=`given up`, closed, given-up-reason=<reason>; history `given up` with the reason as detail; Change event `given up`. Output `<ID>: given up`.
  5. tests/cmd_solved.rs (new): solved after two attempts sets the fields and the Solution section and keeps the attempt lines byte for byte; --attempt 9 is refused and nothing changes; solved on a solved one is refused; gave-up sets its fields; `tried` after solved gives `opened again`, state open and no `closed` field; an LLM may run both; solved on a decision is refused.
  6. Run cargo test --test cmd_solved
- accept: An investigation is closed as solved with what solved it, or as given up with the reason.
- verify: cargo test --test cmd_solved
- verified: 2026-11-23
- commit: 2b4c9e0
- reqs: R-INV-4
- by: Reynan Henry Tandoyo
- note (2026-11-23, planner): added
- note (2026-11-23, haiku-T-300): done

### T-301 · filter.rs: --state
- status: done
- owner: haiku-T-301
- priority: P1 · difficulty: 1 · type: feature
- milestone: T-287
- why: Single dead ends lose the order of the attempts and the link to what finally worked (D41).
- summary: A way to list only the open, the solved or the given-up investigations.
- depends: T-295
- files: src/filter.rs
- read: docs/design.md, docs/format.md, docs/requirements.md
- steps:
  1. Add `#[arg(long)] state: Option<String>` to Filter, after `task`. `matches`: when given, the entry's `state` field must equal it, trimmed, case ignored.
  2. Unit tests next to the existing ones: --state open matches an investigation with state open and not one with `given up`; `--state "given up"` matches that one; without --state nothing changes.
  3. Run cargo test --lib filter::
- accept: list and search can be limited to entries in one state, such as open investigations.
- verify: cargo test --lib filter::
- verified: 2026-11-23
- commit: ed6a701
- reqs: R-INV-5
- by: Reynan Henry Tandoyo
- note (2026-11-23, planner): added
- note (2026-11-23, haiku-T-301): done

### T-302 · list, show and search: mark the state of an investigation
- status: done
- owner: planner
- priority: P1 · type: milestone
- milestone: T-287
- goal: list, show and search mark every investigation as open, solved or given up.
- why: Single dead ends lose the order of the attempts and the link to what finally worked (D41).
- summary: Lists and search results show the state of each investigation.
- depends: T-300, T-301
- accept: Wherever an investigation is listed, the reader sees whether it is open, solved or given up.
- reqs: R-INV-5
- by: Reynan Henry Tandoyo
- note (2026-11-23, planner): added
- note (2026-11-23, planner): split into T-361, T-362, T-363
- note (2026-11-23, planner): done — list, show and search mark an investigation as open, solved or given up.

### T-361 · list: mark the state of an investigation
- status: done
- owner: haiku-T-361
- priority: P1 · difficulty: 1 · type: feature
- milestone: T-302
- why: Single dead ends lose the order of the attempts and the link to what finally worked (D41).
- summary: In list, the reader sees the state of each investigation.
- depends: T-300, T-301
- files: src/cmd/list.rs, tests/cmd_list.rs
- read: docs/design.md, docs/format.md, docs/requirements.md
- steps:
  1. src/cmd/list.rs, in `mark`: for an entry of kind Investigation add the mark `[open]`, `[solved]` or `[given up]` from its `state` field, after the status mark.
  2. tests/cmd_list.rs: an open investigation is shown with `[open]`, and after `dossier solved` with `[solved]`. `list --kind investigation --state open` lists only the open one.
  3. Run cargo test --test cmd_list
- accept: list shows whether an investigation is open, solved or given up.
- verify: cargo test --test cmd_list
- verified: 2026-11-23
- commit: 66f5b83
- reqs: R-INV-5
- by: Reynan Henry Tandoyo
- note (2026-11-23, planner): added
- note (2026-11-23, haiku-T-361): done

### T-362 · show: mark the state of an investigation
- status: done
- owner: haiku-T-362
- priority: P1 · difficulty: 1 · type: feature
- milestone: T-302
- why: Single dead ends lose the order of the attempts and the link to what finally worked (D41).
- summary: In show, the reader sees the state of each investigation.
- depends: T-300, T-301
- files: src/cmd/show.rs, tests/cmd_show.rs
- read: docs/design.md, docs/format.md, docs/requirements.md
- steps:
  1. src/cmd/show.rs, in `marks`: for an entry of kind Investigation add the mark `[open]`, `[solved]` or `[given up]` from its `state` field, after the status mark.
  2. tests/cmd_show.rs: an open investigation is shown with `[open]`, and after `dossier solved` with `[solved]`.
  3. Run cargo test --test cmd_show
- accept: show shows whether an investigation is open, solved or given up.
- verify: cargo test --test cmd_show
- verified: 2026-11-23
- commit: e1e4751
- reqs: R-INV-5
- by: Reynan Henry Tandoyo
- note (2026-11-23, planner): added
- note (2026-11-23, haiku-T-362): done

### T-363 · search: mark the state of an investigation
- status: done
- owner: haiku-T-363
- priority: P1 · difficulty: 1 · type: feature
- milestone: T-302
- why: Single dead ends lose the order of the attempts and the link to what finally worked (D41).
- summary: In search, the reader sees the state of each investigation.
- depends: T-300, T-301
- files: src/cmd/search.rs, tests/cmd_search.rs
- read: docs/design.md, docs/format.md, docs/requirements.md
- steps:
  1. src/cmd/search.rs, in `mark`: for an entry of kind Investigation add the mark `[open]`, `[solved]` or `[given up]` from its `state` field, after the status mark.
  2. tests/cmd_search.rs: an open investigation is shown with `[open]`, and after `dossier solved` with `[solved]`.
  3. Run cargo test --test cmd_search
- accept: search shows whether an investigation is open, solved or given up.
- verify: cargo test --test cmd_search
- verified: 2026-11-23
- commit: 568a43b
- reqs: R-INV-5
- by: Reynan Henry Tandoyo
- note (2026-11-23, planner): added
- note (2026-11-23, haiku-T-363): done

### T-303 · brief: the open investigations
- status: done
- owner: haiku-T-303
- priority: P1 · difficulty: 1 · type: feature
- milestone: T-287
- why: Single dead ends lose the order of the attempts and the link to what finally worked (D41).
- summary: At session start the LLM sees which problems are still being worked on and what was tried last.
- depends: T-300
- files: src/cmd/brief.rs, tests/cmd_brief.rs
- read: docs/design.md, docs/format.md, docs/requirements.md
- steps:
  1. Brief gains a list of lines for the open investigations: in Brief::collect take shown(items, Kind::Investigation, day), keep those whose `state` is open, newest first, and build one line each: `<ID> · <title> — <n> attempts, last: <attempts::summary of the last attempt>`, or `— no attempt yet` when there is none.
  2. Brief::lines prints them under `## Open investigations`, directly after `## Open assumptions`. Brief::cut_one cuts them one by one after the team lines and before the proposals. Brief::data gains `investigations` (their IDs).
  3. tests/cmd_brief.rs: an open investigation with two attempts appears under the heading with `2 attempts, last: 2. failed:`; a solved one does not appear; with --json its ID is in data.investigations; a brief without investigations has no such heading.
  4. Run cargo test --test cmd_brief
- accept: A session starts with the open investigations and their last attempt in view.
- verify: cargo test --test cmd_brief
- verified: 2026-11-23
- commit: 58e4632
- reqs: R-INV-5
- by: Reynan Henry Tandoyo
- note (2026-11-23, planner): added
- note (2026-11-23, haiku-T-303): done

### T-304 · status: open investigations under Waiting
- status: done
- owner: haiku-T-304
- priority: P1 · difficulty: 1 · type: feature
- milestone: T-287
- why: Single dead ends lose the order of the attempts and the link to what finally worked (D41).
- summary: Whoever asks where the project stands also sees the problems that are not solved yet.
- depends: T-300
- files: src/cmd/status.rs, tests/cmd_status.rs
- read: docs/design.md, docs/format.md, docs/requirements.md
- steps:
  1. In project_screen, where the lines of `## Waiting` are built from the blocked tasks and the questions: add, after them, one line per open investigation in scope (kind Investigation, approved, state open; the same `scope` the decisions use, so --milestone limits them): `<ID> · <title> (investigation, <n> attempts)`. The section is printed when any of the three kinds of line exists, also when todo is not available.
  2. The JSON of status gains `investigations`: their IDs.
  3. tests/cmd_status.rs: an open investigation is listed under `## Waiting` with its number of attempts; a solved one is not; without a todo program the line is still there.
  4. Run cargo test --test cmd_status
- accept: status shows the open investigations among what is waiting.
- verify: cargo test --test cmd_status
- verified: 2026-11-23
- commit: a9ae835
- reqs: R-INV-5
- by: Reynan Henry Tandoyo
- note (2026-11-23, planner): added
- note (2026-11-23, haiku-T-304): done

### T-305 · brief --task: the investigations of the task and similar ones
- status: done
- owner: haiku-T-305
- priority: P1 · difficulty: 1 · type: feature
- milestone: T-287
- why: Single dead ends lose the order of the attempts and the link to what finally worked (D41).
- summary: Before starting a task, the LLM is shown the attempts already made and the solutions of similar problems, so it does not repeat a failed one.
- depends: T-300
- files: src/brief_task.rs
- read: docs/design.md, docs/format.md, docs/requirements.md
- steps:
  1. In `lines`, after the dead ends: add the part `## Investigations` as docs/design.md section 15 describes it, at most 15 lines including the heading. First the investigations whose `task` is the task (task_is): an open one as `<ID> · <title> [open]` followed by one line per attempt, indented by two spaces, with attempts::summary; a closed one as one line (step 2). Then up to five other approved investigations, any state, chosen by select::best with the same Query the dead ends use, one line each.
  2. The one-line form: `<ID> · <title> — solved: <first line of the Solution section> (<n> failed)`, or `— given up: <given-up-reason> (<n> failed)`, or for an open one `— open, <n> attempts (<n> failed)`; `<n> failed` counts the attempts with verdict failed.
  3. Investigations printed here are added to the IDs already listed, so that the related entries below do not repeat them.
  4. Unit tests at the end of the file, in the style of the existing ones: the open investigation of the task is printed with its attempts; a solved one of another task whose title shares a word with the task's title is printed as one line with `solved:` and the failed count; an investigation that shares nothing is not printed; the part never exceeds 15 lines with 30 attempts.
  5. Run cargo test --lib brief_task::
- accept: Before a task the reader sees what was tried for it and for similar problems, and what solved them.
- verify: cargo test --lib brief_task::
- verified: 2026-11-23
- commit: 25ba58f
- reqs: R-INV-5, R-INV-7
- by: Reynan Henry Tandoyo
- note (2026-11-23, planner): added
- note (2026-11-23, haiku-T-305): done

### T-306 · history <task>: the attempts carried out as that task
- status: done
- owner: haiku-T-306
- priority: P1 · difficulty: 1 · type: feature
- milestone: T-287
- why: Single dead ends lose the order of the attempts and the link to what finally worked (D41).
- summary: The history of a task also shows the experiments that were run as that task.
- depends: T-299
- files: src/cmd/history.rs, tests/cmd_history.rs
- read: docs/design.md, docs/format.md, docs/requirements.md
- steps:
  1. In task_history: besides the entries whose `task` names the task, collect from every investigation the attempts (attempts::of) whose `task` equals the task, case ignored. Each gives a line `<time>  <who>  <ID> attempt <n>  <text>`. Sort all lines together by time, oldest first.
  2. JSON: each row gains `attempt` (the number, or null for an entry row).
  3. tests/cmd_history.rs: an investigation opened with --task T-012 is listed for T-012; an attempt recorded with --task T-045 is listed for T-045 as `INV-001 attempt 1`, and the investigation itself is not listed for T-045.
  4. Run cargo test --test cmd_history
- accept: Everything recorded about a task includes the investigations opened in it and the attempts carried out as it.
- verify: cargo test --test cmd_history
- verified: 2026-11-23
- commit: 635d580
- reqs: R-INV-6, D42
- by: Reynan Henry Tandoyo
- note (2026-11-23, planner): added
- note (2026-11-23, haiku-T-306): done

### T-307 · review-task: investigations as material, and one more drafting rule
- status: done
- owner: haiku-T-307
- priority: P1 · difficulty: 1 · type: feature
- milestone: T-287
- why: Single dead ends lose the order of the attempts and the link to what finally worked (D41).
- summary: After a task, the trial and error recorded in it is offered as material for a lesson or a how-to.
- depends: T-300
- files: src/cmd/review_task.rs, tests/cmd_review_task.rs
- read: docs/design.md, docs/format.md, docs/requirements.md
- steps:
  1. RULES gains an eleventh text (raise the array length): `A solved investigation with failed attempts is a candidate for a how-to or a lesson.`
  2. Under `## Entries recorded for this task`, an investigation is followed by its attempts, one per line indented by two spaces (attempts::summary), and, when it has a Solution section, a line `  solution: <its first line>`.
  3. tests/cmd_review_task.rs: a solved investigation of the task is printed with its attempts and its solution; the rules count eleven.
  4. Run cargo test --test cmd_review_task
- accept: When a task is reviewed, its investigations are part of the material and the rules point to them.
- verify: cargo test --test cmd_review_task
- verified: 2026-11-23
- commit: caabbff
- reqs: R-INV-8, R-DRAFT-2
- by: Reynan Henry Tandoyo
- note (2026-11-23, planner): added
- note (2026-11-23, haiku-T-307): done

### T-308 · promote: a solved investigation becomes a how-to or lesson draft for the team
- status: done
- owner: planner
- priority: P1 · type: milestone
- milestone: T-287
- goal: A solved investigation is promoted to the team dossier as a how-to or lesson draft, after the confidentiality check.
- why: Single dead ends lose the order of the attempts and the link to what finally worked (D41).
- summary: The way a solved problem and its failed attempts become reusable knowledge for the next project.
- depends: T-300
- accept: A solved investigation reaches the team dossier as a how-to or lesson draft, after the confidentiality check.
- reqs: R-INV-8, R-TEAM-2, R-CONF-2
- by: Reynan Henry Tandoyo
- note (2026-11-23, planner): added
- note (2026-11-23, planner): split into T-364, T-365
- note (2026-11-23, planner): done — A solved investigation is promoted as a how-to or lesson draft whose sections come from its problem, attempts and solution.

### T-364 · promote: an investigation needs --as, and must be solved
- status: done
- owner: haiku-T-364
- priority: P1 · difficulty: 1 · type: feature
- milestone: T-308
- why: Single dead ends lose the order of the attempts and the link to what finally worked (D41).
- summary: The rule for which investigations can go to the team, and as what.
- depends: T-300
- files: src/cmd/promote.rs, tests/cmd_promote.rs
- read: docs/design.md, docs/format.md, docs/requirements.md
- steps:
  1. In target_kind: for an entry of kind Investigation, no --as is Code::Usage `an investigation is promoted as a how-to or a lesson: add --as howto or --as lesson`; `--as howto` gives Kind::Howto, `--as lesson` gives Kind::Lesson.
  2. In `promote`, before anything is read from the team: an investigation whose `state` is not `solved` is Code::Refused `<ID> is not solved yet; close it with dossier solved first`.
  3. For now copied_sections treats an investigation like a finding (the next task fills the sections).
  4. tests/cmd_promote.rs: promote INV-001 without --as is exit 2; an open one with --as howto is exit 1 and writes nothing in the team dossier; a solved one with --as lesson creates a team draft with `source` set.
  5. Run cargo test --test cmd_promote
- accept: promote accepts a solved investigation only with --as howto or --as lesson.
- verify: cargo test --test cmd_promote
- verified: 2026-11-23
- commit: 1405ea3
- reqs: R-INV-8, R-TEAM-2
- by: Reynan Henry Tandoyo
- note (2026-11-23, planner): added
- note (2026-11-23, haiku-T-364): done

### T-365 · promote: the sections of the how-to or lesson made from an investigation
- status: done
- owner: haiku-T-365
- priority: P1 · difficulty: 1 · type: feature
- milestone: T-308
- why: Single dead ends lose the order of the attempts and the link to what finally worked (D41).
- summary: A solved problem and its failed attempts become a reusable how-to or lesson.
- depends: T-364
- files: src/cmd/promote.rs, tests/cmd_promote.rs
- read: docs/design.md, docs/format.md, docs/requirements.md
- steps:
  1. In copied_sections, for a source of kind Investigation, with attempts = attempts::of(&entry) and solution = the Solution section without a last line that starts with `Attempts:`.
  2. Target Lesson: Statement = solution; Evidence = the Problem section, an empty line, then attempts::summary of every attempt, one per line.
  3. Target Howto: Purpose = the Problem section; Steps = solution, then one line `- <text>` per attempt with verdict worked, with `: <did>` appended when it has one; Traps = one line `- <text>` per attempt with verdict failed, with `: <result>` appended when it has one. A section that would be empty is left out.
  4. tests/cmd_promote.rs: a solved investigation with one failed and one worked attempt, promoted --as howto, gives a team draft whose Purpose, Steps and Traps are exactly as in step 3; --as lesson gives Statement and Evidence as in step 2; a confidential term in an attempt line refuses the promotion and names the term.
  5. Run cargo test --test cmd_promote
- accept: The team draft made from an investigation holds the problem, what solved it, what worked and what failed, in the right sections.
- verify: cargo test --test cmd_promote
- verified: 2026-11-23
- commit: 6fc36b1
- reqs: R-INV-8, R-CONF-2
- by: Reynan Henry Tandoyo
- note (2026-11-23, planner): added
- note (2026-11-23, haiku-T-365): done

### T-309 · after: point to investigations after stuck and done
- status: open
- owner: —
- priority: P1 · difficulty: 1 · type: feature
- milestone: T-283
- why: Single dead ends lose the order of the attempts and the link to what finally worked (D41).
- summary: Giving a task back, or finishing one, reminds the LLM to record the trial and error that belongs to it.
- depends: T-306, T-302, T-294
- files: src/cmd/after.rs, tests/cmd_after.rs
- read: docs/design.md, docs/format.md, docs/requirements.md
- steps:
  1. Stuck(id): append to the fixed text: `, or open an investigation: dossier problem "<what goes wrong>" --task <id>`.
  2. Done(id): after the usage part, run own(["list", "--kind", "investigation", "--task", id, "--state", "open", "--json"]) and own(["history", id, "--json"]); collect the IDs of the open investigations of the first and the IDs of the rows of the second that have an `attempt` number, each once; for each add the line of docs/design.md section 15: `<INV-ID> is still open: record the result with dossier tried <INV-ID> "<what was tried>" --worked, --failed or --unclear, or close it with dossier solved <INV-ID> "<what solved it>"`. For an ID from the history rows, add the line only when `list --kind investigation --state open --json` names it.
  3. tests/cmd_after.rs: after `todo stuck T-001` the context names `dossier problem`; with an open investigation of T-001, `todo done T-001` names it as still open; after `solved` it is no longer named.
  4. Run cargo test --test cmd_after
- accept: The hook reminds the agent to open an investigation for a stuck task and to record the result of an open one when its task is done.
- verify: cargo test --test cmd_after
- reqs: R-INV-9, R-AGENT-5
- note (2026-11-23, planner): added

### T-310 · Agent rules and README: investigations
- status: done
- owner: haiku-T-310
- priority: P1 · difficulty: 1 · type: docs
- milestone: T-287
- why: Single dead ends lose the order of the attempts and the link to what finally worked (D41).
- summary: LLMs and people learn how to record trial and error with the new commands.
- depends: T-300
- files: src/agents/rules.md, README.md, AGENTS.md
- read: docs/design.md, docs/format.md, docs/requirements.md
- steps:
  1. src/agents/rules.md: add after the line about `dossier found … --dead-end` two lines: `- When something fails and you start looking for the cause, open an investigation: `dossier problem "<what goes wrong>" --task <id>`, or continue the open one; read its attempts first (`dossier show <ID>`).` and `- Record one line for each hypothesis, also the failed ones, not one for each compile or run: `dossier tried <ID> "<hypothesis>" --failed|--worked|--unclear --result "<what happened>"`; close with `dossier solved <ID> "<what solved it>"`.` The file must stay at 40 lines or fewer.
  2. README.md: under the heading for the newest version's additions, add a short section `Investigations: trial and error` with the four commands, one example each taken from docs/format.md section 28, and two sentences: an attempt is a line, not a task; a solved one is promoted with `promote <ID> --as howto`.
  3. If AGENTS.md of this repository holds a dossier rules block, bring it up to date with cargo run -- agents install --yes; otherwise leave it.
  4. Run cargo test --lib agents:: and cargo test --test cmd_agents --test cmd_skill; update an expected text where a test compares the whole rules text.
- accept: The agent rules tell an LLM when to open an investigation and how to record attempts, and the README explains the commands.
- verify: cargo test --lib agents:: && cargo test --test cmd_agents --test cmd_skill
- verified: 2026-11-23
- commit: 44a7c43
- reqs: R-INV-10, R-AGENT-1
- by: Reynan Henry Tandoyo
- note (2026-11-23, planner): added
- note (2026-11-23, haiku-T-310): done

### T-401 · Release 0.3.1
- status: done
- owner: haiku-T-401
- priority: P1 · difficulty: 1 · type: chore
- milestone: T-284
- why: Dossiers are committed to git (D36), where a credential stays in the history (D40); the check should reach the dossiers in use without waiting for a larger release (D45).
- summary: The version number and the README are ready for the owner to tag release 0.3.1.
- depends: T-290, T-291, T-292
- files: Cargo.toml, Cargo.lock, README.md
- read: docs/design.md, docs/format.md, docs/requirements.md
- steps:
  1. Cargo.toml: version = "0.3.1"; run cargo build so that Cargo.lock follows.
  2. README.md: set the status line at the top to `**Status: v0.3.1**`; add `### What version 0.3.1 adds` after the 0.3 section (text that looks like a credential is refused on every write, --allow-credential-like, check reports it as SECRET); rename `### Not in version 0.3` to `### Not in version 0.3.1` and remove its line about version 0.3.1.
  3. Run cargo fmt --all -- --check, cargo clippy --all-targets --all-features -- -D warnings and cargo test --all-features; all three must pass. Do not tag and do not push: the owner does that.
- accept: Version 0.3.1 builds, passes every check, and the README says what it adds.
- verify: cargo fmt --all -- --check && cargo clippy --all-targets --all-features -- -D warnings && cargo test --all-features
- verified: 2026-11-23
- commit: b725916
- reqs: REL-1, REL-2
- by: Reynan Henry Tandoyo
- note (2026-11-23, planner): added
- note (2026-11-23, haiku-T-401): done

### T-402 · Release 0.4.0
- status: done
- owner: haiku-T-402
- priority: P1 · difficulty: 1 · type: chore
- milestone: T-286
- why: Single dead ends lose the order of the attempts and the link to what finally worked (D41).
- summary: The version number and the README are ready for the owner to tag release 0.4.0.
- depends: T-401, T-303, T-304, T-305, T-306, T-307, T-310, T-361, T-362, T-363, T-365, T-403
- files: Cargo.toml, Cargo.lock, README.md
- read: docs/design.md, docs/format.md, docs/requirements.md
- steps:
  1. Cargo.toml: version = "0.4.0"; run cargo build so that Cargo.lock follows (only the version line of the dossier package may change).
  2. README.md: set the status line at the top to `**Status: v0.4 (phase 4)**`. A `### What version 0.4 adds` section already exists (written by T-310) after `### Not in version 0.3.1`: move it, unchanged, directly after the `### What version 0.3.1 adds` section, and complete it where it is missing something from this list: problem, tried, solved and gave-up; their state in list, show and search; the open investigations in brief and status; brief --task; history <task>; review-task; promote of a solved investigation with --as. Keep it short. Then rename `### Not in version 0.3.1` to `### Not in version 0.4` and remove its phase 4 line (what is left: phases 5 and 6). There must be exactly one `### What version 0.4 adds` heading.
  3. Run cargo fmt --all -- --check, cargo clippy --all-targets --all-features -- -D warnings and cargo test --all-features; all three must pass. Do not tag and do not push: the owner does that.
- accept: Version 0.4.0 builds, passes every check, and the README says what it adds.
- verify: cargo fmt --all -- --check && cargo clippy --all-targets --all-features -- -D warnings && cargo test --all-features
- verified: 2026-11-23
- commit: 527d147
- reqs: REL-1, REL-2
- by: Reynan Henry Tandoyo
- note (2026-11-23, planner): added
- note (2026-11-23, planner): T-310 already wrote the 0.4 README section; the steps now move and complete it.
- note (2026-11-23, haiku-T-402): done

### T-403 · tried: a text that holds a part separator keeps reading back the same
- status: done
- owner: haiku-T-403
- priority: P1 · difficulty: 1 · type: feature
- milestone: T-287
- why: An attempt text with ' — result: ' in it would read back split into the wrong parts (format.md section 28).
- summary: When an attempt's text contains one of the separator forms, it is written with an en dash so the line reads back exactly as recorded.
- depends: T-299
- files: src/cmd/tried.rs, tests/cmd_tried.rs
- read: docs/format.md, src/attempts.rs
- steps:
  1. src/cmd/tried.rs: add a private fn `unseparated(text: &str) -> String` that replaces each of the four separator forms of src/attempts.rs (`" — did: "`, `" — result: "`, `" — run: "`, `" — task: "`, with the em dash U+2014) by the same text with an en dash U+2013 (`" – did: "` and so on). Every other character, including a plain `" — "`, stays as it is.
  2. src/cmd/tried.rs, in `run`: apply `unseparated` to the attempt text (after check_line, before it goes into Attempt) and to `did` and `result` (after optional_line). Do not apply it to `run` and `task`.
  3. Unit tests at the end of src/cmd/tried.rs for `unseparated`: `"CFL 5 — result: diverges"` becomes `"CFL 5 – result: diverges"`; `"CFL 5 — still diverges"` is unchanged; a text with two separator forms has both replaced.
  4. tests/cmd_tried.rs: `tried INV-001 "CFL 5 — result: diverges" --failed` writes an attempt line that contains `CFL 5 – result: diverges` and no ` — result: `; `tried INV-001 "CFL 5 — still diverges" --failed --result "same — did: nothing"` keeps `CFL 5 — still diverges` as written and writes `same – did: nothing` as the result.
  5. Run cargo test --test cmd_tried and cargo test --lib cmd::tried::
- accept: A text, --did or --result that holds a separator form is written with an en dash, every other em dash is kept, and the attempt reads back with the same parts.
- verify: cargo test --test cmd_tried && cargo test --lib cmd::tried::
- verified: 2026-11-23
- commit: 3ede950
- reqs: R-INV-2
- by: Reynan Henry Tandoyo
- note (2026-11-23, planner): added
- note (2026-11-23, planner): Owner, 2026-11-23: replace, do not refuse; only the four separator forms, so plain em dashes stay.
- note (2026-11-23, haiku-T-403): done

### T-405 · transfer.rs: scp in SFTP mode with an unquoted, glob-escaped remote path
- status: done
- owner: haiku-T-405
- priority: P0 · difficulty: 1 · type: feature
- milestone: T-325
- why: scp has used the SFTP protocol by default since OpenSSH 9.0; a shell-quoted remote path is then not found, and an unescaped * copies other files too (tested 2026-11-23).
- summary: Copies to and from the server find exactly the right file, also when its name has spaces, quotes or wildcard characters.
- depends: T-371
- files: src/remote/transfer.rs
- read: docs/design.md
- steps:
  1. src/remote/transfer.rs, argv: put "-s" first, before "-o", "BatchMode=yes"; everything else stays in its order. Update the doc comment to the one of docs/design.md section 17.
  2. src/remote/transfer.rs, remote_path: return `format!("{server}:{}", escaped)` where escaped is `format!("{folder}/{stored}")` with a backslash put in front of each `\`, `*`, `?`, `[` and `]`, and nothing else changed: no quotes, spaces and `'` and non-ASCII stay as they are. Do not call ssh::quote; remove the `use crate::remote::ssh;` line if nothing else uses it. Update the doc comment to the one of docs/design.md section 17.
  3. src/remote/transfer.rs, unit tests: replace the two remote_path tests: `remote_path("anna@server", "/srv/my project", "sim/run 1.sim")` is `"anna@server:/srv/my project/sim/run 1.sim"`; `remote_path("anna@server", "/srv/p", "it's Grüße [2]*?.sim")` is `"anna@server:/srv/p/it's Grüße \\[2\\]\\*\\?.sim"` (in Rust source: each backslash of the result written as `\\`); and argv's first argument is `-s`. Change any other test of argv that lists the arguments so that `-s` comes first.
  4. Run cargo test --lib remote::transfer::
- accept: argv starts with -s, and remote_path gives the path unquoted with \ * ? [ ] escaped by a backslash.
- verify: cargo test --lib remote::transfer::
- verified: 2026-11-23
- commit: 412a5de
- reqs: R-REMOTE-4, R-XOS-2
- by: Reynan Henry Tandoyo
- note (2026-11-23, planner): added
- note (2026-11-23, haiku-T-405): done

### T-406 · transfer.rs: take from a PC sends the versions and the subfolder
- status: done
- owner: haiku-T-406
- priority: P0 · difficulty: 1 · type: feature
- milestone: T-325
- why: Every forwarded command carries the PC's versions and the subfolder (design.md section 16); without them the server skips its version check and a path typed in a subfolder means another file.
- summary: A take typed in a subfolder of a connected folder takes the right file, and the server checks the versions as for any other command.
- depends: T-372
- files: src/remote/forward.rs, src/remote/transfer.rs
- read: docs/design.md
- steps:
  1. src/remote/forward.rs: `fn remote_variables` and `fn sub_folder` become `pub(crate) fn`; nothing else in forward.rs changes.
  2. src/remote/transfer.rs: take_argv gains a parameter `sub: Option<&str>` after `args`: `fn take_argv(args: &[String], sub: Option<&str>, pointer: &Pointer, agent: Option<&str>) -> Vec<String>`. Its env becomes `crate::remote::forward::remote_variables("dossier", agent)` (DOSSIER_AGENT when given, then the two version variables), and it passes `sub` as the third argument of ssh::argv_with in place of None.
  3. src/remote/transfer.rs, in `take`: compute `let sub = std::env::current_dir().ok().and_then(|current| crate::remote::forward::sub_folder(&current, base));` and pass `sub.as_deref()` to take_argv.
  4. src/remote/transfer.rs, the test take_argv_runs_take_with_json_in_the_folder_with_the_agent: call take_argv with None for sub and expect the command with `DOSSIER_AGENT='anna-claude' DOSSIER_CLIENT_VERSION='<version>' DOSSIER_CLIENT_FORMAT='1'` (take the version from env!("CARGO_PKG_VERSION") and build the expected string with format!); add one case with sub Some("sim") whose command differs from the one without sub and contains `sim` (look at the sub tests of src/remote/ssh.rs for the exact form argv_with gives a sub folder, and assert that exact string).
  5. Run cargo test --lib remote::transfer:: and cargo test --lib remote::forward::
- accept: take from a PC runs on the server with DOSSIER_CLIENT_VERSION, DOSSIER_CLIENT_FORMAT and the subfolder of the current folder.
- verify: cargo test --lib remote::transfer:: && cargo test --lib remote::forward::
- verified: 2026-11-23
- commit: 2efb71d
- reqs: R-REMOTE-4, R-XOS-9
- by: Reynan Henry Tandoyo
- note (2026-11-23, planner): added
- note (2026-11-23, haiku-T-406): done

### T-407 · transfer.rs: take copies through a part file and keeps a local file
- status: done
- owner: haiku-T-407
- priority: P0 · difficulty: 1 · type: feature
- milestone: T-325
- why: A repeated take must not wipe local edits, and a failed copy must not leave a half file (D46).
- summary: Taking a file again never overwrites the copy on the PC, and an interrupted copy leaves nothing half-written behind.
- depends: T-406
- files: src/remote/transfer.rs
- read: docs/design.md, docs/requirements.md
- steps:
  1. src/remote/transfer.rs: add `fn part_path(local: &Path) -> PathBuf`: the same path with `.dossier-part` appended to the file name (`sim/a.sim` gives `sim/a.sim.dossier-part`), built with OsString so that non-ASCII names stay as they are.
  2. src/remote/transfer.rs, copy_down returns `Result<bool>`: true when it copied, false when it kept the local file. First, when `local` already exists (`local.exists()`), return Ok(false) without running scp (D46). Otherwise create the folders as now, remove a part file left from before (ignore a NotFound error), run scp with the target `part_path(&local)` in place of `local`, and on success `std::fs::rename(part, local)`; a failed rename is the same Code::Ssh error as a failed copy. On a failed scp, remove the part file (ignore errors) and return the existing error.
  3. src/remote/transfer.rs, copy_all returns `Result<Vec<(String, bool)>>` (path, copied), and `take` prints for each: `<path>: taken and copied` when copied, else `<path>: taken; kept your local copy (delete it and run the same take again to copy the server's version)`.
  4. src/remote/transfer.rs, unit tests: part_path of `sim/a.sim` and of `Größe 1.sim`; copy_down with a local file that already exists (in a temporary folder) returns Ok(false) and the file keeps its content, with DOSSIER_SCP set to a program that does not exist so that a run of scp would fail (hold crate::ENV_LOCK and restore DOSSIER_SCP afterwards, as the other tests of this file do).
  5. Run cargo test --lib remote::transfer::
- accept: A local file that exists is kept and named in the output; a copy lands under its name only when complete.
- verify: cargo test --lib remote::transfer::
- verified: 2026-11-23
- commit: 7a27b3e
- reqs: R-FILE-2, R-REMOTE-4
- by: Reynan Henry Tandoyo
- note (2026-11-23, planner): added
- note (2026-11-23, haiku-T-407): done

### T-408 · transfer.rs: return --done uploads every file before it replaces any
- status: done
- owner: haiku-T-408
- priority: P0 · difficulty: 1 · type: feature
- milestone: T-325
- why: A rerun after a partly finished return --done is refused by --check, because a file already moved counts as changed; uploading all first keeps a failed copy retryable.
- summary: When copying files back to the server fails, the server's files stay untouched and the same command can simply be run again; if something fails later, the message says exactly how to finish.
- depends: T-373
- files: src/remote/transfer.rs
- read: docs/design.md, src/cmd/return_cmd.rs
- steps:
  1. src/remote/transfer.rs: split copy_up into `fn upload_part(local: &Path, name: &str, sub: Option<&str>, pointer: &Pointer) -> bool` (only the scp to `<sub>/<name>.dossier-part` with remote_target, as copy_up does now) and `fn move_into_place(name: &str, sub: Option<&str>, pointer: &Pointer, agent: Option<&str>) -> bool` (only the `mv -f -- <name>.dossier-part <name>` through remote, as copy_up does now). Remove copy_up.
  2. src/remote/transfer.rs, give_back keeps steps (a) `return … --check` and (b) the local files must exist, then: (c) upload_part for every file first; a failure prints `error: could not copy <path> to the server; the take is unchanged, run the same command again` and returns 5, before any file on the server was replaced. (d) move_into_place for every file. (e) `return … --done` as now.
  3. src/remote/transfer.rs: when (d) fails for a file, or (e) ends with a code other than 0, print `error: the files are on the server but the return is not recorded; finish it on the server: ssh <server>, then in <folder>[/<sub>] run: dossier return <files> --done` (server and folder from the pointer, `/<sub>` only with a sub folder, the files as typed and separated by spaces) and return 5. Do not tell the user to run the same command again there: its --check would see the replaced files as changed.
  4. src/remote/transfer.rs, the doc comment of give_back: say the order (check, all uploads, then the moves, then --done) and why (a failed upload leaves the server's files untouched; after the first move, only the server can finish).
  5. src/remote/transfer.rs, unit test: a private `fn finish_on_server(pointer: &Pointer, sub: Option<&str>, files: &[String]) -> String` that builds the message of step 3, tested with and without a sub folder and with two files, one with a space.
  6. Run cargo test --lib remote::transfer::
- accept: A failed upload leaves the server's files as taken and the same command works again; a failure after the first move names the command that finishes the return on the server.
- verify: cargo test --lib remote::transfer::
- verified: 2026-11-23
- commit: 0693af3
- reqs: R-FILE-3, R-REMOTE-4
- by: Reynan Henry Tandoyo
- note (2026-11-23, planner): added
- note (2026-11-23, haiku-T-408): done

### T-409 · transfer.rs: after a failed move, finish only the moved files on the server
- status: done
- owner: haiku-T-409
- priority: P1 · difficulty: 1 · type: feature
- milestone: T-325
- why: After a move fails on one file, finishing every file on the server would record the old, unmoved version of the later files as returned.
- summary: If moving files into place fails partway, the message says exactly which files to finish on the server and which to return again from the PC, with names that can be pasted.
- depends: T-408
- files: src/remote/transfer.rs
- read: docs/design.md
- steps:
  1. src/remote/transfer.rs, finish_on_server: quote each file with crate::remote::ssh::quote when it holds a space or another character the shell treats specially (simplest: quote every file whose quoted form differs from the plain one), so that the printed command can be pasted into a shell.
  2. src/remote/transfer.rs, give_back step (d): count the files that move_into_place moved. When the move of file k fails, print the finish message for the moved files only (files before k), and add: `then run here again: dossier return <the files from k on> --done` (quoted the same way); their files on the server are untouched, so that command passes its --check. When no file was moved yet (k is the first), print the ordinary failed-copy message `could not copy <path> to the server; the take is unchanged, run the same command again`. Return 5 in each case. When (e) `return … --done` fails, keep the message for all files.
  3. src/remote/transfer.rs, unit tests for finish_on_server: a file with a space comes out quoted (`'run 1.sim'`), a plain file stays plain; and a test of a private `fn rest_message(files: &[String]) -> String` that builds the `then run here again` line.
  4. Run cargo test --lib remote::transfer::
- accept: A failed move names the moved files for the server and the rest for a rerun from the PC, each name quoted when it needs it.
- verify: cargo test --lib remote::transfer::
- verified: 2026-11-23
- commit: eff3923
- reqs: R-FILE-3
- by: Reynan Henry Tandoyo
- note (2026-11-23, planner): added
- note (2026-11-23, haiku-T-409): done

### T-410 · transfer.rs: return --done does not copy the PC's file mode to the server
- status: done
- owner: haiku-T-410
- priority: P0 · difficulty: 1 · type: feature
- milestone: T-054
- why: scp -p from Windows sends mode 0666, so a returned file became writable for everyone on the server (found in the end-to-end check, 2026-11-23).
- summary: A file returned from a Windows PC gets the server's usual permissions instead of being writable for everyone.
- depends: T-375
- files: src/remote/transfer.rs
- read: docs/design.md
- steps:
  1. src/remote/transfer.rs, upload_part: build the scp arguments with argv(...) as now, then remove the "-p" element before running scp, with a comment: scp -p from Windows sends the mode 0666 (Windows has no Unix modes), which would make the returned file writable for everyone on the server; without -p the server gives the new file its usual mode. Downloads (copy_down) keep -p.
  2. src/remote/transfer.rs, unit test: a private `fn upload_argv(source: &str, target: &str) -> Vec<String>` used by upload_part, tested to start with `-s` and to contain no `-p`, while argv(...) still contains `-p`.
  3. Run cargo test --lib remote::transfer::
- accept: An upload runs scp without -p; a download keeps -p.
- verify: cargo test --lib remote::transfer::
- verified: 2026-11-23
- commit: 1737e5e
- reqs: R-REMOTE-4
- by: Reynan Henry Tandoyo
- note (2026-11-23, planner): added
- note (2026-11-23, haiku-T-410): done

### T-411 · secret.rs: build the AWS-shaped test values at run time
- status: done
- owner: haiku-T-411
- priority: P1 · difficulty: 1 · type: chore
- milestone: T-054
- why: GitHub secret scanning reports the invented AWS key of the T-288 tests as a leaked secret (alert #1).
- summary: The tests of the credential check no longer contain text that GitHub's secret scanning reports as a leaked key.
- files: src/secret.rs
- read: src/secret.rs
- steps:
  1. src/secret.rs, in the tests module: add `const AWS_BODY: &str = "Q7WXYZ3JKLMN2PQR";` next to `const BODY`, with a comment: the AWS-shaped test values are built at run time, so that the source holds no literal that secret scanners report as a leaked key.
  2. src/secret.rs, the five lines of the AWS test (the AWS test, five lines): build each value with format! from the prefix and AWS_BODY and keep exactly the same text as before: `format!("AKIA{AWS_BODY}")`, `format!("ASIA{AWS_BODY}")`, `format!("AKIA{AWS_BODY}S")`, `format!("AKIA{}", &AWS_BODY[..15])`, `format!("AKIA{}", AWS_BODY.to_lowercase())`. The expected results stay as they are.
  3. Check that no other line of the repository (outside target/) matches the pattern `(AKIA|ASIA)[A-Za-z0-9]{12,}` any more, for example with git grep -E.
  4. Run cargo test --lib secret::
- accept: The tests check the same texts as before, and git grep finds no AKIA or ASIA key-shaped literal.
- verify: cargo test --lib secret::
- verified: 2026-11-23
- commit: bde5f8b
- reqs: R-CONF-7
- by: Reynan Henry Tandoyo
- note (2026-11-23, planner): added
- note (2026-11-23, haiku-T-411): done

### T-414 · Readers' guide and README: the answer 'edit first' of approve
- status: open
- owner: —
- priority: P1 · difficulty: 1 · type: docs
- milestone: T-058
- why: The readers' guide describes the answers of approve, and gains the fourth with T-413.
- summary: The guide for readers explains that a draft can be corrected in the editor during approve.
- depends: T-413
- files: docs/guide-for-readers.md, README.md
- read: src/cmd/approve.rs
- steps:
  1. docs/guide-for-readers.md: where the answers of approve are described, add the fourth, `e` (edit first): it opens the draft in the editor (VISUAL, else EDITOR, else Notepad on Windows); after saving, the changed draft is shown again and approve asks once more; the change is kept and recorded even when the draft is then skipped. Remove the sentence that edits go through the writer, if it is there.
  2. README.md: where approve and its answers are described, name the four answers a, e, r and s in the same way (one sentence). Change nothing else.
  3. Run cargo test --test smoke
- accept: The guide and the README name the four answers of approve.
- verify: cargo test --test smoke
- reqs: R-DRAFT-4, REL-4
- note (2026-11-23, planner): added

### T-416 · hooks.rs: the session-end hook runs in bash
- status: open
- owner: —
- priority: P1 · difficulty: 1 · type: feature
- milestone: T-339
- why: Its command is quoted for a POSIX shell; on Windows without Git Bash, Claude Code would run it in PowerShell, where that quoting breaks.
- summary: The session review hook runs the same way on every PC, because it says which shell runs it.
- depends: T-338
- files: src/hooks.rs
- read: docs/design.md
- steps:
  1. src/hooks.rs, review_installed: the SessionEnd hook object it writes gains `"shell": "bash"` next to `"type": "command"` and `"command"`, with a comment: the command is quoted for a POSIX shell, and Claude Code runs a hook on Windows in Git Bash, or in PowerShell when Git Bash is not installed, unless `shell` says which (code.claude.com/docs/en/hooks). A SessionEnd dossier hook that is replaced in place gets the field too. The other hooks (brief, guard, after) stay as they are: their commands have no quotes.
  2. src/hooks.rs, unit tests: the installed SessionEnd hook has `"shell": "bash"`; replacing an existing one keeps exactly one hook with the field; the SessionStart hook still has no `shell` field.
  3. src/hooks.rs, review_installed: a command that is empty or only spaces is refused with Code::BadValue `the review command must not be empty` before anything is changed; unit test for it.
  4. Run cargo test --lib hooks::
- accept: The SessionEnd hook that hooks install writes says shell: bash; the other hooks are unchanged.
- verify: cargo test --lib hooks::
- reqs: R-DRAFT-9
- note (2026-11-24, planner): added

### T-417 · brief: one summed-up line per digest, pointing to dossier show
- status: done
- owner: haiku-T-417
- priority: P1 · difficulty: 1 · type: feature
- milestone: T-056
- why: The summed-up line pointed to list --task <task>, which cannot name a milestone of several tasks (found in T-333).
- summary: brief tells for each digest how many entries it sums up and how to see them.
- depends: T-333
- files: src/cmd/brief.rs, tests/cmd_brief.rs
- read: docs/design.md
- steps:
  1. src/cmd/brief.rs: replace the one summed-up line (summed_text) by one line per digest in force that covers at least one entry: `<n> entries of <milestone> are summed up in <DIG-ID>: dossier show <DIG-ID>`, or `1 entry of <milestone> is summed up in <DIG-ID>: dossier show <DIG-ID>`, where <milestone> is the digest's `milestone` field and <n> the number of decisions and findings it covers that brief left out. These lines are never cut, like the digest lines. Update the doc comment to docs/design.md section 18.
  2. tests/cmd_brief.rs: change the tests that expect `are summed up in digests: dossier list --task` to the new line; one test with two digests of two milestones expects two lines, each naming its milestone and DIG-ID.
  3. Run cargo test --test cmd_brief
- accept: brief prints one summed-up line per digest with its milestone and DIG-ID and the command dossier show <DIG-ID>.
- verify: cargo test --test cmd_brief
- verified: 2026-11-24
- commit: cd9effa
- reqs: R-CTX-14
- by: Reynan Henry Tandoyo
- note (2026-11-24, planner): added
- note (2026-11-24, haiku-T-417): done

### T-418 · status: the summed-up line names the digest's milestone
- status: done
- owner: haiku-T-418
- priority: P1 · difficulty: 1 · type: feature
- milestone: T-056
- why: brief and status must print the same summed-up line (design.md section 18, changed after T-333).
- summary: status says, like brief, which milestone each digest sums up.
- depends: T-334
- files: src/cmd/status.rs, tests/cmd_status.rs
- read: docs/design.md
- steps:
  1. src/cmd/status.rs: the per-digest count line (now `<n> entries are summed up in digests: dossier show <DIG-ID>`) becomes the line of docs/design.md section 18: `<n> entries of <milestone> are summed up in <DIG-ID>: dossier show <DIG-ID>`, or `1 entry of <milestone> is summed up in <DIG-ID>: dossier show <DIG-ID>`, with <milestone> the digest's `milestone` field. Nothing else changes; update the doc comment.
  2. tests/cmd_status.rs: the tests that expect the old line expect the new one.
  3. Run cargo test --test cmd_status
- accept: status prints the same summed-up line as brief.
- verify: cargo test --test cmd_status
- verified: 2026-11-24
- commit: eb9cf78
- reqs: R-CTX-14
- by: Reynan Henry Tandoyo
- note (2026-11-24, planner): added
- note (2026-11-24, haiku-T-418): done

### T-419 · digest --write: refuse a heading line in the summary
- status: open
- owner: —
- priority: P1 · difficulty: 1 · type: feature
- milestone: T-331
- why: A summary line starting with ## would open a new section in the digest's file (found in T-379); problem refuses headings the same way.
- summary: A digest summary cannot accidentally break the structure of its entry file.
- depends: T-379
- files: src/cmd/digest.rs, tests/cmd_digest.rs
- read: docs/design.md, src/cmd/problem.rs
- steps:
  1. src/cmd/digest.rs, where the summary from standard input is checked (empty, more than 15 lines): also refuse, with Code::BadValue and nothing written, a summary that has a line starting with `##` after leading spaces: `the summary must not contain a heading line (##): line <n>`. A line inside the text that merely contains `##` later is fine.
  2. tests/cmd_digest.rs: a summary with a line `## Results` is refused with exit code of BadValue, the message names the line number, and no DIG file exists afterwards; a line `see ## in the log` is accepted.
  3. Run cargo test --test cmd_digest
- accept: digest --write refuses a summary with a heading line and writes nothing.
- verify: cargo test --test cmd_digest
- reqs: R-CTX-14
- note (2026-11-24, planner): added

### T-421 · search --similar: show why each entry is similar
- status: open
- owner: —
- priority: P2 · difficulty: 1 · type: feature
- milestone: T-058
- why: Each result printed its title twice (found in T-345); the reason of select.rs says more.
- summary: search --similar tells for each result what it shares with the entry, instead of repeating its title.
- depends: T-345, T-392
- files: src/cmd/search.rs, tests/cmd_search.rs
- read: src/select.rs, docs/design.md
- steps:
  1. src/cmd/search.rs, --similar: after the dash, print why the entry was chosen, the reason that select::reason gives since T-392 (`tag match` or `word match`), in place of repeating the title. The plain word search stays as it is. In --json, each result of --similar gains the field `reason` with that text.
  2. tests/cmd_search.rs: the --similar tests expect the reason after the dash and no repeated title; one test checks the `reason` field in --json.
  3. Run cargo test --test cmd_search
- accept: search --similar prints the reason after the dash and gives it in --json.
- verify: cargo test --test cmd_search
- reqs: R-READ-9
- note (2026-11-24, planner): added

## Questions

### T-061 · Does a lock taken through the Samba share see a lock taken on the Linux server? Needs a test on the real server (open point O-1); decides whether R-LOCK-3 refuses or only warns.
- status: done
- owner: planner
- type: question
- about: T-079
- note (2026-11-20, planner): added
- note (2026-11-21, planner): about moved from T-006 (dropped) to T-079 (lock.rs), which now holds the lock work.
- note (2026-11-23, planner): answer — 2026-11-23: no test needed. Every write runs on the server, through ssh (D9, D43), so the lock is always taken on the server, where it works. Writing through the network share stays refused (R-LOCK-3); reading through it is always safe. Reopen with todo reopen T-061 if writing through the share is ever wanted.

### T-317 · The ssh program of Windows cannot keep a connection open for reuse, so 'under half a second per command' (R-REMOTE-7) cannot be met from a Windows PC the way the requirement describes. The plan builds reuse for Linux and macOS only. Is that enough, or should the PC program keep its own helper process running in the background to hold the connection (more code, and a process that stays alive on each PC)?
- status: done
- owner: planner
- type: question
- about: T-053
- note (2026-11-23, planner): added
- note (2026-11-23, planner): answer — Owner, 2026-11-23 (D43): keep it simple. No helper process and no connection reuse; each command makes its own ssh connection, and the project folder can always be read through the network share. R-REMOTE-7 is not built; T-311 and T-316 were changed accordingly.

### T-356 · review-session reads Claude Code's transcript file. Claude Code's documentation says this format is internal and can change with any release. The plan reads it leniently (unknown lines are skipped), so after a format change the material would come out empty until dossier is updated, and nothing breaks. Do you want the session review (R-DRAFT-7, R-DRAFT-9) built on that basis, or left out of version 0.5?
- status: done
- owner: planner
- type: question
- about: T-057
- note (2026-11-23, planner): added
- note (2026-11-23, planner): answer — Owner, 2026-11-23 (D44): dossier gets its own session file format with a version number (format.md section 34). review-session reads that; a Claude Code transcript is converted first, and only the converter (src/claude.rs) knows Claude Code's format. T-336, T-337, T-339 and T-340 were changed accordingly.

## Ideas

### T-404 · take.rs and return_cmd.rs both hold stored_path and ensure_file (copied in T-322 under the three-file limit); move them to one place, for example taken.rs.
- status: open
- owner: —
- type: idea
- note (2026-11-23, planner): added

### T-412 · TEST-10 (D47): before take and return are used on real work, run the manual test on the real server: write refused on the network share; connect --check and approve from PowerShell; take, take again (kept), edit, return --check and --done of a file over 1 GB with a space in its path, checksums compared on both sides; two people: refused take, take --force, the first person's return --done refused; an interrupted copy leaves no half file and the same take copies again; the returned file's permissions let colleagues write (sftp-server umask, see the note on T-054). Plan: docs/requirements.md TEST-10. Promote to a task when large files go into use.
- status: open
- owner: —
- type: idea
- note (2026-11-23, planner): added

### T-413 · approve: the answer 'edit first'
- status: open
- owner: —
- priority: P1
- difficulty: 1
- type: feature
- milestone: T-058
- why: R-DRAFT-4 (MUST) names four answers; approve offered three, so a reader could not correct a draft before approving it (found in T-354).
- summary: During approve a person can open a draft in the editor, correct it, and then approve, reject or skip it.
- files: src/cmd/approve.rs, src/cmd/edit.rs, tests/cmd_approve.rs
- read: docs/requirements.md, src/cmd/edit.rs, tests/cmd_edit.rs
- steps:
  1. src/cmd/edit.rs: `fn editor_program`, `fn edit_in_temp_file` and `fn refuse_tool_lines` become `pub(crate) fn`; nothing else in edit.rs changes.
  2. src/cmd/approve.rs, review: keep `let mut current = item.entry.clone();` and ask in a loop with the prompt `<ID>: approve, edit first, reject or skip? (a/e/r/s) `. `e` or `edit`: run `crate::cmd::edit::edit_in_temp_file(&crate::cmd::edit::editor_program(), &id, &current.text())`; when the text came back unchanged, ask again; else parse it with Entry::parse, call `crate::cmd::edit::refuse_tool_lines(&current, &edited)` (an error ends the command, as in edit), add the history line `changed — edited before approval` with `edited.add_history(tx.time(), who, "changed", "edited before approval")`, set `current = edited`, show it again with show(&current.text()), check rules::problems(&current) as before the first question (problems: skipped with the same warning, but the edited text is still written), and ask again. `a`, `r` and anything else end the loop as now.
  3. src/cmd/approve.rs: approve and reject take the entry to write: `fn approve(tx: &mut Tx, item: &Item, entry: &Entry, who: &Who)` and `fn reject(tx: &mut Tx, item: &Item, entry: &Entry, who: &Who)` start from `entry.clone()` in place of `item.entry.clone()`. A skip after an edit queues the edited draft with `tx.put(item.path.clone(), Some(item), current.text(), change(id, "changed"))`, so that the edit is kept; a skip without an edit writes nothing, as now. Only one tx.put per draft. Update the module doc comment and the doc comment of review to name the four answers (R-DRAFT-4).
  4. tests/cmd_approve.rs: with the stand-in editor of tests/cmd_edit.rs that appends `Added by the editor.` (copy its two texts and write it with Project::write_stand_in), set as VISUAL: answers `e|a` approve the draft, its text contains `Added by the editor.`, its history has `edited before approval` before `approved`; answers `e|s` leave a draft that contains the added line; with an editor that saves the file unchanged, `e|r` rejects it without a `changed` history line.
  5. Run cargo test --test cmd_approve
- accept: approve asks a/e/r/s; e opens the editor, shows the changed draft and asks again; the edit is kept with a history line, whatever the answer after it.
- verify: cargo test --test cmd_approve
- reqs: R-DRAFT-4
- note (2026-11-23, planner): added

### T-415 · src/digest.rs copies the date rule and the newest-first sort of filter.rs and cmd/brief.rs (T-330, three-file limit); share them in one place.
- status: open
- owner: —
- type: idea
- note (2026-11-24, planner): added

### T-420 · add (and the other commands that take a multi-line text) do not refuse a line starting with ## , which opens a new section in the entry file; problem and digest --write refuse it (found in T-379).
- status: open
- owner: —
- type: idea
- note (2026-11-24, planner): added
