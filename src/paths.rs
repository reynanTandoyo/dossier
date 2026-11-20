//! Paths: the slug of a title, names that Windows accepts, and stored and shown paths.

use crate::error::{Code, Error, Result};
use std::path::Path;

/// The file-name part of a title (format.md §4).
pub fn slug(title: &str) -> String {
    String::new()
}

/// Whether `name` is allowed on Windows (R-XOS-4, R-XOS-5).
pub fn windows_safe(name: &str) -> bool {
    false
}

/// A typed path (with / or \, relative to cwd or absolute) as a path relative to root, with /.
/// Outside root: Code::BadValue.
pub fn stored(root: &Path, cwd: &Path, typed: &str) -> Result<String> {
    Err(Error::new(Code::NotImplemented, "paths::stored"))
}

/// A stored path with the separator of this system.
pub fn shown(stored: &str) -> String {
    String::new()
}
