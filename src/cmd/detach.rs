//! The `detach` command (docs/design.md section 9): a file no longer belongs to a task. Its block
//! in the task's list gets the `detached` mark and a history line, and the block stays with its
//! history, so that what was true on a date can be read back (R-FILE-13, R-LIFE-9, format.md §17).
//! The file itself is not touched, and it need not exist any more.

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

/// The arguments of `detach`.
#[derive(clap::Args, Debug)]
pub struct Args {
    /// The task, such as T-042
    pub task: String,
    /// The path of the file or folder, relative to the current folder or absolute
    pub path: String,
}

/// Runs `detach`: marks the block of the path in the task's file list as detached, in one
/// transaction. The first output line is `<TASK>: detached <path>`.
pub fn run(args: &Args, g: &Global) -> Result<Output> {
    let ctx = Ctx::load(g, "detach")?;
    let who = ctx.writer()?.clone();
    let task = task_id(&args.task)?;
    let cwd = std::env::current_dir()?;
    let stored = stored_path(&ctx, &cwd, &args.path)?;

    tx::write(&ctx, |tx| {
        let dossier = tx.ctx().dossier.clone();
        let time = tx.time().to_string();
        let (mut file, old) = filelist::load(&dossier, &task)?;
        check_attached(&file, &stored, &task)?;

        file.set_field(&stored, "detached", &format!("{time} by {}", who.name));
        file.add_history(&stored, &time, &who, "detached", "");
        tx.put_raw(
            filelist::path(&dossier, &task),
            old,
            file.text().into_bytes(),
            Change {
                id: filelist::item(&task),
                event: "detached".to_string(),
                detail: stored.clone(),
            },
        );

        let text = format!("{task}: detached {}", paths::shown(&stored));
        let data = json!({
            "task": task,
            "event": "detached",
            "path": stored,
            "detached": [stored],
        });
        Ok(Output::with(text, data))
    })
}

/// Code::NotFound when the list has no block for `stored`, or when its block is detached already:
/// only a file that belongs to the task now can be detached.
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
            format!("{} is already detached from {task}", paths::shown(stored)),
        ));
    }
    Ok(())
}

/// Whether the block of `key` has a detached mark.
fn is_detached(file: &BlockFile, key: &str) -> bool {
    file.field(key, "detached").is_some_and(|at| !at.is_empty())
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

/// A typed path as stored (R-XOS-2). The path need not exist any more. The project folder itself
/// is not a file of a task, and a path that begins or ends with a space or has a line break cannot
/// be a block key (Code::BadValue).
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
