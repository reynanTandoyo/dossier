//! The `exchange` command (docs/design.md section 9): the received and delivered records in time order. Not built yet.

use crate::cli::Global;
use crate::error::{Code, Error, Result};
use crate::output::Output;

/// The arguments of `exchange`.
#[derive(clap::Args, Debug)]
pub struct Args {}

/// Runs `exchange`. Not built yet.
pub fn run(_args: &Args, _g: &Global) -> Result<Output> {
    Err(Error::new(
        Code::NotImplemented,
        "exchange is not built yet",
    ))
}
