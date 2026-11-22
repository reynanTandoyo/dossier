//! The `reviewed-document` command (docs/design.md section 12, R-IMPORT-3): marks a document that
//! is being reviewed as done in imports.md (format.md §25), or as holding nothing worth keeping
//! with `--nothing`. The document must be listed already: review-document comes first.

use crate::cli::Global;
use crate::ctx::Ctx;
use crate::error::{Code, Error, Result};
use crate::imports;
use crate::output::Output;
use crate::paths;
use crate::tx::{self, Change};
use serde_json::json;
use std::path::Path;

/// The file of the dossier that lists the documents (format.md §25).
const FILE: &str = "imports.md";

/// The item name of the imports file in snapshots and the journal.
const ITEM: &str = "IMPORTS";

/// The arguments of `reviewed-document`.
#[derive(clap::Args, Debug)]
pub struct Args {
    /// The document, as given to review-document
    pub file: String,
    /// The document held nothing worth keeping
    #[arg(long)]
    pub nothing: bool,
}

/// Runs `reviewed-document`: sets the state of the listed document to `done` (history
/// `reviewed`), or to `nothing` with `--nothing` (history `nothing`).
pub fn run(args: &Args, g: &Global) -> Result<Output> {
    let ctx = Ctx::load(g, "reviewed-document")?;
    let who = ctx.writer()?.clone();
    let cwd = std::env::current_dir()?;
    let stored = stored_path(&ctx, &cwd, &args.file)?;
    let (state, event) = if args.nothing {
        ("nothing", "nothing")
    } else {
        ("done", "reviewed")
    };
    tx::write(&ctx, |tx| {
        let dossier = tx.ctx().dossier.clone();
        let (mut file, old) = imports::load(&dossier, &tx.ctx().cfg.code)?;
        if imports::state(&file, &stored).is_none() {
            return Err(Error::new(
                Code::NotFound,
                format!(
                    "{} is not being reviewed: run review-document {} first",
                    paths::shown(&stored),
                    args.file
                ),
            ));
        }
        let time = tx.time().to_string();
        file.set_field(&stored, "state", state);
        file.add_history(&stored, &time, &who, event, "");
        tx.put_raw(
            dossier.join(FILE),
            old,
            file.text().into_bytes(),
            Change {
                id: ITEM.to_string(),
                event: event.to_string(),
                detail: stored.clone(),
            },
        );
        Ok(Output::with(
            format!("{}: {state}", paths::shown(&stored)),
            json!({"document": stored, "state": state}),
        ))
    })
}

/// The stored path of a typed document (R-XOS-2). The project folder itself is not a document,
/// and a path that begins or ends with a space or has a line break cannot be a block key.
fn stored_path(ctx: &Ctx, cwd: &Path, typed: &str) -> Result<String> {
    let stored = paths::stored(&ctx.root, cwd, typed)?;
    if stored.is_empty() {
        return Err(Error::new(
            Code::BadValue,
            "the project folder itself is not a document",
        ));
    }
    if stored.trim() != stored || stored.contains(['\r', '\n']) {
        return Err(Error::new(
            Code::BadValue,
            format!("a path cannot begin or end with a space or have a line break: {typed}"),
        ));
    }
    Ok(stored)
}
