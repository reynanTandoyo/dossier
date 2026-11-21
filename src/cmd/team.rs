//! The `team` command (docs/design.md section 12): not built yet.

use crate::cli::Global;
use crate::error::{Code, Error, Result};
use crate::output::Output;

/// The arguments of `team`.
#[derive(clap::Args, Debug)]
pub struct Args {}

/// Runs `team`.
pub fn run(_args: &Args, _g: &Global) -> Result<Output> {
    Err(Error::new(Code::NotImplemented, "team is not built yet"))
}
