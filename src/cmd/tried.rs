//! The `tried` command (docs/design.md section 15, R-INV-3): records one attempt on an
//! investigation. This file is a stub until its own task builds the command.

use crate::cli::Global;
use crate::error::{Code, Error, Result};
use crate::output::Output;

/// The arguments of `tried`.
#[derive(clap::Args, Debug)]
pub struct Args {}

/// Runs `tried`. It is not built yet, so it fails with Code::NotImplemented (exit code 2).
pub fn run(_args: &Args, _g: &Global) -> Result<Output> {
    Err(Error::new(Code::NotImplemented, "tried is not built yet"))
}
