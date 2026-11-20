//! The `supersede` command.

use crate::error::{Code, Error, Result};
use crate::output::Output;

/// The arguments of `supersede`.
#[derive(clap::Args, Debug)]
pub struct Args {}

/// Runs `supersede`. Not built yet.
pub fn run(_args: &Args, _g: &crate::cli::Global) -> Result<Output> {
    Err(Error::new(
        Code::NotImplemented,
        "supersede is not built yet",
    ))
}
