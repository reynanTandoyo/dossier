//! The `handover` command (docs/design.md section 22, R-HAND-1 and R-HAND-5, D54): gives a task
//! back in todo with a note of what is done and what is next. Only the registration is in place;
//! the behaviour follows in a later task, so the command is refused with Code::NotImplemented.

use crate::cli::Global;
use crate::error::{Code, Error, Result};
use crate::output::Output;

/// The arguments of `handover`.
#[derive(clap::Args, Debug)]
pub struct Args {
    /// The todo task, such as T-042
    pub task: String,
    /// What is done and what is next
    #[arg(long, value_name = "text")]
    pub note: String,
}

/// Runs `handover`. It is not part of this change yet and is refused with Code::NotImplemented.
pub fn run(_args: &Args, _g: &Global) -> Result<Output> {
    Err(Error::new(
        Code::NotImplemented,
        "handover is not available yet",
    ))
}
