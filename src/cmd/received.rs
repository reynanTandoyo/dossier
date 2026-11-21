//! The `received` command (docs/design.md section 9, R-EXCH-1): records the files that came from
//! the customer or another party as an RCV record in exchange.md, with the checksum of each file.
//! With `--task` the files are also attached to that task's list, as `attach` does; a file that
//! the list has already is left as it is. Anyone may record what was received, an LLM too.

use crate::cli::Global;
use crate::ctx::Ctx;
use crate::error::{Code, Error, Result};
use crate::exchange;
use crate::filelist;
use crate::ids;
use crate::names::Who;
use crate::output::Output;
use crate::paths;
use crate::time;
use crate::tx::{self, Change, Tx};
use chrono::NaiveDate;
use serde_json::json;
use std::path::Path;

/// The prefix of received records: RCV-001, RCV-002, ... (format.md §17).
const PREFIX: &str = "RCV";

/// The key a dry run uses in place of a new number, as `create` uses `DEC-NEW` for a new entry.
const PLACEHOLDER: &str = "RCV-NEW";

/// The history event of a received record, and the event of a file attached with it.
const RECEIVED: &str = "received";
const ATTACHED: &str = "attached";

/// The roles `--role` accepts for a received file on a task.
const ROLES: [&str; 2] = ["description", "reference"];

/// The role of a received file when `--role` is not given.
const DEFAULT_ROLE: &str = "reference";

/// The arguments of `received`.
#[derive(clap::Args, Debug)]
pub struct Args {
    /// The files received, as paths relative to the current folder or absolute (at least one)
    #[arg(required = true, value_name = "path")]
    pub paths: Vec<String>,
    /// Who sent the files, such as the customer
    #[arg(long)]
    pub from: String,
    /// The date the files came, YYYY-MM-DD (default: today)
    #[arg(long, value_name = "date")]
    pub date: Option<String>,
    /// The task whose file list gets the files, such as T-042
    #[arg(long)]
    pub task: Option<String>,
    /// The role of the files on the task: description or reference (default: reference)
    #[arg(long, value_name = "role")]
    pub role: Option<String>,
    /// A note on what was received
    #[arg(long)]
    pub note: Option<String>,
}

/// A file received: its path as stored, the fields that describe it (`filelist::describe`), and
/// its checksum.
struct Received {
    stored: String,
    fields: Vec<(String, String)>,
    checksum: String,
}

/// Runs `received`: checks the options and the files, then in one transaction writes the RCV
/// record and, with `--task`, attaches the new files to that task's list. The first output line
/// is `<RCV-ID>: received <n> files from <who>`.
pub fn run(args: &Args, g: &Global) -> Result<Output> {
    let ctx = Ctx::load(g, "received")?;
    let who = ctx.writer()?.clone();
    let from = from_of(&args.from)?;
    let date = date_of(args.date.as_deref())?;
    let role = role_of(args.role.as_deref())?;
    let task = task_of(args.task.as_deref())?;
    let note = trimmed(args.note.as_deref());

    // The files are read before the transaction, so a missing or refused file writes nothing.
    let cwd = std::env::current_dir()?;
    let mut files: Vec<Received> = Vec::new();
    for typed in &args.paths {
        let file = received_file(&ctx.root, &cwd, typed)?;
        if !files.iter().any(|known| known.stored == file.stored) {
            files.push(file);
        }
    }
    let files_text = files
        .iter()
        .map(|file| format!("{}@{}", file.stored, file.checksum))
        .collect::<Vec<_>>()
        .join(", ");

    tx::write(&ctx, |tx| {
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

        let mut fields = vec![
            pair("date", &date),
            pair("from", &from),
            pair("files", &files_text),
        ];
        if let Some(note) = &note {
            fields.push(pair("note", note));
        }
        exchange_file.add_block(&key, &fields);
        exchange_file.add_history(&key, tx.time(), &who, RECEIVED, "");
        let exchange_path = tx.ctx().dossier.join(exchange::FILE);
        tx.put_raw(
            exchange_path,
            exchange_bytes,
            exchange_file.text().into_bytes(),
            Change {
                id: exchange::ITEM.to_string(),
                event: RECEIVED.to_string(),
                detail: key.clone(),
            },
        );

        let attached = match &task {
            Some(task) => attach(tx, task, &role, &files, &who)?,
            None => Vec::new(),
        };
        let message = format!("{key}: received {} files from {from}", files.len());
        let data = json!({
            "id": key,
            "date": date,
            "from": from,
            "files": files
                .iter()
                .map(|file| json!({"path": file.stored, "checksum": file.checksum}))
                .collect::<Vec<_>>(),
            "task": task,
            "attached": attached,
        });
        Ok(Output::with(message, data))
    })
}

/// Adds the files to the list of `task`, as `attach` does: a new block with the role, the fields
/// that describe the file, and an `attached` history line. A file that already has a block in the
/// list is left as it is. The list is written only when a file was attached. Returns the stored
/// paths attached now.
fn attach(
    tx: &mut Tx,
    task: &str,
    role: &str,
    files: &[Received],
    who: &Who,
) -> Result<Vec<String>> {
    let dossier = tx.ctx().dossier.clone();
    let (mut list, bytes) = filelist::load(&dossier, task)?;
    let known = list.keys();
    let mut attached = Vec::new();
    for file in files {
        if known.contains(&file.stored) {
            continue;
        }
        let mut fields = vec![pair("role", role)];
        fields.extend(file.fields.iter().cloned());
        list.add_block(&file.stored, &fields);
        list.add_history(&file.stored, tx.time(), who, ATTACHED, "");
        attached.push(file.stored.clone());
    }
    if !attached.is_empty() {
        tx.put_raw(
            filelist::path(&dossier, task),
            bytes,
            list.text().into_bytes(),
            Change {
                id: filelist::item(task),
                event: ATTACHED.to_string(),
                detail: attached.join(", "),
            },
        );
    }
    Ok(attached)
}

/// The file at `typed` (relative to `cwd`, or absolute) as a received file. Its stored path must
/// be inside the project folder, it must exist, and it must be a file, not a folder.
fn received_file(root: &Path, cwd: &Path, typed: &str) -> Result<Received> {
    if typed.trim().is_empty() {
        return Err(bad("a file is named by its path"));
    }
    let stored = paths::stored(root, cwd, typed)?;
    let fields = filelist::describe(root, &stored)?;
    let checksum = match field(&fields, "checksum") {
        Some(checksum) => checksum.to_string(),
        None => {
            return Err(bad(format!("{stored} is a folder; received records files")));
        }
    };
    Ok(Received {
        stored,
        fields,
        checksum,
    })
}

/// The `--from` option, trimmed. Blank is refused.
fn from_of(text: &str) -> Result<String> {
    trimmed(Some(text)).ok_or_else(|| bad("--from needs who sent the files, such as the customer"))
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

/// The `--role` option, lower-cased: `description` or `reference`, `reference` when it is not
/// given. Code::BadValue for any other role.
fn role_of(text: Option<&str>) -> Result<String> {
    let role = match text {
        None => DEFAULT_ROLE.to_string(),
        Some(text) => text.trim().to_lowercase(),
    };
    if ROLES.contains(&role.as_str()) {
        Ok(role)
    } else {
        Err(bad(format!(
            "--role accepts description or reference, not \"{}\"",
            text.unwrap_or_default().trim()
        )))
    }
}

/// The `--task` option, upper-cased, as the task ID that names its file list (files/<TASK>.md).
/// A blank task is no task. Only letters, digits and `-` are allowed, so that the list stays in
/// the files folder; Code::BadValue otherwise.
fn task_of(text: Option<&str>) -> Result<Option<String>> {
    let Some(task) = trimmed(text).map(|task| task.to_uppercase()) else {
        return Ok(None);
    };
    if task.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
        Ok(Some(task))
    } else {
        Err(bad(format!(
            "--task needs a task ID such as T-042, not \"{task}\""
        )))
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
