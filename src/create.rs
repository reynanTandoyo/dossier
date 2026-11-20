//! Creating an entry inside a transaction.

use crate::entry::Kind;
use crate::error::{Code, Error, Result};
use crate::tx::Tx;

/// What a command asks to create.
pub struct New {
    pub kind: Kind,
    pub title: String,
    pub fields: Vec<(String, String)>,
    pub sections: Vec<(String, String)>,
    pub said_by: Option<String>,
    pub draft: bool,
}

/// Creates one entry inside a transaction and returns its ID and file text.
/// Author and status: a person writing: author = person, approved unless `draft`.
/// An LLM with `said_by` naming a person of the settings: author = that person, via = the LLM,
/// approved unless `draft`. An LLM without `said_by`: author = its person, via = the LLM;
/// approved for decision (see below), assumption and finding; draft for a lesson.
/// An LLM writing a decision without `said_by`: Code::NeedsPerson.
/// Approved entries get `approved`, `valid-from`, two history lines (created, approved) and,
/// for a lesson, approval::record. Refusing unknown tag values is not part of v0.1.
pub fn create(tx: &mut Tx, new: &New) -> Result<(String, String)> {
    Err(Error::new(Code::NotImplemented, "create::create"))
}
