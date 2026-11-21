//! The `drafts` command: the entries waiting for approval, oldest first, with who proposed each,
//! from which task and how many days ago (R-DRAFT-3). `--mine` keeps those proposed for the
//! calling person. Drafts older than the `draft_days` setting are marked `(old)` (R-DRAFT-8).
//! Nothing is changed.

use crate::cli::Global;
use crate::ctx::Ctx;
use crate::entry::{Entry, Kind, Status};
use crate::error::Result;
use crate::output::Output;
use crate::store::{self, Item};
use crate::time;
use chrono::{DateTime, FixedOffset};
use serde_json::{Value, json};
use std::path::{MAIN_SEPARATOR, Path};

/// The arguments of `drafts`.
#[derive(clap::Args, Debug)]
pub struct Args {
    /// Only the drafts proposed for the calling person
    #[arg(long)]
    pub mine: bool,
}

/// Runs `drafts`: the draft entries, oldest `created` first (the instant, not the text of the
/// stamp), with the warnings of reading the dossier. A draft whose `created` is missing or is not
/// a time stamp comes last and has no age.
pub fn run(args: &Args, g: &Global) -> Result<Output> {
    let ctx = Ctx::load(g, "drafts")?;
    let mine = if args.mine {
        Some(ctx.writer()?.person.clone())
    } else {
        None
    };
    let (items, warnings) = store::all(&ctx.dossier);
    let mut drafts: Vec<&Item> = items
        .iter()
        .filter(|item| item.entry.status() == Some(Status::Draft))
        .filter(|item| mine.as_deref().is_none_or(|person| by(&item.entry, person)))
        .collect();
    drafts.sort_by_key(|item| {
        let created = created_at(&item.entry);
        (created.is_none(), created)
    });

    let now = time::now();
    let mut lines: Vec<String> = drafts.iter().map(|item| line(&ctx, item, &now)).collect();
    lines.push(count(drafts.len()));
    let data: Vec<Value> = drafts.iter().map(|item| row(item, &now)).collect();

    let mut output = Output::with(lines.join("\n"), json!(data));
    for warning in &warnings {
        output = output.warn(shown_warning(&ctx.root, warning));
    }
    Ok(output)
}

/// One line of the list: `<ID>  <kind>  <title>  by <author>[ via <via>][  task <task>]  <n> days`,
/// with `  (old)` after the age when the draft is older than `draft_days`.
fn line(ctx: &Ctx, item: &Item, now: &str) -> String {
    let entry = &item.entry;
    let mut text = format!(
        "{}  {}  {}  by {}",
        entry.id(),
        entry.kind().map_or("-", Kind::name),
        entry.title(),
        value(entry, "author").unwrap_or_else(|| "-".to_string())
    );
    if let Some(via) = value(entry, "via") {
        text.push_str(&format!(" via {via}"));
    }
    if let Some(task) = value(entry, "task") {
        text.push_str(&format!("  task {task}"));
    }
    match age(entry, now) {
        Some(days) => {
            text.push_str(&format!("  {}", days_text(days)));
            if days > ctx.cfg.draft_days {
                text.push_str("  (old)");
            }
        }
        None => text.push_str("  age unknown"),
    }
    text
}

/// The data of one draft: its ID, kind, title, author, the LLM that wrote it, the task, the
/// creation time and the age in days. A value the entry does not have is `null`.
fn row(item: &Item, now: &str) -> Value {
    let entry = &item.entry;
    json!({
        "id": entry.id(),
        "kind": entry.kind().map(Kind::name),
        "title": entry.title(),
        "author": value(entry, "author"),
        "via": value(entry, "via"),
        "task": value(entry, "task"),
        "created": created(entry),
        "age_days": age(entry, now),
    })
}

/// Whether the entry was proposed for `person`: its `author` field, ignoring case.
fn by(entry: &Entry, person: &str) -> bool {
    value(entry, "author").is_some_and(|author| author.to_lowercase() == person.to_lowercase())
}

/// The value of a field, when the entry has it and it is not empty.
fn value(entry: &Entry, key: &str) -> Option<String> {
    entry.field(key).filter(|value| !value.is_empty())
}

/// The `created` field, when it is a time stamp in the format of format.md §7.
fn created(entry: &Entry) -> Option<String> {
    entry.field("created").filter(|stamp| time::valid(stamp))
}

/// The `created` field as a point in time, for putting the drafts in order.
fn created_at(entry: &Entry) -> Option<DateTime<FixedOffset>> {
    created(entry).and_then(|stamp| DateTime::parse_from_rfc3339(&stamp).ok())
}

/// Whole days from `created` to `now`, or None when `created` is missing or not a time stamp.
fn age(entry: &Entry, now: &str) -> Option<i64> {
    created(entry).map(|stamp| time::days_between(&stamp, now))
}

/// `1 day`, or `<n> days`.
fn days_text(days: i64) -> String {
    if days == 1 {
        "1 day".to_string()
    } else {
        format!("{days} days")
    }
}

/// `1 draft`, or `<n> drafts`.
fn count(n: usize) -> String {
    if n == 1 {
        "1 draft".to_string()
    } else {
        format!("{n} drafts")
    }
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
