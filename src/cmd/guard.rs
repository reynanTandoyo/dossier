//! The `guard` command (docs/design.md sections 9 and 21.3, D51, R-CODE-8): reads a Claude Code
//! PreToolUse payload on standard input. It blocks an edit of a file inside a DOSSIER folder
//! (`hooks::guard`). It also refuses the first edit of a file in a session when approved entries in
//! force bind the file, and names them, so the LLM reads them before it changes the file; the next
//! edit of that file goes through, because `which --binding --session` leaves out what the session
//! was shown. It needs no dossier and no name for the first check; the second runs this program
//! again in the current folder.

use crate::cli::Global;
use crate::error::{Code, Error, Result};
use crate::hooks;
use crate::output::Output;
use std::process::{Command, Stdio};

/// The arguments of `guard`.
#[derive(clap::Args, Debug)]
pub struct Args {}

/// Runs `guard`. A payload that edits a file inside DOSSIER fails with Code::Blocked, so the exit
/// code is 2, which Claude Code treats as a block, and the message goes to standard error alone,
/// without an `error:` prefix. An edit of a file that entries in force bind, in a session that has
/// not been shown them yet, fails the same way and names those entries. Any other payload, or text
/// that is not one, exits 0 with nothing printed.
pub fn run(_args: &Args, _g: &Global) -> Result<Output> {
    let mut payload = String::new();
    std::io::Read::read_to_string(&mut std::io::stdin(), &mut payload)?;
    if let Some(message) = hooks::guard(&payload) {
        return Err(Error::new(Code::Blocked, message));
    }
    if let Some(message) = binding_message(&payload) {
        return Err(Error::new(Code::Blocked, message));
    }
    Ok(Output::text(""))
}

/// The message to refuse an edit with when the entries in force that bind the edited file have not
/// been shown to the session yet (`which --binding --session`); None to allow. Without a file or
/// without a session nothing is checked, and an empty session counts as none: every edit in it
/// would be refused again.
fn binding_message(payload: &str) -> Option<String> {
    let path = hooks::edited_path(payload)?;
    let session = hooks::session_of(payload).filter(|session| !session.is_empty())?;
    let text = own(&[
        "which",
        "--binding",
        path.as_str(),
        "--session",
        session.as_str(),
    ])?;
    let entries = text.trim_end_matches(['\r', '\n']);
    if entries.trim().is_empty() {
        return None;
    }
    Some(format!(
        "Entries in force for {path}; read them, then make your edit again (dossier show <ID> prints one in full):\n{entries}"
    ))
}

/// The standard output of this program run again with `args`, None unless it ends with exit 0.
fn own(args: &[&str]) -> Option<String> {
    let program = std::env::current_exe().ok()?;
    let output = Command::new(program)
        .args(args)
        .stdin(Stdio::null())
        .output()
        .ok()?;
    output
        .status
        .success()
        .then(|| String::from_utf8_lossy(&output.stdout).into_owned())
}
