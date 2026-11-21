//! The `files` command (docs/design.md section 9): file lists of tasks, with marks. Not built yet.

use crate::cli::Global;
use crate::error::{Code, Error, Result};
use crate::output::Output;

/// The arguments of `files`.
#[derive(clap::Args, Debug)]
pub struct Args {}

/// Runs `files`. Not built yet.
pub fn run(_args: &Args, _g: &Global) -> Result<Output> {
    Err(Error::new(Code::NotImplemented, "files is not built yet"))
}
