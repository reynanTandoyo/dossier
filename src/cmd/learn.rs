//! The `learn` command.

use crate::error::{Code, Error, Result};
use crate::output::Output;

/// The arguments of `learn`.
#[derive(clap::Args, Debug)]
pub struct Args {}

/// Runs `learn`. Not built yet.
pub fn run(_args: &Args, _g: &crate::cli::Global) -> Result<Output> {
    Err(Error::new(Code::NotImplemented, "learn is not built yet"))
}
