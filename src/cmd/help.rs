//! The `help` command: an overview of the commands, or the rules text for LLMs.

use crate::agents;
use crate::cli::Global;
use crate::error::{Code, Error, Result};
use crate::output::Output;
use serde_json::json;

/// The arguments of `help`.
#[derive(clap::Args, Debug)]
pub struct Args {
    /// `agents` prints the rules for LLMs; without a topic, the commands in one overview
    pub topic: Option<String>,
}

/// Every command of docs/design.md section 5, in that order, each with a five-word description.
const COMMANDS: [(&str, &str); 37] = [
    ("init", "Creates the dossier folder here"),
    ("where", "Shows folders and project code"),
    ("people", "Adds or extends a person"),
    ("add", "Adds any kind of entry"),
    ("decide", "Records a decision and reason"),
    ("found", "Records a finding or dead-end"),
    ("learn", "Records a lesson with evidence"),
    ("assume", "Records an assumption, still open"),
    ("set", "Changes fields and named sections"),
    ("retitle", "Changes title and renames file"),
    ("link", "Links two entries both ways"),
    ("edit", "Edits an entry by hand"),
    ("show", "Prints one entry with marks"),
    ("list", "Lists entries matching filter options"),
    ("search", "Finds entries matching given words"),
    ("history", "Shows events and saved snapshots"),
    ("diff", "Shows changes since a snapshot"),
    ("restore", "Brings back a saved snapshot"),
    ("log", "Shows who did what, when"),
    ("review-task", "Shows material and drafting rules"),
    ("drafts", "Lists drafts, oldest entries first"),
    ("approve", "Approves drafts, at a terminal"),
    ("reject", "Rejects a draft, optional reason"),
    ("supersede", "Replaces one entry with another"),
    ("retire", "Ends an entry without replacement"),
    ("remove", "Deletes an entry, person only"),
    ("reopen", "Reopens an entry for revision"),
    ("brief", "Gives session or task context"),
    ("status", "Shows the project's current state"),
    ("check", "Lists problems, one line each"),
    ("agents", "Installs or checks AGENTS.md rules"),
    ("help", "Shows overview or agent rules"),
    ("pack", "Writes a dossier archive file"),
    ("unpack", "Restores a dossier from archive"),
    ("connect", "Points a folder at server"),
    ("projects", "Lists or removes registered projects"),
    ("todo", "Runs todo in project folder"),
];

/// Runs `help`.
pub fn run(args: &Args, _g: &Global) -> Result<Output> {
    match args.topic.as_deref() {
        None => Ok(overview()),
        Some("agents") => Ok(Output::with(agents::RULES, json!({"rules": agents::RULES}))),
        Some(other) => Err(Error::new(
            Code::Usage,
            format!("unknown help topic '{other}'; the topics are: agents"),
        )),
    }
}

/// One line per command: two spaces, the name padded to 12 characters, two spaces, the
/// description.
fn overview() -> Output {
    let lines: Vec<String> = COMMANDS
        .iter()
        .map(|(name, description)| format!("  {name:<12}  {description}"))
        .collect();
    let commands: Vec<serde_json::Value> = COMMANDS
        .iter()
        .map(|(name, description)| json!({"name": name, "description": description}))
        .collect();
    Output::with(lines.join("\n"), json!({"commands": commands}))
}
