//! The glossary of the team dossier (format.md §20).

use crate::blockfile::BlockFile;
use crate::error::{Code, Error, Result};
use std::path::Path;

/// One term of the glossary.
pub struct Term {
    pub term: String,
    pub meaning: String,
    pub symbol: Option<String>,
    pub unit: Option<String>,
    pub also: Vec<String>,
}

/// The glossary file of the dossier and its raw bytes, when it exists.
pub fn load(_dossier: &Path, _code: &str) -> Result<(BlockFile, Option<Vec<u8>>)> {
    Err(Error::new(Code::NotImplemented, "glossary::load"))
}

/// The terms of the glossary file.
pub fn terms(_file: &BlockFile) -> Vec<Term> {
    Vec::new()
}

/// Every spelling of the terms that `word` is a spelling of (term and `also`, any case).
pub fn spellings(_terms: &[Term], _word: &str) -> Vec<String> {
    Vec::new()
}
