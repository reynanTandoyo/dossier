//! The `link` command.

use crate::error::{Code, Error, Result};
use crate::output::Output;

/// The arguments of `link`.
#[derive(clap::Args, Debug)]
pub struct Args {}

/// Runs `link`. Not built yet.
pub fn run(_args: &Args, _g: &crate::cli::Global) -> Result<Output> {
    Err(Error::new(Code::NotImplemented, "link is not built yet"))
}
