//! The `todo` command.

use crate::error::{Code, Error, Result};
use crate::output::Output;

/// The arguments of `todo`.
#[derive(clap::Args, Debug)]
pub struct Args {}

/// Runs `todo`. Not built yet.
pub fn run(_args: &Args, _g: &crate::cli::Global) -> Result<Output> {
    Err(Error::new(Code::NotImplemented, "todo is not built yet"))
}
