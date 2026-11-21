//! The `unpack` command: restores an archive written by `pack` into `<folder>/DOSSIER`
//! (R-BACKUP-2). It needs no dossier and no name, because it makes one, so it does not load a
//! context.

use crate::cli::Global;
use crate::error::Result;
use crate::output::Output;
use crate::tar;
use serde_json::json;
use std::path::PathBuf;

/// The arguments of `unpack`.
#[derive(clap::Args, Debug)]
pub struct Args {
    /// The archive written by `pack`
    pub file: PathBuf,
    /// The folder that receives DOSSIER; DOSSIER inside it must not exist yet
    #[arg(long)]
    pub into: PathBuf,
}

/// Runs `unpack`: restores the archive into `<into>/DOSSIER`. The second line of the text says
/// how to check the restored dossier.
pub fn run(args: &Args, _g: &Global) -> Result<Output> {
    let dossier = args.into.join("DOSSIER");
    let count = tar::unpack(&args.file, &dossier)?;
    let text = format!(
        "unpacked {count} files into {}/DOSSIER\ncheck it with: dossier check",
        args.into.display()
    );
    let data = json!({"files": count, "dossier": dossier.display().to_string()});
    Ok(Output::with(text, data))
}
