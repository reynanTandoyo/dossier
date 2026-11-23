//! The `review-session` command (docs/design.md section 19, R-DRAFT-7, R-DRAFT-9): the decisive
//! parts of a session file, as material for drafting. A stub until the command is built.

use crate::cli::Global;
use crate::error::{Code, Error, Result};
use crate::output::Output;

/// The arguments of `review-session`.
#[derive(clap::Args, Debug)]
pub struct Args {}

/// Runs `review-session`. Not built yet: it fails with Code::NotImplemented.
pub fn run(_args: &Args, _g: &Global) -> Result<Output> {
    Err(Error::new(
        Code::NotImplemented,
        "review-session is not built yet",
    ))
}
