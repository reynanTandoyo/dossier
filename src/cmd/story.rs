//! The `story` command (docs/design.md section 20, R-READ-7): the approved decisions in the order
//! they were made, each with its reason and its task. A decision that was superseded stays in the
//! story, marked with the decision that replaced it; a proposal or a draft is not a decision yet
//! and is left out. `--milestone <task>` keeps the decisions whose `task` is the milestone or one of
//! its tasks in todo. Nothing is changed.

use crate::cli::Global;
use crate::ctx::Ctx;
use crate::entry::{Entry, Kind, Status};
use crate::error::{Code, Error, Result};
use crate::output::Output;
use crate::store::{self, Item};
use crate::{time, todo_bridge};
use chrono::{DateTime, FixedOffset};
use serde_json::{Value, json};
use std::path::Path;

/// What `story --milestone` says when todo is not installed (R-READ-7).
const NEEDS_TODO: &str = "story --milestone needs todo";

/// What `story` says when there is no approved decision to tell.
const NONE: &str = "no decisions yet";

/// The arguments of `story`.
#[derive(clap::Args, Debug)]
pub struct Args {
    /// Only the decisions of this todo milestone and its tasks, such as T-130
    #[arg(long)]
    pub milestone: Option<String>,
}

/// One decision of the story: what the text and the JSON data show.
struct Shown {
    id: String,
    title: String,
    /// The date the decision was created, `YYYY-MM-DD`; empty when the entry has no created time.
    date: String,
    /// The text of the Reason section; empty when it has none.
    reason: String,
    task: Option<String>,
    superseded_by: Option<String>,
    /// `[superseded by <ID>]` (or `[superseded]` without the field) for a superseded decision.
    mark: Option<String>,
}

/// Runs `story`: the approved and superseded decisions, oldest `created` first (then by ID), each
/// with its reason and task. With `--milestone`, only those whose task is the milestone or one of
/// its tasks in todo; without todo that is Code::TodoMissing.
pub fn run(args: &Args, g: &Global) -> Result<Output> {
    let ctx = Ctx::load(g, "story")?;
    let wanted = match args.milestone.as_deref() {
        Some(milestone) => Some(milestone_ids(&ctx.root, milestone)?),
        None => None,
    };
    let (items, warnings) = store::all(&ctx.dossier);
    let mut decisions: Vec<&Item> = items
        .iter()
        .filter(|item| item.entry.kind() == Some(Kind::Decision))
        .filter(|item| {
            matches!(
                item.entry.status(),
                Some(Status::Approved | Status::Superseded)
            )
        })
        .filter(|item| {
            wanted
                .as_ref()
                .is_none_or(|ids| in_milestone(&item.entry, ids))
        })
        .collect();
    decisions.sort_by_key(|item| {
        let instant = created_instant(&item.entry);
        (instant.is_none(), instant, item.entry.id())
    });

    let shown: Vec<Shown> = decisions.iter().map(|item| shown(&item.entry)).collect();
    let output = if shown.is_empty() {
        Output::with(NONE, json!({"decisions": []}))
    } else {
        let mut lines = Vec::new();
        for (index, decision) in shown.iter().enumerate() {
            if index > 0 {
                lines.push(String::new());
            }
            lines.extend(text_lines(decision));
        }
        let data: Vec<Value> = shown.iter().map(json_of).collect();
        Output::with(lines.join("\n"), json!({"decisions": data}))
    };
    Ok(warnings
        .into_iter()
        .fold(output, |output, warning| output.warn(warning)))
}

/// The IDs that count as the milestone's: the milestone itself, then the IDs of its tasks from
/// `todo list --milestone <milestone> --status all`. Without todo: Code::TodoMissing, naming the
/// command. Any other failure of todo is passed on.
fn milestone_ids(root: &Path, milestone: &str) -> Result<Vec<String>> {
    let listed = todo_bridge::json(root, &["list", "--milestone", milestone, "--status", "all"])
        .map_err(|error| {
            if error.code == Code::TodoMissing {
                Error::new(Code::TodoMissing, NEEDS_TODO)
            } else {
                error
            }
        })?;
    let tasks = listed
        .get("tasks")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|task| task.get("id").and_then(Value::as_str))
        .map(|id| id.trim().to_string())
        .filter(|id| !id.is_empty());
    Ok(std::iter::once(milestone.trim().to_string())
        .chain(tasks)
        .collect())
}

/// Whether the entry's `task` field names one of `ids`, ignoring case and surrounding white space.
fn in_milestone(entry: &Entry, ids: &[String]) -> bool {
    entry.field("task").is_some_and(|task| {
        ids.iter()
            .any(|id| id.trim().eq_ignore_ascii_case(task.trim()))
    })
}

/// The created time of an entry (format.md §7), when it parses. Different offsets compare right.
fn created_instant(entry: &Entry) -> Option<DateTime<FixedOffset>> {
    entry
        .field("created")
        .and_then(|stamp| DateTime::parse_from_rfc3339(&stamp).ok())
}

/// What the story shows of one decision.
fn shown(entry: &Entry) -> Shown {
    let superseded_by = present(entry.field("superseded-by"));
    let mark = match entry.status() {
        Some(Status::Superseded) => Some(match &superseded_by {
            Some(by) => format!("[superseded by {by}]"),
            None => "[superseded]".to_string(),
        }),
        _ => None,
    };
    Shown {
        id: entry.id(),
        title: entry.title().trim().to_string(),
        date: entry
            .field("created")
            .map(|stamp| time::date(&stamp).to_string())
            .unwrap_or_default(),
        reason: entry.section("Reason").unwrap_or_default(),
        task: present(entry.field("task")),
        superseded_by,
        mark,
    }
}

/// The lines of one decision: `<date>  <ID> · <title>` with its mark, the Reason indented by two
/// spaces, then `task: <task>` when it has one. A blank line inside the reason stays blank.
fn text_lines(decision: &Shown) -> Vec<String> {
    let named = if decision.title.is_empty() {
        decision.id.clone()
    } else {
        format!("{} · {}", decision.id, decision.title)
    };
    let mut head = if decision.date.is_empty() {
        named
    } else {
        format!("{}  {named}", decision.date)
    };
    if let Some(mark) = &decision.mark {
        head = format!("{head} {mark}");
    }
    let mut lines = vec![head];
    lines.extend(indented(&decision.reason));
    if let Some(task) = &decision.task {
        lines.push(format!("  task: {task}"));
    }
    lines
}

/// Each line of `body` indented by two spaces. A blank line stays blank.
fn indented(body: &str) -> Vec<String> {
    body.lines()
        .map(|line| {
            if line.trim().is_empty() {
                String::new()
            } else {
                format!("  {line}")
            }
        })
        .collect()
}

/// The JSON object of one decision: `id`, `title`, `date`, `reason`, and `task` and
/// `superseded_by` (null when the entry has none).
fn json_of(decision: &Shown) -> Value {
    json!({
        "id": decision.id,
        "title": decision.title,
        "date": decision.date,
        "reason": decision.reason,
        "task": decision.task,
        "superseded_by": decision.superseded_by,
    })
}

/// A field's value, trimmed; None when it is missing or blank.
fn present(value: Option<String>) -> Option<String> {
    value
        .map(|text| text.trim().to_string())
        .filter(|text| !text.is_empty())
}
