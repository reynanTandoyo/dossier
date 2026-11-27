//! The `search` command: finds the entries whose text holds every word given, ignoring case
//! (R-READ-3, R-READ-4). A word also matches inside a longer word, so `speed` finds `speedline`.
//! A word is present when the text holds it, when it holds one of its spellings in the glossaries
//! of the project and of the team, so a German word finds the English entries (R-TERM-4), or when
//! one of its words has the same stem (select::stem), so `replaced` finds `replacement`. A stem is
//! compared in the title, the field values and the section bodies of an entry.
//! The project's own entries are searched and, when the project has a team dossier, the team's
//! too (R-READ-3); a team entry is shown with `team:` before its ID. The results come in three
//! groups: every word in the title, a word in a tag field, then the rest; inside a group the newest
//! created first, then by ID (the team's entries sorted in with the project's). Drafts and other
//! statuses follow the filter rules, so a draft is found only with `--drafts` (R-DRAFT-6) and is
//! marked. Each result carries the marks `show` gives (cmd/show.rs), in the same order.
//! When more than one word is typed, the entries that hold only some of the words follow the full
//! matches, after a line `some of the words:`: at most `PARTIAL_LIMIT`, ranked by select::ranked_in
//! over the words and their glossary spellings, with the weights counted over every entry searched,
//! the full matches too, each shown by its title. The count line then names
//! them too. `--full` prints only the full matches; `--json` marks each result with
//! `"match": "all"` or `"match": "some"`.
//! With `--similar <ID>` no words are needed: the entries most related to that entry are listed
//! instead, by word stems, tags and rare words (select::top), under the same filter rules
//! (R-READ-9, `similar`). Each says why it was chosen, `tag match` or `word match`, after the dash.
//! Under each result come its linked entries (links::linked, R-FIND-5), one typed line each (the
//! label of links::typed_line, `linked:` for a plain link), at most `links::LINKED_LIMIT`; in JSON
//! each result has them as `"linked"`, each with its `"type"`. A result whose entry has a
//! `revisit-if` condition shows it as `revisit if: <condition>` before its linked lines
//! (ruled::condition, R-RULED-2); in JSON each result has `"revisit_if"`, null when none.

use crate::approval::{self, Mark};
use crate::cli::Global;
use crate::ctx::Ctx;
use crate::entry::{Entry, Kind, Status};
use crate::error::{Code, Error, Result};
use crate::filesum;
use crate::filter::Filter;
use crate::glossary::{self, Term};
use crate::links;
use crate::output::Output;
use crate::paths;
use crate::ruled;
use crate::select::{self, Query};
use crate::store::{self, Item};
use crate::time;
use chrono::{DateTime, FixedOffset, NaiveDate};
use serde_json::{Value, json};
use std::cmp::Reverse;
use std::collections::HashSet;
use std::path::Path;

/// The tag fields that put an entry in the second group.
const TAG_FIELDS: [&str; 5] = ["machine", "analysis", "software", "tags", "topic"];

/// The place of the project's own entries, and of the entries of a team dossier (R-READ-3).
const PROJECT: &str = "project";
const TEAM: &str = "team";

/// The line that separates the whole entries printed with `--full`.
const SEPARATOR: &str = "---";

/// The most entries that `search --similar` lists (R-READ-9).
const SIMILAR_LIMIT: usize = 10;

/// The most entries that hold only some of the words that `search` lists after the full matches.
const PARTIAL_LIMIT: usize = 10;

/// The arguments of `search`.
#[derive(clap::Args, Debug)]
pub struct Args {
    /// The words to find; an entry must contain all of them, ignoring case. Not needed with
    /// `--similar`
    #[arg(required_unless_present = "similar")]
    pub words: Vec<String>,
    /// Lists the entries that share the most tags and title words with this one, instead of
    /// finding words; any case, with or without a place prefix (R-READ-9)
    #[arg(long, value_name = "ID")]
    pub similar: Option<String>,
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
    /// 0: every word is in the title; 1: a word is in a tag field; 2: the rest. Not used by
    /// `--similar`, which keeps the order of `select::top`, nor by the results with some of the
    /// words, which keep the order of `select::ranked_in`.
    group: u8,
    created: Option<DateTime<FixedOffset>>,
    mark: Option<String>,
    /// The first line of the entry that holds the first word, trimmed; empty when none does.
    /// For `--similar` and for the results with some of the words it is the title.
    line: String,
    /// For `--similar`: why the entry was chosen (select::why), shown after the dash in place
    /// of the line. None for a word search.
    reason: Option<&'static str>,
    /// For a word search: whether the entry holds every word or only some of them. None for
    /// `--similar`.
    matched: Option<Matched>,
    /// The linked entries shown under the result (links::linked), in their order; empty when the
    /// entry names none that is approved.
    linked: Vec<Linked>,
}

/// A linked entry shown under a result: its typed line (links::typed_line) and its data in JSON.
#[derive(Clone)]
struct Linked {
    line: String,
    data: Value,
}

/// How a word search result holds the search words.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Matched {
    /// Every word is held.
    All,
    /// Only some of the words are held; listed after the full matches.
    Some,
}

impl Matched {
    /// The value of `match` in JSON.
    fn name(self) -> &'static str {
        match self {
            Matched::All => "all",
            Matched::Some => "some",
        }
    }
}

impl Hit {
    /// Whether the result holds only some of the search words.
    fn partial(&self) -> bool {
        self.matched == Some(Matched::Some)
    }
}

/// Runs `search`: finds the entries that pass the filter and hold every word, or one of its
/// spellings, in order, and prints them as one line each, or whole with `--full`, then the count.
/// When more than one word was typed, the entries that hold only some of them follow
/// (partial_hits). The entries of the project come first, then those of its team dossier when the
/// project has one (R-READ-3).
pub fn run(args: &Args, g: &Global) -> Result<Output> {
    if let Some(id) = args.similar.as_deref() {
        if !args.words.is_empty() {
            return Err(Error::new(
                Code::Usage,
                "search takes words or --similar, not both",
            ));
        }
        return similar(id, args, g);
    }
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
    let words: Vec<Wanted> = typed.iter().map(|word| wanted(&terms, word)).collect();
    let now = time::now();
    let today = time::date(&now);
    let mut hits: Vec<Hit> = Vec::new();
    // The entries of each place as read, with the place's Ctx. links::linked looks a linked entry up
    // among all of them, also those the filter leaves out, so they are kept whole.
    let mut dossiers: Vec<(&Ctx, Vec<Item>)> = Vec::new();
    // Every entry the search looks at, the full matches and the others: a copy of each entry that
    // passes the filter, with the index in `dossiers` of its place (homes[i] belongs to searched[i]).
    // Linked entries are looked up only for the results that are printed.
    let mut searched: Vec<Item> = Vec::new();
    let mut homes: Vec<usize> = Vec::new();
    for scanned in places {
        let (items, found) = store::all(&scanned.dossier);
        warnings.extend(found);
        for item in items.iter().filter(|item| args.filter.matches(item)) {
            searched.push(copy_of(item));
            homes.push(dossiers.len());
        }
        dossiers.push((scanned, items));
    }
    // The weight of each word counts every entry searched, so a word that is common in the whole
    // dossier weighs little, even when the entries that hold it are the full matches.
    let index = select::Index::of(&searched);
    // The entries that pass the filter but lack a word, each with the index of its place.
    let mut partial: Vec<(usize, Item)> = Vec::new();
    for (item, home) in searched.into_iter().zip(homes) {
        if !holds_every(&item.entry, &words) {
            partial.push((home, item));
        } else if let Some((ctx, lookup)) = dossiers.get(home) {
            let linked = linked_for(lookup, &item.entry);
            hits.push(hit(ctx, item, &words, today, linked));
        }
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
    hits.extend(partial_hits(partial, &dossiers, &words, &index, today));

    Ok(render(&hits, args.full, warnings))
}

/// Runs `search --similar <ID>` (R-READ-9): lists the entries of the project and of its team that
/// pass the filter and are most related to the entry by word stems, tags and rare words, the entry
/// itself left out, at most `SIMILAR_LIMIT`, best first (select::top). The entry is read from the
/// team dossier when its ID has the place `team:`. Each result has its reason (select::why) in
/// place of a matched line; its `line` in JSON is its title.
fn similar(id: &str, args: &Args, g: &Global) -> Result<Output> {
    let ctx = Ctx::load(g, "search")?;
    let team = if ctx.team.is_some() || team_id(id).is_some() {
        Some(team_of(&ctx)?)
    } else {
        None
    };
    let (home, target_id) = match (team_id(id), team.as_ref()) {
        (Some(rest), Some(team_ctx)) => (team_ctx, rest),
        _ => (&ctx, id),
    };
    let target = store::get(&home.dossier, target_id)?;
    let query = Query {
        tags: TAG_FIELDS
            .iter()
            .flat_map(|key| target.entry.list(key))
            .collect(),
        words: select::words(&target.entry.title()),
    };

    let places: Vec<&Ctx> = std::iter::once(&ctx).chain(team.as_ref()).collect();
    let mut warnings = Vec::new();
    // The entries of each place as read, kept whole for the linked lookups (see run).
    let mut dossiers: Vec<(&Ctx, Vec<Item>)> = Vec::new();
    // The candidates: a copy of each entry that passes the filter and is not the entry itself, with
    // the index in `dossiers` of its place. Linked entries are looked up only for the results shown.
    let mut items: Vec<Item> = Vec::new();
    let mut homes: Vec<usize> = Vec::new();
    for place in places {
        let (found, problems) = store::all(&place.dossier);
        warnings.extend(problems);
        for item in found
            .iter()
            .filter(|item| item.path != target.path && args.filter.matches(item))
        {
            items.push(copy_of(item));
            homes.push(dossiers.len());
        }
        dossiers.push((place, found));
    }

    let now = time::now();
    let today = time::date(&now);
    let hits: Vec<Hit> = select::top(&items, &query, SIMILAR_LIMIT)
        .into_iter()
        .filter_map(|picked| {
            // `top` returns references into `items`, so the index of each one is its position.
            let index = items
                .iter()
                .position(|candidate| std::ptr::eq(candidate, picked))?;
            let (place, lookup) = dossiers.get(*homes.get(index)?)?;
            let item = Item {
                path: picked.path.clone(),
                raw: picked.raw.clone(),
                entry: picked.entry.clone(),
            };
            Some(Hit {
                place: place_of(place),
                group: 0,
                created: created(&item.entry),
                mark: mark(place, &item, today),
                line: item.entry.title(),
                reason: select::why(picked, &query),
                matched: None,
                linked: linked_for(lookup, &item.entry),
                item,
            })
        })
        .collect();
    Ok(render(&hits, args.full, warnings))
}

/// The output of a search: the results as lines or, with `full`, as whole entries, then the count
/// line; the data of each result; and the warnings of the reads. With `full` the results with some
/// of the words are not printed, so the count line counts the full matches only.
fn render(hits: &[Hit], full: bool, warnings: Vec<String>) -> Output {
    let mut text = if full {
        full_text(hits)
    } else {
        result_lines(hits)
    };
    let whole = hits.iter().filter(|hit| !hit.partial()).count();
    let partial = hits.iter().filter(|hit| hit.partial()).count();
    text.push_str(&count_line(whole, if full { 0 } else { partial }));
    let data = Value::Array(hits.iter().map(result_data).collect());
    warnings
        .into_iter()
        .fold(Output::with(text, data), |out, warning| out.warn(warning))
}

/// The ID after the place `team:` (any case), or None when the ID has no such place (R-ID-2).
fn team_id(id: &str) -> Option<&str> {
    let (place, rest) = id.split_once(':')?;
    place.eq_ignore_ascii_case(TEAM).then_some(rest)
}

/// The team dossier of the project as a context, for an ID with the place `team:` (R-ID-2). It
/// names the path when the project names a team dossier that cannot be reached (R-PLACE-8).
fn team_of(ctx: &Ctx) -> Result<Ctx> {
    if ctx.team.is_none()
        && let Some(stored) = &ctx.cfg.team
    {
        return Err(Error::new(
            Code::NoDossier,
            format!(
                "{}: the team dossier cannot be reached",
                ctx.root.join(paths::shown(stored)).display()
            ),
        ));
    }
    ctx.team_ctx()
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

/// A typed search word and what it matches (R-TERM-4, R-READ-3): its spellings, in lower case (see
/// matching_spellings), and its stem (select::stem). A text holds the word when one spelling is in
/// the text or one of its words has the stem. Common words are matched too.
struct Wanted {
    spellings: Vec<String>,
    stem: String,
}

/// The search word `word` (lower case) with its spellings and its stem.
fn wanted(terms: &[Term], word: &str) -> Wanted {
    Wanted {
        spellings: matching_spellings(terms, word),
        stem: select::stem(word),
    }
}

/// The stems of the words of `text`, as select::words and select::stem give them. Common words
/// are kept, so a search word matches them by stem too.
fn stems_of(text: &str) -> HashSet<String> {
    select::words(text)
        .into_iter()
        .map(|word| select::stem(&word))
        .collect()
}

/// The stems of the words of an entry's title, field values and section bodies. Section names and
/// the ID are left out, so a search word does not match them by stem.
fn entry_stems(entry: &Entry) -> HashSet<String> {
    let mut parts = vec![entry.title()];
    parts.extend(entry.fields().into_iter().map(|(_, value)| value));
    parts.extend(entry.sections().into_iter().map(|(_, body)| body));
    parts.iter().flat_map(|part| stems_of(part)).collect()
}

/// Whether a text holds the search word: one of its spellings is in the text (lower case), or one
/// of its words has the stem of the word. `stems` are the stems of the words of the text.
fn holds(text: &str, stems: &HashSet<String>, wanted: &Wanted) -> bool {
    wanted
        .spellings
        .iter()
        .any(|spelling| text.contains(spelling.as_str()))
        || stems.contains(&wanted.stem)
}

/// Whether an entry holds every search word (see holds).
fn holds_every(entry: &Entry, words: &[Wanted]) -> bool {
    let text = entry.text().to_lowercase();
    let stems = entry_stems(entry);
    words.iter().all(|wanted| holds(&text, &stems, wanted))
}

/// The entries that hold only some of the search words, best first (select::ranked_in over the
/// words and their glossary spellings), at most `PARTIAL_LIMIT`, each as a result with its title as
/// the line. `partial` holds each entry with the index in `dossiers` of its place; `index` is the
/// select::Index of every entry searched, full matches included, so the weights do not depend on
/// the partial entries alone; `today` is the day of the run, YYYY-MM-DD. Only the entries listed get
/// their linked entries, looked up among the entries of their place. Nothing is listed when one word
/// was typed: its entries hold it or not.
fn partial_hits(
    partial: Vec<(usize, Item)>,
    dossiers: &[(&Ctx, Vec<Item>)],
    words: &[Wanted],
    index: &select::Index,
    today: &str,
) -> Vec<Hit> {
    if words.len() < 2 {
        return Vec::new();
    }
    let query = Query {
        tags: Vec::new(),
        words: words
            .iter()
            .flat_map(|wanted| wanted.spellings.clone())
            .collect(),
    };
    let (homes, items): (Vec<usize>, Vec<Item>) = partial.into_iter().unzip();
    // The index in `items` of each entry that `select::ranked_in` picks, best first.
    let order: Vec<usize> = select::ranked_in(&items, &query, index)
        .into_iter()
        .take(PARTIAL_LIMIT)
        .filter_map(|(_, picked)| {
            items
                .iter()
                .position(|candidate| std::ptr::eq(candidate, picked))
        })
        .collect();
    let mut slots: Vec<Option<Item>> = items.into_iter().map(Some).collect();
    order
        .into_iter()
        .filter_map(|index| {
            let (ctx, lookup) = dossiers.get(*homes.get(index)?)?;
            let item = slots.get_mut(index)?.take()?;
            let linked = linked_for(lookup, &item.entry);
            Some(Hit {
                place: place_of(ctx),
                group: 2,
                created: created(&item.entry),
                mark: mark(ctx, &item, today),
                line: item.entry.title(),
                reason: None,
                matched: Some(Matched::Some),
                linked,
                item,
            })
        })
        .collect()
}

/// The match of one entry that holds every search word, of the dossier `ctx`: its place, group,
/// creation time, mark, matched line and linked entries. `words` holds each search word (see
/// holds_every); `today` is the day of the run, YYYY-MM-DD.
fn hit(ctx: &Ctx, item: Item, words: &[Wanted], today: &str, linked: Vec<Linked>) -> Hit {
    let entry = &item.entry;
    let title = entry.title().to_lowercase();
    let title_stems = stems_of(&entry.title());
    let group = if words
        .iter()
        .all(|wanted| holds(&title, &title_stems, wanted))
    {
        0
    } else if words.iter().any(|wanted| in_tags(entry, wanted)) {
        1
    } else {
        2
    };
    Hit {
        place: place_of(ctx),
        group,
        created: created(entry),
        mark: mark(ctx, &item, today),
        line: matched_line(entry, &words[0]),
        reason: None,
        matched: Some(Matched::All),
        linked,
        item,
    }
}

/// The linked entries shown under the result `entry`, one of the entries `items` of its place: they
/// are looked up among all of `items`, so that a linked entry the filter leaves out is still shown
/// (links::linked).
fn linked_for(items: &[Item], entry: &Entry) -> Vec<Linked> {
    links::linked(entry, items, links::LINKED_LIMIT)
        .into_iter()
        .map(|item| linked_of(entry, item))
        .collect()
}

/// A copy of an entry as `store::all` read it: its path, its exact bytes and its parsed entry.
fn copy_of(item: &Item) -> Item {
    Item {
        path: item.path.clone(),
        raw: item.raw.clone(),
        entry: item.entry.clone(),
    }
}

/// The line and the JSON data of an entry `item` linked from the result `entry`: the typed line
/// (links::typed_line) and its `type` (links::field_of).
fn linked_of(entry: &Entry, item: &Item) -> Linked {
    Linked {
        line: links::typed_line(entry, item),
        data: json!({
            "id": item.entry.id(),
            "title": item.entry.title(),
            "type": links::field_of(entry, item),
        }),
    }
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

/// Whether an element of a tag field of the entry holds the search word.
fn in_tags(entry: &Entry, wanted: &Wanted) -> bool {
    TAG_FIELDS.iter().any(|key| {
        entry
            .list(key)
            .iter()
            .any(|value| holds(&value.to_lowercase(), &stems_of(value), wanted))
    })
}

/// The creation time of an entry, when it is a time stamp (format.md §7).
fn created(entry: &Entry) -> Option<DateTime<FixedOffset>> {
    DateTime::parse_from_rfc3339(&entry.field("created")?).ok()
}

/// The mark of an entry in search (R-READ-1): its marks in the order `show` gives them
/// (cmd/show.rs), joined with a space, the state of an investigation right after the status.
/// `today` is the day of the run, YYYY-MM-DD. No mark when none applies.
fn mark(ctx: &Ctx, item: &Item, today: &str) -> Option<String> {
    let entry = &item.entry;
    let mut marks: Vec<String> = status_mark(entry).into_iter().collect();
    marks.extend(state_mark(entry));
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

/// The state mark of an investigation (R-INV-5, format.md §28): `[open]`, `[solved]` or
/// `[given up]`, from its `state` field in lower case. None for another kind of entry, even one
/// with a `state` field such as an assumption, and None when the field names no known state.
fn state_mark(entry: &Entry) -> Option<String> {
    if entry.kind() != Some(Kind::Investigation) {
        return None;
    }
    let state = entry.field("state")?.to_lowercase();
    matches!(state.as_str(), "open" | "solved" | "given up").then(|| format!("[{state}]"))
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

/// The first line of the entry that holds the search word (see holds), without its line ending and
/// trimmed; empty when no line holds it.
fn matched_line(entry: &Entry, wanted: &Wanted) -> String {
    entry
        .text()
        .lines()
        .map(str::trim)
        .find(|line| holds(&line.to_lowercase(), &stems_of(line), wanted))
        .unwrap_or_default()
        .to_string()
}

/// The lines of the results: the full matches, one line each, then, when there are any, the line
/// `some of the words:` and the results with some of the words, one line each.
fn result_lines(hits: &[Hit]) -> String {
    let (whole, partial): (Vec<&Hit>, Vec<&Hit>) = hits.iter().partition(|hit| !hit.partial());
    let mut text: String = whole.into_iter().map(result_line).collect();
    if !partial.is_empty() {
        text.push_str("some of the words:\n");
        text.extend(partial.into_iter().map(result_line));
    }
    text
}

/// One result line, ending with a line break: `<ID>  [mark] <title>  — <shown>`, the ID with
/// `team:` before it for a team entry, the mark only when there is one and the part after the dash
/// only when it is not empty. The part after the dash is the reason of a `--similar` result, else
/// the matched line; a result with some of the words has none, its title being its line. The
/// line `revisit if:` of an entry with a condition follows it, then the linked lines of the result
/// (push_linked).
fn result_line(hit: &Hit) -> String {
    let id = shown_id(hit);
    let title = hit.item.entry.title();
    let head = match &hit.mark {
        Some(mark) => format!("{id}  {mark} {title}"),
        None => format!("{id}  {title}"),
    };
    let shown = match hit.reason {
        Some(reason) => reason,
        None if hit.partial() => "",
        None => hit.line.as_str(),
    };
    let mut text = if shown.is_empty() {
        format!("{head}\n")
    } else {
        format!("{head}  — {shown}\n")
    };
    if let Some(condition) = ruled::condition(&hit.item.entry) {
        text.push_str(&format!("  revisit if: {condition}\n"));
    }
    push_linked(&mut text, hit);
    text
}

/// Appends the linked lines of a result, each ending with a line break (links::typed_line).
fn push_linked(text: &mut String, hit: &Hit) {
    for linked in &hit.linked {
        text.push_str(&linked.line);
        text.push('\n');
    }
}

/// The whole text of each full match, its linked lines after it, with a line `---` between two
/// results. Every entry text ends with a line break, so the text can be followed by the count line.
/// The results with some of the words are left out.
fn full_text(hits: &[Hit]) -> String {
    let mut text = String::new();
    for (index, hit) in hits.iter().filter(|hit| !hit.partial()).enumerate() {
        if index > 0 {
            text.push_str(SEPARATOR);
            text.push('\n');
        }
        let body = hit.item.entry.text();
        text.push_str(&body);
        if !body.ends_with('\n') {
            text.push('\n');
        }
        push_linked(&mut text, hit);
    }
    text
}

/// The count line: `1 match`, `2 matches`, `0 matches`; with `partial` results printed, the count of
/// them follows: `, 1 with some of the words`, `, 2 with some of the words`.
fn count_line(count: usize, partial: usize) -> String {
    let matches = if count == 1 {
        "1 match".to_string()
    } else {
        format!("{count} matches")
    };
    if partial == 0 {
        matches
    } else {
        format!("{matches}, {partial} with some of the words")
    }
}

/// The data of one result: its ID, title, status, matched line, place (`project` or `team`),
/// linked entries (`linked`, each with its `id`, `title` and `type`, none is an empty list) and
/// `revisit_if` (the condition of ruled::condition, null when none). The ID is the entry's own,
/// without `team:`; the place says where it is. A word search result has its `match` (`all` or
/// `some`); a `--similar` result has its `reason`.
fn result_data(hit: &Hit) -> Value {
    let entry = &hit.item.entry;
    let linked: Vec<Value> = hit
        .linked
        .iter()
        .map(|linked| linked.data.clone())
        .collect();
    let mut data = json!({
        "id": entry.id(),
        "title": entry.title(),
        "status": entry.status().map_or("", Status::name),
        "line": hit.line,
        "place": hit.place,
        "linked": linked,
        "revisit_if": ruled::condition(entry),
    });
    if let Some(matched) = hit.matched {
        data["match"] = json!(matched.name());
    }
    if let Some(reason) = hit.reason {
        data["reason"] = json!(reason);
    }
    data
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
