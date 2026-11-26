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

- What was decided, assumed, found and learned is in the `DOSSIER` folder. Never edit files there directly; use the `dossier` commands.
- Your first edit of a file may be refused with a list of entries in force for it. That is not an error: read them, then make the same edit again.
- Decisions in force bind you: if your task contradicts one, stop and say so; never work around it.
- Workers: run `dossier help worker`.
- Planner: run `dossier help planner`.
<!-- dossier:end -->
