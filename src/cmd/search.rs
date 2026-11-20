//! The `search` command.

use crate::error::{Code, Error, Result};
use crate::output::Output;

/// The arguments of `search`.
#[derive(clap::Args, Debug)]
pub struct Args {}

/// Runs `search`. Not built yet.
pub fn run(_args: &Args, _g: &crate::cli::Global) -> Result<Output> {
    Err(Error::new(Code::NotImplemented, "search is not built yet"))
}
