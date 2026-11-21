//! The `detach` command (docs/design.md section 9): marks a file of a task detached. Not built yet.

use crate::cli::Global;
use crate::error::{Code, Error, Result};
use crate::output::Output;

/// The arguments of `detach`.
#[derive(clap::Args, Debug)]
pub struct Args {}

/// Runs `detach`. Not built yet.
pub fn run(_args: &Args, _g: &Global) -> Result<Output> {
    Err(Error::new(Code::NotImplemented, "detach is not built yet"))
}
