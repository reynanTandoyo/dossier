//! The `people` command: adds a person and the LLMs that write for them to dossier.toml, and lists
//! them (format.md §10). No name is needed, because the names are what it sets up. Each added
//! person is recorded in the journal with who and when (D29).

use crate::cli::Global;
use crate::config;
use crate::error::{Code, Error, Result};
use crate::find;
use crate::journal;
use crate::names;
use crate::output::Output;
use crate::time;
use serde_json::{Value, json};
use std::path::Path;

/// The settings file inside the DOSSIER folder (format.md §10).
const SETTINGS: &str = "dossier.toml";

/// The state folder inside the DOSSIER folder (format.md §9).
const STATE: &str = ".state";

/// The journal ID of a settings change. Settings are not entries, so the ID is not an entry ID
/// (format.md §11).
const SETTINGS_ID: &str = "SETTINGS";

/// The arguments of `people`.
#[derive(clap::Args, Debug)]
pub struct Args {
    #[command(subcommand)]
    pub action: Action,
}

/// What `people` does.
#[derive(clap::Subcommand, Debug)]
pub enum Action {
    /// Adds a person, or adds LLMs to a person who is already listed
    Add {
        /// The person's name, such as anna
        person: String,
        /// An LLM that writes for the person, such as anna-claude; give the flag again for more
        #[arg(long)]
        llm: Vec<String>,
    },
    /// Lists the people and their LLMs
    List,
}

/// Runs `people`: adds a person to the settings, or lists the people in them.
pub fn run(args: &Args, g: &Global) -> Result<Output> {
    let cwd = std::env::current_dir()
        .map_err(|error| Error::new(Code::Io, format!("the current folder: {error}")))?;
    // A relative --dossier is taken from the current folder, as Ctx::load does.
    let explicit = g.dossier.as_ref().map(|path| cwd.join(path));
    let dossier = find::find(&cwd, explicit.as_deref())?;
    match &args.action {
        Action::Add { person, llm } => add(&dossier, g, person, llm),
        Action::List => list(&dossier),
    }
}

/// Adds `person` with `llms` to dossier.toml, and journals the change. With --dry-run nothing is
/// written.
fn add(dossier: &Path, g: &Global, person: &str, llms: &[String]) -> Result<Output> {
    check_name(person, "a person's name")?;
    for llm in llms {
        check_name(llm, "an LLM's name")?;
    }
    let text = added_text(person, llms);
    if g.dry_run {
        let file = dossier.join(SETTINGS);
        return Ok(Output::text(text).warn(format!("dry run: would write {}", file.display())));
    }
    config::add_person(dossier, person, llms)?;
    record_change(dossier, g, person, llms)?;
    Ok(Output::with(text, json!({"person": person, "llms": llms})))
}

/// The line of a successful add: `added anna`, or `added anna with anna-claude, anna-codex`.
fn added_text(person: &str, llms: &[String]) -> String {
    if llms.is_empty() {
        format!("added {person}")
    } else {
        format!("added {person} with {}", llms.join(", "))
    }
}

/// Appends the journal line of an add: the settings event `changed`, with who and when (D29) and
/// the person and LLMs as its detail.
fn record_change(dossier: &Path, g: &Global, person: &str, llms: &[String]) -> Result<()> {
    let who = names::unchecked(g.agent.as_deref());
    let record = journal::Record {
        time: time::now(),
        name: who.name,
        person: who.person,
        llm: who.is_llm,
        command: "people".to_string(),
        id: SETTINGS_ID.to_string(),
        event: "changed".to_string(),
        detail: change_detail(person, llms),
    };
    journal::append(&dossier.join(STATE), &record)
}

/// `person anna`, or `person anna with anna-claude, anna-codex` when LLMs were given.
fn change_detail(person: &str, llms: &[String]) -> String {
    if llms.is_empty() {
        format!("person {person}")
    } else {
        format!("person {person} with {}", llms.join(", "))
    }
}

/// One line per person, in the order of the settings. The data is a list of {name, llms}.
fn list(dossier: &Path) -> Result<Output> {
    let cfg = config::load(dossier)?;
    let lines: Vec<String> = cfg
        .people
        .iter()
        .map(|person| person_line(&person.name, &person.llms))
        .collect();
    let data: Vec<Value> = cfg
        .people
        .iter()
        .map(|person| json!({"name": person.name, "llms": person.llms}))
        .collect();
    Ok(Output::with(lines.join("\n"), Value::Array(data)))
}

/// `anna: anna-claude, anna-codex`, or `ben:` for a person without LLMs.
fn person_line(name: &str, llms: &[String]) -> String {
    if llms.is_empty() {
        format!("{name}:")
    } else {
        format!("{name}: {}", llms.join(", "))
    }
}

/// A name of a person or an LLM must not be empty and must not contain a space (or any other
/// whitespace), so that it stays one word in the settings, the entries and the journal.
fn check_name(name: &str, what: &str) -> Result<()> {
    if name.is_empty() || name.chars().any(char::is_whitespace) {
        return Err(Error::new(
            Code::BadValue,
            format!("{what} must not be empty or contain a space"),
        ));
    }
    Ok(())
}
