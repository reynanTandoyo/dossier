//! The `agents` command: writes the rules block into AGENTS.md, and says whether it is current.
//! When a dossier is found from the current folder, each write is recorded in its journal with
//! who and when (D29).

use crate::agents;
use crate::cli::Global;
use crate::config;
use crate::error::{Code, Error, Result};
use crate::find;
use crate::journal;
use crate::names;
use crate::output::Output;
use crate::time;
use serde_json::json;
use std::path::{Path, PathBuf};

/// The state folder inside the dossier folder (format.md §9).
const STATE: &str = ".state";

/// The journal ID of AGENTS.md. It is not an entry, so it has a fixed ID of its own, as the
/// settings file has (format.md §11).
const ITEM: &str = "AGENTS";

/// The arguments of `agents`.
#[derive(clap::Args, Debug)]
pub struct Args {
    #[command(subcommand)]
    pub action: Action,
}

/// What `agents` does.
#[derive(clap::Subcommand, Debug)]
pub enum Action {
    /// Writes the rules block into AGENTS.md; without --yes it only says so
    Install {
        /// Writes the file
        #[arg(long)]
        yes: bool,
    },
    /// Says whether the rules block in AGENTS.md is current
    Check,
}

/// Runs `agents`.
pub fn run(args: &Args, g: &Global) -> Result<Output> {
    match &args.action {
        Action::Install { yes } => install(*yes, g),
        Action::Check => check(g),
    }
}

/// AGENTS.md in the current folder.
fn agents_file() -> Result<PathBuf> {
    Ok(std::env::current_dir()?.join("AGENTS.md"))
}

/// The text of AGENTS.md; a missing file reads as empty.
fn read_existing(path: &Path) -> Result<String> {
    match std::fs::read_to_string(path) {
        Ok(text) => Ok(text),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(String::new()),
        Err(error) => Err(error.into()),
    }
}

/// Writes the rules block into AGENTS.md when `yes` is given and the run is not a dry run. In a
/// code project the block leaves out the runs and the confidential terms (R-CODE-5).
fn install(yes: bool, g: &Global) -> Result<Output> {
    let path = agents_file()?;
    let existing = read_existing(&path)?;
    let text = agents::installed(&existing, code_project(g)?);
    if !yes {
        return Ok(Output::with(
            "would write the rules block to AGENTS.md; add --yes",
            json!({"written": false}),
        ));
    }
    if g.dry_run {
        return Ok(Output::with(
            "dry run: would write the rules block to AGENTS.md",
            json!({"written": false}),
        ));
    }
    crate::atomic::replace(&path, text.as_bytes(), None)?;
    record_write(g)?;
    Ok(Output::with(
        "AGENTS.md: rules block written",
        json!({"written": true}),
    ))
}

/// Appends the journal line of a written rules block: who and when (D29), in the dossier that
/// `find` finds from the current folder or the --dossier folder. Without a dossier nothing is
/// written, and that is not an error.
fn record_write(g: &Global) -> Result<()> {
    let cwd = std::env::current_dir()?;
    let dossier = match find::find(&cwd, g.dossier.as_deref()) {
        Ok(dossier) => dossier,
        Err(error) if error.code == Code::NoDossier => return Ok(()),
        Err(error) => return Err(error),
    };
    let who = names::unchecked(g.agent.as_deref());
    let record = journal::Record {
        time: time::now(),
        name: who.name,
        person: who.person,
        llm: who.is_llm,
        command: "agents".to_string(),
        id: ITEM.to_string(),
        event: "changed".to_string(),
        detail: "rules block written".to_string(),
    };
    journal::append(&dossier.join(STATE), &record)
}

/// Whether the dossier found from the current folder, or the --dossier folder, is a code project
/// (its settings say `code_project = true`). Without a dossier it is not one, and that is not an
/// error.
fn code_project(g: &Global) -> Result<bool> {
    let cwd = std::env::current_dir()?;
    match find::find(&cwd, g.dossier.as_deref()) {
        Ok(dossier) => Ok(config::load(&dossier)?.code_project),
        Err(error) if error.code == Code::NoDossier => Ok(false),
        Err(error) => Err(error),
    }
}

/// Succeeds when the rules block in AGENTS.md is current; otherwise Code::Refused.
fn check(g: &Global) -> Result<Output> {
    let existing = read_existing(&agents_file()?)?;
    if agents::current(&existing, code_project(g)?) {
        return Ok(Output::with(
            "AGENTS.md: rules block is current",
            json!({"current": true}),
        ));
    }
    Err(Error::new(
        Code::Refused,
        "AGENTS.md: rules block is missing or out of date; run: dossier agents install --yes",
    ))
}
