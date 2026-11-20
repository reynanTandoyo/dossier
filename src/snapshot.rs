//! Snapshots: the earlier bytes of an entry, kept before each change.
//!
//! The snapshots of entry `DEC-012` are `state/history/DEC-012/000001.md`, `000002.md` and so on,
//! and `index.jsonl` in the same folder has one line per snapshot (format.md §9).

use crate::atomic;
use crate::error::{Code, Error, Result};
use std::io::Write;
use std::path::{Path, PathBuf};

/// One saved snapshot of an entry: its number, when it was taken, by whom and by which command.
pub struct Snap {
    pub n: u32,
    pub time: String,
    pub name: String,
    pub command: String,
}

/// The name of the index file in the history folder of an entry.
const INDEX: &str = "index.jsonl";

/// One line of the index file: `{"n":1,"time":"…","name":"…","command":"…"}`.
#[derive(serde::Serialize, serde::Deserialize)]
struct IndexLine {
    n: u32,
    time: String,
    name: String,
    command: String,
}

/// Saves `bytes` as the next snapshot of entry `id`; returns its number.
pub fn save(
    state: &Path,
    id: &str,
    bytes: &[u8],
    time: &str,
    name: &str,
    command: &str,
) -> Result<u32> {
    let highest = list(state, id)?
        .iter()
        .map(|snap| snap.n)
        .max()
        .unwrap_or(0);
    let n = highest
        .checked_add(1)
        .ok_or_else(|| Error::new(Code::Refused, format!("{id} has no snapshot number left")))?;

    atomic::replace(&file(state, id, n), bytes, None)?;

    let line = IndexLine {
        n,
        time: time.to_string(),
        name: name.to_string(),
        command: command.to_string(),
    };
    append_index(state, id, &line)?;
    Ok(n)
}

/// The snapshots of entry `id`, oldest first. A missing index gives an empty list; index lines
/// that do not parse are skipped.
pub fn list(state: &Path, id: &str) -> Result<Vec<Snap>> {
    let path = folder(state, id).join(INDEX);
    let bytes = match std::fs::read(&path) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(failure(&path, error)),
    };
    Ok(bytes
        .split(|byte| *byte == b'\n')
        .map(|line| line.trim_ascii())
        .filter(|line| !line.is_empty())
        .filter_map(|line| serde_json::from_slice::<IndexLine>(line).ok())
        .map(|line| Snap {
            n: line.n,
            time: line.time,
            name: line.name,
            command: line.command,
        })
        .collect())
}

/// The bytes of snapshot `n` of entry `id`.
pub fn load(state: &Path, id: &str, n: u32) -> Result<Vec<u8>> {
    let path = file(state, id, n);
    match std::fs::read(&path) {
        Ok(bytes) => Ok(bytes),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Err(Error::new(
            Code::NotFound,
            format!("no snapshot {n} of {id}"),
        )),
        Err(error) => Err(failure(&path, error)),
    }
}

/// The history folder of entry `id`: `state/history/<ID>`, with the ID in upper case.
fn folder(state: &Path, id: &str) -> PathBuf {
    state.join("history").join(id.to_ascii_uppercase())
}

/// The file of snapshot `n` of entry `id`: `<n as six digits>.md` in its folder.
fn file(state: &Path, id: &str, n: u32) -> PathBuf {
    folder(state, id).join(format!("{n:06}.md"))
}

/// Appends one index line in a single write, creating the folder and the index when missing.
fn append_index(state: &Path, id: &str, line: &IndexLine) -> Result<()> {
    let folder = folder(state, id);
    std::fs::create_dir_all(&folder).map_err(|error| failure(&folder, error))?;
    let path = folder.join(INDEX);
    let mut text = serde_json::to_string(line).map_err(|error| failure(&path, error))?;
    text.push('\n');
    let mut index = std::fs::OpenOptions::new()
        .append(true)
        .create(true)
        .open(&path)
        .map_err(|error| failure(&path, error))?;
    index
        .write_all(text.as_bytes())
        .map_err(|error| failure(&path, error))
}

/// A failure on `path` as Code::Io, with the path in the message.
fn failure(path: &Path, error: impl std::fmt::Display) -> Error {
    Error::new(Code::Io, format!("{}: {error}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    const WHEN: &str = "2026-11-21T10:15:00+01:00";

    /// A state folder whose path has a space and non-ASCII letters; it does not exist yet.
    fn state_in(dir: &tempfile::TempDir) -> PathBuf {
        dir.path().join("Projekt Übersicht").join(".state")
    }

    #[test]
    fn two_saves_of_one_entry_get_the_numbers_1_and_2() {
        let dir = tempfile::tempdir().unwrap();
        let state = state_in(&dir);
        assert_eq!(
            save(&state, "DEC-012", b"first", WHEN, "anna", "decide").unwrap(),
            1
        );
        assert_eq!(
            save(&state, "DEC-012", b"second", WHEN, "anna", "set").unwrap(),
            2
        );
    }

    #[test]
    fn list_returns_both_snapshots_with_their_data_oldest_first() {
        let dir = tempfile::tempdir().unwrap();
        let state = state_in(&dir);
        save(&state, "DEC-012", b"first", WHEN, "anna", "decide").unwrap();
        save(
            &state,
            "DEC-012",
            b"second",
            "2026-11-21T11:00:00+01:00",
            "ben-codex",
            "set",
        )
        .unwrap();
        let snaps = list(&state, "DEC-012").unwrap();
        assert_eq!(snaps.len(), 2);
        assert_eq!(snaps[0].n, 1);
        assert_eq!(snaps[0].time, WHEN);
        assert_eq!(snaps[0].name, "anna");
        assert_eq!(snaps[0].command, "decide");
        assert_eq!(snaps[1].n, 2);
        assert_eq!(snaps[1].time, "2026-11-21T11:00:00+01:00");
        assert_eq!(snaps[1].name, "ben-codex");
        assert_eq!(snaps[1].command, "set");
    }

    #[test]
    fn load_returns_the_bytes_of_each_snapshot() {
        let dir = tempfile::tempdir().unwrap();
        let state = state_in(&dir);
        save(&state, "DEC-012", b"first", WHEN, "anna", "decide").unwrap();
        save(&state, "DEC-012", b"second", WHEN, "anna", "set").unwrap();
        assert_eq!(load(&state, "DEC-012", 1).unwrap(), b"first");
        assert_eq!(load(&state, "DEC-012", 2).unwrap(), b"second");
    }

    #[test]
    fn load_of_a_missing_number_is_not_found_with_its_message() {
        let dir = tempfile::tempdir().unwrap();
        let state = state_in(&dir);
        save(&state, "DEC-012", b"first", WHEN, "anna", "decide").unwrap();
        let error = load(&state, "DEC-012", 9).unwrap_err();
        assert_eq!(error.code, Code::NotFound);
        assert_eq!(error.exit(), 1);
        assert_eq!(error.message, "no snapshot 9 of DEC-012");
    }

    #[test]
    fn the_folder_and_the_numbering_use_the_upper_case_id() {
        let dir = tempfile::tempdir().unwrap();
        let state = state_in(&dir);
        assert_eq!(
            save(&state, "dec-012", b"first", WHEN, "anna", "decide").unwrap(),
            1
        );
        let file = state.join("history").join("DEC-012").join("000001.md");
        assert_eq!(std::fs::read(&file).unwrap(), b"first");
        assert_eq!(list(&state, "DEC-012").unwrap().len(), 1);
        assert_eq!(load(&state, "Dec-012", 1).unwrap(), b"first");
    }

    #[test]
    fn each_entry_has_its_own_numbering() {
        let dir = tempfile::tempdir().unwrap();
        let state = state_in(&dir);
        assert_eq!(
            save(&state, "DEC-012", b"a", WHEN, "anna", "decide").unwrap(),
            1
        );
        assert_eq!(
            save(&state, "ASM-003", b"b", WHEN, "anna", "assume").unwrap(),
            1
        );
        assert_eq!(list(&state, "ASM-003").unwrap().len(), 1);
        assert_eq!(load(&state, "ASM-003", 1).unwrap(), b"b");
    }

    #[test]
    fn the_index_line_is_one_json_object_per_snapshot() {
        let dir = tempfile::tempdir().unwrap();
        let state = state_in(&dir);
        save(&state, "DEC-012", b"first", WHEN, "anna", "decide").unwrap();
        let index =
            std::fs::read_to_string(state.join("history").join("DEC-012").join("index.jsonl"))
                .unwrap();
        assert_eq!(
            index,
            "{\"n\":1,\"time\":\"2026-11-21T10:15:00+01:00\",\"name\":\"anna\",\
             \"command\":\"decide\"}\n"
        );
    }

    #[test]
    fn an_entry_without_snapshots_lists_nothing() {
        let dir = tempfile::tempdir().unwrap();
        let state = state_in(&dir);
        assert!(list(&state, "DEC-012").unwrap().is_empty());
    }

    #[test]
    fn a_garbage_index_line_is_skipped_and_the_next_number_follows_the_good_ones() {
        let dir = tempfile::tempdir().unwrap();
        let state = state_in(&dir);
        save(&state, "DEC-012", b"first", WHEN, "anna", "decide").unwrap();
        let index = state.join("history").join("DEC-012").join("index.jsonl");
        let mut file = std::fs::OpenOptions::new()
            .append(true)
            .open(&index)
            .unwrap();
        file.write_all(b"this is not json\n\n").unwrap();
        drop(file);
        assert_eq!(
            save(&state, "DEC-012", b"second", WHEN, "anna", "set").unwrap(),
            2
        );
        let snaps = list(&state, "DEC-012").unwrap();
        assert_eq!(snaps.len(), 2);
        assert_eq!(snaps[1].n, 2);
    }
}
