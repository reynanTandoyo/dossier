//! The `remove` command: a person at a terminal deletes an entry for good (R-LIFE-3), for example
//! customer data entered by mistake. The file is deleted, the journal records the removal without
//! the text, and the snapshots of the entry are deleted too, so that no copy of its text remains.
//! Its ID is not reused, since ids.json keeps the highest number given out (R-ID-3). The person
//! types the ID to confirm; anything else changes nothing.

use crate::approval;
use crate::cli::Global;
use crate::ctx::Ctx;
use crate::error::{Code, Error, Result};
use crate::output::Output;
use crate::store;
use crate::term;
use crate::tx::{self, Change};
use serde_json::json;
use std::path::Path;

/// The arguments of `remove`.
#[derive(clap::Args, Debug)]
pub struct Args {
    /// The ID of the entry to delete for good, such as FND-004; any case, with or without a place
    /// prefix
    pub id: String,
    /// Confirms the deletion; without it nothing is deleted
    #[arg(long)]
    pub yes: bool,
}

/// Runs `remove`: checks that a person acts at a terminal and that `--yes` is given, asks for the
/// ID to be typed, then deletes the entry in one transaction. The output is `<ID>: removed`. When
/// the answer is not the ID, nothing changes and the output is `nothing changed`.
pub fn run(args: &Args, g: &Global) -> Result<Output> {
    let ctx = Ctx::load(g, "remove")?;
    let who = ctx.writer()?.clone();
    if who.is_llm {
        return Err(Error::new(
            Code::NeedsPerson,
            format!("remove is for a person; {} is an LLM", who.name),
        ));
    }
    if !term::is_terminal() {
        return Err(Error::new(
            Code::NeedsTerminal,
            "this command must be run by a person at a terminal",
        ));
    }
    if !args.yes {
        return Err(Error::new(
            Code::Usage,
            "remove deletes the entry for good; add --yes",
        ));
    }

    // The ID as the entry has it, read before the question so that an unknown ID is refused
    // without asking anything.
    let id = store::get(&ctx.dossier, &args.id)?.entry.id();
    let answer = term::ask(&format!("type the ID to delete {id} for good: "))?;
    if !answer.eq_ignore_ascii_case(&id) {
        return Ok(Output::with(
            "nothing changed",
            json!({"id": id, "removed": false}),
        ));
    }

    let output = tx::write(&ctx, |tx| {
        let item = store::get(&tx.ctx().dossier, &id)?;
        // A dry run writes nothing, so the checksums are left as they are.
        if !tx.ctx().dry_run {
            approval::forget(&tx.ctx().state, &id)?;
        }
        tx.delete(
            &item,
            Change {
                id: id.clone(),
                event: "removed".to_string(),
                detail: String::new(),
            },
        );
        Ok(Output::with(
            format!("{id}: removed"),
            json!({"id": id, "removed": true}),
        ))
    })?;

    // The write saved a snapshot of the file it deleted; that copy of the text goes too.
    if !ctx.dry_run {
        remove_snapshots(&ctx.state, &id)?;
    }
    Ok(output)
}

/// Deletes the snapshot folder of entry `id`, `state/history/<ID>` with the ID in upper case as
/// snapshot.rs names it. A folder that does not exist is fine. Another failure is reported with
/// the entry already removed, so that the person knows a copy of the text is still there.
fn remove_snapshots(state: &Path, id: &str) -> Result<()> {
    let folder = state.join("history").join(id.to_ascii_uppercase());
    match std::fs::remove_dir_all(&folder) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(Error::new(
            Code::Io,
            format!(
                "{id} is removed, but its snapshots could not be deleted: {}: {error}",
                folder.display()
            ),
        )),
    }
}
