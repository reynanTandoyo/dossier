//! The `retitle` command.

use crate::error::{Code, Error, Result};
use crate::output::Output;

/// The arguments of `retitle`.
#[derive(clap::Args, Debug)]
pub struct Args {}

/// Runs `retitle`. Not built yet.
pub fn run(_args: &Args, _g: &crate::cli::Global) -> Result<Output> {
    Err(Error::new(Code::NotImplemented, "retitle is not built yet"))
}
