//! Snapshots: the earlier bytes of an entry, kept before each change.

use crate::error::{Code, Error, Result};
use std::path::Path;

/// One saved snapshot of an entry: its number, when it was taken, by whom and by which command.
pub struct Snap {
    pub n: u32,
    pub time: String,
    pub name: String,
    pub command: String,
}

/// Saves `bytes` as the next snapshot of entry `id`; returns its number.
pub fn save(
    state: &Path,
    id: &str,
    bytes: &[u8],
    time: &str,
    name: &str,
    command: &str,
) -> Result<u32> {
    Err(Error::new(Code::NotImplemented, "snapshot::save"))
}

/// The snapshots of entry `id`, oldest first.
pub fn list(state: &Path, id: &str) -> Result<Vec<Snap>> {
    Err(Error::new(Code::NotImplemented, "snapshot::list"))
}

/// The bytes of snapshot `n` of entry `id`.
pub fn load(state: &Path, id: &str, n: u32) -> Result<Vec<u8>> {
    Err(Error::new(Code::NotImplemented, "snapshot::load"))
}
