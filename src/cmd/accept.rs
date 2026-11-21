//! The `accept` command (docs/design.md section 9): a decider accepts a proposal, at a terminal. Not built yet.

use crate::cli::Global;
use crate::error::{Code, Error, Result};
use crate::output::Output;

/// The arguments of `accept`.
#[derive(clap::Args, Debug)]
pub struct Args {}

/// Runs `accept`. Not built yet.
pub fn run(_args: &Args, _g: &Global) -> Result<Output> {
    Err(Error::new(Code::NotImplemented, "accept is not built yet"))
}
