//! The `drafts` command.

use crate::error::{Code, Error, Result};
use crate::output::Output;

/// The arguments of `drafts`.
#[derive(clap::Args, Debug)]
pub struct Args {}

/// Runs `drafts`. Not built yet.
pub fn run(_args: &Args, _g: &crate::cli::Global) -> Result<Output> {
    Err(Error::new(Code::NotImplemented, "drafts is not built yet"))
}
