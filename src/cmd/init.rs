//! The `init` command: creates the project dossier `DOSSIER` in the current folder (format.md
//! §9, §10). No name and no dossier are needed, because there is none yet.

use crate::atomic;
use crate::cli::Global;
use crate::config;
use crate::entry::Kind;
use crate::error::{Code, Error, Result};
use crate::output::Output;
use serde_json::json;
use std::path::Path;

/// The name of the project dossier folder (R-PLACE-1).
const FOLDER: &str = "DOSSIER";

/// The state folder inside the dossier (format.md §9).
const STATE: &str = ".state";

/// The settings file inside the dossier (format.md §10).
const SETTINGS: &str = "dossier.toml";

/// The mode of each created folder on Linux: group-writable, with the setgid bit, so that new
/// folders and files keep the group of the dossier folder (R-XOS-8).
const FOLDER_MODE: u32 = 0o2775;

/// The mode of dossier.toml on Linux: readable and writable by the group (R-XOS-8).
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
}

/// Runs `init`: creates DOSSIER in the current folder with its folders and settings.
pub fn run(args: &Args, _g: &Global) -> Result<Output> {
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
    create(&dossier, &args.code)?;
    let text = format!(
        "created DOSSIER for {}\nnext: dossier people add <person> --llm <name>{GROUP_LINE}",
        args.code
    );
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

/// Creates the dossier folder, its folders and its settings file. When a step after the first
/// one fails, the dossier folder is removed again, so that a second `init` can run.
fn create(dossier: &Path, code: &str) -> Result<()> {
    match std::fs::create_dir(dossier) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => return Err(exists()),
        Err(error) => return Err(io_error(dossier, error)),
    }
    let built = build(dossier, code);
    if built.is_err() {
        // This command made the folder a moment ago, so only its own parts are removed.
        let _ = std::fs::remove_dir_all(dossier);
    }
    built
}

/// Sets the mode of the dossier folder, makes the kind folders and the state folder, and writes
/// dossier.toml with the code.
fn build(dossier: &Path, code: &str) -> Result<()> {
    set_mode(dossier, FOLDER_MODE)?;
    for kind in Kind::all() {
        make_dir(&dossier.join(kind.folder()))?;
    }
    make_dir(&dossier.join(STATE))?;
    let settings = dossier.join(SETTINGS);
    atomic::replace(
        &settings,
        config::default_text(code).as_bytes(),
        Some(&None),
    )?;
    set_mode(&settings, FILE_MODE)
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
