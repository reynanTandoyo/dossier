//! A minimal tar archive: ustar, regular files and folders, no compression.

use crate::error::{Code, Error, Result};
use std::path::Path;

/// Writes the files and folders of `folder` into the archive `out`. Returns the number of files.
pub fn pack(folder: &Path, out: &Path) -> Result<u32> {
    Err(Error::new(Code::NotImplemented, "tar::pack"))
}

/// Restores the archive `archive` into `into`, which must be empty or missing. Returns the
/// number of files.
pub fn unpack(archive: &Path, into: &Path) -> Result<u32> {
    Err(Error::new(Code::NotImplemented, "tar::unpack"))
}
