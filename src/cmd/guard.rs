//! The `guard` command (docs/design.md section 9): blocks edits inside a dossier folder, from a PreToolUse payload. Not built yet.

use crate::cli::Global;
use crate::error::{Code, Error, Result};
use crate::output::Output;

/// The arguments of `guard`.
#[derive(clap::Args, Debug)]
pub struct Args {}

/// Runs `guard`. Not built yet.
pub fn run(_args: &Args, _g: &Global) -> Result<Output> {
    Err(Error::new(Code::NotImplemented, "guard is not built yet"))
}
