//! The `help` command: an overview of the commands, the description and examples of one command,
//! or the rules text for LLMs.

use crate::agents;
use crate::cli::Global;
use crate::error::{Code, Error, Result};
use crate::examples;
use crate::output::Output;
use serde_json::json;

/// The arguments of `help`.
#[derive(clap::Args, Debug)]
pub struct Args {
    /// `agents` prints the rules for LLMs; a command name prints its description and examples;
    /// without a topic, the commands in one overview
    pub topic: Option<String>,
}

/// Every command of all phases of docs/design.md, in the order of the help overview, each with a
/// description of at most five words.
pub const COMMANDS: [(&str, &str); 89] = [
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
    ("propose", "Proposes a decision for approval"),
    ("accept", "Accepts a proposal, at terminal"),
    ("decline", "Declines a proposal, at terminal"),
    ("confirm", "Confirms an assumption"),
    ("refute", "Refutes an assumption, at terminal"),
    ("cleared", "Clears one rests-on mark"),
    ("value", "Sets, shows, approves key values"),
    ("values", "Lists values in force"),
    ("attach", "Attaches files to a task"),
    ("detach", "Marks a task file detached"),
    ("reattach", "Records a file's checksum again"),
    ("files", "Lists task files, with marks"),
    ("which", "Shows where a file belongs"),
    ("used", "Records a software usage run"),
    ("runs", "Lists usage records, newest first"),
    ("image", "Adds or removes entry pictures"),
    ("received", "Records files received from someone"),
    ("delivered", "Records files delivered to someone"),
    ("exchange", "Lists received and delivered records"),
    ("map", "Shows or sets the map"),
    ("hooks", "Installs or removes Claude hooks"),
    ("guard", "Blocks edits inside dossier folders"),
    ("after", "Adds context after todo commands"),
    ("skill", "Installs the Claude Code skill"),
    ("team", "Shows or changes team settings"),
    ("confidential", "Manages confidential terms"),
    ("promote", "Copies an entry to the team"),
    ("close", "Sorts findings for promotion"),
    ("howto", "Records a how-to"),
    ("tool", "Adds or updates a tool"),
    ("where-used", "Shows software usage index"),
    ("term", "Adds or looks up terms"),
    ("glossary", "Lists all glossary terms"),
    ("topics", "Lists or adds project topics"),
    ("reviewed", "Confirms an entry still holds"),
    ("review-folder", "Shows material from old folder"),
    ("review-document", "Starts reviewing a document"),
    ("reviewed-document", "Marks a document done"),
    ("import", "Shows import progress"),
    ("problem", "Opens an investigation"),
    ("tried", "Adds an attempt"),
    ("solved", "Closes an investigation as solved"),
    ("gave-up", "Closes an investigation unsolved"),
    ("take", "Records who took a file"),
    ("return", "Checks or ends a take"),
    ("review-session", "Material from a session file"),
    ("session", "Converts a transcript"),
    ("session-end", "Starts the session review"),
    ("digest", "Sums up a closed milestone"),
    ("story", "Decisions in the order made"),
    ("export", "Writes the project as Markdown"),
    ("schema", "Describes commands and JSON"),
];

/// Runs `help`.
pub fn run(args: &Args, _g: &Global) -> Result<Output> {
    match args.topic.as_deref() {
        None => Ok(overview()),
        Some("agents") => Ok(Output::with(agents::RULES, json!({"rules": agents::RULES}))),
        Some(other) => match COMMANDS.iter().find(|(name, _)| *name == other) {
            Some((name, description)) => Ok(command_help(name, description)),
            None => Err(Error::new(
                Code::Usage,
                format!(
                    "unknown help topic '{other}'; the topics are: agents, and the command names \
                     that `dossier help` lists"
                ),
            )),
        },
    }
}

/// The description of one command, then its examples when it has them: two for PowerShell and one
/// for a Linux shell, each indented by two spaces.
fn command_help(name: &str, description: &str) -> Output {
    let examples = examples::of(name);
    let mut lines = vec![format!("{name}: {description}")];
    if let Some([first, second, linux]) = examples {
        lines.push("Examples (PowerShell):".to_string());
        lines.push(format!("  {first}"));
        lines.push(format!("  {second}"));
        lines.push("Example (Linux shell):".to_string());
        lines.push(format!("  {linux}"));
    }
    let data = json!({
        "command": name,
        "description": description,
        "examples": examples.map(|three| three.to_vec()).unwrap_or_default(),
    });
    Output::with(lines.join("\n"), data)
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
