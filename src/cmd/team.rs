//! The `team` command (docs/design.md section 12): shows the team settings of the current dossier,
//! `team use <path>` links a project to its team dossier (R-PLACE-2, format.md §26), `team
//! member add|remove` keeps the list of people who may write there (R-TEAM-7), and `team name
//! add|remove` keeps the plain names of people who are not users, such as a customer contact (D34).
//! The team settings are the members and plain names of dossier.toml (format.md §21); a project with
//! a reachable team dossier also knows the team's members and names, which `Ctx::load` adds. Linking
//! and every member or name change rewrite dossier.toml, so each is recorded in the journal with who
//! and when (D29), as `people` is.
//!
//! `team summary` shows the team dossier at a glance (R-TEAM-5): the entries per kind and status,
//! the drafts waiting, the entries due for review and the newest approved entries. It reads only.

use crate::cli::Global;
use crate::config;
use crate::ctx::Ctx;
use crate::entry::{Entry, Kind, Status};
use crate::error::{Code, Error, Result};
use crate::gitcommit;
use crate::journal;
use crate::lock;
use crate::names::{self, Who};
use crate::output::Output;
use crate::paths;
use crate::store::{self, Item};
use crate::term;
use crate::time;
use chrono::{DateTime, FixedOffset, NaiveDate};
use serde_json::{Value, json};
use std::cmp::Reverse;
use std::path::{Component, Path, PathBuf};

/// The settings file inside a DOSSIER folder (format.md §10).
const SETTINGS: &str = "dossier.toml";

/// The DOSSIER folder name: a team path may name it, or the folder that holds it.
const FOLDER: &str = "DOSSIER";

/// The `kind` of a team dossier (format.md §26).
const TEAM: &str = "team";

/// The settings keys of the team's two lists (format.md §21).
const MEMBERS: &str = "members";
const NAMES: &str = "names";

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
    /// Adds a plain name to the team, or removes one: a person who is not a user (D34)
    Name {
        #[command(subcommand)]
        action: NameAction,
    },
    /// Shows the team dossier: entries per kind and status, drafts waiting, entries due for review
    /// and the newest approved entries (R-TEAM-5)
    Summary,
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

/// What `team name` does. A plain name is a person who is not a user, such as a customer contact:
/// it may be an author or a source, and it never writes (format.md §21).
#[derive(clap::Subcommand, Debug)]
pub enum NameAction {
    /// Adds a plain name; it may contain spaces, such as `Dr. Weber`
    Add {
        /// The name as entries write it: not empty, and without a tab or a line break
        name: String,
    },
    /// Removes a plain name; a person at a terminal does it
    Remove {
        /// The name, in any case
        name: String,
    },
}

/// Runs `team`: shows the team dossier, the members and the plain names, links the project to a
/// team dossier with `team use`, changes the members with `team member` or the plain names with
/// `team name`, or summarises the team dossier with `team summary`.
pub fn run(args: &Args, g: &Global) -> Result<Output> {
    let ctx = Ctx::load(g, "team")?;
    match &args.action {
        None => Ok(show(&ctx)),
        Some(Action::Use { path }) => link(&ctx, g, path),
        Some(Action::Member { action }) => member(settings_ctx(ctx)?, g, action),
        Some(Action::Name { action }) => plain_name(settings_ctx(ctx)?, g, action),
        Some(Action::Summary) => Ok(summary(&summary_ctx(ctx)?)),
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

/// The context the team settings (members and plain names) are changed in (R-TEAM-7): the team
/// dossier of a project that names one, else the current dossier. A team dossier that cannot be
/// reached is Code::NoDossier naming its path (R-PLACE-8): the settings to change are the team's.
fn settings_ctx(ctx: Ctx) -> Result<Ctx> {
    if ctx.cfg.kind == TEAM {
        return Ok(ctx);
    }
    let Some(stored) = ctx.cfg.team.clone() else {
        return Ok(ctx);
    };
    if ctx.team.is_none() {
        return Err(unreachable(&ctx, &stored));
    }
    ctx.team_ctx()
}

/// The context `team summary` reads: the team dossier itself, or the team dossier of a project
/// (R-TEAM-5). A project with no team dossier is refused as `team_ctx` refuses it; one whose team
/// dossier cannot be reached is Code::NoDossier naming its path (R-PLACE-8).
fn summary_ctx(ctx: Ctx) -> Result<Ctx> {
    if ctx.cfg.kind == TEAM {
        return Ok(ctx);
    }
    if ctx.team.is_none()
        && let Some(stored) = ctx.cfg.team.clone()
    {
        return Err(unreachable(&ctx, &stored));
    }
    ctx.team_ctx()
}

/// Code::NoDossier for a team dossier that the project names at `stored` but cannot be reached
/// (R-PLACE-8): the settings to change, or to read, are the team's.
fn unreachable(ctx: &Ctx, stored: &str) -> Error {
    Error::new(
        Code::NoDossier,
        format!(
            "{}: the team dossier cannot be reached",
            ctx.root.join(paths::shown(stored)).display()
        ),
    )
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
    write_list(
        ctx,
        who,
        MEMBERS,
        &members,
        &format!("member {username} added"),
        output,
    )
}

/// Removes `username` from the members. Only a person may do it, at a terminal (R-TEAM-7): an
/// LLM is Code::NeedsPerson, a run without a terminal Code::NeedsTerminal. The member must exist
/// (Code::NotFound, names compared ignoring case). The last member stays: nobody could add a
/// member after it, so it is Code::Refused. With --dry-run nothing is written.
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
    if ctx.cfg.members.len() == 1 {
        return Err(Error::new(
            Code::Refused,
            format!(
                "{stored} is the last member; add another member first: dossier team member add <username>"
            ),
        ));
    }
    let mut members = ctx.cfg.members.clone();
    members.remove(index);
    let output = Output::with(
        format!("removed member {stored}"),
        json!({"member": stored, "members": members}),
    );
    write_list(
        ctx,
        who,
        MEMBERS,
        &members,
        &format!("member {stored} removed"),
        output,
    )
}

/// Runs `team name add` or `team name remove` in `ctx`. As for the members, the name that acts
/// must be a person of the settings or an LLM of one (R-TEAM-7): Code::Usage without any name,
/// Code::UnknownName for anyone else.
fn plain_name(ctx: Ctx, g: &Global, action: &NameAction) -> Result<Output> {
    let who = names::resolve(&ctx.cfg, g.agent.as_deref())?;
    match action {
        NameAction::Add { name } => add_name(&ctx, &who, name),
        NameAction::Remove { name } => remove_name(&ctx, &who, name),
    }
}

/// Adds the plain name `name` to the team's names. The name is checked by `check_name`, and it
/// must not be listed already (Code::Exists, names compared ignoring case). With --dry-run nothing
/// is written.
fn add_name(ctx: &Ctx, who: &Who, name: &str) -> Result<Output> {
    let name = check_name(name)?;
    let mut plain = own_names(ctx)?;
    if plain.iter().any(|known| same(known, &name)) {
        return Err(Error::new(
            Code::Exists,
            format!("{name} is already a plain name"),
        ));
    }
    plain.push(name.clone());
    let output = Output::with(
        format!("added name {name}"),
        json!({"name": name, "names": plain}),
    );
    write_list(
        ctx,
        who,
        NAMES,
        &plain,
        &format!("name {name} added"),
        output,
    )
}

/// Removes the plain name `name` from the team's names. Only a person may do it, at a terminal
/// (R-TEAM-7), as for the members: an LLM is Code::NeedsPerson, a run without a terminal
/// Code::NeedsTerminal. The name must be listed (Code::NotFound, names compared ignoring case).
/// With --dry-run nothing is written.
fn remove_name(ctx: &Ctx, who: &Who, name: &str) -> Result<Output> {
    if who.is_llm {
        return Err(Error::new(
            Code::NeedsPerson,
            format!("team name remove is for a person; {} is an LLM", who.name),
        ));
    }
    if !term::is_terminal() {
        return Err(Error::new(
            Code::NeedsTerminal,
            "team name remove must be run by a person at a terminal",
        ));
    }
    let wanted = name.trim();
    let mut plain = own_names(ctx)?;
    let Some(index) = plain.iter().position(|known| same(known, wanted)) else {
        return Err(Error::new(
            Code::NotFound,
            format!("{wanted} is not a plain name"),
        ));
    };
    let stored = plain.remove(index);
    let output = Output::with(
        format!("removed name {stored}"),
        json!({"name": stored, "names": plain}),
    );
    write_list(
        ctx,
        who,
        NAMES,
        &plain,
        &format!("name {stored} removed"),
        output,
    )
}

/// The plain names as the dossier's own settings list them (format.md §21). The names of a
/// context are not used: a project with a team gets the project's plain names added to them
/// (`Ctx::team_ctx`), and those are not to be copied into the team dossier.
fn own_names(ctx: &Ctx) -> Result<Vec<String>> {
    Ok(config::load(&ctx.dossier)?.names)
}

/// A plain name as it is written: trimmed, not empty, and without a control character such as a
/// tab or a line break, so that it stays one line of the settings and the journal (format.md §21).
/// Code::BadValue otherwise.
fn check_name(name: &str) -> Result<String> {
    let name = name.trim();
    if name.is_empty() || name.chars().any(char::is_control) {
        return Err(Error::new(
            Code::BadValue,
            "a name must not be empty and must not contain a tab or a line break",
        ));
    }
    Ok(name.to_string())
}

/// Writes `values` as the list `key` (`members` or `names`) of the dossier's settings, journals the
/// change with `detail` as who and when (D29), and in a team dossier that is a git repository
/// commits it (D36, R-TEAM-6). The write lock is held until the commit is made, as tx holds it.
/// With --dry-run nothing is written and the output gets the warning `dry run: would write <file>`.
fn write_list(
    ctx: &Ctx,
    who: &Who,
    key: &str,
    values: &[String],
    detail: &str,
    output: Output,
) -> Result<Output> {
    if ctx.dry_run {
        let file = ctx.dossier.join(SETTINGS);
        return Ok(output.warn(format!("dry run: would write {}", file.display())));
    }
    let _guard = lock::acquire(&ctx.state, ctx.cfg.lock_wait_seconds, &who.name, "team")?;
    config::set_list(&ctx.dossier, key, values)?;
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

/// The most lines `team summary` prints (docs/design.md section 20): it is read on a screen.
const SUMMARY_LINES: usize = 40;

/// The most drafts waiting that the text names; a line after them counts the rest.
const DRAFTS_SHOWN: usize = 10;

/// The most entries due for review that the text names.
const DUE_SHOWN: usize = 10;

/// The newest approved entries: the data gives this many, the text as many as fit.
const NEWEST: usize = 10;

/// The entries of one kind by status: approved, drafts, and every other status.
struct Tally {
    kind: Kind,
    approved: usize,
    drafts: usize,
    other: usize,
}

/// Runs `team summary` on the team dossier of `ctx`; nothing is changed. The text starts with the
/// team dossier's path, then each part that has something to tell: `## Entries` (one line per kind
/// that has entries), `## Drafts waiting` (oldest first, with the age), `## Due for review`
/// (approved entries whose review date has passed, the earliest review first) and `## Newest`
/// (approved entries, newest approval first). The text stays within SUMMARY_LINES lines: the
/// newest entries give way first.
fn summary(ctx: &Ctx) -> Output {
    let (items, warnings) = store::all(&ctx.dossier);
    let now = time::now();
    let today = NaiveDate::parse_from_str(time::date(&now), "%Y-%m-%d").ok();
    let kinds = tallies(&items);
    let drafts = waiting_drafts(&items);
    let due = due_entries(&items, today);
    let newest = newest_approved(&items);

    let mut lines = vec![format!("team dossier: {}", ctx.dossier.display())];
    part(
        &mut lines,
        "Entries",
        kinds.iter().map(tally_line).collect(),
    );
    let mut draft_lines: Vec<String> = drafts
        .iter()
        .take(DRAFTS_SHOWN)
        .map(|item| draft_line(item, &now))
        .collect();
    let more = drafts.len().saturating_sub(DRAFTS_SHOWN);
    if more > 0 {
        draft_lines.push(format!("{more} more: dossier drafts --team"));
    }
    part(&mut lines, "Drafts waiting", draft_lines);
    part(
        &mut lines,
        "Due for review",
        due.iter()
            .take(DUE_SHOWN)
            .map(|(item, review)| due_line(item, review))
            .collect(),
    );
    // The heading of the newest entries takes one of the lines that are left.
    let room = SUMMARY_LINES.saturating_sub(lines.len() + 1);
    part(
        &mut lines,
        "Newest",
        newest
            .iter()
            .take(NEWEST.min(room))
            .map(|(item, stamp)| newest_line(item, stamp.as_deref()))
            .collect(),
    );

    let data = json!({
        "counts": kinds.iter().map(tally_data).collect::<Vec<Value>>(),
        "drafts": drafts.iter().map(|item| draft_data(item, &now)).collect::<Vec<Value>>(),
        "due": due.iter().map(|(item, review)| due_data(item, review)).collect::<Vec<Value>>(),
        "newest": newest
            .iter()
            .take(NEWEST)
            .map(|(item, stamp)| newest_data(item, stamp.as_deref()))
            .collect::<Vec<Value>>(),
    });
    let mut output = Output::with(lines.join("\n"), data);
    for warning in warnings {
        output = output.warn(warning);
    }
    output
}

/// Adds `heading` and then `body` to `lines`, unless the body is empty: an empty part is left out.
fn part(lines: &mut Vec<String>, heading: &str, body: Vec<String>) {
    if !body.is_empty() {
        lines.push(format!("## {heading}"));
        lines.extend(body);
    }
}

/// The kinds that have entries, in the order of `Kind::all`, each with its counts by status.
fn tallies(items: &[Item]) -> Vec<Tally> {
    Kind::all()
        .into_iter()
        .filter_map(|kind| {
            let mut tally = Tally {
                kind,
                approved: 0,
                drafts: 0,
                other: 0,
            };
            for item in items.iter().filter(|item| item.entry.kind() == Some(kind)) {
                match item.entry.status() {
                    Some(Status::Approved) => tally.approved += 1,
                    Some(Status::Draft) => tally.drafts += 1,
                    _ => tally.other += 1,
                }
            }
            let total = tally.approved + tally.drafts + tally.other;
            (total > 0).then_some(tally)
        })
        .collect()
}

/// `<kind>: <n> approved, <n> drafts, <n> other`.
fn tally_line(tally: &Tally) -> String {
    format!(
        "{}: {} approved, {} drafts, {} other",
        tally.kind.name(),
        tally.approved,
        tally.drafts,
        tally.other
    )
}

/// The data of a kind: its name and its three counts.
fn tally_data(tally: &Tally) -> Value {
    json!({
        "kind": tally.kind.name(),
        "approved": tally.approved,
        "drafts": tally.drafts,
        "other": tally.other,
    })
}

/// The drafts waiting for approval, oldest `created` first (by the instant); a draft without a
/// creation time comes last.
fn waiting_drafts(items: &[Item]) -> Vec<&Item> {
    let mut drafts: Vec<&Item> = items
        .iter()
        .filter(|item| item.entry.status() == Some(Status::Draft))
        .collect();
    drafts.sort_by_key(|item| {
        let created = stamp_of(&item.entry, "created")
            .as_deref()
            .and_then(instant);
        (created.is_none(), created)
    });
    drafts
}

/// `<ID> · <title> (<n> days)`: the age is from `created` to `now`; without a creation time there
/// is no age.
fn draft_line(item: &Item, now: &str) -> String {
    let entry = &item.entry;
    let head = format!("{} · {}", entry.id(), entry.title());
    match stamp_of(entry, "created") {
        Some(created) => format!("{head} ({})", days_text(time::days_between(&created, now))),
        None => head,
    }
}

/// The data of a draft: its ID, kind, title, creation time and age in days (null when unknown).
fn draft_data(item: &Item, now: &str) -> Value {
    let entry = &item.entry;
    let created = stamp_of(entry, "created");
    let age = created
        .as_deref()
        .map(|stamp| time::days_between(stamp, now));
    json!({
        "id": entry.id(),
        "kind": entry.kind().map(Kind::name),
        "title": entry.title(),
        "created": created,
        "age_days": age,
    })
}

/// `1 day`, or `<n> days`, as the `drafts` command writes an age.
fn days_text(days: i64) -> String {
    if days == 1 {
        "1 day".to_string()
    } else {
        format!("{days} days")
    }
}

/// The approved entries whose `review` date is before `today` (R-LIFE-7), the earliest review
/// first. No entries when `today` is not a date.
fn due_entries(items: &[Item], today: Option<NaiveDate>) -> Vec<(&Item, NaiveDate)> {
    let Some(today) = today else {
        return Vec::new();
    };
    let mut due: Vec<(&Item, NaiveDate)> = items
        .iter()
        .filter(|item| item.entry.status() == Some(Status::Approved))
        .filter_map(|item| {
            review_of(&item.entry)
                .filter(|review| *review < today)
                .map(|review| (item, review))
        })
        .collect();
    due.sort_by_key(|(_, review)| *review);
    due
}

/// The `review` date of an entry (format.md §23), when it is a date of the form YYYY-MM-DD.
fn review_of(entry: &Entry) -> Option<NaiveDate> {
    entry
        .field("review")
        .and_then(|text| NaiveDate::parse_from_str(text.trim(), "%Y-%m-%d").ok())
}

/// `<ID> · <title> (review <date>)`.
fn due_line(item: &Item, review: &NaiveDate) -> String {
    let entry = &item.entry;
    format!("{} · {} (review {review})", entry.id(), entry.title())
}

/// The data of an entry due for review: its ID, kind, title and review date.
fn due_data(item: &Item, review: &NaiveDate) -> Value {
    let entry = &item.entry;
    json!({
        "id": entry.id(),
        "kind": entry.kind().map(Kind::name),
        "title": entry.title(),
        "review": review.to_string(),
    })
}

/// The approved entries with the time each was approved, newest first; an entry without a time
/// comes last, and entries with the same time keep the order of their IDs.
fn newest_approved(items: &[Item]) -> Vec<(&Item, Option<String>)> {
    let mut approved: Vec<(&Item, Option<String>)> = items
        .iter()
        .filter(|item| item.entry.status() == Some(Status::Approved))
        .map(|item| (item, approved_at(&item.entry)))
        .collect();
    approved.sort_by_key(|(_, stamp)| {
        let at = stamp.as_deref().and_then(instant);
        (at.is_none(), Reverse(at))
    });
    approved
}

/// The time an entry was approved: the time before ` by ` in its `approved` field (format.md §5),
/// or else its `created` time.
fn approved_at(entry: &Entry) -> Option<String> {
    entry
        .field("approved")
        .and_then(|value| value.split_whitespace().next().map(str::to_string))
        .filter(|stamp| time::valid(stamp))
        .or_else(|| stamp_of(entry, "created"))
}

/// `<date>  <ID> · <title>`, the date of the approval; `-` when the entry has no time.
fn newest_line(item: &Item, stamp: Option<&str>) -> String {
    let entry = &item.entry;
    format!(
        "{}  {} · {}",
        stamp.map_or("-", time::date),
        entry.id(),
        entry.title()
    )
}

/// The data of an approved entry: its ID, kind, title and the date of its approval (null when
/// unknown).
fn newest_data(item: &Item, stamp: Option<&str>) -> Value {
    let entry = &item.entry;
    json!({
        "id": entry.id(),
        "kind": entry.kind().map(Kind::name),
        "title": entry.title(),
        "date": stamp.map(time::date),
    })
}

/// The value of a time-stamp field of the entry, when it is one (format.md §7).
fn stamp_of(entry: &Entry, key: &str) -> Option<String> {
    entry.field(key).filter(|stamp| time::valid(stamp))
}

/// The instant a time stamp (format.md §7) names, None when it is not one.
fn instant(stamp: &str) -> Option<DateTime<FixedOffset>> {
    DateTime::parse_from_rfc3339(stamp).ok()
}
