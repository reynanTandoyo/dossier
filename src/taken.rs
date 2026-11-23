//! Taken files of the dossier (format.md §30, R-FILE-1 to R-FILE-7). A stub: nothing is taken yet,
//! so the functions answer as an empty file does.

use std::path::Path;

use crate::blockfile::BlockFile;
use crate::error::{Code, Error, Result};

/// The file of a dossier that holds the taken files, `taken.md`.
pub const FILE: &str = "taken.md";

/// The item ID of the taken files (format.md §30).
pub const ITEM: &str = "TAKEN";

/// The taken files file of the dossier and its raw bytes, when it exists. A missing file is
/// BlockFile::empty("Taken files of <code>") and None.
pub fn load(_dossier: &Path, _code: &str) -> Result<(BlockFile, Option<Vec<u8>>)> {
    Err(Error::new(Code::NotImplemented, "not built yet"))
}

/// A file that is taken now, with who took it and the state of the file when it was taken
/// (format.md §30).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Take {
    pub path: String,
    pub person: String,
    pub time: String,
    pub task: String,
    pub size: u64,
    pub modified: String,
    pub checksum: String,
    pub forced: bool,
}

/// The take of a path, None when the path is not taken now.
pub fn current(_file: &BlockFile, _path: &str) -> Option<Take> {
    None
}

/// Every file taken now, in file order.
pub fn all(_file: &BlockFile) -> Vec<Take> {
    Vec::new()
}

/// The history lines of a path: every take, return and abandon (R-FILE-5).
pub fn log(_file: &BlockFile, _path: &str) -> Vec<String> {
    Vec::new()
}
