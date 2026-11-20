//! The `projects` command.

use crate::error::{Code, Error, Result};
use crate::output::Output;

/// The arguments of `projects`.
#[derive(clap::Args, Debug)]
pub struct Args {}

/// Runs `projects`. Not built yet.
pub fn run(_args: &Args, _g: &crate::cli::Global) -> Result<Output> {
    Err(Error::new(
        Code::NotImplemented,
        "projects is not built yet",
    ))
}
