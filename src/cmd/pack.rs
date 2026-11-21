//! The `pack` command: writes the whole dossier, its state folder included, into one archive
//! (R-BACKUP-1, D24). It runs under the write lock, so the archive is one consistent state of
//! the dossier. The files that entries point to are not part of it.

use crate::cli::Global;
use crate::ctx::Ctx;
use crate::error::{Code, Error, Result};
use crate::lock;
use crate::output::Output;
use crate::tar;
use serde_json::json;
use std::path::PathBuf;

/// The arguments of `pack`.
#[derive(clap::Args, Debug)]
pub struct Args {
    /// The archive to write, relative to the current folder; it must not exist yet
    #[arg(long)]
    pub to: PathBuf,
}

/// Runs `pack`: writes the dossier into the archive `--to`. The file must not exist
/// (Code::Exists). The lock is held until the archive is written.
pub fn run(args: &Args, g: &Global) -> Result<Output> {
    let ctx = Ctx::load(g, "pack")?;
    let to = &args.to;
    if to.symlink_metadata().is_ok() {
        return Err(Error::new(
            Code::Exists,
            format!("{} already exists", to.display()),
        ));
    }
    let name = match &ctx.who {
        Some(who) => who.name.as_str(),
        None => "pack",
    };
    let _guard = lock::acquire(&ctx.state, ctx.cfg.lock_wait_seconds, name, "pack")?;
    let count = tar::pack(&ctx.dossier, to)?;
    let text = format!("packed {count} files into {}", to.display());
    let data = json!({"files": count, "archive": to.display().to_string()});
    Ok(Output::with(text, data))
}
