//! Entries: the files of decisions, assumptions, findings and lessons.

pub mod edit;
pub mod new;
pub mod rules;

use crate::error::{Code, Error, Result};

/// The kind of an entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Decision,
    Assumption,
    Finding,
    Lesson,
}

impl Kind {
    /// The four kinds.
    pub fn all() -> [Kind; 4] {
        [
            Kind::Decision,
            Kind::Assumption,
            Kind::Finding,
            Kind::Lesson,
        ]
    }

    /// The name of the kind.
    pub fn name(self) -> &'static str {
        ""
    }

    /// The prefix of IDs of this kind.
    pub fn prefix(self) -> &'static str {
        ""
    }

    /// The folder of this kind inside DOSSIER.
    pub fn folder(self) -> &'static str {
        ""
    }

    /// The kind by name or prefix, in any case.
    pub fn parse(text: &str) -> Option<Kind> {
        None
    }

    /// The kind of an ID, with or without a place prefix (`P-017:DEC-012`), in any case.
    pub fn of_id(id: &str) -> Option<Kind> {
        None
    }
}

/// The status of an entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Draft,
    Approved,
    Rejected,
    Superseded,
    Retired,
}

impl Status {
    /// The name of the status, as the file writes it.
    pub fn name(self) -> &'static str {
        ""
    }

    /// The status by name, in any case.
    pub fn parse(text: &str) -> Option<Status> {
        None
    }
}

/// One entry file, kept as its lines so that every byte that is not changed stays as it was.
#[derive(Debug, Clone)]
pub struct Entry {
    /// Every line, with its own line ending.
    pub lines: Vec<String>,
}

impl Entry {
    /// Parses an entry file. Code::Parse with the line number when it is malformed.
    pub fn parse(text: &str) -> Result<Entry> {
        Err(Error::new(Code::NotImplemented, "entry::Entry::parse"))
    }

    /// The lines joined; equals the text that was parsed.
    pub fn text(&self) -> String {
        String::new()
    }

    /// The ID of the entry.
    pub fn id(&self) -> String {
        String::new()
    }

    /// The title of the entry.
    pub fn title(&self) -> String {
        String::new()
    }

    /// The value of the first line with that key.
    pub fn field(&self, key: &str) -> Option<String> {
        None
    }

    /// Every field in file order, history included.
    pub fn fields(&self) -> Vec<(String, String)> {
        Vec::new()
    }

    /// A comma-separated field, trimmed.
    pub fn list(&self, key: &str) -> Vec<String> {
        Vec::new()
    }

    /// The body of a section (format.md §3).
    pub fn section(&self, name: &str) -> Option<String> {
        None
    }

    /// Every section, in file order.
    pub fn sections(&self) -> Vec<(String, String)> {
        Vec::new()
    }

    /// The kind of the entry.
    pub fn kind(&self) -> Option<Kind> {
        None
    }

    /// The status of the entry.
    pub fn status(&self) -> Option<Status> {
        None
    }

    /// The values of the history lines, in file order.
    pub fn history(&self) -> Vec<String> {
        Vec::new()
    }
}
