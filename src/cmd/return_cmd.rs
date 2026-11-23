//! The `return` command (docs/design.md section 17, R-FILE-2 and R-FILE-3): `--check` tells whether
//! each taken file is still as it was taken; `--done` records the file's new size, time and checksum
//! and ends the take; `--abandon` ends the take with no new version (format.md §30). Only the person
//! who took a file returns it, unless `--force`. The file itself is not copied here (R-FILE-4).

use crate::blockfile::BlockFile;
use crate::cli::Global;
use crate::ctx::Ctx;
use crate::error::{Code, Error, Result};
use crate::filesum;
use crate::output::Output;
use crate::paths;
use crate::taken::{self, Take};
use crate::tx::{self, Change};
use serde_json::json;
use std::path::Path;

/// The arguments of `return`.
#[derive(clap::Args, Debug)]
pub struct Args {
    /// The files, relative to the current folder or absolute
    #[arg(required = true, value_name = "file")]
    pub files: Vec<String>,
    /// Checks that each file is still as it was taken; changes nothing
    #[arg(long)]
    pub check: bool,
    /// Records the file's new size, time and checksum and ends the take
    #[arg(long)]
    pub done: bool,
    /// Ends the take without a new version
    #[arg(long)]
    pub abandon: bool,
    /// Returns a file that another person took (ignored by --check)
    #[arg(long)]
    pub force: bool,
}

/// The one mode the arguments name.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mode {
    Check,
    Done,
    Abandon,
}

/// Runs `return` in one of its three modes. The first output line is `unchanged: <n> file(s)` for
/// `--check`, `returned: <n> file(s)` for `--done` and `abandoned: <n> file(s)` for `--abandon`;
/// the others are one line per file.
pub fn run(args: &Args, g: &Global) -> Result<Output> {
    let mode = mode(args)?;
    let ctx = Ctx::load(g, "return")?;
    let cwd = std::env::current_dir()?;
    let stored = stored_paths(&ctx, &cwd, &args.files)?;
    match mode {
        Mode::Check => check(&ctx, &stored),
        Mode::Done | Mode::Abandon => end_take(&ctx, mode, args.force, &stored),
    }
}

/// The mode the arguments name: exactly one of `--check`, `--done` and `--abandon`, else
/// Code::Usage.
fn mode(args: &Args) -> Result<Mode> {
    match (args.check, args.done, args.abandon) {
        (true, false, false) => Ok(Mode::Check),
        (false, true, false) => Ok(Mode::Done),
        (false, false, true) => Ok(Mode::Abandon),
        _ => Err(Error::new(
            Code::Usage,
            "give exactly one of --check, --done or --abandon",
        )),
    }
}

/// `--check`: compares each file with its take and writes nothing. Code::Refused naming each file
/// that differs, with what differs, when one does.
fn check(ctx: &Ctx, stored: &[String]) -> Result<Output> {
    let (file, _) = taken::load(&ctx.dossier, &ctx.cfg.code)?;
    let takes = current_takes(&file, stored)?;

    let mut changed: Vec<String> = Vec::new();
    for take in &takes {
        let differences = differences(&ctx.root, take)?;
        if !differences.is_empty() {
            changed.push(format!(
                "  {}: {}",
                paths::shown(&take.path),
                differences.join(", ")
            ));
        }
    }
    if !changed.is_empty() {
        return Err(Error::new(
            Code::Refused,
            format!(
                "changed since taken: {} {}\n{}",
                changed.len(),
                noun(changed.len()),
                changed.join("\n")
            ),
        ));
    }

    let count = takes.len();
    let files: Vec<serde_json::Value> = takes
        .iter()
        .map(|take| json!({"path": take.path, "checksum": take.checksum}))
        .collect();
    Ok(Output::with(
        format!("unchanged: {count} {}", noun(count)),
        json!({"event": "unchanged", "files": files}),
    ))
}

/// What differs between a take and the file on disk now, in the order `size`, `time`, `content`.
/// `missing` alone when the path holds no file now. Empty when the file is as it was taken.
fn differences(root: &Path, take: &Take) -> Result<Vec<&'static str>> {
    let full = root.join(&take.path);
    match std::fs::metadata(&full) {
        Ok(metadata) if metadata.is_file() => {}
        Ok(_) => return Ok(vec!["missing"]),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(vec!["missing"]);
        }
        Err(error) => return Err(Error::new(Code::Io, format!("{}: {error}", full.display()))),
    }
    let stamp = filesum::stamp(&full)?;
    let checksum = filesum::checksum(&full)?;
    let mut found = Vec::new();
    if stamp.size != take.size {
        found.push("size");
    }
    if stamp.modified != take.modified {
        found.push("time");
    }
    if checksum != take.checksum {
        found.push("content");
    }
    Ok(found)
}

/// `--done` and `--abandon`: ends the take of each file in one transaction. Every file must be
/// taken now, and taken by the caller unless `--force`; `--done` also needs the file on disk.
fn end_take(ctx: &Ctx, mode: Mode, force: bool, stored: &[String]) -> Result<Output> {
    let who = ctx.writer()?.clone();
    let done = mode == Mode::Done;
    let event = if done { "returned" } else { "abandoned" };
    tx::write(ctx, |tx| {
        let dossier = tx.ctx().dossier.clone();
        let root = tx.ctx().root.clone();
        let code = tx.ctx().cfg.code.clone();
        let time = tx.time().to_string();
        let (mut file, old) = taken::load(&dossier, &code)?;

        // Every file is checked before any is changed, so that a refusal writes nothing.
        let takes = current_takes(&file, stored)?;
        for take in &takes {
            if take.person != who.person && !force {
                return Err(held_by_other(take));
            }
            if done {
                ensure_file(&root, &take.path)?;
            }
        }

        // Each file: its path and the checksum it has now (the new one for --done).
        let mut rows: Vec<(String, String)> = Vec::new();
        for take in takes {
            let checksum = if done {
                let full = root.join(&take.path);
                let stamp = filesum::stamp(&full)?;
                let checksum = filesum::checksum(&full)?;
                file.set_field(&take.path, "size", &stamp.size.to_string());
                file.set_field(&take.path, "modified", &stamp.modified);
                file.set_field(&take.path, "checksum", &checksum);
                file.add_history(&take.path, &time, &who, event, &checksum);
                checksum
            } else {
                file.add_history(&take.path, &time, &who, event, "");
                take.checksum
            };
            // An empty value removes the field: the take is over.
            file.set_field(&take.path, "taken", "");
            file.set_field(&take.path, "forced", "");
            rows.push((take.path, checksum));
        }

        let detail = rows
            .iter()
            .map(|(path, _)| path.as_str())
            .collect::<Vec<_>>()
            .join(", ");
        tx.put_raw(
            dossier.join(taken::FILE),
            old,
            file.text().into_bytes(),
            Change {
                id: taken::ITEM.to_string(),
                event: event.to_string(),
                detail,
            },
        );

        let mut text = format!("{event}: {} {}", rows.len(), noun(rows.len()));
        for (path, checksum) in &rows {
            text.push_str(&format!("\n  {}  {checksum}", paths::shown(path)));
        }
        let files: Vec<serde_json::Value> = rows
            .iter()
            .map(|(path, checksum)| json!({"path": path, "checksum": checksum}))
            .collect();
        Ok(Output::with(text, json!({"event": event, "files": files})))
    })
}

/// The take of each stored path, in order. Code::Refused `<path> is not taken` for a path that is
/// not taken now.
fn current_takes(file: &BlockFile, stored: &[String]) -> Result<Vec<Take>> {
    stored
        .iter()
        .map(|path| {
            taken::current(file, path).ok_or_else(|| {
                Error::new(
                    Code::Refused,
                    format!("{} is not taken", paths::shown(path)),
                )
            })
        })
        .collect()
}

/// The refusal for a file another person holds (R-FILE-1): the holder, the task and the time.
fn held_by_other(take: &Take) -> Error {
    Error::new(
        Code::Refused,
        format!(
            "{} is taken by {} for {} since {}; only they return it, or add --force",
            paths::shown(&take.path),
            take.person,
            take.task,
            take.time
        ),
    )
}

/// The stored paths of the typed paths, in order, each once (see `stored_path`).
fn stored_paths(ctx: &Ctx, cwd: &Path, typed: &[String]) -> Result<Vec<String>> {
    let mut stored: Vec<String> = Vec::new();
    for path in typed {
        let path = stored_path(ctx, cwd, path)?;
        if !stored.contains(&path) {
            stored.push(path);
        }
    }
    Ok(stored)
}

/// A typed path as stored (R-XOS-2). The project folder itself cannot be returned, and a path that
/// begins or ends with a space or has a line break cannot be a block key (Code::BadValue).
fn stored_path(ctx: &Ctx, cwd: &Path, typed: &str) -> Result<String> {
    let stored = paths::stored(&ctx.root, cwd, typed)?;
    if stored.is_empty() {
        return Err(bad("the project folder itself cannot be returned"));
    }
    if stored.trim() != stored || stored.contains(['\r', '\n']) {
        return Err(bad(format!(
            "a path cannot begin or end with a space or have a line break: {typed}"
        )));
    }
    Ok(stored)
}

/// Refuses `--done` for a stored path that is not a regular file: Code::NotFound when it does not
/// exist, Code::BadValue for a folder. The path is named in the message.
fn ensure_file(root: &Path, stored: &str) -> Result<()> {
    let full = root.join(stored);
    let metadata = std::fs::metadata(&full).map_err(|error| match error.kind() {
        std::io::ErrorKind::NotFound => Error::new(
            Code::NotFound,
            format!("{} does not exist", paths::shown(stored)),
        ),
        _ => Error::new(Code::Io, format!("{}: {error}", full.display())),
    })?;
    if metadata.is_file() {
        return Ok(());
    }
    Err(bad(format!(
        "{} is not a file; return takes files",
        paths::shown(stored)
    )))
}

/// "file" for one, "files" for any other number.
fn noun(count: usize) -> &'static str {
    if count == 1 { "file" } else { "files" }
}

/// A failure for a value the person gave, Code::BadValue.
fn bad(message: impl Into<String>) -> Error {
    Error::new(Code::BadValue, message)
}
