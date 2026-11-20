//! The `edit` command.

use crate::error::{Code, Error, Result};
use crate::output::Output;

/// The arguments of `edit`.
#[derive(clap::Args, Debug)]
pub struct Args {}

/// Runs `edit`. Not built yet.
pub fn run(_args: &Args, _g: &crate::cli::Global) -> Result<Output> {
    Err(Error::new(Code::NotImplemented, "edit is not built yet"))
}
