//! The `init` command: creates the project dossier `DOSSIER` in the current folder (format.md
//! §9, §10), or with `--team-dossier` the team dossier (format.md §20). No dossier is needed,
//! because there is none yet. The creation is recorded in the journal with who and when (D29).

use crate::atomic;
use crate::cli::Global;
use crate::config;
use crate::entry::Kind;
use crate::error::{Code, Error, Result};
use crate::journal;
use crate::names;
use crate::output::Output;
use crate::time;
use serde_json::json;
use std::path::Path;

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
}

/// Runs `init`: creates DOSSIER in the current folder with its folders, its settings and the
/// journal line that records who created it.
pub fn run(args: &Args, g: &Global) -> Result<Output> {
    if args.team_dossier && args.code_project {
        return Err(Error::new(
            Code::BadValue,
            "a team dossier is not a code project: drop --code-project or --team-dossier",
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
    let text = if args.team_dossier {
        format!("created the team dossier {}{GROUP_LINE}", args.code)
    } else {
        let code_line = if args.code_project {
            "\ncode project: map and code pointers are on"
        } else {
            ""
        };
        format!(
            "created DOSSIER for {}{code_line}\nnext: dossier people add <person> --llm <name>{GROUP_LINE}",
            args.code
        )
    };
    let data = json!({"code": args.code, "dossier": dossier.display().to_string()});
    Ok(Output::with(text, data))
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
