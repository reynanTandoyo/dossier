//! The `handover` command (docs/design.md section 22, R-HAND-1 and R-HAND-5, D54): gives a task
//! back in todo with a note of what is done and what is next. It is refused while the writer holds
//! files taken for the task (R-HAND-1). Otherwise the note `handover — <text>` is added in todo, the
//! task is released there, and the journal records the event. A todo that refuses or fails stops
//! the command with todo's message and no journal line (R-HAND-5).

use crate::cli::Global;
use crate::ctx::Ctx;
use crate::error::{Code, Error, Result};
use crate::journal;
use crate::output::Output;
use crate::paths;
use crate::taken::{self, Take};
use crate::time;
use crate::todo_bridge;
use serde_json::json;

/// The arguments of `handover`.
#[derive(clap::Args, Debug)]
pub struct Args {
    /// The todo task, such as T-042
    pub task: String,
    /// What is done and what is next
    #[arg(long, value_name = "text")]
    pub note: String,
}

/// Runs `handover`. The first output line is `<task> handed over: handover — <text>`. With
/// --dry-run todo is not called and no journal line is written; a warning says what would happen.
pub fn run(args: &Args, g: &Global) -> Result<Output> {
    let ctx = Ctx::load(g, "handover")?;
    let who = ctx.writer()?;
    let task = args.task.as_str();
    let text = args.note.as_str();

    let (file, _) = taken::load(&ctx.dossier, &ctx.cfg.code)?;
    let held: Vec<Take> = taken::all(&file)
        .into_iter()
        .filter(|take| take.task.eq_ignore_ascii_case(task) && take.person == who.person)
        .collect();
    if !held.is_empty() {
        return Err(Error::new(Code::Refused, held_message(task, &held)));
    }

    let todo_name = match std::env::var("TODO_AGENT") {
        Ok(name) if !name.is_empty() => name,
        _ => who.name.clone(),
    };
    let todo_name = todo_name.as_str();
    let note = format!("handover — {text}");
    let output = Output::with(
        format!("{task} handed over: {note}"),
        json!({"task": task, "note": text}),
    );
    if g.dry_run {
        return Ok(output.warn(format!(
            "dry run: would note \"{note}\" on {task} and release it in todo"
        )));
    }
    todo_bridge::json(
        &ctx.root,
        &["note", task, note.as_str(), "--agent", todo_name],
    )?;
    todo_bridge::json(&ctx.root, &["release", task, "--agent", todo_name])?;

    let record = journal::Record {
        time: time::now(),
        name: who.name.clone(),
        person: who.person.clone(),
        llm: who.is_llm,
        command: "handover".to_string(),
        id: task.to_string(),
        event: "handover".to_string(),
        detail: text.to_string(),
    };
    journal::append(&ctx.state, &record)?;

    Ok(output)
}

/// The refusal for files the writer holds for the task: the task, each file with its take time,
/// and the return command that ends the takes.
fn held_message(task: &str, held: &[Take]) -> String {
    let mut message = format!("{task}: return the files you took first:");
    for take in held {
        message.push_str(&format!(
            "\n  {} (taken {})",
            paths::shown(&take.path),
            take.time
        ));
    }
    let files: Vec<String> = held
        .iter()
        .map(|take| word(&paths::shown(&take.path)))
        .collect();
    message.push_str(&format!(
        "\ndossier return {} --done (or --abandon to drop your changes)",
        files.join(" ")
    ));
    message
}

/// A path as one word of the command line: in double quotes when it has a space.
fn word(path: &str) -> String {
    if path.contains(' ') {
        format!("\"{path}\"")
    } else {
        path.to_string()
    }
}
