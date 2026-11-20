//! The bridge to the `todo` program: runs it and reads its JSON output.
//!
//! The program is `todo`, or the value of `DOSSIER_TODO` (tests point it at a stand-in).

use crate::error::{Code, Error, Result};
use std::path::Path;

/// Whether the program runs: `todo --version` works.
pub fn available() -> bool {
    false
}

/// Runs `todo <args> --json` in `root` and returns the `data` of the envelope.
/// Program missing: Code::TodoMissing. `ok:false`: Code::Refused with todo's message.
pub fn json(root: &Path, args: &[&str]) -> Result<serde_json::Value> {
    Err(Error::new(Code::NotImplemented, "todo_bridge::json"))
}
