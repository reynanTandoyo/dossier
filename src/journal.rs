//! The journal: a record of every change, who made it and when.

use crate::error::{Code, Error, Result};
use std::path::Path;

/// One journal record.
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct Record {
    pub time: String,
    pub name: String,
    pub person: String,
    pub llm: bool,
    pub command: String,
    pub id: String,
    pub event: String,
    pub detail: String,
}

/// Appends a record to the journal in `state`.
pub fn append(state: &Path, record: &Record) -> Result<()> {
    Err(Error::new(Code::NotImplemented, "journal::append"))
}

/// Every record, oldest first; bad lines are skipped.
pub fn read(state: &Path) -> Result<Vec<Record>> {
    Err(Error::new(Code::NotImplemented, "journal::read"))
}
