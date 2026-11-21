//! Checksums and stamps of files outside the dossier (format.md §18), and counts of a folder.

use crate::error::{Code, Error, Result};
use std::path::Path;

/// Files above this size (1 GiB) get the short checksum (format.md §18).
pub const SHORT_ABOVE: u64 = 1 << 30;

/// "sha256:<hex>", or "short-sha256:<hex>" above SHORT_ABOVE (format.md §18). Reads in chunks.
pub fn checksum(_path: &Path) -> Result<String> {
    Err(Error::new(Code::NotImplemented, "filesum::checksum"))
}

/// The size and modification time of a file.
pub struct Stamp {
    pub size: u64,
    /// format.md §7, local time.
    pub modified: String,
}

/// The size and the modification time of the file at `path`.
pub fn stamp(_path: &Path) -> Result<Stamp> {
    Err(Error::new(Code::NotImplemented, "filesum::stamp"))
}

/// The number of regular files in a folder, recursive.
pub fn count_files(_folder: &Path) -> Result<u64> {
    Err(Error::new(Code::NotImplemented, "filesum::count_files"))
}
