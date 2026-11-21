//! The files of a task (format.md §17, files/<task>.md): one block per attached path.

use crate::blockfile::BlockFile;
use crate::config::Config;
use crate::error::{Code, Error, Result};
use std::path::{Path, PathBuf};

/// The roles a file of a task can have (format.md §17); the settings may add more.
pub const ROLES: [&str; 10] = [
    "description",
    "geometry",
    "mesh",
    "model",
    "setup",
    "script",
    "result",
    "report",
    "reference",
    "other",
];

/// The item name of a task's file list in snapshots and the journal: "FILES-<TASK>", with the
/// task upper-cased.
pub fn item(_task: &str) -> String {
    String::new()
}

/// The file of a task's list: dossier/files/<TASK>.md.
pub fn path(_dossier: &Path, _task: &str) -> PathBuf {
    PathBuf::new()
}

/// The list of a task and its bytes as read; a missing file is BlockFile::empty("Files of
/// <TASK>"), None.
pub fn load(_dossier: &Path, _task: &str) -> Result<(BlockFile, Option<Vec<u8>>)> {
    Err(Error::new(Code::NotImplemented, "filelist::load"))
}

/// The tasks that have a list, sorted.
pub fn tasks(_dossier: &Path) -> Vec<String> {
    Vec::new()
}

/// Whether a role is one of ROLES or of the settings' roles, ignoring case.
pub fn valid_role(_cfg: &Config, _role: &str) -> bool {
    false
}

/// The fields describing `stored` now: type, size or files, modified, checksum.
pub fn describe(_root: &Path, _stored: &str) -> Result<Vec<(String, String)>> {
    Err(Error::new(Code::NotImplemented, "filelist::describe"))
}

/// Marks of a block: "detached", "missing", "changed since attached", "outdated" (a made-from
/// checksum differs from that file now), "rests on a wrong assumption".
pub fn marks(_root: &Path, _file: &BlockFile, _key: &str) -> Vec<String> {
    Vec::new()
}
