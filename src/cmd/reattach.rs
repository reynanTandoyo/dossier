//! The `reattach` command (docs/design.md section 9): after an intended change, a file's size,
//! time and checksum are recorded again, so that the file list says the file is current (R-FILE-13,
//! format.md §17). The block keeps its role, note and made-from; a history line `reattached` says
//! what changed. A path that is not attached to the task, or is detached from it, is refused.

use crate::blockfile::BlockFile;
use crate::cli::Global;
use crate::ctx::Ctx;
use crate::error::{Code, Error, Result};
use crate::filelist;
use crate::output::Output;
use crate::paths;
use crate::tx::{self, Change};
use serde_json::json;
use std::path::Path;

/// The fields that describe a path and are recorded again (format.md §17). A field that the
/// describe does not give now is removed.
const DESCRIBED: [&str; 5] = ["type", "size", "files", "modified", "checksum"];

/// The arguments of `reattach`.
#[derive(clap::Args, Debug)]
pub struct Args {
    /// The task, such as T-042
    pub task: String,
    /// The path of the file or folder, relative to the current folder or absolute
    pub path: String,
}

/// Runs `reattach`: records the size, time and checksum of the path again in the block of the
/// task's file list, in one transaction. The first output line is `<TASK>: reattached <path>:
/// <what changed>`, where what changed is `size <old> to <new>`, `checksum changed`, `files <old>
/// to <new>` for a folder, or `unchanged`.
pub fn run(args: &Args, g: &Global) -> Result<Output> {
    let ctx = Ctx::load(g, "reattach")?;
    let who = ctx.writer()?.clone();
    let task = task_id(&args.task)?;
    let cwd = std::env::current_dir()?;
    let stored = stored_path(&ctx, &cwd, &args.path)?;

    tx::write(&ctx, |tx| {
        let dossier = tx.ctx().dossier.clone();
        let root = tx.ctx().root.clone();
        let time = tx.time().to_string();
        let (mut file, old) = filelist::load(&dossier, &task)?;
        check_attached(&file, &stored, &task)?;

        let described = filelist::describe(&root, &stored)?;
        let change = change_of(&file, &stored, &described);
        for name in DESCRIBED {
            file.set_field(&stored, name, &value_of(&described, name));
        }
        file.add_history(&stored, &time, &who, "reattached", &change);
        tx.put_raw(
            filelist::path(&dossier, &task),
            old,
            file.text().into_bytes(),
            Change {
                id: filelist::item(&task),
                event: "reattached".to_string(),
                detail: format!("{stored}: {change}"),
            },
        );

        let text = format!("{task}: reattached {}: {change}", paths::shown(&stored));
        let data = json!({
            "task": task,
            "event": "reattached",
            "path": stored,
            "detail": change,
        });
        Ok(Output::with(text, data))
    })
}

/// Code::NotFound when the list has no block for `stored`, or when its block is detached: only a
/// file that belongs to the task now can be reattached (a detached one is attached again first).
fn check_attached(file: &BlockFile, stored: &str, task: &str) -> Result<()> {
    if !file.keys().iter().any(|key| key == stored) {
        return Err(Error::new(
            Code::NotFound,
            format!("{} is not attached to {task}", paths::shown(stored)),
        ));
    }
    if is_detached(file, stored) {
        return Err(Error::new(
            Code::NotFound,
            format!(
                "{} is detached from {task}; attach it again first",
                paths::shown(stored)
            ),
        ));
    }
    Ok(())
}

/// What changed since the block recorded the path, for the history line: a change of type, then
/// the number of files of a folder, then the size, then the checksum; `unchanged` when none of
/// them differs. The modification time alone is not a change.
fn change_of(file: &BlockFile, key: &str, described: &[(String, String)]) -> String {
    let recorded = |name: &str| text_of(file, key, name);
    let now = |name: &str| value_of(described, name);
    if recorded("type") != now("type") {
        return format!("type {} to {}", recorded("type"), now("type"));
    }
    if now("type") == "folder" {
        if recorded("files") != now("files") {
            return format!("files {} to {}", recorded("files"), now("files"));
        }
        return "unchanged".to_string();
    }
    if recorded("size") != now("size") {
        return format!("size {} to {}", recorded("size"), now("size"));
    }
    if recorded("checksum") != now("checksum") {
        return "checksum changed".to_string();
    }
    "unchanged".to_string()
}

/// Whether the block of `key` has a detached mark.
fn is_detached(file: &BlockFile, key: &str) -> bool {
    file.field(key, "detached").is_some_and(|at| !at.is_empty())
}

/// The value of a field of a block, or the empty text.
fn text_of(file: &BlockFile, key: &str, name: &str) -> String {
    file.field(key, name).unwrap_or_default()
}

/// The value of a named field of a described path, or the empty text.
fn value_of(described: &[(String, String)], name: &str) -> String {
    described
        .iter()
        .find(|(field, _)| field == name)
        .map_or_else(String::new, |(_, value)| value.clone())
}

/// The task as the list names it: upper-cased. A task is letters, digits, `-` and `_`, since its
/// name is a file name (Code::BadValue otherwise).
fn task_id(task: &str) -> Result<String> {
    let task = task.trim();
    if task.is_empty()
        || !task
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        return Err(bad(format!("a task is written like T-042: {task}")));
    }
    Ok(task.to_uppercase())
}

/// A typed path as stored (R-XOS-2). The path must exist to be recorded again. The project folder
/// itself is not a file of a task, and a path that begins or ends with a space or has a line break
/// cannot be a block key (Code::BadValue).
fn stored_path(ctx: &Ctx, cwd: &Path, typed: &str) -> Result<String> {
    let stored = paths::stored(&ctx.root, cwd, typed)?;
    if stored.is_empty() {
        return Err(bad("the project folder itself is not a file of a task"));
    }
    if stored.trim() != stored || stored.contains(['\r', '\n']) {
        return Err(bad(format!(
            "a path cannot begin or end with a space or have a line break: {typed}"
        )));
    }
    Ok(stored)
}

/// A failure for a value the person gave, Code::BadValue.
fn bad(message: impl Into<String>) -> Error {
    Error::new(Code::BadValue, message)
}
