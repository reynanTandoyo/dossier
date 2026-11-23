//! The `export` command (docs/design.md section 20, R-OUT-7): one Markdown document with the
//! decisions, findings, investigations and usage records in time order. A stub until the command
//! is built.

use crate::cli::Global;
use crate::error::{Code, Error, Result};
use crate::output::Output;

/// The arguments of `export`.
#[derive(clap::Args, Debug)]
pub struct Args {}

/// Runs `export`. Not built yet: it fails with Code::NotImplemented.
pub fn run(_args: &Args, _g: &Global) -> Result<Output> {
    Err(Error::new(Code::NotImplemented, "export is not built yet"))
}
