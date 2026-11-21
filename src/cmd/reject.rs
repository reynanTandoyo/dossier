//! The `reject` command: a person turns a draft down (R-DRAFT-5). The entry is kept with status
//! `rejected`, so that the same proposal can be recognised later; the reason, when given, goes in
//! the field `rejected-reason` and in the history line. Only a draft can be rejected, and only a
//! person can reject it.

use crate::cli::Global;
use crate::ctx::Ctx;
use crate::entry::Status;
use crate::error::{Code, Error, Result};
use crate::output::Output;
use crate::store;
use crate::tx::{self, Change};
use serde_json::json;

/// The arguments of `reject`.
#[derive(clap::Args, Debug)]
pub struct Args {
    /// The ID of the draft to reject, such as LES-004; any case, with or without a place prefix
    pub id: String,
    /// Why the draft is rejected; kept in the entry
    #[arg(long)]
    pub reason: Option<String>,
}

/// Runs `reject`: checks that a person acts, then, in one transaction, sets the draft to
/// `rejected`, keeps the reason and adds the history line. The output is `<ID>: rejected`.
pub fn run(args: &Args, g: &Global) -> Result<Output> {
    let ctx = Ctx::load(g, "reject")?;
    let who = ctx.writer()?.clone();
    if who.is_llm {
        return Err(Error::new(
            Code::NeedsPerson,
            format!("reject is for a person; {} is an LLM", who.name),
        ));
    }
    let reason = args.reason.as_deref().map(str::trim).unwrap_or("");
    tx::write(&ctx, |tx| {
        let item = store::get(&tx.ctx().dossier, &args.id)?;
        let id = item.entry.id();
        match item.entry.status() {
            Some(Status::Draft) => {}
            Some(status) => {
                return Err(Error::new(
                    Code::Refused,
                    format!("only a draft can be rejected; {id} is {}", status.name()),
                ));
            }
            None => {
                return Err(Error::new(
                    Code::Refused,
                    format!("only a draft can be rejected; {id} has no valid status"),
                ));
            }
        }
        let time = tx.time().to_string();
        let mut entry = item.entry.clone();
        entry.set_field("status", Status::Rejected.name());
        if !reason.is_empty() {
            entry.set_field("rejected-reason", reason);
        }
        entry.add_history(&time, &who, "rejected", reason);
        let text = entry.text();
        tx.put(
            item.path.clone(),
            Some(&item),
            text,
            Change {
                id: id.clone(),
                event: "rejected".to_string(),
                detail: reason.to_string(),
            },
        );
        Ok(Output::with(
            format!("{id}: rejected"),
            json!({"id": id, "status": Status::Rejected.name()}),
        ))
    })
}
