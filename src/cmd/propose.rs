//! The `propose` command: suggests a decision for a decider to accept or decline (R-WRITE-12).
//! Anyone may propose, an LLM too. A proposal has status `proposed` and is not in force until a
//! decider accepts it with `accept`. It is written like `decide` writes a decision, with the draft
//! flag, so create::create makes it a proposal.

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

/// The arguments of `propose`.
#[derive(clap::Args, Debug)]
pub struct Args {
    /// The decision that is suggested: what should be decided
    pub decision: String,
    /// Why it should be decided this way
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
    /// The machines it is about, comma-separated
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

/// Runs `propose`: creates the proposal in one transaction and says its ID, its status and how a
/// decider accepts it. The data is {id, status}.
pub fn run(args: &Args, g: &Global) -> Result<Output> {
    let ctx = Ctx::load(g, "propose")?;
    let new = new_proposal(args);
    tx::write(&ctx, |tx| {
        let (id, text) = create::create(tx, &new)?;
        let status = Entry::parse(&text)?
            .status()
            .ok_or_else(|| Error::new(Code::Parse, format!("{id} has no status")))?;
        let message = format!(
            "{id}: {}; a decider accepts it with: dossier accept {id}",
            status.name()
        );
        let data = json!({"id": id, "status": status.name()});
        Ok(Output::with(message, data))
    })
}

/// The entry that `propose` asks to create: the decision and its reason, the rejected options
/// when given, the tags and task when given. It is a draft decision, so it is a proposal, and it
/// names no person: create::create writes a person's proposal as the person's, and an LLM's as
/// the LLM's own person, via the LLM.
fn new_proposal(args: &Args) -> New {
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
        said_by: None,
        draft: true,
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
