//! The `topics` command (docs/design.md section 21.2, R-TOPIC-1 to 5): prints the topics of the
//! project's list, one per line, each with the number of approved entries that name it, and adds
//! a topic to it with `topics add` (a person only). The list is the top-level `topics` key of
//! dossier.toml.

use crate::cli::Global;
use crate::config;
use crate::ctx::Ctx;
use crate::entry::{Entry, Kind, Status};
use crate::error::{Code, Error, Result};
use crate::journal;
use crate::lock;
use crate::names::Who;
use crate::output::Output;
use crate::store;
use crate::time;
use serde_json::json;

/// The settings file inside a DOSSIER folder (format.md §10).
const SETTINGS: &str = "dossier.toml";

/// The journal ID of the settings events (team.rs and people.rs use the same one).
const SETTINGS_ID: &str = "SETTINGS";

/// The arguments of `topics`: with no action it lists the topics.
#[derive(clap::Args, Debug)]
pub struct Args {
    #[command(subcommand)]
    pub action: Option<Action>,
}

/// What `topics` does besides listing the topics.
#[derive(clap::Subcommand, Debug)]
pub enum Action {
    /// Adds a topic to the project's list; needs a person
    Add {
        /// The name of the topic, such as pumps
        name: String,
    },
}

/// Runs `topics`: with no action the topics of the settings with their entry counts, with `add` a
/// topic is added to the list.
pub fn run(args: &Args, g: &Global) -> Result<Output> {
    let ctx = Ctx::load(g, "topics")?;
    match &args.action {
        None => list(&ctx),
        Some(Action::Add { name }) => add(&ctx, name),
    }
}

/// The topics of the settings, each with the approved entries that name it, then the approved
/// decisions, findings and lessons that name no topic. Drafts are left out. Warnings of
/// `store::all` are passed on.
fn list(ctx: &Ctx) -> Result<Output> {
    let (items, warnings) = store::all(&ctx.dossier);
    let approved: Vec<&Entry> = items
        .iter()
        .map(|item| &item.entry)
        .filter(|entry| entry.status() == Some(Status::Approved))
        .collect();
    let counts: Vec<(&str, usize)> = ctx
        .cfg
        .topics
        .iter()
        .map(|topic| {
            let entries = approved.iter().filter(|entry| names(entry, topic)).count();
            (topic.as_str(), entries)
        })
        .collect();
    let without = approved
        .iter()
        .filter(|entry| is_counted_without_topic(entry) && entry.list("topic").is_empty())
        .count();

    let text = if counts.is_empty() {
        "no topics: dossier topics add <name>".to_string()
    } else {
        let mut lines: Vec<String> = counts
            .iter()
            .map(|(name, entries)| format!("{name}  {entries}"))
            .collect();
        if without > 0 {
            lines.push(format!("(no topic)  {without}"));
        }
        lines.join("\n")
    };
    let topics: Vec<serde_json::Value> = counts
        .iter()
        .map(|(name, entries)| json!({"name": name, "entries": entries}))
        .collect();
    let data = json!({"topics": topics, "without": without});
    Ok(warnings
        .into_iter()
        .fold(Output::with(text, data), |output, warning| {
            output.warn(warning)
        }))
}

/// Whether the entry names the topic, ignoring case: one of the values of its `topic` field.
fn names(entry: &Entry, topic: &str) -> bool {
    let wanted = topic.to_lowercase();
    entry
        .list("topic")
        .iter()
        .any(|named| named.to_lowercase() == wanted)
}

/// Whether a missing topic is counted for the entry: it is a decision, a finding or a lesson.
fn is_counted_without_topic(entry: &Entry) -> bool {
    matches!(
        entry.kind(),
        Some(Kind::Decision | Kind::Finding | Kind::Lesson)
    )
}

/// Adds `name` at the end of the topics of the settings. Only a person may do it: an LLM is
/// Code::NeedsPerson. A name on the list already (ignoring case) changes nothing and is said so.
/// With --dry-run nothing is written.
fn add(ctx: &Ctx, name: &str) -> Result<Output> {
    let who = ctx.writer()?;
    if who.is_llm {
        return Err(Error::new(
            Code::NeedsPerson,
            "a person adds a topic: dossier topics add <name>",
        ));
    }
    let name = check_name(name)?;
    if ctx
        .cfg
        .topics
        .iter()
        .any(|topic| topic.to_lowercase() == name)
    {
        return Ok(Output::with(
            format!("{name} is already a topic"),
            json!({"topic": name, "added": false, "topics": ctx.cfg.topics}),
        ));
    }
    let mut topics = ctx.cfg.topics.clone();
    topics.push(name.clone());
    let output = Output::with(
        format!("added topic {name}"),
        json!({"topic": name, "added": true, "topics": topics}),
    );
    if ctx.dry_run {
        let file = ctx.dossier.join(SETTINGS);
        return Ok(output.warn(format!("dry run: would write {}", file.display())));
    }
    let _guard = lock::acquire(&ctx.state, ctx.cfg.lock_wait_seconds, &who.name, "topics")?;
    config::set_list(&ctx.dossier, "topics", &topics)?;
    record_change(ctx, who, &name)?;
    Ok(output)
}

/// A topic name as it is written: trimmed and in lower case. Code::Usage when it is empty, holds a
/// comma or a line break, or has more than three words.
fn check_name(name: &str) -> Result<String> {
    let name = name.trim().to_lowercase();
    if name.is_empty() {
        return Err(Error::new(
            Code::Usage,
            "a topic needs a name: dossier topics add <name>",
        ));
    }
    if name.contains([',', '\n', '\r']) {
        return Err(Error::new(
            Code::Usage,
            "a topic is one name: it has no comma and no line break",
        ));
    }
    if name.split_whitespace().count() > 3 {
        return Err(Error::new(Code::Usage, "a topic has at most three words"));
    }
    Ok(name)
}

/// Appends the journal line of an added topic: the settings event `changed`, with who and when
/// (D29) and `topic added: <name>` as its detail.
fn record_change(ctx: &Ctx, who: &Who, name: &str) -> Result<()> {
    let record = journal::Record {
        time: time::now(),
        name: who.name.clone(),
        person: who.person.clone(),
        llm: who.is_llm,
        command: "topics".to_string(),
        id: SETTINGS_ID.to_string(),
        event: "changed".to_string(),
        detail: format!("topic added: {name}"),
    };
    journal::append(&ctx.state, &record)
}
