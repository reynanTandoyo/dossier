//! The `restore` command.

use crate::error::{Code, Error, Result};
use crate::output::Output;

/// The arguments of `restore`.
#[derive(clap::Args, Debug)]
pub struct Args {}

/// Runs `restore`. Not built yet.
pub fn run(_args: &Args, _g: &crate::cli::Global) -> Result<Output> {
    Err(Error::new(Code::NotImplemented, "restore is not built yet"))
}
