//! The `decide` command.

use crate::error::{Code, Error, Result};
use crate::output::Output;

/// The arguments of `decide`.
#[derive(clap::Args, Debug)]
pub struct Args {}

/// Runs `decide`. Not built yet.
pub fn run(_args: &Args, _g: &crate::cli::Global) -> Result<Output> {
    Err(Error::new(Code::NotImplemented, "decide is not built yet"))
}
