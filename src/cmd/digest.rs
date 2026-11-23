//! The `digest` command (docs/design.md section 18, R-CTX-14, R-CTX-15): the material for summing
//! up a closed milestone, and the digest entry that a person approves. A stub until the command
//! is built.

use crate::cli::Global;
use crate::error::{Code, Error, Result};
use crate::output::Output;

/// The arguments of `digest`.
#[derive(clap::Args, Debug)]
pub struct Args {}

/// Runs `digest`. Not built yet: it fails with Code::NotImplemented.
pub fn run(_args: &Args, _g: &Global) -> Result<Output> {
    Err(Error::new(Code::NotImplemented, "digest is not built yet"))
}
