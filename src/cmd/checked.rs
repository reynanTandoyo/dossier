//! The `checked` command (docs/design.md section 25.5, R-WRITE-18, D62): says that the check of a
//! lesson was shown to fail on the mistake the lesson is about. Only the registration is in place;
//! the behaviour follows in a later task, so the command is refused with Code::NotImplemented.

use crate::cli::Global;
use crate::error::{Code, Error, Result};
use crate::output::Output;

/// The arguments of `checked`.
#[derive(clap::Args, Debug)]
pub struct Args {
    /// The lesson, such as LES-004; any case, with or without a place prefix
    pub id: String,
    /// How the check was shown to fail on the mistake, such as "the test fails without the fix"
    #[arg(long)]
    pub by: String,
    /// The test, lint, hook or review step that catches the mistake; needed when the lesson names
    /// none
    #[arg(long)]
    pub check: Option<String>,
    /// The person who decided this; needed when an LLM runs the command (D57)
    #[arg(long, value_name = "PERSON")]
    pub said_by: Option<String>,
}

/// Runs `checked`. It is not part of this change yet and is refused with Code::NotImplemented.
pub fn run(_args: &Args, _g: &Global) -> Result<Output> {
    Err(Error::new(
        Code::NotImplemented,
        "checked is not available yet",
    ))
}
