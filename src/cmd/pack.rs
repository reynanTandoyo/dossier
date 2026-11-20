//! The `pack` command.

use crate::error::{Code, Error, Result};
use crate::output::Output;

/// The arguments of `pack`.
#[derive(clap::Args, Debug)]
pub struct Args {}

/// Runs `pack`. Not built yet.
pub fn run(_args: &Args, _g: &crate::cli::Global) -> Result<Output> {
    Err(Error::new(Code::NotImplemented, "pack is not built yet"))
}
