//! Taken files of the dossier (format.md §30, R-FILE-1 to R-FILE-7): `taken.md`, one block per file
//! that was ever taken, keyed by its stored path. A block with a `taken` field is a file taken now;
//! its history lines are the log of the file's versions.

use std::path::Path;

use crate::blockfile::BlockFile;
use crate::error::{Code, Error, Result};

/// The file of a dossier that holds the taken files, `taken.md`.
pub const FILE: &str = "taken.md";

/// The item ID of the taken files (format.md §30).
pub const ITEM: &str = "TAKEN";

/// The words between the time and the person of a `taken` field: `<time> by <person>`.
const BY: &str = " by ";

/// The taken files file of the dossier and its raw bytes, when it exists. A missing file is
/// BlockFile::empty("Taken files of <code>") and None.
pub fn load(dossier: &Path, code: &str) -> Result<(BlockFile, Option<Vec<u8>>)> {
    let bytes = match std::fs::read(dossier.join(FILE)) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok((BlockFile::empty(&format!("Taken files of {code}")), None));
        }
        Err(error) => return Err(error.into()),
    };
    let text = std::str::from_utf8(&bytes)
        .map_err(|_| Error::new(Code::Parse, format!("{FILE}: not UTF-8")))?;
    let file = BlockFile::parse(text)
        .map_err(|error| Error::new(error.code, format!("{FILE}: {}", error.message)))?;
    Ok((file, Some(bytes)))
}

/// A file that is taken now, with who took it and the state of the file when it was taken
/// (format.md §30).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Take {
    pub path: String,
    pub person: String,
    pub time: String,
    pub task: String,
    pub size: u64,
    pub modified: String,
    pub checksum: String,
    pub forced: bool,
}

/// The take of a path, None when the path is not taken now: its block has no `taken` field. A
/// `size` that is missing or not a number reads as 0. A `taken` value without ` by ` is all time.
pub fn current(file: &BlockFile, path: &str) -> Option<Take> {
    let taken = file.field(path, "taken").filter(|text| !text.is_empty())?;
    let (time, person) = match taken.split_once(BY) {
        Some((time, person)) => (time.to_string(), person.to_string()),
        None => (taken.to_string(), String::new()),
    };
    let path = path.trim();
    Some(Take {
        path: path.to_string(),
        person,
        time,
        task: field_text(file, path, "task"),
        size: file
            .field(path, "size")
            .and_then(|text| text.parse().ok())
            .unwrap_or(0),
        modified: field_text(file, path, "modified"),
        checksum: field_text(file, path, "checksum"),
        forced: file.field(path, "forced").is_some_and(|text| text == "yes"),
    })
}

/// Every file taken now, in file order. A key that occurs twice counts once, from its first
/// block, as `BlockFile::field` reads it.
pub fn all(file: &BlockFile) -> Vec<Take> {
    let mut paths: Vec<String> = Vec::new();
    for key in file.keys() {
        if !paths.contains(&key) {
            paths.push(key);
        }
    }
    paths
        .iter()
        .filter_map(|path| current(file, path))
        .collect()
}

/// The history lines of a path: every take, return and abandon (R-FILE-5), in file order. Empty
/// when the path has no block.
pub fn log(file: &BlockFile, path: &str) -> Vec<String> {
    file.history(path)
}

/// The value of a field of a block, empty when the block or the field has none.
fn field_text(file: &BlockFile, path: &str, name: &str) -> String {
    file.field(path, name).unwrap_or_default()
}

/// The stored paths of the typed paths, in order, each once (see `stored_path`). `done` is the
/// past form of the command (`taken` or `returned`) for its messages.
pub fn stored_paths(
    ctx: &crate::ctx::Ctx,
    cwd: &Path,
    typed: &[String],
    done: &str,
) -> Result<Vec<String>> {
    let mut stored: Vec<String> = Vec::new();
    for path in typed {
        let path = stored_path(ctx, cwd, path, done)?;
        if !stored.contains(&path) {
            stored.push(path);
        }
    }
    Ok(stored)
}

/// A typed path as stored (R-XOS-2). The project folder itself cannot be taken or returned, and a
/// path that begins or ends with a space or has a line break cannot be a block key
/// (Code::BadValue). `done` is the past form of the command (`taken` or `returned`).
pub fn stored_path(ctx: &crate::ctx::Ctx, cwd: &Path, typed: &str, done: &str) -> Result<String> {
    let stored = crate::paths::stored(&ctx.root, cwd, typed)?;
    if stored.is_empty() {
        return Err(Error::new(
            Code::BadValue,
            format!("the project folder itself cannot be {done}"),
        ));
    }
    if stored.trim() != stored || stored.contains(['\r', '\n']) {
        return Err(Error::new(
            Code::BadValue,
            format!("a path cannot begin or end with a space or have a line break: {typed}"),
        ));
    }
    Ok(stored)
}

/// Refuses a stored path that is not a regular file: Code::NotFound when it does not exist,
/// Code::BadValue for a folder. The path is named in the message; `command` (`take` or
/// `return`) names the command in it.
pub fn ensure_file(root: &Path, stored: &str, command: &str) -> Result<()> {
    let full = root.join(stored);
    let metadata = std::fs::metadata(&full).map_err(|error| match error.kind() {
        std::io::ErrorKind::NotFound => Error::new(
            Code::NotFound,
            format!("{} does not exist", crate::paths::shown(stored)),
        ),
        _ => Error::new(Code::Io, format!("{}: {error}", full.display())),
    })?;
    if metadata.is_file() {
        return Ok(());
    }
    Err(Error::new(
        Code::BadValue,
        format!(
            "{} is not a file; {command} takes files",
            crate::paths::shown(stored)
        ),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A field pair as `add_block` takes it.
    fn pair(name: &str, value: &str) -> (String, String) {
        (name.to_string(), value.to_string())
    }

    /// The taken files of P-017 with invented data: `runs/case-a.dat` is taken by anna for a task,
    /// `runs/case-b.dat` was taken and returned, `runs/case-c.dat` is taken by ben with `--force`
    /// and has no size. Built with add_block and set_field, as the commands build it.
    fn sample() -> BlockFile {
        let mut file = BlockFile::empty("Taken files of P-017");

        file.add_block(
            "runs/case-a.dat",
            &[
                pair("task", "design.massflow"),
                pair("size", "4200000000"),
                pair("modified", "2026-11-19T18:30:00+01:00"),
                pair("checksum", "sha256-a1"),
                pair(
                    "history",
                    "2026-11-20T10:00:00+01:00 anna: taken — design.massflow sha256-a1",
                ),
            ],
        );
        file.set_field(
            "runs/case-a.dat",
            "taken",
            "2026-11-20T10:00:00+01:00 by anna",
        );

        file.add_block(
            "runs/case-b.dat",
            &[
                pair("task", "design.gas"),
                pair("size", "800"),
                pair("modified", "2026-11-18T09:00:00+01:00"),
                pair("checksum", "sha256-b2"),
                pair(
                    "history",
                    "2026-11-18T09:00:00+01:00 anna: taken — design.gas sha256-b1",
                ),
                pair(
                    "history",
                    "2026-11-19T09:00:00+01:00 anna: returned — sha256-b2",
                ),
            ],
        );

        file.add_block(
            "runs/case-c.dat",
            &[
                pair("task", "design.speed"),
                pair("modified", "2026-11-21T07:00:00+01:00"),
                pair("checksum", "sha256-c1"),
                pair(
                    "history",
                    "2026-11-21T11:00:00+01:00 ben: taken by force — design.speed sha256-c1, from anna",
                ),
            ],
        );
        file.set_field("runs/case-c.dat", "forced", "yes");
        file.set_field(
            "runs/case-c.dat",
            "taken",
            "2026-11-21T11:00:00+01:00 by ben",
        );
        file
    }

    #[test]
    fn a_block_with_taken_is_current_and_one_without_is_not() {
        let file = sample();
        let take = current(&file, "runs/case-a.dat").expect("case-a is taken");
        assert_eq!(
            take,
            Take {
                path: "runs/case-a.dat".to_string(),
                person: "anna".to_string(),
                time: "2026-11-20T10:00:00+01:00".to_string(),
                task: "design.massflow".to_string(),
                size: 4_200_000_000,
                modified: "2026-11-19T18:30:00+01:00".to_string(),
                checksum: "sha256-a1".to_string(),
                forced: false,
            }
        );
        assert_eq!(current(&file, "runs/case-b.dat"), None, "returned");
        assert_eq!(current(&file, "runs/missing.dat"), None, "never taken");
    }

    #[test]
    fn a_forced_take_is_marked_and_a_missing_size_reads_as_zero() {
        let file = sample();
        let take = current(&file, "runs/case-c.dat").expect("case-c is taken");
        assert_eq!(take.person, "ben");
        assert_eq!(take.time, "2026-11-21T11:00:00+01:00");
        assert!(take.forced);
        assert_eq!(take.size, 0);
    }

    #[test]
    fn all_lists_only_the_taken_files_in_file_order() {
        let paths: Vec<String> = all(&sample()).into_iter().map(|take| take.path).collect();
        assert_eq!(paths, ["runs/case-a.dat", "runs/case-c.dat"]);
    }

    #[test]
    fn log_returns_the_history_lines_of_a_path_in_order() {
        let file = sample();
        assert_eq!(
            log(&file, "runs/case-b.dat"),
            [
                "2026-11-18T09:00:00+01:00 anna: taken — design.gas sha256-b1",
                "2026-11-19T09:00:00+01:00 anna: returned — sha256-b2",
            ]
        );
        assert!(log(&file, "runs/missing.dat").is_empty());
    }

    #[test]
    fn a_path_with_spaces_and_non_ascii_letters_is_taken_and_read_like_any_other() {
        let path = "Läufe/Lauf 01 — Entwurf.dat";
        let mut file = BlockFile::empty("Taken files of P-017");
        file.add_block(
            path,
            &[pair("task", "design.gas"), pair("checksum", "sha256-d1")],
        );
        file.set_field(path, "taken", "2026-11-23T08:00:00+01:00 by clara");

        let take = current(&file, path).expect("the path is taken");
        assert_eq!(take.path, path);
        assert_eq!(take.person, "clara");
        assert_eq!(take.task, "design.gas");
        assert_eq!(all(&file).len(), 1);
        assert_eq!(current(&file, "Läufe/Lauf 01.dat"), None);
    }

    #[test]
    fn load_of_a_missing_file_is_an_empty_file_and_no_bytes() {
        let dir = tempfile::tempdir().expect("a temporary folder");
        let (file, bytes) = load(dir.path(), "P-017").expect("loads");
        assert_eq!(file.text(), BlockFile::empty("Taken files of P-017").text());
        assert!(bytes.is_none());
    }

    #[test]
    fn load_reads_the_file_and_keeps_its_bytes() {
        let dir = tempfile::tempdir().expect("a temporary folder");
        let text = sample().text();
        std::fs::write(dir.path().join(FILE), &text).expect("writes the file");
        let (file, bytes) = load(dir.path(), "P-017").expect("loads");
        assert_eq!(file.text(), text);
        assert_eq!(bytes.as_deref(), Some(text.as_bytes()));
    }

    #[test]
    fn load_refuses_a_file_that_is_not_utf_8() {
        let dir = tempfile::tempdir().expect("a temporary folder");
        std::fs::write(dir.path().join(FILE), [0xff, 0xfe, b'#']).expect("writes the file");
        let error = load(dir.path(), "P-017").expect_err("not UTF-8");
        assert_eq!(error.code, Code::Parse);
    }
}
