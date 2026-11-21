//! The `skill` command (docs/design.md section 9): writes the Claude Code skill file. Not built yet.

use crate::cli::Global;
use crate::error::{Code, Error, Result};
use crate::output::Output;

/// The arguments of `skill`.
#[derive(clap::Args, Debug)]
pub struct Args {}

/// Runs `skill`. Not built yet.
pub fn run(_args: &Args, _g: &Global) -> Result<Output> {
    Err(Error::new(Code::NotImplemented, "skill is not built yet"))
}
