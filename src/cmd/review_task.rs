//! The `review-task` command: prints the material for drafting lessons about one task (R-DRAFT-1,
//! R-DRAFT-2): the task as `todo show` gives it, the entries recorded for the task, and the
//! drafting rules with the two commands that write a draft or a dead end. dossier never calls a
//! model (D7): the LLM in the session drafts from this text. Nothing is changed.

use crate::approval::{self, Mark};
use crate::cli::Global;
use crate::ctx::Ctx;
use crate::entry::{Kind, Status};
use crate::error::Result;
use crate::output::Output;
use crate::store::{self, Item};
use crate::todo_bridge;
use serde_json::{Value, json};

/// The drafting rules of R-DRAFT-2, in the order of the requirement.
const RULES: [&str; 10] = [
    "Propose a lesson only for something that would change how a later project is done.",
    "Propose none when nothing was learned.",
    "State the conditions.",
    "Give the evidence.",
    "Say what was not checked.",
    "One lesson per draft.",
    "Never include a confidential term in a team draft.",
    "Check for an existing entry first.",
    "Record what was tried and did not work as a dead end, because that is what gets repeated.",
    "Record what had to be assumed.",
];

/// The arguments of `review-task`.
#[derive(clap::Args, Debug)]
pub struct Args {
    /// The todo task to review, such as T-042
    pub task: String,
}

/// Runs `review-task`: reads the task from `todo show`, finds the entries whose `task` field
/// names it, and prints the material and the drafting rules. Nothing is written.
pub fn run(args: &Args, g: &Global) -> Result<Output> {
    let ctx = Ctx::load(g, "review-task")?;
    let data = todo_bridge::json(&ctx.root, &["show", args.task.as_str()])?;
    let (items, warnings) = store::all(&ctx.dossier);
    let recorded: Vec<&Item> = items
        .iter()
        .filter(|item| task_is(item, &args.task))
        .collect();
    let examples = example_commands(&args.task);

    let mut lines = vec![format!("# Review of {}", args.task), String::new()];
    lines.extend(task_lines(&data));
    lines.extend([
        String::new(),
        "## Entries recorded for this task".to_string(),
    ]);
    if recorded.is_empty() {
        lines.push("none".to_string());
    } else {
        lines.extend(recorded.iter().map(|item| entry_line(&ctx, item)));
    }
    lines.extend([String::new(), "## Drafting rules".to_string()]);
    lines.extend(
        RULES
            .iter()
            .enumerate()
            .map(|(index, rule)| format!("{}. {rule}", index + 1)),
    );
    lines.extend([String::new(), "Examples:".to_string()]);
    lines.extend(examples.iter().cloned());

    let entries: Vec<Value> = recorded.iter().map(|item| entry_data(&ctx, item)).collect();
    let output = Output::with(
        lines.join("\n"),
        json!({
            "task": args.task,
            "todo": data,
            "entries": entries,
            "rules": RULES,
            "examples": examples,
        }),
    );
    Ok(warnings
        .into_iter()
        .fold(output, |output, warning| output.warn(warning)))
}

/// The lines that describe the task, from the data of `todo show`: its scalar fields (and lists
/// of scalars) as `key: value`, then the fields object as `key: value`, then the steps numbered,
/// then the notes as `<date> <agent>: <text>`. Null values, objects and lists of objects are not
/// printed as fields; steps and notes have their own lines.
fn task_lines(data: &Value) -> Vec<String> {
    let Some(object) = data.as_object() else {
        return Vec::new();
    };
    let mut lines = Vec::new();
    for (key, value) in object {
        if matches!(key.as_str(), "steps" | "notes" | "fields") {
            continue;
        }
        if let Some(text) = text_of(value) {
            lines.push(format!("{key}: {text}"));
        }
    }
    if let Some(Value::Object(fields)) = object.get("fields") {
        for (key, value) in fields {
            if let Some(text) = text_of(value) {
                lines.push(format!("{key}: {text}"));
            }
        }
    }
    if let Some(Value::Array(steps)) = object.get("steps").filter(|steps| is_filled(steps)) {
        lines.push("steps:".to_string());
        for (index, step) in steps.iter().enumerate() {
            let text = text_of(step).unwrap_or_else(|| step.to_string());
            lines.push(format!("{}. {text}", index + 1));
        }
    }
    if let Some(Value::Array(notes)) = object.get("notes").filter(|notes| is_filled(notes)) {
        lines.push("notes:".to_string());
        lines.extend(notes.iter().map(note_line));
    }
    lines
}

/// Whether a value is a list with at least one item.
fn is_filled(value: &Value) -> bool {
    value.as_array().is_some_and(|items| !items.is_empty())
}

/// The text of a value that is printed on one line: a string, a number, a boolean, or a list of
/// those (its items joined with `, `). Null, an object and a list that holds an object give None.
fn text_of(value: &Value) -> Option<String> {
    match value {
        Value::String(text) => Some(text.clone()),
        Value::Number(_) | Value::Bool(_) => Some(value.to_string()),
        Value::Array(items) => {
            let parts = items
                .iter()
                .filter(|item| !item.is_null())
                .map(text_of)
                .collect::<Option<Vec<String>>>()?;
            (!parts.is_empty()).then(|| parts.join(", "))
        }
        Value::Null | Value::Object(_) => None,
    }
}

/// One note of `todo show` as `<date> <agent>: <text>`. A note that is not an object is printed
/// as its text.
fn note_line(note: &Value) -> String {
    if !note.is_object() {
        return text_of(note).unwrap_or_else(|| note.to_string());
    }
    let field = |key: &str| note.get(key).and_then(text_of).unwrap_or_default();
    let head = [field("date"), field("agent")]
        .into_iter()
        .filter(|part| !part.is_empty())
        .collect::<Vec<String>>()
        .join(" ");
    let text = field("text");
    if head.is_empty() {
        text
    } else {
        format!("{head}: {text}")
    }
}

/// The two commands that write what the drafting rules ask for, with the task filled in: a
/// lesson (a draft when an LLM writes it) and a dead end (a finding, approved at once).
fn example_commands(task: &str) -> Vec<String> {
    vec![
        format!(
            "dossier learn \"<statement>\" --valid-for \"<text>\" --evidence \"<text>\" \
             --confidence <measured|simulated|experience|guess> --task {task}"
        ),
        format!("dossier found --dead-end \"<what was tried and what happened>\" --task {task}"),
    ]
}

/// Whether the entry's `task` field names `task`, ignoring case and surrounding white space.
fn task_is(item: &Item, task: &str) -> bool {
    item.entry
        .field("task")
        .is_some_and(|value| value.trim().eq_ignore_ascii_case(task.trim()))
}

/// The marks of an entry, the same as `show` gives them: its status, then `[changed after
/// approval]` for an approved lesson whose text differs from its recorded checksum.
fn marks(ctx: &Ctx, item: &Item) -> Vec<String> {
    let entry = &item.entry;
    let mut marks = Vec::new();
    match entry.status() {
        Some(Status::Draft) => marks.push("[draft]".to_string()),
        Some(Status::Rejected) => marks.push("[rejected]".to_string()),
        Some(Status::Superseded) => marks.push(
            match entry.field("superseded-by").filter(|by| !by.is_empty()) {
                Some(by) => format!("[superseded by {by}]"),
                None => "[superseded]".to_string(),
            },
        ),
        Some(Status::Retired) => marks.push("[retired]".to_string()),
        Some(Status::Proposed) => marks.push("[proposed]".to_string()),
        Some(Status::Declined) => marks.push("[declined]".to_string()),
        Some(Status::Approved) | None => {}
    }
    if entry.kind() == Some(Kind::Lesson)
        && entry.status() == Some(Status::Approved)
        && matches!(
            approval::mark(&ctx.state, &entry.id(), &item.raw),
            Mark::Changed
        )
    {
        marks.push("[changed after approval]".to_string());
    }
    marks
}

/// One entry as `<ID> · <kind> · <marks> · <title>`; the marks are left out when there are none.
fn entry_line(ctx: &Ctx, item: &Item) -> String {
    let entry = &item.entry;
    let mut parts = vec![
        entry.id(),
        entry.kind().map_or("unknown", Kind::name).to_string(),
    ];
    parts.extend(marks(ctx, item));
    parts.push(entry.title());
    parts.join(" · ")
}

/// The data of one entry for `--json`: ID, kind, status, marks and title.
fn entry_data(ctx: &Ctx, item: &Item) -> Value {
    let entry = &item.entry;
    json!({
        "id": entry.id(),
        "kind": entry.kind().map(Kind::name),
        "status": entry.status().map(Status::name),
        "marks": marks(ctx, item),
        "title": entry.title(),
    })
}
