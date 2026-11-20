//! The `add` command.

use crate::error::{Code, Error, Result};
use crate::output::Output;

/// The arguments of `add`.
#[derive(clap::Args, Debug)]
pub struct Args {}

/// Runs `add`. Not built yet.
pub fn run(_args: &Args, _g: &crate::cli::Global) -> Result<Output> {
    Err(Error::new(Code::NotImplemented, "add is not built yet"))
}
