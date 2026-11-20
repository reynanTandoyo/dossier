//! The write lock of a dossier, and the check for network drives.

use crate::error::{Code, Error, Result};
use std::path::{Path, PathBuf};

/// A held write lock. The lock is released when the guard is dropped.
pub struct Guard {
    file: std::fs::File,
    holder: PathBuf,
}

/// Locks `<state>/write.lock` with `File::try_lock`, retrying every 50 ms for `wait_seconds`,
/// then Code::Busy naming the holder from holder.json. Writes holder.json; Drop removes it.
pub fn acquire(state: &Path, wait_seconds: u64, name: &str, command: &str) -> Result<Guard> {
    Err(Error::new(Code::NotImplemented, "lock::acquire"))
}

/// Windows: UNC path or a drive of type DRIVE_REMOTE. Linux: the mount of the path in
/// /proc/mounts has type cifs, smb3, smbfs, nfs or nfs4.
pub fn is_network(path: &Path) -> bool {
    false
}
