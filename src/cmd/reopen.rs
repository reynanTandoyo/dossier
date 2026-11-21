//! The `reopen` command: takes an entry back one step (R-LIFE-1). An approved or rejected entry
//! goes back to draft, so it is no longer knowledge until it is approved again. A retired entry
//! is in force again as approved, and only a person can do that. A draft and a superseded entry
//! cannot be reopened. A lesson that is approved again gets its checksum recorded (R-LIFE-6), and
//! a lesson that goes back to draft loses its checksum.

use crate::approval;
use crate::cli::Global;
use crate::ctx::Ctx;
use crate::entry::{Kind, Status};
use crate::error::{Code, Error, Result};
use crate::output::Output;
use crate::store;
use crate::tx::{self, Change};
use serde_json::json;

/// The history detail of an entry that goes back to draft.
const BACK_TO_DRAFT: &str = "back to draft";
/// The history detail of a retired entry that is in force again.
const IN_FORCE_AGAIN: &str = "in force again";

/// The arguments of `reopen`.
#[derive(clap::Args, Debug)]
pub struct Args {
    /// The ID of the entry to reopen, such as LES-004; any case, with or without a place prefix
    pub id: String,
}

/// Runs `reopen`: in one transaction, an approved or rejected entry becomes a draft, and a
/// retired entry becomes approved again, with a history line for the change. The output is
/// `<ID>: reopened (<new status>)`. The checksum of a lesson follows the change once the file is
/// written, unless this is a dry run.
pub fn run(args: &Args, g: &Global) -> Result<Output> {
    let ctx = Ctx::load(g, "reopen")?;
    let who = ctx.writer()?.clone();

    // The checksum changes only after the file is written, so that a refused change leaves it
    // alone (as `retire` does).
    let mut forget: Option<String> = None;
    let mut record: Option<(String, Vec<u8>)> = None;
    let output = tx::write(&ctx, |tx| {
        let item = store::get(&tx.ctx().dossier, &args.id)?;
        let id = item.entry.id();
        let mut entry = item.entry.clone();
        let (status, detail) = match item.entry.status() {
            Some(Status::Approved | Status::Rejected) => {
                entry.set_field("status", Status::Draft.name());
                entry.remove_field("approved");
                entry.remove_field("valid-from");
                entry.remove_field("rejected-reason");
                (Status::Draft, BACK_TO_DRAFT)
            }
            Some(Status::Retired) => {
                if who.is_llm {
                    return Err(Error::new(
                        Code::NeedsPerson,
                        format!(
                            "reopening a retired entry is for a person; {} is an LLM",
                            who.name
                        ),
                    ));
                }
                entry.set_field("status", Status::Approved.name());
                entry.remove_field("valid-until");
                entry.remove_field("retired-reason");
                (Status::Approved, IN_FORCE_AGAIN)
            }
            Some(status) => {
                return Err(Error::new(
                    Code::Refused,
                    format!("a {} entry cannot be reopened", status.name()),
                ));
            }
            None => {
                return Err(Error::new(
                    Code::Refused,
                    format!("{id} has no valid status, so it cannot be reopened"),
                ));
            }
        };
        let time = tx.time().to_string();
        entry.add_history(&time, &who, "reopened", detail);
        let text = entry.text();
        if !tx.ctx().dry_run {
            match status {
                Status::Approved => {
                    if entry.kind().or_else(|| Kind::of_id(&id)) == Some(Kind::Lesson) {
                        record = Some((id.clone(), text.as_bytes().to_vec()));
                    }
                }
                _ => forget = Some(id.clone()),
            }
        }
        tx.put(
            item.path.clone(),
            Some(&item),
            text,
            Change {
                id: id.clone(),
                event: "reopened".to_string(),
                detail: detail.to_string(),
            },
        );
        Ok(Output::with(
            format!("{id}: reopened ({})", status.name()),
            json!({"id": id, "status": status.name()}),
        ))
    })?;

    if !ctx.dry_run {
        if let Some(id) = forget {
            approval::forget(&ctx.state, &id)?;
        }
        if let Some((id, bytes)) = record {
            approval::record(&ctx.state, &id, &bytes)?;
        }
    }
    Ok(output)
}
