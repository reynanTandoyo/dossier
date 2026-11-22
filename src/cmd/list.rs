//! The `list` command: one line per entry the filter chooses, then a count (R-READ-2). `--wide`
//! adds who created an entry and when, and who approved it and when (R-HIST-6).

use crate::approval::{self, Mark};
use crate::cli::Global;
use crate::cmd::show::not_tested_with;
use crate::ctx::Ctx;
use crate::entry::{Entry, Kind, Status};
use crate::error::Result;
use crate::filesum;
use crate::filter::Filter;
use crate::output::Output;
use crate::store::{self, Item};
use crate::time;
use chrono::NaiveDate;
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

    let now = time::now();
    let today = time::date(&now);
    let mut lines: Vec<String> = chosen
        .iter()
        .map(|item| line(&ctx, item, today, args.wide))
        .collect();
    lines.push(count(chosen.len()));
    let data: Vec<serde_json::Value> = chosen.iter().map(|item| row(item)).collect();

    let output = Output::with(lines.join("\n"), json!(data));
    Ok(warnings.iter().fold(output, |output, warning| {
        output.warn(shown_warning(&ctx.root, warning))
    }))
}

/// One line of the list: `<ID>  <kind>  <marks> <title>`, the marks only when there are some (see
/// `mark`). A how-to or a tool adds what it is for and what it was tested with (see
/// `purpose_and_tests`). With `wide`, the creation and approval follow in parentheses. `today` is
/// the day of the run, YYYY-MM-DD.
fn line(ctx: &Ctx, item: &Item, today: &str, wide: bool) -> String {
    let entry = &item.entry;
    let mark = mark(ctx, item, today).map_or_else(String::new, |mark| format!("{mark} "));
    let mut text = format!(
        "{}  {}  {mark}{}",
        entry.id(),
        entry.kind().map_or("-", Kind::name),
        entry.title()
    );
    text.push_str(&purpose_and_tests(entry));
    if wide {
        text.push_str(&format!("  ({})", created_and_approved(entry)));
    }
    text
}

/// For a how-to or a tool: `  — <first line of Purpose>` and `  (tested with <first line of Tested
/// with>)`, each left out when its section is missing or empty (R-USE-5, R-USE-10). Other kinds
/// give nothing.
fn purpose_and_tests(entry: &Entry) -> String {
    if !matches!(entry.kind(), Some(Kind::Howto | Kind::Tool)) {
        return String::new();
    }
    let mut text = String::new();
    if let Some(purpose) = first_line(entry, "Purpose") {
        text.push_str(&format!("  — {purpose}"));
    }
    if let Some(tested) = first_line(entry, "Tested with") {
        text.push_str(&format!("  (tested with {tested})"));
    }
    text
}

/// The first line of the section `name`, trimmed. None when the section is missing or its first
/// line is empty.
fn first_line(entry: &Entry, name: &str) -> Option<String> {
    let body = entry.section(name)?;
    let first = body.lines().next()?.trim();
    (!first.is_empty()).then(|| first.to_string())
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

/// The marks of an entry, joined with a space, in the order `show` gives them (cmd/show.rs): its
/// status mark (design.md §5), `[due for review]` (R-LIFE-7), `[imported]` (R-IMPORT-5),
/// `[changed after approval]` (R-LIFE-6), `[file changed since approval]` (R-USE-11), then
/// `[rests on <IDs>]` (format.md §14) and `[not tested with <version>]` (R-LIFE-8, from
/// `show::not_tested_with`). `today` is the day of the run, YYYY-MM-DD. An entry with no mark has
/// none.
fn mark(ctx: &Ctx, item: &Item, today: &str) -> Option<String> {
    let entry = &item.entry;
    let mut marks: Vec<String> = status_mark(entry).into_iter().collect();
    if due_for_review(entry, today) {
        marks.push("[due for review]".to_string());
    }
    if imported(entry) {
        marks.push("[imported]".to_string());
    }
    if changed_after_approval(ctx, item) {
        marks.push("[changed after approval]".to_string());
    }
    if file_changed(&ctx.root, entry) {
        marks.push("[file changed since approval]".to_string());
    }
    marks.extend(rests_on_mark(entry));
    marks.extend(not_tested_with(ctx, entry));
    (!marks.is_empty()).then(|| marks.join(" "))
}

/// The status mark of an entry (design.md §5): `[draft]`, `[proposed]`, `[rejected]`, `[declined]`,
/// `[superseded by <ID>]`, `[retired]`. An approved entry has no status mark.
fn status_mark(entry: &Entry) -> Option<String> {
    match entry.status()? {
        Status::Draft => Some("[draft]".to_string()),
        Status::Proposed => Some("[proposed]".to_string()),
        Status::Rejected => Some("[rejected]".to_string()),
        Status::Declined => Some("[declined]".to_string()),
        Status::Superseded => Some(
            match entry.field("superseded-by").filter(|by| !by.is_empty()) {
                Some(by) => format!("[superseded by {by}]"),
                None => "[superseded]".to_string(),
            },
        ),
        Status::Retired => Some("[retired]".to_string()),
        Status::Approved => None,
    }
}

/// Whether the `review` date of the entry (format.md §23) is before `today`, both as YYYY-MM-DD
/// (R-LIFE-7). A review value that is not a date gives no mark.
fn due_for_review(entry: &Entry, today: &str) -> bool {
    let review = entry.field("review").and_then(|text| parse_day(&text));
    matches!((review, parse_day(today)), (Some(review), Some(today)) if review < today)
}

/// A date written YYYY-MM-DD (format.md §23, the `review` field), or None.
fn parse_day(text: &str) -> Option<NaiveDate> {
    NaiveDate::parse_from_str(text.trim(), "%Y-%m-%d").ok()
}

/// Whether the entry was imported from a document: `imported: yes` (format.md §23, R-IMPORT-5).
fn imported(entry: &Entry) -> bool {
    entry
        .field("imported")
        .is_some_and(|value| value.eq_ignore_ascii_case("yes"))
}

/// Whether an approved lesson, how-to or tool (approval::guarded) differs from the checksum
/// recorded at its approval (R-LIFE-6).
fn changed_after_approval(ctx: &Ctx, item: &Item) -> bool {
    let entry = &item.entry;
    entry.kind().is_some_and(approval::guarded)
        && entry.status() == Some(Status::Approved)
        && matches!(
            approval::mark(&ctx.state, &entry.id(), &item.raw),
            Mark::Changed
        )
}

/// Whether a tool's script changed since its checksum was recorded (format.md §22, R-USE-11): the
/// `location` is a file that exists, the entry has a `checksum`, and the file's checksum now
/// differs. The location is relative to the project folder, or absolute. A location that is a
/// command name or a URL, a missing checksum, and a file that cannot be read give no mark.
fn file_changed(root: &Path, entry: &Entry) -> bool {
    if entry.kind() != Some(Kind::Tool) {
        return false;
    }
    let (Some(location), Some(recorded)) = (entry.field("location"), entry.field("checksum"))
    else {
        return false;
    };
    let path = root.join(&location);
    !recorded.is_empty()
        && path.is_file()
        && filesum::checksum(&path).is_ok_and(|now| now != recorded)
}

/// `[rests on <IDs>]` from the field `rests-on` (format.md §14), the IDs joined with `, `. None
/// when the field names no ID.
fn rests_on_mark(entry: &Entry) -> Option<String> {
    let ids = entry.list("rests-on");
    (!ids.is_empty()).then(|| format!("[rests on {}]", ids.join(", ")))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rests_on_gives_the_mark_with_its_ids_and_none_without_any() {
        let one = Entry::parse("# DEC-101 · Invented\n- rests-on: ASM-101\n").expect("parses");
        assert_eq!(rests_on_mark(&one).as_deref(), Some("[rests on ASM-101]"));
        let two = Entry::parse("# DEC-101 · Invented\n- rests-on: ASM-101,  ASM-102 ,\n")
            .expect("parses");
        assert_eq!(
            rests_on_mark(&two).as_deref(),
            Some("[rests on ASM-101, ASM-102]")
        );
        let empty = Entry::parse("# DEC-101 · Invented\n- rests-on:\n").expect("parses");
        assert_eq!(rests_on_mark(&empty), None);
        let missing = Entry::parse("# DEC-101 · Invented\n").expect("parses");
        assert_eq!(rests_on_mark(&missing), None);
    }
}
