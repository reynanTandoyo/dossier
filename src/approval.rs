//! Approval records: a checksum of each approved entry, so that a changed entry shows as changed.
//!
//! The checksum covers the file without its `- history:` lines and without the `status`,
//! `approved`, `valid-from` and `valid-until` lines.

use crate::error::{Code, Error, Result};
use std::path::Path;

/// How an entry compares with the checksum recorded at approval.
pub enum Mark {
    Unchanged,
    Changed,
    NotRecorded,
}

/// Records the checksum of an approved entry.
pub fn record(state: &Path, id: &str, bytes: &[u8]) -> Result<()> {
    Err(Error::new(Code::NotImplemented, "approval::record"))
}

/// Forgets the recorded checksum of entry `id`.
pub fn forget(state: &Path, id: &str) -> Result<()> {
    Err(Error::new(Code::NotImplemented, "approval::forget"))
}

/// Compares an entry with its recorded checksum.
pub fn mark(state: &Path, id: &str, bytes: &[u8]) -> Mark {
    Mark::NotRecorded
}
