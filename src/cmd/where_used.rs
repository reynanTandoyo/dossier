//! The `where-used` command (docs/design.md section 12): not built yet.

use crate::cli::Global;
use crate::error::{Code, Error, Result};
use crate::output::Output;

/// The arguments of `where-used`.
#[derive(clap::Args, Debug)]
pub struct Args {}

/// Runs `where-used`.
pub fn run(_args: &Args, _g: &Global) -> Result<Output> {
    Err(Error::new(
        Code::NotImplemented,
        "where-used is not built yet",
    ))
}
