//! The `cleared` command (docs/design.md section 9, R-WRITE-15): a person clears one rests-on mark.
//! The target is an entry ID, such as DEC-012, or the path of a file or folder. An entry loses the
//! assumption from its `rests-on` field; a path loses it from the block of that path in every file
//! list whose block has it. Each change gets one history line, `cleared`, with the assumption and
//! the `--note` text, so that the history keeps why the mark was removed (format.md §6, §14, §17).
//!
//! A person clears a mark; an LLM does so with `--said-by <person>` (D57). No terminal is needed.
//! A refused change writes nothing.

use crate::cli::Global;
use crate::ctx::Ctx;
use crate::entry::Kind;
use crate::error::{Code, Error, Result};
use crate::filelist;
use crate::names;
use crate::names::Who;
use crate::output::Output;
use crate::paths;
use crate::store;
use crate::tx::{self, Change, Tx};
use serde_json::json;
use std::path::Path;

/// The field that names the wrong assumptions an entry or a block rests on (format.md §14).
const RESTS_ON: &str = "rests-on";
/// The history event and the journal event of a cleared mark (format.md §14 and §17).
const CLEARED: &str = "cleared";

/// The arguments of `cleared`.
#[derive(clap::Args, Debug)]
pub struct Args {
    /// The ID of an entry, such as DEC-012, or the path of a file or folder, relative to the
    /// current folder or absolute
    pub target: String,
    /// The assumption whose mark is removed, such as ASM-003; any case, with or without a place
    /// prefix
    #[arg(long, value_name = "ID")]
    pub assumption: String,
    /// Why the mark is removed, in one line, such as "the rig measured a flat inlet profile"
    #[arg(long)]
    pub note: String,
    /// The person who decided this; needed when an LLM runs the command (D57)
    #[arg(long, value_name = "PERSON")]
    pub said_by: Option<String>,
}

/// What `cleared` removed: the entries and the files (task and stored path) it cleared.
struct Cleared {
    entries: Vec<String>,
    files: Vec<(String, String)>,
}

/// Runs `cleared`: checks who acts (names::said_by_gate, D57), the assumption and the note, then,
/// in one transaction, removes the mark from the entry or from the blocks of the path. An LLM runs
/// it with `--said-by <person>` for its person; it needs no terminal and asks nothing. The first
/// output line is `<ID>: cleared <ASM>` for an entry, or `<TASK> file <path>: cleared <ASM>` for a
/// file; the data has the same lists.
pub fn run(args: &Args, g: &Global) -> Result<Output> {
    let ctx = Ctx::load(g, "cleared")?;
    let who = ctx.writer()?.clone();
    names::said_by_gate(&who, args.said_by.as_deref(), "cleared")?;
    let assumption = assumption_id(&args.assumption)?;
    let note = note_text(&args.note)?;
    let target = args.target.trim();
    let is_entry = Kind::of_id(target).is_some();
    let stored = if is_entry {
        String::new()
    } else {
        stored_path(&ctx, &std::env::current_dir()?, &args.target)?
    };

    tx::write(&ctx, |tx| {
        let cleared = if is_entry {
            clear_entry(tx, target, &assumption, &note, &who)?
        } else {
            clear_path(tx, &stored, &assumption, &note, &who)?
        };
        Ok(output(&cleared, &assumption))
    })
}

/// Removes `assumption` from the `rests-on` field of the entry `wanted`, with one history line.
/// Code::NotFound when the entry does not name the assumption.
fn clear_entry(
    tx: &mut Tx<'_>,
    wanted: &str,
    assumption: &str,
    note: &str,
    who: &Who,
) -> Result<Cleared> {
    let item = store::get(&tx.ctx().dossier, wanted)?;
    let id = item.entry.id();
    let mut wrong = item.entry.list(RESTS_ON);
    let Some(at) = wrong
        .iter()
        .position(|known| normal_id(known) == assumption)
    else {
        return Err(not_found(format!("{id} has no mark for {assumption}")));
    };
    wrong.remove(at);

    let time = tx.time().to_string();
    let detail = format!("{assumption}: {note}");
    let mut entry = item.entry.clone();
    entry.set_field(RESTS_ON, &wrong.join(", "));
    entry.add_history(&time, who, CLEARED, &detail);
    tx.put(
        item.path.clone(),
        Some(&item),
        entry.text(),
        Change {
            id: id.clone(),
            event: CLEARED.to_string(),
            detail,
        },
    );
    Ok(Cleared {
        entries: vec![id],
        files: Vec::new(),
    })
}

/// Removes `assumption` from the `rests-on` field of the block `stored` in every file list that
/// has it, with one history line per block. Code::NotFound when no block of the path has it.
fn clear_path(
    tx: &mut Tx<'_>,
    stored: &str,
    assumption: &str,
    note: &str,
    who: &Who,
) -> Result<Cleared> {
    let dossier = tx.ctx().dossier.clone();
    let time = tx.time().to_string();
    let detail = format!("{assumption}: {note}");
    let mut files = Vec::new();
    for task in filelist::tasks(&dossier) {
        let (mut file, bytes) = filelist::load(&dossier, &task)?;
        let mut wrong = comma_list(&file.field(stored, RESTS_ON).unwrap_or_default());
        let Some(at) = wrong
            .iter()
            .position(|known| normal_id(known) == assumption)
        else {
            continue;
        };
        wrong.remove(at);
        file.set_field(stored, RESTS_ON, &wrong.join(", "));
        file.add_history(stored, &time, who, CLEARED, &detail);
        tx.put_raw(
            filelist::path(&dossier, &task),
            bytes,
            file.text().into_bytes(),
            Change {
                id: filelist::item(&task),
                event: CLEARED.to_string(),
                detail: format!("{stored}: {CLEARED} — {detail}"),
            },
        );
        files.push((task, stored.to_string()));
    }
    if files.is_empty() {
        return Err(not_found(format!(
            "{} has no mark for {assumption}",
            paths::shown(stored)
        )));
    }
    Ok(Cleared {
        entries: Vec::new(),
        files,
    })
}

/// The output of a cleared mark: one line per entry and per file, and the same lists as data.
fn output(cleared: &Cleared, assumption: &str) -> Output {
    let mut lines: Vec<String> = cleared
        .entries
        .iter()
        .map(|id| format!("{id}: {CLEARED} {assumption}"))
        .collect();
    lines.extend(
        cleared.files.iter().map(|(task, path)| {
            format!("{task} file {}: {CLEARED} {assumption}", paths::shown(path))
        }),
    );
    let data = json!({
        "assumption": assumption,
        "entries": cleared.entries,
        "files": cleared
            .files
            .iter()
            .map(|(task, path)| json!({"task": task, "path": path}))
            .collect::<Vec<_>>(),
    });
    Output::with(lines.join("\n"), data)
}

/// The ID of the assumption as the lists write it: upper-cased, without a place prefix. It must
/// be an assumption's ID (Code::BadValue otherwise).
fn assumption_id(typed: &str) -> Result<String> {
    let id = normal_id(typed);
    if Kind::of_id(&id) != Some(Kind::Assumption) {
        return Err(bad(format!(
            "--assumption names an assumption, such as ASM-003: {}",
            typed.trim()
        )));
    }
    Ok(id)
}

/// An ID with the place prefix and the spaces left out, upper-cased (`P-001:asm-003` is ASM-003).
fn normal_id(typed: &str) -> String {
    let typed = typed.trim();
    typed
        .rsplit_once(':')
        .map_or(typed, |(_, rest)| rest)
        .trim()
        .to_uppercase()
}

/// The note: one line that is not empty (Code::BadValue otherwise).
fn note_text(typed: &str) -> Result<String> {
    let note = typed.trim();
    if note.is_empty() {
        return Err(bad(
            "--note says why the mark is cleared; it must not be empty",
        ));
    }
    if note.contains(['\r', '\n']) {
        return Err(bad("the --note text is one line"));
    }
    Ok(note.to_string())
}

/// A typed path as stored (R-XOS-2): relative to the project folder, with `/`. The path need not
/// exist any more. The project folder itself is not a file of a task, and a path that begins or
/// ends with a space or has a line break cannot be a block key (Code::BadValue).
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

/// The items of a comma-separated field, trimmed, without empty items.
fn comma_list(text: &str) -> Vec<String> {
    text.split(',')
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .map(str::to_string)
        .collect()
}

/// A failure for a mark that is not there, Code::NotFound.
fn not_found(message: impl Into<String>) -> Error {
    Error::new(Code::NotFound, message)
}

/// A failure for a value the person gave, Code::BadValue.
fn bad(message: impl Into<String>) -> Error {
    Error::new(Code::BadValue, message)
}
