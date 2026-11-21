//! The `supersede` command: replaces an approved entry with a newer approved entry of the same
//! kind. The old entry is closed with the time it was superseded and points to the new one, and
//! the new one points back. Nothing is deleted or overwritten: both files keep their history
//! (R-LIFE-4, R-ENTRY-14, D20). Only a person can do it.

use crate::cli::Global;
use crate::ctx::Ctx;
use crate::entry::{Entry, Kind, Status};
use crate::error::{Code, Error, Result};
use crate::output::Output;
use crate::store::{self, Item};
use crate::tx::{self, Change};
use serde_json::json;

/// The arguments of `supersede`.
#[derive(clap::Args, Debug)]
pub struct Args {
    /// The entry that is replaced; it must be approved
    pub old: String,
    /// The newer entry that replaces it; it must be approved and of the same kind
    #[arg(long)]
    pub by: String,
}

/// Runs `supersede`: closes the old entry and links it with the new one in one transaction. The
/// output is `<old ID>: superseded by <new ID>`.
pub fn run(args: &Args, g: &Global) -> Result<Output> {
    let ctx = Ctx::load(g, "supersede")?;
    let who = ctx.writer()?.clone();
    if who.is_llm {
        return Err(Error::new(
            Code::NeedsPerson,
            format!("supersede is for a person; {} is an LLM", who.name),
        ));
    }
    tx::write(&ctx, |tx| {
        let old = store::get(&tx.ctx().dossier, &args.old)?;
        let new = store::get(&tx.ctx().dossier, &args.by)?;
        check(&old, &new)?;
        let old_id = old.entry.id();
        let new_id = new.entry.id();
        let time = tx.time().to_string();

        // The old entry: closed, with the time and the ID of its replacement. Its valid-until
        // is set before superseded-by, the order of format.md §5.
        let mut closed = old.entry.clone();
        closed.set_field("status", Status::Superseded.name());
        closed.set_field("valid-until", &time);
        closed.set_field("superseded-by", &new_id);
        let detail = format!("by {new_id}");
        closed.add_history(&time, &who, "superseded", &detail);
        tx.put(
            old.path.clone(),
            Some(&old),
            closed.text(),
            Change {
                id: old_id.clone(),
                event: "superseded".to_string(),
                detail,
            },
        );

        // The new entry: it points back to the one it replaces.
        let mut replacing = new.entry.clone();
        replacing.set_field("supersedes", &old_id);
        let detail = format!("supersedes {old_id}");
        replacing.add_history(&time, &who, "changed", &detail);
        tx.put(
            new.path.clone(),
            Some(&new),
            replacing.text(),
            Change {
                id: new_id.clone(),
                event: "changed".to_string(),
                detail,
            },
        );

        Ok(Output::with(
            format!("{old_id}: superseded by {new_id}"),
            json!({"old": old_id, "new": new_id}),
        ))
    })
}

/// What must hold before the two entries are linked: they are not the same entry, they are of
/// the same kind, and both are approved.
fn check(old: &Item, new: &Item) -> Result<()> {
    let old_id = old.entry.id();
    let new_id = new.entry.id();
    if old_id == new_id {
        return Err(Error::new(
            Code::BadValue,
            format!("{old_id} cannot supersede itself: give the ID of the newer entry"),
        ));
    }
    if old.entry.kind() != new.entry.kind() {
        return Err(Error::new(
            Code::Refused,
            format!(
                "both entries must be of the same kind ({old_id} is {}, {new_id} is {})",
                kind_of(&old.entry),
                kind_of(&new.entry)
            ),
        ));
    }
    if old.entry.status() != Some(Status::Approved) {
        return Err(Error::new(
            Code::Refused,
            format!(
                "{old_id} is not approved (status: {}); only an approved entry can be superseded",
                status_of(&old.entry)
            ),
        ));
    }
    if new.entry.status() != Some(Status::Approved) {
        return Err(Error::new(
            Code::Refused,
            format!(
                "{new_id} is not approved (status: {}); the entry that replaces it must be approved",
                status_of(&new.entry)
            ),
        ));
    }
    Ok(())
}

/// The kind of an entry as its name, or a phrase for a file without one.
fn kind_of(entry: &Entry) -> &'static str {
    entry.kind().map_or("without a kind", Kind::name)
}

/// The status of an entry as its name, or a phrase for a file without one.
fn status_of(entry: &Entry) -> &'static str {
    entry.status().map_or("without a status", Status::name)
}
