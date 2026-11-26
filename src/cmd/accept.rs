//! The `accept` command: a decider puts a proposal in force (R-WRITE-12, D28). A person at a
//! terminal, or an LLM with `--said-by <person>` (D57), can accept; the person must be named in
//! `deciders` (names::may_decide). A person is shown the proposal on standard error and answers
//! `yes`; any other answer changes nothing. With `--said-by` nothing is shown or asked. An accepted
//! proposal is an approved decision: `approved` names the person who accepted it and when, and
//! `valid-from` is the time of the acceptance.

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

    /// The person who decided this; needed when an LLM runs the command (D57)
    #[arg(long, value_name = "PERSON")]
    pub said_by: Option<String>,
}

/// Runs `accept`: checks who acts (names::said_by_gate, D57) and, unless an LLM carries out its
/// person's decision with `--said-by`, that a person is at a terminal. A person is shown the
/// proposal and asked whether to accept it; a `yes` makes it an approved decision in one
/// transaction, any other answer writes nothing. With `--said-by` nothing is shown or asked. The
/// output is `<ID>: accepted` or `<ID>: nothing changed`; the data is {id, status, accepted}.
pub fn run(args: &Args, g: &Global) -> Result<Output> {
    let ctx = Ctx::load(g, "accept")?;
    let who = ctx.writer()?.clone();
    let said = names::said_by_gate(&who, args.said_by.as_deref(), "accept")?;
    if !said && !term::is_terminal() {
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
        if !said {
            show(&item.entry.text());
            let answer = term::ask(&format!("{id}: accept? (yes/no) "))?;
            if !answer.eq_ignore_ascii_case("yes") {
                return Ok(Output::with(
                    format!("{id}: nothing changed"),
                    json!({"id": id, "status": Status::Proposed.name(), "accepted": false}),
                ));
            }
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
