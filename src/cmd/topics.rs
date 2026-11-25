//! The `topics` command (docs/design.md section 21.2, R-TOPIC-1 to 5): prints the topics of the
//! project's list, one per line, each with the number of approved entries that name it, and adds
//! a topic to it with `topics add` (a person only). The list is the top-level `topics` key of
//! dossier.toml.

use crate::cli::Global;
use crate::ctx::Ctx;
use crate::entry::{Entry, Kind, Status};
use crate::error::{Code, Error, Result};
use crate::output::Output;
use crate::store;
use serde_json::json;

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

/// Runs `topics`: with no action the topics of the settings with their entry counts. `topics add`
/// is not part of this change yet and is refused with Code::NotImplemented.
pub fn run(args: &Args, g: &Global) -> Result<Output> {
    let ctx = Ctx::load(g, "topics")?;
    match &args.action {
        None => list(&ctx),
        Some(Action::Add { .. }) => Err(Error::new(
            Code::NotImplemented,
            "dossier topics add is not available yet",
        )),
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
