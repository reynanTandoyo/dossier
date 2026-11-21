//! The `hooks` command (docs/design.md section 9): Claude Code hooks in .claude/settings.json. Not built yet.

use crate::cli::Global;
use crate::error::{Code, Error, Result};
use crate::output::Output;

/// The arguments of `hooks`.
#[derive(clap::Args, Debug)]
pub struct Args {}

/// Runs `hooks`. Not built yet.
pub fn run(_args: &Args, _g: &Global) -> Result<Output> {
    Err(Error::new(Code::NotImplemented, "hooks is not built yet"))
}
