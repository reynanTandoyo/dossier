//! Running the program on a server through ssh.
//!
//! The ssh program is `ssh`, or the value of `DOSSIER_SSH` (tests point it at a stand-in).

use crate::error::{Code, Error, Result};

/// One shell word for a POSIX shell: wrapped in single quotes, each ' as '\''.
pub fn quote(arg: &str) -> String {
    String::new()
}

/// The argument list for the ssh program: -o BatchMode=yes -o ConnectTimeout=<n>,
/// -t when `terminal`, the server, then one string:
/// `cd <quoted folder> && DOSSIER_AGENT=<quoted> <quoted program> <quoted args…>`.
pub fn argv(
    server: &str,
    folder: &str,
    agent: Option<&str>,
    program: &str,
    args: &[String],
    terminal: bool,
    timeout: u64,
) -> Vec<String> {
    Vec::new()
}

/// Runs the ssh program with inherited input and output and returns the remote exit code.
/// ssh itself failing (exit 255, or the program missing): Code::Ssh.
pub fn run(argv: &[String]) -> Result<u8> {
    Err(Error::new(Code::NotImplemented, "remote::ssh::run"))
}
