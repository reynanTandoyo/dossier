//! The journal: a record of every change, who made it and when.

use crate::error::{Code, Error, Result};
use std::io::Write;
use std::path::Path;

/// The name of the journal file in the state folder.
const FILE: &str = "journal.jsonl";

/// One journal record.
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct Record {
    pub time: String,
    pub name: String,
    pub person: String,
    pub llm: bool,
    pub command: String,
    pub id: String,
    pub event: String,
    pub detail: String,
}

/// Appends a record to the journal in `state`.
pub fn append(state: &Path, record: &Record) -> Result<()> {
    std::fs::create_dir_all(state).map_err(|error| io_failure(state, error))?;
    let path = state.join(FILE);
    let mut line = serde_json::to_string(record)
        .map_err(|error| Error::new(Code::Io, format!("{}: {error}", path.display())))?;
    line.push('\n');
    let mut file = std::fs::OpenOptions::new()
        .append(true)
        .create(true)
        .open(&path)
        .map_err(|error| io_failure(&path, error))?;
    file.write_all(line.as_bytes())
        .map_err(|error| io_failure(&path, error))
}

/// Every record, oldest first; bad lines are skipped.
pub fn read(state: &Path) -> Result<Vec<Record>> {
    let path = state.join(FILE);
    let bytes = match std::fs::read(&path) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(io_failure(&path, error)),
    };
    Ok(bytes
        .split(|byte| *byte == b'\n')
        .map(|line| line.trim_ascii())
        .filter(|line| !line.is_empty())
        .filter_map(|line| serde_json::from_slice::<Record>(line).ok())
        .collect())
}

/// An I/O failure, naming the file or folder it happened to.
fn io_failure(path: &Path, error: std::io::Error) -> Error {
    Error::new(Code::Io, format!("{}: {error}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn found() -> Record {
        Record {
            time: "2026-11-21T10:15:00+01:00".into(),
            name: "anna".into(),
            person: "anna".into(),
            llm: false,
            command: "found".into(),
            id: "FND-007".into(),
            event: "created".into(),
            detail: String::new(),
        }
    }

    fn learned() -> Record {
        Record {
            time: "2026-11-21T10:20:31+01:00".into(),
            name: "ben-codex".into(),
            person: "ben".into(),
            llm: true,
            command: "learn".into(),
            id: "LES-004".into(),
            event: "created".into(),
            detail: "waiting for approval".into(),
        }
    }

    fn assert_same(actual: &Record, expected: &Record) {
        assert_eq!(actual.time, expected.time);
        assert_eq!(actual.name, expected.name);
        assert_eq!(actual.person, expected.person);
        assert_eq!(actual.llm, expected.llm);
        assert_eq!(actual.command, expected.command);
        assert_eq!(actual.id, expected.id);
        assert_eq!(actual.event, expected.event);
        assert_eq!(actual.detail, expected.detail);
    }

    #[test]
    fn two_appended_records_come_back_in_order_with_equal_fields() {
        let folder = tempfile::tempdir().unwrap();
        let state = folder.path().join("state");
        append(&state, &found()).unwrap();
        append(&state, &learned()).unwrap();
        let read_back = read(&state).unwrap();
        assert_eq!(read_back.len(), 2);
        assert_same(&read_back[0], &found());
        assert_same(&read_back[1], &learned());
    }

    #[test]
    fn a_record_is_one_line_with_the_keys_of_format_section_11() {
        let folder = tempfile::tempdir().unwrap();
        let state = folder.path().join("state");
        append(&state, &found()).unwrap();
        let text = std::fs::read_to_string(state.join("journal.jsonl")).unwrap();
        assert_eq!(
            text,
            "{\"time\":\"2026-11-21T10:15:00+01:00\",\"name\":\"anna\",\"person\":\"anna\",\
             \"llm\":false,\"command\":\"found\",\"id\":\"FND-007\",\"event\":\"created\",\
             \"detail\":\"\"}\n"
        );
    }

    #[test]
    fn a_garbage_line_between_two_records_is_skipped() {
        let folder = tempfile::tempdir().unwrap();
        let state = folder.path().join("state");
        append(&state, &found()).unwrap();
        let mut file = std::fs::OpenOptions::new()
            .append(true)
            .open(state.join("journal.jsonl"))
            .unwrap();
        file.write_all(b"this is not json\n\xff\xfe broken bytes\n\n{\"time\":\"x\"}\n")
            .unwrap();
        drop(file);
        append(&state, &learned()).unwrap();
        let read_back = read(&state).unwrap();
        assert_eq!(read_back.len(), 2);
        assert_same(&read_back[0], &found());
        assert_same(&read_back[1], &learned());
    }

    #[test]
    fn a_missing_file_gives_an_empty_list() {
        let folder = tempfile::tempdir().unwrap();
        assert!(read(folder.path()).unwrap().is_empty());
        assert!(
            read(&folder.path().join("never-created"))
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn a_state_folder_with_spaces_and_non_ascii_names_works() {
        let folder = tempfile::tempdir().unwrap();
        let state = folder.path().join("Projekt Übersicht").join(".state");
        append(&state, &learned()).unwrap();
        let read_back = read(&state).unwrap();
        assert_eq!(read_back.len(), 1);
        assert_same(&read_back[0], &learned());
    }

    #[test]
    fn a_state_path_that_is_a_file_is_an_io_error_naming_it() {
        let folder = tempfile::tempdir().unwrap();
        let blocker = folder.path().join("state");
        std::fs::write(&blocker, b"not a folder").unwrap();
        let error = append(&blocker, &found()).err();
        assert_eq!(error.as_ref().map(|error| error.code), Some(Code::Io));
        assert!(
            error.is_some_and(|error| error.message.starts_with(&*blocker.display().to_string()))
        );
    }
}
