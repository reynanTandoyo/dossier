//! The pointer file: a project folder's record of the server folder its dossier is kept in.

use crate::error::{Code, Error, Result};
use std::path::{Path, PathBuf};

/// Where the dossier of a project lives on a server.
pub struct Pointer {
    pub server: String,
    pub folder: String,
    pub code: String,
}

/// The pointer file found walking up from `start`, and the folder it is in.
pub fn find(start: &Path) -> Option<(PathBuf, Pointer)> {
    None
}

/// Writes the pointer file into `folder`.
pub fn write(folder: &Path, pointer: &Pointer) -> Result<()> {
    Err(Error::new(Code::NotImplemented, "remote::pointer::write"))
}
