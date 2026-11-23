//! The `export` command (docs/design.md section 20, R-OUT-7): one Markdown document with the approved
//! decisions, findings, investigations and usage records in time order, each with its sections. With
//! `--milestone` only the entries of the tasks todo lists under that milestone. It changes nothing in
//! the dossier: with `--to` the document goes to a file outside it, and `--dry-run` writes nothing.

use crate::atomic;
use crate::cli::Global;
use crate::ctx::Ctx;
use crate::entry::{Entry, Kind, Status};
use crate::error::{Code, Error, Result};
use crate::output::Output;
use crate::store::{self, Item};
use crate::time;
use crate::todo_bridge;
use serde_json::{Value, json};
use std::path::{Component, Path, PathBuf};

/// The kinds the document holds (R-OUT-7): decisions, findings, investigations and usage records.
const KINDS: [Kind; 4] = [
    Kind::Decision,
    Kind::Finding,
    Kind::Investigation,
    Kind::Usage,
];

/// The arguments of `export`.
#[derive(clap::Args, Debug)]
pub struct Args {
    /// Only the entries of the tasks under this todo milestone, such as T-130
    #[arg(long)]
    pub milestone: Option<String>,
    /// Write the document to this file, relative to the current folder, instead of printing it;
    /// the file must lie outside the dossier and an existing file is replaced
    #[arg(long)]
    pub to: Option<PathBuf>,
}

/// Runs `export`. Without `--to` the document is the text of the output. With `--to` it is written to
/// that file (atomic::replace) and the output says how many entries it holds. A file inside the
/// dossier is Code::BadValue. With `--dry-run` the file is not written and the text says so.
pub fn run(args: &Args, g: &Global) -> Result<Output> {
    let ctx = Ctx::load(g, "export")?;
    let tasks = match args.milestone.as_deref() {
        Some(milestone) => Some(milestone_ids(&ctx.root, milestone)?),
        None => None,
    };
    let (items, warnings) = store::all(&ctx.dossier);
    let chosen = selected(&items, tasks.as_deref());
    let count = chosen.len();
    let now = time::now();
    let document = document(&ctx.cfg.code, time::date(&now), &chosen);

    let mut output = match &args.to {
        None => Output::with(document, json!({"entries": count, "file": null})),
        Some(to) => {
            let cwd = std::env::current_dir()?;
            let target = cwd.join(to);
            refuse_inside(&ctx.dossier, &target)?;
            let shown = to.display().to_string();
            let data = json!({"entries": count, "file": shown});
            if ctx.dry_run {
                Output::with(
                    format!("dry run: would export {count} entries to {shown}"),
                    data,
                )
            } else {
                atomic::replace(&target, document.as_bytes(), None)?;
                Output::with(format!("exported {count} entries to {shown}"), data)
            }
        }
    };
    for warning in warnings {
        output = output.warn(warning);
    }
    Ok(output)
}

/// The IDs that count for `milestone`: the milestone itself, then the tasks todo lists under it,
/// done or not. That is the same call as `story` and `digest` make: `list --milestone <milestone>
/// --status all`. Without todo it is Code::TodoMissing.
fn milestone_ids(root: &Path, milestone: &str) -> Result<Vec<String>> {
    let milestone = milestone.trim();
    if milestone.is_empty() {
        return Err(Error::new(Code::BadValue, "the milestone is empty"));
    }
    let listed = todo_bridge::json(root, &["list", "--milestone", milestone, "--status", "all"])?;
    let tasks = listed
        .get("tasks")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|task| task.get("id").and_then(Value::as_str))
        .map(|id| id.trim().to_string());
    Ok(std::iter::once(milestone.to_string())
        .chain(tasks)
        .collect())
}

/// The approved entries of the kinds in KINDS, and of the milestone's tasks when `tasks` is given,
/// oldest `created` first. The sort is stable, so entries created at the same time keep the ID order
/// of `store::all`.
fn selected<'a>(items: &'a [Item], tasks: Option<&[String]>) -> Vec<&'a Item> {
    let mut chosen: Vec<&Item> = items
        .iter()
        .filter(|item| item.entry.status() == Some(Status::Approved))
        .filter(|item| item.entry.kind().is_some_and(|kind| KINDS.contains(&kind)))
        .filter(|item| tasks.is_none_or(|ids| in_milestone(&item.entry, ids)))
        .collect();
    chosen.sort_by(|left, right| {
        left.entry
            .field("created")
            .cmp(&right.entry.field("created"))
    });
    chosen
}

/// Whether the entry's `task` field is one of `ids`, ignoring case.
fn in_milestone(entry: &Entry, ids: &[String]) -> bool {
    entry
        .field("task")
        .is_some_and(|task| ids.iter().any(|id| id.eq_ignore_ascii_case(task.trim())))
}

/// The whole document: the title line `# <code>: decisions, findings and runs`, the line
/// `exported <day>`, then one part per entry. Parts are separated by a blank line, and the text
/// ends with one line break.
fn document(code: &str, day: &str, chosen: &[&Item]) -> String {
    let mut parts = vec![
        format!("# {code}: decisions, findings and runs"),
        format!("exported {day}"),
    ];
    parts.extend(chosen.iter().map(|item| entry_part(&item.entry)));
    let mut text = parts.join("\n\n");
    text.push('\n');
    text
}

/// One entry: `## <ID> · <title>`, the line of who and when, then `### <name>` and its text for each
/// section in file order. A section with no text keeps only its heading.
fn entry_part(entry: &Entry) -> String {
    let mut parts = vec![
        format!("## {}", named_text(&entry.id(), &entry.title())),
        meta_line(entry),
    ];
    for (name, body) in entry.sections() {
        parts.push(format!("### {name}"));
        let text = body.lines().collect::<Vec<_>>().join("\n");
        if !text.is_empty() {
            parts.push(text);
        }
    }
    parts.join("\n\n")
}

/// `<kind>, <date>, <author>[, task <task>]`, and for a usage record also `software <name>` and
/// `version <v>`. A part the entry does not have is left out.
fn meta_line(entry: &Entry) -> String {
    let mut parts = Vec::new();
    if let Some(kind) = entry.kind() {
        parts.push(kind.name().to_string());
    }
    parts.extend(
        entry
            .field("created")
            .map(|created| time::date(&created).to_string())
            .filter(|date| !date.is_empty()),
    );
    parts.extend(labelled(entry, "author", ""));
    parts.extend(labelled(entry, "task", "task "));
    if entry.kind() == Some(Kind::Usage) {
        parts.extend(labelled(entry, "software", "software "));
        parts.extend(labelled(entry, "version", "version "));
    }
    parts.join(", ")
}

/// The field `key` with `label` in front, when it has a value.
fn labelled(entry: &Entry, key: &str, label: &str) -> Option<String> {
    let value = entry.field(key)?;
    let value = value.trim();
    (!value.is_empty()).then(|| format!("{label}{value}"))
}

/// `<ID> · <title>`, or the ID alone when there is no title.
fn named_text(id: &str, title: &str) -> String {
    let title = title.trim();
    if title.is_empty() {
        id.to_string()
    } else {
        format!("{id} · {title}")
    }
}

/// Code::BadValue when `target` lies inside the dossier: the document must not be written into it.
fn refuse_inside(dossier: &Path, target: &Path) -> Result<()> {
    if lexical(target).starts_with(lexical(dossier)) {
        return Err(Error::new(
            Code::BadValue,
            format!(
                "{} is inside the dossier; export writes outside it",
                target.display()
            ),
        ));
    }
    Ok(())
}

/// The path with its `.` and `..` parts resolved by their text alone, so that a file that does not
/// exist yet can be compared with the dossier.
fn lexical(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for part in path.components() {
        match part {
            Component::CurDir => {}
            Component::ParentDir => {
                out.pop();
            }
            other => out.push(other),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lexical_resolves_the_dot_parts_of_a_path() {
        assert_eq!(
            lexical(Path::new("/projects/P-001/sub/../DOSSIER/./x.md")),
            PathBuf::from("/projects/P-001/DOSSIER/x.md")
        );
        assert_eq!(
            lexical(Path::new("/projects/P-001/DOSSIER/../../x.md")),
            PathBuf::from("/projects/x.md")
        );
    }
}
