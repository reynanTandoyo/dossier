//! The files of a team dossier that hold lists (format.md §20).

use crate::blockfile::BlockFile;
use crate::error::{Code, Error, Result};
use std::path::Path;

/// projects.md of the team dossier and its raw bytes, when it exists.
pub fn projects(_team: &Path, _code: &str) -> Result<(BlockFile, Option<Vec<u8>>)> {
    Err(Error::new(Code::NotImplemented, "teamfiles::projects"))
}

/// usage-index.md of the team dossier and its raw bytes, when it exists.
pub fn usage(_team: &Path, _code: &str) -> Result<(BlockFile, Option<Vec<u8>>)> {
    Err(Error::new(Code::NotImplemented, "teamfiles::usage"))
}
