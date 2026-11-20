//! Who acts: the name of a person or an LLM, from `--agent` or the environment.

use crate::config::Config;
use crate::error::{Code, Error, Result};

/// The name of whoever acts, and the person it acts for.
#[derive(Debug, Clone, PartialEq)]
pub struct Who {
    pub name: String,
    pub person: String,
    pub is_llm: bool,
}

/// Name from `flag`, else DOSSIER_AGENT, else TODO_AGENT. None: Code::Usage.
/// Not a person and not an LLM in cfg: Code::UnknownName, message lists the known names.
pub fn resolve(cfg: &Config, flag: Option<&str>) -> Result<Who> {
    Err(Error::new(Code::NotImplemented, "names::resolve"))
}

/// Whether `name` is one of the people of cfg.
pub fn is_person(cfg: &Config, name: &str) -> bool {
    false
}

/// "anna" or "anna via anna-claude".
pub fn label(who: &Who) -> String {
    String::new()
}
