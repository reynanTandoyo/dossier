//! The `which` command (docs/design.md section 9, R-FILE-15, R-CODE-2): what a file belongs to.
//! It prints each task whose file list has the path, with the role and marks of that block and
//! what the block was made from; each block of any task that was made from the path; and each
//! entry whose `code` field points at the path or at a folder that holds it. Nothing is written,
//! except the list of entries shown in a session (`--binding --session`, R-CODE-8): with
//! `--binding` only the approved entries in force that bind the path are printed (R-CODE-6).

use crate::approval::{self, Mark};
use crate::blockfile::BlockFile;
use crate::cli::Global;
use crate::codeptr;
use crate::ctx::Ctx;
use crate::entry::{Entry, Kind, Status};
use crate::error::{Code, Error, Result};
use crate::filelist;
use crate::filter::in_force_on;
use crate::links;
use crate::output::Output;
use crate::paths;
use crate::shown;
use crate::store::{self, Item};
use crate::time;
use serde_json::json;
use std::path::Path;

/// The arguments of `which`.
#[derive(clap::Args, Debug)]
pub struct Args {
    /// The path of the file or folder, relative to the current folder or absolute
    pub path: String,
    /// Prints only the approved entries in force whose code pointers cover the path
    #[arg(long)]
    pub binding: bool,
    /// With --binding: leaves out the entries this session was shown before
    #[arg(long, value_name = "ID", requires = "binding")]
    pub session: Option<String>,
}

/// A block of a task's list that is the path.
struct Belonging {
    task: String,
    /// The role of the block; None when it has none
    role: Option<String>,
    /// The marks of the block, as `files` gives them
    marks: Vec<String>,
    /// The stored paths the block was made from
    made_from: Vec<String>,
}

/// A block of a task's list that was made from the path.
struct MadeInto {
    task: String,
    /// The stored path of that block
    path: String,
}

/// An entry whose `code` field points at the path.
struct Pointing {
    id: String,
    title: String,
    /// The marks of the entry, without brackets
    marks: Vec<String>,
}

/// Runs `which`. A path that no task's list has is "not attached to any task", which is not an
/// error. A list that cannot be read is a warning.
pub fn run(args: &Args, g: &Global) -> Result<Output> {
    let ctx = Ctx::load(g, "which")?;
    let cwd = std::env::current_dir()?;
    let stored = stored_path(&ctx, &cwd, &args.path)?;
    if args.binding {
        return binding(&ctx, &stored, args.session.as_deref());
    }

    let mut warnings = Vec::new();
    let mut belongings = Vec::new();
    let mut made_into = Vec::new();
    for task in filelist::tasks(&ctx.dossier) {
        let (file, _) = match filelist::load(&ctx.dossier, &task) {
            Ok(loaded) => loaded,
            Err(error) => {
                warnings.push(format!(
                    "{}: {}",
                    shown_path(&ctx.root, &filelist::path(&ctx.dossier, &task)),
                    error.message
                ));
                continue;
            }
        };
        for key in keys(&file) {
            let sources = made_from(&file, &key);
            if sources.contains(&stored) {
                made_into.push(MadeInto {
                    task: task.clone(),
                    path: key.clone(),
                });
            }
            if key == stored {
                belongings.push(Belonging {
                    task: task.clone(),
                    role: non_empty(file.field(&key, "role")),
                    marks: filelist::marks(&ctx.root, &file, &key),
                    made_from: sources,
                });
            }
        }
    }

    let (items, store_warnings) = store::all(&ctx.dossier);
    warnings.extend(store_warnings);
    let pointing: Vec<Pointing> = items
        .iter()
        .filter(|item| points_at(&item.entry, &stored))
        .map(|item| Pointing {
            id: item.entry.id(),
            title: item.entry.title(),
            marks: entry_marks(&ctx, item),
        })
        .collect();

    let text = render(&belongings, &made_into, &pointing);
    let tasks: Vec<serde_json::Value> = belongings
        .iter()
        .map(|belonging| {
            json!({
                "task": belonging.task,
                "role": belonging.role,
                "marks": belonging.marks,
                "made_from": belonging.made_from,
            })
        })
        .collect();
    let made: Vec<serde_json::Value> = made_into
        .iter()
        .map(|item| json!({"task": item.task, "path": item.path}))
        .collect();
    let entries: Vec<serde_json::Value> = pointing
        .iter()
        .map(|item| json!({"id": item.id, "title": item.title, "marks": item.marks}))
        .collect();
    let data = json!({
        "path": stored,
        "tasks": tasks,
        "made_into": made,
        "entries": entries,
    });
    Ok(warnings
        .into_iter()
        .fold(Output::with(text, data), |output, warning| {
            output.warn(warning)
        }))
}

/// The `--binding` form (R-CODE-6, R-CODE-8): the approved entries in force whose `code` covers
/// `stored`, in the order of the store, each as `<ID> · <kind> · <title>` and, when it has one, the
/// first line of its statement (`statement_line`) indented by two spaces. Under each entry come its
/// linked entries (`links::linked`, `links::typed_line`; JSON gives each its `type`), except those
/// this output lists as entries of their own. With `session`, the entries that session has not
/// been shown yet are kept (`shown::unseen`), and the linked entries of an entry are shown with
/// it, never recorded.
/// Nothing else is printed. An entry counts only when it is in force today (`filter::in_force_on`).
fn binding(ctx: &Ctx, stored: &str, session: Option<&str>) -> Result<Output> {
    let (items, warnings) = store::all(&ctx.dossier);
    let now = time::now();
    let today = time::date(&now);
    let mut binding: Vec<&Item> = items
        .iter()
        .filter(|item| item.entry.status() == Some(Status::Approved))
        .filter(|item| in_force_on(&item.entry, today))
        .filter(|item| points_at(&item.entry, stored))
        .collect();
    if let Some(session) = session {
        let ids: Vec<String> = binding.iter().map(|item| item.entry.id()).collect();
        let fresh = shown::unseen(&ctx.state, session, &ids)?;
        binding.retain(|item| fresh.contains(&item.entry.id()));
    }

    let listed: Vec<String> = binding.iter().map(|item| item.entry.id()).collect();
    let mut lines: Vec<String> = Vec::new();
    let mut entries: Vec<serde_json::Value> = Vec::new();
    for item in &binding {
        let entry = &item.entry;
        let kind = entry.kind().map_or("", Kind::name);
        let title = entry.title();
        let text = statement_line(entry);
        let linked: Vec<&Item> = links::linked(entry, &items, links::LINKED_LIMIT)
            .into_iter()
            .filter(|other| !listed.contains(&other.entry.id()))
            .collect();
        lines.push(format!("{} · {kind} · {title}", entry.id()));
        if let Some(text) = &text {
            lines.push(format!("  {text}"));
        }
        for other in &linked {
            lines.push(links::typed_line(entry, other));
        }
        let linked_json: Vec<serde_json::Value> = linked
            .iter()
            .map(|other| {
                json!({
                    "id": other.entry.id(),
                    "title": other.entry.title(),
                    "type": links::field_of(entry, other),
                })
            })
            .collect();
        entries.push(json!({
            "id": entry.id(),
            "kind": kind,
            "title": title,
            "text": text,
            "linked": linked_json,
        }));
    }
    let data = json!({"path": stored, "entries": entries});
    Ok(warnings
        .into_iter()
        .fold(Output::with(lines.join("\n"), data), |output, warning| {
            output.warn(warning)
        }))
}

/// The first line of the Decision, Statement or Observation section of an entry: of the first of
/// these names the entry has, in that order. None when it has none of them, or the body is empty.
fn statement_line(entry: &Entry) -> Option<String> {
    ["Decision", "Statement", "Observation"]
        .into_iter()
        .find_map(|name| entry.section(name))
        .and_then(|body| body.lines().next().map(str::to_string))
}

/// The text: the task lines, each with its made-from line under it; then the made-into lines;
/// then the entry lines. With no task line, "not attached to any task" comes first.
fn render(belongings: &[Belonging], made_into: &[MadeInto], pointing: &[Pointing]) -> String {
    let mut lines: Vec<String> = Vec::new();
    if belongings.is_empty() {
        lines.push("not attached to any task".to_string());
    }
    for belonging in belongings {
        let mut parts = vec![
            belonging.task.clone(),
            belonging
                .role
                .clone()
                .unwrap_or_else(|| "(no role)".to_string()),
        ];
        if !belonging.marks.is_empty() {
            parts.push(format!("[{}]", belonging.marks.join(", ")));
        }
        lines.push(parts.join("  "));
        if !belonging.made_from.is_empty() {
            lines.push(format!("  made from: {}", belonging.made_from.join(", ")));
        }
    }
    for item in made_into {
        lines.push(format!("made into: {} {}", item.task, item.path));
    }
    for item in pointing {
        let mut line = format!("entries: {} {}", item.id, item.title);
        if !item.marks.is_empty() {
            line.push_str(&format!("  [{}]", item.marks.join(", ")));
        }
        lines.push(line);
    }
    lines.join("\n")
}

/// The keys of a list in file order, each once: the first block of a repeated key counts.
fn keys(file: &BlockFile) -> Vec<String> {
    let mut keys: Vec<String> = Vec::new();
    for key in file.keys() {
        if !keys.contains(&key) {
            keys.push(key);
        }
    }
    keys
}

/// The stored paths a block was made from: the made-from field without the checksums (format.md
/// §17). Each entry is `<stored path>@<checksum>`, or `<stored path>@` for a folder.
fn made_from(file: &BlockFile, key: &str) -> Vec<String> {
    file.field(key, "made-from")
        .unwrap_or_default()
        .split(',')
        .map(str::trim)
        .filter(|entry| !entry.is_empty())
        .map(|entry| {
            entry
                .rsplit_once('@')
                .map_or(entry, |(path, _)| path)
                .to_string()
        })
        .collect()
}

/// Whether an entry's `code` field has a pointer that covers `stored` (R-CODE-2, R-CODE-6): the
/// pointer's path is the stored path, or a folder that holds it (`codeptr::covers`).
fn points_at(entry: &Entry, stored: &str) -> bool {
    entry
        .list("code")
        .iter()
        .any(|pointer| codeptr::covers(pointer, stored))
}

/// The marks of an entry, without brackets: its status, then `changed after approval` for an
/// approved lesson whose text differs from its recorded checksum (as `list` and `show` give them).
fn entry_marks(ctx: &Ctx, item: &Item) -> Vec<String> {
    let entry = &item.entry;
    let mut marks: Vec<String> = Vec::new();
    match entry.status() {
        Some(Status::Draft) => marks.push("draft".to_string()),
        Some(Status::Proposed) => marks.push("proposed".to_string()),
        Some(Status::Rejected) => marks.push("rejected".to_string()),
        Some(Status::Declined) => marks.push("declined".to_string()),
        Some(Status::Superseded) => marks.push(
            match entry.field("superseded-by").filter(|by| !by.is_empty()) {
                Some(by) => format!("superseded by {by}"),
                None => "superseded".to_string(),
            },
        ),
        Some(Status::Retired) => marks.push("retired".to_string()),
        Some(Status::Approved) | None => {}
    }
    if entry.kind() == Some(Kind::Lesson)
        && entry.status() == Some(Status::Approved)
        && matches!(
            approval::mark(&ctx.state, &entry.id(), &item.raw),
            Mark::Changed
        )
    {
        marks.push("changed after approval".to_string());
    }
    marks
}

/// A typed path as stored (R-XOS-2). The project folder itself is not a file of a task, so it is
/// refused (Code::BadValue), as is a path outside the project folder.
fn stored_path(ctx: &Ctx, cwd: &Path, typed: &str) -> Result<String> {
    let stored = paths::stored(&ctx.root, cwd, typed)?;
    if stored.is_empty() {
        return Err(bad("the project folder itself is not a file of a task"));
    }
    Ok(stored)
}

/// A text that is empty is no value.
fn non_empty(text: Option<String>) -> Option<String> {
    text.filter(|text| !text.is_empty())
}

/// A path as people see it: relative to the project folder, with `/` as the separator.
fn shown_path(root: &Path, path: &Path) -> String {
    let relative = path.strip_prefix(root).unwrap_or(path);
    relative
        .to_string_lossy()
        .replace(std::path::MAIN_SEPARATOR, "/")
}

/// A failure for a value the person gave, Code::BadValue.
fn bad(message: impl Into<String>) -> Error {
    Error::new(Code::BadValue, message)
}
