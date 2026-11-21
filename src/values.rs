//! Key values of a project (format.md §17, values.md): one block per value, with its history.

use crate::blockfile::BlockFile;
use crate::error::{Code, Error, Result};
use std::path::Path;

/// The file of a dossier that holds the values, `values.md`.
pub const FILE: &str = "values.md";

/// The item name of the values file in snapshots and the journal.
pub const ITEM: &str = "VALUES";

/// One value of the project, read from its block of values.md.
pub struct Value {
    pub name: String,
    pub value: String,
    pub unit: Option<String>,
    pub source: Option<String>,
    pub note: Option<String>,
    pub key: bool,
    pub valid_from: Option<String>,
    pub proposed: Option<String>,
}

/// A valid name of a value: lower-case letters, digits, `.`, `_` and `-`, starting with a letter
/// (format.md §17).
pub fn valid_name(_name: &str) -> bool {
    false
}

/// "12.4 kg/s" -> ("12.4", Some("kg/s")): the unit is the last word. With `dimensionless` the
/// whole text is the value. One word without `dimensionless`: Code::BadValue.
pub fn split(_text: &str, _dimensionless: bool) -> Result<(String, Option<String>)> {
    Err(Error::new(Code::NotImplemented, "values::split"))
}

/// The file and its bytes as read; a missing file is BlockFile::empty("Values of <code>"), None.
pub fn load(_dossier: &Path, _code: &str) -> Result<(BlockFile, Option<Vec<u8>>)> {
    Err(Error::new(Code::NotImplemented, "values::load"))
}

/// Every value of the file, in file order.
pub fn all(_file: &BlockFile) -> Vec<Value> {
    Vec::new()
}

/// The value of the file with this name.
pub fn get(_file: &BlockFile, _name: &str) -> Option<Value> {
    None
}

/// "<value>[ <unit>]" in force at the end of `date` (YYYY-MM-DD), from the `set` history lines.
pub fn on(_file: &BlockFile, _name: &str, _date: &str) -> Option<String> {
    None
}
