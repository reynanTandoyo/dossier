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
use crate::glossary::{self, Term};
use crate::output::Output;
use crate::select::{self, Query};
use crate::store::{self, Item};
use crate::tx::{self, Change};
use chrono::NaiveDate;
use serde_json::json;

/// The values of `confidence` of a lesson (format.md §5, R-ENTRY-6).
const CONFIDENCE: [&str; 4] = ["measured", "simulated", "experience", "guess"];

/// The longest title made from the statement, in characters.
const TITLE_CHARS: usize = 80;

/// How many similar lessons a new lesson names (R-WRITE-11).
const SIMILAR_LIMIT: usize = 5;

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
    /// and confidence are replaced, and the tags and the review date given are set
    #[arg(long)]
    pub update: Option<String>,
    /// The date the lesson should be checked again, as YYYY-MM-DD, such as 2027-06-01; none by
    /// default (R-LIFE-7)
    #[arg(long)]
    pub review: Option<String>,
    /// Where the check is that catches the mistake: a test, a lint rule, a hook or a step of a
    /// review; the lesson then owes this check until dossier checked says it is there (R-WRITE-18)
    #[arg(long, value_name = "TEXT", conflicts_with = "update")]
    pub check: Option<String>,
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
    /// The topics it is about, from the project's list (dossier topics), comma-separated
    #[arg(long)]
    pub topic: Option<String>,
    /// The code it is about: paths, each optionally with ::name, comma-separated (src/pump or src/pump/seal.rs::leak_model)
    #[arg(long)]
    pub code: Option<String>,
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
    // Checked before the dossier is looked for, like the confidence.
    let review = review_of(args.review.as_deref())?;

    let ctx = Ctx::load(g, "learn")?;
    match args.update.as_deref() {
        Some(id) => update(&ctx, args, id, &confidence, review.as_deref()),
        None => add(&ctx, args, &confidence, review.as_deref()),
    }
}

/// Writes a new lesson in one transaction. Before it is written, the lessons most like it are
/// named as warnings, chosen by word stems and rare words, with the glossary's spellings added to
/// the query (select::top). Files that do not parse are left out here; `check` reports them.
fn add(ctx: &Ctx, args: &Args, confidence: &str, review: Option<&str>) -> Result<Output> {
    let new = lesson(args, confidence, review);
    let query = similarity_query(args, &new.title);
    tx::write(ctx, |tx| {
        let (items, _) = store::all(&tx.ctx().dossier);
        let candidates: Vec<Item> = items.into_iter().filter(in_use_lesson).collect();
        let query = select::widened(&query, &glossary_terms(tx.ctx()));
        let warnings: Vec<String> = select::top(&candidates, &query, SIMILAR_LIMIT)
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
/// author stay as they are; a review date given is set. The rules of `set` apply to an approved
/// lesson: an LLM's change makes it a draft, and a person's change is recorded as its new approved
/// text. A person's change that leaves the lesson unable to be approved (a section left empty) is
/// refused.
fn update(
    ctx: &Ctx,
    args: &Args,
    id: &str,
    confidence: &str,
    review: Option<&str>,
) -> Result<Output> {
    check_topics(&ctx.cfg.topics, args.topic.as_deref())?;
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
        if let Some(review) = review {
            entry.set_field("review", review);
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

/// When the project has topics, each comma-separated piece of `--topic`, trimmed, must equal a
/// topic of the list ignoring case, as `set` checks it (R-TOPIC-2). Code::Usage names the unknown
/// piece and the topics of the project. Used by `--update`, which does not go through create.
fn check_topics(topics: &[String], topic: Option<&str>) -> Result<()> {
    if topics.is_empty() {
        return Ok(());
    }
    let Some(value) = given(topic) else {
        return Ok(());
    };
    for name in value
        .split(',')
        .map(str::trim)
        .filter(|name| !name.is_empty())
    {
        let wanted = name.to_lowercase();
        if !topics.iter().any(|topic| topic.to_lowercase() == wanted) {
            return Err(Error::new(
                Code::Usage,
                format!(
                    "unknown topic {name}; the topics of this project: {} (dossier topics)",
                    topics.join(", ")
                ),
            ));
        }
    }
    Ok(())
}

/// The new lesson: its sections, its fields (confidence, the check it owes, task, the review date
/// and the tags that were given) and who said it. Empty options are left out. A check given with
/// `--check` is written as owed (R-WRITE-18).
fn lesson(args: &Args, confidence: &str, review: Option<&str>) -> New {
    let mut fields = vec![("confidence".to_string(), confidence.to_string())];
    if let Some(check) = given(args.check.as_deref()) {
        fields.push(("check".to_string(), check));
        fields.push(("check-state".to_string(), "owed".to_string()));
    }
    if let Some(task) = given(args.task.as_deref()) {
        fields.push(("task".to_string(), task.to_uppercase()));
    }
    if let Some(review) = review {
        fields.push(("review".to_string(), review.to_string()));
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

/// The tag options that were given, and the code pointers, as (field, value), without the empty
/// ones.
fn tag_fields(args: &Args) -> Vec<(&'static str, String)> {
    let mut fields: Vec<(&'static str, String)> = [
        ("machine", args.machine.as_deref()),
        ("analysis", args.analysis.as_deref()),
        ("software", args.software.as_deref()),
        ("tags", args.tags.as_deref()),
        ("topic", args.topic.as_deref()),
    ]
    .into_iter()
    .filter_map(|(key, value)| given(value).map(|value| (key, value)))
    .collect();
    if let Some(code) = code_of(args) {
        fields.push(("code", code));
    }
    fields
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

/// The `--review` option as YYYY-MM-DD, or None when it is not given. Code::BadValue when it is
/// not a date of that form.
fn review_of(text: Option<&str>) -> Result<Option<String>> {
    text.map(|text| {
        NaiveDate::parse_from_str(text.trim(), "%Y-%m-%d")
            .map(|day| day.format("%Y-%m-%d").to_string())
            .map_err(|_| {
                Error::new(
                    Code::BadValue,
                    format!("--review needs a date such as 2027-06-01, not \"{text}\""),
                )
            })
    })
    .transpose()
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

/// The glossary terms of the dossier, or none when its glossary cannot be read: the similar
/// lessons are then chosen by the words alone.
fn glossary_terms(ctx: &Ctx) -> Vec<Term> {
    match glossary::load(&ctx.dossier, &ctx.cfg.code) {
        Ok((file, _)) => glossary::terms(&file),
        Err(_) => Vec::new(),
    }
}

/// What a new lesson is compared with: the words of its title and statement, and the values of
/// its tag options (each comma-separated value on its own, as the entries hold them). Common
/// words are left out by select::top, so they alone make no lessons similar.
fn similarity_query(args: &Args, title: &str) -> Query {
    let words = select::words(&format!("{title} {}", args.statement));
    let tags = [
        args.machine.as_deref(),
        args.analysis.as_deref(),
        args.software.as_deref(),
        args.tags.as_deref(),
        args.topic.as_deref(),
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
