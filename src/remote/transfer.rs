//! Copies of large files between a connected PC folder and the server (R-REMOTE-4): the scp
//! arguments, and `take` and `return` in a connected folder.

use crate::remote::pointer::Pointer;
use std::ffi::OsString;
use std::path::Path;

/// The scp program: DOSSIER_SCP, else `scp`.
pub fn program() -> OsString {
    OsString::from("scp")
}

/// The arguments of scp: -o BatchMode=yes -o ConnectTimeout=<n> -p, then source and target.
pub fn argv(_source: &str, _target: &str, _timeout: u64) -> Vec<String> {
    Vec::new()
}

/// "<server>:<quoted folder/stored>" for scp.
pub fn remote_path(_server: &str, _folder: &str, _stored: &str) -> String {
    String::new()
}

/// Called by forward::maybe for `take` and `return` when the folder is connected; Some(exit code).
pub fn maybe(
    _command: &str,
    _args: &[String],
    _base: &Path,
    _pointer: &Pointer,
    _agent: Option<&str>,
) -> Option<u8> {
    None
}
