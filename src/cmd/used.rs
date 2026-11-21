//! The `used` command (docs/design.md section 9): a usage record of software, made from a model. Not built yet.

use crate::cli::Global;
use crate::error::{Code, Error, Result};
use crate::output::Output;

/// The arguments of `used`.
#[derive(clap::Args, Debug)]
pub struct Args {}

/// Runs `used`. Not built yet.
pub fn run(_args: &Args, _g: &Global) -> Result<Output> {
    Err(Error::new(Code::NotImplemented, "used is not built yet"))
}
