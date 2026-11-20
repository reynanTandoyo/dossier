//! Atomic replacement of a file, checked against the bytes that were read.

use crate::error::{Code, Error, Result};
use std::io::Write;
use std::path::Path;

/// What a file looked like when it was read: its length and its checksum.
#[derive(Debug, Clone, PartialEq)]
pub struct Seen {
    pub len: u64,
    pub hash: String,
}

/// The length and checksum of the file at `path`; None when the file does not exist.
pub fn seen(path: &Path) -> Result<Option<Seen>> {
    match std::fs::read(path) {
        Ok(bytes) => Ok(Some(Seen {
            len: bytes.len() as u64,
            hash: crate::hash::sha256_hex(&bytes),
        })),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(io_error(path, error)),
    }
}

/// Writes `<name>.tmp-<pid>` next to the file, flushes, renames. With `expected`, first checks
/// that the file is still that (None = must not exist); otherwise Code::Changed.
pub fn replace(path: &Path, bytes: &[u8], expected: Option<&Option<Seen>>) -> Result<()> {
    if let Some(expected) = expected {
        let now = seen(path)?;
        if now != *expected {
            return Err(Error::new(
                Code::Changed,
                format!(
                    "{} was changed by someone else; run the command again",
                    path.display()
                ),
            ));
        }
    }

    if let Some(folder) = path
        .parent()
        .filter(|folder| !folder.as_os_str().is_empty())
    {
        std::fs::create_dir_all(folder).map_err(|error| io_error(folder, error))?;
    }

    let mut tmp_name = path
        .file_name()
        .ok_or_else(|| {
            Error::new(
                Code::Io,
                format!("{}: the path has no file name", path.display()),
            )
        })?
        .to_os_string();
    tmp_name.push(format!(".tmp-{}", std::process::id()));
    let tmp = path.with_file_name(tmp_name);

    let staged = write_file(&tmp, bytes).and_then(|()| rename_over(&tmp, path));
    if let Err(error) = staged {
        // Nothing replaced the file, so the temporary file is only litter. It may not exist
        // at all when creating it failed; that is fine.
        let _ = std::fs::remove_file(&tmp);
        return Err(io_error(path, error));
    }
    Ok(())
}

/// Creates `tmp`, writes `bytes` and flushes them to disk before the file is closed.
fn write_file(tmp: &Path, bytes: &[u8]) -> std::io::Result<()> {
    let mut file = std::fs::File::create(tmp)?;
    file.write_all(bytes)?;
    file.sync_all()
}

/// How many times a rename that failed with a permission error is retried on Windows, and the
/// pause before each retry. A file that another program has open can be renamed a moment later.
#[cfg(windows)]
const RENAME_RETRIES: u32 = 10;
#[cfg(windows)]
const RENAME_PAUSE: std::time::Duration = std::time::Duration::from_millis(50);

/// Renames `from` over `to`, which replaces `to` (std::fs::rename does this on Windows too).
#[cfg(windows)]
fn rename_over(from: &Path, to: &Path) -> std::io::Result<()> {
    let mut retries = 0;
    loop {
        match std::fs::rename(from, to) {
            Err(error)
                if error.kind() == std::io::ErrorKind::PermissionDenied
                    && retries < RENAME_RETRIES =>
            {
                retries += 1;
                std::thread::sleep(RENAME_PAUSE);
            }
            result => return result,
        }
    }
}

/// Renames `from` over `to`, which replaces `to` in one step.
#[cfg(not(windows))]
fn rename_over(from: &Path, to: &Path) -> std::io::Result<()> {
    std::fs::rename(from, to)
}

/// A failure on `path` as Code::Io, with the path in the message.
fn io_error(path: &Path, error: std::io::Error) -> Error {
    Error::new(Code::Io, format!("{}: {error}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The names of the entries of `folder`, sorted.
    fn names_in(folder: &Path) -> Vec<String> {
        let mut names: Vec<String> = std::fs::read_dir(folder)
            .unwrap()
            .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        names.sort();
        names
    }

    #[test]
    fn seen_is_none_for_a_missing_file() {
        let folder = tempfile::tempdir().unwrap();
        assert_eq!(seen(&folder.path().join("missing.md")).unwrap(), None);
    }

    #[test]
    fn seen_gives_the_length_and_the_sha256() {
        let folder = tempfile::tempdir().unwrap();
        let path = folder.path().join("entry.md");
        std::fs::write(&path, b"abc").unwrap();
        assert_eq!(
            seen(&path).unwrap(),
            Some(Seen {
                len: 3,
                hash: "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad".into(),
            })
        );
    }

    #[test]
    fn a_new_file_is_written() {
        let folder = tempfile::tempdir().unwrap();
        let path = folder.path().join("DEC-001 first.md");
        replace(&path, b"new text", Some(&None)).unwrap();
        assert_eq!(std::fs::read(&path).unwrap(), b"new text");
        assert_eq!(names_in(folder.path()), ["DEC-001 first.md"]);
    }

    #[test]
    fn a_new_file_is_written_without_a_check() {
        let folder = tempfile::tempdir().unwrap();
        let path = folder.path().join("entry.md");
        replace(&path, b"new text", None).unwrap();
        assert_eq!(std::fs::read(&path).unwrap(), b"new text");
    }

    #[test]
    fn a_missing_folder_is_created_with_non_ascii_names() {
        let folder = tempfile::tempdir().unwrap();
        let path = folder
            .path()
            .join("Projekt Übersicht")
            .join("DEC-002 Größe.md");
        replace(&path, b"x", Some(&None)).unwrap();
        assert_eq!(std::fs::read(&path).unwrap(), b"x");
    }

    #[test]
    fn an_existing_file_is_replaced() {
        let folder = tempfile::tempdir().unwrap();
        let path = folder.path().join("entry.md");
        std::fs::write(&path, b"old text").unwrap();
        let before = seen(&path).unwrap();
        replace(&path, b"new text", Some(&before)).unwrap();
        assert_eq!(std::fs::read(&path).unwrap(), b"new text");
        assert_eq!(names_in(folder.path()), ["entry.md"]);
    }

    #[test]
    fn expected_none_refuses_an_existing_file() {
        let folder = tempfile::tempdir().unwrap();
        let path = folder.path().join("entry.md");
        std::fs::write(&path, b"old text").unwrap();
        let error = replace(&path, b"new text", Some(&None)).unwrap_err();
        assert_eq!(error.code, Code::Changed);
        assert_eq!(
            error.message,
            format!(
                "{} was changed by someone else; run the command again",
                path.display()
            )
        );
        assert_eq!(std::fs::read(&path).unwrap(), b"old text");
        assert_eq!(names_in(folder.path()), ["entry.md"]);
    }

    #[test]
    fn a_changed_file_gives_changed_and_keeps_the_other_content() {
        let folder = tempfile::tempdir().unwrap();
        let path = folder.path().join("entry.md");
        std::fs::write(&path, b"old text").unwrap();
        let before = seen(&path).unwrap();
        std::fs::write(&path, b"written by someone else").unwrap();
        let error = replace(&path, b"new text", Some(&before)).unwrap_err();
        assert_eq!(error.code, Code::Changed);
        assert_eq!(error.exit(), 3);
        assert_eq!(std::fs::read(&path).unwrap(), b"written by someone else");
        assert_eq!(names_in(folder.path()), ["entry.md"]);
    }

    #[test]
    fn a_file_removed_since_it_was_read_is_not_recreated() {
        let folder = tempfile::tempdir().unwrap();
        let path = folder.path().join("entry.md");
        std::fs::write(&path, b"old text").unwrap();
        let before = seen(&path).unwrap();
        std::fs::remove_file(&path).unwrap();
        let error = replace(&path, b"new text", Some(&before)).unwrap_err();
        assert_eq!(error.code, Code::Changed);
        assert!(!path.exists());
        assert!(names_in(folder.path()).is_empty());
    }

    #[test]
    fn no_temporary_file_is_left_behind() {
        let folder = tempfile::tempdir().unwrap();
        let path = folder.path().join("entry.md");
        replace(&path, b"first", None).unwrap();
        let before = seen(&path).unwrap();
        replace(&path, b"second", Some(&before)).unwrap();
        assert_eq!(std::fs::read(&path).unwrap(), b"second");
        assert_eq!(names_in(folder.path()), ["entry.md"]);
    }

    #[test]
    fn a_failed_rename_is_an_io_error_and_removes_the_temporary_file() {
        // A folder sits where the file should be, so the rename over it fails.
        let folder = tempfile::tempdir().unwrap();
        let path = folder.path().join("entry.md");
        std::fs::create_dir(&path).unwrap();
        let error = replace(&path, b"new text", None).unwrap_err();
        assert_eq!(error.code, Code::Io);
        assert!(error.message.contains("entry.md"), "{}", error.message);
        assert_eq!(names_in(folder.path()), ["entry.md"]);
    }
}
