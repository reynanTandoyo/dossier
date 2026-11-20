//! The `log` command.

use crate::error::{Code, Error, Result};
use crate::output::Output;

/// The arguments of `log`.
#[derive(clap::Args, Debug)]
pub struct Args {}

/// Runs `log`. Not built yet.
pub fn run(_args: &Args, _g: &crate::cli::Global) -> Result<Output> {
    Err(Error::new(Code::NotImplemented, "log is not built yet"))
}
