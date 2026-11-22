//! The `review-folder` command (R-IMPORT-2, R-IMPORT-4; docs/design.md section 12): the material an
//! LLM needs to draft entries from an old project folder. It prints the folder tree, the documents
//! found with the review state each has in imports.md, how many are already reviewed, the import
//! rules and the commands that record a review. The documents are not opened here: the LLM opens
//! the ones it needs. Nothing is changed.

use crate::cli::Global;
use crate::ctx::Ctx;
use crate::error::Result;
use crate::folderscan::{self, Doc};
use crate::imports;
use crate::output::Output;
use crate::paths;
use crate::time;
use serde_json::{Value, json};
use std::path::{Path, PathBuf};

/// The import rules of R-IMPORT-4, in the order of the requirement, one per line.
const RULES: [&str; 7] = [
    "Draft only what a later project would use.",
    "Prefer the report's conclusions and the reasons behind choices over descriptions of what was done.",
    "Record dead ends.",
    "Give the page the entry comes from.",
    "Set confidence to what the document supports (measured, simulated, experience or guess), and use guess when it only implies something.",
    "Write in English, also when the document is not in English.",
    "Never copy customer names or product names into the text of a lesson.",
];

/// The commands that record a review of a document and an entry drafted from it (R-IMPORT-3).
const EXAMPLES: [&str; 2] = [
    "dossier review-document <file>",
    "dossier learn <statement> --valid-for <text> --evidence <text> --confidence <level> --from-document <file>",
];

/// The arguments of `review-folder`.
#[derive(clap::Args, Debug)]
pub struct Args {
    /// The folder to review: the folder the settings describe, else the project folder
    pub path: Option<PathBuf>,
    /// How many levels of folders the tree shows
    #[arg(long, default_value_t = 3)]
    pub depth: usize,
}

/// A document of the folder, with its key in imports.md and the state it has there.
struct Listed {
    /// The path from the base folder (the folder the settings describe, else the project folder),
    /// with `/`: the form imports.md uses.
    key: String,
    doc: Doc,
    /// `reviewing`, `done` or `nothing`, or "" for a listed document without a state. None when
    /// the document is not listed in imports.md.
    state: Option<String>,
}

/// Runs `review-folder`: the tree and the documents of the folder, then how many are already
/// reviewed, the import rules and the example commands. Nothing is written, and no name is needed.
pub fn run(args: &Args, g: &Global) -> Result<Output> {
    let ctx = Ctx::load(g, "review-folder")?;
    let cwd = std::env::current_dir()?;
    let base = base_folder(&ctx);
    let folder = match &args.path {
        Some(path) => cwd.join(path),
        None => base.clone(),
    };
    let typed = folder.to_string_lossy().into_owned();
    // The keys of imports.md are paths from the base folder. A folder outside it is refused, as
    // review-document refuses a file outside it.
    let prefix = paths::stored(&base, &cwd, &typed)?;
    let shown = shown_folder(&ctx.root, &cwd, &typed);

    let tree = folderscan::tree(&folder, args.depth)?;
    let (imports, _) = imports::load(&ctx.dossier, &ctx.cfg.code)?;
    let documents: Vec<Listed> = folderscan::documents(&folder)?
        .into_iter()
        .map(|doc| {
            let key = join(&prefix, &doc.stored);
            let state = imports::state(&imports, &key);
            Listed { key, doc, state }
        })
        .collect();
    let reviewed = documents
        .iter()
        .filter(|listed| matches!(listed.state.as_deref(), Some("done" | "nothing")))
        .count();
    let being = documents
        .iter()
        .filter(|listed| listed.state.as_deref() == Some("reviewing"))
        .count();
    let document_lines: Vec<String> = documents.iter().map(document_line).collect();

    let mut lines = vec![
        format!("# Folder {shown}"),
        String::new(),
        "## Tree".to_string(),
    ];
    lines.extend(or_none(&tree));
    lines.extend([String::new(), "## Documents".to_string()]);
    lines.extend(or_none(&document_lines));
    lines.extend([
        String::new(),
        "## Already reviewed".to_string(),
        format!(
            "{reviewed} of {} documents reviewed (done or nothing), {being} being reviewed.",
            documents.len()
        ),
        String::new(),
        "## Import rules".to_string(),
    ]);
    lines.extend(
        RULES
            .iter()
            .enumerate()
            .map(|(index, rule)| format!("{}. {rule}", index + 1)),
    );
    lines.extend([String::new(), "## Examples".to_string()]);
    lines.extend(EXAMPLES.iter().map(|example| (*example).to_string()));

    let data = json!({
        "folder": shown,
        "tree": tree,
        "documents": documents.iter().map(document_data).collect::<Vec<Value>>(),
        "reviewed": reviewed,
        "being_reviewed": being,
        "rules": RULES,
        "examples": EXAMPLES,
    });
    Ok(Output::with(lines.join("\n"), data))
}

/// The folder the settings describe (`describes`, written by `init --in`), else the project
/// folder.
fn base_folder(ctx: &Ctx) -> PathBuf {
    match &ctx.cfg.describes {
        Some(described) => ctx.root.join(paths::shown(described)),
        None => ctx.root.clone(),
    }
}

/// The folder as shown to people: relative to the project folder when it is inside it (`.` for
/// the project folder itself), else in full.
fn shown_folder(root: &Path, cwd: &Path, folder: &str) -> String {
    match paths::stored(root, cwd, folder) {
        Ok(stored) if stored.is_empty() => ".".to_string(),
        Ok(stored) => paths::shown(&stored),
        Err(_) => folder.to_string(),
    }
}

/// The key of a document under the folder with key `prefix` ("" for the base folder itself).
fn join(prefix: &str, stored: &str) -> String {
    if prefix.is_empty() {
        stored.to_string()
    } else {
        format!("{prefix}/{stored}")
    }
}

/// One document as `<path>  <kind>  <size>  <modified date>  [<state>]`, the path shown for this
/// system. The size is in bytes and the date is the local date of the last change.
fn document_line(listed: &Listed) -> String {
    format!(
        "{}  {}  {}  {}  [{}]",
        paths::shown(&listed.key),
        listed.doc.kind,
        listed.doc.size,
        time::date(&listed.doc.modified),
        state_text(listed.state.as_deref()),
    )
}

/// The state of a document as the listing shows it.
fn state_text(state: Option<&str>) -> &str {
    match state {
        None => "not reviewed",
        Some("") => "no state",
        Some(state) => state,
    }
}

/// One document as data: its key, kind, size, modified time and state (null when not listed).
fn document_data(listed: &Listed) -> Value {
    json!({
        "stored": listed.key,
        "kind": listed.doc.kind,
        "size": listed.doc.size,
        "modified": listed.doc.modified,
        "state": listed.state,
    })
}

/// The lines given, or the single line `none` when there are none.
fn or_none(lines: &[String]) -> Vec<String> {
    if lines.is_empty() {
        vec!["none".to_string()]
    } else {
        lines.to_vec()
    }
}
