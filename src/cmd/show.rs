//! The `show` command.

use crate::error::{Code, Error, Result};
use crate::output::Output;

/// The arguments of `show`.
#[derive(clap::Args, Debug)]
pub struct Args {}

/// Runs `show`. Not built yet.
pub fn run(_args: &Args, _g: &crate::cli::Global) -> Result<Output> {
    Err(Error::new(Code::NotImplemented, "show is not built yet"))
}
