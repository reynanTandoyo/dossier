//! The confidential terms of a project and the check that keeps them out of the team dossier
//! (format.md §24).

use crate::blockfile::BlockFile;
use crate::error::{Code, Error, Result};
use std::path::Path;

/// The line a project states when it has no confidential terms.
pub const NONE: &str = "(no confidential terms)";

/// Lower case; runs of space, '-' and '_' become one space.
pub fn normalize(_text: &str) -> String {
    String::new()
}

/// The terms file of the dossier and its raw bytes, when it exists.
pub fn load(_dossier: &Path, _code: &str) -> Result<(BlockFile, Option<Vec<u8>>)> {
    Err(Error::new(Code::NotImplemented, "conf::load"))
}

/// The terms not removed, NONE left out.
pub fn terms(_file: &BlockFile) -> Vec<String> {
    Vec::new()
}

/// Whether the file states that the project has no confidential terms.
pub fn stated_none(_file: &BlockFile) -> bool {
    false
}

/// Each term found in `text` with the 1-based line it is on.
pub fn find(_terms: &[String], _text: &str) -> Vec<(String, usize)> {
    Vec::new()
}

/// The terms of a project before writing to the team dossier: Code::Refused when there are none
/// and none was stated (R-CONF-5, the message says how to add them or state none).
pub fn gate(_project: &Path, _code: &str) -> Result<Vec<String>> {
    Err(Error::new(Code::NotImplemented, "conf::gate"))
}

/// Code::Refused naming each term and line when `text` holds one (R-CONF-2).
pub fn check(_terms: &[String], _text: &str) -> Result<()> {
    Err(Error::new(Code::NotImplemented, "conf::check"))
}
