//! The `learn` command: records a lesson, what holds beyond this project, with the conditions it
//! holds under, the evidence for it and how sure we are (R-WRITE-4, R-ENTRY-6). What a person
//! writes is approved at once; what an LLM writes is a draft until a person approves it (D5).
//!
//! A new lesson first names the lessons most like it, as warnings, so that the writer can update
//! one of them instead (R-WRITE-11). `--update <ID>` changes an existing lesson instead of adding
//! one, under the same rules as `set` (R-WRITE-8).

use crate::approval;
use crate::cli::Global;
use crate::create::{self, New};
use crate::ctx::Ctx;
use crate::entry::{Entry, Kind, Status, rules};
use crate::error::{Code, Error, Result};
use crate::output::Output;
use crate::select::{self, Query};
use crate::store::{self, Item};
use crate::tx::{self, Change};
use serde_json::json;

/// The values of `confidence` of a lesson (format.md §5, R-ENTRY-6).
const CONFIDENCE: [&str; 4] = ["measured", "simulated", "experience", "guess"];

/// The longest title made from the statement, in characters.
const TITLE_CHARS: usize = 80;

/// How many similar lessons a new lesson names (R-WRITE-11).
const SIMILAR_LIMIT: usize = 5;

/// Common English words. They are left out of the words a new lesson is compared by, so that
/// "the" or "with" alone does not make two lessons look alike.
const COMMON_WORDS: &[&str] = &[
    "about", "after", "also", "although", "and", "are", "been", "before", "being", "between",
    "both", "but", "can", "could", "does", "during", "each", "for", "from", "had", "has", "have",
    "into", "its", "may", "more", "most", "must", "not", "only", "our", "out", "over", "should",
    "some", "such", "than", "that", "the", "their", "them", "then", "there", "these", "they",
    "this", "those", "through", "under", "very", "was", "were", "when", "where", "which", "while",
    "will", "with", "without", "would",
];

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
    /// Changes the lesson with this ID instead of adding one: its statement, valid-for, evidence
    /// and confidence are replaced, and the tags given are set
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

/// Runs `learn`: checks the confidence and the statement, then writes the lesson, or changes the
/// lesson named by `--update`, in one transaction. The first line of the output is
/// `<ID>: added (approved)` or `<ID>: added (draft, waiting for approval)` for a new lesson, and
/// `<ID>: updated` (with `(back to draft, waiting for approval)` after it when an LLM took an
/// approved lesson back to draft) for an update.
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
    match args.update.as_deref() {
        Some(id) => update(&ctx, args, id, &confidence),
        None => add(&ctx, args, &confidence),
    }
}

/// Writes a new lesson in one transaction. Before it is written, the lessons most like it are
/// named as warnings. Files that do not parse are left out here; `check` reports them.
fn add(ctx: &Ctx, args: &Args, confidence: &str) -> Result<Output> {
    let new = lesson(args, confidence);
    let query = similarity_query(args, &new.title);
    tx::write(ctx, |tx| {
        let (items, _) = store::all(&tx.ctx().dossier);
        let candidates: Vec<Item> = items.into_iter().filter(in_use_lesson).collect();
        let warnings: Vec<String> = select::best(&candidates, &query, SIMILAR_LIMIT)
            .into_iter()
            .map(similar_warning)
            .collect();

        let (id, text) = create::create(tx, &new)?;
        let status = Entry::parse(&text)?
            .status()
            .ok_or_else(|| Error::new(Code::Parse, "the new entry has no status"))?;
        let mut output = Output::with(
            format!("{id}: added ({})", status_words(status)),
            json!({"id": id, "status": status.name()}),
        );
        for warning in warnings {
            output = output.warn(warning);
        }
        Ok(output)
    })
}

/// Changes the lesson `id` in one transaction (R-WRITE-11). Its statement, valid-for and evidence
/// sections and its confidence are replaced, and the tags given are set; its title, task and
/// author stay as they are. The rules of `set` apply to an approved lesson: an LLM's change makes
/// it a draft, and a person's change is recorded as its new approved text. A person's change that
/// leaves the lesson unable to be approved (a section left empty) is refused.
fn update(ctx: &Ctx, args: &Args, id: &str, confidence: &str) -> Result<Output> {
    let who = ctx.writer()?.clone();
    tx::write(ctx, |tx| {
        let item = store::get(&tx.ctx().dossier, id)?;
        let entry_id = item.entry.id();
        if item.entry.kind() != Some(Kind::Lesson) {
            return Err(Error::new(
                Code::Refused,
                format!("{entry_id} is not a lesson; --update changes a lesson only"),
            ));
        }

        let mut entry = item.entry.clone();
        entry.set_section("Statement", &args.statement);
        entry.set_section("Valid for", &args.valid_for);
        entry.set_section("Evidence", &args.evidence);
        entry.set_field("confidence", confidence);
        for (key, value) in tag_fields(args) {
            entry.set_field(key, &value);
        }

        let approved = entry.status() == Some(Status::Approved);
        let mut detail = "updated".to_string();
        let mut back_to_draft = false;
        if approved && who.is_llm {
            entry.set_field("status", Status::Draft.name());
            entry.remove_field("approved");
            entry.remove_field("valid-from");
            detail.push_str(", back to draft");
            back_to_draft = true;
        } else if approved {
            let what: Vec<String> = rules::problems(&entry)
                .into_iter()
                .map(|problem| problem.what)
                .collect();
            if !what.is_empty() {
                return Err(Error::new(
                    Code::MissingField,
                    format!("cannot approve lesson: {}", what.join("; ")),
                ));
            }
        }
        entry.add_history(tx.time(), &who, "changed", &detail);

        let text = entry.text();
        if approved && !tx.ctx().dry_run {
            let state = &tx.ctx().state;
            if who.is_llm {
                approval::forget(state, &entry_id)?;
            } else {
                approval::record(state, &entry_id, text.as_bytes())?;
            }
        }

        let status = entry.status().map_or("", Status::name);
        tx.put(
            item.path.clone(),
            Some(&item),
            text,
            Change {
                id: entry_id.clone(),
                event: "changed".to_string(),
                detail: detail.clone(),
            },
        );
        let message = if back_to_draft {
            format!("{entry_id}: updated (back to draft, waiting for approval)")
        } else {
            format!("{entry_id}: updated")
        };
        Ok(Output::with(
            message,
            json!({"id": entry_id, "status": status}),
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
    fields.extend(
        tag_fields(args)
            .into_iter()
            .map(|(key, value)| (key.to_string(), value)),
    );
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

/// The tag options that were given, as (field, value), without the empty ones.
fn tag_fields(args: &Args) -> Vec<(&'static str, String)> {
    [
        ("machine", args.machine.as_deref()),
        ("analysis", args.analysis.as_deref()),
        ("software", args.software.as_deref()),
        ("tags", args.tags.as_deref()),
    ]
    .into_iter()
    .filter_map(|(key, value)| given(value).map(|value| (key, value)))
    .collect()
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

/// Whether an entry is a lesson that is still in use: any status except superseded and retired.
fn in_use_lesson(item: &Item) -> bool {
    item.entry.kind() == Some(Kind::Lesson)
        && item
            .entry
            .status()
            .is_some_and(|status| !matches!(status, Status::Superseded | Status::Retired))
}

/// What a new lesson is compared with: the words of its title and statement, without the common
/// English words, and the values of its tag options (each comma-separated value on its own, as
/// the entries hold them).
fn similarity_query(args: &Args, title: &str) -> Query {
    let words = select::words(&format!("{title} {}", args.statement))
        .into_iter()
        .filter(|word| !COMMON_WORDS.contains(&word.as_str()))
        .collect();
    let tags = [
        args.machine.as_deref(),
        args.analysis.as_deref(),
        args.software.as_deref(),
        args.tags.as_deref(),
    ]
    .into_iter()
    .flatten()
    .flat_map(|value| value.split(','))
    .map(str::trim)
    .filter(|tag| !tag.is_empty())
    .map(str::to_string)
    .collect();
    Query { tags, words }
}

/// The warning that names a similar lesson: its ID and title, and its status in brackets when it
/// is a draft or rejected.
fn similar_warning(item: &Item) -> String {
    let mark = match item.entry.status() {
        Some(Status::Draft) => " [draft]",
        Some(Status::Rejected) => " [rejected]",
        _ => "",
    };
    format!("similar: {} {}{mark}", item.entry.id(), item.entry.title())
}

/// The words that say what became of the lesson, after its ID.
fn status_words(status: Status) -> &'static str {
    match status {
        Status::Approved => "approved",
        Status::Draft => "draft, waiting for approval",
        other => other.name(),
    }
}
