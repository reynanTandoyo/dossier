//! The `after` command (docs/design.md section 14, R-AGENT-5): a Claude Code hook runs it after a
//! `todo` command, and it prints the dossier's context for that moment. It never fails: every
//! error ends as no output and exit 0. Its text comes from running this program again in the
//! current folder.

use crate::cli::Global;
use crate::error::Result;
use crate::hooks::{self, TodoEvent};
use crate::output::Output;
use serde_json::Value;
use std::io::Read;
use std::process::{Command, Stdio};

/// The arguments of `after`: none.
#[derive(clap::Args, Debug)]
pub struct Args {}

/// Runs `after`: reads the Claude Code payload from standard input and, when it holds a todo
/// event, prints the event's text as `hooks::context` gives it. Any other input, and an event
/// with no text, prints nothing and succeeds.
pub fn run(_args: &Args, _g: &Global) -> Result<Output> {
    let payload = read_payload();
    let Some(event) = hooks::todo_event(&payload) else {
        return Ok(Output::text(""));
    };
    let text = text_of(&event);
    if text.is_empty() {
        return Ok(Output::text(""));
    }
    Ok(Output::text(hooks::context(&text)))
}

/// Standard input as text. A read error, or input that is not UTF-8, counts as empty.
fn read_payload() -> String {
    let mut payload = String::new();
    if std::io::stdin().read_to_string(&mut payload).is_err() {
        payload.clear();
    }
    payload
}

/// The text of an event (design.md section 14): the parts that ran, each without its trailing
/// line breaks, joined with one empty line. Empty when no part gives text.
fn text_of(event: &TodoEvent) -> String {
    let parts = match event {
        TodoEvent::Claimed(id) => vec![own(&["brief", "--task", id.as_str()])],
        TodoEvent::Done(id) => vec![own(&["review-task", id.as_str()]), usage_reminder(id)],
        TodoEvent::Stuck(id) => vec![Some(stuck_text(id))],
    };
    parts
        .into_iter()
        .flatten()
        .map(|part| part.trim_end().to_string())
        .filter(|part| !part.is_empty())
        .collect::<Vec<String>>()
        .join("\n\n")
}

/// The standard output of this program run with `args` in the current folder, with standard
/// input empty and standard error captured. None unless it started and ended with exit 0.
fn own(args: &[&str]) -> Option<String> {
    let program = std::env::current_exe().ok()?;
    let output = Command::new(program)
        .args(args)
        .stdin(Stdio::null())
        .output()
        .ok()?;
    output
        .status
        .success()
        .then(|| String::from_utf8_lossy(&output.stdout).into_owned())
}

/// The reminder that a finished task has no usage record: given when `list --kind usage` answers
/// with an empty `data` for the task. None when it does not answer or the task has a record.
fn usage_reminder(id: &str) -> Option<String> {
    let text = own(&["list", "--kind", "usage", "--task", id, "--all", "--json"])?;
    let value: Value = serde_json::from_str(&text).ok()?;
    let entries = value.get("data")?.as_array()?;
    entries.is_empty().then(|| {
        format!(
            "No usage record for {id}: if software was used, record it with dossier used <software> --version <v> --task {id}"
        )
    })
}

/// The text for a task that was given back: what to record, so that the planner finds it.
fn stuck_text(id: &str) -> String {
    format!(
        "{id} was given back. Record what was missing or did not work, so that the planner finds it: dossier found \"<what was tried>\" --dead-end --task {id}, or dossier assume \"<what was assumed>\" --reason \"<why>\" --task {id}"
    )
}
