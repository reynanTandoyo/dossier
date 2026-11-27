//! The `sweep` command (docs/design.md section 25.6, R-SWEEP-1, D63): lists the pairs of approved
//! entries in force that look alike, followed by the rules for judging them. Only the registration
//! is in place; the behaviour follows in a later task, so the command is refused with
//! Code::NotImplemented.

use crate::cli::Global;
use crate::error::{Code, Error, Result};
use crate::output::Output;

/// The arguments of `sweep`.
#[derive(clap::Args, Debug)]
pub struct Args {
    /// The most pairs to list
    #[arg(long, default_value_t = 20)]
    pub limit: usize,
}

/// Runs `sweep`. It is not part of this change yet and is refused with Code::NotImplemented.
pub fn run(_args: &Args, _g: &Global) -> Result<Output> {
    Err(Error::new(
        Code::NotImplemented,
        "sweep is not available yet",
    ))
}
