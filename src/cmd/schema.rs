//! The `schema` command (docs/design.md section 20, R-OUT-6): a machine-readable description of
//! the commands, their JSON shapes and the error codes. A stub until the command is built.

use crate::cli::Global;
use crate::error::{Code, Error, Result};
use crate::output::Output;

/// The arguments of `schema`.
#[derive(clap::Args, Debug)]
pub struct Args {}

/// Runs `schema`. Not built yet: it fails with Code::NotImplemented.
pub fn run(_args: &Args, _g: &Global) -> Result<Output> {
    Err(Error::new(Code::NotImplemented, "schema is not built yet"))
}
