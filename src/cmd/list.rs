//! The `list` command.

use crate::error::{Code, Error, Result};
use crate::output::Output;

/// The arguments of `list`.
#[derive(clap::Args, Debug)]
pub struct Args {}

/// Runs `list`. Not built yet.
pub fn run(_args: &Args, _g: &crate::cli::Global) -> Result<Output> {
    Err(Error::new(Code::NotImplemented, "list is not built yet"))
}
