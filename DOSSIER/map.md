<!-- dossier-format: 1 -->
# Map of DOSSIER

## src

- purpose: The library: one module per concern, all declared in lib.rs; entry files change only through tx.rs.
- start: src/lib.rs
- history: 2026-11-25T07:34:45+01:00 reyna via reyna - claude: set

## src/cmd

- purpose: One file per command; cli.rs registers each command and dispatches to its run function.
- start: src/cli.rs
- history: 2026-11-25T07:34:45+01:00 reyna via reyna - claude: set

## src/entry

- purpose: Entry files: parsing, the rules per kind, and writing a new entry.
- start: src/entry/mod.rs
- history: 2026-11-25T07:34:45+01:00 reyna via reyna - claude: set

## src/remote

- purpose: Remote mode from a PC: ssh, the pointer file, forwarding, version check and scp transfer.
- start: src/remote/forward.rs
- history: 2026-11-25T07:34:45+01:00 reyna via reyna - claude: set

## src/agents

- purpose: The rules text for LLMs that agents install writes into AGENTS.md.
- start: src/agents/rules.md
- history: 2026-11-25T07:34:45+01:00 reyna via reyna - claude: set

## tests

- purpose: Integration tests, one file per command; they run the built program in temporary folders.
- start: tests/common/mod.rs
- history: 2026-11-25T07:34:45+01:00 reyna via reyna - claude: set

## docs

- purpose: What to build (requirements), the file format, and the design with every signature.
- start: docs/requirements.md
- history: 2026-11-25T07:34:45+01:00 reyna via reyna - claude: set

## .github/workflows

- purpose: CI on Windows and Linux, and the release that a v* tag builds and publishes.
- start: .github/workflows/release.yml
- history: 2026-11-25T07:34:45+01:00 reyna via reyna - claude: set

