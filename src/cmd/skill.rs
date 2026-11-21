//! The `skill` command (docs/design.md section 9, R-AGENT-4): writes the Claude Code skill
//! `.claude/skills/dossier/SKILL.md` into the current folder. The text comes from `agents::skill`.
//! A dossier found from the current folder gets a journal line with who and when (D29).

use crate::agents;
use crate::atomic;
use crate::cli::Global;
use crate::error::{Code, Result};
use crate::find;
use crate::journal;
use crate::names;
use crate::output::Output;
use crate::time;
use serde_json::json;
use std::path::{Path, PathBuf};

/// The file the skill is written to, relative to the current folder.
const FILE: &str = ".claude/skills/dossier/SKILL.md";

/// The name of the state folder inside the dossier (format.md §9).
const STATE: &str = ".state";

/// The ID of the journal lines of this command (format.md §11).
const ID: &str = "SKILL";

/// The arguments of `skill`.
#[derive(clap::Args, Debug)]
pub struct Args {
    #[command(subcommand)]
    pub action: Action,
}

/// What `skill` does.
#[derive(clap::Subcommand, Debug)]
pub enum Action {
    /// Writes .claude/skills/dossier/SKILL.md; without --yes it only says so
    Install {
        /// Writes the file
        #[arg(long)]
        yes: bool,
    },
}

/// Runs `skill`.
pub fn run(args: &Args, g: &Global) -> Result<Output> {
    match &args.action {
        Action::Install { yes } => install(*yes, g),
    }
}

/// Writes the skill file when `yes` is given and the run is not a dry run. Without `yes` nothing
/// is written. The dossier is found before the file is written, so that a refused `--dossier`
/// changes nothing; the journal line comes after the write, and only when a dossier is found.
fn install(yes: bool, g: &Global) -> Result<Output> {
    if !yes {
        return Ok(Output::with(
            format!("would write {FILE}; add --yes"),
            json!({"file": FILE, "written": false}),
        ));
    }
    if g.dry_run {
        return Ok(Output::with(
            format!("dry run: would write {FILE}"),
            json!({"file": FILE, "written": false}),
        ));
    }

    let cwd = std::env::current_dir()?;
    let state = journal_state(&cwd, g)?;
    atomic::replace(&target(&cwd), agents::skill().as_bytes(), None)?;
    if let Some(state) = state {
        journal::append(&state, &record(g))?;
    }
    Ok(Output::with(
        format!("{FILE}: skill written"),
        json!({"file": FILE, "written": true}),
    ))
}

/// The skill file in `cwd`, with the folders of FILE.
fn target(cwd: &Path) -> PathBuf {
    FILE.split('/')
        .fold(cwd.to_path_buf(), |path, part| path.join(part))
}

/// The state folder of the dossier that `skill` journals to: the dossier found from `cwd`, or the
/// one `--dossier` names. None when no dossier is found; a `--dossier` that is not one is refused.
fn journal_state(cwd: &Path, g: &Global) -> Result<Option<PathBuf>> {
    let explicit = g.dossier.as_ref().map(|path| cwd.join(path));
    match find::find(cwd, explicit.as_deref()) {
        Ok(dossier) => Ok(Some(dossier.join(STATE))),
        Err(error) if error.code == Code::NoDossier && explicit.is_none() => Ok(None),
        Err(error) => Err(error),
    }
}

/// The journal line of a change made by `skill`, with who and when (D29).
fn record(g: &Global) -> journal::Record {
    let who = names::unchecked(g.agent.as_deref());
    journal::Record {
        time: time::now(),
        name: who.name,
        person: who.person,
        llm: who.is_llm,
        command: "skill".to_string(),
        id: ID.to_string(),
        event: "changed".to_string(),
        detail: "skill written".to_string(),
    }
}
