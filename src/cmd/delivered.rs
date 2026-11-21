//! The `delivered` command (docs/design.md section 9): a DLV record of files delivered. Not built yet.

use crate::cli::Global;
use crate::error::{Code, Error, Result};
use crate::output::Output;

/// The arguments of `delivered`.
#[derive(clap::Args, Debug)]
pub struct Args {}

/// Runs `delivered`. Not built yet.
pub fn run(_args: &Args, _g: &Global) -> Result<Output> {
    Err(Error::new(
        Code::NotImplemented,
        "delivered is not built yet",
    ))
}
