//! The `approve` command.

use crate::error::{Code, Error, Result};
use crate::output::Output;

/// The arguments of `approve`.
#[derive(clap::Args, Debug)]
pub struct Args {}

/// Runs `approve`. Not built yet.
pub fn run(_args: &Args, _g: &crate::cli::Global) -> Result<Output> {
    Err(Error::new(Code::NotImplemented, "approve is not built yet"))
}
