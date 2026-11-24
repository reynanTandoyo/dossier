//! The `after` command (docs/design.md section 14, R-AGENT-5): a Claude Code hook runs it after a
//! `todo` command, and it prints the dossier's context for that moment. This is the stub; the text
//! itself comes in a later task.

use crate::cli::Global;
use crate::error::Result;
use crate::output::Output;

/// The arguments of `after`: none.
#[derive(clap::Args, Debug)]
pub struct Args {}

/// Runs `after`. Until its text is written it prints nothing and succeeds.
pub fn run(_args: &Args, _g: &Global) -> Result<Output> {
    Ok(Output::text(""))
}
