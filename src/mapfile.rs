//! The map of a code project (format.md §17, map.md): one block per described folder or package.

use crate::blockfile::BlockFile;
use crate::error::{Code, Error, Result};
use std::path::Path;

/// The file of a dossier that holds the map, `map.md`.
pub const FILE: &str = "map.md";

/// The item name of the map file in snapshots and the journal.
pub const ITEM: &str = "MAP";

/// The map file of a project and its bytes as read; a missing file is
/// BlockFile::empty("Map of <code>"), None.
pub fn load(_dossier: &Path, _code: &str) -> Result<(BlockFile, Option<Vec<u8>>)> {
    Err(Error::new(Code::NotImplemented, "mapfile::load"))
}

/// The map as lines `<path>  <purpose>[ (start: <start>)]`, sorted by path, detached left out.
pub fn lines(_file: &BlockFile) -> Vec<String> {
    Vec::new()
}
