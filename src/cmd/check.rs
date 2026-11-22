//! The `check` command: reads every entry of the dossier and every block file (values.md, map.md,
//! exchange.md and files/<task>.md, format.md §17), and reports each problem on its own line as
//! `<file>:<line>: <ID> <KIND> <message>` (R-CHECK-1, design.md section 5). A block file has no
//! entry ID, so its ID is `-`. Nothing is changed. A dossier without problems says
//! `DOSSIER: ok (0 problems)`; otherwise the command fails with exit code 1, with or without
//! `--json`. A SHOULD report, such as a TERM report of an abbreviation that no glossary explains
//! (R-TERM-3), is a warning: it does not fail the command.

use crate::approval::{self, Mark};
use crate::blockfile::BlockFile;
use crate::cli::Global;
use crate::ctx::Ctx;
use crate::entry::rules;
use crate::entry::{Entry, Kind, Status};
use crate::error::{Code, Error, Result};
use crate::exchange;
use crate::filelist;
use crate::filesum;
use crate::glossary::{self, Term};
use crate::mapfile;
use crate::output::Output;
use crate::paths;
use crate::store::{self, Item};
use crate::time;
use crate::values;
use chrono::NaiveDate;
use std::collections::{BTreeMap, HashSet};
use std::path::Path;

/// The arguments of `check`.
#[derive(clap::Args, Debug)]
pub struct Args {}

/// The line that says the dossier has no problems.
const OK: &str = "DOSSIER: ok (0 problems)";

/// The byte-order mark a file may start with (format.md §2).
const BYTE_ORDER_MARK: char = '\u{feff}';

/// The glossary file of a dossier (format.md §20).
const GLOSSARY: &str = "glossary.md";

/// The prefixes of the IDs of exchange records (RCV, DLV; ids::next_prefix). The entries name
/// those IDs, and an ID is not an abbreviation.
const RECORD_PREFIXES: [&str; 2] = ["RCV", "DLV"];

/// The number of entries an abbreviation must appear in for a TERM report (R-TERM-3).
const MIN_ENTRIES: usize = 3;

/// Runs `check`. No problem: the ok line, with each SHOULD report as a warning. Otherwise
/// Code::Refused, whose message holds every problem line, sorted by file and line, then the
/// SHOULD reports as `warning:` lines, then the count line of the problems.
pub fn run(_args: &Args, g: &Global) -> Result<Output> {
    let ctx = Ctx::load(g, "check")?;
    let (items, warnings) = store::all(&ctx.dossier);
    let known: HashSet<String> = items.iter().map(|item| item.entry.id()).collect();

    let mut found: Vec<Found> = warnings
        .iter()
        .map(|warning| parse_problem(&ctx.root, warning))
        .collect();
    found.extend(duplicates(&ctx.root, &items));
    let today = time::date(&time::now()).to_string();
    for item in &items {
        found.extend(entry_problems(&ctx, item, &known, &today));
    }
    found.extend(block_problems(&ctx, &known));
    let (glossary_problems, mut should) = glossary_check(&ctx, &items);
    found.extend(glossary_problems);

    found.sort();
    should.sort();
    if found.is_empty() {
        return Ok(should.iter().fold(Output::text(OK), |output, report| {
            output.warn(report.text())
        }));
    }
    let mut lines: Vec<String> = found.iter().map(Found::text).collect();
    lines.extend(
        should
            .iter()
            .map(|report| format!("warning: {}", report.text())),
    );
    lines.push(format!("DOSSIER: {} problems", found.len()));
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
    parse_found(root, Path::new(path), message)
}

/// The PARSE problem of the file at `path`, from a parse message: `line <n>: <message>` gives
/// line n, anything else gives line 1.
fn parse_found(root: &Path, path: &Path, message: &str) -> Found {
    let (line, message) = numbered(message);
    Found::new(&relative(root, path), line, "-", "PARSE", message)
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

/// The problems of one entry, except duplicated IDs (which need the other entries). `today` is
/// the date (YYYY-MM-DD) that review dates are compared with.
fn entry_problems(ctx: &Ctx, item: &Item, known: &HashSet<String>, today: &str) -> Vec<Found> {
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

    // LINK: every ID that a link field names must be an entry of the dossier (format.md §14).
    let links = [
        ("related", entry.list("related")),
        ("supersedes", single_id(entry, "supersedes")),
        ("superseded-by", single_id(entry, "superseded-by")),
        ("rests-on", entry.list("rests-on")),
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

    // CHANGED: an approved lesson, how-to or tool (approval::guarded) whose text differs from the
    // checksum recorded at approval.
    if entry.kind().is_some_and(approval::guarded)
        && entry.status() == Some(Status::Approved)
        && matches!(approval::mark(&ctx.state, &id, &item.raw), Mark::Changed)
    {
        let kind = entry.kind().map_or("entry", Kind::name);
        add(
            1,
            "CHANGED",
            format!("approved {kind} changed after approval"),
        );
    }

    // REVIEW: a review date that has passed, or that is not a date (R-LIFE-7, format.md §23).
    if let Some(message) = review_problem(entry, today) {
        add(field_line(entry, "review"), "REVIEW", message);
    }

    // TOOL: a tool whose file is gone, or changed since its checksum (R-USE-11, format.md §22).
    if let Some((line, message)) = tool_problem(ctx, entry) {
        add(line, "TOOL", message);
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

/// The REVIEW message of an entry whose `review` date (YYYY-MM-DD, format.md §23) is before
/// `today`, or that is not a date at all. An empty value is no review date, and a review date of
/// today is not yet past (R-LIFE-7). None when `today` is not a date.
fn review_problem(entry: &Entry, today: &str) -> Option<String> {
    let value = entry.field("review").filter(|value| !value.is_empty())?;
    let Some(date) = review_date(&value) else {
        return Some(format!("review is not a date: \"{value}\""));
    };
    let today = review_date(today)?;
    (date < today).then(|| format!("due for review: review date {value} has passed"))
}

/// A date of the form YYYY-MM-DD, None for any other text.
fn review_date(text: &str) -> Option<NaiveDate> {
    NaiveDate::parse_from_str(text, "%Y-%m-%d")
        .ok()
        .filter(|_| text.len() == 10)
}

/// The TOOL problem of a tool that has a `checksum` and a `location` (format.md §22): its file is
/// gone, or it no longer has the recorded checksum. The location is a path, relative to the
/// project folder or absolute. A checksum is written only for a location that is a file, so a
/// tool without one (a command name or a URL) is not checked. The line is the location's for a
/// missing file and the checksum's for a changed one.
fn tool_problem(ctx: &Ctx, entry: &Entry) -> Option<(usize, String)> {
    if entry.kind() != Some(Kind::Tool) {
        return None;
    }
    let recorded = entry.field("checksum").filter(|value| !value.is_empty())?;
    let location = entry.field("location").filter(|value| !value.is_empty())?;
    let path = Path::new(&location);
    let path = if path.is_absolute() {
        path.to_path_buf()
    } else {
        ctx.root.join(path)
    };
    if !path.exists() {
        return Some((
            field_line(entry, "location"),
            format!("file is gone: {location}"),
        ));
    }
    match filesum::checksum(&path) {
        Ok(now) if now == recorded => None,
        Ok(_) => Some((
            field_line(entry, "checksum"),
            format!("file changed since its checksum: {location}"),
        )),
        Err(error) => Some((
            field_line(entry, "location"),
            format!("cannot read the file {location}: {}", error.message),
        )),
    }
}

/// The problems of the block files (format.md §17): values.md, map.md, exchange.md and every
/// files/<task>.md. A file that cannot be read, is not UTF-8 or does not parse is PARSE. A
/// `rests-on` field of one of its blocks that names an ID which is not an entry is LINK. A file
/// that does not exist has no problem.
fn block_problems(ctx: &Ctx, known: &HashSet<String>) -> Vec<Found> {
    let mut paths = vec![
        ctx.dossier.join(values::FILE),
        ctx.dossier.join(mapfile::FILE),
        ctx.dossier.join(exchange::FILE),
    ];
    // filelist::tasks gives each name without `.md`, so this is the file name as it is on disk
    // (filelist::path would upper-case it).
    paths.extend(
        filelist::tasks(&ctx.dossier)
            .iter()
            .map(|task| ctx.dossier.join("files").join(format!("{task}.md"))),
    );
    let mut found = Vec::new();
    for path in &paths {
        match read_block(&ctx.root, path) {
            Ok(Some(file)) => found.extend(block_links(&ctx.root, path, &file, known)),
            Ok(None) => {}
            Err(problem) => found.push(problem),
        }
    }
    found
}

/// Reads a block file as BlockFile::parse reads it: None when the file does not exist. A file
/// that cannot be read, is not UTF-8 or does not parse gives its PARSE problem.
fn read_block(root: &Path, path: &Path) -> std::result::Result<Option<BlockFile>, Found> {
    let bytes = match std::fs::read(path) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(parse_found(root, path, &error.to_string())),
    };
    let text = std::str::from_utf8(&bytes).map_err(|_| parse_found(root, path, "not UTF-8"))?;
    BlockFile::parse(text)
        .map(Some)
        .map_err(|error| parse_found(root, path, &error.message))
}

/// The LINK problems of the `rests-on` fields of one block file (format.md §14, §17). Every block
/// counts, and each field is reported on the line it is written on. A block has no entry ID, so
/// the ID column is `-`.
fn block_links(root: &Path, path: &Path, file: &BlockFile, known: &HashSet<String>) -> Vec<Found> {
    let file_name = relative(root, path);
    let mut found = Vec::new();
    let mut block: Option<&str> = None;
    for (number, text) in numbered_lines(&file.lines) {
        if let Some(key) = text.strip_prefix("## ") {
            block = Some(key.trim());
            continue;
        }
        let (Some(key), Some(value)) = (block, field_value(text, "rests-on")) else {
            continue;
        };
        for target in value.split(',').map(str::trim).filter(|id| !id.is_empty()) {
            if !known.contains(&target_id(target)) {
                found.push(Found::new(
                    &file_name,
                    number,
                    "-",
                    "LINK",
                    format!("{key} rests-on names {target}, which does not exist"),
                ));
            }
        }
    }
    found
}

/// The trimmed value of a field line `- <name>: <value>` (format.md §3), None for any other line.
fn field_value<'a>(text: &'a str, name: &str) -> Option<&'a str> {
    let (key, value) = text.strip_prefix("- ")?.split_once(':')?;
    (key == name).then_some(value.trim())
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

/// The lines of a file (an entry or a block file) without their line endings (the byte-order mark
/// of the first line is removed), each with its 1-based number.
fn numbered_lines(lines: &[String]) -> Vec<(usize, &str)> {
    lines
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
    numbered_lines(&entry.lines)
        .into_iter()
        .find(|(_, text)| text.starts_with("# "))
        .map_or(1, |(number, _)| number)
}

/// The line of the first field `- <key>: ...` in the field block (format.md §3); 1 when there
/// is none.
fn field_line(entry: &Entry, key: &str) -> usize {
    numbered_lines(&entry.lines)
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
    numbered_lines(&entry.lines)
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

/// Where an abbreviation first appears, and in how many entries it appears.
struct Seen {
    entries: usize,
    file: String,
    line: usize,
    id: String,
}

/// The TERM reports of the glossaries (R-TERM-3) and the PARSE problems of the glossaries that
/// cannot be read. The terms of the project's glossary and the team's both count. When a glossary
/// cannot be read, no TERM report is made, since the terms are not all known.
fn glossary_check(ctx: &Ctx, items: &[Item]) -> (Vec<Found>, Vec<Found>) {
    let mut dossiers: Vec<&Path> = vec![ctx.dossier.as_path()];
    dossiers.extend(ctx.team.as_deref());
    let mut terms = Vec::new();
    let mut problems = Vec::new();
    for dossier in dossiers {
        // The code only names a missing glossary, which is empty and is not an error.
        match glossary::load(dossier, &ctx.cfg.code) {
            Ok((file, _)) => terms.extend(glossary::terms(&file)),
            Err(error) => problems.push(glossary_problem(
                &ctx.root,
                &dossier.join(GLOSSARY),
                &error.message,
            )),
        }
    }
    if problems.is_empty() {
        (problems, term_reports(ctx, items, &terms))
    } else {
        (problems, Vec::new())
    }
}

/// The PARSE problem of a glossary that cannot be read. glossary::load starts its message with the
/// file name, which is left out, since the file is shown by its path.
fn glossary_problem(root: &Path, path: &Path, message: &str) -> Found {
    let message = message
        .strip_prefix(GLOSSARY)
        .and_then(|rest| rest.strip_prefix(": "))
        .unwrap_or(message);
    parse_found(root, path, message)
}

/// The TERM reports (R-TERM-3): each abbreviation that appears in at least MIN_ENTRIES entries and
/// is in no glossary term, once, at the first entry in ID order where it appears and the first line
/// of that entry where it appears. Entries are counted, not occurrences.
fn term_reports(ctx: &Ctx, items: &[Item], terms: &[Term]) -> Vec<Found> {
    let excluded = not_abbreviations(&ctx.cfg.code);
    let names = glossary_names(terms);
    let mut seen: BTreeMap<String, Seen> = BTreeMap::new();
    for item in items {
        for (word, line) in abbreviations(&item.entry, &excluded) {
            if names.contains(&word.to_lowercase()) {
                continue;
            }
            seen.entry(word)
                .or_insert_with(|| Seen {
                    entries: 0,
                    file: relative(&ctx.root, &item.path),
                    line,
                    id: item.entry.id(),
                })
                .entries += 1;
        }
    }
    seen.into_iter()
        .filter(|(_, seen)| seen.entries >= MIN_ENTRIES)
        .map(|(word, seen)| {
            let message = format!(
                "{word} appears in {} entries and is in no glossary; add it with: dossier term add {word} \"<meaning>\"",
                seen.entries
            );
            Found::new(&seen.file, seen.line, &seen.id, "TERM", message)
        })
        .collect()
}

/// The abbreviations of an entry, each with the line where it first appears (R-TERM-3). A word is
/// what lies between characters that are not letters or digits; the words of the title and of the
/// section text count, and a word in `excluded` does not.
fn abbreviations(entry: &Entry, excluded: &HashSet<String>) -> BTreeMap<String, usize> {
    let mut found = BTreeMap::new();
    for (number, text) in text_lines(entry) {
        for word in text.split(|c: char| !c.is_alphanumeric()) {
            if is_abbreviation(word) && !excluded.contains(word) {
                found.entry(word.to_string()).or_insert(number);
            }
        }
    }
    found
}

/// Whether a word has the form of an abbreviation: 2 to 6 ASCII capital letters.
fn is_abbreviation(word: &str) -> bool {
    (2..=6).contains(&word.len()) && word.bytes().all(|byte| byte.is_ascii_uppercase())
}

/// The lines of an entry that hold its title and the text of its sections, each with its number.
/// The field lines before the first section (format.md §3) and the section headings are left out.
fn text_lines(entry: &Entry) -> Vec<(usize, &str)> {
    let mut in_sections = false;
    let mut lines = Vec::new();
    for (number, text) in numbered_lines(&entry.lines) {
        if text.starts_with("## ") {
            in_sections = true;
        } else if in_sections || text.starts_with("# ") {
            lines.push((number, text));
        }
    }
    lines
}

/// The words that are not abbreviations, even though they have the form of one: the ID prefix of
/// every kind of entry, the prefixes of exchange records, and the words of the project code.
fn not_abbreviations(code: &str) -> HashSet<String> {
    Kind::all()
        .into_iter()
        .map(Kind::prefix)
        .chain(RECORD_PREFIXES)
        .map(str::to_string)
        .chain(
            code.split(|c: char| !c.is_alphanumeric())
                .map(str::to_string),
        )
        .collect()
}

/// The names of the glossary terms, each term's, symbol and other spellings, in lower case.
fn glossary_names(terms: &[Term]) -> HashSet<String> {
    terms
        .iter()
        .flat_map(|term| {
            std::iter::once(term.term.as_str())
                .chain(term.symbol.as_deref())
                .chain(term.also.iter().map(String::as_str))
        })
        .map(|name| name.trim().to_lowercase())
        .collect()
}
