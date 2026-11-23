//! The `solved` command (docs/design.md section 15, R-INV-4): closes an open investigation as
//! solved. The solution goes in its Solution section; when attempts led to it, their numbers
//! follow it as `Attempts: 2, 3`, in the order they were given. Only an approved investigation
//! whose state is `open` is closed, by a person or an LLM. A refused change writes nothing.

use crate::attempts;
use crate::cli::Global;
use crate::ctx::Ctx;
use crate::entry::{Entry, Kind, Status};
use crate::error::{Code, Error, Result};
use crate::output::Output;
use crate::store;
use crate::tx::{self, Change};
use serde_json::json;

/// The value of `kind` in dossier.toml of a team dossier.
const TEAM: &str = "team";

/// The state of an investigation that is solved.
const SOLVED: &str = "solved";

/// The state of an investigation that nobody has closed yet.
const OPEN: &str = "open";

/// The section that holds the solution of an investigation (format.md §28).
const SOLUTION: &str = "Solution";

/// The arguments of `solved`.
#[derive(clap::Args, Debug)]
pub struct Args {
    /// The ID of the investigation, such as INV-003; any case, with or without a place prefix
    pub id: String,
    /// What solved it, in one line
    pub text: String,
    /// The number of an attempt that led to the solution, such as 2; repeat it for several, in
    /// the order they count
    #[arg(long, value_name = "N")]
    pub attempt: Vec<u32>,
}

/// Runs `solved`: checks the solution and the investigation, then, in one transaction, sets the
/// state to solved, the closing time and the Solution section, with a history line and a journal
/// line. The output is `<ID>: solved`, and the data is {id, state}.
pub fn run(args: &Args, g: &Global) -> Result<Output> {
    let ctx = Ctx::load(g, "solved")?;
    ensure_project(&ctx)?;
    let who = ctx.writer()?.clone();
    let text = one_line_text(&args.text, "the solution")?;
    let numbers = distinct(&args.attempt);

    tx::write(&ctx, |tx| {
        let item = store::get(&tx.ctx().dossier, &args.id)?;
        let id = item.entry.id();
        ensure_open(&item.entry, &id, "solved")?;
        let known: Vec<u32> = attempts::of(&item.entry)
            .iter()
            .map(|attempt| attempt.n)
            .collect();
        if let Some(missing) = numbers.iter().find(|n| !known.contains(n)) {
            return Err(Error::new(
                Code::NotFound,
                format!("no attempt {missing} in {id}"),
            ));
        }

        let time = tx.time().to_string();
        let mut entry = item.entry.clone();
        entry.set_field("state", SOLVED);
        entry.set_field("closed", &format!("{time} by {}", who.person));
        entry.set_section(SOLUTION, &solution_body(text, &numbers));
        entry.add_history(&time, &who, SOLVED, "");
        tx.put(
            item.path.clone(),
            Some(&item),
            entry.text(),
            Change {
                id: id.clone(),
                event: SOLVED.to_string(),
                detail: text.to_string(),
            },
        );
        Ok(Output::with(
            format!("{id}: solved"),
            json!({"id": id, "state": SOLVED}),
        ))
    })
}

/// Refuses (Code::Refused) a team dossier: investigations belong to a project (R-INV-1). Shared
/// with `gave-up`.
pub(crate) fn ensure_project(ctx: &Ctx) -> Result<()> {
    if ctx.cfg.kind == TEAM {
        return Err(refused(
            "investigations belong to a project, not to the team dossier".to_string(),
        ));
    }
    Ok(())
}

/// Refuses (Code::Refused) an entry that is not an approved investigation whose state is `open`.
/// The message names what the entry is instead; `command` is the command that refuses, such as
/// `solved`. Shared with `gave-up`.
pub(crate) fn ensure_open(entry: &Entry, id: &str, command: &str) -> Result<()> {
    match entry.kind().or_else(|| Kind::of_id(id)) {
        Some(Kind::Investigation) => {}
        Some(kind) => {
            return Err(refused(format!(
                "{command} closes investigations only; {id} is of kind {}",
                kind.name()
            )));
        }
        None => {
            return Err(refused(format!(
                "{command} closes investigations only; {id} has no valid kind"
            )));
        }
    }
    match entry.status() {
        Some(Status::Approved) => {}
        Some(status) => {
            return Err(refused(format!(
                "only an approved investigation can be closed; {id} is {}",
                status.name()
            )));
        }
        None => {
            return Err(refused(format!(
                "{id} has no valid status, so it cannot be closed"
            )));
        }
    }
    match entry.field("state").as_deref() {
        Some(OPEN) => Ok(()),
        Some(state) => Err(refused(format!(
            "only an open investigation can be closed; {id} is {state}"
        ))),
        None => Err(refused(format!(
            "{id} has no valid state, so it cannot be closed"
        ))),
    }
}

/// The text with its ends trimmed. Refuses (Code::BadValue) a blank text, a text of more than one
/// line, and a text that starts with a heading mark, since it would start a section of its own.
/// `what` names the text in the message, such as `the solution` or `--reason`. Shared with
/// `gave-up`.
pub(crate) fn one_line_text<'a>(text: &'a str, what: &str) -> Result<&'a str> {
    let text = text.trim();
    if text.is_empty() {
        return Err(Error::new(
            Code::BadValue,
            format!("{what} must not be empty"),
        ));
    }
    if text.contains(['\r', '\n']) {
        return Err(Error::new(
            Code::BadValue,
            format!("{what} must be one line"),
        ));
    }
    if text.starts_with("## ") {
        return Err(Error::new(
            Code::BadValue,
            format!("{what} must not start with the heading mark ##"),
        ));
    }
    Ok(text)
}

/// The numbers in the order given, each one once.
fn distinct(numbers: &[u32]) -> Vec<u32> {
    let mut found: Vec<u32> = Vec::new();
    for &number in numbers {
        if !found.contains(&number) {
            found.push(number);
        }
    }
    found
}

/// The body of the Solution section: the solution, then `Attempts: 2, 3` after a blank line when
/// attempts were named.
fn solution_body(text: &str, numbers: &[u32]) -> String {
    if numbers.is_empty() {
        return text.to_string();
    }
    let list: Vec<String> = numbers.iter().map(u32::to_string).collect();
    format!("{text}\n\nAttempts: {}", list.join(", "))
}

/// A refusal with the given message, Code::Refused.
fn refused(message: String) -> Error {
    Error::new(Code::Refused, message)
}
