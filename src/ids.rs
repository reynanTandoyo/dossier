//! Entry IDs: the next free number of each kind.

use crate::entry::Kind;
use crate::error::{Code, Error, Result};
use std::path::Path;

/// The next ID of a kind: 1 + max(the number in ids.json, the highest number among the
/// entries on disk). Writes ids.json. Call only while holding the lock.
pub fn next(dossier: &Path, state: &Path, kind: Kind) -> Result<String> {
    Err(Error::new(Code::NotImplemented, "ids::next"))
}
