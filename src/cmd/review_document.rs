//! The `review-document` command (docs/design.md section 12): not built yet.

use crate::cli::Global;
use crate::error::{Code, Error, Result};
use crate::output::Output;

/// The arguments of `review-document`.
#[derive(clap::Args, Debug)]
pub struct Args {}

/// Runs `review-document`.
pub fn run(_args: &Args, _g: &Global) -> Result<Output> {
    Err(Error::new(
        Code::NotImplemented,
        "review-document is not built yet",
    ))
}
