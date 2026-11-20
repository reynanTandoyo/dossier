//! The `agents` command: writes the rules block into AGENTS.md, and says whether it is current.

use crate::agents;
use crate::cli::Global;
use crate::error::{Code, Error, Result};
use crate::output::Output;
use serde_json::json;
use std::path::{Path, PathBuf};

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
        Action::Check => check(),
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

/// Writes the rules block into AGENTS.md when `yes` is given and the run is not a dry run.
fn install(yes: bool, g: &Global) -> Result<Output> {
    let path = agents_file()?;
    let existing = read_existing(&path)?;
    let text = agents::installed(&existing);
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
    std::fs::write(&path, text.as_bytes())?;
    Ok(Output::with(
        "AGENTS.md: rules block written",
        json!({"written": true}),
    ))
}

/// Succeeds when the rules block in AGENTS.md is current; otherwise Code::Refused.
fn check() -> Result<Output> {
    let existing = read_existing(&agents_file()?)?;
    if agents::current(&existing) {
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
