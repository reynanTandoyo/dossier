//! The `team` command (docs/design.md section 12): shows the team settings of the current dossier,
//! `team use <path>` links a project to its team dossier (R-PLACE-2, format.md §26), and `team
//! member add|remove` keeps the list of people who may write there (R-TEAM-7). The team settings
//! are the members and plain names of dossier.toml (format.md §21); a project with a reachable team
//! dossier also knows the team's members and names, which `Ctx::load` adds. Linking and every
//! member change rewrite dossier.toml, so each is recorded in the journal with who and when (D29),
//! as `people` is.

use crate::cli::Global;
use crate::config;
use crate::ctx::Ctx;
use crate::error::{Code, Error, Result};
use crate::gitcommit;
use crate::journal;
use crate::lock;
use crate::names::{self, Who};
use crate::output::Output;
use crate::paths;
use crate::term;
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
    /// Adds a member to the team, or removes one (R-TEAM-7)
    Member {
        #[command(subcommand)]
        action: MemberAction,
    },
}

/// What `team member` does.
#[derive(clap::Subcommand, Debug)]
pub enum MemberAction {
    /// Adds a member: the username, the operating-system login of a person
    Add {
        /// The username, such as tony: one word, with no space
        username: String,
    },
    /// Removes a member; a person at a terminal does it
    Remove {
        /// The username of the member, in any case
        username: String,
    },
}

/// Runs `team`: shows the team dossier, the members and the plain names, links the project to a
/// team dossier with `team use`, or changes the members with `team member`.
pub fn run(args: &Args, g: &Global) -> Result<Output> {
    let ctx = Ctx::load(g, "team")?;
    match &args.action {
        None => Ok(show(&ctx)),
        Some(Action::Use { path }) => link(&ctx, g, path),
        Some(Action::Member { action }) => member(members_ctx(ctx)?, g, action),
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
    let who = ctx
        .who
        .clone()
        .unwrap_or_else(|| names::unchecked(g.agent.as_deref()));
    record_change(ctx, &who, &format!("team {stored}"))?;
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

/// The context the members are changed in (R-TEAM-7): the team dossier of a project that names
/// one, else the current dossier. A team dossier that cannot be reached is Code::NoDossier naming
/// its path (R-PLACE-8): the members to change are the team's.
fn members_ctx(ctx: Ctx) -> Result<Ctx> {
    if ctx.cfg.kind == TEAM {
        return Ok(ctx);
    }
    let Some(stored) = ctx.cfg.team.clone() else {
        return Ok(ctx);
    };
    if ctx.team.is_none() {
        return Err(Error::new(
            Code::NoDossier,
            format!(
                "{}: the team dossier cannot be reached",
                ctx.root.join(paths::shown(&stored)).display()
            ),
        ));
    }
    ctx.team_ctx()
}

/// Runs `team member add` or `team member remove` in `ctx`. Only members write to the team
/// dossier (R-TEAM-7), so the name that acts must be a person of its settings or an LLM of one:
/// Code::Usage without any name, Code::UnknownName for anyone else.
fn member(ctx: Ctx, g: &Global, action: &MemberAction) -> Result<Output> {
    let who = names::resolve(&ctx.cfg, g.agent.as_deref())?;
    match action {
        MemberAction::Add { username } => add_member(&ctx, &who, username),
        MemberAction::Remove { username } => remove_member(&ctx, &who, username),
    }
}

/// Adds `username` to the members. The name is one word (Code::BadValue) and not a member yet
/// (Code::Exists, names compared ignoring case). With --dry-run nothing is written.
fn add_member(ctx: &Ctx, who: &Who, username: &str) -> Result<Output> {
    check_username(username)?;
    if ctx.cfg.members.iter().any(|member| same(member, username)) {
        return Err(Error::new(
            Code::Exists,
            format!("{username} is already a member"),
        ));
    }
    let mut members = ctx.cfg.members.clone();
    members.push(username.to_string());
    let output = Output::with(
        format!("added member {username}"),
        json!({"member": username, "members": members}),
    );
    write_members(
        ctx,
        who,
        &members,
        &format!("member {username} added"),
        output,
    )
}

/// Removes `username` from the members. Only a person may do it, at a terminal (R-TEAM-7): an
/// LLM is Code::NeedsPerson, a run without a terminal Code::NeedsTerminal. The member must exist
/// (Code::NotFound, names compared ignoring case). With --dry-run nothing is written.
fn remove_member(ctx: &Ctx, who: &Who, username: &str) -> Result<Output> {
    if who.is_llm {
        return Err(Error::new(
            Code::NeedsPerson,
            format!("team member remove is for a person; {} is an LLM", who.name),
        ));
    }
    if !term::is_terminal() {
        return Err(Error::new(
            Code::NeedsTerminal,
            "team member remove must be run by a person at a terminal",
        ));
    }
    let Some(index) = ctx
        .cfg
        .members
        .iter()
        .position(|member| same(member, username))
    else {
        return Err(Error::new(
            Code::NotFound,
            format!("{username} is not a member"),
        ));
    };
    let stored = ctx.cfg.members[index].clone();
    let mut members = ctx.cfg.members.clone();
    members.remove(index);
    let output = Output::with(
        format!("removed member {stored}"),
        json!({"member": stored, "members": members}),
    );
    write_members(
        ctx,
        who,
        &members,
        &format!("member {stored} removed"),
        output,
    )
}

/// Writes `members` as the members of the dossier's settings, journals the change with `detail`
/// as who and when (D29), and in a team dossier that is a git repository commits it (D36,
/// R-TEAM-6). The write lock is held until the commit is made, as tx holds it. With --dry-run
/// nothing is written and the output gets the warning `dry run: would write <file>`.
fn write_members(
    ctx: &Ctx,
    who: &Who,
    members: &[String],
    detail: &str,
    output: Output,
) -> Result<Output> {
    if ctx.dry_run {
        let file = ctx.dossier.join(SETTINGS);
        return Ok(output.warn(format!("dry run: would write {}", file.display())));
    }
    let _guard = lock::acquire(&ctx.state, ctx.cfg.lock_wait_seconds, &who.name, "team")?;
    config::set_list(&ctx.dossier, "members", members)?;
    record_change(ctx, who, detail)?;
    Ok(commit_change(ctx, output, &format!("team: {detail}")))
}

/// A username is one word: not empty, and without a space or any other whitespace, so that it
/// stays one word in the settings and the journal (format.md §21). Code::BadValue otherwise.
fn check_username(username: &str) -> Result<()> {
    if username.is_empty() || username.chars().any(char::is_whitespace) {
        return Err(Error::new(
            Code::BadValue,
            "a username must not be empty or contain a space",
        ));
    }
    Ok(())
}

/// Two usernames are the same when they differ in case only, as the settings compare them
/// (format.md §21).
fn same(a: &str, b: &str) -> bool {
    a.to_lowercase() == b.to_lowercase()
}

/// Commits the change of a team dossier that is a git repository, with `subject` as the message
/// (D36, R-TEAM-6). Other dossiers are not committed. A failure is a warning on `output`, never
/// an error: the change is written already.
fn commit_change(ctx: &Ctx, output: Output, subject: &str) -> Output {
    if ctx.cfg.kind != TEAM {
        return output;
    }
    match gitcommit::commit(&ctx.root, &ctx.dossier, subject) {
        Ok(_) => output,
        Err(error) => output.warn(format!("git: {}", error.message)),
    }
}

/// Appends the journal line of a settings change: the settings event `changed`, with who and when
/// (D29) and `detail` as its detail. The journal is the one of the dossier in `ctx`.
fn record_change(ctx: &Ctx, who: &Who, detail: &str) -> Result<()> {
    let record = journal::Record {
        time: time::now(),
        name: who.name.clone(),
        person: who.person.clone(),
        llm: who.is_llm,
        command: "team".to_string(),
        id: SETTINGS_ID.to_string(),
        event: "changed".to_string(),
        detail: detail.to_string(),
    };
    journal::append(&ctx.state, &record)
}
