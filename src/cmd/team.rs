//! The `team` command (docs/design.md section 12): shows the team settings of the current dossier,
//! and `team use <path>` links a project to its team dossier (R-PLACE-2, format.md §26). The team
//! settings are the members and plain names of dossier.toml (format.md §21); a project with a
//! reachable team dossier also knows the team's members and names, which `Ctx::load` adds. Linking
//! changes dossier.toml, so it is recorded in the journal with who and when (D29), as `people` is.

use crate::cli::Global;
use crate::config;
use crate::ctx::Ctx;
use crate::error::{Code, Error, Result};
use crate::journal;
use crate::names;
use crate::output::Output;
use crate::paths;
use crate::time;
use serde_json::json;
use std::path::{Component, Path, PathBuf};

/// The settings file inside a DOSSIER folder (format.md §10).
const SETTINGS: &str = "dossier.toml";

/// The DOSSIER folder name: a team path may name it, or the folder that holds it.
const FOLDER: &str = "DOSSIER";

/// The `kind` of a team dossier (format.md §26).
const TEAM: &str = "team";

/// The journal ID of a settings change. Settings are not entries, so the ID is not an entry ID
/// (format.md §11).
const SETTINGS_ID: &str = "SETTINGS";

/// The arguments of `team`.
#[derive(clap::Args, Debug)]
pub struct Args {
    #[command(subcommand)]
    pub action: Option<Action>,
}

/// What `team` does besides showing the team settings.
#[derive(clap::Subcommand, Debug)]
pub enum Action {
    /// Links this project to the team dossier at <path>
    Use {
        /// The team dossier: its DOSSIER folder, or the folder that holds it
        path: PathBuf,
    },
}

/// Runs `team`: shows the team dossier, the members and the plain names, or links the project to
/// a team dossier with `team use`.
pub fn run(args: &Args, g: &Global) -> Result<Output> {
    let ctx = Ctx::load(g, "team")?;
    match &args.action {
        None => Ok(show(&ctx)),
        Some(Action::Use { path }) => link(&ctx, g, path),
    }
}

/// Three lines: the team dossier (`none` when there is none), the members and the plain names.
/// The data is {team, members, names}, with `team` as dossier.toml stores it, or null. A team
/// dossier that cannot be reached is a warning, and its members and names are then not listed.
fn show(ctx: &Ctx) -> Output {
    let team = ctx
        .cfg
        .team
        .as_deref()
        .map_or_else(|| "none".to_string(), paths::shown);
    let text = format!(
        "team dossier: {team}\nmembers: {}\nnames: {}",
        listed(&ctx.cfg.members),
        listed(&ctx.cfg.names)
    );
    let data = json!({
        "team": ctx.cfg.team,
        "members": ctx.cfg.members,
        "names": ctx.cfg.names,
    });
    let output = Output::with(text, data);
    match (&ctx.cfg.team, &ctx.team) {
        (Some(stored), None) => output.warn(format!(
            "the team dossier cannot be reached: {}",
            ctx.root.join(paths::shown(stored)).display()
        )),
        _ => output,
    }
}

/// The values joined with ", ", or `none` for an empty list.
fn listed(values: &[String]) -> String {
    if values.is_empty() {
        "none".to_string()
    } else {
        values.join(", ")
    }
}

/// Links the project to the team dossier that `path` names (R-PLACE-2): dossier.toml gets the
/// team's DOSSIER folder as an absolute path with / separators, and the journal records who did
/// it and when. With --dry-run nothing is written.
fn link(ctx: &Ctx, g: &Global, path: &Path) -> Result<Output> {
    // `team use` changes the project's settings. Inside a team dossier, or with --team, the
    // context is the team's, and a team dossier does not link to another team.
    if ctx.cfg.kind == TEAM {
        return Err(Error::new(
            Code::BadValue,
            "a team dossier is not linked to a team: run team use in the project folder, without --team",
        ));
    }
    let folder = team_dossier(path)?;
    let stored = stored(&folder)?;
    let text = format!("linked to the team dossier {}", paths::shown(&stored));
    let data = json!({"team": stored});
    if ctx.dry_run {
        let file = ctx.dossier.join(SETTINGS);
        return Ok(
            Output::with(text, data).warn(format!("dry run: would write {}", file.display()))
        );
    }
    config::set_value(&ctx.dossier, "team", &quoted(&stored))?;
    record_change(ctx, g, &stored)?;
    Ok(Output::with(text, data))
}

/// The DOSSIER folder of the team dossier that `path` names: `path` itself when it holds
/// dossier.toml, else `path`/DOSSIER. Code::NoDossier when neither holds one; Code::BadValue when
/// the settings there are not those of a team dossier (format.md §26).
fn team_dossier(path: &Path) -> Result<PathBuf> {
    let base = absolute(path)?;
    let folder = if base.join(SETTINGS).is_file() {
        base.clone()
    } else {
        base.join(FOLDER)
    };
    if !folder.join(SETTINGS).is_file() {
        return Err(Error::new(
            Code::NoDossier,
            format!(
                "{}: no team dossier here: there is no {SETTINGS} in it or in {FOLDER}",
                base.display()
            ),
        ));
    }
    if config::load(&folder)?.kind != TEAM {
        return Err(Error::new(
            Code::BadValue,
            format!(
                "{} is not a team dossier: its {SETTINGS} is not kind = \"team\"",
                folder.display()
            ),
        ));
    }
    Ok(folder)
}

/// The path as an absolute path without `.` and `..` parts. A relative path is taken from the
/// current folder. Nothing on disk is read, so the team dossier need not exist yet.
fn absolute(path: &Path) -> Result<PathBuf> {
    let full = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .map_err(|error| Error::new(Code::Io, format!("the current folder: {error}")))?
            .join(path)
    };
    let mut out = PathBuf::new();
    for part in full.components() {
        match part {
            Component::CurDir => {}
            Component::ParentDir => {
                out.pop();
            }
            other => out.push(other.as_os_str()),
        }
    }
    Ok(out)
}

/// The team DOSSIER folder as dossier.toml stores it: absolute, with / separators on Windows
/// (format.md §26). A path that is not text is Code::BadValue.
fn stored(folder: &Path) -> Result<String> {
    let text = folder.to_str().ok_or_else(|| {
        Error::new(
            Code::BadValue,
            format!("{}: the path is not text", folder.display()),
        )
    })?;
    Ok(if cfg!(windows) {
        text.replace('\\', "/")
    } else {
        text.to_string()
    })
}

/// A TOML basic string holding `text`: quotes and backslashes escaped (format.md §10).
fn quoted(text: &str) -> String {
    format!("\"{}\"", text.replace('\\', "\\\\").replace('"', "\\\""))
}

/// Appends the journal line of a link: the settings event `changed`, with who and when (D29). The
/// name is the one that acts when it is in the settings; otherwise the name given by --agent or
/// the environment, as `names::unchecked` resolves it.
fn record_change(ctx: &Ctx, g: &Global, stored: &str) -> Result<()> {
    let who = ctx
        .who
        .clone()
        .unwrap_or_else(|| names::unchecked(g.agent.as_deref()));
    let record = journal::Record {
        time: time::now(),
        name: who.name,
        person: who.person,
        llm: who.is_llm,
        command: "team".to_string(),
        id: SETTINGS_ID.to_string(),
        event: "changed".to_string(),
        detail: format!("team {stored}"),
    };
    journal::append(&ctx.state, &record)
}
