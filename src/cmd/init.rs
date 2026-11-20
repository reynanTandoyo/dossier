//! The `init` command.

use crate::error::{Code, Error, Result};
use crate::output::Output;

/// The arguments of `init`.
#[derive(clap::Args, Debug)]
pub struct Args {}

/// Runs `init`. Not built yet.
pub fn run(_args: &Args, _g: &crate::cli::Global) -> Result<Output> {
    Err(Error::new(Code::NotImplemented, "init is not built yet"))
}
