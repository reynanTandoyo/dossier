//! The `status` command.

use crate::error::{Code, Error, Result};
use crate::output::Output;

/// The arguments of `status`.
#[derive(clap::Args, Debug)]
pub struct Args {}

/// Runs `status`. Not built yet.
pub fn run(_args: &Args, _g: &crate::cli::Global) -> Result<Output> {
    Err(Error::new(Code::NotImplemented, "status is not built yet"))
}
