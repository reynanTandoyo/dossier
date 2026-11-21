//! The `refute` command (docs/design.md section 9): a person refutes an assumption, at a terminal. Not built yet.

use crate::cli::Global;
use crate::error::{Code, Error, Result};
use crate::output::Output;

/// The arguments of `refute`.
#[derive(clap::Args, Debug)]
pub struct Args {}

/// Runs `refute`. Not built yet.
pub fn run(_args: &Args, _g: &Global) -> Result<Output> {
    Err(Error::new(Code::NotImplemented, "refute is not built yet"))
}
