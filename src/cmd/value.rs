//! The `value` command (docs/design.md section 9): key values: set, show, approve, reject. Not built yet.

use crate::cli::Global;
use crate::error::{Code, Error, Result};
use crate::output::Output;

/// The arguments of `value`.
#[derive(clap::Args, Debug)]
pub struct Args {}

/// Runs `value`. Not built yet.
pub fn run(_args: &Args, _g: &Global) -> Result<Output> {
    Err(Error::new(Code::NotImplemented, "value is not built yet"))
}
