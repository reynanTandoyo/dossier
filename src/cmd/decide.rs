//! The `decide` command: records a decision with its reason, and who made it. A person's
//! decision is approved at once; an LLM's needs `--said-by <person>` (create::create).

use crate::cli::Global;
use crate::create::{self, New};
use crate::ctx::Ctx;
use crate::entry::{Entry, Kind};
use crate::error::{Code, Error, Result};
use crate::output::Output;
use crate::tx;
use serde_json::json;

/// The longest title made from the decision, in characters.
const TITLE_CHARS: usize = 80;

/// The arguments of `decide`.
#[derive(clap::Args, Debug)]
pub struct Args {
    /// The decision: what was decided
    pub decision: String,
    /// Why it was decided
    #[arg(long)]
    pub reason: String,
    /// What was considered and rejected, and why it was not chosen
    #[arg(long)]
    pub rejected: Option<String>,
    /// The title; by default the first 80 characters of the decision
    #[arg(long)]
    pub title: Option<String>,
    /// The todo task it came from, such as T-042
    #[arg(long)]
    pub task: Option<String>,
    /// The person who made the decision; needed when an LLM records it
    #[arg(long)]
    pub said_by: Option<String>,
    // Accepted but not used yet: the task 'decide: related decisions' implements --replaces.
    #[arg(long)]
    pub replaces: Option<String>,
    // Accepted but not used yet: the task 'decide: related decisions' implements --besides.
    #[arg(long)]
    pub besides: bool,
    /// The machines the decision is about, comma-separated
    #[arg(long)]
    pub machine: Option<String>,
    /// The analyses it is about, comma-separated
    #[arg(long)]
    pub analysis: Option<String>,
    /// The software it is about, comma-separated
    #[arg(long)]
    pub software: Option<String>,
    /// Other tags, comma-separated
    #[arg(long)]
    pub tags: Option<String>,
}

/// Runs `decide`: creates the decision in one transaction and returns its ID and status.
pub fn run(args: &Args, g: &Global) -> Result<Output> {
    let ctx = Ctx::load(g, "decide")?;
    let new = new_decision(args);
    tx::write(&ctx, |tx| {
        let (id, text) = create::create(tx, &new)?;
        let status = Entry::parse(&text)?
            .status()
            .ok_or_else(|| Error::new(Code::Parse, format!("{id} has no status")))?;
        Ok(Output::with(
            format!("{id}: added ({})", status.name()),
            json!({ "id": id, "status": status.name() }),
        ))
    })
}

/// The entry that `decide` asks to create: the decision and its reason, the rejected options
/// when given, the tags and task when given, and the person who said it.
fn new_decision(args: &Args) -> New {
    let mut sections = vec![
        ("Decision".to_string(), args.decision.clone()),
        ("Reason".to_string(), args.reason.clone()),
    ];
    if let Some(rejected) = &args.rejected {
        sections.push(("Rejected".to_string(), rejected.clone()));
    }
    let tags = [
        ("machine", &args.machine),
        ("analysis", &args.analysis),
        ("software", &args.software),
        ("tags", &args.tags),
    ];
    let mut fields: Vec<(String, String)> = tags
        .into_iter()
        .filter_map(|(key, value)| {
            non_empty(value.as_deref()).map(|value| (key.to_string(), value))
        })
        .collect();
    if let Some(task) = non_empty(args.task.as_deref()) {
        fields.push(("task".to_string(), task.to_uppercase()));
    }
    let title = non_empty(args.title.as_deref()).unwrap_or_else(|| default_title(&args.decision));
    New {
        kind: Kind::Decision,
        title,
        fields,
        sections,
        said_by: args.said_by.clone(),
        draft: false,
    }
}

/// The value without its surrounding spaces, or None when nothing is left of it.
fn non_empty(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

/// The first 80 characters of the decision (characters, not bytes), with line breaks turned into
/// spaces and the ends trimmed.
fn default_title(decision: &str) -> String {
    decision
        .chars()
        .take(TITLE_CHARS)
        .map(|c| if matches!(c, '\r' | '\n') { ' ' } else { c })
        .collect::<String>()
        .trim()
        .to_string()
}
