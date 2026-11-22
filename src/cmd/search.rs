//! The `search` command: finds the entries whose text holds every word given, ignoring case
//! (R-READ-3, R-READ-4). A word also matches inside a longer word, so `speed` finds `speedline`.
//! A word is present when the text holds it or one of its spellings in the glossaries of the
//! project and of the team, so a German word finds the English entries (R-TERM-4).
//! The project's own entries are searched and, when the project has a team dossier, the team's
//! too (R-READ-3); a team entry is shown with `team:` before its ID. The results come in three
//! groups: every word in the title, a word in a tag field, then the rest; inside a group the newest
//! created first, then by ID (the team's entries sorted in with the project's). Drafts and other
//! statuses follow the filter rules, so a draft is found only with `--drafts` (R-DRAFT-6) and is
//! marked. Each result carries the marks `show` gives (cmd/show.rs), in the same order.

use crate::approval::{self, Mark};
use crate::cli::Global;
use crate::ctx::Ctx;
use crate::entry::{Entry, Kind, Status};
use crate::error::{Code, Error, Result};
use crate::filesum;
use crate::filter::Filter;
use crate::glossary::{self, Term};
use crate::output::Output;
use crate::store::{self, Item};
use crate::time;
use chrono::{DateTime, FixedOffset, NaiveDate};
use serde_json::{Value, json};
use std::cmp::Reverse;
use std::path::Path;

/// The tag fields that put an entry in the second group.
const TAG_FIELDS: [&str; 4] = ["machine", "analysis", "software", "tags"];

/// The place of the project's own entries, and of the entries of a team dossier (R-READ-3).
const PROJECT: &str = "project";
const TEAM: &str = "team";

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
struct Hit {
    item: Item,
    /// `project` for the project's own entries, `team` for those of its team dossier.
    place: &'static str,
    /// 0: every word is in the title; 1: a word is in a tag field; 2: the rest.
    group: u8,
    created: Option<DateTime<FixedOffset>>,
    mark: Option<String>,
    /// The first line of the entry that contains the first word, trimmed; empty when none does.
    line: String,
}

/// Runs `search`: finds the entries that pass the filter and hold every word, or one of its
/// spellings, in order, and prints them as one line each, or whole with `--full`, then the count.
/// The entries of the project come first, then those of its team dossier when the project has one
/// (R-READ-3).
pub fn run(args: &Args, g: &Global) -> Result<Output> {
    let typed = search_words(&args.words)?;
    let ctx = Ctx::load(g, "search")?;
    let team = if ctx.team.is_some() {
        Some(ctx.team_ctx()?)
    } else {
        None
    };
    let places: Vec<&Ctx> = std::iter::once(&ctx).chain(team.as_ref()).collect();
    let mut warnings = Vec::new();
    let terms = glossary_terms(&places, &mut warnings);
    let words: Vec<Vec<String>> = typed
        .iter()
        .map(|word| matching_spellings(&terms, word))
        .collect();
    let now = time::now();
    let today = time::date(&now);
    let mut hits: Vec<Hit> = Vec::new();
    for scanned in places {
        let (items, found) = store::all(&scanned.dossier);
        warnings.extend(found);
        hits.extend(
            items
                .into_iter()
                .filter(|item| args.filter.matches(item))
                .filter_map(|item| hit(scanned, item, &words, today)),
        );
    }
    // Newest first within a group, then by ID in the order `store::all` uses. An entry of each
    // place with the same ID keeps the project's first, so the order is the same on every run.
    hits.sort_by_cached_key(|hit| {
        (
            hit.group,
            Reverse(hit.created),
            id_order(&hit.item.entry.id()),
            hit.place == TEAM,
        )
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

/// The glossary terms of the dossiers searched, the project's and the team's, in the order of the
/// places (R-TERM-4). A glossary that cannot be read gives a warning, and the search goes on
/// without its terms.
fn glossary_terms(places: &[&Ctx], warnings: &mut Vec<String>) -> Vec<Term> {
    let mut terms = Vec::new();
    for place in places {
        match glossary::load(&place.dossier, &place.cfg.code) {
            Ok((file, _)) => terms.extend(glossary::terms(&file)),
            Err(error) => warnings.push(error.message),
        }
    }
    terms
}

/// The spellings that a search word matches (R-TERM-4): the word itself, then every spelling of
/// the glossary terms it is a spelling of, all in lower case. The word is present in a text when
/// one of them is.
fn matching_spellings(terms: &[Term], word: &str) -> Vec<String> {
    std::iter::once(word.to_string())
        .chain(
            glossary::spellings(terms, word)
                .into_iter()
                .map(|spelling| spelling.to_lowercase()),
        )
        .collect()
}

/// Whether `text` (in lower case) holds one of the spellings of a search word.
fn holds(text: &str, spellings: &[String]) -> bool {
    spellings
        .iter()
        .any(|spelling| text.contains(spelling.as_str()))
}

/// The match of one entry of the dossier `ctx`: its place, group, creation time, mark and matched
/// line. `words` holds the spellings of each search word; `today` is the day of the run,
/// YYYY-MM-DD. None when a word is missing from the entry's text.
fn hit(ctx: &Ctx, item: Item, words: &[Vec<String>], today: &str) -> Option<Hit> {
    let entry = &item.entry;
    let text = entry.text().to_lowercase();
    if !words.iter().all(|spellings| holds(&text, spellings)) {
        return None;
    }
    let title = entry.title().to_lowercase();
    let group = if words.iter().all(|spellings| holds(&title, spellings)) {
        0
    } else if words.iter().any(|spellings| in_tags(entry, spellings)) {
        1
    } else {
        2
    };
    Some(Hit {
        place: place_of(ctx),
        group,
        created: created(entry),
        mark: mark(ctx, &item, today),
        line: matched_line(entry, &words[0]),
        item,
    })
}

/// The place of the entries of the dossier `ctx`: `team` for a team dossier, else `project`.
fn place_of(ctx: &Ctx) -> &'static str {
    if ctx.cfg.kind == "team" {
        TEAM
    } else {
        PROJECT
    }
}

/// The ID of a result as it is shown: a team entry has `team:` before its ID (R-READ-3).
fn shown_id(hit: &Hit) -> String {
    let id = hit.item.entry.id();
    if hit.place == TEAM {
        format!("{TEAM}:{id}")
    } else {
        id
    }
}

/// The order `store::all` gives entries by ID, as a sort key: the number after the prefix (an ID
/// with no number last), then the prefix. Kept in step with `store::all` by hand, since search
/// sorts the entries of two dossiers together.
fn id_order(id: &str) -> (bool, u64, String) {
    let (prefix, rest) = id.split_once('-').unwrap_or((id, ""));
    let digits: String = rest.chars().take_while(char::is_ascii_digit).collect();
    match digits.parse::<u64>() {
        Ok(number) => (false, number, prefix.to_string()),
        Err(_) => (true, 0, prefix.to_string()),
    }
}

/// Whether an element of a tag field of the entry holds one of the spellings (lower case).
fn in_tags(entry: &Entry, spellings: &[String]) -> bool {
    TAG_FIELDS.iter().any(|key| {
        entry
            .list(key)
            .iter()
            .any(|value| holds(&value.to_lowercase(), spellings))
    })
}

/// The creation time of an entry, when it is a time stamp (format.md §7).
fn created(entry: &Entry) -> Option<DateTime<FixedOffset>> {
    DateTime::parse_from_rfc3339(&entry.field("created")?).ok()
}

/// The mark of an entry in search (R-READ-1): its marks in the order `show` gives them
/// (cmd/show.rs), joined with a space. `today` is the day of the run, YYYY-MM-DD. No mark when none
/// applies.
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
    (!marks.is_empty()).then(|| marks.join(" "))
}

/// The status mark of an entry: `[draft]`, `[proposed]`, `[rejected]`, `[declined]`, `[superseded
/// by <ID>]`, `[retired]`. An approved entry has no status mark.
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
/// differs. The location is relative to the dossier's project folder, or absolute. A location that
/// is a command name or a URL, a missing checksum, and a file that cannot be read give no mark.
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

/// The first line of the entry that holds one of the spellings (lower case), without its line
/// ending and trimmed; empty when no line holds one.
fn matched_line(entry: &Entry, spellings: &[String]) -> String {
    entry
        .text()
        .lines()
        .map(str::trim)
        .find(|line| holds(&line.to_lowercase(), spellings))
        .unwrap_or_default()
        .to_string()
}

/// One line per result, each ending with a line break: `<ID>  [mark] <title>  — <line>`, the ID
/// with `team:` before it for a team entry, the mark only when there is one and the line only when
/// it is not empty.
fn result_lines(hits: &[Hit]) -> String {
    let mut text = String::new();
    for hit in hits {
        let id = shown_id(hit);
        let title = hit.item.entry.title();
        let head = match &hit.mark {
            Some(mark) => format!("{id}  {mark} {title}"),
            None => format!("{id}  {title}"),
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

/// The data of one result: its ID, title, status, matched line and place (`project` or `team`).
/// The ID is the entry's own, without `team:`; the place says where it is.
fn result_data(hit: &Hit) -> Value {
    let entry = &hit.item.entry;
    json!({
        "id": entry.id(),
        "title": entry.title(),
        "status": entry.status().map_or("", Status::name),
        "line": hit.line,
        "place": hit.place,
    })
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
