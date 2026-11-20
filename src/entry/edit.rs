//! Changes to an entry: its fields, title, sections and history.

use super::Entry;
use crate::names::Who;

impl Entry {
    /// Replaces the field's line, or adds it before the first history line (else at the end of
    /// the block). New lines take their line ending as format.md §2 says.
    pub fn set_field(&mut self, key: &str, value: &str) {}

    /// Removes the field's line.
    pub fn remove_field(&mut self, key: &str) {}

    /// Replaces the title.
    pub fn set_title(&mut self, title: &str) {}

    /// Replaces the body of a section, or adds the section at the end of the file.
    pub fn set_section(&mut self, name: &str, body: &str) {}

    /// Adds a history line.
    pub fn add_history(&mut self, time: &str, who: &Who, event: &str, detail: &str) {}
}
