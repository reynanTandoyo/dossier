//! Commits the changes of a team dossier to git (D36).

use crate::error::{Code, Error, Result};
use std::path::Path;

/// Runs `git add -- <path>` and `git commit -m <message> -- <path>` in `repo`. Ok(false) when
/// `repo` is not a git work tree or there is nothing to commit; Code::Io when git fails.
pub fn commit(_repo: &Path, _path: &Path, _message: &str) -> Result<bool> {
    Err(Error::new(Code::NotImplemented, "gitcommit::commit"))
}
