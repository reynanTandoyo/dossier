//! The `delivered` command (docs/design.md section 9, R-EXCH-2, R-EXCH-4): records the files that
//! went to the customer or another party as a DLV record in exchange.md, with the checksum of each
//! file and of everything named with `--based-on`. Before it writes, it looks at what the files are
//! based on. It warns about a task named whose files are outdated or rest on a wrong assumption,
//! or whose entries rest on one; about a file named that has such a mark in a file list; and about
//! an open approved assumption whose Affects section names a task or file named. With warnings, a
//! person at a terminal is asked to confirm; an LLM, or no terminal, gets Code::NeedsTerminal
//! listing them, and nothing is written.

use crate::cli::Global;
use crate::ctx::Ctx;
use crate::entry::{Kind, Status};
use crate::error::{Code, Error, Result};
use crate::exchange;
use crate::filelist;
use crate::ids;
use crate::output::Output;
use crate::paths;
use crate::store::{self, Item};
use crate::term;
use crate::time;
use crate::tx::{self, Change};
use chrono::NaiveDate;
use serde_json::json;
use std::io::Write;
use std::path::{MAIN_SEPARATOR, Path};

/// The prefix of delivered records: DLV-001, DLV-002, ... (format.md §17).
const PREFIX: &str = "DLV";

/// The key a dry run uses in place of a new number, as `received` uses `RCV-NEW`.
const PLACEHOLDER: &str = "DLV-NEW";

/// The history event of a delivered record, and its journal event.
const DELIVERED: &str = "delivered";

/// The question asked before a delivery with warnings is written.
const QUESTION: &str = "deliver anyway? (yes/no) ";

/// The state of an assumption that nobody has confirmed or found wrong (format.md §14).
const OPEN: &str = "open";

/// The mark of a file that is wrong because what it was made from changed (format.md §17).
const OUTDATED: &str = "outdated";

/// The mark of a file or entry that rests on an assumption found wrong (format.md §14).
const WRONG_ASSUMPTION: &str = "rests on a wrong assumption";

/// The arguments of `delivered`.
#[derive(clap::Args, Debug)]
pub struct Args {
    /// The files delivered, as paths relative to the current folder or absolute (at least one)
    #[arg(required = true, value_name = "path")]
    pub paths: Vec<String>,
    /// Who the files were sent to, such as the customer
    #[arg(long)]
    pub to: String,
    /// The date the files went, YYYY-MM-DD (default: today)
    #[arg(long, value_name = "date")]
    pub date: Option<String>,
    /// What the files are based on: a task such as T-042, or a file (repeatable)
    #[arg(long = "based-on", value_name = "task or path")]
    pub based_on: Vec<String>,
    /// A note on what was delivered
    #[arg(long)]
    pub note: Option<String>,
}

/// A file sent: its path as stored and its checksum (format.md §18).
struct Sent {
    stored: String,
    checksum: String,
}

/// What the files are based on: a task, or a file with its checksum.
#[derive(Debug, Clone, PartialEq)]
enum BasedOn {
    Task(String),
    Path { stored: String, checksum: String },
}

impl BasedOn {
    /// The task ID, or the stored path, as the warnings name it.
    fn name(&self) -> &str {
        match self {
            BasedOn::Task(task) => task.as_str(),
            BasedOn::Path { stored, .. } => stored.as_str(),
        }
    }

    /// The item as the `based-on` field names it: the task ID, or `<stored path>@<checksum>`.
    fn text(&self) -> String {
        match self {
            BasedOn::Task(task) => task.clone(),
            BasedOn::Path { stored, checksum } => format!("{stored}@{checksum}"),
        }
    }

    /// The item as `--json` gives it.
    fn json(&self) -> serde_json::Value {
        match self {
            BasedOn::Task(task) => json!({"task": task}),
            BasedOn::Path { stored, checksum } => json!({"path": stored, "checksum": checksum}),
        }
    }
}

/// Runs `delivered`: checks the options and the files, looks at what the delivery is based on,
/// asks a person when there are warnings, then in one transaction writes the DLV record. The
/// first output line is `<DLV-ID>: delivered <n> files to <who>`, or `nothing changed` when a
/// person answers anything but yes to the question.
pub fn run(args: &Args, g: &Global) -> Result<Output> {
    let ctx = Ctx::load(g, "delivered")?;
    let who = ctx.writer()?.clone();
    let to = to_of(&args.to)?;
    let date = date_of(args.date.as_deref())?;
    let note = trimmed(args.note.as_deref());

    // The files and what they are based on are read before the transaction, so a missing or
    // refused path writes nothing.
    let cwd = std::env::current_dir()?;
    let mut files: Vec<Sent> = Vec::new();
    for typed in &args.paths {
        let file = sent_file(&ctx.root, &cwd, typed)?;
        if !files.iter().any(|known| known.stored == file.stored) {
            files.push(file);
        }
    }
    let based_on = based_on_of(&ctx.root, &cwd, &args.based_on)?;

    let (items, unreadable) = store::all(&ctx.dossier);
    let warnings = warnings_of(&ctx, &items, &based_on)?;
    if !warnings.is_empty() {
        if who.is_llm || !term::is_terminal() {
            return Err(Error::new(
                Code::NeedsTerminal,
                format!(
                    "the delivery has warnings that a person at a terminal must confirm: {}",
                    warnings.join("; ")
                ),
            ));
        }
        show(&warnings);
        let answer = term::ask(QUESTION)?;
        if !answer.eq_ignore_ascii_case("yes") {
            return Ok(Output::with(
                "nothing changed",
                json!({"id": null, "delivered": false, "warnings": warnings}),
            ));
        }
    }

    let output = tx::write(&ctx, |tx| {
        let (mut exchange_file, exchange_bytes) =
            exchange::load(&tx.ctx().dossier, &tx.ctx().cfg.code)?;
        let key = if tx.ctx().dry_run {
            PLACEHOLDER.to_string()
        } else {
            ids::next_prefix(
                &tx.ctx().state,
                PREFIX,
                exchange::highest(&exchange_file, PREFIX),
            )?
        };

        let files_text = files
            .iter()
            .map(|file| format!("{}@{}", file.stored, file.checksum))
            .collect::<Vec<_>>()
            .join(", ");
        let mut fields = vec![
            pair("date", &date),
            pair("to", &to),
            pair("files", &files_text),
        ];
        if !based_on.is_empty() {
            let based_on_text = based_on
                .iter()
                .map(BasedOn::text)
                .collect::<Vec<_>>()
                .join(", ");
            fields.push(pair("based-on", &based_on_text));
        }
        if let Some(note) = &note {
            fields.push(pair("note", note));
        }
        exchange_file.add_block(&key, &fields);
        exchange_file.add_history(&key, tx.time(), &who, DELIVERED, "");
        let exchange_path = tx.ctx().dossier.join(exchange::FILE);
        tx.put_raw(
            exchange_path,
            exchange_bytes,
            exchange_file.text().into_bytes(),
            Change {
                id: exchange::ITEM.to_string(),
                event: DELIVERED.to_string(),
                detail: key.clone(),
            },
        );

        let message = format!("{key}: delivered {} files to {to}", files.len());
        let data = json!({
            "id": key,
            "date": date,
            "to": to,
            "files": files
                .iter()
                .map(|file| json!({"path": file.stored, "checksum": file.checksum}))
                .collect::<Vec<_>>(),
            "based_on": based_on.iter().map(BasedOn::json).collect::<Vec<_>>(),
            "warnings": warnings,
        });
        Ok(Output::with(message, data))
    })?;
    Ok(unreadable.iter().fold(output, |output, warning| {
        output.warn(shown_warning(&ctx.root, warning))
    }))
}

/// The warnings of a delivery, in order: for each task named, its files marked outdated or
/// resting on a wrong assumption and its entries that rest on one; for each file named, its
/// marks of those kinds in the file lists; then each approved assumption that is open and whose
/// Affects section names one of the things named.
fn warnings_of(ctx: &Ctx, items: &[Item], based_on: &[BasedOn]) -> Result<Vec<String>> {
    let mut warnings = Vec::new();
    for named in based_on {
        match named {
            BasedOn::Task(task) => task_warnings(ctx, items, task, &mut warnings)?,
            BasedOn::Path { stored, .. } => path_warnings(ctx, stored, &mut warnings)?,
        }
    }
    assumption_warnings(items, based_on, &mut warnings);
    Ok(warnings)
}

/// The warnings about a task: its files marked outdated or resting on a wrong assumption, and its
/// entries whose `rests-on` field names an assumption.
fn task_warnings(ctx: &Ctx, items: &[Item], task: &str, warnings: &mut Vec<String>) -> Result<()> {
    let (list, _) = filelist::load(&ctx.dossier, task)?;
    for key in list.keys() {
        for mark in filelist::marks(&ctx.root, &list, &key) {
            if let Some(words) = warning_words(&mark) {
                push(warnings, format!("{task}: {key} {words}"));
            }
        }
    }
    for item in items {
        let entry = &item.entry;
        let about_task = entry.field("task").is_some_and(|value| same(&value, task));
        let rests_on = entry.list("rests-on");
        if about_task && !rests_on.is_empty() {
            push(
                warnings,
                format!(
                    "{task}: {} {WRONG_ASSUMPTION}: {}",
                    entry.id(),
                    rests_on.join(", ")
                ),
            );
        }
    }
    Ok(())
}

/// The warnings about a file: its marks of the kinds that warn, in each file list that has it.
fn path_warnings(ctx: &Ctx, stored: &str, warnings: &mut Vec<String>) -> Result<()> {
    for task in filelist::tasks(&ctx.dossier) {
        let (list, _) = filelist::load(&ctx.dossier, &task)?;
        if !list.keys().iter().any(|key| key.as_str() == stored) {
            continue;
        }
        for mark in filelist::marks(&ctx.root, &list, stored) {
            if let Some(words) = warning_words(&mark) {
                push(warnings, format!("{stored} {words} (files of {task})"));
            }
        }
    }
    Ok(())
}

/// The warnings about assumptions: each approved assumption that is still open and whose Affects
/// section names a task or file named. The name is searched for in the text, without regard to
/// case.
fn assumption_warnings(items: &[Item], based_on: &[BasedOn], warnings: &mut Vec<String>) {
    for item in items {
        let entry = &item.entry;
        let open = entry.kind() == Some(Kind::Assumption)
            && entry.status() == Some(Status::Approved)
            && entry.field("state").as_deref() == Some(OPEN);
        if !open {
            continue;
        }
        let Some(affects) = entry.section("Affects") else {
            continue;
        };
        let affects = affects.to_lowercase();
        for named in based_on {
            if affects.contains(&named.name().to_lowercase()) {
                push(
                    warnings,
                    format!("{} is open and affects {}", entry.id(), named.name()),
                );
            }
        }
    }
}

/// The words of the warning for a mark of a file, or None for a mark that does not warn.
fn warning_words(mark: &str) -> Option<&'static str> {
    match mark {
        OUTDATED => Some("is outdated"),
        WRONG_ASSUMPTION => Some(WRONG_ASSUMPTION),
        _ => None,
    }
}

/// Adds a warning unless the same one is there already.
fn push(warnings: &mut Vec<String>, warning: String) {
    if !warnings.contains(&warning) {
        warnings.push(warning);
    }
}

/// Writes the warnings to standard error, the review before the question. A failed write is
/// ignored: the answer is what matters.
fn show(warnings: &[String]) {
    let mut stderr = std::io::stderr();
    for warning in warnings {
        let _ = writeln!(stderr, "warning: {warning}");
    }
}

/// The `--based-on` options, in order. A task ID is kept as the task ID in upper case; anything
/// else is a file, stored with its checksum (R-EXCH-2). An item given twice is kept once.
fn based_on_of(root: &Path, cwd: &Path, typed: &[String]) -> Result<Vec<BasedOn>> {
    let mut items: Vec<BasedOn> = Vec::new();
    for text in typed {
        let item = based_on_item(root, cwd, text)?;
        if !items.contains(&item) {
            items.push(item);
        }
    }
    Ok(items)
}

/// One `--based-on` option as a task or a file. Blank is refused.
fn based_on_item(root: &Path, cwd: &Path, typed: &str) -> Result<BasedOn> {
    let typed = typed.trim();
    if typed.is_empty() {
        return Err(bad("--based-on needs a task such as T-042, or a file"));
    }
    if is_task_id(typed) {
        return Ok(BasedOn::Task(typed.to_uppercase()));
    }
    let file = sent_file(root, cwd, typed)?;
    Ok(BasedOn::Path {
        stored: file.stored,
        checksum: file.checksum,
    })
}

/// Whether `text` is a task ID: `T-` and digits, in any case.
fn is_task_id(text: &str) -> bool {
    text.split_once('-').is_some_and(|(head, digits)| {
        head.eq_ignore_ascii_case("T")
            && !digits.is_empty()
            && digits.bytes().all(|byte| byte.is_ascii_digit())
    })
}

/// The file at `typed` (relative to `cwd`, or absolute) as a file that was sent. Its stored path
/// must be inside the project folder, it must exist, and it must be a file, not a folder.
fn sent_file(root: &Path, cwd: &Path, typed: &str) -> Result<Sent> {
    if typed.trim().is_empty() {
        return Err(bad("a file is named by its path"));
    }
    let stored = paths::stored(root, cwd, typed)?;
    let fields = filelist::describe(root, &stored)?;
    let checksum = match field(&fields, "checksum") {
        Some(checksum) => checksum.to_string(),
        None => {
            return Err(bad(format!(
                "{stored} is a folder; a delivery records files with their checksums"
            )));
        }
    };
    Ok(Sent { stored, checksum })
}

/// The `--to` option, trimmed. Blank is refused.
fn to_of(text: &str) -> Result<String> {
    trimmed(Some(text)).ok_or_else(|| bad("--to needs who the files went to, such as the customer"))
}

/// The `--date` option as YYYY-MM-DD, or today's date when it is not given. Code::BadValue when it
/// is not a date of that form.
fn date_of(text: Option<&str>) -> Result<String> {
    match text {
        None => Ok(time::date(&time::now()).to_string()),
        Some(text) => NaiveDate::parse_from_str(text.trim(), "%Y-%m-%d")
            .map(|day| day.format("%Y-%m-%d").to_string())
            .map_err(|_| {
                bad(format!(
                    "--date needs a date such as 2026-11-21, not \"{text}\""
                ))
            }),
    }
}

/// Whether two IDs or names are the same, ignoring case and surrounding white space.
fn same(a: &str, b: &str) -> bool {
    a.trim().to_lowercase() == b.trim().to_lowercase()
}

/// A warning of reading the dossier with its path shown relative to the project folder, as
/// `brief` shows it.
fn shown_warning(root: &Path, warning: &str) -> String {
    let root_text = root.display().to_string();
    let Some(rest) = warning.strip_prefix(root_text.as_str()) else {
        return warning.to_string();
    };
    let rest = rest.strip_prefix(['\\', '/']).unwrap_or(rest);
    match rest.split_once(": ") {
        Some((path, message)) => format!("{}: {message}", path.replace(MAIN_SEPARATOR, "/")),
        None => warning.to_string(),
    }
}

/// The text with its ends trimmed, or None when it is not given or blank.
fn trimmed(text: Option<&str>) -> Option<String> {
    text.map(str::trim)
        .filter(|text| !text.is_empty())
        .map(str::to_string)
}

/// A failure for a value the person gave, Code::BadValue.
fn bad(message: impl Into<String>) -> Error {
    Error::new(Code::BadValue, message)
}

/// The value of the field `name` in a list of fields, if there is one.
fn field<'a>(fields: &'a [(String, String)], name: &str) -> Option<&'a str> {
    fields
        .iter()
        .find(|(field, _)| field == name)
        .map(|(_, value)| value.as_str())
}

/// A field as a pair of owned strings.
fn pair(name: &str, value: &str) -> (String, String) {
    (name.to_string(), value.to_string())
}
