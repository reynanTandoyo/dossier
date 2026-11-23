//! The `solved` command (docs/design.md section 15): closes an open investigation with its
//! solution and the attempts that led to it. Not built yet.

use crate::cli::Global;
use crate::error::{Code, Error, Result};
use crate::output::Output;

/// The arguments of `solved`.
#[derive(clap::Args, Debug)]
pub struct Args {}

/// Runs `solved`. Until it is built, every call fails with Code::NotImplemented.
pub fn run(_args: &Args, _g: &Global) -> Result<Output> {
    Err(Error::new(Code::NotImplemented, "solved is not built yet"))
}
