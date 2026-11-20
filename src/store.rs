//! Reading the entry files of a dossier.

use crate::entry::{Entry, Kind};
use crate::error::{Code, Error, Result};
use std::path::{Path, PathBuf};

/// One entry file as it was read: where it is, its exact bytes, and the parsed entry.
pub struct Item {
    pub path: PathBuf,
    pub raw: Vec<u8>,
    pub entry: Entry,
}

/// Every entry file of the dossier, sorted by ID. Files that fail to parse become warnings
/// `<path>:<line>: <message>`.
pub fn all(dossier: &Path) -> (Vec<Item>, Vec<String>) {
    (Vec::new(), Vec::new())
}

/// By ID, with or without a place prefix (`P-017:DEC-012`), any case. Code::NotFound.
pub fn get(dossier: &Path, id: &str) -> Result<Item> {
    Err(Error::new(Code::NotImplemented, "store::get"))
}

/// The path of the file of an entry with this kind, ID and title.
pub fn path_for(dossier: &Path, kind: Kind, id: &str, title: &str) -> PathBuf {
    PathBuf::new()
}
