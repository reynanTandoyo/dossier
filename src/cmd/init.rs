//! The `init` command: creates the project dossier `DOSSIER` in the current folder (format.md
//! §9, §10), or with `--team-dossier` the team dossier (format.md §20). With `--with-team <path>`
//! the project is also linked to that team dossier and its code is registered in the team's
//! `projects.md` (R-PLACE-2, R-PLACE-4). No dossier is needed, because there is none yet. The
//! creation is recorded in the journal with who and when (D29).

use crate::atomic;
use crate::blockfile::BlockFile;
use crate::cli::Global;
use crate::cmd::team;
use crate::config;
use crate::ctx::Ctx;
use crate::entry::Kind;
use crate::error::{Code, Error, Result};
use crate::journal;
use crate::names;
use crate::output::Output;
use crate::paths;
use crate::teamfiles;
use crate::time;
use crate::tx::{self, Change};
use serde_json::json;
use std::path::{Component, Path, PathBuf};

/// The name of the project dossier folder (R-PLACE-1).
const FOLDER: &str = "DOSSIER";

/// The state folder inside the dossier (format.md §9).
const STATE: &str = ".state";

/// The settings file inside the dossier (format.md §10).
const SETTINGS: &str = "dossier.toml";

/// The journal file inside the state folder (format.md §11).
const JOURNAL: &str = "journal.jsonl";

/// The folders of a team dossier (format.md §20, R-TEAM-1): no decisions, assumptions, findings or
/// usage records.
const TEAM_FOLDERS: [&str; 3] = ["lessons", "howtos", "tools"];

/// The ignore file of the state folder of a team dossier (format.md §20): the lock and the holder
/// file are not shared through git.
const IGNORE: &str = ".gitignore";

/// The text of the ignore file of a team dossier.
const IGNORE_TEXT: &str = "write.lock\nholder.json\n";

/// The mode of each created folder on Linux: group-writable, with the setgid bit, so that new
/// folders and files keep the group of the dossier folder (R-XOS-8).
const FOLDER_MODE: u32 = 0o2775;

/// The mode of dossier.toml and the journal on Linux: readable and writable by the group
/// (R-XOS-8).
const FILE_MODE: u32 = 0o664;

/// The line that says which group can write the files. Written on Linux only.
#[cfg(unix)]
const GROUP_LINE: &str = "\ngroup: files are writable by the group of this folder";
#[cfg(not(unix))]
const GROUP_LINE: &str = "";

/// The kind of a team dossier (format.md §26).
const TEAM: &str = "team";

/// The file of the team dossier that lists the project codes and their folders (format.md §20).
const PROJECTS_FILE: &str = "projects.md";

/// The item of projects.md in the journal and the snapshots: the whole file (format.md §17).
const PROJECTS_ITEM: &str = "PROJECTS";

/// The event of a registration: in the journal, and as the history line of the code's block.
const REGISTERED: &str = "registered";

/// The field of a code's block that names its project folder (format.md §20).
const FOLDER_FIELD: &str = "folder";

/// The arguments of `init`.
#[derive(clap::Args, Debug)]
pub struct Args {
    /// The project code, such as P-017: letters, digits and `-`
    #[arg(long)]
    pub code: String,
    /// The project is a code project: its dossier holds the map and code pointers (R-CODE-5)
    #[arg(long)]
    pub code_project: bool,
    /// Makes a team dossier instead of a project dossier: lessons, how-tos and tools (R-PLACE-3)
    #[arg(long)]
    pub team_dossier: bool,
    /// Links the project to the team dossier at this path (its DOSSIER folder, or the folder
    /// that holds it) and registers the code there (R-PLACE-2, R-PLACE-4)
    #[arg(long = "with-team")]
    pub with_team: Option<String>,
}

/// The team side of `init --with-team`, checked before anything is created.
struct TeamPlan {
    /// The team's DOSSIER folder, absolute.
    folder: PathBuf,
    /// The team dossier's settings, and who acts there (Ctx::load with the team's folder).
    team: Ctx,
    /// The team's folder as dossier.toml stores it: absolute, with / separators.
    stored_team: String,
    /// The project folder as the team's register stores it.
    project: String,
    /// Whether the code is registered now; false when it is registered for this folder already.
    add: bool,
}

/// Runs `init`: creates DOSSIER in the current folder with its folders, its settings and the
/// journal line that records who created it. With `--with-team` the team is checked first, before
/// anything is created; the project is then linked to the team dossier and its code registered
/// there. If that fails, the new dossier is removed again.
pub fn run(args: &Args, g: &Global) -> Result<Output> {
    if args.team_dossier && args.code_project {
        return Err(Error::new(
            Code::BadValue,
            "a team dossier is not a code project: drop --code-project or --team-dossier",
        ));
    }
    if args.team_dossier && args.with_team.is_some() {
        return Err(Error::new(
            Code::BadValue,
            "a team dossier is not linked to a team: drop --with-team or --team-dossier",
        ));
    }
    let root = std::env::current_dir()
        .map_err(|error| Error::new(Code::Io, format!("the current folder: {error}")))?;
    let dossier = root.join(FOLDER);
    if dossier.symlink_metadata().is_ok() {
        return Err(exists());
    }
    if !valid_code(&args.code) {
        return Err(Error::new(
            Code::BadValue,
            "a project code has only letters, digits and -, such as P-017",
        ));
    }
    // Checked before anything is created, so that a refused team or code leaves no folder behind.
    let team_plan = match &args.with_team {
        Some(path) => Some(plan_team(&root, &args.code, path, g)?),
        None => None,
    };
    if let Some(plan) = &team_plan
        && g.dry_run
    {
        return Ok(dry_run(&dossier, args, plan));
    }
    let who = names::unchecked(g.agent.as_deref());
    let detail = if args.team_dossier {
        format!("team dossier, code {}", args.code)
    } else {
        format!("code {}", args.code)
    };
    let record = journal::Record {
        time: time::now(),
        name: who.name,
        person: who.person,
        llm: who.is_llm,
        command: "init".to_string(),
        id: FOLDER.to_string(),
        event: "created".to_string(),
        detail,
    };
    create(&dossier, args, &record)?;
    let warnings = match &team_plan {
        Some(plan) => match join_team(&dossier, &args.code, g, plan) {
            Ok(warnings) => warnings,
            Err(error) => {
                // This command made the folder a moment ago, so it is removed again.
                let _ = std::fs::remove_dir_all(&dossier);
                return Err(error);
            }
        },
        None => Vec::new(),
    };
    let text = if args.team_dossier {
        format!("created the team dossier {}{GROUP_LINE}", args.code)
    } else {
        let code_line = if args.code_project {
            "\ncode project: map and code pointers are on"
        } else {
            ""
        };
        let team_line = match &team_plan {
            Some(plan) => format!("\n{}", team_text(&args.code, plan)),
            None => String::new(),
        };
        format!(
            "created DOSSIER for {}{code_line}{team_line}\nnext: dossier people add <person> --llm <name>{GROUP_LINE}",
            args.code
        )
    };
    let data = json!({
        "code": args.code,
        "dossier": dossier.display().to_string(),
        "team": team_plan.as_ref().map(|plan| plan.stored_team.clone()),
        "registered": team_plan.as_ref().map(|plan| plan.add),
    });
    let mut output = Output::with(text, data);
    for warning in warnings {
        output = output.warn(warning);
    }
    Ok(output)
}

/// Checks `--with-team <path>` before anything is created: the team dossier must be one, the code
/// must not be registered for another folder (R-PLACE-4), and the name that acts must be able to
/// write to the team when the code has to be registered (R-TEAM-7, R-CFG-4).
fn plan_team(root: &Path, code: &str, path: &str, g: &Global) -> Result<TeamPlan> {
    let folder = team_folder(Path::new(path))?;
    let team_global = Global {
        dossier: Some(folder.clone()),
        team: false,
        ..g.clone()
    };
    let team = Ctx::load(&team_global, "init")?;
    let project = stored(root)?;
    let (file, _) = teamfiles::projects(&team.dossier, &team.cfg.code)?;
    let add = needs_registration(&file, code, &project)?;
    if add {
        names::resolve(&team.cfg, g.agent.as_deref())?;
    }
    let stored_team = stored(&folder)?;
    Ok(TeamPlan {
        folder,
        team,
        stored_team,
        project,
        add,
    })
}

/// Links the project to the team dossier with the link of `team use` (R-PLACE-2), then registers
/// the code there when it is not registered yet. Returns the warnings of the team's write.
fn join_team(dossier: &Path, code: &str, g: &Global, plan: &TeamPlan) -> Result<Vec<String>> {
    let link = Global {
        dossier: Some(dossier.to_path_buf()),
        team: false,
        ..g.clone()
    };
    team::run(
        &team::Args {
            action: Some(team::Action::Use {
                path: plan.folder.clone(),
            }),
        },
        &link,
    )?;
    if plan.add {
        register(&plan.team, code, &plan.project)
    } else {
        Ok(Vec::new())
    }
}

/// Adds the block of `code` with its project folder and the history line `registered` to the
/// team's projects.md, in one transaction on the team dossier (R-PLACE-4). Returns the warnings of
/// that transaction, such as a git commit that failed.
fn register(team: &Ctx, code: &str, project: &str) -> Result<Vec<String>> {
    let output = tx::write(team, |tx| {
        let who = tx.ctx().writer()?.clone();
        let (mut file, old) = teamfiles::projects(&team.dossier, &team.cfg.code)?;
        if needs_registration(&file, code, project)? {
            let fields = [(FOLDER_FIELD.to_string(), project.to_string())];
            file.add_block(code, &fields);
            file.add_history(code, tx.time(), &who, REGISTERED, "");
            tx.put_raw(
                team.dossier.join(PROJECTS_FILE),
                old,
                file.text().into_bytes(),
                Change {
                    id: PROJECTS_ITEM.to_string(),
                    event: REGISTERED.to_string(),
                    detail: code.to_string(),
                },
            );
        }
        Ok(Output::text(format!(
            "registered {code} in the team dossier"
        )))
    })?;
    Ok(output.warnings)
}

/// Whether `code` must be registered for `project` in the team's projects.md: false when it is
/// registered for that folder already. A code registered for another folder is Code::Exists.
fn needs_registration(file: &BlockFile, code: &str, project: &str) -> Result<bool> {
    if !file.keys().iter().any(|key| key == code) {
        return Ok(true);
    }
    match file.field(code, FOLDER_FIELD) {
        Some(folder) if same_folder(&folder, project) => Ok(false),
        Some(folder) => Err(Error::new(
            Code::Exists,
            format!("{code} is already registered for {folder}"),
        )),
        None => Err(Error::new(
            Code::Exists,
            format!("{code} is already registered"),
        )),
    }
}

/// Whether two stored folders are the same folder. Folder names on Windows ignore case.
fn same_folder(a: &str, b: &str) -> bool {
    if cfg!(windows) {
        a.eq_ignore_ascii_case(b)
    } else {
        a == b
    }
}

/// The DOSSIER folder of the team dossier that `path` names, as `team use` takes it: `path` when it
/// holds dossier.toml, else `path`/DOSSIER. Code::NoDossier when neither holds one, Code::BadValue
/// when its settings are not those of a team dossier (format.md §26).
fn team_folder(path: &Path) -> Result<PathBuf> {
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

/// A folder as dossier.toml and the team's register store it: absolute, with / separators on
/// Windows (format.md §26). A path that is not text is Code::BadValue.
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

/// The two lines of `init --with-team` about the team dossier: the link, and the registration, or
/// that the code is registered for this folder already.
fn team_text(code: &str, plan: &TeamPlan) -> String {
    let registration = if plan.add {
        format!("registered {code} in the team dossier")
    } else {
        format!("{code} is registered in the team dossier already")
    };
    format!(
        "linked to the team dossier {}\n{registration}",
        paths::shown(&plan.stored_team)
    )
}

/// What `init --with-team --dry-run` would do, as warnings: nothing is created or written.
fn dry_run(dossier: &Path, args: &Args, plan: &TeamPlan) -> Output {
    let text = format!(
        "dry run: DOSSIER for {} would be linked to the team dossier {}",
        args.code,
        paths::shown(&plan.stored_team)
    );
    let mut output =
        Output::text(text).warn(format!("dry run: would create {}", dossier.display()));
    if plan.add {
        let file = plan.folder.join(PROJECTS_FILE);
        output = output.warn(format!("dry run: would write {}", file.display()));
    }
    output
}

/// Whether `code` is a project code: not empty, and only ASCII letters, digits and `-`.
fn valid_code(code: &str) -> bool {
    !code.is_empty() && code.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
}

/// The failure for a DOSSIER that is already there.
fn exists() -> Error {
    Error::new(Code::Exists, "DOSSIER already exists here")
}

/// Creates the dossier folder, its folders, its settings file and its first journal line. When
/// a step after the first one fails, the dossier folder is removed again, so that a second
/// `init` can run.
fn create(dossier: &Path, args: &Args, record: &journal::Record) -> Result<()> {
    match std::fs::create_dir(dossier) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => return Err(exists()),
        Err(error) => return Err(io_error(dossier, error)),
    }
    let built = build(dossier, args, record);
    if built.is_err() {
        // This command made the folder a moment ago, so only its own parts are removed.
        let _ = std::fs::remove_dir_all(dossier);
    }
    built
}

/// Sets the mode of the dossier folder, makes the folders and the state folder, writes
/// dossier.toml with the code, and appends the journal line that records the creation. A team
/// dossier's state folder also gets its ignore file.
fn build(dossier: &Path, args: &Args, record: &journal::Record) -> Result<()> {
    set_mode(dossier, FOLDER_MODE)?;
    for folder in folders(args.team_dossier) {
        make_dir(&dossier.join(folder))?;
    }
    let state = dossier.join(STATE);
    make_dir(&state)?;
    if args.team_dossier {
        let ignore = state.join(IGNORE);
        atomic::replace(&ignore, IGNORE_TEXT.as_bytes(), Some(&None))?;
        set_mode(&ignore, FILE_MODE)?;
    }
    let settings = dossier.join(SETTINGS);
    atomic::replace(
        &settings,
        settings_text(&args.code, settings_line(args)).as_bytes(),
        Some(&None),
    )?;
    set_mode(&settings, FILE_MODE)?;
    journal::append(&state, record)?;
    set_mode(&state.join(JOURNAL), FILE_MODE)
}

/// The folders made inside the dossier folder: the team's three, or the kind folders of a
/// project (format.md §9, §20).
fn folders(team_dossier: bool) -> Vec<&'static str> {
    if team_dossier {
        TEAM_FOLDERS.to_vec()
    } else {
        Kind::all().into_iter().map(Kind::folder).collect()
    }
}

/// The setting that follows the code line of dossier.toml: `kind = "team"` (format.md §26) for a
/// team dossier, `code_project = true` (format.md §19) for a code project, else none.
fn settings_line(args: &Args) -> Option<&'static str> {
    if args.team_dossier {
        Some("kind = \"team\"")
    } else if args.code_project {
        Some("code_project = true")
    } else {
        None
    }
}

/// The text of dossier.toml for `code` (format.md §10). With `line`, that line follows the code
/// line.
fn settings_text(code: &str, line: Option<&str>) -> String {
    let text = config::default_text(code);
    let Some(line) = line else {
        return text;
    };
    let mut out = String::new();
    for text_line in text.split_inclusive('\n') {
        out.push_str(text_line);
        if text_line.starts_with("code = ") {
            out.push_str(line);
            out.push('\n');
        }
    }
    out
}

/// Creates a folder inside the dossier folder and sets its mode.
fn make_dir(path: &Path) -> Result<()> {
    std::fs::create_dir(path).map_err(|error| io_error(path, error))?;
    set_mode(path, FOLDER_MODE)
}

/// Sets the Linux mode of a file or folder this command created.
#[cfg(unix)]
fn set_mode(path: &Path, mode: u32) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(mode))
        .map_err(|error| io_error(path, error))
}

/// Windows has no such modes: nothing is set.
#[cfg(not(unix))]
fn set_mode(_path: &Path, _mode: u32) -> Result<()> {
    Ok(())
}

/// A failure on `path` as Code::Io, with the path in the message.
fn io_error(path: &Path, error: std::io::Error) -> Error {
    Error::new(Code::Io, format!("{}: {error}", path.display()))
}
