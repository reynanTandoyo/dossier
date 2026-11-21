//! The list of documents brought in from finished projects (format.md §25).

use crate::blockfile::BlockFile;
use crate::error::{Code, Error, Result};
use std::path::Path;

/// The imports file of the dossier and its raw bytes, when it exists.
pub fn load(_dossier: &Path, _code: &str) -> Result<(BlockFile, Option<Vec<u8>>)> {
    Err(Error::new(Code::NotImplemented, "imports::load"))
}

/// The state of the document `stored` in the imports file, when it is listed.
pub fn state(_file: &BlockFile, _stored: &str) -> Option<String> {
    None
}
