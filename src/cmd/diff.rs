//! The `diff` command.

use crate::error::{Code, Error, Result};
use crate::output::Output;

/// The arguments of `diff`.
#[derive(clap::Args, Debug)]
pub struct Args {}

/// Runs `diff`. Not built yet.
pub fn run(_args: &Args, _g: &crate::cli::Global) -> Result<Output> {
    Err(Error::new(Code::NotImplemented, "diff is not built yet"))
}
