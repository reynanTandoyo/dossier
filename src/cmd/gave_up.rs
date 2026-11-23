//! The `gave-up` command (docs/design.md section 15, R-INV-4): closes an open investigation
//! without a solution, with the reason. Only an approved investigation whose state is `open` is
//! closed, by a person or an LLM. A refused change writes nothing.

use crate::cli::Global;
use crate::cmd::solved::{ensure_open, ensure_project, one_line_text};
use crate::ctx::Ctx;
use crate::error::Result;
use crate::output::Output;
use crate::store;
use crate::tx::{self, Change};
use serde_json::json;

/// The state of an investigation that was closed without a solution.
const GIVEN_UP: &str = "given up";

/// The arguments of `gave-up`.
#[derive(clap::Args, Debug)]
pub struct Args {
    /// The ID of the investigation, such as INV-003; any case, with or without a place prefix
    pub id: String,
    /// Why it was given up, in one line
    #[arg(long)]
    pub reason: String,
}

/// Runs `gave-up`: checks the reason and the investigation, then, in one transaction, sets the
/// state to `given up`, the closing time and the reason, with a history line and a journal line.
/// The output is `<ID>: given up`, and the data is {id, state}.
pub fn run(args: &Args, g: &Global) -> Result<Output> {
    let ctx = Ctx::load(g, "gave-up")?;
    ensure_project(&ctx)?;
    let who = ctx.writer()?.clone();
    let reason = one_line_text(&args.reason, "--reason")?;

    tx::write(&ctx, |tx| {
        let item = store::get(&tx.ctx().dossier, &args.id)?;
        let id = item.entry.id();
        ensure_open(&item.entry, &id, "gave-up")?;

        let time = tx.time().to_string();
        let mut entry = item.entry.clone();
        entry.set_field("state", GIVEN_UP);
        entry.set_field("closed", &format!("{time} by {}", who.person));
        entry.set_field("given-up-reason", reason);
        entry.add_history(&time, &who, GIVEN_UP, reason);
        tx.put(
            item.path.clone(),
            Some(&item),
            entry.text(),
            Change {
                id: id.clone(),
                event: GIVEN_UP.to_string(),
                detail: reason.to_string(),
            },
        );
        Ok(Output::with(
            format!("{id}: given up"),
            json!({"id": id, "state": GIVEN_UP}),
        ))
    })
}
