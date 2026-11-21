//! The `accept` command: a decider puts a proposal in force (R-WRITE-12, D28). Only a person, at a
//! terminal, and a person named in `deciders` (names::may_decide) can accept. The proposal is
//! shown on standard error and the person answers `yes`; any other answer changes nothing. An
//! accepted proposal is an approved decision: `approved` names the person who accepted it and
//! when, and `valid-from` is the time of the acceptance.

use crate::cli::Global;
use crate::ctx::Ctx;
use crate::entry::Status;
use crate::error::{Code, Error, Result};
use crate::names::{self, Who};
use crate::output::Output;
use crate::store::{self, Item};
use crate::term;
use crate::tx::{self, Change, Tx};
use serde_json::json;
use std::io::Write;

/// The history event and the journal event of an accepted proposal (format.md §14).
const ACCEPTED: &str = "accepted";

/// The arguments of `accept`.
#[derive(clap::Args, Debug)]
pub struct Args {
    /// The ID of the proposal to accept, such as DEC-012; any case, with or without a place prefix
    pub id: String,
}

/// Runs `accept`: checks who acts, then shows the proposal and asks whether to accept it. A `yes`
/// makes it an approved decision in one transaction; any other answer writes nothing. The output
/// is `<ID>: accepted` or `<ID>: nothing changed`; the data is {id, status, accepted}.
pub fn run(args: &Args, g: &Global) -> Result<Output> {
    let ctx = Ctx::load(g, "accept")?;
    let who = ctx.writer()?.clone();
    if who.is_llm {
        return Err(Error::new(
            Code::NeedsPerson,
            format!("accept is for a person; {} is an LLM", who.name),
        ));
    }
    if !term::is_terminal() {
        return Err(Error::new(
            Code::NeedsTerminal,
            "this command must be run by a person at a terminal",
        ));
    }
    if !names::may_decide(&ctx.cfg, &who.person) {
        return Err(Error::new(
            Code::Refused,
            format!("{} is not in deciders", who.person),
        ));
    }
    tx::write(&ctx, |tx| {
        let item = store::get(&tx.ctx().dossier, &args.id)?;
        let id = item.entry.id();
        match item.entry.status() {
            Some(Status::Proposed) => {}
            Some(status) => {
                return Err(Error::new(
                    Code::Refused,
                    format!("only a proposal can be accepted; {id} is {}", status.name()),
                ));
            }
            None => {
                return Err(Error::new(
                    Code::Refused,
                    format!("only a proposal can be accepted; {id} has no valid status"),
                ));
            }
        }
        show(&item.entry.text());
        let answer = term::ask(&format!("{id}: accept? (yes/no) "))?;
        if !answer.eq_ignore_ascii_case("yes") {
            return Ok(Output::with(
                format!("{id}: nothing changed"),
                json!({"id": id, "status": Status::Proposed.name(), "accepted": false}),
            ));
        }
        accept(tx, &item, &who);
        Ok(Output::with(
            format!("{id}: accepted"),
            json!({"id": id, "status": Status::Approved.name(), "accepted": true}),
        ))
    })
}

/// Queues the acceptance of a proposal: status approved, who accepted it and when, the valid-from
/// time and the history line.
fn accept(tx: &mut Tx, item: &Item, who: &Who) {
    let time = tx.time().to_string();
    let mut entry = item.entry.clone();
    entry.set_field("status", Status::Approved.name());
    entry.set_field("approved", &format!("{time} by {}", who.person));
    entry.set_field("valid-from", &time);
    entry.add_history(&time, who, ACCEPTED, "");
    let id = entry.id();
    let text = entry.text();
    tx.put(
        item.path.clone(),
        Some(item),
        text,
        Change {
            id,
            event: ACCEPTED.to_string(),
            detail: String::new(),
        },
    );
}

/// Writes the text of a proposal to standard error, the review screen. A failed write is ignored:
/// the answer is what matters.
fn show(text: &str) {
    let mut stderr = std::io::stderr();
    let _ = write!(stderr, "{text}");
    if !text.ends_with('\n') {
        let _ = writeln!(stderr);
    }
    let _ = writeln!(stderr);
}
