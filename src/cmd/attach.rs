//! The `attach` command (docs/design.md section 9): adds files or folders to the file list of a
//! task, with their role, note, what they were made from, and their size or number of files,
//! time and checksum (R-FILE-8, R-FILE-9, R-FILE-10). With `--moved` it changes a path and keeps
//! the history of both paths (R-FILE-13). The list is files/<TASK>.md (format.md §17).

use crate::blockfile::BlockFile;
use crate::cli::Global;
use crate::ctx::Ctx;
use crate::error::{Code, Error, Result};
use crate::filelist;
use crate::names::Who;
use crate::output::Output;
use crate::paths;
use crate::tx::{self, Change};
use serde_json::json;
use std::path::Path;

/// The fields that are refreshed when a detached file is attached again (format.md §17). A value
/// that the new attach does not give is removed.
const REFRESHED: [&str; 9] = [
    "type",
    "size",
    "files",
    "modified",
    "checksum",
    "role",
    "note",
    "made-from",
    "moved-from",
];

/// Named field values of a block: the pairs of `BlockFile::add_block`.
type Fields = Vec<(String, String)>;

/// The arguments of `attach`.
#[derive(clap::Args, Debug)]
pub struct Args {
    /// The task, such as T-042
    pub task: String,
    /// The paths of the files or folders, relative to the current folder or absolute; with
    /// --moved, the old path and then the new one
    pub paths: Vec<String>,
    /// The role of the files, such as model or result (required unless --moved)
    #[arg(long, value_name = "role")]
    pub role: Option<String>,
    /// A note on the files
    #[arg(long, value_name = "text")]
    pub note: Option<String>,
    /// A file these files were made from, as a path (repeatable)
    #[arg(long = "from", value_name = "path")]
    pub from: Vec<String>,
    /// Changes a path: the paths are exactly <old> <new>; the role, note and made-from are kept
    #[arg(long)]
    pub moved: bool,
}

/// Runs `attach`: adds the paths to the task's file list in one transaction, or with --moved
/// changes one path. The first output line is `<TASK>: attached <n> files` or `<TASK>: moved
/// <old> to <new>`.
pub fn run(args: &Args, g: &Global) -> Result<Output> {
    let ctx = Ctx::load(g, "attach")?;
    let who = ctx.writer()?.clone();
    let task = task_id(&args.task)?;
    let cwd = std::env::current_dir()?;
    if args.moved {
        moved(&ctx, args, &who, &task, &cwd)
    } else {
        attached(&ctx, args, &who, &task, &cwd)
    }
}

/// Adds each path of `args` to the list, with the role, note and made-from that the command gives.
fn attached(ctx: &Ctx, args: &Args, who: &Who, task: &str, cwd: &Path) -> Result<Output> {
    if args.paths.is_empty() {
        return Err(usage(
            "attach needs the path of a file or folder: attach <task> <path>… --role <role>",
        ));
    }
    let role = role_of(ctx, args.role.as_deref())?;
    let stored = stored_paths(ctx, cwd, &args.paths)?;
    let from = stored_paths(ctx, cwd, &args.from)?;
    let note = args.note.as_deref().map(str::trim).unwrap_or_default();

    tx::write(ctx, |tx| {
        let dossier = tx.ctx().dossier.clone();
        let root = tx.ctx().root.clone();
        let time = tx.time().to_string();
        let (mut file, old) = filelist::load(&dossier, task)?;
        let made_from = made_from_entries(&root, &from)?;

        let mut warnings: Vec<String> =
            stored.iter().flat_map(|path| name_warnings(path)).collect();
        warnings.extend(case_warnings(&attached_keys(&file), &stored));
        for path in &stored {
            let described = filelist::describe(&root, path)?;
            let fields = block_fields(&role, note, described, &made_from.join(", "), None);
            put_block(&mut file, path, &fields, &time, who, "attached", "")?;
        }
        tx.put_raw(
            filelist::path(&dossier, task),
            old,
            file.text().into_bytes(),
            Change {
                id: filelist::item(task),
                event: "attached".to_string(),
                detail: stored.join(", "),
            },
        );

        let files: Vec<serde_json::Value> = stored
            .iter()
            .map(|path| file_json(path, &role, &made_from, None))
            .collect();
        let mut text = format!("{task}: attached {} {}", stored.len(), noun(stored.len()));
        for path in &stored {
            text.push_str(&format!("\n  {}: {role}", paths::shown(path)));
            if !from.is_empty() {
                text.push_str(&format!(", made from {}", shown_list(&from)));
            }
        }
        let data = json!({"task": task, "event": "attached", "files": files, "detached": []});
        Ok(with_warnings(Output::with(text, data), warnings))
    })
}

/// Changes the path `old` of the list to `new`: `old` is detached and keeps its history, and
/// `new` gets a block with the role, note and made-from of `old`, and `moved-from`.
fn moved(ctx: &Ctx, args: &Args, who: &Who, task: &str, cwd: &Path) -> Result<Output> {
    if args.role.is_some() || args.note.is_some() || !args.from.is_empty() {
        return Err(usage(
            "--moved keeps the role, note and made-from of the file: give only the old and the new path",
        ));
    }
    let [old, new] = args.paths.as_slice() else {
        return Err(usage(
            "--moved takes two paths, the old and the new one: attach <task> <old> <new> --moved",
        ));
    };
    let old = stored_path(ctx, cwd, old)?;
    let new = stored_path(ctx, cwd, new)?;
    if old == new {
        return Err(bad("the old and the new path are the same"));
    }

    tx::write(ctx, |tx| {
        let dossier = tx.ctx().dossier.clone();
        let root = tx.ctx().root.clone();
        let time = tx.time().to_string();
        let (mut file, bytes) = filelist::load(&dossier, task)?;
        if !attached_keys(&file).contains(&old) {
            return Err(Error::new(
                Code::NotFound,
                format!("{} is not attached to {task}", paths::shown(&old)),
            ));
        }
        let role = text_of(&file, &old, "role");
        let note = text_of(&file, &old, "note");
        let made_from = text_of(&file, &old, "made-from");
        let described = filelist::describe(&root, &new)?;
        let mut warnings = name_warnings(&new);
        let others: Vec<String> = attached_keys(&file)
            .into_iter()
            .filter(|key| key != &old)
            .collect();
        warnings.extend(case_warnings(&others, std::slice::from_ref(&new)));

        file.set_field(&old, "detached", &format!("{time} by {}", who.name));
        file.add_history(&old, &time, who, "moved", &format!("to {new}"));
        let fields = block_fields(&role, &note, described, &made_from, Some(&old));
        put_block(
            &mut file,
            &new,
            &fields,
            &time,
            who,
            "moved",
            &format!("from {old}"),
        )?;
        tx.put_raw(
            filelist::path(&dossier, task),
            bytes,
            file.text().into_bytes(),
            Change {
                id: filelist::item(task),
                event: "moved".to_string(),
                detail: format!("{old} to {new}"),
            },
        );

        let made = list(&made_from);
        let text = format!(
            "{task}: moved {} to {}",
            paths::shown(&old),
            paths::shown(&new)
        );
        let data = json!({
            "task": task,
            "event": "moved",
            "files": [file_json(&new, &role, &made, Some(&old))],
            "detached": [old],
        });
        Ok(with_warnings(Output::with(text, data), warnings))
    })
}

/// The block of `key` in the list gets `fields` and one history line `event` — when the key has
/// no block yet, the block is added. A detached block is attached again: its detached mark goes
/// and its fields are refreshed, so that a field the new attach does not give is removed. A block
/// that is attached already is refused (Code::Exists).
fn put_block(
    file: &mut BlockFile,
    key: &str,
    fields: &[(String, String)],
    time: &str,
    who: &Who,
    event: &str,
    detail: &str,
) -> Result<()> {
    if file.keys().iter().any(|known| known == key) {
        if !is_detached(file, key) {
            return Err(Error::new(
                Code::Exists,
                "already attached; use dossier reattach",
            ));
        }
        file.set_field(key, "detached", "");
        for name in REFRESHED {
            let value = fields
                .iter()
                .find(|(field, _)| field == name)
                .map_or("", |(_, value)| value.as_str());
            file.set_field(key, name, value);
        }
    } else {
        file.add_block(key, fields);
    }
    file.add_history(key, time, who, event, detail);
    Ok(())
}

/// The fields of a new block, in the order of format.md §17: role, type, note, the other
/// description fields, made-from, and moved-from when the file was moved. Empty values are left
/// out when the block is written.
fn block_fields(
    role: &str,
    note: &str,
    described: Vec<(String, String)>,
    made_from: &str,
    moved_from: Option<&str>,
) -> Vec<(String, String)> {
    let (kind, rest): (Fields, Fields) =
        described.into_iter().partition(|(name, _)| name == "type");
    let mut fields = vec![pair("role", role)];
    fields.extend(kind);
    fields.push(pair("note", note));
    fields.extend(rest);
    fields.push(pair("made-from", made_from));
    if let Some(from) = moved_from {
        fields.push(pair("moved-from", from));
    }
    fields
}

/// A field with its name and its value.
fn pair(name: &str, value: &str) -> (String, String) {
    (name.to_string(), value.to_string())
}

/// The made-from entries of the given files: `<stored path>@<checksum>` for each. A folder has no
/// checksum, so its entry ends with `@`.
fn made_from_entries(root: &Path, from: &[String]) -> Result<Vec<String>> {
    from.iter()
        .map(|path| {
            let described = filelist::describe(root, path)?;
            let checksum = value_of(&described, "checksum");
            Ok(format!("{path}@{checksum}"))
        })
        .collect()
}

/// The role as the list and the settings spell it. Code::Usage without a role, Code::BadValue
/// for a role that is neither one of the roles nor in the settings (R-FILE-9).
fn role_of(ctx: &Ctx, role: Option<&str>) -> Result<String> {
    let role = role.map(str::trim).unwrap_or_default();
    if role.is_empty() {
        return Err(usage(format!(
            "attach needs --role <role>, one of: {}",
            roles_text(ctx)
        )));
    }
    if !filelist::valid_role(&ctx.cfg, role) {
        return Err(bad(format!(
            "{role} is not a role; the roles are: {}",
            roles_text(ctx)
        )));
    }
    let spelled = filelist::ROLES
        .iter()
        .map(|known| known.to_string())
        .chain(ctx.cfg.roles.iter().cloned())
        .find(|known| known.to_lowercase() == role.to_lowercase());
    Ok(spelled.unwrap_or_else(|| role.to_string()))
}

/// The roles of the tool and of the settings, separated by commas.
fn roles_text(ctx: &Ctx) -> String {
    filelist::ROLES
        .iter()
        .map(|role| role.to_string())
        .chain(ctx.cfg.roles.iter().cloned())
        .collect::<Vec<String>>()
        .join(", ")
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

/// The stored paths of the typed paths, in order (see `stored_path`).
fn stored_paths(ctx: &Ctx, cwd: &Path, typed: &[String]) -> Result<Vec<String>> {
    typed
        .iter()
        .map(|path| stored_path(ctx, cwd, path))
        .collect()
}

/// A typed path as stored (R-XOS-2). The project folder itself cannot be attached, and a path
/// that begins or ends with a space or has a line break cannot be a block key (Code::BadValue).
fn stored_path(ctx: &Ctx, cwd: &Path, typed: &str) -> Result<String> {
    let stored = paths::stored(&ctx.root, cwd, typed)?;
    if stored.is_empty() {
        return Err(bad("the project folder itself cannot be attached"));
    }
    if stored.trim() != stored || stored.contains(['\r', '\n']) {
        return Err(bad(format!(
            "a path cannot begin or end with a space or have a line break: {typed}"
        )));
    }
    Ok(stored)
}

/// The keys of the blocks that are not detached: the files that belong to the task now.
fn attached_keys(file: &BlockFile) -> Vec<String> {
    file.keys()
        .into_iter()
        .filter(|key| !is_detached(file, key))
        .collect()
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

/// The entries of a comma-separated field, trimmed, empty ones left out.
fn list(text: &str) -> Vec<String> {
    text.split(',')
        .map(str::trim)
        .filter(|entry| !entry.is_empty())
        .map(str::to_string)
        .collect()
}

/// The JSON of one file of the list: its stored path, role, made-from entries and the path it
/// was moved from.
fn file_json(
    path: &str,
    role: &str,
    made_from: &[String],
    moved_from: Option<&str>,
) -> serde_json::Value {
    json!({"path": path, "role": role, "made_from": made_from, "moved_from": moved_from})
}

/// Warnings for the names of a stored path that Windows cannot have (R-XOS-5): one warning for
/// each such name of the path.
fn name_warnings(stored: &str) -> Vec<String> {
    stored
        .split('/')
        .filter(|name| !paths::windows_safe(name))
        .map(|name| {
            format!(
                "{} has the name {name}, which Windows cannot have; the file cannot be copied to a PC",
                paths::shown(stored)
            )
        })
        .collect()
}

/// Warnings for paths that differ only in upper and lower case (R-XOS-5): each new path is
/// compared with the paths already in the list and with the new paths before it.
fn case_warnings(known: &[String], new: &[String]) -> Vec<String> {
    let mut warnings = Vec::new();
    for (index, path) in new.iter().enumerate() {
        let earlier = known.iter().chain(&new[..index]);
        if let Some(other) = earlier
            .clone()
            .find(|other| *other != path && other.to_lowercase() == path.to_lowercase())
        {
            warnings.push(format!(
                "{} and {} differ only in upper and lower case; both cannot be copied to a PC",
                paths::shown(path),
                paths::shown(other)
            ));
        }
    }
    warnings
}

/// The output with each warning added, in order.
fn with_warnings(output: Output, warnings: Vec<String>) -> Output {
    warnings
        .into_iter()
        .fold(output, |output, warning| output.warn(warning))
}

/// The stored paths shown with the separators of this system, joined by commas.
fn shown_list(stored: &[String]) -> String {
    stored
        .iter()
        .map(|path| paths::shown(path))
        .collect::<Vec<String>>()
        .join(", ")
}

/// "file" for one, "files" for any other number.
fn noun(count: usize) -> &'static str {
    if count == 1 { "file" } else { "files" }
}

/// A usage failure, Code::Usage.
fn usage(message: impl Into<String>) -> Error {
    Error::new(Code::Usage, message)
}

/// A failure for a value the person gave, Code::BadValue.
fn bad(message: impl Into<String>) -> Error {
    Error::new(Code::BadValue, message)
}
