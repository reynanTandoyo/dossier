//! The `digest` command (docs/design.md section 18, R-CTX-14, R-CTX-15): the material for summing
//! up a milestone. The title, goal, status and tasks come from `todo`; the approved decisions,
//! assumptions, findings, dead ends and investigations of the milestone's tasks come from the
//! dossier, with the rules for the summary. It changes nothing. Writing the digest entry
//! (`--write --stdin`) is not built yet.

use crate::cli::Global;
use crate::ctx::Ctx;
use crate::digest::RULES;
use crate::entry::{Entry, Kind, Status};
use crate::error::{Code, Error, Result};
use crate::output::Output;
use crate::store::{self, Item};
use crate::todo_bridge;
use serde_json::{Value, json};
use std::path::Path;

/// The heading over the tasks of the milestone.
const TASKS: &str = "## Tasks";
/// The heading over the approved decisions of the tasks.
const DECISIONS: &str = "## Decisions";
/// The heading over the approved assumptions of the tasks.
const ASSUMPTIONS: &str = "## Assumptions";
/// The heading over the approved findings that are not dead ends.
const FINDINGS: &str = "## Findings";
/// The heading over the approved findings whose outcome is a dead end.
const DEAD_ENDS: &str = "## Dead ends";
/// The heading over the approved investigations of the tasks.
const INVESTIGATIONS: &str = "## Investigations";
/// The heading over the rules for the summary.
const RULES_HEADING: &str = "## Rules for the summary";

/// The arguments of `digest`.
#[derive(clap::Args, Debug)]
pub struct Args {
    /// The milestone to sum up, such as T-130
    pub milestone: String,
    /// Write the digest entry from the summary on standard input (not built yet)
    #[arg(long)]
    pub write: bool,
    /// Read the summary from standard input; needs --write
    #[arg(long, requires = "write")]
    pub stdin: bool,
    /// The title of the digest entry; needs --write
    #[arg(long, requires = "write")]
    pub title: Option<String>,
}

/// The milestone as todo gives it: its title, goal and status, and its tasks.
struct Milestone {
    title: String,
    goal: String,
    status: String,
    tasks: Vec<Task>,
}

/// One task of the milestone.
struct Task {
    id: String,
    title: String,
    status: String,
}

/// One heading of the material and the entries under it.
struct Heading {
    name: &'static str,
    entries: Vec<Shown>,
}

/// One entry of the material: its ID and the lines it takes.
struct Shown {
    id: String,
    lines: Vec<String>,
}

/// Runs `digest <milestone>`: the material for the summary. `--write` is not built yet: without
/// `--stdin` it is a usage error, with it Code::NotImplemented.
pub fn run(args: &Args, g: &Global) -> Result<Output> {
    if args.write {
        if !args.stdin {
            return Err(Error::new(
                Code::Usage,
                "digest --write needs --stdin: the summary is read from standard input",
            ));
        }
        return Err(Error::new(
            Code::NotImplemented,
            "digest --write is not built yet",
        ));
    }
    let ctx = Ctx::load(g, "digest")?;
    let milestone = read_milestone(&ctx.root, &args.milestone)?;
    let (items, warnings) = store::all(&ctx.dossier);
    let task_ids = milestone_ids(&args.milestone, &milestone.tasks);
    let headings = headings(&items, &task_ids);

    let mut text = vec![title_line(&args.milestone, &milestone.title)];
    if !milestone.goal.is_empty() {
        text.push(format!("goal: {}", milestone.goal));
    }
    if !milestone.tasks.is_empty() {
        text.push(TASKS.to_string());
        text.extend(milestone.tasks.iter().map(task_line));
    }
    for heading in headings
        .iter()
        .filter(|heading| !heading.entries.is_empty())
    {
        text.push(heading.name.to_string());
        text.extend(heading.entries.iter().flat_map(|shown| shown.lines.clone()));
    }
    text.push(RULES_HEADING.to_string());
    text.extend(
        RULES
            .iter()
            .enumerate()
            .map(|(index, rule)| format!("{}. {rule}", index + 1)),
    );
    text.push(format!(
        "Write it with: dossier digest {} --write --stdin",
        args.milestone
    ));

    let entries: Vec<String> = headings
        .iter()
        .flat_map(|heading| heading.entries.iter().map(|shown| shown.id.clone()))
        .collect();
    let tasks: Vec<Value> = milestone
        .tasks
        .iter()
        .map(|task| json!({"id": task.id, "title": task.title, "status": task.status}))
        .collect();
    let data = json!({
        "milestone": args.milestone,
        "title": milestone.title,
        "goal": milestone.goal,
        "status": milestone.status,
        "tasks": tasks,
        "entries": entries,
        "rules": RULES,
    });
    let mut output = Output::with(text.join("\n"), data);
    for warning in warnings {
        output = output.warn(warning);
    }
    Ok(output)
}

/// The milestone from todo: `show <milestone>` gives its title, goal and status, and
/// `list --milestone <milestone> --status all` its tasks. A call that fails fails the command;
/// without todo it is Code::TodoMissing.
fn read_milestone(root: &Path, milestone: &str) -> Result<Milestone> {
    let shown = todo_bridge::json(root, &["show", milestone])?;
    let listed = todo_bridge::json(root, &["list", "--milestone", milestone, "--status", "all"])?;
    let tasks = listed
        .get("tasks")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .map(|task| Task {
            id: text_of(task, "id"),
            title: text_of(task, "title"),
            status: text_of(task, "status"),
        })
        .collect();
    Ok(Milestone {
        title: first_text([shown.get("title"), inner(&shown, "title")]),
        goal: first_text([shown.get("goal"), inner(&shown, "goal")]),
        status: first_text([shown.get("status"), inner(&shown, "status")]),
        tasks,
    })
}

/// The IDs that count as the milestone's: the milestone itself, then its tasks.
fn milestone_ids(milestone: &str, tasks: &[Task]) -> Vec<String> {
    std::iter::once(milestone.to_string())
        .chain(tasks.iter().map(|task| task.id.clone()))
        .filter(|id| !id.is_empty())
        .collect()
}

/// The headings of the material with their entries, in the order of the text: the approved
/// entries whose `task` is one of `task_ids`, each in the order of `store::all`. A dead end is
/// listed under its own heading and not under Findings.
fn headings(items: &[Item], task_ids: &[String]) -> Vec<Heading> {
    let mine: Vec<&Item> = items
        .iter()
        .filter(|item| item.entry.status() == Some(Status::Approved))
        .filter(|item| in_milestone(&item.entry, task_ids))
        .collect();
    let (dead_ends, findings): (Vec<&Item>, Vec<&Item>) = of_kind(&mine, Kind::Finding)
        .into_iter()
        .partition(|item| is_dead_end(&item.entry));
    vec![
        Heading {
            name: DECISIONS,
            entries: of_kind(&mine, Kind::Decision)
                .into_iter()
                .map(|item| shown(item, false))
                .collect(),
        },
        Heading {
            name: ASSUMPTIONS,
            entries: of_kind(&mine, Kind::Assumption)
                .into_iter()
                .map(|item| shown(item, true))
                .collect(),
        },
        Heading {
            name: FINDINGS,
            entries: findings
                .into_iter()
                .map(|item| shown(item, false))
                .collect(),
        },
        Heading {
            name: DEAD_ENDS,
            entries: dead_ends
                .into_iter()
                .map(|item| shown(item, false))
                .collect(),
        },
        Heading {
            name: INVESTIGATIONS,
            entries: of_kind(&mine, Kind::Investigation)
                .into_iter()
                .map(|item| shown(item, true))
                .collect(),
        },
    ]
}

/// The items of one kind, in their order.
fn of_kind<'a>(items: &[&'a Item], kind: Kind) -> Vec<&'a Item> {
    items
        .iter()
        .copied()
        .filter(|item| item.entry.kind() == Some(kind))
        .collect()
}

/// Whether the entry's `task` field names one of `task_ids`, ignoring case.
fn in_milestone(entry: &Entry, task_ids: &[String]) -> bool {
    entry
        .field("task")
        .is_some_and(|task| task_ids.iter().any(|id| same(id, &task)))
}

/// Whether the finding's outcome is a dead end (format.md §5).
fn is_dead_end(entry: &Entry) -> bool {
    entry
        .field("outcome")
        .is_some_and(|outcome| same(&outcome, "dead end"))
}

/// One entry of the material: `<ID> · <title>`, with ` [<state>]` when `with_state` and the entry
/// has a state, then the first line of its first section indented by two spaces.
fn shown(item: &Item, with_state: bool) -> Shown {
    let entry = &item.entry;
    let mut head = named_text(&entry.id(), &entry.title());
    if with_state {
        let state = entry.field("state").unwrap_or_default();
        let state = state.trim();
        if !state.is_empty() {
            head = format!("{head} [{state}]");
        }
    }
    let mut lines = vec![head];
    if let Some(first) = first_line_of_first_section(entry) {
        lines.push(format!("  {first}"));
    }
    Shown {
        id: entry.id(),
        lines,
    }
}

/// The first line of the entry's first section in file order; None when it has none or the
/// section is empty.
fn first_line_of_first_section(entry: &Entry) -> Option<String> {
    let (_, body) = entry.sections().into_iter().next()?;
    body.lines().next().map(|line| line.trim_end().to_string())
}

/// The first line of the material: `# Digest of <milestone> · <title>`.
fn title_line(milestone: &str, title: &str) -> String {
    format!("# Digest of {}", named_text(milestone, title))
}

/// One task of the milestone: `<ID> · <title> (<status>)`, the parts that are empty left out.
fn task_line(task: &Task) -> String {
    let named = named_text(&task.id, &task.title);
    if task.status.is_empty() {
        named
    } else {
        format!("{named} ({})", task.status)
    }
}

/// `<ID> · <title>`, or the ID alone when there is no title.
fn named_text(id: &str, title: &str) -> String {
    if title.is_empty() {
        id.to_string()
    } else {
        format!("{id} · {title}")
    }
}

/// Whether two IDs or names are the same, ignoring case and surrounding white space.
fn same(a: &str, b: &str) -> bool {
    a.trim().eq_ignore_ascii_case(b.trim())
}

/// The text of `key` in one object of todo's data; empty when it is missing.
fn text_of(value: &Value, key: &str) -> String {
    value
        .get(key)
        .and_then(Value::as_str)
        .unwrap_or_default()
        .trim()
        .to_string()
}

/// The value of `key` in the `fields` object of todo's data, when there is one.
fn inner<'a>(data: &'a Value, key: &str) -> Option<&'a Value> {
    data.get("fields")?.get(key)
}

/// The first string among `values`, trimmed; empty when none of them is a string.
fn first_text<'a>(values: impl IntoIterator<Item = Option<&'a Value>>) -> String {
    values
        .into_iter()
        .flatten()
        .find_map(Value::as_str)
        .unwrap_or_default()
        .trim()
        .to_string()
}
