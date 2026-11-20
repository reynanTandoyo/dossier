//! The `reopen` command.

use crate::error::{Code, Error, Result};
use crate::output::Output;

/// The arguments of `reopen`.
#[derive(clap::Args, Debug)]
pub struct Args {}

/// Runs `reopen`. Not built yet.
pub fn run(_args: &Args, _g: &crate::cli::Global) -> Result<Output> {
    Err(Error::new(Code::NotImplemented, "reopen is not built yet"))
}
