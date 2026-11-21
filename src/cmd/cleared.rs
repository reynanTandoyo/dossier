//! The `cleared` command (docs/design.md section 9): a person clears one rests-on mark. Not built yet.

use crate::cli::Global;
use crate::error::{Code, Error, Result};
use crate::output::Output;

/// The arguments of `cleared`.
#[derive(clap::Args, Debug)]
pub struct Args {}

/// Runs `cleared`. Not built yet.
pub fn run(_args: &Args, _g: &Global) -> Result<Output> {
    Err(Error::new(Code::NotImplemented, "cleared is not built yet"))
}
