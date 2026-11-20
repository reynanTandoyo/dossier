//! The `assume` command.

use crate::error::{Code, Error, Result};
use crate::output::Output;

/// The arguments of `assume`.
#[derive(clap::Args, Debug)]
pub struct Args {}

/// Runs `assume`. Not built yet.
pub fn run(_args: &Args, _g: &crate::cli::Global) -> Result<Output> {
    Err(Error::new(Code::NotImplemented, "assume is not built yet"))
}
