//! The `retire` command: a person ends an approved entry without a replacement (R-LIFE-5). The
//! entry is kept with status `retired`; `valid-until` is the time it was retired, and
//! `retired-reason` and the history line carry the reason. Only an approved entry can be retired,
//! and only a person can retire it. A retired lesson is no longer approved, so its recorded
//! checksum is forgotten.

use crate::approval;
use crate::cli::Global;
use crate::ctx::Ctx;
use crate::entry::{Kind, Status};
use crate::error::{Code, Error, Result};
use crate::output::Output;
use crate::store;
use crate::tx::{self, Change};
use serde_json::json;

/// The arguments of `retire`.
#[derive(clap::Args, Debug)]
pub struct Args {
    /// The ID of the approved entry to retire, such as DEC-012; any case, with or without a place
    /// prefix
    pub id: String,
    /// Why the entry no longer holds; kept in the entry and in its history
    #[arg(long)]
    pub reason: String,
}

/// Runs `retire`: checks that a person acts and that the reason is not blank, then, in one
/// transaction, closes the entry with the time and the reason. The output is `<ID>: retired`.
/// The checksum of a retired lesson is forgotten after the file is written, unless this is a
/// dry run.
pub fn run(args: &Args, g: &Global) -> Result<Output> {
    let ctx = Ctx::load(g, "retire")?;
    let who = ctx.writer()?.clone();
    if who.is_llm {
        return Err(Error::new(
            Code::NeedsPerson,
            format!("retire is for a person; {} is an LLM", who.name),
        ));
    }
    let reason = args.reason.trim();
    if reason.is_empty() {
        return Err(Error::new(Code::BadValue, "the reason must not be empty"));
    }

    // The ID of a retired lesson. Its checksum is forgotten after the file is written, so that
    // a refused change leaves the checksum alone.
    let mut lesson: Option<String> = None;
    let output = tx::write(&ctx, |tx| {
        let item = store::get(&tx.ctx().dossier, &args.id)?;
        let id = item.entry.id();
        match item.entry.status() {
            Some(Status::Approved) => {}
            Some(status) => {
                return Err(Error::new(
                    Code::Refused,
                    format!(
                        "only an approved entry can be retired; {id} is {}",
                        status.name()
                    ),
                ));
            }
            None => {
                return Err(Error::new(
                    Code::Refused,
                    format!("only an approved entry can be retired; {id} has no valid status"),
                ));
            }
        }
        let time = tx.time().to_string();
        let mut entry = item.entry.clone();
        entry.set_field("status", Status::Retired.name());
        entry.set_field("valid-until", &time);
        entry.set_field("retired-reason", reason);
        entry.add_history(&time, &who, "retired", reason);
        if entry.kind().or_else(|| Kind::of_id(&id)) == Some(Kind::Lesson) {
            lesson = Some(id.clone());
        }
        tx.put(
            item.path.clone(),
            Some(&item),
            entry.text(),
            Change {
                id: id.clone(),
                event: "retired".to_string(),
                detail: reason.to_string(),
            },
        );
        Ok(Output::with(
            format!("{id}: retired"),
            json!({"id": id, "status": Status::Retired.name()}),
        ))
    })?;

    if let Some(id) = lesson.filter(|_| !ctx.dry_run) {
        approval::forget(&ctx.state, &id)?;
    }
    Ok(output)
}
