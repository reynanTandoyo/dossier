//! The `take` command (docs/design.md section 17, R-FILE-1 to R-FILE-7): records who takes a large
//! file for a task, with its size, modification time and checksum, in `taken.md` (format.md §30).
//! A file that another person holds is refused unless `--force`, which takes it anyway and records
//! that. The file itself is not copied here (R-FILE-4).

use crate::blockfile::BlockFile;
use crate::cli::Global;
use crate::ctx::Ctx;
use crate::error::{Code, Error, Result};
use crate::filesum;
use crate::names::Who;
use crate::output::Output;
use crate::paths;
use crate::taken::{self, Take};
use crate::tx::{self, Change};
use serde_json::json;
use std::path::Path;

/// The arguments of `take`.
#[derive(clap::Args, Debug)]
pub struct Args {
    /// The files, relative to the current folder or absolute
    #[arg(required = true, value_name = "file")]
    pub files: Vec<String>,
    /// The task the files are taken for, such as T-042
    #[arg(long, value_name = "task")]
    pub task: String,
    /// Takes files that another person holds; such a take is recorded as forced
    #[arg(long)]
    pub force: bool,
}

/// What `take` does with one file: take it now, or leave it as the caller already holds it.
enum Step {
    /// Taken now. `forced_from` is the person whose take `--force` overrode.
    Take { forced_from: Option<String> },
    /// Already taken by the caller: nothing changes, and the take as recorded is reported.
    Already(Take),
}

/// One file as the output and the record show it.
struct Row {
    path: String,
    size: u64,
    modified: String,
    checksum: String,
    /// Taken already by the caller, so left as recorded.
    already: bool,
}

/// Runs `take`: records the files as taken for the task in one transaction. The first output line
/// is `taken for <TASK>: <n> file(s)`, then one line per file.
pub fn run(args: &Args, g: &Global) -> Result<Output> {
    let ctx = Ctx::load(g, "take")?;
    let who = ctx.writer()?.clone();
    let task = task_id(&args.task)?;
    let cwd = std::env::current_dir()?;
    let stored = stored_paths(&ctx, &cwd, &args.files)?;

    tx::write(&ctx, |tx| {
        let dossier = tx.ctx().dossier.clone();
        let root = tx.ctx().root.clone();
        let code = tx.ctx().cfg.code.clone();
        let time = tx.time().to_string();
        let (mut file, old) = taken::load(&dossier, &code)?;

        let mut warnings: Vec<String> = stored
            .iter()
            .filter_map(|path| windows_warning(path))
            .collect();
        warnings.extend(case_warnings(&stored));

        // Every file is checked before any is hashed, so that a refusal is quick and writes nothing.
        let mut steps = Vec::new();
        for path in &stored {
            ensure_file(&root, path)?;
            let step = match taken::current(&file, path) {
                Some(take) if take.person == who.person => Step::Already(take),
                Some(take) if !args.force => return Err(refused(path, &take)),
                Some(take) => Step::Take {
                    forced_from: Some(take.person),
                },
                None => Step::Take { forced_from: None },
            };
            steps.push((path.clone(), step));
        }

        let mut rows = Vec::new();
        let mut changed = Vec::new();
        for (path, step) in steps {
            match step {
                Step::Already(take) => rows.push(Row {
                    path,
                    size: take.size,
                    modified: take.modified,
                    checksum: take.checksum,
                    already: true,
                }),
                Step::Take { forced_from } => {
                    let full = root.join(&path);
                    let stamp = filesum::stamp(&full)?;
                    let row = Row {
                        path,
                        size: stamp.size,
                        modified: stamp.modified,
                        checksum: filesum::checksum(&full)?,
                        already: false,
                    };
                    record(&mut file, &row, &task, forced_from.as_deref(), &time, &who);
                    changed.push(row.path.clone());
                    rows.push(row);
                }
            }
        }
        if !changed.is_empty() {
            tx.put_raw(
                dossier.join(taken::FILE),
                old,
                file.text().into_bytes(),
                Change {
                    id: taken::ITEM.to_string(),
                    event: "taken".to_string(),
                    detail: changed.join(", "),
                },
            );
        }

        let mut text = format!("taken for {task}: {} {}", rows.len(), noun(rows.len()));
        for row in &rows {
            if row.already {
                text.push_str(&format!(
                    "\n  {}  already taken by you",
                    paths::shown(&row.path)
                ));
            } else {
                text.push_str(&format!(
                    "\n  {}  {} bytes  {}",
                    paths::shown(&row.path),
                    row.size,
                    row.checksum
                ));
            }
        }
        let files: Vec<serde_json::Value> = rows
            .iter()
            .map(|row| {
                json!({
                    "path": row.path,
                    "size": row.size,
                    "modified": row.modified,
                    "checksum": row.checksum,
                    "already": row.already,
                })
            })
            .collect();
        let data = json!({"task": task, "files": files});
        Ok(warnings
            .into_iter()
            .fold(Output::with(text, data), |output, warning| {
                output.warn(warning)
            }))
    })
}

/// Records one take in the block of `row.path`: the block is added when it is missing. Its `taken`,
/// `task`, `size`, `modified` and `checksum` fields are set, and `forced` is `yes` only for a take
/// over another person's (else it is removed). One history line: `taken`, or `taken by force` with
/// the person the file was taken from.
fn record(
    file: &mut BlockFile,
    row: &Row,
    task: &str,
    forced_from: Option<&str>,
    time: &str,
    who: &Who,
) {
    let fields = vec![
        pair("taken", &format!("{time} by {}", who.person)),
        pair("task", task),
        pair("size", &row.size.to_string()),
        pair("modified", &row.modified),
        pair("checksum", &row.checksum),
        pair("forced", if forced_from.is_some() { "yes" } else { "" }),
    ];
    if file.keys().contains(&row.path) {
        for (name, value) in &fields {
            file.set_field(&row.path, name, value);
        }
    } else {
        file.add_block(&row.path, &fields);
    }
    let (event, detail) = match forced_from {
        Some(from) => (
            "taken by force",
            format!("{task} {}, from {from}", row.checksum),
        ),
        None => ("taken", format!("{task} {}", row.checksum)),
    };
    file.add_history(&row.path, time, who, event, &detail);
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

/// A typed path as stored (R-XOS-2). The project folder itself cannot be taken, and a path that
/// begins or ends with a space or has a line break cannot be a block key (Code::BadValue).
fn stored_path(ctx: &Ctx, cwd: &Path, typed: &str) -> Result<String> {
    let stored = paths::stored(&ctx.root, cwd, typed)?;
    if stored.is_empty() {
        return Err(bad("the project folder itself cannot be taken"));
    }
    if stored.trim() != stored || stored.contains(['\r', '\n']) {
        return Err(bad(format!(
            "a path cannot begin or end with a space or have a line break: {typed}"
        )));
    }
    Ok(stored)
}

/// Refuses a stored path that is not a regular file: Code::NotFound when it does not exist,
/// Code::BadValue for a folder. The path is named in the message.
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
        "{} is not a file; take takes files",
        paths::shown(stored)
    )))
}

/// The refusal for a file that another person holds (R-FILE-1): the person, the task and the time.
fn refused(stored: &str, take: &Take) -> Error {
    Error::new(
        Code::Refused,
        format!(
            "{} is taken by {} for {} since {}; ask them, or add --force",
            paths::shown(stored),
            take.person,
            take.task,
            take.time
        ),
    )
}

/// A warning for a path with a name that Windows cannot have (R-XOS-5), None when every name of
/// the path is allowed.
fn windows_warning(stored: &str) -> Option<String> {
    let unsafe_name = stored.split('/').any(|name| !paths::windows_safe(name));
    unsafe_name.then(|| format!("{} cannot be copied to a Windows PC", paths::shown(stored)))
}

/// Warnings for the given paths that differ only in upper and lower case (R-XOS-5): each path is
/// compared with the paths before it.
fn case_warnings(stored: &[String]) -> Vec<String> {
    stored
        .iter()
        .enumerate()
        .filter_map(|(index, path)| {
            stored[..index]
                .iter()
                .find(|other| other.to_lowercase() == path.to_lowercase())
                .map(|other| {
                    format!(
                        "{} and {} differ only in upper and lower case; both cannot be copied to a PC",
                        paths::shown(path),
                        paths::shown(other)
                    )
                })
        })
        .collect()
}

/// "file" for one, "files" for any other number.
fn noun(count: usize) -> &'static str {
    if count == 1 { "file" } else { "files" }
}

/// A field with its name and its value.
fn pair(name: &str, value: &str) -> (String, String) {
    (name.to_string(), value.to_string())
}

/// A failure for a value the person gave, Code::BadValue.
fn bad(message: impl Into<String>) -> Error {
    Error::new(Code::BadValue, message)
}
