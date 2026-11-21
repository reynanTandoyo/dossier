//! The `received` command (docs/design.md section 9): an RCV record of files received. Not built yet.

use crate::cli::Global;
use crate::error::{Code, Error, Result};
use crate::output::Output;

/// The arguments of `received`.
#[derive(clap::Args, Debug)]
pub struct Args {}

/// Runs `received`. Not built yet.
pub fn run(_args: &Args, _g: &Global) -> Result<Output> {
    Err(Error::new(
        Code::NotImplemented,
        "received is not built yet",
    ))
}
