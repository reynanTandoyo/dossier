//! The terminal: whether a person is at it, and asking one question.
//!
//! `DOSSIER_ANSWERS` (answers separated by `|`) replaces the terminal in tests.

use crate::error::{Code, Error, Result};

/// Whether stdin and stdout are terminals.
pub fn is_terminal() -> bool {
    false
}

/// Asks for one line. Not a terminal: Code::NeedsTerminal.
pub fn ask(prompt: &str) -> Result<String> {
    Err(Error::new(Code::NotImplemented, "term::ask"))
}
