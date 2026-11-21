//! The `confidential` command (docs/design.md section 12): not built yet.

use crate::cli::Global;
use crate::error::{Code, Error, Result};
use crate::output::Output;

/// The arguments of `confidential`.
#[derive(clap::Args, Debug)]
pub struct Args {}

/// Runs `confidential`.
pub fn run(_args: &Args, _g: &Global) -> Result<Output> {
    Err(Error::new(
        Code::NotImplemented,
        "confidential is not built yet",
    ))
}
