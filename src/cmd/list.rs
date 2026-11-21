//! The `list` command: one line per entry the filter chooses, then a count (R-READ-2). `--wide`
//! adds who created an entry and when, and who approved it and when (R-HIST-6).

use crate::approval::{self, Mark};
use crate::cli::Global;
use crate::ctx::Ctx;
use crate::entry::{Entry, Kind, Status};
use crate::error::Result;
use crate::filter::Filter;
use crate::output::Output;
use crate::store::{self, Item};
use serde_json::json;
use std::path::{MAIN_SEPARATOR, Path};

/// The arguments of `list`.
#[derive(clap::Args, Debug)]
pub struct Args {
    #[command(flatten)]
    pub filter: Filter,
    /// Also show when and by whom each entry was created and approved
    #[arg(long)]
    pub wide: bool,
}

/// Runs `list`: the entries the filter chooses, in ID order, with the warnings of reading the
/// dossier. The data is one object per entry.
pub fn run(args: &Args, g: &Global) -> Result<Output> {
    let ctx = Ctx::load(g, "list")?;
    let (items, warnings) = store::all(&ctx.dossier);
    let chosen: Vec<&Item> = items
        .iter()
        .filter(|item| args.filter.matches(item))
        .collect();

    let mut lines: Vec<String> = chosen
        .iter()
        .map(|item| line(&ctx, item, args.wide))
        .collect();
    lines.push(count(chosen.len()));
    let data: Vec<serde_json::Value> = chosen.iter().map(|item| row(item)).collect();

    let output = Output::with(lines.join("\n"), json!(data));
    Ok(warnings.iter().fold(output, |output, warning| {
        output.warn(shown_warning(&ctx.root, warning))
    }))
}

/// One line of the list: `<ID>  <kind>  <mark> <title>`, the mark only when the entry is not
/// plainly approved. With `wide`, the creation and approval follow in parentheses.
fn line(ctx: &Ctx, item: &Item, wide: bool) -> String {
    let entry = &item.entry;
    let mark = mark(ctx, item).map_or_else(String::new, |mark| format!("{mark} "));
    let mut text = format!(
        "{}  {}  {mark}{}",
        entry.id(),
        entry.kind().map_or("-", Kind::name),
        entry.title()
    );
    if wide {
        text.push_str(&format!("  ({})", created_and_approved(entry)));
    }
    text
}

/// `created <time> by <author>`, and `, approved <time> by <person>` when the entry has an
/// approval. A missing value is shown as `-`.
fn created_and_approved(entry: &Entry) -> String {
    let created = entry.field("created").unwrap_or_else(|| "-".to_string());
    let author = entry.field("author").unwrap_or_else(|| "-".to_string());
    let mut text = format!("created {created} by {author}");
    if let Some(approved) = entry.field("approved") {
        text.push_str(&format!(", approved {approved}"));
    }
    text
}

/// The mark of an entry (design.md §5): `[draft]`, `[rejected]`, `[superseded by <ID>]`,
/// `[retired]`, and `[changed after approval]` for an approved lesson whose text differs from
/// the approved text. An approved entry without a change has no mark.
fn mark(ctx: &Ctx, item: &Item) -> Option<String> {
    let entry = &item.entry;
    match entry.status()? {
        Status::Draft => Some("[draft]".to_string()),
        Status::Rejected => Some("[rejected]".to_string()),
        Status::Superseded => Some(match entry.field("superseded-by") {
            Some(by) => format!("[superseded by {by}]"),
            None => "[superseded]".to_string(),
        }),
        Status::Retired => Some("[retired]".to_string()),
        Status::Approved => {
            let changed = entry.kind() == Some(Kind::Lesson)
                && matches!(
                    approval::mark(&ctx.state, &entry.id(), &item.raw),
                    Mark::Changed
                );
            changed.then(|| "[changed after approval]".to_string())
        }
    }
}

/// The data of one entry: its ID, kind, status, title, author, creation and approval. A value
/// the entry does not have is `null`.
fn row(item: &Item) -> serde_json::Value {
    let entry = &item.entry;
    json!({
        "id": entry.id(),
        "kind": entry.kind().map(Kind::name),
        "status": entry.status().map(Status::name),
        "title": entry.title(),
        "author": entry.field("author"),
        "created": entry.field("created"),
        "approved": entry.field("approved"),
    })
}

/// `1 entry`, or `<n> entries`.
fn count(n: usize) -> String {
    if n == 1 {
        "1 entry".to_string()
    } else {
        format!("{n} entries")
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
