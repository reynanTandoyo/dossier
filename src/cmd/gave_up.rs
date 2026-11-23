//! The `gave-up` command (docs/design.md section 15): closes an open investigation without a
//! solution, with the reason. Not built yet.

use crate::cli::Global;
use crate::error::{Code, Error, Result};
use crate::output::Output;

/// The arguments of `gave-up`.
#[derive(clap::Args, Debug)]
pub struct Args {}

/// Runs `gave-up`. Until it is built, every call fails with Code::NotImplemented.
pub fn run(_args: &Args, _g: &Global) -> Result<Output> {
    Err(Error::new(Code::NotImplemented, "gave-up is not built yet"))
}
