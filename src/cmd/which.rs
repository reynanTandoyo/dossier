//! The `which` command (docs/design.md section 9): what a path is made from and made into. Not built yet.

use crate::cli::Global;
use crate::error::{Code, Error, Result};
use crate::output::Output;

/// The arguments of `which`.
#[derive(clap::Args, Debug)]
pub struct Args {}

/// Runs `which`. Not built yet.
pub fn run(_args: &Args, _g: &Global) -> Result<Output> {
    Err(Error::new(Code::NotImplemented, "which is not built yet"))
}
