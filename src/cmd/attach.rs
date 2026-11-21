//! The `attach` command (docs/design.md section 9): attaches or moves files of a task. Not built yet.

use crate::cli::Global;
use crate::error::{Code, Error, Result};
use crate::output::Output;

/// The arguments of `attach`.
#[derive(clap::Args, Debug)]
pub struct Args {}

/// Runs `attach`. Not built yet.
pub fn run(_args: &Args, _g: &Global) -> Result<Output> {
    Err(Error::new(Code::NotImplemented, "attach is not built yet"))
}
