//! The `reattach` command (docs/design.md section 9): records a file's size, time and checksum again. Not built yet.

use crate::cli::Global;
use crate::error::{Code, Error, Result};
use crate::output::Output;

/// The arguments of `reattach`.
#[derive(clap::Args, Debug)]
pub struct Args {}

/// Runs `reattach`. Not built yet.
pub fn run(_args: &Args, _g: &Global) -> Result<Output> {
    Err(Error::new(
        Code::NotImplemented,
        "reattach is not built yet",
    ))
}
