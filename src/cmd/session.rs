//! The `session` command (docs/design.md section 19, format.md section 34, D44): converts a
//! transcript of Claude Code into a session file. A stub until the command is built.

use crate::cli::Global;
use crate::error::{Code, Error, Result};
use crate::output::Output;

/// The arguments of `session`.
#[derive(clap::Args, Debug)]
pub struct Args {}

/// Runs `session`. Not built yet: it fails with Code::NotImplemented.
pub fn run(_args: &Args, _g: &Global) -> Result<Output> {
    Err(Error::new(Code::NotImplemented, "session is not built yet"))
}
