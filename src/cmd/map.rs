//! The `map` command (docs/design.md section 9): the map of a code project. Not built yet.

use crate::cli::Global;
use crate::error::{Code, Error, Result};
use crate::output::Output;

/// The arguments of `map`.
#[derive(clap::Args, Debug)]
pub struct Args {}

/// Runs `map`. Not built yet.
pub fn run(_args: &Args, _g: &Global) -> Result<Output> {
    Err(Error::new(Code::NotImplemented, "map is not built yet"))
}
