//! The `found` command: records a finding in one line. A finding is a fact of the project and is
//! approved at once, whoever writes it (R-WRITE-3). `--dead-end` marks something tried that did
//! not work (R-WRITE-16, R-ENTRY-15). `--later` saves a quick note: a draft with the tag `inbox`
//! to be sorted out later (R-WRITE-7).

use crate::cli::Global;
use crate::create::{self, New};
use crate::ctx::Ctx;
use crate::entry::{Entry, Kind, Status};
use crate::error::{Code, Error, Result};
use crate::output::Output;
use crate::tx;
use serde_json::json;

/// How many characters of the observation the default title takes.
const TITLE_CHARS: usize = 80;

/// The tag of a quick note that is sorted out later (R-WRITE-7).
const INBOX: &str = "inbox";

/// The arguments of `found`.
#[derive(clap::Args, Debug)]
pub struct Args {
    /// What was seen in this project, or what was tried and what happened
    pub observation: String,
    /// What the observation rests on, such as a run or a plot
    #[arg(long)]
    pub evidence: Option<String>,
    /// Marks a dead end: something was tried and did not work (outcome: dead end)
    #[arg(long)]
    pub dead_end: bool,
    /// Saves a quick note: a draft with the tag inbox, to be sorted out later
    #[arg(long)]
    pub later: bool,
    /// The title; the default is the first 80 characters of the observation
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
    /// The topics it is about, from the project's list (dossier topics), comma-separated
    #[arg(long)]
    pub topic: Option<String>,
    /// The code it is about: paths, each optionally with ::name, comma-separated (src/pump or src/pump/seal.rs::leak_model)
    #[arg(long)]
    pub code: Option<String>,
}

/// Runs `found`: creates the finding in a transaction and says its ID and status.
pub fn run(args: &Args, g: &Global) -> Result<Output> {
    let observation = args.observation.trim();
    if observation.is_empty() {
        return Err(Error::new(
            Code::BadValue,
            "the observation must not be empty",
        ));
    }
    let ctx = Ctx::load(g, "found")?;
    let new = new_finding(args, observation);
    tx::write(&ctx, |tx| {
        let (id, text) = create::create(tx, &new)?;
        let status = Entry::parse(&text)?.status();
        let message = format!("{id}: added ({})", status_words(status));
        let data = json!({"id": id, "status": status.map_or("", Status::name)});
        Ok(Output::with(message, data))
    })
}

/// What `found` asks to create: the observation as the Observation section, the evidence as the
/// Evidence section when given, the fields the options give, and a draft when `--later` is given.
fn new_finding(args: &Args, observation: &str) -> New {
    let mut fields = Vec::new();
    if args.dead_end {
        fields.push(pair("outcome", "dead end"));
    }
    if let Some(task) = trimmed(args.task.as_deref()) {
        fields.push(pair("task", &task.to_uppercase()));
    }
    for (key, value) in [
        ("machine", &args.machine),
        ("analysis", &args.analysis),
        ("software", &args.software),
        ("topic", &args.topic),
    ] {
        if let Some(value) = trimmed(value.as_deref()) {
            fields.push(pair(key, value));
        }
    }
    if let Some(code) = code_of(args) {
        fields.push(pair("code", &code));
    }
    if let Some(tags) = tags_of(args) {
        fields.push(pair("tags", &tags));
    }
    let mut sections = vec![pair("Observation", observation)];
    if let Some(evidence) = trimmed(args.evidence.as_deref()) {
        sections.push(pair("Evidence", evidence));
    }
    New {
        kind: Kind::Finding,
        title: title_of(args, observation),
        fields,
        sections,
        said_by: None,
        draft: args.later,
    }
}

/// The title: `--title` when it is not blank, else the first 80 characters of the observation.
/// Line breaks become spaces.
fn title_of(args: &Args, observation: &str) -> String {
    match trimmed(args.title.as_deref()) {
        Some(title) => one_line(title),
        None => one_line(&observation.chars().take(TITLE_CHARS).collect::<String>()),
    }
}

/// The value of the tags field: the tags given, with `inbox` added at the end for a quick note.
fn tags_of(args: &Args) -> Option<String> {
    match (trimmed(args.tags.as_deref()), args.later) {
        (Some(tags), true) => Some(format!("{tags}, {INBOX}")),
        (Some(tags), false) => Some(tags.to_string()),
        (None, true) => Some(INBOX.to_string()),
        (None, false) => None,
    }
}

/// The value of the code field: each pointer of `--code` trimmed, with backslashes as slashes,
/// the empty ones left out, joined with `, `; None when no pointer is left.
fn code_of(args: &Args) -> Option<String> {
    let pointers: Vec<String> = args
        .code
        .as_deref()
        .unwrap_or("")
        .split(',')
        .map(|pointer| pointer.trim().replace('\\', "/"))
        .filter(|pointer| !pointer.is_empty())
        .collect();
    (!pointers.is_empty()).then(|| pointers.join(", "))
}

/// The words after the ID in the first output line: `approved`, or `draft, sort it out later`.
fn status_words(status: Option<Status>) -> &'static str {
    match status {
        Some(Status::Draft) => "draft, sort it out later",
        Some(status) => status.name(),
        None => "",
    }
}

/// The text with its ends trimmed, or None when it is not given or blank.
fn trimmed(text: Option<&str>) -> Option<&str> {
    text.map(str::trim).filter(|text| !text.is_empty())
}

/// The text on one line: each line trimmed, the non-empty lines joined with a space.
fn one_line(text: &str) -> String {
    text.split(['\r', '\n'])
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

/// A field or section as a pair of owned strings.
fn pair(key: &str, value: &str) -> (String, String) {
    (key.to_string(), value.to_string())
}
