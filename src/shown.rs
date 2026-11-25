//! Entries shown in a session: the entries that `dossier guard` has already shown to the LLM of a
//! Claude Code session, so that a later edit does not repeat them (R-CODE-8, D51).
//!
//! The folder `<state>/shown/` holds one file per session, `<session>.txt`, with one ID per line
//! (format.md section 36).

use crate::atomic;
use crate::error::{Code, Error, Result};
use std::collections::HashSet;
use std::path::Path;
use std::time::{Duration, SystemTime};

/// The folder in the state folder that holds one file per session.
const FOLDER: &str = "shown";

/// Files in the folder that were last changed more than this long ago are removed.
const MAX_AGE: Duration = Duration::from_secs(30 * 24 * 60 * 60);

/// The IDs of `ids` that the session has not been shown yet, in their order, recorded as shown; an empty or unusual session name gives all of `ids` and writes nothing.
pub fn unseen(state: &Path, session: &str, ids: &[String]) -> Result<Vec<String>> {
    if !valid_session(session) {
        return Ok(ids.to_vec());
    }

    let now = SystemTime::now();
    let folder = state.join(FOLDER);
    let file = folder.join(format!("{session}.txt"));

    let text = read_text(&file)?;
    let shown: HashSet<&str> = text
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect();
    let fresh: Vec<String> = ids
        .iter()
        .filter(|id| !shown.contains(id.as_str()))
        .cloned()
        .collect();

    if !fresh.is_empty() {
        let mut content = text;
        if !content.is_empty() && !content.ends_with('\n') {
            content.push('\n');
        }
        for id in &fresh {
            content.push_str(id);
            content.push('\n');
        }
        atomic::replace(&file, content.as_bytes(), None)?;
    }

    remove_old(&folder, &file, now);
    Ok(fresh)
}

/// True for a name of ASCII letters, digits and `-` only, so that it is safe as a file name.
fn valid_session(session: &str) -> bool {
    !session.is_empty()
        && session
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || character == '-')
}

/// The text of `file`; empty when the file does not exist yet.
fn read_text(file: &Path) -> Result<String> {
    match std::fs::read_to_string(file) {
        Ok(text) => Ok(text),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(String::new()),
        Err(error) => Err(io_error(file, error)),
    }
}

/// Removes the files in `folder` other than `keep` that were last changed more than `MAX_AGE`
/// before `now`. A failure here is ignored: the files are only cleaned up, never needed.
fn remove_old(folder: &Path, keep: &Path, now: SystemTime) {
    let Ok(entries) = std::fs::read_dir(folder) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path == keep {
            continue;
        }
        let Ok(metadata) = entry.metadata() else {
            continue;
        };
        if !metadata.is_file() {
            continue;
        }
        let Ok(modified) = metadata.modified() else {
            continue;
        };
        if now.duration_since(modified).is_ok_and(|age| age > MAX_AGE) {
            let _ = std::fs::remove_file(&path);
        }
    }
}

/// A failure on `path` as Code::Io, with the path in the message.
fn io_error(path: &Path, error: std::io::Error) -> Error {
    Error::new(Code::Io, format!("{}: {error}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;

    const DAY: u64 = 24 * 60 * 60;

    fn ids(list: &[&str]) -> Vec<String> {
        list.iter().map(|id| id.to_string()).collect()
    }

    #[test]
    fn the_first_call_returns_both_ids_and_records_them() {
        let state = tempfile::tempdir().expect("temporary folder");

        let fresh = unseen(state.path(), "s1", &ids(&["E-1", "E-2"])).expect("unseen");

        assert_eq!(fresh, ids(&["E-1", "E-2"]));
        let file = state.path().join("shown").join("s1.txt");
        assert_eq!(std::fs::read_to_string(file).expect("read"), "E-1\nE-2\n");
    }

    #[test]
    fn a_second_call_returns_only_the_id_that_is_new() {
        let state = tempfile::tempdir().expect("temporary folder");
        unseen(state.path(), "s1", &ids(&["E-1", "E-2"])).expect("first call");

        let fresh = unseen(state.path(), "s1", &ids(&["E-1", "E-3"])).expect("second call");

        assert_eq!(fresh, ids(&["E-3"]));
        let file = state.path().join("shown").join("s1.txt");
        assert_eq!(
            std::fs::read_to_string(file).expect("read"),
            "E-1\nE-2\nE-3\n"
        );
    }

    #[test]
    fn another_session_starts_empty() {
        let state = tempfile::tempdir().expect("temporary folder");
        unseen(state.path(), "s1", &ids(&["E-1"])).expect("first session");

        let fresh = unseen(state.path(), "s2", &ids(&["E-1"])).expect("other session");

        assert_eq!(fresh, ids(&["E-1"]));
    }

    #[test]
    fn an_unusual_session_name_gives_everything_and_creates_no_file() {
        let state = tempfile::tempdir().expect("temporary folder");

        for session in ["", "a/b", "a b", "a.b", "\u{fc}ber"] {
            let fresh = unseen(state.path(), session, &ids(&["E-1", "E-2"])).expect("unseen");
            assert_eq!(fresh, ids(&["E-1", "E-2"]), "session {session:?}");
        }

        assert!(!state.path().join("shown").exists());
    }

    #[test]
    fn a_file_older_than_30_days_is_removed_by_the_next_call_for_another_session() {
        let state = tempfile::tempdir().expect("temporary folder");
        let folder = state.path().join("shown");
        unseen(state.path(), "old", &ids(&["E-1"])).expect("old session");
        unseen(state.path(), "recent", &ids(&["E-1"])).expect("recent session");
        let now = SystemTime::now();
        set_age(&folder.join("old.txt"), now - Duration::from_secs(31 * DAY));
        set_age(
            &folder.join("recent.txt"),
            now - Duration::from_secs(29 * DAY),
        );

        unseen(state.path(), "new", &ids(&["E-1"])).expect("new session");

        assert!(!folder.join("old.txt").exists());
        assert!(folder.join("recent.txt").exists());
    }

    fn set_age(file: &Path, when: SystemTime) {
        let file = File::options()
            .write(true)
            .open(file)
            .expect("open test file");
        file.set_modified(when).expect("set modified time");
    }
}
