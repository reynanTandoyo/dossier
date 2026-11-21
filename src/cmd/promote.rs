//! The `promote` command (docs/design.md section 12): not built yet.

use crate::cli::Global;
use crate::error::{Code, Error, Result};
use crate::output::Output;

/// The arguments of `promote`.
#[derive(clap::Args, Debug)]
pub struct Args {}

/// Runs `promote`.
pub fn run(_args: &Args, _g: &Global) -> Result<Output> {
    Err(Error::new(Code::NotImplemented, "promote is not built yet"))
}
