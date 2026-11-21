//! The `edit` command: opens an entry in an editor under the write lock (R-EDIT-2). The editor is
//! VISUAL, else EDITOR, else notepad on Windows and vi elsewhere. What the editor saves is checked
//! before anything is written: the lines the tool writes itself (format.md §5) must not change, and
//! an approved lesson changed by an LLM goes back to draft, as `set` does (R-WRITE-8).

use crate::approval;
use crate::cli::Global;
use crate::ctx::Ctx;
use crate::entry::{Entry, Kind, Status};
use crate::error::{Code, Error, Result};
use crate::output::Output;
use crate::store;
use crate::tx::{self, Change};
use serde_json::json;
use std::ffi::{OsStr, OsString};
use std::path::Path;
use std::process::Command;

/// The fields the tool writes itself (format.md §5). An edit that changes one of them is refused.
/// The same list as `set` refuses.
const TOOL_OWNED: [&str; 8] = [
    "status",
    "created",
    "author",
    "via",
    "approved",
    "valid-from",
    "valid-until",
    "history",
];

/// The detail of the history line that an edit adds.
const EDITED: &str = "edited";

/// The detail added to the history line when an LLM's edit takes an approved lesson back to draft.
const BACK_TO_DRAFT: &str = "back to draft";

/// The arguments of `edit`.
#[derive(clap::Args, Debug)]
pub struct Args {
    /// The ID of the entry, such as DEC-012
    pub id: String,
}

/// Runs `edit`: the entry is opened in the editor inside one transaction, so the lock is held
/// while the editor runs. The first output line is `<ID>: changed`, with `(back to draft,
/// waiting for approval)` after it when an LLM's edit took an approved lesson back to draft, or
/// `<ID>: no change` when the editor saved the text as it was.
pub fn run(args: &Args, g: &Global) -> Result<Output> {
    let ctx = Ctx::load(g, "edit")?;
    let who = ctx.writer()?.clone();
    let editor = editor_program();

    tx::write(&ctx, |tx| {
        let item = store::get(&tx.ctx().dossier, &args.id)?;
        let id = item.entry.id();
        let before = item.entry.text();
        let after = edit_in_temp_file(&editor, &id, &before)?;
        if after == before {
            return Ok(Output::with(
                format!("{id}: no change"),
                json!({"id": id, "status": status_name(&item.entry), "changed": false}),
            ));
        }

        let mut entry = Entry::parse(&after)?;
        refuse_tool_lines(&item.entry, &entry)?;

        let approved_lesson =
            entry.kind() == Some(Kind::Lesson) && entry.status() == Some(Status::Approved);
        let mut detail = EDITED.to_string();
        let mut back_to_draft = false;
        if approved_lesson && who.is_llm {
            entry.set_field("status", "draft");
            entry.remove_field("approved");
            entry.remove_field("valid-from");
            detail.push_str(", ");
            detail.push_str(BACK_TO_DRAFT);
            back_to_draft = true;
        }
        entry.add_history(tx.time(), &who, "changed", &detail);

        let text = entry.text();
        if approved_lesson && !tx.ctx().dry_run {
            let state = &tx.ctx().state;
            if who.is_llm {
                approval::forget(state, &id)?;
            } else {
                approval::record(state, &id, text.as_bytes())?;
            }
        }

        let status = status_name(&entry);
        tx.put(
            item.path.clone(),
            Some(&item),
            text,
            Change {
                id: id.clone(),
                event: "changed".to_string(),
                detail,
            },
        );
        let message = if back_to_draft {
            format!("{id}: changed (back to draft, waiting for approval)")
        } else {
            format!("{id}: changed")
        };
        Ok(Output::with(
            message,
            json!({"id": id, "status": status, "changed": true, "back_to_draft": back_to_draft}),
        ))
    })
}

/// The editor program: VISUAL, else EDITOR, each when it is not blank; else notepad on Windows
/// and vi elsewhere.
fn editor_program() -> OsString {
    ["VISUAL", "EDITOR"]
        .into_iter()
        .find_map(|name| {
            std::env::var_os(name).filter(|value| !value.to_string_lossy().trim().is_empty())
        })
        .unwrap_or_else(|| OsString::from(if cfg!(windows) { "notepad" } else { "vi" }))
}

/// Writes `text` to a temporary file named for the entry (`<ID>-<process id>.md`, so that tests
/// running in parallel do not share it), runs the editor on it and waits, then reads the file back
/// and deletes it. Returns the text as the editor saved it.
fn edit_in_temp_file(editor: &OsStr, id: &str, text: &str) -> Result<String> {
    let path = std::env::temp_dir().join(format!("{id}-{}.md", std::process::id()));
    std::fs::write(&path, text)?;
    let edited = run_editor(editor, &path);
    // The file goes whether or not the editor succeeded. A file that cannot be removed does not
    // stop the edit: the text has been read already.
    let _ = std::fs::remove_file(&path);
    edited
}

/// Runs the editor with the path as its one argument, inheriting the terminal, and waits for it.
/// Then reads the file. An editor that cannot start is Code::Io; one that ends with an error
/// (a non-zero exit) is refused, and nothing is written.
fn run_editor(editor: &OsStr, path: &Path) -> Result<String> {
    let status = Command::new(editor).arg(path).status().map_err(|error| {
        Error::new(
            Code::Io,
            format!(
                "cannot run the editor {}: {error}",
                editor.to_string_lossy()
            ),
        )
    })?;
    if !status.success() {
        return Err(Error::new(
            Code::Refused,
            format!("the editor ended with {status}; nothing was changed"),
        ));
    }
    Ok(std::fs::read_to_string(path)?)
}

/// Code::Refused when the edit changed the ID or a line the tool writes (format.md §5), naming
/// them. A changed kind is refused too: the folder and the ID prefix depend on it.
fn refuse_tool_lines(before: &Entry, after: &Entry) -> Result<()> {
    let mut changed: Vec<&str> = Vec::new();
    if before.id() != after.id() {
        changed.push("ID");
    }
    for key in TOOL_OWNED {
        if values(before, key) != values(after, key) {
            changed.push(key);
        }
    }
    if !changed.is_empty() {
        return Err(Error::new(
            Code::Refused,
            format!(
                "these lines are written by the tool: {}",
                changed.join(", ")
            ),
        ));
    }
    if before.kind() != after.kind() {
        return Err(Error::new(
            Code::Refused,
            "the kind of an entry cannot change; add a new entry instead",
        ));
    }
    Ok(())
}

/// Every value of the lines with this key, in file order. Comparing these lists catches a line
/// that is added, removed or changed.
fn values(entry: &Entry, key: &str) -> Vec<String> {
    entry
        .fields()
        .into_iter()
        .filter(|(name, _)| name == key)
        .map(|(_, value)| value)
        .collect()
}

/// The name of the entry's status, or the empty text when it has none.
fn status_name(entry: &Entry) -> &'static str {
    entry.status().map_or("", Status::name)
}
