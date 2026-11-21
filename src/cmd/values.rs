//! The `values` command (docs/design.md section 9): the values in force. Not built yet.

use crate::cli::Global;
use crate::error::{Code, Error, Result};
use crate::output::Output;

/// The arguments of `values`.
#[derive(clap::Args, Debug)]
pub struct Args {}

/// Runs `values`. Not built yet.
pub fn run(_args: &Args, _g: &Global) -> Result<Output> {
    Err(Error::new(Code::NotImplemented, "values is not built yet"))
}
