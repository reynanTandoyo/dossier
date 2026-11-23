//! The `history` command: what happened to an entry, or to a task, in time order (R-READ-6,
//! R-HIST-2, R-INV-6). An entry ID gives the entry's history lines, then its snapshots, one per
//! earlier version. Anything else is a task: one line per entry whose `task` field names it, and
//! one per attempt of an investigation whose `task` names it, oldest first. Nothing is changed.

use crate::attempts::{self, Attempt};
use crate::cli::Global;
use crate::ctx::Ctx;
use crate::entry::{Entry, Kind};
use crate::error::{Code, Error, Result};
use crate::output::Output;
use crate::snapshot::{self, Snap};
use crate::store::{self, Item};
use chrono::{DateTime, FixedOffset};
use serde_json::{Value, json};
use std::path::{MAIN_SEPARATOR, Path};

/// The arguments of `history`.
#[derive(clap::Args, Debug)]
pub struct Args {
    /// An entry ID, such as DEC-012 (any case, with or without a place prefix), or a task, such
    /// as T-042
    pub target: String,
}

/// One row of a task's history: the line the text output shows, the object the JSON output shows
/// and the instant the row is sorted by.
struct Row {
    instant: Option<DateTime<FixedOffset>>,
    line: String,
    data: Value,
}

/// Runs `history`. A target that `Kind::of_id` takes for an entry ID gives that entry's history
/// and snapshots; any other target is a task. Problems in other files of the dossier are passed
/// on as warnings.
pub fn run(args: &Args, g: &Global) -> Result<Output> {
    let ctx = Ctx::load(g, "history")?;
    let target = args.target.trim();
    if target.is_empty() {
        return Err(Error::new(
            Code::BadValue,
            "history needs an entry ID or a task, such as DEC-012 or T-042",
        ));
    }
    let (items, warnings) = store::all(&ctx.dossier);
    let output = if Kind::of_id(target).is_some() {
        entry_history(&ctx, target)?
    } else {
        task_history(target, &items)
    };
    Ok(warnings.iter().fold(output, |output, warning| {
        output.warn(shown_warning(&ctx.root, warning))
    }))
}

/// The history of one entry: its history lines in the order they were written, then, when there
/// are any, a blank line; then `snapshots:` and one line per snapshot, or `none`. The data is the
/// ID, the history lines and the snapshots.
fn entry_history(ctx: &Ctx, target: &str) -> Result<Output> {
    let item = store::get(&ctx.dossier, target)?;
    let id = item.entry.id();
    let history = item.entry.history();
    let snaps = snapshot::list(&ctx.state, &id)?;

    let mut lines = history.clone();
    if !lines.is_empty() {
        lines.push(String::new());
    }
    lines.push("snapshots:".to_string());
    if snaps.is_empty() {
        lines.push("none".to_string());
    } else {
        lines.extend(snaps.iter().map(snap_line));
    }
    let data = json!({
        "id": id,
        "history": history,
        "snapshots": snaps.iter().map(snap_data).collect::<Vec<Value>>(),
    });
    Ok(Output::with(lines.join("\n"), data))
}

/// One snapshot line: `n <n>  <time>  <name>  <command>`.
fn snap_line(snap: &Snap) -> String {
    format!(
        "n {}  {}  {}  {}",
        snap.n, snap.time, snap.name, snap.command
    )
}

/// The data of one snapshot: its number, time, name and command.
fn snap_data(snap: &Snap) -> Value {
    json!({
        "n": snap.n,
        "time": snap.time,
        "name": snap.name,
        "command": snap.command,
    })
}

/// The rows of a task, oldest first: the entries whose `task` field names it, and the attempts of
/// the investigations whose attempt `task` names it. An investigation's own row is one of the
/// entry rows; each of its attempts that names the task is a row of its own. The rows are sorted
/// together by the instant each one names; a row without a usable time goes last. The sort is
/// stable, so rows at the same instant keep the order of `store::all` (entries by ID), with the
/// attempts after the entries. The text has one line per row; the data is the task and one
/// object per row.
fn task_history(task: &str, items: &[Item]) -> Output {
    let mut rows: Vec<Row> = items
        .iter()
        .filter(|item| names_task(&item.entry, task))
        .map(|item| entry_row(&item.entry))
        .collect();
    rows.extend(
        items
            .iter()
            .flat_map(|item| attempt_rows(&item.entry, task)),
    );
    rows.sort_by_key(|row| (row.instant.is_none(), row.instant));

    let text = if rows.is_empty() {
        format!("nothing recorded for {task}")
    } else {
        rows.iter()
            .map(|row| row.line.as_str())
            .collect::<Vec<&str>>()
            .join("\n")
    };
    let entries: Vec<Value> = rows.into_iter().map(|row| row.data).collect();
    Output::with(text, json!({"task": task, "entries": entries}))
}

/// The row of an entry: `<created>  <author>  <ID>  <title>`. Its JSON object has `attempt` null.
fn entry_row(entry: &Entry) -> Row {
    Row {
        instant: created_instant(entry),
        line: task_line(entry),
        data: json!({
            "id": entry.id(),
            "title": entry.title(),
            "author": entry.field("author"),
            "created": entry.field("created"),
            "attempt": null,
        }),
    }
}

/// The rows of the attempts of an investigation that name `task`, ignoring case, in the order
/// they were recorded. An entry that is not an investigation has no attempts.
fn attempt_rows(entry: &Entry, task: &str) -> Vec<Row> {
    let id = entry.id();
    attempts::of(entry)
        .into_iter()
        .filter(|attempt| {
            attempt
                .task
                .as_deref()
                .is_some_and(|named| same_task(named, task))
        })
        .map(|attempt| attempt_row(&id, &attempt))
        .collect()
}

/// The row of one attempt: `<time>  <who>  <ID> attempt <n>  <text>`. Its JSON object gives the
/// investigation's ID, the text of the attempt as the title, who and when as author and created,
/// and the number of the attempt.
fn attempt_row(id: &str, attempt: &Attempt) -> Row {
    Row {
        instant: instant_of(&attempt.time),
        line: format!(
            "{}  {}  {} attempt {}  {}",
            attempt.time, attempt.who, id, attempt.n, attempt.text
        ),
        data: json!({
            "id": id,
            "title": attempt.text,
            "author": attempt.who,
            "created": attempt.time,
            "attempt": attempt.n,
        }),
    }
}

/// Whether the entry's `task` field names `task`, ignoring case (as the filter does).
fn names_task(entry: &Entry, task: &str) -> bool {
    entry
        .field("task")
        .is_some_and(|named| same_task(&named, task))
}

/// Whether `named` is `task`, ignoring case and the spaces around `named`.
fn same_task(named: &str, task: &str) -> bool {
    named.trim().to_lowercase() == task.to_lowercase()
}

/// The instant of an entry's created time (format.md §7), when it has one that parses.
fn created_instant(entry: &Entry) -> Option<DateTime<FixedOffset>> {
    entry.field("created").and_then(|stamp| instant_of(&stamp))
}

/// The instant a time stamp (format.md §7) names, when it parses. Different offsets compare right.
fn instant_of(stamp: &str) -> Option<DateTime<FixedOffset>> {
    DateTime::parse_from_rfc3339(stamp).ok()
}

/// One task line of an entry: `<created>  <author>  <ID>  <title>`, with `-` for a missing created
/// time or author.
fn task_line(entry: &Entry) -> String {
    format!(
        "{}  {}  {}  {}",
        entry.field("created").unwrap_or_else(|| "-".to_string()),
        entry.field("author").unwrap_or_else(|| "-".to_string()),
        entry.id(),
        entry.title()
    )
}

/// A warning of `store::all` (`<path>: <message>`) with its path shown relative to the project
/// folder, with `/` between the parts. A warning that does not start in the project folder is
/// returned as it is.
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
