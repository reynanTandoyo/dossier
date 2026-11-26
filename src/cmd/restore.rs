//! The `restore` command: a person at a terminal, or an LLM with `--said-by <person>` (D57), brings
//! a snapshot of an entry back (R-HIST-2). The snapshot's text becomes the entry, except for its
//! history: every history line of the current entry that the snapshot lacks is put back, so that
//! the history keeps every earlier line, and a `restored` line is added last. The text before the
//! restore is saved as a snapshot too, so a restore can itself be undone. A person confirms with
//! `yes`; with `--said-by` nothing is asked. Anything else changes nothing.

use crate::approval;
use crate::cli::Global;
use crate::ctx::Ctx;
use crate::entry::{Entry, Kind, Status};
use crate::error::{Code, Error, Result};
use crate::names::{self, Who};
use crate::output::Output;
use crate::snapshot;
use crate::store;
use crate::term;
use crate::tx::{self, Change};
use serde_json::json;

/// The arguments of `restore`.
#[derive(clap::Args, Debug)]
pub struct Args {
    /// The ID of the entry, such as DEC-012; any case, with or without a place prefix
    pub id: String,
    /// The number of the snapshot to bring back, as `history <ID>` lists it
    pub n: u32,

    /// The person who decided this; needed when an LLM runs the command (D57)
    #[arg(long, value_name = "PERSON")]
    pub said_by: Option<String>,
}

/// Runs `restore`: checks who acts (names::said_by_gate, D57), that a person is at a terminal
/// unless an LLM carries out its person's decision with `--said-by`, that the entry and the
/// snapshot exist, and, for a person, asks for confirmation. Then the snapshot is written in one
/// transaction. The output is `<ID>: restored to snapshot <n>`. Any answer other than `yes` gives
/// `nothing changed`.
pub fn run(args: &Args, g: &Global) -> Result<Output> {
    let ctx = Ctx::load(g, "restore")?;
    let who = ctx.writer()?.clone();
    let said = names::said_by_gate(&who, args.said_by.as_deref(), "restore")?;
    if !said && !term::is_terminal() {
        return Err(Error::new(
            Code::NeedsTerminal,
            "this command must be run by a person at a terminal",
        ));
    }

    // The ID as the entry has it, and the snapshot, both checked before the question so that an
    // unknown entry or snapshot is refused without asking anything.
    let id = store::get(&ctx.dossier, &args.id)?.entry.id();
    snapshot::load(&ctx.state, &id, args.n)?;
    if !said {
        let answer = term::ask(&format!("restore {id} to snapshot {}? (yes/no) ", args.n))?;
        if !answer.eq_ignore_ascii_case("yes") {
            return Ok(Output::with(
                "nothing changed",
                json!({"id": id, "snapshot": args.n, "restored": false}),
            ));
        }
    }

    // The status and the text of a restored lesson, how-to or tool. Its checksum is written after
    // the file, so that a refused restore leaves the checksum alone (as retitle does).
    let mut guarded: Option<(bool, String)> = None;
    let output = tx::write(&ctx, |tx| {
        let item = store::get(&tx.ctx().dossier, &id)?;
        let bytes = snapshot::load(&tx.ctx().state, &id, args.n)?;
        let snap_text = std::str::from_utf8(&bytes).map_err(|_| {
            Error::new(
                Code::Parse,
                format!("snapshot {} of {id} is not UTF-8", args.n),
            )
        })?;
        let entry = restored_entry(&item.entry, snap_text, args.n, &who, tx.time())?;
        let text = entry.text();

        // The file is named by its kind, its ID and the slug of its title (format.md §4), so a
        // title that the snapshot had again moves the file back to its name.
        let kind = entry.kind().or_else(|| Kind::of_id(&id)).ok_or_else(|| {
            Error::new(
                Code::Parse,
                format!("{id} has no kind, so its file cannot be named"),
            )
        })?;
        let new_path = store::path_for(&tx.ctx().dossier, kind, &id, &entry.title());
        let detail = format!("snapshot {}", args.n);
        let change = Change {
            id: id.clone(),
            event: "restored".to_string(),
            detail,
        };
        if approval::guarded(kind) {
            guarded = Some((entry.status() == Some(Status::Approved), text.clone()));
        }
        if new_path == item.path {
            tx.put(item.path.clone(), Some(&item), text, change);
        } else {
            tx.rename(&item, new_path, text, change);
        }

        let status = entry.status().map_or("", Status::name);
        Ok(Output::with(
            format!("{id}: restored to snapshot {}", args.n),
            json!({"id": id, "snapshot": args.n, "restored": true, "status": status}),
        ))
    })?;

    // A restored lesson, how-to or tool is approved again when its restored text was approved, and
    // otherwise has no approved checksum. A dry run writes nothing, so it records nothing.
    if let Some((approved, text)) = guarded.filter(|_| !ctx.dry_run) {
        if approved {
            approval::record(&ctx.state, &id, text.as_bytes())?;
        } else {
            approval::forget(&ctx.state, &id)?;
        }
    }
    Ok(output)
}

/// The entry as the snapshot has it, with the history of `current` kept. The history lines of
/// `current` that the snapshot does not have are put back in their order, then the `restored`
/// line is added after them (format.md §6). Only whole lines are added; every other line of the
/// snapshot keeps its bytes. Code::Refused when the snapshot belongs to another entry.
fn restored_entry(
    current: &Entry,
    snap_text: &str,
    n: u32,
    who: &Who,
    time: &str,
) -> Result<Entry> {
    let mut entry = Entry::parse(snap_text)?;
    if entry.id() != current.id() || entry.kind() != current.kind() {
        return Err(Error::new(
            Code::Refused,
            format!("snapshot {n} is not a snapshot of {}", current.id()),
        ));
    }
    let missing = missing_history(current, &entry);
    entry.add_history(time, who, "restored", &format!("snapshot {n}"));

    // The restored line is the last history line now; the missing lines go in front of it.
    let at = history_lines(&entry.lines)
        .last()
        .copied()
        .ok_or_else(|| Error::new(Code::Parse, "the restored entry has no history line"))?;
    let ending = if entry.lines[at].ends_with("\r\n") {
        "\r\n"
    } else {
        "\n"
    };
    for (offset, line) in missing.iter().enumerate() {
        let body = line.trim_end_matches(['\r', '\n']);
        entry.lines.insert(at + offset, format!("{body}{ending}"));
    }
    Ok(entry)
}

/// The lines of the history of `current` that `snapshot` does not have, in file order. A history
/// line is never changed, so each value of the snapshot's history covers one line of `current`
/// with the same value, and the rest are missing.
fn missing_history(current: &Entry, snapshot: &Entry) -> Vec<String> {
    let mut have = snapshot.history();
    let values = current.history();
    let mut missing = Vec::new();
    for (value, index) in values.iter().zip(history_lines(&current.lines)) {
        match have.iter().position(|seen| seen == value) {
            Some(at) => {
                have.remove(at);
            }
            None => missing.push(current.lines[index].clone()),
        }
    }
    missing
}

/// The indices of the history lines among `lines`, in order. `Entry::history` decides what a
/// history line is: a line is one when the number of history values grows once it is included.
fn history_lines(lines: &[String]) -> Vec<usize> {
    let mut found = Vec::new();
    let mut count = 0;
    for index in 0..lines.len() {
        let seen = Entry {
            lines: lines[..=index].to_vec(),
        }
        .history()
        .len();
        if seen > count {
            found.push(index);
            count = seen;
        }
    }
    found
}
