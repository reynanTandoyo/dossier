//! The `brief` command.

use crate::error::{Code, Error, Result};
use crate::output::Output;

/// The arguments of `brief`.
#[derive(clap::Args, Debug)]
pub struct Args {}

/// Runs `brief`. Not built yet.
pub fn run(_args: &Args, _g: &crate::cli::Global) -> Result<Output> {
    Err(Error::new(Code::NotImplemented, "brief is not built yet"))
}
