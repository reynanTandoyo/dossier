//! The `remove` command.

use crate::error::{Code, Error, Result};
use crate::output::Output;

/// The arguments of `remove`.
#[derive(clap::Args, Debug)]
pub struct Args {}

/// Runs `remove`. Not built yet.
pub fn run(_args: &Args, _g: &crate::cli::Global) -> Result<Output> {
    Err(Error::new(Code::NotImplemented, "remove is not built yet"))
}
