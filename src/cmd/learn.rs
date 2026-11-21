//! The `learn` command: records a lesson, what holds beyond this project, with the conditions it
//! holds under, the evidence for it and how sure we are (R-WRITE-4, R-ENTRY-6). What a person
//! writes is approved at once; what an LLM writes is a draft until a person approves it (D5).

use crate::cli::Global;
use crate::create::{self, New};
use crate::ctx::Ctx;
use crate::entry::{Entry, Kind, Status};
use crate::error::{Code, Error, Result};
use crate::output::Output;
use crate::tx;
use serde_json::json;

/// The values of `confidence` of a lesson (format.md §5, R-ENTRY-6).
const CONFIDENCE: [&str; 4] = ["measured", "simulated", "experience", "guess"];

/// The longest title made from the statement, in characters.
const TITLE_CHARS: usize = 80;

/// The arguments of `learn`.
#[derive(clap::Args, Debug)]
pub struct Args {
    /// What was learned, in a sentence or two
    pub statement: String,
    /// The conditions the lesson holds under; a lesson without them cannot be approved
    #[arg(long)]
    pub valid_for: String,
    /// What the lesson rests on: a test, a simulation or a report, with its place
    #[arg(long)]
    pub evidence: String,
    /// How sure we are: measured, simulated, experience or guess
    #[arg(long)]
    pub confidence: String,
    /// The title; by default the first 80 characters of the statement
    #[arg(long)]
    pub title: Option<String>,
    /// The todo task it came from, such as T-042
    #[arg(long)]
    pub task: Option<String>,
    /// The person who stated it, when an LLM writes it for them in a session
    #[arg(long)]
    pub said_by: Option<String>,
    // Not used yet: the task "learn: similar entries" implements it, so that a new lesson can
    // update an existing entry instead of adding one.
    /// Not used yet
    #[arg(long)]
    pub update: Option<String>,
    /// Tag: a comma-separated list of machines, such as axial compressor
    #[arg(long)]
    pub machine: Option<String>,
    /// Tag: a comma-separated list of analyses, such as steady RANS, speedline
    #[arg(long)]
    pub analysis: Option<String>,
    /// Tag: a comma-separated list of software, such as ExampleSolver 2406
    #[arg(long)]
    pub software: Option<String>,
    /// Tag: a comma-separated list of free tags
    #[arg(long)]
    pub tags: Option<String>,
}

/// Runs `learn`: checks the confidence and the statement, then writes the lesson in one
/// transaction. The first line of the output is `<ID>: added (approved)` for a person, or
/// `<ID>: added (draft, waiting for approval)` for an LLM.
pub fn run(args: &Args, g: &Global) -> Result<Output> {
    // Checked first, before the dossier is even looked for: a wrong value is refused the same
    // way wherever the command is run.
    let confidence = args.confidence.to_lowercase();
    if !CONFIDENCE.contains(&confidence.as_str()) {
        return Err(Error::new(
            Code::BadValue,
            "confidence must be measured, simulated, experience or guess",
        ));
    }
    if args.statement.trim().is_empty() {
        return Err(Error::new(
            Code::MissingField,
            "the statement is empty: say what was learned",
        ));
    }

    let ctx = Ctx::load(g, "learn")?;
    let new = lesson(args, &confidence);
    tx::write(&ctx, |tx| {
        let (id, text) = create::create(tx, &new)?;
        let status = Entry::parse(&text)?
            .status()
            .ok_or_else(|| Error::new(Code::Parse, "the new entry has no status"))?;
        Ok(Output::with(
            format!("{id}: added ({})", status_words(status)),
            json!({"id": id, "status": status.name()}),
        ))
    })
}

/// The new lesson: its sections, its fields (confidence, task and the tags that were given) and
/// who said it. Empty options are left out.
fn lesson(args: &Args, confidence: &str) -> New {
    let mut fields = vec![("confidence".to_string(), confidence.to_string())];
    if let Some(task) = given(args.task.as_deref()) {
        fields.push(("task".to_string(), task.to_uppercase()));
    }
    let tags = [
        ("machine", args.machine.as_deref()),
        ("analysis", args.analysis.as_deref()),
        ("software", args.software.as_deref()),
        ("tags", args.tags.as_deref()),
    ];
    for (key, value) in tags {
        if let Some(value) = given(value) {
            fields.push((key.to_string(), value));
        }
    }
    New {
        kind: Kind::Lesson,
        title: title_of(args),
        fields,
        sections: vec![
            ("Statement".to_string(), args.statement.clone()),
            ("Valid for".to_string(), args.valid_for.clone()),
            ("Evidence".to_string(), args.evidence.clone()),
        ],
        said_by: args.said_by.clone(),
        draft: false,
    }
}

/// The title given with `--title`, or else the default made from the statement.
fn title_of(args: &Args) -> String {
    given(args.title.as_deref()).unwrap_or_else(|| default_title(&args.statement))
}

/// The first 80 characters of the statement, with line breaks turned into spaces and the ends
/// trimmed. Characters, not bytes, so that a letter with an accent is never cut in two.
fn default_title(statement: &str) -> String {
    let flat = statement
        .split(['\r', '\n'])
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join(" ");
    flat.chars()
        .take(TITLE_CHARS)
        .collect::<String>()
        .trim()
        .to_string()
}

/// The value of an option without its surrounding spaces; None when nothing is left of it.
fn given(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

/// The words that say what became of the lesson, after its ID.
fn status_words(status: Status) -> &'static str {
    match status {
        Status::Approved => "approved",
        Status::Draft => "draft, waiting for approval",
        other => other.name(),
    }
}
