<!-- todo-format: 3 -->
<!-- todo-tool: 0.8.1 -->
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
- status: open
- owner: —
- priority: P1 · type: milestone
- goal: The project's key numbers, the files of each task, the simulation runs and the exchange with the customer are recorded, so that for a delivered report the dossier names the model, runs, values and assumptions behind every result. Requirements: docs/requirements.md section 11, Phase 2 (v0.2). Each task here is split when the phase starts.
- note (2026-11-20, planner): added

### T-032 · Key values: one sheet of named numbers with units
- status: open
- owner: —
- priority: P1 · type: milestone
- milestone: T-031
- goal: value set, value show and values work as in R-VALUE-1 to R-VALUE-6; brief and status print the key values.
- why: One number written in five places is soon five numbers, and an LLM cannot tell which is current.
- depends: T-030
- accept: value set, value show and values work as in R-VALUE-1 to R-VALUE-6; brief and status print the key values.
- reqs: R-VALUE-1 to R-VALUE-6, D23
- note (2026-11-20, planner): added
- note (2026-11-21, planner): split into T-175, T-176, T-177

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
- status: open
- owner: —
- priority: P1 · type: milestone
- milestone: T-031
- goal: The file list of each task by role with made-from links works as in R-FILE-8 to R-FILE-16.
- why: A reader must find the simulation file, geometry and results of a task without searching the disk, and see which results are outdated.
- depends: T-030
- accept: The file list of each task by role with made-from links works as in R-FILE-8 to R-FILE-16.
- reqs: R-FILE-8 to R-FILE-16, D18
- note (2026-11-20, planner): added
- note (2026-11-20, planner): Also covers R-XOS-5: warn about file names that cannot exist on Windows or differ only in case.
- note (2026-11-21, planner): split into T-168, T-169, T-170, T-171, T-172, T-173, T-174

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
- status: open
- owner: —
- priority: P1 · type: milestone
- milestone: T-031
- goal: used, ran, runs and runs --compare work as in R-USE-2, R-USE-14 and R-USE-15, with software names kept in the project.
- why: Which software version and which parameters produced a result is asked in every review.
- depends: T-033
- accept: used, ran, runs and runs --compare work as in R-USE-2, R-USE-14 and R-USE-15, with software names kept in the project.
- reqs: R-USE-2, R-USE-14, R-USE-15, R-ENTRY-9
- note (2026-11-20, planner): added
- note (2026-11-21, planner): split into T-195, T-196

### T-195 · used: record a run of a software
- status: claimed
- owner: haiku-t-195
- claimed: 2026-11-21T12:19+01:00
- session: 4ec70cb4-7c49-41fb-a6e6-07f59dad766d
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
- reqs: R-USE-2, R-USE-14, R-ENTRY-9
- note (2026-11-21, planner): added

### T-196 · runs: the recorded runs, and what differs between two
- status: open
- owner: —
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
- reqs: R-USE-15
- note (2026-11-21, planner): added

### T-035 · Pictures in entries
- status: open
- owner: —
- priority: P1 · type: milestone
- milestone: T-031
- goal: --image, image add and image remove copy small pictures into the dossier and show them as in R-ENTRY-16 and R-ENTRY-17.
- why: Engineering evidence is mostly plots, and a lesson whose plot was deleted has lost its evidence.
- depends: T-030
- accept: --image, image add and image remove copy small pictures into the dossier and show them as in R-ENTRY-16 and R-ENTRY-17.
- reqs: R-ENTRY-16, R-ENTRY-17, D26
- note (2026-11-20, planner): added
- note (2026-11-21, planner): split into T-197, T-198

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
- status: open
- owner: —
- priority: P1 · type: milestone
- milestone: T-031
- goal: propose, accept and decline work as in R-WRITE-12, and decide follows R-WRITE-13.
- why: Three people and three LLMs must be able to suggest a direction without it becoming binding by accident.
- depends: T-030
- accept: propose, accept and decline work as in R-WRITE-12, and decide follows R-WRITE-13.
- reqs: R-WRITE-12, R-WRITE-13, D21
- note (2026-11-20, planner): added
- note (2026-11-21, planner): split into T-186, T-187, T-188, T-189, T-190

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
- status: open
- owner: —
- priority: P1 · type: milestone
- milestone: T-031
- goal: confirm, refute and cleared work as in R-WRITE-15, with marks shown in show, files, status and brief.
- why: Results built on a wrong assumption must be found again.
- depends: T-033
- accept: confirm, refute and cleared work as in R-WRITE-15, with marks shown in show, files, status and brief.
- reqs: R-WRITE-15
- note (2026-11-20, planner): added
- note (2026-11-21, planner): split into T-191, T-192, T-193, T-194

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
- status: open
- owner: —
- priority: P2 · type: milestone
- milestone: T-031
- goal: --on <date> works on the commands named in R-LIFE-9.
- why: What did we know and decide on the day the report went out must have an answer.
- depends: T-033
- accept: --on <date> works on the commands named in R-LIFE-9.
- reqs: R-LIFE-9
- note (2026-11-20, planner): added
- note (2026-11-21, planner): split into T-185

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
- status: open
- owner: —
- priority: P1 · type: milestone
- milestone: T-031
- goal: received, delivered and exchange work as in R-EXCH-1 to R-EXCH-5, including the warning before delivering outdated results.
- why: What was received and delivered, when and in which version, is the first thing asked in a dispute.
- depends: T-033
- accept: received, delivered and exchange work as in R-EXCH-1 to R-EXCH-5, including the warning before delivering outdated results.
- reqs: R-EXCH-1 to R-EXCH-5
- note (2026-11-20, planner): added
- note (2026-11-21, planner): split into T-180, T-181, T-182, T-183, T-184

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
- status: claimed
- owner: haiku-t-184
- claimed: 2026-11-21T12:19+01:00
- session: 4ec70cb4-7c49-41fb-a6e6-07f59dad766d
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
- reqs: R-EXCH-3
- note (2026-11-21, planner): added

### T-040 · Code projects: map of the code, pointers into code, init --code-project
- status: open
- owner: —
- priority: P2 · type: milestone
- milestone: T-031
- goal: map set, map, code pointers and init --code-project work as in R-CODE-1 to R-CODE-5.
- why: The owner's own software is the first test bed, and an LLM needs orientation in thousands of files before it searches.
- depends: T-030
- accept: map set, map, code pointers and init --code-project work as in R-CODE-1 to R-CODE-5.
- reqs: R-CODE-1 to R-CODE-5
- note (2026-11-20, planner): added
- note (2026-11-21, planner): split into T-178, T-179

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
- status: open
- owner: —
- priority: P1 · type: milestone
- milestone: T-031
- goal: R-CTX-16 holds; format.md and the tests cover every phase 2 addition; version 0.2.0 is released.
- why: What phase 2 records only helps when it reaches the LLM's context.
- depends: T-033
- accept: R-CTX-16 holds; format.md and the tests cover every phase 2 addition; version 0.2.0 is released.
- reqs: R-CTX-16
- note (2026-11-20, planner): added
- note (2026-11-21, planner): split into T-199, T-200, T-201, T-202, T-203, T-204, T-205, T-206

### T-199 · show: marks for proposals, declined entries and rests-on
- status: open
- owner: —
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
- reqs: R-WRITE-15, R-READ-1
- note (2026-11-21, planner): added

### T-200 · brief: map, key values, proposals, rests-on marks and --on
- status: claimed
- owner: haiku-t-200
- claimed: 2026-11-21T12:19+01:00
- session: 4ec70cb4-7c49-41fb-a6e6-07f59dad766d
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
- reqs: R-CTX-16, R-CODE-1, R-LIFE-9
- note (2026-11-21, planner): added

### T-201 · status: key values, proposals, marks, last exchange and --on
- status: claimed
- owner: haiku-t-201
- claimed: 2026-11-21T12:19+01:00
- session: 4ec70cb4-7c49-41fb-a6e6-07f59dad766d
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
- reqs: R-CTX-16, R-EXCH-5, R-LIFE-9
- note (2026-11-21, planner): added

### T-202 · brief --task: files, dead ends, pictures and code pointers
- status: claimed
- owner: haiku-t-202
- claimed: 2026-11-21T12:19+01:00
- session: 4ec70cb4-7c49-41fb-a6e6-07f59dad766d
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
- reqs: R-CTX-16, R-FILE-14, R-ENTRY-17, R-CODE-4
- note (2026-11-21, planner): added

### T-203 · check: block files and rests-on links
- status: open
- owner: —
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
- reqs: R-CHECK-1
- note (2026-11-21, planner): added

### T-204 · The rules text for LLMs: phase 2 commands
- status: open
- owner: —
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
- reqs: R-AGENT-1
- note (2026-11-21, planner): added

### T-205 · README for v0.2
- status: open
- owner: —
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
- reqs: REL-2
- note (2026-11-21, planner): added

### T-206 · Release 0.2.0
- status: open
- owner: —
- priority: P1 · difficulty: 2 · type: chore
- milestone: T-041
- why: Phase 2 ends with a release that is used on real work.
- final: Checking every command by hand cannot be listed in advance.
- depends: T-199, T-200, T-201, T-202, T-203, T-204, T-205, T-032, T-033, T-034, T-035, T-036, T-037, T-038, T-039, T-040, T-155, T-156, T-157, T-158, T-159, T-160, T-161, T-162, T-163, T-164, T-165, T-166, T-167
- files: Cargo.toml, Cargo.lock
- read: docs/requirements.md
- steps:
  1. Set the version in Cargo.toml to 0.2.0 and run cargo build so Cargo.lock follows.
  2. Run every check on Windows, cargo clippy --target x86_64-unknown-linux-gnu --all-targets --all-features -- -D warnings, and run every phase 2 command once by hand in a temporary folder; note anything that fails with todo add.
  3. Tell the owner that the tag v0.2.0 can be pushed after CI passes; do not push.
- accept: Version 0.2.0 builds and every check passes on both systems.
- verify: cargo build
- reqs: REL-1
- note (2026-11-21, planner): added

### T-042 · Team: knowledge reaches the next project without customer data
- status: open
- owner: —
- priority: P1 · type: milestone
- goal: One team dossier holds lessons, how-tos, scripts, the software list and a glossary; entries are promoted from projects only after the confidentiality check; finished projects of the past are brought in. Requirements: docs/requirements.md section 11, Phase 3 (v0.3). Each task here is split when the phase starts.
- note (2026-11-20, planner): added

### T-043 · Team dossier: init --team-dossier, project codes, team settings, team members
- status: open
- owner: —
- priority: P1 · difficulty: 5 · type: feature
- milestone: T-042
- why: Knowledge locked in one project folder is lost to the next project.
- depends: T-041
- accept: R-PLACE-3 to R-PLACE-6, R-TEAM-1, R-CFG-2 and R-CFG-4 hold; people move from the project settings to the team settings.
- reqs: R-PLACE-3 to R-PLACE-6, R-TEAM-1, R-TEAM-7, R-CFG-2, R-CFG-4, D3, D29
- note (2026-11-20, planner): added
- note (2026-11-21, planner): Owner 2026-11-21: one team dossier path, given with init --team. Only members write to it; team member add <person> [--llm <name>] and team member remove <person> (a person at a terminal), each saved with who and when (R-TEAM-7, D29).

### T-044 · Confidentiality: confidential terms and the check before any team write
- status: open
- owner: —
- priority: P0 · difficulty: 5 · type: feature
- milestone: T-042
- why: The team works for several customers under confidentiality agreements; this must exist before anything is promoted.
- depends: T-043
- accept: R-CONF-1 to R-CONF-6 and R-ENTRY-18 hold, with the tests of TEST-6.
- reqs: R-CONF-1 to R-CONF-6, R-ENTRY-18, D8, D32, TEST-6
- note (2026-11-20, planner): added
- note (2026-11-21, planner): Owner 2026-11-21 (D32): anyone in the project adds a confidential term, also an LLM when its person tells it in the session (the rules text must say: add it at once with dossier confidential add); confidential remove needs a person at a terminal. No automatic terms; writing to the team dossier is refused until the project has at least one term or a person ran confidential none --yes.

### T-045 · Promotion and closing a project: promote, close
- status: open
- owner: —
- priority: P1 · difficulty: 5 · type: feature
- milestone: T-042
- why: Promotion is the only way project knowledge becomes team knowledge.
- depends: T-044
- accept: promote and close work as in R-TEAM-2 to R-TEAM-4 and R-TEAM-6; search and brief include the team dossier.
- reqs: R-TEAM-2 to R-TEAM-4, R-TEAM-6, R-WRITE-5, R-WRITE-6, D28, D30
- note (2026-11-20, planner): added
- note (2026-11-21, planner): Owner 2026-11-21 (D30): only knowledge (entry text) is promoted to the team dossier; simulation files, CAD parts, spreadsheets and scripts are never copied, only pointed to from their project (small pictures are the exception, confirmed one by one, R-ENTRY-18). The team copy says source: <code>:<ID>, the project entry promoted: team:<ID>. close is run by a person in deciders (D28).

### T-046 · How-tos and the usage index: howto, where-used (software names come from tools, T-047)
- status: open
- owner: —
- priority: P1 · difficulty: 5 · type: feature
- milestone: T-042
- why: How and where a software was used for a task is asked at the start of every similar task.
- depends: T-044
- accept: R-USE-1, R-USE-3 to R-USE-7 and R-ENTRY-8 hold.
- reqs: R-USE-3 to R-USE-7, R-ENTRY-8, R-ENTRY-9, D31
- note (2026-11-20, planner): added
- note (2026-11-21, planner): Owner 2026-11-21 (D31): there is no separate software list; software is a tool of type software (T-047). The usage index and how-tos name tools by ID or name.

### T-047 · Tools: one kind for scripts, commands, installed software and git repositories (TOOL-001)
- status: open
- owner: —
- priority: P1 · difficulty: 5 · type: feature
- milestone: T-042
- why: A script nobody can find or trust is rewritten.
- depends: T-044
- accept: R-USE-8 to R-USE-13 and R-ENTRY-12 hold.
- reqs: R-USE-8 to R-USE-13, R-USE-1, R-ENTRY-12, D19, D31
- note (2026-11-20, planner): added
- note (2026-11-21, planner): Owner 2026-11-21 (D31): one entry kind tool (prefix TOOL) with type script | command | software | repo and a location (a path on the server, a command name on PATH, the install place, a git URL). Checks: a script file still matches its checksum, a command still exists on the server; software and repos are not checked. Sections Purpose, How to run, Inputs, Outputs, Traps, Tested with. Replaces the software list, script entries and script_root; R-USE-1 and R-USE-8 to R-USE-13 are rewritten when phase 3 starts. The team standardises its tools in the meantime.

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
- note (2026-11-21, planner): Owner 2026-11-21 (D33): no default review date; it is set per entry with --review <date>.

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

### T-057 · Session review (R-DRAFT-7, R-DRAFT-9)
- status: open
- owner: —
- priority: P3 · difficulty: 5 · type: feature
- milestone: T-055
- why: Capture only becomes a habit when the routine steps happen by themselves.
- depends: T-054
- accept: R-DRAFT-7, R-DRAFT-9, R-AGENT-3, R-AGENT-4, R-CTX-7 hold.
- reqs: R-DRAFT-7, R-DRAFT-9, R-AGENT-3, R-AGENT-4, R-CTX-7
- note (2026-11-20, planner): added
- note (2026-11-21, planner): Claude Code hooks and the skill (R-AGENT-3, R-AGENT-4, R-CTX-7) moved to phase 2 by the owner (D33): T-163 to T-167. What stays here is the session review (R-DRAFT-7, R-DRAFT-9).

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
- note (2026-11-21, planner): Owner 2026-11-21 (D33): export writes Markdown only.

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
- status: open
- owner: —
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
- reqs: D29
- note (2026-11-21, planner): added

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
- status: open
- owner: —
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
- reqs: R-AGENT-3, R-CTX-7, D33
- note (2026-11-21, planner): added

### T-166 · guard: the PreToolUse hook that blocks direct edits of DOSSIER
- status: open
- owner: —
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
- reqs: R-AGENT-3
- note (2026-11-21, planner): added

### T-167 · skill install
- status: open
- owner: —
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
- reqs: R-AGENT-4
- note (2026-11-21, planner): added

## Questions

### T-061 · Does a lock taken through the Samba share see a lock taken on the Linux server? Needs a test on the real server (open point O-1); decides whether R-LOCK-3 refuses or only warns.
- status: open
- owner: —
- type: question
- about: T-079
- note (2026-11-20, planner): added
- note (2026-11-21, planner): about moved from T-006 (dropped) to T-079 (lock.rs), which now holds the lock work.

