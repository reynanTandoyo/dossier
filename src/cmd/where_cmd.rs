//! The `where` command.

use crate::error::{Code, Error, Result};
use crate::output::Output;

/// The arguments of `where`.
#[derive(clap::Args, Debug)]
pub struct Args {}

/// Runs `where`. Not built yet.
pub fn run(_args: &Args, _g: &crate::cli::Global) -> Result<Output> {
    Err(Error::new(Code::NotImplemented, "where is not built yet"))
}
