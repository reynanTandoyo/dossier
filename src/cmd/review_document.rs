//! The `review-document` command (docs/design.md section 12, R-IMPORT-3, R-IMPORT-4): records in
//! imports.md (format.md §25) that an old document is being reviewed, and prints the import rules
//! the LLM drafts by. dossier does not open the document: the LLM reads it itself (R-IMPORT-2).

use crate::blockfile::BlockFile;
use crate::cli::Global;
use crate::ctx::Ctx;
use crate::error::{Code, Error, Result};
use crate::imports;
use crate::names::Who;
use crate::output::Output;
use crate::paths;
use crate::tx::{self, Change};
use serde_json::json;
use std::path::Path;

/// The file of the dossier that lists the documents (format.md §25).
const FILE: &str = "imports.md";

/// The item name of the imports file in snapshots and the journal.
const ITEM: &str = "IMPORTS";

/// The import rules of R-IMPORT-4, printed for the LLM. review-folder prints the same rules.
const RULES: [&str; 7] = [
    "Draft only what a later project would use.",
    "Prefer the report's conclusions and the reasons behind choices over descriptions of what was done.",
    "Record dead ends.",
    "Give the page.",
    "Set confidence to what the document supports, and guess when it only implies something.",
    "Write in English, also when the document isn't.",
    "Never copy customer names or product names into the text of a lesson.",
];

/// The arguments of `review-document`.
#[derive(clap::Args, Debug)]
pub struct Args {
    /// The document, relative to the current folder or absolute, inside the project folder
    pub file: String,
}

/// Runs `review-document`: lists the document in imports.md as `reviewing` (a new block when it
/// is not listed yet), with the history line `reviewing`, and prints the import rules and the
/// command that drafts from the document.
pub fn run(args: &Args, g: &Global) -> Result<Output> {
    let ctx = Ctx::load(g, "review-document")?;
    let who = ctx.writer()?.clone();
    let cwd = std::env::current_dir()?;
    let stored = stored_path(&ctx, &cwd, &args.file)?;
    if !ctx.root.join(paths::shown(&stored)).is_file() {
        return Err(Error::new(
            Code::NotFound,
            format!("no document at {}", args.file),
        ));
    }
    tx::write(&ctx, |tx| {
        let dossier = tx.ctx().dossier.clone();
        let (mut file, old) = imports::load(&dossier, &tx.ctx().cfg.code)?;
        let time = tx.time().to_string();
        record(&mut file, &stored, "reviewing", "reviewing", &time, &who);
        tx.put_raw(
            dossier.join(FILE),
            old,
            file.text().into_bytes(),
            Change {
                id: ITEM.to_string(),
                event: "reviewing".to_string(),
                detail: stored.clone(),
            },
        );
        Ok(Output::with(
            reviewing_text(&stored, &args.file),
            json!({"document": stored, "state": "reviewing", "rules": RULES}),
        ))
    })
}

/// Sets the state of the document's block, adding the block when the document is not listed,
/// and adds one history line `<time> <who>: <event>` to it.
fn record(file: &mut BlockFile, stored: &str, state: &str, event: &str, time: &str, who: &Who) {
    if imports::state(file, stored).is_some() {
        file.set_field(stored, "state", state);
    } else {
        file.add_block(stored, &[("state".to_string(), state.to_string())]);
    }
    file.add_history(stored, time, who, event, "");
}

/// What `review-document` prints: the state recorded, the import rules one per line, and the
/// command that drafts from the document, with the path as the person typed it.
fn reviewing_text(stored: &str, typed: &str) -> String {
    let mut lines = vec![
        format!("{}: reviewing", paths::shown(stored)),
        "import rules:".to_string(),
    ];
    lines.extend(RULES.iter().map(|rule| format!("- {rule}")));
    lines.push(format!("draft with: --from-document {}", quoted(typed)));
    lines.join("\n")
}

/// A path in double quotes when it has a space, so that the command line it is printed in reads
/// back as one argument.
fn quoted(typed: &str) -> String {
    if typed.contains(' ') {
        format!("\"{typed}\"")
    } else {
        typed.to_string()
    }
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
