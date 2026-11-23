//! The `unpack` command: restores an archive written by `pack` into `<folder>/DOSSIER`, then runs
//! `check` on the restored dossier (R-BACKUP-2). It needs no dossier and no name, because it makes
//! one, so it does not load a context.

use crate::cli::Global;
use crate::cmd::check;
use crate::error::{Code, Error, Result};
use crate::output::Output;
use crate::tar;
use serde_json::json;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

/// The name of the dossier folder inside `<folder>`.
const FOLDER: &str = "DOSSIER";

/// The state folder inside the dossier (format.md §9).
const STATE: &str = ".state";

/// The lock note of the `pack` run that wrote the archive. It belongs to that run, not to the
/// restored dossier, so it is removed after the restore.
const HOLDER: &str = "holder.json";

/// The arguments of `unpack`.
#[derive(clap::Args, Debug)]
pub struct Args {
    /// The archive written by `pack`
    pub file: PathBuf,
    /// The folder that receives DOSSIER; DOSSIER inside it must not exist yet
    #[arg(long)]
    pub into: PathBuf,
}

/// Runs `unpack`: restores the archive into `<into>/DOSSIER`, removes the pack run's lock note,
/// and runs `check` on the result. The text is the line with the file count, then the check
/// text. When `check` finds problems the command fails with its code, and the message says the
/// files were restored.
pub fn run(args: &Args, _g: &Global) -> Result<Output> {
    let dossier = args.into.join(FOLDER);
    let count = tar::unpack(&args.file, &dossier)?;
    remove_holder(&dossier)?;

    let absolute = std::path::absolute(&dossier)
        .map_err(|error| Error::new(Code::Io, format!("{}: {error}", dossier.display())))?;
    let global = Global {
        dossier: Some(absolute),
        ..Default::default()
    };
    let heading = format!(
        "unpacked {count} files into {}/{FOLDER}",
        args.into.display()
    );
    match check::run(&check::Args { fix: false }, &global) {
        Ok(checked) => {
            let text = format!("{heading}\n{}", checked.text);
            let data = json!({
                "files": count,
                "dossier": dossier.display().to_string(),
                "check": checked.text,
            });
            let output = checked
                .warnings
                .into_iter()
                .fold(Output::with(text, data), |output, warning| {
                    output.warn(warning)
                });
            Ok(output)
        }
        Err(error) => Err(Error::new(
            error.code,
            format!("{heading}, but check found problems:\n{}", error.message),
        )),
    }
}

/// Removes `<dossier>/.state/holder.json` when the archive had it. A missing file is not an
/// error; any other failure is Code::Io with the path.
fn remove_holder(dossier: &Path) -> Result<()> {
    let path = dossier.join(STATE).join(HOLDER);
    match std::fs::remove_file(&path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(()),
        Err(error) => Err(Error::new(Code::Io, format!("{}: {error}", path.display()))),
    }
}
