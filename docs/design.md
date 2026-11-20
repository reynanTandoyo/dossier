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
   of `tests/common/mod.rs`, run with `cargo test --test cmd_<name>`.

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
}
```

Tests never touch a real project, a real server or the user's register (`DOSSIER_REGISTER`,
`DOSSIER_SSH` and `DOSSIER_TODO` point into the temporary folder).
