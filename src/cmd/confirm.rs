//! The `confirm` command (docs/design.md section 9): a person confirms an assumption. Not built yet.

use crate::cli::Global;
use crate::error::{Code, Error, Result};
use crate::output::Output;

/// The arguments of `confirm`.
#[derive(clap::Args, Debug)]
pub struct Args {}

/// Runs `confirm`. Not built yet.
pub fn run(_args: &Args, _g: &Global) -> Result<Output> {
    Err(Error::new(Code::NotImplemented, "confirm is not built yet"))
}
