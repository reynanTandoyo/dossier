//! The `agents` command: writes the rules block into AGENTS.md, and says whether it is current.
//! `install --yes` also renews the Claude Code skill .claude/skills/dossier/SKILL.md of the current
//! folder when that file exists and is not current; `check` fails when either is out of date.
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

/// The journal ID of the Claude Code skill file, the same ID that `skill install` uses.
const SKILL_ITEM: &str = "SKILL";

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

/// The Claude Code skill file in the current folder (the file that `skill install` writes).
fn skill_file() -> Result<PathBuf> {
    Ok(std::env::current_dir()?
        .join(".claude")
        .join("skills")
        .join("dossier")
        .join("SKILL.md"))
}

/// The text of AGENTS.md; a missing file reads as empty.
fn read_existing(path: &Path) -> Result<String> {
    match std::fs::read_to_string(path) {
        Ok(text) => Ok(text),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(String::new()),
        Err(error) => Err(error.into()),
    }
}

/// Writes the rules block into AGENTS.md when `yes` is given and the run is not a dry run, and
/// then renews the skill file when it is out of date (see `renew_skill`).
fn install(yes: bool, g: &Global) -> Result<Output> {
    let path = agents_file()?;
    let existing = read_existing(&path)?;
    let text = agents::installed(&existing);
    if !yes {
        return Ok(Output::with(
            "would write the rules block to AGENTS.md; add --yes",
            json!({"written": false, "skill": false}),
        ));
    }
    if g.dry_run {
        return Ok(Output::with(
            "dry run: would write the rules block to AGENTS.md",
            json!({"written": false, "skill": false}),
        ));
    }
    crate::atomic::replace(&path, text.as_bytes(), None)?;
    record_write(g, ITEM, "rules block written")?;
    let skill = renew_skill(g)?;
    let message = if skill {
        "AGENTS.md: rules block written\n.claude/skills/dossier/SKILL.md: skill written"
    } else {
        "AGENTS.md: rules block written"
    };
    Ok(Output::with(
        message,
        json!({"written": true, "skill": skill}),
    ))
}

/// Rewrites the skill file of the current folder with `agents::skill` when the file exists and is
/// not current, and journals that write. A missing file is left alone, and no folder is created.
/// Returns whether the file was rewritten.
fn renew_skill(g: &Global) -> Result<bool> {
    let path = skill_file()?;
    if !path.is_file() || agents::skill_current(&read_existing(&path)?) {
        return Ok(false);
    }
    crate::atomic::replace(&path, agents::skill().as_bytes(), None)?;
    record_write(g, SKILL_ITEM, "skill written")?;
    Ok(true)
}

/// Appends the journal line of a written `id` with the detail `detail`: who and when (D29), in the
/// dossier that `find` finds from the current folder or the --dossier folder. The writer is
/// resolved against the settings as the other commands do (D34); when that fails, the name is
/// taken as it is. Without a dossier nothing is written, and that is not an error.
fn record_write(g: &Global, id: &str, detail: &str) -> Result<()> {
    let cwd = std::env::current_dir()?;
    let dossier = match find::find(&cwd, g.dossier.as_deref()) {
        Ok(dossier) => dossier,
        Err(error) if error.code == Code::NoDossier => return Ok(()),
        Err(error) => return Err(error),
    };
    let who = config::load(&dossier)
        .and_then(|cfg| names::resolve(&cfg, g.agent.as_deref()))
        .unwrap_or_else(|_| names::unchecked(g.agent.as_deref()));
    let record = journal::Record {
        time: time::now(),
        name: who.name,
        person: who.person,
        llm: who.is_llm,
        command: "agents".to_string(),
        id: id.to_string(),
        event: "changed".to_string(),
        detail: detail.to_string(),
    };
    journal::append(&dossier.join(STATE), &record)
}

/// Succeeds when the rules block in AGENTS.md is current and, when the skill file exists, the skill
/// file is current too; otherwise Code::Refused.
fn check() -> Result<Output> {
    let existing = read_existing(&agents_file()?)?;
    if !agents::current(&existing) {
        return Err(Error::new(
            Code::Refused,
            "AGENTS.md: rules block is missing or out of date; run: dossier agents install --yes",
        ));
    }
    let skill = skill_file()?;
    if skill.is_file() && !agents::skill_current(&read_existing(&skill)?) {
        return Err(Error::new(
            Code::Refused,
            ".claude/skills/dossier/SKILL.md is out of date; run: dossier agents install --yes",
        ));
    }
    Ok(Output::with(
        "AGENTS.md: rules block is current",
        json!({"current": true}),
    ))
}
