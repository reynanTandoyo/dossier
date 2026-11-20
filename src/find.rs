//! Finding the DOSSIER folder of a project.

use crate::error::{Code, Error, Result};
use std::path::{Path, PathBuf};

/// The DOSSIER folder: `explicit` if given, else the first `DOSSIER` folder found walking up
/// from `start`. Missing: Code::NoDossier, message names `dossier init`.
pub fn find(start: &Path, explicit: Option<&Path>) -> Result<PathBuf> {
    Err(Error::new(Code::NotImplemented, "find::find"))
}
