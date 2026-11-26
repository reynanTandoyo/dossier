//! The `takeover` command (docs/design.md section 22, R-HAND-2, R-HAND-3 and R-HAND-5, D54): takes
//! over a task in todo, also one a colleague holds, and shows its progress. Only the registration
//! is in place; the behaviour follows in a later task, so the command is refused with
//! Code::NotImplemented.

use crate::cli::Global;
use crate::error::{Code, Error, Result};
use crate::output::Output;

/// The arguments of `takeover`.
#[derive(clap::Args, Debug)]
pub struct Args {
    /// The todo task, such as T-042
    pub task: String,
    /// The person who agreed to the takeover; an LLM needs it (D52)
    #[arg(long, value_name = "PERSON")]
    pub said_by: Option<String>,
}

/// Runs `takeover`. It is not part of this change yet and is refused with Code::NotImplemented.
pub fn run(_args: &Args, _g: &Global) -> Result<Output> {
    Err(Error::new(
        Code::NotImplemented,
        "takeover is not available yet",
    ))
}
