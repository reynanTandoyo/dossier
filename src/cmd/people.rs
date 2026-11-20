//! The `people` command.

use crate::error::{Code, Error, Result};
use crate::output::Output;

/// The arguments of `people`.
#[derive(clap::Args, Debug)]
pub struct Args {}

/// Runs `people`. Not built yet.
pub fn run(_args: &Args, _g: &crate::cli::Global) -> Result<Output> {
    Err(Error::new(Code::NotImplemented, "people is not built yet"))
}
