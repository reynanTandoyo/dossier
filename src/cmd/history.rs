//! The `history` command: what happened to an entry, or to a task, in time order (R-READ-6,
//! R-HIST-2). An entry ID gives the entry's history lines, then its snapshots, one per earlier
//! version. Anything else is a task: one line per entry whose `task` field names it, oldest
//! first. Nothing is changed.

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

/// The entries whose `task` field names `task`, ignoring case, oldest first. One line each:
/// `<created>  <author>  <ID>  <title>`. The data is the task and one object per entry.
fn task_history(task: &str, items: &[Item]) -> Output {
    let mut chosen: Vec<&Item> = items
        .iter()
        .filter(|item| names_task(&item.entry, task))
        .collect();
    // Oldest first by the instant the created time names, so that different offsets compare
    // right; an entry without a usable created time goes last. The sort is stable, so entries
    // created at the same instant keep the ID order of `store::all`.
    chosen.sort_by_key(|item| {
        let created = created_instant(&item.entry);
        (created.is_none(), created)
    });

    let text = if chosen.is_empty() {
        format!("nothing recorded for {task}")
    } else {
        chosen
            .iter()
            .map(|item| task_line(&item.entry))
            .collect::<Vec<String>>()
            .join("\n")
    };
    let entries: Vec<Value> = chosen
        .iter()
        .map(|item| {
            json!({
                "id": item.entry.id(),
                "title": item.entry.title(),
                "author": item.entry.field("author"),
                "created": item.entry.field("created"),
            })
        })
        .collect();
    Output::with(text, json!({"task": task, "entries": entries}))
}

/// Whether the entry's `task` field names `task`, ignoring case (as the filter does).
fn names_task(entry: &Entry, task: &str) -> bool {
    entry
        .field("task")
        .is_some_and(|value| value.trim().to_lowercase() == task.to_lowercase())
}

/// The instant of an entry's created time (format.md §7), when it has one that parses.
fn created_instant(entry: &Entry) -> Option<DateTime<FixedOffset>> {
    entry
        .field("created")
        .and_then(|stamp| DateTime::parse_from_rfc3339(&stamp).ok())
}

/// One task line: `<created>  <author>  <ID>  <title>`, with `-` for a missing created time or
/// author.
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
