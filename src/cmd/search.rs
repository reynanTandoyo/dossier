//! The `search` command: finds the entries whose text holds every word given, ignoring case
//! (R-READ-3, R-READ-4). A word also matches inside a longer word, so `speed` finds `speedline`.
//! The results come in three groups: every word in the title, a word in a tag field, then the
//! rest; inside a group the newest created first, then by ID. Drafts and other statuses follow the
//! filter rules, so a draft is found only with `--drafts` (R-DRAFT-6) and is marked.

use crate::approval::{self, Mark};
use crate::cli::Global;
use crate::ctx::Ctx;
use crate::entry::{Entry, Kind, Status};
use crate::error::{Code, Error, Result};
use crate::filter::Filter;
use crate::output::Output;
use crate::store::{self, Item};
use chrono::{DateTime, FixedOffset};
use serde_json::{Value, json};

/// The tag fields that put an entry in the second group.
const TAG_FIELDS: [&str; 4] = ["machine", "analysis", "software", "tags"];

/// The line that separates the whole entries printed with `--full`.
const SEPARATOR: &str = "---";

/// The arguments of `search`.
#[derive(clap::Args, Debug)]
pub struct Args {
    /// The words to find; an entry must contain all of them, ignoring case
    #[arg(required = true)]
    pub words: Vec<String>,
    /// Prints each whole entry instead of one line per entry
    #[arg(long)]
    pub full: bool,
    #[command(flatten)]
    pub filter: Filter,
}

/// One entry that matched, with what is printed about it.
struct Hit<'a> {
    item: &'a Item,
    /// 0: every word is in the title; 1: a word is in a tag field; 2: the rest.
    group: u8,
    created: Option<DateTime<FixedOffset>>,
    mark: Option<String>,
    /// The first line of the entry that contains the first word, trimmed; empty when none does.
    line: String,
}

/// Runs `search`: finds the entries that pass the filter and hold every word, in order, and
/// prints them as one line each, or whole with `--full`, then the count.
pub fn run(args: &Args, g: &Global) -> Result<Output> {
    let words = search_words(&args.words)?;
    let ctx = Ctx::load(g, "search")?;
    let (items, warnings) = store::all(&ctx.dossier);
    let mut hits: Vec<Hit> = items
        .iter()
        .filter(|item| args.filter.matches(item))
        .filter_map(|item| hit(&ctx, item, &words))
        .collect();
    // `store::all` gives the entries by ID and the sort is stable, so entries with the same
    // group and creation time keep their ID order.
    hits.sort_by(|a, b| {
        a.group
            .cmp(&b.group)
            .then_with(|| b.created.cmp(&a.created))
    });

    let mut text = if args.full {
        full_text(&hits)
    } else {
        result_lines(&hits)
    };
    text.push_str(&count_line(hits.len()));
    let data = Value::Array(hits.iter().map(result_data).collect());
    let output = Output::with(text, data);
    Ok(warnings
        .into_iter()
        .fold(output, |out, warning| out.warn(warning)))
}

/// The words to find, trimmed and in lower case. A blank word would match every entry, so blank
/// words are dropped, and Code::Usage when none is left.
fn search_words(given: &[String]) -> Result<Vec<String>> {
    let words: Vec<String> = given
        .iter()
        .map(|word| word.trim().to_lowercase())
        .filter(|word| !word.is_empty())
        .collect();
    if words.is_empty() {
        return Err(Error::new(
            Code::Usage,
            "search needs at least one word that is not blank",
        ));
    }
    Ok(words)
}

/// The match of one entry: its group, creation time, mark and matched line. None when a word is
/// missing from the entry's text.
fn hit<'a>(ctx: &Ctx, item: &'a Item, words: &[String]) -> Option<Hit<'a>> {
    let entry = &item.entry;
    let text = entry.text().to_lowercase();
    if !words.iter().all(|word| text.contains(word)) {
        return None;
    }
    let title = entry.title().to_lowercase();
    let group = if words.iter().all(|word| title.contains(word)) {
        0
    } else if words.iter().any(|word| in_tags(entry, word)) {
        1
    } else {
        2
    };
    Some(Hit {
        item,
        group,
        created: created(entry),
        mark: mark(ctx, item),
        line: matched_line(entry, &words[0]),
    })
}

/// Whether an element of a tag field of the entry contains `word` (lower case).
fn in_tags(entry: &Entry, word: &str) -> bool {
    TAG_FIELDS.iter().any(|key| {
        entry
            .list(key)
            .iter()
            .any(|value| value.to_lowercase().contains(word))
    })
}

/// The creation time of an entry, when it is a time stamp (format.md §7).
fn created(entry: &Entry) -> Option<DateTime<FixedOffset>> {
    DateTime::parse_from_rfc3339(&entry.field("created")?).ok()
}

/// The mark of an entry in search (R-READ-1): its status when it is not approved, and
/// `changed after approval` for an approved lesson whose file changed since it was approved.
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
            let lesson = entry.kind() == Some(Kind::Lesson);
            let changed = matches!(
                approval::mark(&ctx.state, &entry.id(), &item.raw),
                Mark::Changed
            );
            (lesson && changed).then(|| "[changed after approval]".to_string())
        }
    }
}

/// The first line of the entry that contains `word` (lower case), without its line ending and
/// trimmed; empty when no line contains it.
fn matched_line(entry: &Entry, word: &str) -> String {
    entry
        .text()
        .lines()
        .map(str::trim)
        .find(|line| line.to_lowercase().contains(word))
        .unwrap_or_default()
        .to_string()
}

/// One line per result, each ending with a line break: `<ID>  [mark] <title>  — <line>`, the
/// mark only when there is one and the line only when it is not empty.
fn result_lines(hits: &[Hit]) -> String {
    let mut text = String::new();
    for hit in hits {
        let entry = &hit.item.entry;
        let head = match &hit.mark {
            Some(mark) => format!("{}  {mark} {}", entry.id(), entry.title()),
            None => format!("{}  {}", entry.id(), entry.title()),
        };
        if hit.line.is_empty() {
            text.push_str(&head);
        } else {
            text.push_str(&format!("{head}  — {}", hit.line));
        }
        text.push('\n');
    }
    text
}

/// The whole text of each result, with a line `---` between two results. Every entry text ends
/// with a line break, so the text can be followed by the count line.
fn full_text(hits: &[Hit]) -> String {
    let mut text = String::new();
    for (index, hit) in hits.iter().enumerate() {
        if index > 0 {
            text.push_str(SEPARATOR);
            text.push('\n');
        }
        let body = hit.item.entry.text();
        text.push_str(&body);
        if !body.ends_with('\n') {
            text.push('\n');
        }
    }
    text
}

/// The count line: `1 match`, `2 matches`, `0 matches`.
fn count_line(count: usize) -> String {
    if count == 1 {
        "1 match".to_string()
    } else {
        format!("{count} matches")
    }
}

/// The data of one result: its ID, title, status and matched line.
fn result_data(hit: &Hit) -> Value {
    let entry = &hit.item.entry;
    json!({
        "id": entry.id(),
        "title": entry.title(),
        "status": entry.status().map_or("", Status::name),
        "line": hit.line,
    })
}
