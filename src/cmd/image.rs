//! The `image` command (docs/design.md section 9): adds or removes pictures of an entry. Not built yet.

use crate::cli::Global;
use crate::error::{Code, Error, Result};
use crate::output::Output;

/// The arguments of `image`.
#[derive(clap::Args, Debug)]
pub struct Args {}

/// Runs `image`. Not built yet.
pub fn run(_args: &Args, _g: &Global) -> Result<Output> {
    Err(Error::new(Code::NotImplemented, "image is not built yet"))
}
