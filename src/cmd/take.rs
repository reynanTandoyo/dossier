//! The `take` command (docs/design.md section 17, R-FILE-1 to R-FILE-7): records who takes a
//! large file for a task. Not built yet; this stub only lets the command compile.

use crate::cli::Global;
use crate::error::{Code, Error, Result};
use crate::output::Output;

/// The arguments of `take`.
#[derive(clap::Args, Debug)]
pub struct Args {}

/// Runs `take`. Not built yet: always Code::NotImplemented.
pub fn run(_args: &Args, _g: &Global) -> Result<Output> {
    Err(Error::new(Code::NotImplemented, "take is not built yet"))
}
