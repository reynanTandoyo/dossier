//! The `reject` command: a person turns a draft down (R-DRAFT-5). The entry is kept with status
//! `rejected`, so that the same proposal can be recognised later; the reason, when given, goes in
//! the field `rejected-reason` and in the history line. Only a draft can be rejected. A person
//! rejects it; an LLM carries out its person's decision with `--said-by <person>` (D57).

use crate::cli::Global;
use crate::ctx::Ctx;
use crate::entry::Status;
use crate::error::{Code, Error, Result};
use crate::names;
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
    /// The person who decided this; needed when an LLM runs the command (D57)
    #[arg(long, value_name = "PERSON")]
    pub said_by: Option<String>,
}

/// Runs `reject`: checks who acts (names::said_by_gate, D57), then, in one transaction, sets the
/// draft to `rejected`, keeps the reason and adds the history line. An LLM runs it with
/// `--said-by <person>` for its person; it needs no terminal and asks nothing. The output is
/// `<ID>: rejected`.
pub fn run(args: &Args, g: &Global) -> Result<Output> {
    let ctx = Ctx::load(g, "reject")?;
    let who = ctx.writer()?.clone();
    names::said_by_gate(&who, args.said_by.as_deref(), "reject")?;
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
