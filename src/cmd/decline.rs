//! The `decline` command (docs/design.md section 9): a decider declines a proposal, at a terminal. Not built yet.

use crate::cli::Global;
use crate::error::{Code, Error, Result};
use crate::output::Output;

/// The arguments of `decline`.
#[derive(clap::Args, Debug)]
pub struct Args {}

/// Runs `decline`. Not built yet.
pub fn run(_args: &Args, _g: &Global) -> Result<Output> {
    Err(Error::new(Code::NotImplemented, "decline is not built yet"))
}
