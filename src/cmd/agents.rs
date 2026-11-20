//! The `agents` command.

use crate::error::{Code, Error, Result};
use crate::output::Output;

/// The arguments of `agents`.
#[derive(clap::Args, Debug)]
pub struct Args {}

/// Runs `agents`. Not built yet.
pub fn run(_args: &Args, _g: &crate::cli::Global) -> Result<Output> {
    Err(Error::new(Code::NotImplemented, "agents is not built yet"))
}
