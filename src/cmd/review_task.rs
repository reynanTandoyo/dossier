//! The `review-task` command.

use crate::error::{Code, Error, Result};
use crate::output::Output;

/// The arguments of `review-task`.
#[derive(clap::Args, Debug)]
pub struct Args {}

/// Runs `review-task`. Not built yet.
pub fn run(_args: &Args, _g: &crate::cli::Global) -> Result<Output> {
    Err(Error::new(
        Code::NotImplemented,
        "review-task is not built yet",
    ))
}
