//! The `log` command: who did what when, across the whole dossier, newest first (R-HIST-3,
//! R-HIST-4, R-HIST-6). One line per journal record: the time, the person (with `via` and the
//! LLM when an LLM made the change), the event, the entry and its title. A record about a
//! settings file (its ID is not an entry ID, such as DOSSIER) shows its detail in place of the
//! title. The filters narrow the lines; `--limit` caps how many are printed.

use crate::cli::Global;
use crate::ctx::Ctx;
use crate::entry::Kind;
use crate::error::{Code, Error, Result};
use crate::journal::{self, Record};
use crate::output::Output;
use crate::store;
use crate::time;
use chrono::{DateTime, FixedOffset, NaiveDate};
use serde_json::{Value, json};
use std::cmp::Reverse;
use std::collections::HashMap;

/// The text shown in place of the title of an entry that is no longer in the dossier.
const REMOVED: &str = "(removed)";

/// The text shown where the title goes: the title of the record's entry, `(removed)` for an
/// entry that is gone, and for a record about a settings file (its ID is not an entry ID) the
/// record's detail.
fn shown_title(record: &Record, titles: &HashMap<String, String>) -> String {
    if Kind::of_id(&record.id).is_none() {
        return record.detail.clone();
    }
    titles
        .get(record.id.as_str())
        .cloned()
        .unwrap_or_else(|| REMOVED.to_string())
}

/// The arguments of `log`.
#[derive(clap::Args, Debug)]
pub struct Args {
    /// Only events on entries of this kind: decision, assumption, finding or lesson, or its
    /// prefix (DEC, ASM, FND, LES), in any case
    #[arg(long)]
    pub kind: Option<String>,
    /// Only events of this person, ignoring case
    #[arg(long)]
    pub person: Option<String>,
    /// Only events with this name, exactly, such as created or approved
    #[arg(long)]
    pub event: Option<String>,
    /// Only events on or after this date (YYYY-MM-DD)
    #[arg(long)]
    pub since: Option<String>,
    /// Only events on or before this date (YYYY-MM-DD)
    #[arg(long)]
    pub until: Option<String>,
    /// The most lines to print
    #[arg(long, default_value_t = 50)]
    pub limit: usize,
}

/// The filters of `log`, checked: the kind, the person in lower case, the event, and the two
/// dates in the form YYYY-MM-DD. A missing option matches every record.
struct Choice {
    kind: Option<Kind>,
    person: Option<String>,
    event: Option<String>,
    since: Option<String>,
    until: Option<String>,
}

impl Choice {
    /// The filters from the arguments. Code::BadValue for a kind or a date that is not one.
    fn from_args(args: &Args) -> Result<Choice> {
        Ok(Choice {
            kind: kind_option(args.kind.as_deref())?,
            person: args
                .person
                .as_deref()
                .map(|person| person.trim().to_lowercase()),
            event: args.event.clone(),
            since: date_option("since", args.since.as_deref())?,
            until: date_option("until", args.until.as_deref())?,
        })
    }

    /// Whether a record passes every filter that is given. The date is the date part of the
    /// record's time stamp, and both dates are inclusive.
    fn holds(&self, record: &Record) -> bool {
        let day = time::date(&record.time);
        self.kind
            .is_none_or(|kind| Kind::of_id(&record.id) == Some(kind))
            && self
                .person
                .as_ref()
                .is_none_or(|person| record.person.to_lowercase() == *person)
            && self
                .event
                .as_ref()
                .is_none_or(|event| record.event == *event)
            && self
                .since
                .as_ref()
                .is_none_or(|since| day >= since.as_str())
            && self
                .until
                .as_ref()
                .is_none_or(|until| day <= until.as_str())
    }
}

/// The kind option by name or prefix. Code::BadValue when it is none of the four kinds.
fn kind_option(text: Option<&str>) -> Result<Option<Kind>> {
    text.map(|text| {
        Kind::parse(text.trim()).ok_or_else(|| {
            Error::new(
                Code::BadValue,
                format!("unknown kind \"{text}\": use decision, assumption, finding or lesson"),
            )
        })
    })
    .transpose()
}

/// A date option, normalized to YYYY-MM-DD. Code::BadValue when it is not a date of that form.
fn date_option(name: &str, text: Option<&str>) -> Result<Option<String>> {
    text.map(|text| {
        NaiveDate::parse_from_str(text.trim(), "%Y-%m-%d")
            .map(|day| day.format("%Y-%m-%d").to_string())
            .map_err(|_| {
                Error::new(
                    Code::BadValue,
                    format!("--{name} needs a date such as 2026-11-21, not \"{text}\""),
                )
            })
    })
    .transpose()
}

/// Runs `log`: the records the filters choose, newest first, at most `limit` lines, then the
/// count and, when more matched, how many more there are. The data is the same records with
/// their title (null when the entry is gone).
pub fn run(args: &Args, g: &Global) -> Result<Output> {
    let ctx = Ctx::load(g, "log")?;
    let choice = Choice::from_args(args)?;
    let (items, warnings) = store::all(&ctx.dossier);
    let titles: HashMap<String, String> = items
        .iter()
        .map(|item| (item.entry.id(), item.entry.title()))
        .collect();

    let records: Vec<Record> = journal::read(&ctx.state)?
        .into_iter()
        .filter(|record| choice.holds(record))
        .collect();
    let total = records.len();
    let newest = newest_first(records);
    let shown = &newest[..total.min(args.limit)];

    let mut lines: Vec<String> = shown
        .iter()
        .map(|record| line(record, &shown_title(record, &titles)))
        .collect();
    lines.push(count_line(shown.len()));
    if total > shown.len() {
        lines.push(format!("({} more; use --limit)", total - shown.len()));
    }

    let data: Vec<Value> = shown
        .iter()
        .map(|record| {
            json!({
                "time": record.time,
                "name": record.name,
                "person": record.person,
                "llm": record.llm,
                "command": record.command,
                "id": record.id,
                "event": record.event,
                "detail": record.detail,
                "title": titles.get(record.id.as_str()),
            })
        })
        .collect();

    let output = Output::with(lines.join("\n"), Value::Array(data));
    Ok(warnings
        .into_iter()
        .fold(output, |output, warning| output.warn(warning)))
}

/// The records newest first. The journal is oldest first, so the list is reversed, and then
/// sorted by the instant of the time stamp; the sort is stable, so records with the same instant
/// keep the reversed journal order (the later write first). A stamp that does not parse sorts last.
fn newest_first(mut records: Vec<Record>) -> Vec<Record> {
    records.reverse();
    records.sort_by_key(|record| Reverse(instant(&record.time)));
    records
}

/// The instant of a time stamp (format.md §7), when it parses.
fn instant(stamp: &str) -> Option<DateTime<FixedOffset>> {
    DateTime::parse_from_rfc3339(stamp).ok()
}

/// One line: `<time>  <person>[ via <name>]  <event>  <ID>  <title>`. The `via` part is only
/// there when an LLM made the change. A record with no title (no detail either) ends at its ID.
fn line(record: &Record, title: &str) -> String {
    let via = if record.llm {
        format!(" via {}", record.name)
    } else {
        String::new()
    };
    format!(
        "{}  {}{via}  {}  {}  {title}",
        record.time, record.person, record.event, record.id
    )
    .trim_end()
    .to_string()
}

/// `1 event`, or `<n> events`.
fn count_line(n: usize) -> String {
    if n == 1 {
        "1 event".to_string()
    } else {
        format!("{n} events")
    }
}
