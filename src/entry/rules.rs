//! What an entry needs before it can be approved.

use super::{Entry, Kind};

/// One thing that is missing or wrong.
pub struct Problem {
    pub what: String,
}

/// The sections that every entry of `kind` must have.
pub fn required_sections(kind: Kind) -> &'static [&'static str] {
    &[]
}

/// The order of the sections of an entry of `kind` (format.md §8).
pub fn section_order(kind: Kind) -> &'static [&'static str] {
    &[]
}

/// What is missing for approval: required sections, `confidence` for a lesson with one of the
/// four values, a valid `state` for an assumption, a valid `outcome` if present.
pub fn problems(entry: &Entry) -> Vec<Problem> {
    Vec::new()
}
