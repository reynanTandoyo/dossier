//! The `check` command: reads every entry of the dossier and reports each problem on its own
//! line as `<file>:<line>: <ID> <KIND> <message>` (R-CHECK-1, design.md section 5). Nothing is
//! changed. A dossier without problems says `DOSSIER: ok (0 problems)`; otherwise the command
//! fails with exit code 1, with or without `--json`.

use crate::approval::{self, Mark};
use crate::cli::Global;
use crate::ctx::Ctx;
use crate::entry::rules;
use crate::entry::{Entry, Kind, Status};
use crate::error::{Code, Error, Result};
use crate::output::Output;
use crate::paths;
use crate::store::{self, Item};
use crate::time;
use std::collections::{BTreeMap, HashSet};
use std::path::Path;

/// The arguments of `check`.
#[derive(clap::Args, Debug)]
pub struct Args {}

/// The line that says the dossier has no problems.
const OK: &str = "DOSSIER: ok (0 problems)";

/// The byte-order mark a file may start with (format.md §2).
const BYTE_ORDER_MARK: char = '\u{feff}';

/// Runs `check`. No problem: the ok line. Otherwise Code::Refused, whose message holds every
/// problem line, sorted by file and line, and then the count line.
pub fn run(_args: &Args, g: &Global) -> Result<Output> {
    let ctx = Ctx::load(g, "check")?;
    let (items, warnings) = store::all(&ctx.dossier);
    let known: HashSet<String> = items.iter().map(|item| item.entry.id()).collect();

    let mut found: Vec<Found> = warnings
        .iter()
        .map(|warning| parse_problem(&ctx.root, warning))
        .collect();
    found.extend(duplicates(&ctx.root, &items));
    for item in &items {
        found.extend(entry_problems(&ctx, item, &known));
    }
    if found.is_empty() {
        return Ok(Output::text(OK));
    }

    found.sort();
    let mut lines: Vec<String> = found.iter().map(Found::text).collect();
    lines.push(format!("DOSSIER: {} problems", lines.len()));
    Err(Error::new(Code::Refused, lines.join("\n")))
}

/// One problem: the file (relative to the project folder, with `/`), the line, the ID of the
/// entry (`-` when it has none), the kind and the message. The derived order is the printed order.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Found {
    file: String,
    line: usize,
    id: String,
    kind: &'static str,
    message: String,
}

impl Found {
    fn new(
        file: &str,
        line: usize,
        id: &str,
        kind: &'static str,
        message: impl Into<String>,
    ) -> Found {
        Found {
            file: file.to_string(),
            line,
            id: id.to_string(),
            kind,
            message: message.into(),
        }
    }

    /// The printed line: `<file>:<line>: <ID> <KIND> <message>`.
    fn text(&self) -> String {
        format!(
            "{}:{}: {} {} {}",
            self.file, self.line, self.id, self.kind, self.message
        )
    }
}

/// The problem of a warning of `store::all`, which reads `<path>: line <n>: <message>` or
/// `<path>: <message>`. The problem has no ID, so it shows `-`.
fn parse_problem(root: &Path, warning: &str) -> Found {
    let (path, message) = split_warning(warning);
    let (line, message) = numbered(message);
    Found::new(
        &relative(root, Path::new(path)),
        line,
        "-",
        "PARSE",
        message,
    )
}

/// Splits a warning into its path and its message. The path ends at the first `: ` that is
/// followed by a path which exists, so that a file name containing `: ` still works. When none
/// exists, the first `: ` ends the path.
fn split_warning(warning: &str) -> (&str, &str) {
    let ends: Vec<usize> = warning
        .match_indices(": ")
        .map(|(index, _)| index)
        .collect();
    let existing = ends
        .iter()
        .copied()
        .find(|&end| Path::new(&warning[..end]).exists());
    match existing.or_else(|| ends.first().copied()) {
        Some(end) => (&warning[..end], &warning[end + 2..]),
        None => (warning, ""),
    }
}

/// The line and the message of a parse message: `line <n>: <message>` gives n, anything else
/// gives line 1.
fn numbered(message: &str) -> (usize, &str) {
    message
        .strip_prefix("line ")
        .and_then(|rest| rest.split_once(": "))
        .and_then(|(number, rest)| number.parse::<usize>().ok().map(|line| (line, rest)))
        .unwrap_or((1, message))
}

/// The DUPLICATE problems: every entry whose ID is also the ID of another file, on its heading
/// line, naming the other files.
fn duplicates(root: &Path, items: &[Item]) -> Vec<Found> {
    let mut by_id: BTreeMap<String, Vec<&Item>> = BTreeMap::new();
    for item in items {
        by_id.entry(item.entry.id()).or_default().push(item);
    }
    let mut found = Vec::new();
    for (id, group) in &by_id {
        if group.len() < 2 {
            continue;
        }
        for item in group {
            let others: Vec<String> = group
                .iter()
                .filter(|other| other.path != item.path)
                .map(|other| relative(root, &other.path))
                .collect();
            found.push(Found::new(
                &relative(root, &item.path),
                heading_line(&item.entry),
                id,
                "DUPLICATE",
                format!("the same ID is also in {}", others.join(", ")),
            ));
        }
    }
    found
}

/// The problems of one entry, except duplicated IDs (which need the other entries).
fn entry_problems(ctx: &Ctx, item: &Item, known: &HashSet<String>) -> Vec<Found> {
    let entry = &item.entry;
    let id = entry.id();
    let file = relative(&ctx.root, &item.path);
    let mut found = Vec::new();
    let mut add = |line: usize, kind: &'static str, message: String| {
        found.push(Found::new(&file, line, &id, kind, message));
    };

    // NAME: the file name starts with the ID and can exist on Windows (R-XOS-4).
    let name = item
        .path
        .file_name()
        .map_or_else(String::new, |name| name.to_string_lossy().into_owned());
    if !name.starts_with(&format!("{id}-")) {
        add(1, "NAME", format!("file name does not start with {id}-"));
    }
    if !paths::windows_safe(&name) {
        add(
            1,
            "NAME",
            format!("file name {name} cannot exist on Windows"),
        );
    }

    // STATUS: one of the five statuses.
    if entry.status().is_none() {
        let line = field_line(entry, "status");
        let message = match entry.field("status").filter(|value| !value.is_empty()) {
            Some(value) => format!(
                "status must be draft, approved, rejected, superseded or retired, not \"{value}\""
            ),
            None => "missing field status".to_string(),
        };
        add(line, "STATUS", message);
    }

    // INCOMPLETE: an approved entry must still have what approval needs (design.md section 5).
    if entry.status() == Some(Status::Approved) {
        for problem in rules::problems(entry) {
            add(
                problem_line(entry, &problem.what),
                "INCOMPLETE",
                problem.what,
            );
        }
    }

    // LINK: every ID that a link field names must be an entry of the dossier.
    let links = [
        ("related", entry.list("related")),
        ("supersedes", single_id(entry, "supersedes")),
        ("superseded-by", single_id(entry, "superseded-by")),
    ];
    for (key, targets) in links {
        for target in targets {
            if !known.contains(&target_id(&target)) {
                add(
                    field_line(entry, key),
                    "LINK",
                    format!("{key} names {target}, which does not exist"),
                );
            }
        }
    }

    // CHANGED: an approved lesson whose text differs from the checksum recorded at approval.
    if entry.kind() == Some(Kind::Lesson)
        && entry.status() == Some(Status::Approved)
        && matches!(approval::mark(&ctx.state, &id, &item.raw), Mark::Changed)
    {
        add(
            1,
            "CHANGED",
            "approved lesson changed after approval".to_string(),
        );
    }

    // TIME: created, and approved when there is one (or the entry is approved), starts with a
    // time stamp of format.md §7.
    match entry.field("created") {
        Some(value) if !starts_with_time(&value) => add(
            field_line(entry, "created"),
            "TIME",
            format!("created is not a time: \"{value}\""),
        ),
        Some(_) => {}
        None => add(1, "TIME", "missing field created".to_string()),
    }
    match entry.field("approved") {
        Some(value) if !starts_with_time(&value) => add(
            field_line(entry, "approved"),
            "TIME",
            format!("approved is not a time: \"{value}\""),
        ),
        None if entry.status() == Some(Status::Approved) => {
            add(1, "TIME", "missing field approved".to_string());
        }
        _ => {}
    }

    // WHO: every entry names its author and has at least one history line.
    match entry.field("author") {
        Some(author) if !author.is_empty() => {}
        Some(_) => add(
            field_line(entry, "author"),
            "WHO",
            "field author is empty".to_string(),
        ),
        None => add(1, "WHO", "missing field author".to_string()),
    }
    if entry.history().is_empty() {
        add(1, "WHO", "no history line".to_string());
    }
    found
}

/// The path of a file relative to the project folder, with `/` between its parts on every
/// system.
fn relative(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .components()
        .map(|part| part.as_os_str().to_string_lossy().into_owned())
        .collect::<Vec<_>>()
        .join("/")
}

/// The lines of an entry without their line endings (the byte-order mark of the first line is
/// removed), each with its 1-based number.
fn numbered_lines(entry: &Entry) -> Vec<(usize, &str)> {
    entry
        .lines
        .iter()
        .enumerate()
        .map(|(index, line)| {
            let line = line.as_str();
            let text = line
                .strip_suffix("\r\n")
                .or_else(|| line.strip_suffix('\n'))
                .unwrap_or(line);
            let text = if index == 0 {
                text.strip_prefix(BYTE_ORDER_MARK).unwrap_or(text)
            } else {
                text
            };
            (index + 1, text)
        })
        .collect()
}

/// The line of the heading (`# `, format.md §3); 1 when there is none.
fn heading_line(entry: &Entry) -> usize {
    numbered_lines(entry)
        .into_iter()
        .find(|(_, text)| text.starts_with("# "))
        .map_or(1, |(number, _)| number)
}

/// The line of the first field `- <key>: ...` in the field block (format.md §3); 1 when there
/// is none.
fn field_line(entry: &Entry, key: &str) -> usize {
    numbered_lines(entry)
        .into_iter()
        .take_while(|(_, text)| !text.starts_with("## "))
        .find(|(_, text)| {
            text.strip_prefix("- ")
                .and_then(|rest| rest.split_once(':'))
                .is_some_and(|(name, _)| name == key)
        })
        .map_or(1, |(number, _)| number)
}

/// The line of the `## <name>` heading of a section; 1 when there is none.
fn section_line(entry: &Entry, name: &str) -> usize {
    numbered_lines(entry)
        .into_iter()
        .find(|(_, text)| {
            text.strip_prefix("## ")
                .is_some_and(|title| title.trim() == name)
        })
        .map_or(1, |(number, _)| number)
}

/// The line that a problem of `rules::problems` is about: the section of `missing section <name>`,
/// the field of `missing field <key>`, or the field that starts the text (`state must be ...`).
/// 1 when the line is not in the file.
fn problem_line(entry: &Entry, what: &str) -> usize {
    if let Some(name) = what.strip_prefix("missing section ") {
        section_line(entry, name)
    } else if let Some(key) = what.strip_prefix("missing field ") {
        field_line(entry, key)
    } else {
        what.split_whitespace()
            .next()
            .map_or(1, |key| field_line(entry, key))
    }
}

/// Whether a value starts with a time stamp of format.md §7 (the first 25 characters are one).
fn starts_with_time(value: &str) -> bool {
    time::valid(&value.chars().take(25).collect::<String>())
}

/// The ID a link names, read as `store::get` reads it: a place prefix is dropped and the case
/// is ignored.
fn target_id(target: &str) -> String {
    target
        .rsplit_once(':')
        .map_or(target, |(_, rest)| rest)
        .to_uppercase()
}

/// The value of a field that names one ID, when it is not empty.
fn single_id(entry: &Entry, key: &str) -> Vec<String> {
    entry
        .field(key)
        .filter(|value| !value.is_empty())
        .into_iter()
        .collect()
}
