# dossier

`dossier` is a command-line program that keeps what a small engineering team learns and decides
while doing its projects: decisions, assumptions, findings, lessons, how-tos, scripts, and the
files and simulation runs behind each result. People and their LLMs write to it and read from it,
so that neither has to rely on memory.

**Status: planning.** Nothing is built yet. The requirements and the work plan are complete for
the first phase.

- [`docs/requirements.md`](docs/requirements.md): what the tool does, the decisions behind it,
  and the five phases it is built in.
- [`TODO.md`](TODO.md): the work plan.

## What it needs

- [`todo`](https://github.com/reynanTandoyo/todo) 0.8 or newer. `todo` manages the tasks of a
  project in `TODO.md`; `dossier` manages the knowledge next to it and reads the tasks through
  `todo --json`.
- `ssh`, for using a dossier on a server from a PC.
