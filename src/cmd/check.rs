//! The `check` command.

use crate::error::{Code, Error, Result};
use crate::output::Output;

/// The arguments of `check`.
#[derive(clap::Args, Debug)]
pub struct Args {}

/// Runs `check`. Not built yet.
pub fn run(_args: &Args, _g: &crate::cli::Global) -> Result<Output> {
    Err(Error::new(Code::NotImplemented, "check is not built yet"))
}
