//! The `guard` command (docs/design.md section 9, R-AGENT-3): reads a Claude Code PreToolUse
//! payload on standard input and blocks an edit of a file inside a DOSSIER folder. It needs no
//! dossier and no name; the decision is `hooks::guard`.

use crate::cli::Global;
use crate::error::{Code, Error, Result};
use crate::hooks;
use crate::output::Output;

/// The arguments of `guard`.
#[derive(clap::Args, Debug)]
pub struct Args {}

/// Runs `guard`. A payload that edits a file inside DOSSIER fails with Code::Usage, so the exit
/// code is 2, which Claude Code treats as a block, and the message goes to standard error. Any
/// other payload, or text that is not one, exits 0 with nothing printed.
pub fn run(_args: &Args, _g: &Global) -> Result<Output> {
    let mut payload = String::new();
    std::io::Read::read_to_string(&mut std::io::stdin(), &mut payload)?;
    match hooks::guard(&payload) {
        Some(message) => Err(Error::new(Code::Usage, message)),
        None => Ok(Output::text("")),
    }
}
