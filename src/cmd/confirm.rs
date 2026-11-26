//! The `confirm` command: a person confirms an assumption (R-WRITE-15, format.md §14). The
//! assumption must be approved and its state `open`; it becomes `confirmed`, with the time and
//! the person who confirmed it in `confirmed`, and the `--by` text (who and how) as its
//! *Confirmation* section and in the history. A person confirms; an LLM carries out its person's
//! decision with `--said-by <person>` (D57), and no terminal is needed for either. A refused
//! change writes nothing.

use crate::cli::Global;
use crate::ctx::Ctx;
use crate::entry::{Entry, Kind, Status};
use crate::error::{Code, Error, Result};
use crate::names;
use crate::output::Output;
use crate::store;
use crate::tx::{self, Change};
use serde_json::json;

/// The state of an assumption that is confirmed (format.md §14).
const CONFIRMED: &str = "confirmed";
/// The state of an assumption that nobody has confirmed or found wrong yet.
const OPEN: &str = "open";
/// The name of the section that holds the `--by` text.
const CONFIRMATION: &str = "Confirmation";

/// The arguments of `confirm`.
#[derive(clap::Args, Debug)]
pub struct Args {
    /// The ID of the assumption to confirm, such as ASM-003; any case, with or without a place
    /// prefix
    pub id: String,
    /// Who confirmed it and how, in one line, such as "measured on the rig, report 3"
    #[arg(long)]
    pub by: String,
    /// The person who decided this; needed when an LLM runs the command (D57)
    #[arg(long, value_name = "PERSON")]
    pub said_by: Option<String>,
}

/// Runs `confirm`: checks who acts (names::said_by_gate, D57) and that `--by` is one line, then,
/// in one transaction, sets the assumption to confirmed. An LLM runs it with `--said-by <person>`
/// for its person; it needs no terminal and asks nothing. The output is `<ID>: confirmed`, and
/// the data is {id, state}.
pub fn run(args: &Args, g: &Global) -> Result<Output> {
    let ctx = Ctx::load(g, "confirm")?;
    let who = ctx.writer()?.clone();
    names::said_by_gate(&who, args.said_by.as_deref(), "confirm")?;
    let by = args.by.trim();
    if by.is_empty() {
        return Err(Error::new(
            Code::BadValue,
            "the --by text must not be empty",
        ));
    }
    if by.contains(['\r', '\n']) {
        return Err(Error::new(Code::BadValue, "the --by text is one line"));
    }
    // A line of a section that starts with `## ` would start a section of its own.
    if by.starts_with("## ") {
        return Err(Error::new(
            Code::BadValue,
            "the --by text cannot start with \"## \"",
        ));
    }

    tx::write(&ctx, |tx| {
        let item = store::get(&tx.ctx().dossier, &args.id)?;
        let id = item.entry.id();
        ensure_open_assumption(&item.entry, &id)?;

        let time = tx.time().to_string();
        let mut entry = item.entry.clone();
        entry.set_field("state", CONFIRMED);
        entry.set_field("confirmed", &format!("{time} by {}", who.person));
        entry.set_section(CONFIRMATION, by);
        entry.add_history(&time, &who, CONFIRMED, by);
        tx.put(
            item.path.clone(),
            Some(&item),
            entry.text(),
            Change {
                id: id.clone(),
                event: CONFIRMED.to_string(),
                detail: by.to_string(),
            },
        );
        Ok(Output::with(
            format!("{id}: confirmed"),
            json!({"id": id, "state": CONFIRMED}),
        ))
    })
}

/// Refuses (Code::Refused) an entry that is not an approved assumption whose state is `open`.
/// The message names what the entry is instead.
fn ensure_open_assumption(entry: &Entry, id: &str) -> Result<()> {
    if entry.kind().or_else(|| Kind::of_id(id)) != Some(Kind::Assumption) {
        return Err(refused(format!(
            "only an assumption can be confirmed; {id} is not one"
        )));
    }
    match entry.status() {
        Some(Status::Approved) => {}
        Some(status) => {
            return Err(refused(format!(
                "only an approved assumption can be confirmed; {id} is {}",
                status.name()
            )));
        }
        None => {
            return Err(refused(format!(
                "{id} has no valid status, so it cannot be confirmed"
            )));
        }
    }
    match entry.field("state").as_deref() {
        Some(OPEN) => Ok(()),
        Some(state) => Err(refused(format!(
            "only an open assumption can be confirmed; {id} is {state}"
        ))),
        None => Err(refused(format!(
            "{id} has no valid state, so it cannot be confirmed"
        ))),
    }
}

/// A refusal with the given message, Code::Refused.
fn refused(message: String) -> Error {
    Error::new(Code::Refused, message)
}
