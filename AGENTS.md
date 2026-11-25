# Working on this repository

`dossier` is a Rust command-line tool that keeps what an engineering team learns and decides in
its projects. It works next to [`todo`](https://github.com/reynanTandoyo/todo) and needs `todo`
0.8 or newer.

## Read first

- [`docs/requirements.md`](docs/requirements.md): what to build. Requirement IDs such as
  `R-WRITE-2` are referenced everywhere.
- [`docs/format.md`](docs/format.md): the files of a dossier. Follow it exactly.
- [`docs/design.md`](docs/design.md): how the code is laid out. Every type and function
  signature is fixed there; one task fills in one file.
- [`TODO.md`](TODO.md): the work plan, changed only through `todo`.

## How to work

1. Take a task: `todo next --claim --brief --agent <your name>`.
2. Change only the files the task lists, and follow its steps. If a signature in `design.md`
   doesn't fit, don't change it on your own: give the task back with `todo stuck`.
3. Run the task's `verify` command, then the checks below.
4. Commit only the task's files, starting the message with the task ID.
5. `todo done <id> --commit HEAD --agent <your name>`.
6. Never push. The owner decides when to push.

## Checks

```
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
```

## Code rules

- Rust 1.89 or newer, edition 2024. Only the dependencies in `Cargo.toml`; a new one needs a
  reason in the commit message.
- No `unwrap`, `expect`, `panic!`, `todo!` or `unimplemented!` outside tests. Every failure is an
  `Error` with a code (`src/error.rs`).
- No `println!` or `eprintln!` for command output; return an `Output` (`src/output.rs`).
- Never write an entry file directly; every change goes through `tx::write` (`src/tx.rs`).
- Never rebuild an entry file from parsed data. Change whole lines and keep every other byte.
- Every change records who and when: a history line in the entry and a journal line.
- Paths with spaces and non-ASCII characters must work, on Windows and Linux.

## Test rules

- Test data is invented. Never copy a real project, customer name or person's name into this
  repository.
- Tests run in temporary folders and never touch a real dossier, a real server or the user's
  project register.
- Use the fixed clock (`DOSSIER_NOW`) so that tests give the same result every time.

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

<!-- dossier:begin -->
# Working with the dossier

- Use one unique name per agent: pass `--agent <name>` or set `DOSSIER_AGENT`.
- Write under your own name: `<your person> - <your model>`, for example `tony - claude opus 5.5 high` (DOSSIER_AGENT).
- Start a session with `dossier brief`.
- Before a task, run `dossier brief --task <id>`.
- Before work that no task asks for, run `dossier status`.
- Work only on a task you have claimed in `todo`.
- Decisions in force bind you: if your task contradicts one, stop and ask your person; never work around it.
- When your person decides something, record it at once: `dossier decide "<decision>" --reason "<why>" --said-by <person>`.
- If `dossier decide` lists related decisions, run it again with `--replaces <ID>` when yours replaces one, or `--besides` when both stay in force.
- Take every number from `dossier values`, never from memory.
- When your person is not a decider, suggest with `dossier propose`; a decider accepts it.
- Record what you observe with `dossier found "<observation>"`, and what was tried and failed with `dossier found "<what was tried>" --dead-end`.
- When something fails and you start looking for the cause, open an investigation: `dossier problem "<what goes wrong>" --task <id>`, or continue the open one; read its attempts first (`dossier show <ID>`).
- Record one line for each hypothesis, also the failed ones, not one for each compile or run: `dossier tried <ID> "<hypothesis>" --failed|--worked|--unclear --result "<what happened>"`; close with `dossier solved <ID> "<what solved it>"`.
- Attach the files a task reads and writes with `dossier attach <task> <path> --role <role>`.
- Record a simulation run with `dossier used <software> --version <v> --task <task> --model <file> --output <path>`.
- Record what you had to assume with `dossier assume "<assumption>" --reason "<why>"`.
- After `todo done <id>`, run `dossier review-task <id>`, then propose lessons with `dossier learn` (it needs `--valid-for`, `--evidence` and `--confidence`), or propose none if nothing was learned.
- When a milestone is closed in todo, sum it up: run `dossier digest <milestone>`, then write at most 15 lines with `dossier digest <milestone> --write --stdin`; a person approves it.
- Search before you propose something new: `dossier search <words>`.
- `dossier search` lists entries that hold only some of your words after the line `some of the words:`. Before you conclude that nothing is recorded, search once more with other words, and look at `dossier list --topic <name>`.
- Before writing a script, look for a tool: `dossier list --kind tool`.
- Write every entry in English, also when the session is in another language.
- When drafting from an old document, add `--from-document <file>`.
- Give every entry its topic with `--topic`. `dossier topics` lists the project's topics; if none fits, ask your person to add one. You cannot add a topic yourself.
- When an entry is about particular code, name it with `--code`: a file, a `file::name`, or the folder when it holds for a whole part. Whoever edits that code later is then shown your entry.
- Never edit files under `DOSSIER/` directly; use the commands.
- Your first edit of a file may be refused with a list of entries in force for it. They bind your change: read them (`dossier show <ID>` prints one in full), then make the edit again.
- Never approve, reject, retire or remove an entry: a person does that.
- Never confirm or refute an assumption, never accept or decline a proposal: a person does that.
- When your person says a name or term is confidential, record it at once: `dossier confidential add "<term>"`.
- Never write a confidential term into the team dossier; propose team knowledge with `dossier promote <ID>`.
- Never write a password, key or token into an entry; write where it is kept. The dossier refuses text that looks like one.
- Exit code `1`: a rule refused the change; read the message and do not repeat the command unchanged.
- Exit code `3`: busy; run the same command again.
<!-- dossier:end -->
