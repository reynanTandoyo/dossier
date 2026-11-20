//! The context of a command: the dossier it works on, its settings, and who acts.

use crate::cli::Global;
use crate::config::Config;
use crate::error::{Code, Error, Result};
use crate::names::Who;
use std::path::PathBuf;

/// The context of one command.
pub struct Ctx {
    /// The parent of `dossier`: the project folder.
    pub root: PathBuf,
    /// The DOSSIER folder.
    pub dossier: PathBuf,
    /// `dossier/.state`.
    pub state: PathBuf,
    pub cfg: Config,
    pub who: Option<Who>,
    pub json: bool,
    pub dry_run: bool,
    pub command: String,
}

impl Ctx {
    /// Finds the dossier from the current folder (or g.dossier), loads the settings, and
    /// resolves the name if one is given (no name is not an error here).
    pub fn load(g: &Global, command: &str) -> Result<Ctx> {
        Err(Error::new(Code::NotImplemented, "ctx::load"))
    }

    /// The name that acts. Code::Usage "this command needs --agent" when there is none.
    pub fn writer(&self) -> Result<&Who> {
        Err(Error::new(Code::NotImplemented, "ctx::writer"))
    }
}
