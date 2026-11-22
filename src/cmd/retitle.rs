//! The `retitle` command: changes the title of an entry and renames its file to the slug of the new
//! title (R-WRITE-8, R-ID-3). The ID stays. As `set` does, an LLM's change to an approved lesson,
//! how-to or tool takes it back to draft (approval::guarded), and a person's change to one is
//! recorded as its new approved text.

use crate::approval;
use crate::cli::Global;
use crate::ctx::Ctx;
use crate::entry::{Kind, Status};
use crate::error::{Code, Error, Result};
use crate::output::Output;
use crate::store;
use crate::tx::{self, Change};
use serde_json::json;

/// The part of the entry that changed, as the history line names it.
const TITLE: &str = "title";

/// Added to the detail of the history line when an LLM's change takes an approved lesson, how-to
/// or tool back to draft.
const BACK_TO_DRAFT: &str = "back to draft";

/// The arguments of `retitle`.
#[derive(clap::Args, Debug)]
pub struct Args {
    /// The ID of the entry, such as DEC-012
    pub id: String,
    /// The new title
    pub title: String,
}

/// Runs `retitle`: sets the new title and moves the file to the slug of that title, in one
/// transaction. The first output line is `<ID>: retitled`, with `(back to draft, waiting for
/// approval)` after it when an LLM took an approved lesson, how-to or tool back to draft, or
/// `<ID>: no change` when the entry already has this title. A blank title is Code::BadValue.
pub fn run(args: &Args, g: &Global) -> Result<Output> {
    let ctx = Ctx::load(g, "retitle")?;
    let title = args.title.trim();
    if title.is_empty() {
        return Err(Error::new(Code::BadValue, "the title must not be empty"));
    }
    let who = ctx.writer()?.clone();

    // The ID and the new text of an approved lesson, how-to or tool that changed. Its checksum is
    // written after the file, so that a refused change leaves the checksum alone.
    let mut approved: Option<(String, String)> = None;
    let output = tx::write(&ctx, |tx| {
        let item = store::get(&tx.ctx().dossier, &args.id)?;
        let mut entry = item.entry.clone();
        let before = entry.text();
        entry.set_title(title);
        let id = entry.id();
        if entry.text() == before {
            return Ok(Output::with(
                format!("{id}: no change"),
                json!({"id": id, "changed": []}),
            ));
        }

        let kind = entry.kind().or_else(|| Kind::of_id(&id)).ok_or_else(|| {
            Error::new(
                Code::Parse,
                format!("{id} has no kind, so its file cannot be named"),
            )
        })?;
        let approved_guarded = approval::guarded(kind) && entry.status() == Some(Status::Approved);
        let mut detail = TITLE.to_string();
        let mut back_to_draft = false;
        if approved_guarded && who.is_llm {
            entry.set_field("status", Status::Draft.name());
            entry.remove_field("approved");
            entry.remove_field("valid-from");
            detail.push_str(", ");
            detail.push_str(BACK_TO_DRAFT);
            back_to_draft = true;
        }
        entry.add_history(tx.time(), &who, "changed", &detail);

        let text = entry.text();
        if approved_guarded {
            approved = Some((id.clone(), text.clone()));
        }
        // The file is named by its kind, its ID and the slug of its title (format.md §4).
        let new_path = store::path_for(&tx.ctx().dossier, kind, &id, &entry.title());
        let change = Change {
            id: id.clone(),
            event: "changed".to_string(),
            detail,
        };
        if new_path == item.path {
            tx.put(item.path.clone(), Some(&item), text, change);
        } else {
            tx.rename(&item, new_path, text, change);
        }

        let message = if back_to_draft {
            format!("{id}: retitled (back to draft, waiting for approval)")
        } else {
            format!("{id}: retitled")
        };
        let status = entry.status().map_or("", Status::name);
        Ok(Output::with(
            message,
            json!({"id": id, "title": entry.title(), "status": status}),
        ))
    })?;

    if let Some((id, text)) = approved.filter(|_| !ctx.dry_run) {
        if who.is_llm {
            approval::forget(&ctx.state, &id)?;
        } else {
            approval::record(&ctx.state, &id, text.as_bytes())?;
        }
    }
    Ok(output)
}
