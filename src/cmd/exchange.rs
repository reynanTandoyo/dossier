//! The `exchange` command (docs/design.md section 9, R-EXCH-3): the received and delivered records
//! of exchange.md in time order, one heading per record with its files and, for a delivery, what
//! it is based on. `--received` or `--delivered` shows one kind; with both or neither, every record
//! is shown. Records are sorted by date, then by the time they were written (the start of the
//! first history line), then by key: the prefix (`DLV` before `RCV`), then the number. Reading
//! writes nothing, and needs no name.

use crate::blockfile::BlockFile;
use crate::cli::Global;
use crate::ctx::Ctx;
use crate::error::Result;
use crate::exchange;
use crate::output::Output;
use serde_json::{Value, json};
use std::path::MAIN_SEPARATOR;

/// The arguments of `exchange`.
#[derive(clap::Args, Debug)]
pub struct Args {
    /// Only the received records
    #[arg(long)]
    pub received: bool,
    /// Only the delivered records
    #[arg(long)]
    pub delivered: bool,
}

/// Which way a record went: received from someone, or delivered to someone (format.md §17).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Direction {
    Received,
    Delivered,
}

impl Direction {
    /// The key prefix of the direction: `RCV` or `DLV`.
    fn prefix(self) -> &'static str {
        match self {
            Direction::Received => "RCV",
            Direction::Delivered => "DLV",
        }
    }

    /// The field that names who the record is with: `from` or `to`.
    fn word(self) -> &'static str {
        match self {
            Direction::Received => "from",
            Direction::Delivered => "to",
        }
    }

    /// The direction as `--json` gives it.
    fn name(self) -> &'static str {
        match self {
            Direction::Received => "received",
            Direction::Delivered => "delivered",
        }
    }
}

/// A stored path with its checksum, written `<stored path>@<checksum>` (format.md §17, §18). The
/// checksum is missing only in a record that was written by hand.
#[derive(Debug, Clone, PartialEq)]
struct Stored {
    path: String,
    checksum: Option<String>,
}

impl Stored {
    /// Splits the text at its last `@`. A text with no `@`, or with nothing after it, has no
    /// checksum.
    fn parse(text: &str) -> Stored {
        match text.rsplit_once('@') {
            Some((path, checksum)) if !checksum.is_empty() => Stored {
                path: path.to_string(),
                checksum: Some(checksum.to_string()),
            },
            Some((path, _)) => Stored {
                path: path.to_string(),
                checksum: None,
            },
            None => Stored {
                path: text.to_string(),
                checksum: None,
            },
        }
    }

    /// The file as `--json` gives it: its path and checksum.
    fn data(&self) -> Value {
        json!({"path": self.path, "checksum": self.checksum})
    }
}

/// What a delivery is based on: a task such as T-042, or a stored path with its checksum, as the
/// `based-on` field of a delivered record has them (format.md §17).
#[derive(Debug, Clone, PartialEq)]
enum BasedOn {
    Task(String),
    Path(Stored),
}

impl BasedOn {
    /// An item with an `@` is a stored path; any other item is a task ID.
    fn parse(text: &str) -> BasedOn {
        if text.contains('@') {
            BasedOn::Path(Stored::parse(text))
        } else {
            BasedOn::Task(text.to_string())
        }
    }

    /// The task ID, or the stored path without its checksum, as the text listing shows it.
    fn shown(&self) -> &str {
        match self {
            BasedOn::Task(task) => task,
            BasedOn::Path(stored) => &stored.path,
        }
    }

    /// The item as `--json` gives it: `{"task": ...}`, or the file as `Stored::data` gives it.
    fn data(&self) -> Value {
        match self {
            BasedOn::Task(task) => json!({"task": task}),
            BasedOn::Path(stored) => stored.data(),
        }
    }
}

/// The number of characters at the start of a history line that give its time: the local time
/// with its UTC offset of format.md §7, such as `2026-11-10T09:00:00+01:00`.
const TIME_CHARS: usize = 25;

/// One record of exchange.md, read for listing.
#[derive(Debug, Clone, PartialEq)]
struct Record {
    key: String,
    direction: Direction,
    /// The number after the prefix. A number too large for a u64 is u64::MAX.
    number: u64,
    /// When the record was written, from its first history line (`written_at`).
    written: String,
    /// The `date` field, `YYYY-MM-DD`; None when it is missing or empty.
    date: Option<String>,
    /// The `from` or `to` field; None when it is missing or empty.
    who: Option<String>,
    files: Vec<Stored>,
    based_on: Vec<BasedOn>,
    note: Option<String>,
}

impl Record {
    /// The heading line: `<date>  <key>  from <who>` or `<date>  <key>  to <who>`. A missing date
    /// or name is shown as `-`.
    fn heading(&self) -> String {
        format!(
            "{}  {}  {} {}",
            self.date.as_deref().unwrap_or("-"),
            self.key,
            self.direction.word(),
            self.who.as_deref().unwrap_or("-")
        )
    }

    /// The record as `--json` gives it. A value the record does not have is `null`, and the
    /// lists are empty.
    fn data(&self) -> Value {
        json!({
            "key": self.key,
            "date": self.date,
            "direction": self.direction.name(),
            "who": self.who,
            "files": self.files.iter().map(Stored::data).collect::<Vec<_>>(),
            "based_on": self.based_on.iter().map(BasedOn::data).collect::<Vec<_>>(),
            "note": self.note,
        })
    }
}

/// Runs `exchange`: the records the flags choose, in time order, one heading line each with its
/// files and what it is based on, then a count. A block whose key is not a record is left out
/// with a warning.
pub fn run(args: &Args, g: &Global) -> Result<Output> {
    let ctx = Ctx::load(g, "exchange")?;
    let (file, _) = exchange::load(&ctx.dossier, &ctx.cfg.code)?;
    let shown = shown_path(&ctx);

    let mut records: Vec<Record> = Vec::new();
    let mut warnings: Vec<String> = Vec::new();
    for key in file.keys() {
        match record_key(&key) {
            Some((direction, number)) => {
                if wanted(args, direction) {
                    records.push(record_of(&file, &key, direction, number));
                }
            }
            None => warnings.push(format!(
                "{shown}: {key} is not a received or delivered record; it is left out"
            )),
        }
    }
    records.sort_by(|a, b| {
        a.date
            .cmp(&b.date)
            .then_with(|| a.written.cmp(&b.written))
            .then_with(|| a.direction.prefix().cmp(b.direction.prefix()))
            .then_with(|| a.number.cmp(&b.number))
    });

    let mut lines: Vec<String> = Vec::new();
    for record in &records {
        lines.push(record.heading());
        lines.extend(
            record
                .files
                .iter()
                .map(|stored| format!("  {}", stored.path)),
        );
        lines.extend(
            record
                .based_on
                .iter()
                .map(|item| format!("  based on {}", item.shown())),
        );
    }
    lines.push(count(records.len()));
    let data = Value::Array(records.iter().map(Record::data).collect());

    let output = Output::with(lines.join("\n"), data);
    Ok(warnings
        .into_iter()
        .fold(output, |output, warning| output.warn(warning)))
}

/// Whether a record of this direction is listed. With both flags or neither, every direction is.
fn wanted(args: &Args, direction: Direction) -> bool {
    if args.received == args.delivered {
        return true;
    }
    match direction {
        Direction::Received => args.received,
        Direction::Delivered => args.delivered,
    }
}

/// The direction and the number of a key `RCV-<n>` or `DLV-<n>`, None for any other key. The
/// number is read as digits only, so `RCV-4 later` is not a record.
fn record_key(key: &str) -> Option<(Direction, u64)> {
    let (prefix, digits) = key.split_once('-')?;
    let direction = match prefix {
        "RCV" => Direction::Received,
        "DLV" => Direction::Delivered,
        _ => return None,
    };
    if digits.is_empty() || !digits.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    Some((direction, digits.parse().unwrap_or(u64::MAX)))
}

/// The record of a block: its fields as format.md §17 names them. A field that is missing or
/// empty is None, or an empty list.
fn record_of(file: &BlockFile, key: &str, direction: Direction, number: u64) -> Record {
    let text = |name: &str| file.field(key, name).filter(|value| !value.is_empty());
    Record {
        key: key.to_string(),
        direction,
        number,
        written: written_at(file, key),
        date: text("date"),
        who: text(direction.word()),
        files: list(file, key, "files")
            .iter()
            .map(|item| Stored::parse(item))
            .collect(),
        based_on: list(file, key, "based-on")
            .iter()
            .map(|item| BasedOn::parse(item))
            .collect(),
        note: text("note"),
    }
}

/// When a record was written: the first `TIME_CHARS` characters of its first history line, which
/// begins with that time (format.md §6, §7). Times with the same offset sort in time order as text.
/// A record with no history line has "", which sorts first.
fn written_at(file: &BlockFile, key: &str) -> String {
    file.history(key)
        .first()
        .map(|line| line.chars().take(TIME_CHARS).collect::<String>())
        .unwrap_or_default()
}

/// The items of a comma-separated field of a block, trimmed. Empty items are left out.
fn list(file: &BlockFile, key: &str, name: &str) -> Vec<String> {
    file.field(key, name)
        .unwrap_or_default()
        .split(',')
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .map(str::to_string)
        .collect()
}

/// `1 record`, or `<n> records`.
fn count(n: usize) -> String {
    if n == 1 {
        "1 record".to_string()
    } else {
        format!("{n} records")
    }
}

/// The exchange file, relative to the project folder, with `/` between the parts.
fn shown_path(ctx: &Ctx) -> String {
    let path = ctx.dossier.join(exchange::FILE);
    let relative = path.strip_prefix(&ctx.root).unwrap_or(path.as_path());
    relative.display().to_string().replace(MAIN_SEPARATOR, "/")
}
