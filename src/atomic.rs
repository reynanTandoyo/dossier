//! Atomic replacement of a file, checked against the bytes that were read.

use crate::error::{Code, Error, Result};
use std::path::Path;

/// What a file looked like when it was read: its length and its checksum.
#[derive(Debug, Clone, PartialEq)]
pub struct Seen {
    pub len: u64,
    pub hash: String,
}

/// The length and checksum of the file at `path`; None when the file does not exist.
pub fn seen(path: &Path) -> Result<Option<Seen>> {
    Err(Error::new(Code::NotImplemented, "atomic::seen"))
}

/// Writes `<name>.tmp-<pid>` next to the file, flushes, renames. With `expected`, first checks
/// that the file is still that (None = must not exist); otherwise Code::Changed.
pub fn replace(path: &Path, bytes: &[u8], expected: Option<&Option<Seen>>) -> Result<()> {
    Err(Error::new(Code::NotImplemented, "atomic::replace"))
}
