//! The `decline` command: a decider turns a proposal down (R-WRITE-12, D28). Only a person, at a
//! terminal, and a person named in `deciders` (names::may_decide) can decline, and only a proposal
//! can be declined. The declined proposal is kept, hidden like a rejected draft: its status is
//! `declined`, the reason is in `declined-reason`, and the history line `declined` carries it too.

use crate::cli::Global;
use crate::ctx::Ctx;
use crate::entry::Status;
use crate::error::{Code, Error, Result};
use crate::names;
use crate::output::Output;
use crate::store;
use crate::term;
use crate::tx::{self, Change};
use serde_json::json;

/// The history event and the journal event of a declined proposal (format.md §14).
const DECLINED: &str = "declined";

/// The arguments of `decline`.
#[derive(clap::Args, Debug)]
pub struct Args {
    /// The ID of the proposal to decline, such as DEC-012; any case, with or without a place prefix
    pub id: String,
    /// Why the proposal is turned down; kept in the entry
    #[arg(long)]
    pub reason: String,
}

/// Runs `decline`: checks who acts, then, in one transaction, sets the proposal to `declined`,
/// keeps the reason and adds the history line. The output is `<ID>: declined`; the data is
/// {id, status}.
pub fn run(args: &Args, g: &Global) -> Result<Output> {
    let ctx = Ctx::load(g, "decline")?;
    let who = ctx.writer()?.clone();
    if who.is_llm {
        return Err(Error::new(
            Code::NeedsPerson,
            format!("decline is for a person; {} is an LLM", who.name),
        ));
    }
    let reason = args.reason.trim();
    if reason.is_empty() {
        return Err(Error::new(Code::BadValue, "the reason must not be empty"));
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
                    format!("only a proposal can be declined; {id} is {}", status.name()),
                ));
            }
            None => {
                return Err(Error::new(
                    Code::Refused,
                    format!("only a proposal can be declined; {id} has no valid status"),
                ));
            }
        }
        let time = tx.time().to_string();
        let mut entry = item.entry.clone();
        entry.set_field("status", Status::Declined.name());
        entry.set_field("declined-reason", reason);
        entry.add_history(&time, &who, DECLINED, reason);
        let text = entry.text();
        tx.put(
            item.path.clone(),
            Some(&item),
            text,
            Change {
                id: id.clone(),
                event: DECLINED.to_string(),
                detail: reason.to_string(),
            },
        );
        Ok(Output::with(
            format!("{id}: declined"),
            json!({"id": id, "status": Status::Declined.name()}),
        ))
    })
}
