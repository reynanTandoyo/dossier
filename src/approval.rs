//! Approval records: a checksum of each approved entry, so that a changed entry shows as changed.
//!
//! The checksum covers the file without its `- history:` lines and without the `status`,
//! `approved`, `valid-from` and `valid-until` lines.

use crate::atomic;
use crate::entry::Kind;
use crate::error::{Code, Error, Result};
use crate::hash;
use serde_json::{Map, Value};
use std::path::Path;

/// How an entry compares with the checksum recorded at approval.
pub enum Mark {
    Unchanged,
    Changed,
    NotRecorded,
}

/// The lines that do not count as a change.
const IGNORED_LINES: [&[u8]; 5] = [
    b"- history:",
    b"- status:",
    b"- approved:",
    b"- valid-from:",
    b"- valid-until:",
];

/// The file in the state folder that holds the checksums.
const FILE: &str = "approved.json";

/// Lessons, how-tos and tools: an LLM writes them as drafts, approval records their checksum.
pub fn guarded(kind: Kind) -> bool {
    matches!(kind, Kind::Lesson | Kind::Howto | Kind::Tool)
}

/// Records the checksum of an approved entry. A broken `approved.json` is refused, not replaced.
pub fn record(state: &Path, id: &str, bytes: &[u8]) -> Result<()> {
    let mut records = load(state)?;
    records.insert(key(id), Value::String(checksum(bytes)));
    save(state, &records)
}

/// Forgets the recorded checksum of entry `id`. A broken `approved.json` is refused, not
/// replaced.
pub fn forget(state: &Path, id: &str) -> Result<()> {
    let mut records = load(state)?;
    records.remove(&key(id));
    save(state, &records)
}

/// Compares an entry with its recorded checksum. A broken `approved.json` records nothing.
pub fn mark(state: &Path, id: &str, bytes: &[u8]) -> Mark {
    let Ok(records) = load(state) else {
        return Mark::NotRecorded;
    };
    let Some(recorded) = records.get(&key(id)) else {
        return Mark::NotRecorded;
    };
    if recorded.as_str() == Some(checksum(bytes).as_str()) {
        Mark::Unchanged
    } else {
        Mark::Changed
    }
}

/// The checksum of the part of an entry that counts.
fn checksum(bytes: &[u8]) -> String {
    hash::sha256_hex(&stable(bytes))
}

/// The entry text without the ignored lines, with every CRLF turned into LF.
fn stable(bytes: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(bytes.len());
    for line in bytes.split_inclusive(|&byte| byte == b'\n') {
        if IGNORED_LINES.iter().any(|prefix| line.starts_with(prefix)) {
            continue;
        }
        match line.strip_suffix(b"\r\n") {
            Some(body) => {
                out.extend_from_slice(body);
                out.push(b'\n');
            }
            None => out.extend_from_slice(line),
        }
    }
    out
}

/// The key of an entry in the file: its ID in upper case.
fn key(id: &str) -> String {
    id.to_uppercase()
}

/// The recorded checksums. A missing file is an empty object. A file that cannot be read or is
/// not a JSON object is Code::Parse, so that nothing overwrites it.
fn load(state: &Path) -> Result<Map<String, Value>> {
    let path = state.join(FILE);
    match std::fs::read(&path) {
        Ok(bytes) => serde_json::from_slice(&bytes).map_err(|_| broken(&path)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(Map::new()),
        Err(_) => Err(broken(&path)),
    }
}

/// The refusal for a broken `approved.json` at `path`.
fn broken(path: &Path) -> Error {
    Error::new(
        Code::Parse,
        format!(
            "{}: not a valid approved.json; fix or delete it",
            path.display()
        ),
    )
}

/// Writes the checksums as one line.
fn save(state: &Path, records: &Map<String, Value>) -> Result<()> {
    let mut text =
        serde_json::to_string(records).map_err(|error| Error::new(Code::Io, error.to_string()))?;
    text.push('\n');
    atomic::replace(&state.join(FILE), text.as_bytes(), None)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// An approved lesson, with LF endings.
    fn entry() -> String {
        [
            "<!-- dossier-format: 1 -->",
            "# LES-031 · Warm the cache first",
            "",
            "- kind: lesson",
            "- status: approved",
            "- created: 2026-11-20T14:03:27+01:00",
            "- author: anna",
            "- approved: 2026-11-20T14:03:27+01:00 by anna",
            "- valid-from: 2026-11-20T14:03:27+01:00",
            "- history: 2026-11-20T14:03:27+01:00 anna: created",
            "- history: 2026-11-20T14:03:27+01:00 anna: approved",
            "",
            "## Statement",
            "",
            "The cache must be warm before the first run.",
            "",
        ]
        .join("\n")
    }

    fn file(state: &Path) -> std::path::PathBuf {
        state.join(FILE)
    }

    #[test]
    fn record_then_mark_is_unchanged() {
        let state = tempfile::tempdir().unwrap();
        record(state.path(), "LES-031", entry().as_bytes()).unwrap();
        assert!(matches!(
            mark(state.path(), "LES-031", entry().as_bytes()),
            Mark::Unchanged
        ));
    }

    #[test]
    fn ids_are_kept_in_upper_case_and_found_in_any_case() {
        let state = tempfile::tempdir().unwrap();
        record(state.path(), "les-031", entry().as_bytes()).unwrap();
        let text = std::fs::read_to_string(file(state.path())).unwrap();
        let records: Map<String, Value> = serde_json::from_str(&text).unwrap();
        assert!(records.contains_key("LES-031"), "{text}");
        assert!(matches!(
            mark(state.path(), "Les-031", entry().as_bytes()),
            Mark::Unchanged
        ));
    }

    #[test]
    fn history_status_and_approval_lines_do_not_count() {
        let state = tempfile::tempdir().unwrap();
        record(state.path(), "LES-031", entry().as_bytes()).unwrap();
        let changed = entry()
            .replace("- status: approved", "- status: retired")
            .replace(
                "- approved: 2026-11-20T14:03:27+01:00 by anna",
                "- approved: 2026-11-21T09:00:00+01:00 by ben",
            )
            .replace(
                "- valid-from: 2026-11-20T14:03:27+01:00\n",
                "- valid-from: 2026-11-20T14:03:27+01:00\n- valid-until: 2026-11-21T09:00:00+01:00\n",
            )
            .replace(
                "- history: 2026-11-20T14:03:27+01:00 anna: approved",
                "- history: 2026-11-20T14:03:27+01:00 anna: approved\n- history: 2026-11-21T09:00:00+01:00 ben: retired",
            );
        assert_ne!(changed, entry());
        assert!(matches!(
            mark(state.path(), "LES-031", changed.as_bytes()),
            Mark::Unchanged
        ));
    }

    #[test]
    fn crlf_endings_do_not_count() {
        let state = tempfile::tempdir().unwrap();
        record(state.path(), "LES-031", entry().as_bytes()).unwrap();
        let windows = entry().replace('\n', "\r\n");
        assert!(matches!(
            mark(state.path(), "LES-031", windows.as_bytes()),
            Mark::Unchanged
        ));
    }

    #[test]
    fn a_changed_word_in_a_section_is_changed() {
        let state = tempfile::tempdir().unwrap();
        record(state.path(), "LES-031", entry().as_bytes()).unwrap();
        let changed = entry().replace("must be warm", "must be cold");
        assert!(matches!(
            mark(state.path(), "LES-031", changed.as_bytes()),
            Mark::Changed
        ));
    }

    #[test]
    fn a_changed_title_is_changed() {
        let state = tempfile::tempdir().unwrap();
        record(state.path(), "LES-031", entry().as_bytes()).unwrap();
        let changed = entry().replace("Warm the cache first", "Warm the cache early");
        assert!(matches!(
            mark(state.path(), "LES-031", changed.as_bytes()),
            Mark::Changed
        ));
    }

    #[test]
    fn an_entry_that_was_never_recorded_is_not_recorded() {
        let state = tempfile::tempdir().unwrap();
        assert!(matches!(
            mark(state.path(), "LES-031", entry().as_bytes()),
            Mark::NotRecorded
        ));
    }

    /// File texts that are not a JSON object.
    const BROKEN: [&str; 4] = ["[\"LES-031\"]", "null", "{\"LES-031\": ", ""];

    /// The refusal of a broken file at `path`.
    fn assert_refused(error: &Error, path: &Path) {
        assert_eq!(error.code, Code::Parse, "{}", error.message);
        assert_eq!(
            error.message,
            format!(
                "{}: not a valid approved.json; fix or delete it",
                path.display()
            )
        );
    }

    #[test]
    fn mark_treats_a_broken_file_as_nothing_recorded() {
        for broken in BROKEN {
            let state = tempfile::tempdir().unwrap();
            std::fs::write(file(state.path()), broken).unwrap();
            assert!(matches!(
                mark(state.path(), "LES-031", entry().as_bytes()),
                Mark::NotRecorded
            ));
        }
    }

    #[test]
    fn record_refuses_a_broken_file_and_leaves_it_unchanged() {
        for broken in BROKEN {
            let root = tempfile::tempdir().unwrap();
            let state = root.path().join("Projekt Übersicht");
            std::fs::create_dir(&state).unwrap();
            std::fs::write(file(&state), broken).unwrap();
            let error = record(&state, "LES-031", entry().as_bytes()).unwrap_err();
            assert_refused(&error, &file(&state));
            assert_eq!(std::fs::read_to_string(file(&state)).unwrap(), broken);
        }
    }

    #[test]
    fn forget_refuses_a_broken_file_and_leaves_it_unchanged() {
        for broken in BROKEN {
            let state = tempfile::tempdir().unwrap();
            std::fs::write(file(state.path()), broken).unwrap();
            let error = forget(state.path(), "LES-031").unwrap_err();
            assert_refused(&error, &file(state.path()));
            assert_eq!(std::fs::read_to_string(file(state.path())).unwrap(), broken);
        }
    }

    #[test]
    fn a_file_that_cannot_be_read_is_refused_and_kept() {
        let state = tempfile::tempdir().unwrap();
        // A folder where the file should be: it cannot be read as a file.
        std::fs::create_dir(file(state.path())).unwrap();
        let error = record(state.path(), "LES-031", entry().as_bytes()).unwrap_err();
        assert_refused(&error, &file(state.path()));
        let error = forget(state.path(), "LES-031").unwrap_err();
        assert_refused(&error, &file(state.path()));
        assert!(file(state.path()).is_dir());
        assert!(matches!(
            mark(state.path(), "LES-031", entry().as_bytes()),
            Mark::NotRecorded
        ));
    }

    #[test]
    fn a_missing_file_is_an_empty_object() {
        let state = tempfile::tempdir().unwrap();
        forget(state.path(), "LES-031").unwrap();
        assert_eq!(std::fs::read_to_string(file(state.path())).unwrap(), "{}\n");
    }

    #[test]
    fn forget_gives_not_recorded() {
        let state = tempfile::tempdir().unwrap();
        record(state.path(), "LES-031", entry().as_bytes()).unwrap();
        forget(state.path(), "LES-031").unwrap();
        assert!(matches!(
            mark(state.path(), "LES-031", entry().as_bytes()),
            Mark::NotRecorded
        ));
    }

    #[test]
    fn forget_keeps_the_other_records() {
        let state = tempfile::tempdir().unwrap();
        let other = entry().replace("LES-031", "LES-032");
        record(state.path(), "LES-031", entry().as_bytes()).unwrap();
        record(state.path(), "LES-032", other.as_bytes()).unwrap();
        forget(state.path(), "LES-031").unwrap();
        assert!(matches!(
            mark(state.path(), "LES-032", other.as_bytes()),
            Mark::Unchanged
        ));
        assert!(matches!(
            mark(state.path(), "LES-031", entry().as_bytes()),
            Mark::NotRecorded
        ));
    }

    #[test]
    fn the_file_is_one_line_and_is_created_in_a_folder_that_does_not_exist() {
        let root = tempfile::tempdir().unwrap();
        let state = root.path().join("Projekt Übersicht").join(".state");
        record(&state, "LES-031", entry().as_bytes()).unwrap();
        let text = std::fs::read_to_string(file(&state)).unwrap();
        assert!(text.ends_with('\n'), "{text:?}");
        assert_eq!(text.matches('\n').count(), 1, "{text:?}");
        assert!(matches!(
            mark(&state, "LES-031", entry().as_bytes()),
            Mark::Unchanged
        ));
    }

    #[test]
    fn stable_drops_the_ignored_lines_and_turns_crlf_into_lf() {
        assert_eq!(
            stable(b"# T\r\n- status: draft\r\n- history: t anna: created\r\n\r\nBody\r\n"),
            b"# T\n\nBody\n".to_vec()
        );
    }

    #[test]
    fn lessons_howtos_and_tools_are_guarded_and_the_other_kinds_are_not() {
        for kind in Kind::all() {
            let expected = matches!(kind, Kind::Lesson | Kind::Howto | Kind::Tool);
            assert_eq!(guarded(kind), expected, "{}", kind.name());
        }
        assert!(guarded(Kind::Howto));
        assert!(!guarded(Kind::Usage));
    }
}
