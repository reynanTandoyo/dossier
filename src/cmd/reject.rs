//! The `reject` command.

use crate::error::{Code, Error, Result};
use crate::output::Output;

/// The arguments of `reject`.
#[derive(clap::Args, Debug)]
pub struct Args {}

/// Runs `reject`. Not built yet.
pub fn run(_args: &Args, _g: &crate::cli::Global) -> Result<Output> {
    Err(Error::new(Code::NotImplemented, "reject is not built yet"))
}
