//! The settings of a project: dossier.toml (format.md §10).

use crate::error::{Code, Error, Result};
use std::path::Path;

/// A person of the project, and the LLMs that write for them.
pub struct Person {
    pub name: String,
    pub llms: Vec<String>,
}

/// The tag values the project allows.
pub struct Tags {
    pub machine: Vec<String>,
    pub analysis: Vec<String>,
    pub software: Vec<String>,
}

/// The settings of a project.
pub struct Config {
    pub format: u32,
    pub code: String,
    pub tags: Tags,
    pub people: Vec<Person>,
    pub brief_lines: usize,
    pub status_lines: usize,
    pub draft_days: i64,
    pub lock_wait_seconds: u64,
    pub ssh_timeout_seconds: u64,
}

/// Reads the settings of `dossier`. An unknown key: Code::Config.
pub fn load(dossier: &Path) -> Result<Config> {
    Err(Error::new(Code::NotImplemented, "config::load"))
}

/// The text that `init` writes for a project with `code`.
pub fn default_text(code: &str) -> String {
    String::new()
}

/// Adds a person, or extends a person's LLMs, and keeps the other lines.
pub fn add_person(dossier: &Path, person: &str, llms: &[String]) -> Result<()> {
    Err(Error::new(Code::NotImplemented, "config::add_person"))
}
