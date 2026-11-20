//! The register: the projects this user works on, and where each one is kept (format.md §12).

use crate::error::{Code, Error, Result};
use std::path::PathBuf;

/// One project of the register.
pub struct Project {
    pub code: String,
    pub server: String,
    pub folder: String,
    pub local: Option<String>,
}

/// The register file (format.md §12). `DOSSIER_REGISTER` overrides it.
pub fn file() -> Result<PathBuf> {
    Err(Error::new(Code::NotImplemented, "remote::register::file"))
}

/// Every project of the register.
pub fn load() -> Result<Vec<Project>> {
    Err(Error::new(Code::NotImplemented, "remote::register::load"))
}

/// Replaces the project with the same code, or adds it.
pub fn put(project: &Project) -> Result<()> {
    Err(Error::new(Code::NotImplemented, "remote::register::put"))
}

/// Removes the project with `code`. True when there was one.
pub fn remove(code: &str) -> Result<bool> {
    Err(Error::new(Code::NotImplemented, "remote::register::remove"))
}
