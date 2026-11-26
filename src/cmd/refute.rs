//! The `refute` command (docs/design.md section 9, R-WRITE-15): a person at a terminal, or an LLM
//! with `--said-by <person>` (D57), finds an approved assumption wrong. The assumption becomes
//! `wrong`, with `refuted` and `valid-until` set and the `--by` text as its Confirmation. Then
//! every entry and file that rests on it gets the field `rests-on` with its ID and one history
//! line, and everything marked is printed.
//!
//! What rests on the assumption is what its Affects section names (format.md §14): a task marks
//! the approved entries whose `task` is that task and the non-detached files of its list; an
//! entry marks itself; a path marks every block with that key and every block made from it, and a
//! value is only named, with the usage records that use it.

use crate::affects::{self, Item as Affect};
use crate::blockfile::BlockFile;
use crate::cli::Global;
use crate::ctx::Ctx;
use crate::entry::{Entry, Kind, Status};
use crate::error::{Code, Error, Result};
use crate::filelist;
use crate::names::{self, Who};
use crate::output::Output;
use crate::paths;
use crate::store;
use crate::term;
use crate::tx::{self, Change, Tx};
use crate::values;
use serde_json::json;
use std::path::Path;

/// The state of a refuted assumption (format.md §14).
const WRONG: &str = "wrong";
/// The history event and the journal event of a refuted assumption.
const REFUTED: &str = "refuted";
/// The history event and the journal event of an entry or a file block that gets a mark.
const CHANGED: &str = "changed";
/// The field that names the wrong assumptions an entry or a block rests on (format.md §14).
const RESTS_ON: &str = "rests-on";
/// The section that holds the `--by` text of an assumption (format.md §8).
const CONFIRMATION: &str = "Confirmation";

/// The arguments of `refute`.
#[derive(clap::Args, Debug)]
pub struct Args {
    /// The ID of the assumption, such as ASM-003; any case, with or without a place prefix
    pub id: String,
    /// Who found it wrong and how, such as "anna, from the rig measurements"
    #[arg(long, value_name = "who and how")]
    pub by: String,

    /// The person who decided this; needed when an LLM runs the command (D57)
    #[arg(long, value_name = "PERSON")]
    pub said_by: Option<String>,
}

/// Runs `refute`: checks who acts (an LLM runs it with `--said-by`, D57) and the assumption, then
/// writes the assumption and every mark in one transaction. The first output line is
/// `<ID>: refuted`; then one line per marked entry, one per marked file, one per value named and
/// one per text not recognised. The data has the same lists.
pub fn run(args: &Args, g: &Global) -> Result<Output> {
    let ctx = Ctx::load(g, "refute")?;
    let who = ctx.writer()?.clone();
    person_at_terminal(&who, args.said_by.as_deref())?;
    let by = args.by.trim();
    if by.is_empty() {
        return Err(bad(
            "--by says who found the assumption wrong and how; it must not be empty",
        ));
    }
    if by.lines().any(|line| line.starts_with("## ")) {
        return Err(bad(
            "a line of --by cannot start with ## : it would start a section",
        ));
    }
    tx::write(&ctx, |tx| refute(tx, &args.id, by, &who))
}

/// Queues the refutation of the assumption `wanted` and the marks of everything that rests on it.
fn refute(tx: &mut Tx<'_>, wanted: &str, by: &str, who: &Who) -> Result<Output> {
    let dossier = tx.ctx().dossier.clone();
    let time = tx.time().to_string();
    let item = store::get(&dossier, wanted)?;
    check_assumption(&item.entry)?;
    let id = item.entry.id();

    let (values_file, _) = values::load(&dossier, &tx.ctx().cfg.code)?;
    let value_names: Vec<String> = values::all(&values_file)
        .into_iter()
        .map(|value| value.name)
        .collect();
    let mut lists = load_lists(&dossier)?;
    let attached = attached_paths(&lists);
    let named = Named::of(affects::parse(
        &item.entry.section("Affects").unwrap_or_default(),
        &value_names,
        &attached,
    ));
    let (entries, _) = store::all(&dossier);

    let mut warnings = Vec::new();
    for wanted_id in &named.entries {
        if *wanted_id != id && !entries.iter().any(|other| other.entry.id() == *wanted_id) {
            warnings.push(format!("no entry {wanted_id}: nothing marked for it"));
        }
    }

    let mut assumption = item.entry.clone();
    assumption.set_field("state", WRONG);
    assumption.set_field("refuted", &format!("{time} by {}", who.person));
    assumption.set_field("valid-until", &time);
    assumption.set_section(CONFIRMATION, by);
    assumption.add_history(&time, who, REFUTED, "");
    tx.put(
        item.path.clone(),
        Some(&item),
        assumption.text(),
        Change {
            id: id.clone(),
            event: REFUTED.to_string(),
            detail: String::new(),
        },
    );

    let mut marked_entries = Vec::new();
    for other in &entries {
        let other_id = other.entry.id();
        if other_id == id || !rests_on_entry(&other.entry, &named) {
            continue;
        }
        marked_entries.push(other_id.clone());
        let mut wrong = other.entry.list(RESTS_ON);
        if wrong.contains(&id) {
            continue;
        }
        wrong.push(id.clone());
        let detail = rests_detail(&id);
        let mut edited = other.entry.clone();
        edited.set_field(RESTS_ON, &wrong.join(", "));
        edited.add_history(&time, who, CHANGED, &detail);
        tx.put(
            other.path.clone(),
            Some(other),
            edited.text(),
            Change {
                id: other_id,
                event: CHANGED.to_string(),
                detail,
            },
        );
    }

    let mut marked_files: Vec<(String, String)> = Vec::new();
    for list in &mut lists {
        let mut details = Vec::new();
        for key in list.file.keys() {
            if !rests_on_block(&list.file, &key, &list.task, &named) {
                continue;
            }
            marked_files.push((list.task.clone(), key.clone()));
            let mut wrong = comma_list(&list.file.field(&key, RESTS_ON).unwrap_or_default());
            if wrong.contains(&id) {
                continue;
            }
            wrong.push(id.clone());
            let detail = rests_detail(&id);
            list.file.set_field(&key, RESTS_ON, &wrong.join(", "));
            list.file.add_history(&key, &time, who, CHANGED, &detail);
            details.push(format!("{key}: {CHANGED} — {detail}"));
        }
        if !details.is_empty() {
            tx.put_raw(
                filelist::path(&dossier, &list.task),
                list.bytes.clone(),
                list.file.text().into_bytes(),
                Change {
                    id: filelist::item(&list.task),
                    event: CHANGED.to_string(),
                    detail: details.join("; "),
                },
            );
        }
    }

    let mut value_uses: Vec<(String, Vec<String>)> = Vec::new();
    for name in &named.values {
        let reference = format!("{{{name}}}");
        let used_by: Vec<String> = entries
            .iter()
            .filter(|other| {
                other.entry.kind() == Some(Kind::Usage)
                    && other
                        .entry
                        .section("Parameters")
                        .is_some_and(|body| body.contains(&reference))
            })
            .map(|other| other.entry.id())
            .collect();
        value_uses.push((name.clone(), used_by));
    }

    let detail = rests_detail(&id);
    let mut lines = vec![format!("{id}: refuted")];
    lines.extend(
        marked_entries
            .iter()
            .map(|other| format!("{other}: {detail}")),
    );
    lines.extend(
        marked_files
            .iter()
            .map(|(task, key)| format!("{task} file {}: {detail}", paths::shown(key))),
    );
    lines.extend(value_uses.iter().map(|(name, used_by)| {
        if used_by.is_empty() {
            format!("values named: {name}")
        } else {
            format!("values named: {name} (used by {})", used_by.join(", "))
        }
    }));
    lines.extend(
        named
            .texts
            .iter()
            .map(|text| format!("not recognised: {text}")),
    );

    let data = json!({
        "id": id,
        "state": WRONG,
        "entries": marked_entries,
        "files": marked_files
            .iter()
            .map(|(task, path)| json!({"task": task, "path": path}))
            .collect::<Vec<_>>(),
        "values": value_uses
            .iter()
            .map(|(name, used_by)| json!({"name": name, "used_by": used_by}))
            .collect::<Vec<_>>(),
        "not_recognised": named.texts,
    });
    let output = Output::with(lines.join("\n"), data);
    Ok(warnings
        .into_iter()
        .fold(output, |output, warning| output.warn(warning)))
}

/// The items of an Affects section by kind, each listed once in the order they are written.
#[derive(Default)]
struct Named {
    tasks: Vec<String>,
    entries: Vec<String>,
    paths: Vec<String>,
    values: Vec<String>,
    texts: Vec<String>,
}

impl Named {
    /// Sorts the items of an Affects section by kind, leaving out repeats.
    fn of(items: Vec<Affect>) -> Named {
        let mut named = Named::default();
        for item in items {
            match item {
                Affect::Task(task) => push_unique(&mut named.tasks, task),
                Affect::Entry(id) => push_unique(&mut named.entries, id),
                Affect::Path(path) => push_unique(&mut named.paths, path),
                Affect::Value(name) => push_unique(&mut named.values, name),
                Affect::Text(text) => push_unique(&mut named.texts, text),
            }
        }
        named
    }
}

/// A file list of a task as read: its task, its bytes as read (None for a missing list) and the
/// parsed block file.
struct List {
    task: String,
    file: BlockFile,
    bytes: Option<Vec<u8>>,
}

/// Every file list of the dossier, loaded once each.
fn load_lists(dossier: &Path) -> Result<Vec<List>> {
    filelist::tasks(dossier)
        .into_iter()
        .map(|task| {
            let (file, bytes) = filelist::load(dossier, &task)?;
            Ok(List { task, file, bytes })
        })
        .collect()
}

/// The stored paths of every non-detached block of every list, sorted and without repeats: the
/// attached paths that Affects can name.
fn attached_paths(lists: &[List]) -> Vec<String> {
    let mut stored: Vec<String> = lists
        .iter()
        .flat_map(|list| {
            list.file
                .keys()
                .into_iter()
                .filter(|key| !is_detached(&list.file, key))
        })
        .collect();
    stored.sort();
    stored.dedup();
    stored
}

/// Whether an entry rests on the assumption: an approved entry whose `task` is one of the tasks
/// named, or an entry named by its ID.
fn rests_on_entry(entry: &Entry, named: &Named) -> bool {
    let by_task = entry.status() == Some(Status::Approved)
        && entry.field("task").is_some_and(|task| {
            named
                .tasks
                .iter()
                .any(|known| known.eq_ignore_ascii_case(task.trim()))
        });
    by_task || named.entries.contains(&entry.id())
}

/// Whether a block of a file list rests on the assumption: a non-detached block of the list of a
/// task named, a block whose key is a path named, or a block made from a path named. A block made
/// from a path named counts even when it is detached, as a block with that key does.
fn rests_on_block(file: &BlockFile, key: &str, task: &str, named: &Named) -> bool {
    let by_task = !is_detached(file, key)
        && named
            .tasks
            .iter()
            .any(|known| known.eq_ignore_ascii_case(task));
    let by_path = named.paths.iter().any(|path| path == key)
        || made_from(file, key)
            .iter()
            .any(|stored| named.paths.contains(stored));
    by_task || by_path
}

/// The stored paths a block was made from: the part before `@` of each `made-from` entry.
fn made_from(file: &BlockFile, key: &str) -> Vec<String> {
    comma_list(&file.field(key, "made-from").unwrap_or_default())
        .into_iter()
        .map(|entry| match entry.rsplit_once('@') {
            Some((stored, _)) => stored.trim().to_string(),
            None => entry,
        })
        .collect()
}

/// Whether a block is detached (format.md §17): it has a `detached` field with a value.
fn is_detached(file: &BlockFile, key: &str) -> bool {
    file.field(key, "detached").is_some_and(|at| !at.is_empty())
}

/// Refuses an entry that is not an assumption, one that is not approved, and an assumption that
/// is not open or confirmed. Code::Refused for each; a refuted assumption is refused again.
fn check_assumption(entry: &Entry) -> Result<()> {
    let id = entry.id();
    if entry.kind() != Some(Kind::Assumption) {
        let kind = entry.kind().map_or("unknown", Kind::name);
        return Err(refused(format!(
            "refute is for an assumption; {id} is a {kind}"
        )));
    }
    match entry.status() {
        Some(Status::Approved) => {}
        Some(status) => {
            return Err(refused(format!(
                "only an approved assumption can be refuted; {id} is {}",
                status.name()
            )));
        }
        None => {
            return Err(refused(format!(
                "only an approved assumption can be refuted; {id} has no valid status"
            )));
        }
    }
    match entry.field("state").as_deref() {
        Some("open" | "confirmed") => Ok(()),
        Some(WRONG) => Err(refused(format!("{id} is already {WRONG}"))),
        _ => Err(refused(format!(
            "{id} has no valid state; it must be open, confirmed or wrong"
        ))),
    }
}

/// Checks who acts with names::said_by_gate (D57), then refuses a run with no terminal
/// (Code::NeedsTerminal) unless an LLM carries out its person's decision with `--said-by`.
fn person_at_terminal(who: &Who, said_by: Option<&str>) -> Result<()> {
    let said = names::said_by_gate(who, said_by, "refute")?;
    if !said && !term::is_terminal() {
        return Err(Error::new(
            Code::NeedsTerminal,
            "this command must be run by a person at a terminal",
        ));
    }
    Ok(())
}

/// The detail of a mark: `rests on <ID>`.
fn rests_detail(id: &str) -> String {
    format!("rests on {id}")
}

/// The items of a comma-separated field, trimmed, without empty items.
fn comma_list(text: &str) -> Vec<String> {
    text.split(',')
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .map(str::to_string)
        .collect()
}

/// Adds an item to a list unless it is already there.
fn push_unique(list: &mut Vec<String>, item: String) {
    if !list.contains(&item) {
        list.push(item);
    }
}

/// A failure for a state the assumption is not in, Code::Refused.
fn refused(message: impl Into<String>) -> Error {
    Error::new(Code::Refused, message)
}

/// A failure for a value the person gave, Code::BadValue.
fn bad(message: impl Into<String>) -> Error {
    Error::new(Code::BadValue, message)
}
