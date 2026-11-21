//! The `propose` command (docs/design.md section 9): a decision with status proposed, from anyone. Not built yet.

use crate::cli::Global;
use crate::error::{Code, Error, Result};
use crate::output::Output;

/// The arguments of `propose`.
#[derive(clap::Args, Debug)]
pub struct Args {}

/// Runs `propose`. Not built yet.
pub fn run(_args: &Args, _g: &Global) -> Result<Output> {
    Err(Error::new(Code::NotImplemented, "propose is not built yet"))
}
