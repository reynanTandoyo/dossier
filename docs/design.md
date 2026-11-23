# dossier — design for v0.1

This document fixes how the code of v0.1 is laid out, so that each task fills in one file and
needs to know nothing about the others beyond what is written here. It follows
[requirements.md](requirements.md) (what) and [format.md](format.md) (the files).

## 1. Principles

1. **One file, one task.** Every library module and every command lives in its own file and is
   written by its own task. The skeleton tasks create every file as a stub that compiles, so a
   later task never has to register anything in a shared file.
2. **A command owns its arguments.** Each `src/cmd/<name>.rs` defines `pub struct Args`
   (`#[derive(clap::Args)]`) and `pub fn run(args: &Args, g: &Global) -> Result<Output>`.
   `src/cli.rs` only lists the commands and never changes after the skeleton.
3. **All writes go through `tx`.** A command never writes an entry file itself; `tx::write`
   takes the lock, saves the snapshot, replaces the file, adds the journal line, or prints the
   change with `--dry-run`.
4. **Who and when, always.** Every change adds a history line to the entry (format.md §6) and a
   journal line, with the person, the LLM and the time from `time::now()`.
5. **No panics, no `println!`.** Every failure is an `Error`; output goes through `output::emit`.
6. **Tests.** A library module has its tests at the end of its own file
   (`#[cfg(test)] mod tests`), run with `cargo test --lib <module>::`. A command has
   `tests/cmd_<name>.rs`, which runs the built program in a temporary folder using the helpers
   of `tests/common/mod.rs`, run with `cargo test --test cmd_<name>`. Unit tests run in parallel
   threads of one process: a unit test that sets or removes an environment variable, or changes
   the current folder, holds `crate::ENV_LOCK` (a test-only `Mutex<()>` in `lib.rs`) for its
   whole run.

## 2. Files

```
Cargo.toml  .cargo/config.toml
src/main.rs            calls dossier::cli::main_exit()
src/lib.rs             `pub mod` for every module below
src/cli.rs             Cli, Global, Command enum, dispatch, main_exit
src/error.rs  output.rs  time.rs  hash.rs  paths.rs  find.rs  config.rs  names.rs
src/lock.rs  atomic.rs  ctx.rs  term.rs
src/entry/mod.rs  entry/edit.rs  entry/new.rs  entry/rules.rs
src/store.rs  ids.rs  journal.rs  snapshot.rs  tx.rs  approval.rs  create.rs
src/filter.rs  select.rs  brief_task.rs  todo_bridge.rs  tar.rs
src/remote/mod.rs  remote/pointer.rs  remote/register.rs  remote/ssh.rs  remote/forward.rs
src/agents/mod.rs  agents/rules.md
src/cmd/mod.rs         `pub mod` for every command
src/cmd/<name>.rs      init where_cmd people add decide found learn assume set retitle link edit
                       show list search history diff restore log review_task drafts approve
                       reject supersede retire remove reopen brief status check agents help
                       pack unpack connect projects todo
tests/common/mod.rs    helpers
tests/cmd_<name>.rs    one per command
```

## 3. Library modules

Signatures are exact. `Result<T>` is `crate::error::Result<T>`.

### error.rs

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Code { Usage, NotImplemented, Refused, NotFound, Exists, NeedsPerson, NeedsTerminal,
    UnknownName, MissingField, BadValue, Busy, Changed, NoDossier, NetworkDrive, Parse, Config,
    Io, TooNew, TodoMissing, Ssh }
#[derive(Debug, Clone)]
pub struct Error { pub code: Code, pub message: String }
pub type Result<T> = std::result::Result<T, Error>;
impl Error {
    pub fn new(code: Code, message: impl Into<String>) -> Error;
    pub fn exit(&self) -> u8;          // see table
    pub fn name(&self) -> &'static str; // SCREAMING_SNAKE_CASE of the code: Code::NoDossier -> "NO_DOSSIER"
}
impl From<std::io::Error> for Error;   // Code::Io with the error's text
```

| Exit | Codes |
|---|---|
| 1 | Refused, NotFound, Exists, NeedsPerson, NeedsTerminal, UnknownName, MissingField, BadValue |
| 2 | Usage, NotImplemented |
| 3 | Busy, Changed |
| 4 | NoDossier, NetworkDrive, Parse, Config, Io, TooNew |
| 5 | TodoMissing, Ssh |

### output.rs

```rust
pub struct Output { pub text: String, pub data: serde_json::Value, pub warnings: Vec<String> }
impl Output {
    pub fn text(text: impl Into<String>) -> Output;              // data = Null
    pub fn with(text: impl Into<String>, data: serde_json::Value) -> Output;
    pub fn warn(self, warning: impl Into<String>) -> Output;
}
/// Writes the result and returns the exit code. Never panics on a closed pipe.
pub fn emit(command: &str, json: bool, result: &Result<Output>) -> u8;
```

Without `--json`: `text` to standard output (a final line break is added if missing), each
warning as `warning: <text>` and an error as `error: <message>` to standard error. With `--json`:
one line on standard output,
`{"schema":"dossier.<command>.v1","ok":true,"data":…,"warnings":[…]}` or
`{"schema":"dossier.<command>.v1","ok":false,"error":{"code":"<name>","message":"…"},"warnings":[]}`.
Write with `std::io::Write` and ignore a `BrokenPipe` error.

### time.rs

```rust
pub fn now() -> String;                 // format.md §7; DOSSIER_NOW overrides when it parses
pub fn valid(stamp: &str) -> bool;
pub fn date(stamp: &str) -> &str;       // the first 10 characters
pub fn days_between(earlier: &str, later: &str) -> i64;
```

### hash.rs

```rust
pub fn sha256_hex(bytes: &[u8]) -> String;
```

### paths.rs

```rust
pub fn slug(title: &str) -> String;                       // format.md §4
pub fn windows_safe(name: &str) -> bool;                  // R-XOS-4, R-XOS-5
/// A typed path (with / or \, relative to cwd or absolute) as a path relative to root, with /.
pub fn stored(root: &Path, cwd: &Path, typed: &str) -> Result<String>;  // outside root: BadValue
pub fn shown(stored: &str) -> String;                     // with the separator of this system
```

### find.rs

```rust
/// The DOSSIER folder: `explicit` if given, else the first `DOSSIER` folder found walking up
/// from `start`. Missing: Code::NoDossier, message names `dossier init`.
pub fn find(start: &Path, explicit: Option<&Path>) -> Result<PathBuf>;
```

### config.rs

```rust
pub struct Person { pub name: String, pub llms: Vec<String> }
pub struct Tags { pub machine: Vec<String>, pub analysis: Vec<String>, pub software: Vec<String> }
pub struct Config { pub format: u32, pub code: String, pub tags: Tags, pub people: Vec<Person>,
    pub brief_lines: usize, pub status_lines: usize, pub draft_days: i64,
    pub lock_wait_seconds: u64, pub ssh_timeout_seconds: u64 }
pub fn load(dossier: &Path) -> Result<Config>;      // format.md §10; unknown key: Code::Config
pub fn default_text(code: &str) -> String;          // the text `init` writes
pub fn add_person(dossier: &Path, person: &str, llms: &[String]) -> Result<()>; // keeps other lines
```

### names.rs

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct Who { pub name: String, pub person: String, pub is_llm: bool }
/// Name from `flag`, else DOSSIER_AGENT, else TODO_AGENT. None: Code::Usage.
/// Not a person and not an LLM in cfg: Code::UnknownName, message lists the known names.
pub fn resolve(cfg: &Config, flag: Option<&str>) -> Result<Who>;
pub fn is_person(cfg: &Config, name: &str) -> bool;
/// "anna" or "anna via anna-claude".
pub fn label(who: &Who) -> String;
```

### lock.rs

```rust
pub struct Guard { /* holds the open file */ }
/// Locks `<state>/write.lock` with `File::try_lock`, retrying every 50 ms for `wait_seconds`,
/// then Code::Busy naming the holder from holder.json. Writes holder.json; Drop removes it.
pub fn acquire(state: &Path, wait_seconds: u64, name: &str, command: &str) -> Result<Guard>;
/// Windows: UNC path or a drive of type DRIVE_REMOTE. Linux: the mount of the path in
/// /proc/mounts has type cifs, smb3, smbfs, nfs or nfs4.
pub fn is_network(path: &Path) -> bool;
```

### atomic.rs

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct Seen { pub len: u64, pub hash: String }
pub fn seen(path: &Path) -> Result<Option<Seen>>;   // None when the file does not exist
/// Writes `<name>.tmp-<pid>` next to the file, flushes, renames. With `expected`, first checks
/// that the file is still that (None = must not exist); otherwise Code::Changed.
pub fn replace(path: &Path, bytes: &[u8], expected: Option<&Option<Seen>>) -> Result<()>;
```

### term.rs

```rust
pub fn is_terminal() -> bool;                        // stdin and stdout are terminals
pub fn ask(prompt: &str) -> Result<String>;          // one line; not a terminal: Code::NeedsTerminal
```

`DOSSIER_ANSWERS` (answers separated by `|`) replaces the terminal in tests: when it is set,
`is_terminal()` is true and `ask` returns the next answer.

### ctx.rs

```rust
pub struct Ctx { pub root: PathBuf, pub dossier: PathBuf, pub state: PathBuf, pub cfg: Config,
    pub who: Option<Who>, pub json: bool, pub dry_run: bool, pub command: String }
impl Ctx {
    /// Finds the dossier from the current folder (or g.dossier), loads the settings, and
    /// resolves the name if one is given (no name is not an error here).
    pub fn load(g: &crate::cli::Global, command: &str) -> Result<Ctx>;
    pub fn writer(&self) -> Result<&Who>;   // Code::Usage "this command needs --agent" when None
}
```

`root` is the parent of `dossier`; `state` is `dossier/.state`.

### entry/mod.rs

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind { Decision, Assumption, Finding, Lesson }
impl Kind { pub fn all() -> [Kind; 4]; pub fn name(self) -> &'static str;
    pub fn prefix(self) -> &'static str; pub fn folder(self) -> &'static str;
    pub fn parse(text: &str) -> Option<Kind>;        // by name or prefix, any case
    pub fn of_id(id: &str) -> Option<Kind>; }
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status { Draft, Approved, Rejected, Superseded, Retired }
impl Status { pub fn name(self) -> &'static str; pub fn parse(text: &str) -> Option<Status>; }
#[derive(Debug, Clone)]
pub struct Entry { pub lines: Vec<String> }          // every line with its own ending
impl Entry {
    pub fn parse(text: &str) -> Result<Entry>;       // Code::Parse with the line number
    pub fn text(&self) -> String;                    // the lines joined: equals the input
    pub fn id(&self) -> String;
    pub fn title(&self) -> String;
    pub fn field(&self, key: &str) -> Option<String>;        // first line with that key
    pub fn fields(&self) -> Vec<(String, String)>;           // in file order, history included
    pub fn list(&self, key: &str) -> Vec<String>;            // a comma-separated field, trimmed
    pub fn section(&self, name: &str) -> Option<String>;     // body, format.md §3
    pub fn sections(&self) -> Vec<(String, String)>;
    pub fn kind(&self) -> Option<Kind>;
    pub fn status(&self) -> Option<Status>;
    pub fn history(&self) -> Vec<String>;                    // the values of the history lines
}
```

### entry/edit.rs

```rust
impl Entry {
    pub fn set_field(&mut self, key: &str, value: &str);   // replaces the line, or adds it
                                   // before the first history line (else at the block's end)
    pub fn remove_field(&mut self, key: &str);
    pub fn set_title(&mut self, title: &str);
    pub fn set_section(&mut self, name: &str, body: &str); // replaces the body, or adds the
                                                           // section at the end of the file
    pub fn add_history(&mut self, time: &str, who: &Who, event: &str, detail: &str);
}
```

New lines take their line ending as format.md §2 says.

### entry/new.rs

```rust
pub struct Draft { pub kind: Kind, pub id: String, pub title: String,
    pub fields: Vec<(String, String)>, pub sections: Vec<(String, String)> }
/// The text of a new entry file exactly as format.md §1, fields in the order of §5 (unknown
/// keys after the known ones, in the order given), sections in the order of §8.
pub fn render(draft: &Draft) -> String;
```

### entry/rules.rs

```rust
pub struct Problem { pub what: String }
pub fn required_sections(kind: Kind) -> &'static [&'static str];
pub fn section_order(kind: Kind) -> &'static [&'static str];
/// What is missing for approval: required sections, `confidence` for a lesson with one of the
/// four values, a valid `state` for an assumption, a valid `outcome` if present.
pub fn problems(entry: &Entry) -> Vec<Problem>;
```

### store.rs

```rust
pub struct Item { pub path: PathBuf, pub raw: Vec<u8>, pub entry: Entry }
/// Every entry file of the dossier, sorted by ID. Files that fail to parse become warnings
/// `<path>:<line>: <message>`.
pub fn all(dossier: &Path) -> (Vec<Item>, Vec<String>);
/// By ID, with or without a place prefix (`P-017:DEC-012`), any case. Code::NotFound.
pub fn get(dossier: &Path, id: &str) -> Result<Item>;
pub fn path_for(dossier: &Path, kind: Kind, id: &str, title: &str) -> PathBuf;
```

### ids.rs

```rust
/// The next ID of a kind: 1 + max(the number in ids.json, the highest number among the
/// entries on disk). Writes ids.json. Call only while holding the lock.
pub fn next(dossier: &Path, state: &Path, kind: Kind) -> Result<String>;
```

### journal.rs

```rust
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct Record { pub time: String, pub name: String, pub person: String, pub llm: bool,
    pub command: String, pub id: String, pub event: String, pub detail: String }
pub fn append(state: &Path, record: &Record) -> Result<()>;
pub fn read(state: &Path) -> Result<Vec<Record>>;     // oldest first; bad lines are skipped
```

### snapshot.rs

```rust
pub struct Snap { pub n: u32, pub time: String, pub name: String, pub command: String }
pub fn save(state: &Path, id: &str, bytes: &[u8], time: &str, name: &str, command: &str)
    -> Result<u32>;
pub fn list(state: &Path, id: &str) -> Result<Vec<Snap>>;
pub fn load(state: &Path, id: &str, n: u32) -> Result<Vec<u8>>;
```

### approval.rs

```rust
pub enum Mark { Unchanged, Changed, NotRecorded }
pub fn record(state: &Path, id: &str, bytes: &[u8]) -> Result<()>;
pub fn forget(state: &Path, id: &str) -> Result<()>;
pub fn mark(state: &Path, id: &str, bytes: &[u8]) -> Mark;
```

The checksum covers the file without its `- history:` lines and without the `status`,
`approved`, `valid-from` and `valid-until` lines.

### tx.rs

```rust
pub struct Tx<'a> { /* ctx, pending changes */ }
pub struct Change { pub id: String, pub event: String, pub detail: String }
impl<'a> Tx<'a> {
    pub fn ctx(&self) -> &Ctx;
    pub fn time(&self) -> &str;                      // one time for the whole command
    /// Queue writing `text` to `path`. `old` is the item as read (None for a new file).
    pub fn put(&mut self, path: PathBuf, old: Option<&Item>, text: String, change: Change);
    pub fn delete(&mut self, old: &Item, change: Change);
    pub fn rename(&mut self, old: &Item, new_path: PathBuf, text: String, change: Change);
}
/// Refuses a network drive, takes the lock, runs `f`, then for each queued change: saves a
/// snapshot of the old file, replaces the file with atomic::replace (expecting the old bytes),
/// appends the journal record. With ctx.dry_run nothing is written and the Output gets one
/// warning per change: `dry run: would write <path>`. An error in `f` writes nothing.
pub fn write<F>(ctx: &Ctx, f: F) -> Result<Output>
    where F: FnOnce(&mut Tx) -> Result<Output>;
```

### create.rs

```rust
pub struct New { pub kind: Kind, pub title: String, pub fields: Vec<(String, String)>,
    pub sections: Vec<(String, String)>, pub said_by: Option<String>, pub draft: bool }
/// Creates one entry inside a transaction and returns its ID and file text.
/// Author and status: a person writing: author = person, approved unless `draft`.
/// An LLM with `said_by` naming a person of the settings: author = that person, via = the LLM,
/// approved unless `draft`. An LLM without `said_by`: author = its person, via = the LLM;
/// approved for decision (see below), assumption and finding; draft for a lesson.
/// An LLM writing a decision without `said_by`: Code::NeedsPerson.
/// Approved entries get `approved`, `valid-from`, two history lines (created, approved) and,
/// for a lesson, approval::record. Refuses unknown tag values is not part of v0.1.
pub fn create(tx: &mut Tx, new: &New) -> Result<(String, String)>;
```

### filter.rs

```rust
#[derive(clap::Args, Debug, Default, Clone)]
pub struct Filter { /* --kind --status --machine --analysis --software --tag --author --task
                       --since <date> --where key=value (repeatable) --drafts --all */ }
impl Filter { pub fn matches(&self, item: &Item) -> bool; }
```

Without `--status`, `--drafts` and `--all`, only approved entries match. `--drafts` adds drafts,
`--all` matches every status. List values match when any element equals, ignoring case.

### select.rs

```rust
pub fn words(text: &str) -> Vec<String>;        // lower case, split at non-alphanumerics,
                                                // words under 3 characters dropped
pub struct Query { pub tags: Vec<String>, pub words: Vec<String> }
/// 5 points per query word in the title, 3 per query tag among the entry's tag fields,
/// 1 per query word in the sections. 0 = no match.
pub fn score(item: &Item, query: &Query) -> u32;
/// Items with score > 0, highest first, then newest `created` first, then ID.
pub fn best<'a>(items: &'a [Item], query: &Query, limit: usize) -> Vec<&'a Item>;
```

### todo_bridge.rs

```rust
pub fn available() -> bool;                               // `todo --version` runs
/// Runs `todo <args> --json` in `root` and returns the `data` of the envelope.
/// Program missing: Code::TodoMissing. `ok:false`: Code::Refused with todo's message.
pub fn json(root: &Path, args: &[&str]) -> Result<serde_json::Value>;
```

The program name is `todo`, or the value of `DOSSIER_TODO` (tests point it at a stand-in).

### brief_task.rs

```rust
/// The lines of `brief --task`: the task's title and goal lines from todo, the entries whose
/// `task` field names it, the decisions whose task is one of its milestones, then
/// select::best over the remaining approved entries with the task's machine, analysis and
/// software fields as tags and its title as words.
pub fn lines(ctx: &Ctx, task: &str, limit: usize) -> Result<Vec<String>>;
```

### tar.rs

```rust
/// A minimal ustar writer and reader for regular files and folders (no compression).
pub fn pack(folder: &Path, out: &Path) -> Result<u32>;     // number of files
pub fn unpack(archive: &Path, into: &Path) -> Result<u32>; // `into` must be empty or missing
```

### remote

```rust
// pointer.rs
pub struct Pointer { pub server: String, pub folder: String, pub code: String }
pub fn find(start: &Path) -> Option<(PathBuf, Pointer)>;   // walking up; the folder it is in
pub fn write(folder: &Path, pointer: &Pointer) -> Result<()>;
// register.rs
pub struct Project { pub code: String, pub server: String, pub folder: String,
    pub local: Option<String> }
pub fn file() -> Result<PathBuf>;       // format.md §12; DOSSIER_REGISTER overrides
pub fn load() -> Result<Vec<Project>>;
pub fn put(project: &Project) -> Result<()>;      // replaces the one with the same code
pub fn remove(code: &str) -> Result<bool>;
// ssh.rs
/// One shell word for a POSIX shell: wrapped in single quotes, each ' as '\''.
pub fn quote(arg: &str) -> String;
/// The argument list for the ssh program: -o BatchMode=yes -o ConnectTimeout=<n>,
/// -t when `terminal`, the server, then one string:
/// `cd <quoted folder> && DOSSIER_AGENT=<quoted> <quoted program> <quoted args…>`.
pub fn argv(server: &str, folder: &str, agent: Option<&str>, program: &str, args: &[String],
    terminal: bool, timeout: u64) -> Vec<String>;
/// Runs it with inherited input and output and returns the remote exit code.
/// ssh itself failing (exit 255, or the program missing): Code::Ssh.
pub fn run(argv: &[String]) -> Result<u8>;
// forward.rs
/// Called first by main_exit. Some(exit code) when the command was run on a server:
/// `--project <code>` is given, or the current folder has a pointer and the command is not
/// `connect`, `projects` or `help`. None: run locally.
pub fn maybe(args: &[String]) -> Option<u8>;
```

The ssh program is `ssh`, or the value of `DOSSIER_SSH` (tests point it at a stand-in).

### agents

`agents/rules.md` is the rules text. `agents/mod.rs`:

```rust
pub const RULES: &str = include_str!("rules.md");
pub const BEGIN: &str = "<!-- dossier:begin -->";
pub const END: &str = "<!-- dossier:end -->";
pub fn block() -> String;                                  // BEGIN, rules, END
/// The text of AGENTS.md with the block added (after todo's block if there is one, else at the
/// end) or replaced; every other line unchanged.
pub fn installed(existing: &str) -> String;
pub fn current(existing: &str) -> bool;
```

## 4. cli.rs

```rust
#[derive(clap::Args, Debug, Clone, Default)]
pub struct Global {
    #[arg(long, global = true)] pub dossier: Option<PathBuf>,
    #[arg(long, global = true)] pub agent: Option<String>,
    #[arg(long, global = true)] pub json: bool,
    #[arg(long, global = true)] pub dry_run: bool,
    #[arg(long, global = true)] pub allow_network_drive: bool,
    #[arg(long, global = true)] pub project: Option<String>,
}
```

`Command` has one variant per file of `src/cmd/`, named after the command
(`ReviewTask(cmd::review_task::Args)` is `review-task`, `Where(cmd::where_cmd::Args)` is
`where`). clap's own `help` subcommand is switched off; `help` is `cmd::help`.
`main_exit() -> u8`: collect the arguments, try `remote::forward::maybe`, else parse, call the
command's `run`, and return `output::emit(name, g.json, &result)`. A parse error prints clap's
message and returns 2.

## 5. Commands of v0.1

Every command also takes the global options. `ID` accepts any case.

| Command | Arguments | Result |
|---|---|---|
| `init` | `--code <code>` | creates `DOSSIER/` (format.md §9, §10); refuses an existing one (Exists) |
| `where` | — | project folder, dossier, state folder, project code |
| `people` | `add <person> [--llm <name>]…` | adds or extends a person in dossier.toml |
| `add` | `<kind> --title <t> [--field key=value]… [--section "Name=text"]… [--from <file>] [--stdin] [--task <task>] [--said-by <person>] [--draft]` | creates an entry; `--from`/`--stdin` give a whole entry body (sections) in Markdown |
| `decide` | `<decision> --reason <text> [--rejected <text>] [--title <t>] [--task <task>] [--said-by <person>] [--replaces <ID>] [--besides] [tag options]` | a decision; title defaults to the first 80 characters of the decision |
| `found` | `<observation> [--evidence <text>] [--dead-end] [--later] [--title <t>] [--task <task>] [tag options]` | a finding; `--later` makes it a draft with tag `inbox` |
| `learn` | `<statement> --valid-for <text> --evidence <text> --confidence <level> [--title <t>] [--task <task>] [--said-by <person>] [--update <ID>] [tag options]` | a lesson |
| `assume` | `<assumption> --reason <text> [--affects <text>] [--title <t>] [--task <task>] [tag options]` | an assumption with `state: open` |
| `set` | `<ID> [--field key=value]… [--section "Name=text"]…` | changes fields and sections |
| `retitle` | `<ID> <title>` | changes the title and renames the file |
| `link` | `<ID> <ID>` | adds each to the other's `related` |
| `edit` | `<ID>` | editor under the lock |
| `show` | `<ID>` | the file text with marks |
| `list` | filter options, `--wide` | one line per entry, then a count |
| `search` | `<words>… [--full]`, filter options | matches |
| `history` | `<ID or task>` | events in time order, and the snapshots of an entry |
| `diff` | `<ID> [<n>]` | changes between snapshot n (default: the latest) and the file |
| `restore` | `<ID> <n>` | brings a snapshot back; a person at a terminal |
| `log` | `[--kind] [--person] [--event] [--since] [--until]` | who did what when |
| `review-task` | `<task>` | material and drafting rules |
| `drafts` | `[--mine]` | drafts, oldest first |
| `approve` | `<ID>…` | approves drafts; a person at a terminal |
| `reject` | `<ID> [--reason <text>]` | rejects a draft |
| `supersede` | `<old ID> --by <new ID>` | replaces an entry |
| `retire` | `<ID> --reason <text>` | ends an entry without a replacement |
| `remove` | `<ID> --yes` | deletes an entry; a person at a terminal |
| `reopen` | `<ID>` | approved or rejected back to draft; retired back to approved (a person) |
| `brief` | `[--task <task>]` | context for a session or a task |
| `status` | `[--since <date or last>] [--milestone <task>]` | the state of the project |
| `check` | — | problems, one line each |
| `agents` | `install [--yes]`, `check` | the rules block in AGENTS.md |
| `help` | `[agents]` | overview, or the rules text |
| `pack` | `--to <file>` | archive of the dossier |
| `unpack` | `<file> --into <folder>` | restore an archive |
| `connect` | `<user@server>:<folder> [--code <code>]`, `--check` | pointer file and register entry |
| `projects` | `[remove <code>]` | the register |
| `todo` | `<arguments>…` | runs `todo` in the project folder |

Tag options: `--machine <list> --analysis <list> --software <list> --tags <list>`.

First output line of a write: `<ID>: <event>`, for example `DEC-012: added (approved)` or
`LES-004: added (draft, waiting for approval)`.

Marks in `show`, `list`, `search`, `brief`: `[draft]`, `[rejected]`, `[superseded by <ID>]`,
`[retired]`, `[changed after approval]`.

## 6. Test helpers: tests/common/mod.rs

```rust
pub struct Project { pub dir: tempfile::TempDir }
impl Project {
    /// A temporary project folder with `dossier init --code P-001` run in it, the people
    /// anna (LLM anna-claude) and ben (LLM ben-codex) added, DOSSIER_NOW fixed.
    pub fn new() -> Project;
    pub fn root(&self) -> &std::path::Path;
    /// The program run in the project folder as `agent` ("" = no name).
    pub fn run(&self, agent: &str, args: &[&str]) -> assert_cmd::assert::Assert;
    pub fn run_at(&self, agent: &str, now: &str, args: &[&str]) -> assert_cmd::assert::Assert;
    pub fn stdout(&self, agent: &str, args: &[&str]) -> String;   // asserts success
    pub fn read(&self, relative: &str) -> String;
    pub fn write(&self, relative: &str, text: &str);
    pub fn entry_file(&self, id: &str) -> std::path::PathBuf;     // finds DOSSIER/*/<id>-*.md
    /// The prepared command, not yet run, so that a test can add variables such as
    /// DOSSIER_ANSWERS or EDITOR. run_at is command(agent, now).args(args).assert().
    pub fn command(&self, agent: &str, now: &str) -> assert_cmd::Command;
    /// Where DOSSIER_TODO and DOSSIER_SSH point: <project>/.test/<program>, with .cmd on Windows.
    pub fn stand_in(&self, program: &str) -> std::path::PathBuf;
    /// Writes a stand-in program there (the Windows or the Unix text, executable) and returns it.
    pub fn write_stand_in(&self, program: &str, windows: &str, unix: &str) -> std::path::PathBuf;
}
```

Tests never touch a real project, a real server or the user's register (`DOSSIER_REGISTER` is
`<project>/.test/projects.toml`; `DOSSIER_SSH` and `DOSSIER_TODO` are the stand-in paths, where
nothing exists until a test writes a stand-in).

---

# Phase 2 (v0.2)

Same principles as §1: one file, one task; a skeleton task first creates every new file as a stub
that compiles; every write goes through `tx`; every change records who and when (D29). The
formats are in format.md §14 to §19.

## 7. Changes to existing modules

```rust
// entry/mod.rs
pub enum Kind { Decision, Assumption, Finding, Lesson, Usage }      // all() -> [Kind; 5]
                                                                    // usage: prefix USE, folder usage
pub enum Status { Draft, Approved, Rejected, Superseded, Retired, Proposed, Declined }

// entry/rules.rs: usage needs the fields software, version, task; sections in order
// Parameters, Outputs, Deviations, Notes (none required). `Pictures` is never required and
// always comes after a kind's own sections.

// config.rs: Config gains
pub deciders: Vec<String>, pub roles: Vec<String>, pub code_project: bool,

// names.rs
/// True when cfg.deciders is empty or names `person` (ignoring case).
pub fn may_decide(cfg: &Config, person: &str) -> bool;
/// For commands that run before or without people in the settings (init, people add): the name
/// from `flag`, DOSSIER_AGENT or TODO_AGENT, else the operating-system user (USERNAME, USER),
/// else "unknown"; person = name; is_llm = false. Never an error.
pub fn unchecked(flag: Option<&str>) -> Who;

// create.rs: for a decision, a draft is a proposal (R-LIFE-1): a decision that is not approved
// gets status `proposed` instead of `draft`. That is a decision written with `draft: true` (this
// is how `propose` and `add decision --draft` write), and a decision whose author may not decide
// (names::may_decide). It has no `approved` and `valid-from` fields, the history lines `created`
// and `proposed`, and Change detail `proposed`. An LLM's draft decision without `said_by` is
// such a proposal too (author: its person, via: the LLM; R-WRITE-12); only an LLM's decision
// that is not a draft still needs `said_by` (Code::NeedsPerson). A usage record is approved at once like a
// finding.

// tx.rs
/// Queue writing `bytes` to `path` (any file inside the dossier). `old` is the file as read (None
/// for a new file). The old bytes are snapshotted under change.id when the path ends in `.md`.
pub fn put_raw(&mut self, path: PathBuf, old: Option<Vec<u8>>, bytes: Vec<u8>, change: Change);
/// Queue removing a file that is still `old`.
pub fn delete_raw(&mut self, path: PathBuf, old: Vec<u8>, change: Change);

// ids.rs
/// The next number for a prefix that is not an entry kind (RCV, DLV): 1 + max(ids.json, on_disk).
/// Writes ids.json; call under the lock. Returns e.g. "RCV-001".
pub fn next_prefix(state: &Path, prefix: &str, on_disk: u64) -> Result<String>;

// filter.rs: a new option --on <date>. With it, an entry matches when its valid-from date is on
// or before the date and its valid-until is empty or after it, whatever its status; the other
// options still apply. Without --status, --drafts and --all, `proposed` behaves like `draft`.

// remote/forward.rs: `hooks`, `guard` and `skill` run locally like `connect`; `accept`,
// `decline`, `confirm`, `refute`, `cleared` and `value` ask ssh for a terminal like `approve`.

// remote/register.rs: Project gains `pub added: String` (the time it was put, format.md §7),
// written as `added = "..."`; a register entry without it reads as "".

// agents/mod.rs
/// The Claude Code skill: a front matter (name: dossier, description: one line) and RULES.
pub fn skill() -> String;
```

## 8. New library modules

### blockfile.rs

```rust
/// A block file (format.md §17), kept as its lines so that every byte not changed stays.
#[derive(Debug, Clone)]
pub struct BlockFile { pub lines: Vec<String> }
impl BlockFile {
    pub fn parse(text: &str) -> Result<BlockFile>;     // TooNew above format 1; Parse: no heading
    pub fn empty(title: &str) -> BlockFile;            // format line, "# <title>", one blank line
    pub fn text(&self) -> String;                      // equals the parsed text
    pub fn keys(&self) -> Vec<String>;                 // `## ` keys in file order
    pub fn field(&self, key: &str, name: &str) -> Option<String>;
    pub fn fields(&self, key: &str) -> Vec<(String, String)>;   // history included
    pub fn history(&self, key: &str) -> Vec<String>;
    /// Appends `## <key>`, a blank line, the fields (history pairs last), a blank line.
    pub fn add_block(&mut self, key: &str, fields: &[(String, String)]);
    /// Replaces the field or adds it before the block's first history line; an empty value
    /// removes it; `history` changes nothing; a missing block changes nothing.
    pub fn set_field(&mut self, key: &str, name: &str, value: &str);
    pub fn add_history(&mut self, key: &str, time: &str, who: &Who, event: &str, detail: &str);
}
```

New lines follow format.md §2 (LF, or CRLF after a CRLF line).

### filesum.rs

```rust
pub const SHORT_ABOVE: u64 = 1 << 30;
/// "sha256:<hex>", or "short-sha256:<hex>" above SHORT_ABOVE (format.md §18). Reads in chunks.
pub fn checksum(path: &Path) -> Result<String>;
pub struct Stamp { pub size: u64, pub modified: String }   // modified: format.md §7, local time
pub fn stamp(path: &Path) -> Result<Stamp>;
pub fn count_files(folder: &Path) -> Result<u64>;           // regular files, recursive
```

### values.rs

```rust
pub const FILE: &str = "values.md";
pub const ITEM: &str = "VALUES";
pub struct Value { pub name: String, pub value: String, pub unit: Option<String>,
    pub source: Option<String>, pub note: Option<String>, pub key: bool,
    pub valid_from: Option<String>, pub proposed: Option<String> }
pub fn valid_name(name: &str) -> bool;                       // format.md §17
/// "12.4 kg/s" -> ("12.4", Some("kg/s")): the unit is the last word. With `dimensionless` the
/// whole text is the value. One word without `dimensionless`: Code::BadValue.
pub fn split(text: &str, dimensionless: bool) -> Result<(String, Option<String>)>;
/// The file and its bytes as read; a missing file is BlockFile::empty("Values of <code>"), None.
pub fn load(dossier: &Path, code: &str) -> Result<(BlockFile, Option<Vec<u8>>)>;
pub fn all(file: &BlockFile) -> Vec<Value>;                  // file order
pub fn get(file: &BlockFile, name: &str) -> Option<Value>;
/// "<value>[ <unit>]" in force at the end of `date` (YYYY-MM-DD), from the `set` history lines.
pub fn on(file: &BlockFile, name: &str, date: &str) -> Option<String>;
```

### filelist.rs

```rust
pub const ROLES: [&str; 10] = ["description", "geometry", "mesh", "model", "setup", "script",
    "result", "report", "reference", "other"];
pub fn item(task: &str) -> String;                    // "FILES-<TASK>", task upper-cased
pub fn path(dossier: &Path, task: &str) -> PathBuf;   // dossier/files/<TASK>.md
pub fn load(dossier: &Path, task: &str) -> Result<(BlockFile, Option<Vec<u8>>)>; // empty: "Files of <TASK>"
pub fn tasks(dossier: &Path) -> Vec<String>;          // tasks that have a list, sorted
pub fn valid_role(cfg: &Config, role: &str) -> bool;  // ROLES or cfg.roles, any case
/// The fields describing `stored` now: type, size or files, modified, checksum.
pub fn describe(root: &Path, stored: &str) -> Result<Vec<(String, String)>>;
/// Marks of a block: "detached", "missing", "changed since attached", "outdated" (a made-from
/// checksum differs from that file now), "rests on a wrong assumption".
pub fn marks(root: &Path, file: &BlockFile, key: &str) -> Vec<String>;
```

### exchange.rs

```rust
pub const FILE: &str = "exchange.md";
pub const ITEM: &str = "EXCHANGE";
pub fn load(dossier: &Path, code: &str) -> Result<(BlockFile, Option<Vec<u8>>)>; // empty: "Exchange of <code>"
pub fn highest(file: &BlockFile, prefix: &str) -> u64;     // highest number of RCV or DLV keys
```

### mapfile.rs

```rust
pub const FILE: &str = "map.md";
pub const ITEM: &str = "MAP";
pub fn load(dossier: &Path, code: &str) -> Result<(BlockFile, Option<Vec<u8>>)>; // empty: "Map of <code>"
/// The map as lines `<path>  <purpose>[ (start: <start>)]`, sorted by path, detached left out.
pub fn lines(file: &BlockFile) -> Vec<String>;
```

### affects.rs

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum Item { Task(String), Entry(String), Value(String), Path(String), Text(String) }
/// The items of an Affects section (format.md §14). `values` are the known value names, `paths`
/// the attached stored paths.
pub fn parse(text: &str, values: &[String], paths: &[String]) -> Vec<Item>;
```

### picture.rs

```rust
pub const MAX_BYTES: u64 = 2 * 1024 * 1024;
pub const MAX_PER_ENTRY: usize = 10;
pub fn extension(bytes: &[u8]) -> Option<&'static str>;  // "png", "jpg" or "svg" by content
/// The file name for a new picture in `folder` (format.md §16).
pub fn file_name(folder: &Path, original: &str, ext: &str) -> String;
pub fn line(id: &str, file: &str, caption: &str) -> String;   // ![caption](../images/<ID>/<file>)
```

### hooks.rs

```rust
pub const BRIEF: &str = "dossier brief";
pub const GUARD: &str = "dossier guard";
/// .claude/settings.json with a SessionStart hook running BRIEF and a PreToolUse hook (matcher
/// "Edit|Write|MultiEdit") running GUARD, each added once; other keys kept. Empty text is "{}".
/// Code::Parse when the text is not a JSON object.
pub fn installed(existing: &str) -> Result<String>;
pub fn uninstalled(existing: &str) -> Result<String>;   // removes exactly those two entries
/// The message to block with when a Claude Code PreToolUse payload edits a file inside a
/// folder named DOSSIER; None to allow.
pub fn guard(payload: &str) -> Option<String>;
```

Written with `serde_json::to_string_pretty` and a final LF; the old file is kept as
`settings.json.bak`.

## 9. Commands of v0.2

| Command | Arguments | Result |
|---|---|---|
| `propose` | `<decision> --reason <t> [--rejected <t>] [--title <t>] [--task <task>] [tag options]` | a decision with status `proposed`; anyone, also an LLM without `--said-by` |
| `accept` | `<ID>` | a decider at a terminal: proposed to approved |
| `decline` | `<ID> --reason <t>` | a decider at a terminal: proposed to declined |
| `confirm` | `<ID> --by <who and how>` | a person: an assumption to confirmed |
| `refute` | `<ID> --by <who and how>` | a person at a terminal: an assumption to wrong; marks what rests on it and prints it |
| `cleared` | `<ID or path> --assumption <ID> --note <t>` | a person: removes one `rests-on` mark |
| `value` | `set <name> <value> [--dimensionless] [--source <t>] [--note <t>] [--key] [--said-by <person>]`, `show <name>`, `approve <name>`, `reject <name>` | key values; approve and reject: a person at a terminal |
| `values` | `[<prefix>] [--on <date>]` | the values in force |
| `attach` | `<task> <path>… --role <role> [--note <t>] [--from <path>…]`, or `<task> --moved <old> <new>` | adds or moves files of a task |
| `detach` | `<task> <path>` | sets `detached` |
| `reattach` | `<task> <path>` | records size, time and checksum again |
| `files` | `[--task <task>] [--role <role>] [<words>…] [--on <date>]` | file lists with marks |
| `which` | `<path>` | tasks, role, made from, made into, entries whose `code` points there |
| `used` | `<software> --version <v> --task <task> [--model <path>] [--param <n>=<v>]… [--output <path>]… [--howto <ID>] [--deviations <t>] [--note <t>]` | a usage record; outputs attached to the task with role `result`, made from the model |
| `runs` | `[--task <task>] [--model <path>] [--software <name>]`, or `--compare <ID> <ID>` | usage records, newest first |
| `image` | `add <ID> <file> --caption <t>`, `remove <ID> <file>` | pictures of an entry |
| `received` | `<path>… --from <who> [--date <date>] [--task <task>] [--note <t>]` | an RCV record; with `--task` the files are attached with role `reference` |
| `delivered` | `<path>… --to <who> [--date <date>] [--based-on <task or path>…] [--note <t>]` | a DLV record; warnings need a person at a terminal to confirm |
| `exchange` | `[--received] [--delivered]` | records in time order |
| `map` | (none), `set <path> <purpose> [--start <where>]`, `remove <path>` | the map of a code project |
| `hooks` | `install [--yes]`, `uninstall [--yes]` | Claude Code hooks in `.claude/settings.json` |
| `guard` | — | reads a PreToolUse payload on standard input; exit 2 with a message blocks the edit |
| `skill` | `install [--yes]` | writes `.claude/skills/dossier/SKILL.md` |

Changed commands: `init --code-project`; `decide` (a non-decider's decision becomes a proposal);
`list` and `search` (`--on`, marks `[proposed]`, `[declined]`, `[rests on <ID>]`); `brief` and
`status` (values, proposals, marks, `--on`, the map, the last exchange); `brief --task` (dead ends,
the task's files, pictures, code pointers); `init`, `people`, `agents` and `connect` (journal and
register lines, D29).

---

# Phase 3 (v0.3)

Same principles as §1 and §7: skeleton tasks first; one file, one task; every write through `tx`;
who and when for everything (D29); `dossier` never manages tasks (D38). Formats: format.md §20
to §26.

## 10. Changes to existing modules

```rust
// entry/mod.rs
pub enum Kind { Decision, Assumption, Finding, Lesson, Usage, Howto, Tool }  // all() -> [Kind; 7]
                          // howto: prefix HOW, folder howtos; tool: prefix TOOL, folder tools
// entry/rules.rs, entry/new.rs: sections and fields of format.md §22 and §23.

// approval.rs
/// Lessons, how-tos and tools: an LLM writes them as drafts, approval records their checksum.
pub fn guarded(kind: Kind) -> bool;

// config.rs: Config gains
pub kind: String /* "project" or "team" */, pub team: Option<String>, pub members: Vec<String>,
pub names: Vec<String>, pub past: bool, pub describes: Option<String>,
/// Sets a top-level array key (members, names) to `values`, keeping every other line; adds the
/// line before the first table when it is missing.
pub fn set_list(dossier: &Path, key: &str, values: &[String]) -> Result<()>;
/// Sets a top-level string or bool key (team, past, describes, kind) the same way; `value` is
/// written as given (already quoted for a string).
pub fn set_value(dossier: &Path, key: &str, value: &str) -> Result<()>;

// names.rs (format.md §21)
/// The name from flag, DOSSIER_AGENT, TODO_AGENT, USERNAME, USER; then person, `<person> -
/// <anything>` LLM, or registered LLM. Unknown: Code::UnknownName listing members and the
/// form for an LLM. No name at all: Code::Usage.
pub fn resolve(cfg: &Config, flag: Option<&str>) -> Result<Who>;
pub fn is_person(cfg: &Config, name: &str) -> bool;     // members or [[people]], any case
/// A person or a plain name of `names`: may be an author, a source or --said-by.
pub fn is_known(cfg: &Config, name: &str) -> bool;

// cli.rs Global gains
#[arg(long, global = true)] pub team: bool,
#[arg(long, global = true)] pub from_document: Option<String>,

// ctx.rs: Ctx gains
pub team: Option<PathBuf>,            // the team dossier of a project (cfg.team), if it exists
pub from_document: Option<String>,    // g.from_document, stored (paths::stored)
pub project: Option<PathBuf>,         // with --team from a project: that project's DOSSIER
// Ctx::load: when g.team is set and the current dossier is a project, the context is the team
// dossier's (its folders and settings) and `who` is resolved against the team's members only
// (R-TEAM-7: only members write there); the project's plain names are added so they can be
// named as a source. A project's cfg gains the team's members and names in memory.
// A team dossier that cannot be reached: Code::NoDossier naming the path (R-PLACE-8).
/// The context of this project's team dossier, with the same who, json and dry_run.
pub fn team_ctx(&self) -> Result<Ctx>;

// create.rs: with ctx.from_document the entry is a draft and gets imported: yes and
// source-document; approval::record for every guarded kind (not only lessons).
// When ctx.project is Some (a team write from a project), create first runs
// conf::gate(project) and conf::check(terms, rendered text) (R-WRITE-6, R-CONF-2).

// tx.rs: after the changes of a team dossier (cfg.kind == "team") are written,
// gitcommit::commit(<folder holding DOSSIER>, <DOSSIER>, "<command>: <ids>"); a failure is a
// warning on the Output, never an error.

// hooks.rs
/// .claude/settings.local.json with env.DOSSIER_AGENT set to `agent` (D34), other keys kept.
pub fn local_installed(existing: &str, agent: &str) -> Result<String>;
pub fn local_uninstalled(existing: &str) -> Result<String>;   // removes that one key
```

The approval rule of lessons in `set`, `edit`, `retitle`, `image`, `approve`, `reopen`,
`retire`, `restore`, `show`, `brief`, `status` and `check` becomes `approval::guarded(kind)`.

## 11. New library modules

```rust
// conf.rs (format.md §24)
pub const NONE: &str = "(no confidential terms)";
pub fn normalize(text: &str) -> String;     // lower case; runs of space, '-', '_' become one space
pub fn load(dossier: &Path, code: &str) -> Result<(BlockFile, Option<Vec<u8>>)>;
pub fn terms(file: &BlockFile) -> Vec<String>;       // not removed, NONE left out
pub fn stated_none(file: &BlockFile) -> bool;
/// Each term found in `text` with the 1-based line it is on.
pub fn find(terms: &[String], text: &str) -> Vec<(String, usize)>;
/// The terms of a project before writing to the team dossier: Code::Refused when there are none
/// and none was stated (R-CONF-5, the message says how to add them or state none).
pub fn gate(project: &Path, code: &str) -> Result<Vec<String>>;
/// Code::Refused naming each term and line when `text` holds one (R-CONF-2).
pub fn check(terms: &[String], text: &str) -> Result<()>;

// gitcommit.rs (D36)
/// Runs `git add -- <path>` and `git commit -m <message> -- <path>` in `repo`. Ok(false) when
/// `repo` is not a git work tree or there is nothing to commit; Code::Io when git fails.
pub fn commit(repo: &Path, path: &Path, message: &str) -> Result<bool>;

// teamfiles.rs (format.md §20)
pub fn projects(team: &Path, code: &str) -> Result<(BlockFile, Option<Vec<u8>>)>;   // projects.md
pub fn usage(team: &Path, code: &str) -> Result<(BlockFile, Option<Vec<u8>>)>;      // usage-index.md

// glossary.rs (format.md §20)
pub struct Term { pub term: String, pub meaning: String, pub symbol: Option<String>,
    pub unit: Option<String>, pub also: Vec<String> }
pub fn load(dossier: &Path, code: &str) -> Result<(BlockFile, Option<Vec<u8>>)>;
pub fn terms(file: &BlockFile) -> Vec<Term>;
/// Every spelling of the terms that `word` is a spelling of (term and `also`, any case).
pub fn spellings(terms: &[Term], word: &str) -> Vec<String>;

// imports.rs (format.md §25)
pub fn load(dossier: &Path, code: &str) -> Result<(BlockFile, Option<Vec<u8>>)>;
pub fn state(file: &BlockFile, stored: &str) -> Option<String>;

// folderscan.rs (R-IMPORT-2)
pub struct Doc { pub stored: String, pub size: u64, pub modified: String, pub kind: String }
/// The folder tree to `depth` levels as indented lines `<name>/` and `<name>  <size>`.
pub fn tree(root: &Path, depth: usize) -> Result<Vec<String>>;
/// Documents found under root (reports .pdf .docx .doc .odt .md .txt, presentations .pptx
/// .ppt, spreadsheets .xlsx .xls .csv, scripts .py .java .m .sh .bat .ps1), newest first.
pub fn documents(root: &Path) -> Result<Vec<Doc>>;
```

## 12. Commands of v0.3

| Command | Arguments | Result |
|---|---|---|
| `init` | also `--team-dossier`, `--with-team <path>`, `--past`, `--in <folder>` | a team dossier; a project linked to its team and registered in `projects.md`; a finished project |
| `team` | (none), `use <path>`, `member add <username>`, `member remove <username>`, `name add <name>`, `name remove <name>` | the team settings; removing needs a person at a terminal |
| `confidential` | `add <term>`, `remove <term>`, `list`, `none --yes`, `scan` | confidential terms; remove and none: a person at a terminal |
| `promote` | `<ID> [--as lesson\|howto]` | copies a project entry into the team dossier as a draft, after the check |
| `close` | — | a decider at a terminal sorts the approved findings, lessons and how-tos not yet promoted |
| `howto` | `--software <name> --title <t> [--section "Name=text"]… [--from <file>] [--stdin] [--review <date>] [tag options]` | a how-to (team with `--team`) |
| `tool` | `add --title <t> --type <type> [--location <x>] [--software <s>] [--language <l>] [--section …]… [--from <file>] [--stdin]`, `update <ID>` | a tool; update records the script's checksum again |
| `where-used` | `<software> [--analysis <a>] [--machine <m>]` | the team's usage index, newest first |
| `term` | `add <term> <meaning> [--symbol <s>] [--unit <u>] [--also <list>]`, or `<word>` | glossary terms (team with `--team`) |
| `glossary` | — | every term of the project and the team |
| `reviewed` | `<ID> [--next <date>]` | a person confirms an entry still holds |
| `review-folder` | `[<path>] [--depth <n>]` | the material for drafting from an old folder |
| `review-document` | `<file>` | records that a document is being reviewed; prints the import rules |
| `reviewed-document` | `<file> [--nothing]` | marks a document done |
| `import` | `status` | documents found, reviewed, nothing, drafts, approved, promoted |

Global options gain `--team` (work on the team dossier; `init` names a team path with `--with-team` instead, R-PLACE-2) and `--from-document <file>` (entries
written are imported drafts from that document).

Changed commands: `used` adds a line to the team's usage index; `search` searches the project and
the team together, marking the place, and finds a term's other spellings; `list` marks
`[due for review]`, `[imported]` and `[file changed since approval]`; `show` the same; `brief`
adds the team's lessons and how-tos that match the project's tags and the news line; `brief
--task` adds team entries, glossary terms and the last three usage-index lines; `status --since
last` includes the team dossier; `drafts --imported`; `check` reports entries due for review and
changed tool files; `hooks install` also writes `.claude/settings.local.json` (D34).

---

# Additions of 2026-11-22 (D39 to D42)

Same principles as §1: a skeleton task first creates every new file as a stub that compiles; one
file, one task; every write goes through `tx`; who and when for everything (D29); `dossier` never
manages tasks (D38). Formats: format.md §27 to §29. The three parts below are independent of each
other and of phase 5. D45 places them: credentials (§13) in the patch release 0.3.1,
investigations in phase 4 (v0.4), and the hook after shell commands in phase 6 (v0.6). The
reminder about investigations after `todo stuck` and `todo done` needs that hook, so it is built
with it.

## 13. Credentials are refused (R-CONF-7, D40)

```rust
// secret.rs (format.md §27)
pub const FILE: &str = "credential-ok.json";       // in the state folder
#[derive(Debug, Clone, PartialEq)]
pub struct Hit { pub line: usize, pub kind: &'static str, pub text: String }
                       // line: 1-based; kind: "private key", "access token" or "password";
                       // text: the whole line, trimmed (never printed, only hashed)
/// Every line of `text` that looks like it holds a credential.
pub fn find(text: &str) -> Vec<Hit>;
/// The hits of `new` that are new: their line text is no hit line of `old` and its hash is not in
/// `allowed`. `old` is the file before the change (None for a new file).
pub fn added(old: Option<&str>, new: &str, allowed: &[String]) -> Vec<Hit>;
/// The hashes of the lines a person allowed (sha256 hex of the trimmed line); a missing file is
/// an empty list.
pub fn allowed(state: &Path) -> Result<Vec<String>>;
/// Adds the hashes of these hits to the file (each once, sorted); written with atomic::replace.
pub fn allow(state: &Path, hits: &[Hit]) -> Result<()>;
/// "<file>: line 12 (password); line 14 (private key)" for a refusal. Never the text.
pub fn listed(file: &str, hits: &[Hit]) -> String;

// cli.rs Global gains
#[arg(long, global = true)] pub allow_credential_like: bool,

// tx.rs, in `write`, after `f(&mut tx)` and before the dry-run branch: for every queued Put,
// Rename and PutRaw whose path ends in `.md` and whose bytes are UTF-8, `secret::added(old text,
// new text, &secret::allowed(&ctx.state)?)`. No hits: go on. Hits and the program was not started
// with `--allow-credential-like` (read from the arguments like `--allow-network-drive`):
// Code::Refused, "this looks like a credential and was not written: <listed>; write where it is
// kept instead. A person at a terminal can add --allow-credential-like when it is a false alarm".
// <listed> names the change's ID (`FND-001`) in place of the file: an entry's file name is made
// from its title and could hold the credential itself (T-289).
// With the flag: an LLM is Code::NeedsPerson, no terminal is Code::NeedsTerminal; a person at a
// terminal: after `apply`, `secret::allow` and one journal record per file (event `allowed`,
// detail `credential-like text, line <n>[, <n>…]`). A dry run refuses the same way and allows
// nothing.

// remote/forward.rs: a command with `--allow-credential-like` among its arguments asks ssh for a
// terminal like the commands of TERMINAL_COMMANDS.

// cmd/check.rs: kind SECRET for every entry file and block file (`*.md` in the dossier, the
// state folder left out): one problem per hit of secret::find whose hash is not allowed,
// "looks like a credential (<kind>); remove it, or a person allows it with
// --allow-credential-like on the command that writes it".
```

Only `.md` files are checked: settings, pictures and the state folder are not. `unpack` restores
an archive as it is and checks nothing; `check` runs afterwards (R-BACKUP-2).

## 14. Hooks after `todo` commands (R-AGENT-5, D39)

```rust
// hooks.rs
pub const AFTER: &str = "dossier after";
/// The matcher of the PostToolUse hook: the tools that run a shell command.
const SHELL_TOOLS: &str = "Bash|PowerShell";
/// Seconds Claude Code waits for `dossier after` (a server that does not answer takes 10).
const AFTER_TIMEOUT: u64 = 30;
#[derive(Debug, Clone, PartialEq)]
pub enum TodoEvent { Claimed(String), Done(String), Stuck(String) }   // the task ID, upper case
/// The todo event of a Claude Code PostToolUse payload of a shell tool; None for anything else.
pub fn todo_event(payload: &str) -> Option<TodoEvent>;
/// {"hookSpecificOutput":{"hookEventName":"PostToolUse","additionalContext":<text>}} as one line.
pub fn context(text: &str) -> String;
// `installed` also adds a PostToolUse entry {"matcher": SHELL_TOOLS, "hooks": [{"type":
// "command", "command": AFTER, "timeout": AFTER_TIMEOUT}]}, once; `uninstalled` removes it.
```

`todo_event` reads `tool_input.command` and, for a claim made with `next`, `tool_response.stdout`.
The command text is cut into simple commands at `&&`, `||`, `;`, `|` and line breaks, and each
into words at spaces (quotes around a word removed). A simple command counts when one of its words
is `todo` (also with a path in front and `.exe` behind), or `dossier` followed by `todo`. Of the
words after it:

| Words | Event |
|---|---|
| `done` and a task ID | `Done(id)` |
| `stuck` and a task ID | `Stuck(id)` |
| `claim` and a task ID | `Claimed(id)` |
| `next` and `--claim` | `Claimed(id)`: the first task ID in `tool_response.stdout` |

A task ID is letters, `-`, digits (`T-042`), found in any case and returned in upper case. A
command with `--dry-run`, `--help` or `-h` is no event. The first event of the command text
counts. Claude Code runs the hook only after a command that succeeded.

`after` (src/cmd/after.rs) takes no arguments, reads the payload from standard input, and never
fails: every error ends as no output and exit 0. It runs locally, also in a connected folder
(`LOCAL_COMMANDS`), and gets its text by running the program itself (`std::env::current_exe`) in
the current folder with the output captured, so that remote mode works without more code:

| Event | Text |
|---|---|
| `Claimed(id)` | the output of `brief --task <id>` |
| `Done(id)` | the output of `review-task <id>`; then, when `list --kind usage --task <id> --all --json` gives no entry: `No usage record for <id>: if software was used, record it with dossier used <software> --version <v> --task <id>` |
| `Stuck(id)` | `<id> was given back. Record what was missing or did not work, so that the planner finds it: dossier found "<what was tried>" --dead-end --task <id>, or dossier assume "<what was assumed>" --reason "<why>" --task <id>` |

A run of the program that does not end with exit 0 adds nothing. When the text is not empty,
`after` prints `hooks::context(text)`; otherwise nothing.

## 15. Investigations (R-INV-1 to R-INV-10, D41, D42)

```rust
// entry/mod.rs
pub enum Kind { Decision, Assumption, Finding, Lesson, Usage, Howto, Tool, Investigation }
                 // all() -> [Kind; 8]; name investigation, prefix INV, folder investigations
// entry/rules.rs: investigation: required section Problem; order Problem, Attempts, Solution;
// `state` must be one of INVESTIGATION_STATES = ["open", "solved", "given up"].
// entry/new.rs: FIELD_ORDER gains `closed` and `given-up-reason`, in this order after `refuted`.
// approval.rs: an investigation is not guarded: it is approved at once, whoever writes it.
// ctx.rs WRITING and cmd/help.rs COMMANDS gain problem, tried, solved, gave-up.

// attempts.rs (format.md §28)
pub const SECTION: &str = "Attempts";
pub const VERDICTS: [&str; 3] = ["failed", "worked", "unclear"];
#[derive(Debug, Clone, PartialEq)]
pub struct Attempt { pub n: u32, pub verdict: String, pub text: String,
    pub did: Option<String>, pub result: Option<String>, pub run: Option<String>,
    pub task: Option<String>, pub time: String, pub who: String }
/// The attempts of an Attempts section body, in file order; other lines are skipped.
pub fn parse(body: &str) -> Vec<Attempt>;
/// One attempt as its line (format.md §28). parse(line(a)) gives a back.
pub fn line(attempt: &Attempt) -> String;
/// 1 + the highest number in the body; 1 for an empty body.
pub fn next_number(body: &str) -> u32;
/// "<n>. <verdict>: <text>[ — <result>]", for brief, status and history.
pub fn summary(attempt: &Attempt) -> String;
/// The attempts of an entry (its Attempts section); empty for another kind.
pub fn of(entry: &Entry) -> Vec<Attempt>;

// filter.rs: a new option --state <state>: the entry's `state` field equals it, case ignored.
```

| Command | Arguments | Result |
|---|---|---|
| `problem` | `<what goes wrong> [--observed <t>] [--title <t>] [--task <task>] [tag options]` | an investigation with `state: open`; first line `INV-001: added (open)`, then up to five lines `similar: <ID> [<state>] <title>` (select::best over the investigations, words of the title and the tags) |
| `tried` | `<ID> <hypothesis, or what was tried> --failed \| --worked \| --unclear [--did <t>] [--result <t>] [--run <USE-ID>] [--task <task>]` | adds one attempt; `INV-001: attempt 3 recorded (failed)`; on a closed investigation also sets `state: open`, removes `closed` and `given-up-reason`, and adds `, opened again` to the line |
| `solved` | `<ID> <what solved it> [--attempt <n>]…` | `state: solved`, `closed: <time> by <person>`, section Solution (the text, then `Attempts: 2, 3` when given); `INV-001: solved` |
| `gave-up` | `<ID> --reason <t>` | `state: given up`, `closed`, `given-up-reason`; `INV-001: given up` |

Texts are one line each (a line break or a text starting with `## ` is Code::BadValue). Exactly
one verdict is required (Code::Usage). `--run` must name a usage record, `--attempt` an attempt
that exists (Code::NotFound). `tried`, `solved` and `gave-up` refuse an entry that is not an
approved investigation (Code::Refused); `solved` and `gave-up` refuse one that is not open. All
four are refused in a team dossier (R-INV-1). History events: `tried` (detail `<n> <verdict>`),
`solved`, `given up`, `reopened`.

Changed commands: `list`, `show` and `search` mark an investigation `[open]`, `[solved]` or
`[given up]`. `brief` gains `## Open investigations` after `## Open assumptions`, one line each:
`<ID> · <title> — <n> attempts, last: <attempts::summary>` (when the limit is reached they are cut
after the lessons, findings and team lines and before the proposals: open work in progress stays
longer; T-303). `status` lists them under `## Waiting` as `<ID> · <title> (investigation, <n>
attempts)`. `brief --task` gains `## Investigations`, at most 15 lines: the open ones whose `task`
is the task (title line, then one indented `attempts::summary` per attempt), then closed ones of
the task and up to five others chosen by select::best with the task's tags and title words, one
line each: `<ID> · <title> — solved: <first line of Solution> (<n> failed)` or `— given up:
<reason> (<n> failed)`. `history <task>` also lists the attempts whose `task` is the task:
`<time>  <who>  <ID> attempt <n>  <text>`. `review-task` prints, under each investigation of the
task, its attempts and its solution, and its rules gain: `A solved investigation with failed
attempts is a candidate for a how-to or a lesson.` `promote <INV-ID>` needs `--as howto` or `--as
lesson` (Code::Usage otherwise): a lesson draft gets Statement = Solution and Evidence = Problem
and the attempts; a how-to draft gets Purpose = Problem, Steps = Solution and the attempts that
worked, Traps = the attempts that failed. `after` (§14): the `Stuck` text also names `dossier
problem "<what goes wrong>" --task <id>`, and after `Done(id)` each open investigation that
`history <id> --json` or `list --kind investigation --task <id> --state open --json` names adds
`<INV-ID> is still open: record the result with dossier tried <INV-ID> … or close it with dossier
solved`.

---

# Phase 5 (v0.5)

Same principles as before. Formats: format.md §30 and §31. Two requirements that phase 1 lists
were found not built when this phase was planned and are built here: R-XOS-9 (the PC and the
server compare versions) and R-XOS-3 (a path typed in a folder below the connected folder).

## 16. Remote mode: changes

```rust
// remote/ssh.rs
/// As `argv`, with more: `env` are variables set for the remote command (NAME=<quoted value>, in
/// order, after `cd`), `sub` is a folder below `folder` (with /) to run in when it exists:
/// `cd <quoted folder/sub> 2>/dev/null || cd <quoted folder> && …`. `argv` is
/// argv_with(server, folder, None, DOSSIER_AGENT when given, …).
pub fn argv_with(server: &str, folder: &str, sub: Option<&str>, env: &[(String, String)],
    program: &str, args: &[String], terminal: bool, timeout: u64) -> Vec<String>;
/// Runs the ssh program with the output captured: the exit code, standard output, standard error.
pub fn capture(argv: &[String]) -> Result<(u8, String, String)>;
// Each command makes its own connection (D43): nothing is kept open, R-REMOTE-7 is not built.

// remote/version.rs (R-XOS-9)
pub const FORMAT: u32 = 1;
pub const CLIENT_VERSION: &str = "DOSSIER_CLIENT_VERSION";   // set by the PC for each command
pub const CLIENT_FORMAT: &str = "DOSSIER_CLIENT_FORMAT";
/// On the server, from the two variables: Err(Code::TooNew) naming the side to update when the
/// formats differ; Ok(Some(warning)) when only the program versions differ; Ok(None) when equal
/// or when the variables are not set.
pub fn check(client_version: Option<&str>, client_format: Option<&str>) -> Result<Option<String>>;

// remote/forward.rs: every forwarded `dossier` command gets the two variables (argv_with env)
// and `sub`: the current folder relative to the folder that holds the pointer, with `/`
// (R-XOS-3); none with `--project`. cli.rs main_exit: after forward::maybe, version::check on
// the two variables; an error is emitted and ends the command, a warning is added to the Output.
```

`connect --check` (R-REMOTE-8) runs three commands on the server and prints one line each:
`ssh: ok`, `dossier on the server: <version>` (a warning when it differs from the PC's),
`todo on the server: <version>` (Code::TodoMissing when it is missing), `dossier folder: ok
(<code>)` (Code::NoDossier when the folder has none). When ssh itself fails, the message holds
what ssh wrote to standard error.

## 17. Large files: take and return (R-FILE-1 to R-FILE-7, R-REMOTE-4)

```rust
// taken.rs (format.md §30); replaces the files.json of requirements §7 with a readable file
pub const FILE: &str = "taken.md";
pub const ITEM: &str = "TAKEN";
pub fn load(dossier: &Path, code: &str) -> Result<(BlockFile, Option<Vec<u8>>)>; // empty: "Taken files of <code>"
pub struct Take { pub path: String, pub person: String, pub time: String, pub task: String,
    pub size: u64, pub modified: String, pub checksum: String, pub forced: bool }
pub fn current(file: &BlockFile, path: &str) -> Option<Take>;   // None when not taken now
pub fn all(file: &BlockFile) -> Vec<Take>;                      // taken now, file order
/// The history lines of a path: every take, return and abandon (R-FILE-5).
pub fn log(file: &BlockFile, path: &str) -> Vec<String>;

// config.rs: Config gains `pub take_days: i64` ([limits] take_days, default 14).

// remote/transfer.rs (R-REMOTE-4): `take` and `return` in a connected folder
/// The scp program: DOSSIER_SCP, else `scp`.
pub fn program() -> std::ffi::OsString;
/// The arguments of scp: -s -o BatchMode=yes -o ConnectTimeout=<n> -p, then source and target.
/// -s forces the SFTP protocol (OpenSSH 8.7 or newer; the default from 9.0 on), so that the remote
/// path is read the same way by every scp. An upload (`return --done`) leaves out -p: from Windows
/// it would send the mode 0666 and make the returned file writable for everyone (T-410).
pub fn argv(source: &str, target: &str, timeout: u64) -> Vec<String>;
/// "<server>:<folder/stored>" for scp, not shell-quoted: in SFTP mode the path reaches the server
/// as written, and quotes would become part of the name. Each `\`, `*`, `?`, `[` and `]` gets a
/// backslash in front, so that the SFTP glob matches only this file (tested against OpenSSH 10.2,
/// 2026-11-23: a quoted path is not found, and an unescaped `a*b.sim` also copies `axxb.sim`).
/// For the source of a download (`take`).
pub fn remote_path(server: &str, folder: &str, stored: &str) -> String;
/// "<server>:<folder/stored>" for the target of an upload (`return --done`): unquoted and not
/// escaped, because SFTP does not glob a target and a backslash would become part of the name
/// (tested 2026-11-23).
pub fn remote_target(server: &str, folder: &str, stored: &str) -> String;
/// Called by forward::maybe for `take` and `return` when the folder is connected; Some(exit code).
pub fn maybe(command: &str, args: &[String], base: &Path, pointer: &Pointer, agent: Option<&str>)
    -> Option<u8>;
```

| Command | Arguments | Result |
|---|---|---|
| `take` | `<file>… --task <task> [--force]` | records who took each file, for which task, when, with size, time and checksum; refuses a file another person holds (Code::Refused naming person, task and time) unless `--force`; a file the caller already holds is left as it is and reported again |
| `return` | `<file>… --check \| --done \| --abandon [--force]` | `--check`: exit 0 when every file is still as taken, else Code::Refused naming the changed ones; `--done`: records the new size, time and checksum and ends the take; `--abandon`: ends it; only the person who took a file returns it, or anyone with `--force` |
| `files` | also `--taken`, `--log <file>` | the files taken now (file, person, task, since; `[taken <n> days]` above `take_days`); every recorded version of one file |

`return` is `src/cmd/return_cmd.rs` (`Return(cmd::return_cmd::Args)`). Changed commands: `files`
marks a taken file `[taken by <person>]`; `status` lists the taken files under `## Working now`.
`take` warns about names that cannot exist on Windows and names that differ only in case
(R-XOS-5).

In a connected folder (`transfer::maybe`): every command it runs on the server carries the
variables and the subfolder of every forwarded command (§16: `forward::remote_variables`,
`forward::sub_folder` of the current folder below the connected one), so a path typed in a
subfolder means the same file on both sides. `take` runs `dossier take … --json` on the server;
on exit 0 it copies each file with scp from the server into the same relative path below the
connected folder, first to `<name>.dossier-part` beside it, renamed over the name only when the
copy is complete. A local file that already exists is never overwritten (D46): the line says the
local copy was kept, and that deleting it and running the same `take` copies the server's
version. A failed copy ends with exit 5, leaves no part file, and says that the take is recorded
and that the same `take` command copies again. `return --done` runs `return … --check` on the
server and stops when it fails; copies every file (the current folder joined with the path as
typed) to `remote_target(<folder>/<sub>/<path>.dossier-part)` first, then moves each over its file
with `mv -f` through ssh, in the subfolder; then runs `return … --done`. A failed copy leaves the
server's files and the take as they were, so the same command works again. After the first move
the server's file has changed and a rerun's `--check` would refuse it, so a failure there names
the command that finishes the return on the server (`dossier return <files> --done` in the
folder), which records the new checksums without that check (T-408). When a move fails partway,
that command names only the files already moved, and a second line says to run
`dossier return <the rest> --done` again from the PC, whose server files are still untouched;
names that hold anything but letters, digits and `- _ . /` are quoted (T-409).
`--check` and `--abandon` alone are forwarded like any other command. With `--project` and no
connected folder, `take` and `return --done` are Code::Usage.

---

# Phase 6 (v0.6)

Formats: format.md §32 and §33.

## 18. Milestone digests (R-CTX-14, R-CTX-15)

```rust
// entry/mod.rs: Kind gains Digest (name digest, prefix DIG, folder digests); all() -> [Kind; 9].
// entry/rules.rs: digest: required section Summary and the fields `milestone` and `tasks`;
// order Summary. entry/new.rs: FIELD_ORDER gains `milestone`, `tasks` and `standing` after `task`.
// approval.rs: guarded(Digest) is true: an LLM's digest is a draft until a person approves it.

// digest.rs
/// The approved digests in force (on `day`, when given), newest first.
pub fn in_force<'a>(items: &'a [Item], day: Option<&str>) -> Vec<&'a Item>;
/// The digest that covers this entry: the entry's `task` is in the digest's `tasks` list, and the
/// entry is not a decision with `standing: yes` and not an open assumption or investigation.
pub fn covering<'a>(digests: &[&'a Item], entry: &Entry) -> Option<&'a Item>;
/// The rules for the summary, printed by `digest`.
pub const RULES: [&str; 5];
```

| Command | Arguments | Result |
|---|---|---|
| `digest` | `<milestone>` | the material: the milestone's title and goal (`todo show`), its tasks with status (`todo list --milestone`), the decisions, assumptions, findings, dead ends and investigations of those tasks, then RULES; changes nothing |
| `digest` | `<milestone> --write --stdin` | a digest entry: Summary from standard input (at most 15 lines, Code::BadValue above), `milestone`, `tasks` (the milestone and its tasks, from todo); refused while the milestone is not done in todo; a second digest for a milestone supersedes the first |
| `decide` | also `--standing` | the field `standing: yes` |

`brief` and `status` print, for each digest in force, `<DIG-ID> · <milestone> · <title>` and its
Summary lines indented, in place of the decisions and findings it covers, and one line `<n>
entries are summed up in digests: dossier list --task <task>`.

## 19. Session review (R-DRAFT-7, R-DRAFT-9, D44)

```rust
// session.rs (format.md §34): dossier's own session format
pub const VERSION: u32 = 1;
#[derive(Debug, Clone, PartialEq)]
pub struct Turn { pub role: String, pub text: String }   // role: "user" or "assistant"
/// The turns of a session file. A version above VERSION: Code::TooNew, the message says to
/// update. A missing format line reads as version 1.
pub fn parse(text: &str) -> Result<Vec<Turn>>;
/// A session file of the current version: the format line, `# Session`, a `- tool:` line, then
/// one `## <role>` block per turn. parse(render(tool, turns)) gives the turns back.
pub fn render(tool: &str, turns: &[Turn]) -> String;
/// The turns that state a decision or discuss a result (a word of CUES in them), each with the
/// turn before it, in order, cut from the oldest until they fit `limit` characters.
pub fn relevant(turns: &[Turn], limit: usize) -> Vec<String>;
pub const CUES: [&str; 16];

// claude.rs: the only place that knows the transcript format of Claude Code, which is internal
// and may change with any release. Reading is lenient and never fails on a line it does not know.
/// One turn per JSON line that has a `type` of user or assistant and text in `message.content`
/// (a string, or an array whose `text` items are joined); every other line is skipped.
pub fn turns(transcript: &str) -> Vec<Turn>;

// hooks.rs
pub const SESSION_END: &str = "dossier session-end";
/// The settings with a SessionEnd hook running `dossier session-end --run <quoted command>`. The
/// command is quoted for a POSIX shell, so the hook says `"shell": "bash"`: on Windows Claude Code
/// would otherwise run it in PowerShell when Git Bash is not installed (T-416).
pub fn review_installed(existing: &str, command: &str) -> Result<String>;
```

| Command | Arguments | Result |
|---|---|---|
| `review-session` | `<file> [--claude] [--limit <characters>]` | the material (default limit 20000) and the drafting rules of `review-task`; the file is a session file, or with `--claude` a transcript of Claude Code that is converted first; no turn found: one line saying so |
| `session` | `convert --claude <transcript> [--to <file>]` | the session file made from a transcript of Claude Code, printed or written |
| `session-end` | `--run <command>` | reads the SessionEnd payload from standard input, writes the output of `review-session --claude <transcript_path>` to `.claude/dossier-session-review.md`, starts `<command>` with that path as its last argument without waiting for it, and never fails |
| `hooks` | `install` also takes `--session-review <command>` | adds the SessionEnd hook; `uninstall` removes it |

`review-session`, `session` and `session-end` read files of the machine the session ran on. They
need no dossier and always run locally, also in a connected folder (`LOCAL_COMMANDS`): the server
could not read a path of the PC. What the review then writes (`learn`, `found`) goes to the server
like every other command.

## 20. Extras

| Command | Arguments | Result | Requirement |
|---|---|---|---|
| `story` | `[--milestone <task>]` | the approved decisions, oldest first: `<date>  <ID> · <title>`, the Reason indented, `task: <task>` | R-READ-7 |
| `search` | also `--similar <ID>` (then no words are needed) | the entries sharing most tags and title words with it (select::best, limit 10) | R-READ-9 |
| `brief` | also `--budget <lines>` | the limit for this call | R-CTX-6 |
| `brief --task --json` | — | each item with `reason`: `linked`, `milestone`, `tag match` or `word match` | R-CTX-6 |
| `check` | also `--fix` | renames files that do not start with their ID and slug, adds a missing format line, puts the tool's own fields in the order of format.md; then checks again | R-CHECK-2 |
| `check` | — | warning kind UNIT: a `name: value` or `name = value` line in the Settings or Statement of a how-to or lesson whose value is a bare number | R-ENTRY-11 |
| `export` | `[--milestone <task>] [--to <file>]` | one Markdown document: decisions, findings, investigations and usage records in time order | R-OUT-7 |
| `schema` | — | JSON: every command with its description and schema name, the exit codes, the error codes | R-OUT-6 |
| `team summary` | — | the team dossier: entries per kind and status, drafts waiting, entries due for review, the newest ten | R-TEAM-5 |
| `help` | `<command>` | the description and three examples (two PowerShell, one Linux shell) from `examples.rs` | R-OUT-5 |

```rust
// examples.rs
/// Two PowerShell examples and one for a Linux shell; None for a command without examples.
pub fn of(command: &str) -> Option<[&'static str; 3]>;
// select.rs
/// Which rule gave the points: Some("tag match") when a query tag matched, else
/// Some("word match") when a word matched, else None.
pub fn reason(item: &Item, query: &Query) -> Option<&'static str>;
```

`docs/guide-for-readers.md` (REL-4) is one page for team members who only read and approve:
`drafts`, `approve`, `search`, `show`. R-CTX-8 needs a change in `todo` and is not built here;
R-AGENT-5 gives the same result for Claude Code.
