//! The `session-end` command (docs/design.md section 19, R-DRAFT-9): the command the SessionEnd
//! hook of Claude Code runs with `--run <command>`. This file declares the arguments and the
//! command; its steps are added by the tasks that follow.

use crate::cli::Global;
use crate::error::Result;
use crate::output::Output;

/// The arguments of `session-end`.
#[derive(clap::Args, Debug)]
pub struct Args {
    /// The team's review command. It is started with the path of the review material as its
    /// last argument.
    #[arg(long)]
    pub run: String,
}

/// Runs `session-end`. The steps are not added yet, so it prints nothing and exits 0.
pub fn run(_args: &Args, _g: &Global) -> Result<Output> {
    Ok(Output::text(""))
}
