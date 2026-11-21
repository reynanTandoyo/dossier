//! The `runs` command (docs/design.md section 9): usage records, newest first, or a comparison. Not built yet.

use crate::cli::Global;
use crate::error::{Code, Error, Result};
use crate::output::Output;

/// The arguments of `runs`.
#[derive(clap::Args, Debug)]
pub struct Args {}

/// Runs `runs`. Not built yet.
pub fn run(_args: &Args, _g: &Global) -> Result<Output> {
    Err(Error::new(Code::NotImplemented, "runs is not built yet"))
}
