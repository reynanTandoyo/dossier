//! Scans the folder of a finished project for documents to bring in (R-IMPORT-2).

use crate::error::{Code, Error, Result};
use std::path::Path;

/// One document found in a folder.
pub struct Doc {
    pub stored: String,
    pub size: u64,
    pub modified: String,
    pub kind: String,
}

/// The folder tree to `depth` levels as indented lines `<name>/` and `<name>  <size>`.
pub fn tree(_root: &Path, _depth: usize) -> Result<Vec<String>> {
    Err(Error::new(Code::NotImplemented, "folderscan::tree"))
}

/// Documents found under root (reports .pdf .docx .doc .odt .md .txt, presentations .pptx
/// .ppt, spreadsheets .xlsx .xls .csv, scripts .py .java .m .sh .bat .ps1), newest first.
pub fn documents(_root: &Path) -> Result<Vec<Doc>> {
    Err(Error::new(Code::NotImplemented, "folderscan::documents"))
}
