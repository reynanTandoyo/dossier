//! The `sweep` command (docs/design.md section 25.6, R-SWEEP-1, R-SWEEP-2, D63, D7): lists the
//! pairs of approved decisions, findings, lessons and how-tos in force that look alike and that
//! no link names yet, then the rules for judging them. The pairs come from the rule of
//! `search --similar` (select.rs), so no model is needed to list them (D7); a person judges each
//! pair (D5, D57). The command takes no lock and writes nothing (D63), so it does not use tx::write.

use crate::cli::Global;
use crate::ctx::Ctx;
use crate::entry::{Entry, Kind, Status};
use crate::error::Result;
use crate::output::Output;
use crate::select::{self, Query};
use crate::store::{self, Item};
use serde_json::{Value, json};
use std::collections::BTreeMap;

/// The tag fields of an entry, as `search --similar` reads them (R-READ-9).
const TAG_FIELDS: [&str; 5] = ["machine", "analysis", "software", "tags", "topic"];

/// The fields that name another entry. A pair where either entry names the other in one of them
/// is not listed (R-SWEEP-1).
const NAMING_FIELDS: [&str; 6] = [
    "related",
    "refines",
    "refined-by",
    "contradicts",
    "supersedes",
    "superseded-by",
];

/// The line printed when no pair looks alike (R-SWEEP-1).
const NONE: &str = "no entries in force look alike";

/// The rules printed after the pairs (R-SWEEP-2).
const RULES: &str = "\
Judge each pair with your person; change nothing without their yes:
- Read both: dossier show <ID>.
- They say the same: the newer replaces the older (dossier supersede <old> --by <new>).
- One no longer holds: dossier retire <ID> --reason \"<why>\".
- Both hold but pull against each other: dossier link <A> <B> --as contradicts, and ask which one holds.
- One narrows the other: dossier link <narrower> <wider> --as refines.
- They are only about the same thing: dossier link <A> <B>.
- The same lesson under other conditions (Valid for) is not a duplicate.
- Advice that depends on its context is not a contradiction.";

/// The arguments of `sweep`.
#[derive(clap::Args, Debug)]
pub struct Args {
    /// The most pairs to list
    #[arg(long, default_value_t = 20)]
    pub limit: usize,
}

/// Runs `sweep` (R-SWEEP-1): lists at most `args.limit` pairs of approved decisions, findings,
/// lessons and how-tos that look alike and name neither the other, the highest score first, then
/// the rules for judging them (R-SWEEP-2). The reads of the store become warnings of the output.
pub fn run(args: &Args, g: &Global) -> Result<Output> {
    let ctx = Ctx::load(g, "sweep")?;
    let (items, warnings) = store::all(&ctx.dossier);
    let candidates: Vec<Item> = items.into_iter().filter(is_candidate).collect();
    let index = select::Index::of(&candidates);

    // Each pair once, keyed by the positions of its two entries in `candidates`, with the higher
    // of the two scores found from either side.
    let mut scores: BTreeMap<(usize, usize), u32> = BTreeMap::new();
    for (candidate_at, candidate) in candidates.iter().enumerate() {
        let query = Query {
            tags: TAG_FIELDS
                .iter()
                .flat_map(|key| candidate.entry.list(key))
                .collect(),
            words: select::words(&candidate.entry.title()),
        };
        for (score, other) in select::ranked_in(&candidates, &query, &index) {
            if other.path == candidate.path || linked(&candidate.entry, &other.entry) {
                continue;
            }
            let Some(other_at) = candidates.iter().position(|item| item.path == other.path) else {
                continue;
            };
            let pair = (candidate_at.min(other_at), candidate_at.max(other_at));
            scores
                .entry(pair)
                .and_modify(|kept| *kept = (*kept).max(score))
                .or_insert(score);
        }
    }

    let mut pairs: Vec<(u32, &Item, &Item)> = scores
        .into_iter()
        .filter_map(|((first, second), score)| {
            Some((score, candidates.get(first)?, candidates.get(second)?))
        })
        .collect();
    pairs.sort_by(|left, right| {
        right
            .0
            .cmp(&left.0)
            .then_with(|| left.1.entry.id().cmp(&right.1.entry.id()))
            .then_with(|| left.2.entry.id().cmp(&right.2.entry.id()))
    });
    pairs.truncate(args.limit);

    let text = text_of(&pairs);
    let data = json!({
        "pairs": Value::Array(pairs.iter().map(|(score, first, second)| json!({
            "a": entry_data(first),
            "b": entry_data(second),
            "score": score,
        })).collect()),
    });
    Ok(warnings
        .into_iter()
        .fold(Output::with(text, data), |out, warning| out.warn(warning)))
}

/// Whether an entry is approved and of a kind that `sweep` compares: a decision, finding, lesson
/// or howto. Drafts and the other kinds are left out.
fn is_candidate(item: &Item) -> bool {
    item.entry.status() == Some(Status::Approved)
        && matches!(
            item.entry.kind(),
            Some(Kind::Decision | Kind::Finding | Kind::Lesson | Kind::Howto)
        )
}

/// Whether one of the two entries names the other in one of NAMING_FIELDS, ignoring case.
fn linked(first: &Entry, second: &Entry) -> bool {
    names(first, second) || names(second, first)
}

/// Whether `entry` names the ID of `other` in one of NAMING_FIELDS, ignoring case.
fn names(entry: &Entry, other: &Entry) -> bool {
    let id = other.id();
    NAMING_FIELDS.iter().any(|key| {
        entry
            .list(key)
            .iter()
            .any(|wanted| wanted.eq_ignore_ascii_case(&id))
    })
}

/// The text of the result: the count line, a blank line, two lines and a blank line per pair, then
/// RULES; or NONE when there is no pair.
fn text_of(pairs: &[(u32, &Item, &Item)]) -> String {
    if pairs.is_empty() {
        return NONE.to_string();
    }
    let count = if pairs.len() == 1 {
        "1 pair of entries in force looks alike:".to_string()
    } else {
        format!("{} pairs of entries in force look alike:", pairs.len())
    };
    let mut text = format!("{count}\n\n");
    for (_, first, second) in pairs {
        text.push_str(&format!("{}\n{}\n\n", line_of(first), line_of(second)));
    }
    text.push_str(RULES);
    text
}

/// The line of an entry in the list: `<ID> · <kind> · <title>`.
fn line_of(item: &Item) -> String {
    format!(
        "{} · {} · {}",
        item.entry.id(),
        kind_name(item),
        item.entry.title()
    )
}

/// The kind of an entry as its name, empty when it has no kind.
fn kind_name(item: &Item) -> &'static str {
    item.entry.kind().map_or("", Kind::name)
}

/// The JSON data of one entry of a pair: its ID, kind and title.
fn entry_data(item: &Item) -> Value {
    json!({
        "id": item.entry.id(),
        "kind": kind_name(item),
        "title": item.entry.title(),
    })
}
