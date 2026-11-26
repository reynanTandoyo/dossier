//! The `reviewed` command (docs/design.md section 12, R-LIFE-7, D33): a person at a terminal, or
//! an LLM with `--said-by <person>` (D57), confirms that an entry still holds. The `reviewed` field
//! gets the time and the person, the `review` field gets the next review date given with `--next`
//! or is removed (format.md §23), and the history line `reviewed` names the next date. On an
//! approved lesson, how-to or tool the new text is recorded as its approved text, as `set` does for
//! a person's change (approval::guarded), so the confirmation is not taken for a hand change.

use crate::approval;
use crate::cli::Global;
use crate::ctx::Ctx;
use crate::entry::{Entry, Status};
use crate::error::{Code, Error, Result};
use crate::names;
use crate::output::Output;
use crate::store;
use crate::term;
use crate::tx::{self, Change};
use chrono::NaiveDate;
use serde_json::json;

/// The field that holds the next review date (format.md §23).
const REVIEW: &str = "review";

/// The field that holds the time and the person of the last confirmation (format.md §23).
const REVIEWED: &str = "reviewed";

/// The history event and the journal event of a confirmation (format.md §6).
const EVENT: &str = "reviewed";

/// The arguments of `reviewed`.
#[derive(clap::Args, Debug)]
pub struct Args {
    /// The ID of the entry, such as DEC-012; any case, with or without a place prefix
    pub id: String,
    /// The next date to review the entry on, as YYYY-MM-DD; without it the review date is removed
    #[arg(long, value_name = "date")]
    pub next: Option<String>,

    /// The person who decided this; needed when an LLM runs the command (D57)
    #[arg(long, value_name = "PERSON")]
    pub said_by: Option<String>,
}

/// Runs `reviewed`: checks who acts (names::said_by_gate, D57), that a person is at a terminal
/// unless an LLM confirms for its person with `--said-by`, and that `--next` is a date. Then, in
/// one transaction, sets `reviewed` to `<time> by <person>` and sets or removes `review`. The
/// first output line is `<ID>: reviewed`, with `, next review <date>` after it when a date was
/// given. The data is {id, reviewed, review}, where review is null when no date was given.
pub fn run(args: &Args, g: &Global) -> Result<Output> {
    let ctx = Ctx::load(g, "reviewed")?;
    let who = ctx.writer()?.clone();
    let said = names::said_by_gate(&who, args.said_by.as_deref(), "reviewed")?;
    if !said && !term::is_terminal() {
        return Err(Error::new(
            Code::NeedsTerminal,
            "this command must be run by a person at a terminal",
        ));
    }
    let next = args.next.as_deref().map(day).transpose()?;

    tx::write(&ctx, |tx| {
        let item = store::get(&tx.ctx().dossier, &args.id)?;
        let id = item.entry.id();
        let time = tx.time().to_string();
        let reviewed = format!("{time} by {}", who.person);

        let mut entry = item.entry.clone();
        entry.set_field(REVIEWED, &reviewed);
        match &next {
            Some(day) => entry.set_field(REVIEW, day),
            None => entry.remove_field(REVIEW),
        }
        let detail = match &next {
            Some(day) => format!("next {day}"),
            None => String::new(),
        };
        entry.add_history(&time, &who, EVENT, &detail);

        let text = entry.text();
        if is_approved_guarded(&entry) && !tx.ctx().dry_run {
            approval::record(&tx.ctx().state, &id, text.as_bytes())?;
        }
        tx.put(
            item.path.clone(),
            Some(&item),
            text,
            Change {
                id: id.clone(),
                event: EVENT.to_string(),
                detail,
            },
        );

        let message = match &next {
            Some(day) => format!("{id}: reviewed, next review {day}"),
            None => format!("{id}: reviewed"),
        };
        Ok(Output::with(
            message,
            json!({"id": id, "reviewed": reviewed, "review": next}),
        ))
    })
}

/// Whether the entry is an approved lesson, how-to or tool, whose approved text is recorded
/// (approval::guarded).
fn is_approved_guarded(entry: &Entry) -> bool {
    entry.status() == Some(Status::Approved) && entry.kind().is_some_and(approval::guarded)
}

/// The `--next` date as YYYY-MM-DD: the form is checked first, then chrono must parse it.
/// Code::BadValue otherwise.
fn day(text: &str) -> Result<String> {
    let trimmed = text.trim();
    let form_ok = trimmed.len() == 10
        && trimmed.bytes().enumerate().all(|(index, byte)| {
            if index == 4 || index == 7 {
                byte == b'-'
            } else {
                byte.is_ascii_digit()
            }
        });
    let parsed = if form_ok {
        NaiveDate::parse_from_str(trimmed, "%Y-%m-%d").ok()
    } else {
        None
    };
    match parsed {
        Some(date) => Ok(date.format("%Y-%m-%d").to_string()),
        None => Err(Error::new(
            Code::BadValue,
            format!("--next needs a date such as 2027-11-20, not \"{text}\""),
        )),
    }
}
