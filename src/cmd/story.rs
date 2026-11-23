//! The `story` command (docs/design.md section 20, R-READ-7): the approved decisions in the order
//! they were made. A stub until the command is built.

use crate::cli::Global;
use crate::error::{Code, Error, Result};
use crate::output::Output;

/// The arguments of `story`.
#[derive(clap::Args, Debug)]
pub struct Args {}

/// Runs `story`. Not built yet: it fails with Code::NotImplemented.
pub fn run(_args: &Args, _g: &Global) -> Result<Output> {
    Err(Error::new(Code::NotImplemented, "story is not built yet"))
}
