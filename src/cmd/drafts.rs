//! The `drafts` command: the entries waiting for approval, oldest first, with who proposed each,
//! from which task and how many days ago (R-DRAFT-3). `--mine` keeps those proposed for the
//! calling person. Drafts older than the `draft_days` setting are marked `(old)` (R-DRAFT-8).
//! Imported drafts are marked `[imported]`; `--imported` lists only them, grouped under the
//! document each was drafted from (R-IMPORT-6). Nothing is changed.

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

/// The heading of the drafts that have no source document under `--imported`.
const NO_DOCUMENT: &str = "(no source document)";

/// The arguments of `drafts`.
#[derive(clap::Args, Debug)]
pub struct Args {
    /// Only the drafts proposed for the calling person
    #[arg(long)]
    pub mine: bool,
    /// Only the imported drafts, grouped under the document each was drafted from (R-IMPORT-6)
    #[arg(long)]
    pub imported: bool,
}

/// Runs `drafts`: the draft entries, oldest `created` first (the instant, not the text of the
/// stamp), with the warnings of reading the dossier. A draft whose `created` is missing or is not
/// a time stamp comes last and has no age. With `--imported`, the imported drafts only, grouped by
/// their `source-document` (see `grouped`).
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
        .filter(|item| !args.imported || imported(&item.entry))
        .filter(|item| mine.as_deref().is_none_or(|person| by(&item.entry, person)))
        .collect();
    drafts.sort_by_key(|item| {
        let created = created_at(&item.entry);
        (created.is_none(), created)
    });

    let now = time::now();
    let (mut lines, shown): (Vec<String>, Vec<&Item>) = if args.imported {
        grouped(&ctx, &drafts, &now)
    } else {
        let lines = drafts
            .iter()
            .map(|item| line(&ctx, item, &now, true))
            .collect();
        (lines, drafts)
    };
    lines.push(count(shown.len()));
    let data: Vec<Value> = shown.iter().map(|item| row(item, &now)).collect();

    let mut output = Output::with(lines.join("\n"), json!(data));
    for warning in &warnings {
        output = output.warn(shown_warning(&ctx.root, warning));
    }
    Ok(output)
}

/// The lines of `--imported`, and the drafts in the order they are listed: the drafts grouped
/// under a `## <source-document>` heading each, a blank line between groups, the groups in the
/// order of their document name with the drafts that have no document last, and the drafts of a
/// group oldest first.
fn grouped<'a>(ctx: &Ctx, drafts: &[&'a Item], now: &str) -> (Vec<String>, Vec<&'a Item>) {
    let mut groups: Vec<(Option<String>, Vec<&'a Item>)> = Vec::new();
    for &item in drafts {
        let document = value(&item.entry, "source-document");
        match groups.iter().position(|(key, _)| *key == document) {
            Some(index) => groups[index].1.push(item),
            None => groups.push((document, vec![item])),
        }
    }
    groups.sort_by(|(a, _), (b, _)| a.is_none().cmp(&b.is_none()).then_with(|| a.cmp(b)));

    let mut lines = Vec::new();
    let mut shown = Vec::new();
    for (index, (document, members)) in groups.iter().enumerate() {
        if index > 0 {
            lines.push(String::new());
        }
        lines.push(format!("## {}", document.as_deref().unwrap_or(NO_DOCUMENT)));
        for &item in members {
            lines.push(line(ctx, item, now, false));
            shown.push(item);
        }
    }
    (lines, shown)
}

/// One line of the list: `<ID>  <kind>  <title>  by <author>[ via <via>][  task <task>]  <n> days`,
/// with `  (old)` after the age when the draft is older than `draft_days`, and `  [imported]` after
/// that when `mark_imported` is set and the draft is imported (R-IMPORT-5).
fn line(ctx: &Ctx, item: &Item, now: &str, mark_imported: bool) -> String {
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
    if mark_imported && imported(entry) {
        text.push_str("  [imported]");
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

/// Whether the entry is an imported draft: `imported: yes` (format.md §23, R-IMPORT-5).
fn imported(entry: &Entry) -> bool {
    entry
        .field("imported")
        .is_some_and(|text| text.eq_ignore_ascii_case("yes"))
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
