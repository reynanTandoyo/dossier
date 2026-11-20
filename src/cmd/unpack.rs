//! The `unpack` command.

use crate::error::{Code, Error, Result};
use crate::output::Output;

/// The arguments of `unpack`.
#[derive(clap::Args, Debug)]
pub struct Args {}

/// Runs `unpack`. Not built yet.
pub fn run(_args: &Args, _g: &crate::cli::Global) -> Result<Output> {
    Err(Error::new(Code::NotImplemented, "unpack is not built yet"))
}
