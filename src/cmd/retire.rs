//! The `retire` command.

use crate::error::{Code, Error, Result};
use crate::output::Output;

/// The arguments of `retire`.
#[derive(clap::Args, Debug)]
pub struct Args {}

/// Runs `retire`. Not built yet.
pub fn run(_args: &Args, _g: &crate::cli::Global) -> Result<Output> {
    Err(Error::new(Code::NotImplemented, "retire is not built yet"))
}
