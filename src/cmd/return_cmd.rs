//! The `return` command (docs/design.md section 17, R-FILE-1 to R-FILE-7): gives back a large
//! file taken for a task. Not built yet; this stub only lets the command compile.

use crate::cli::Global;
use crate::error::{Code, Error, Result};
use crate::output::Output;

/// The arguments of `return`.
#[derive(clap::Args, Debug)]
pub struct Args {}

/// Runs `return`. Not built yet: always Code::NotImplemented.
pub fn run(_args: &Args, _g: &Global) -> Result<Output> {
    Err(Error::new(Code::NotImplemented, "return is not built yet"))
}
