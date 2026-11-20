//! Entry IDs: the next free number of each kind.

use crate::atomic;
use crate::entry::Kind;
use crate::error::{Code, Error, Result};
use crate::store;
use std::collections::BTreeMap;
use std::path::Path;

/// The file of the state folder that holds the highest number given out per prefix.
const NUMBERS_FILE: &str = "ids.json";

/// The byte-order mark that a file may start with (format.md §2).
const BYTE_ORDER_MARK: &[u8] = &[0xEF, 0xBB, 0xBF];

/// The next ID of a kind: 1 + max(the number in ids.json, the highest number among the
/// entries on disk). Writes ids.json. Call only while holding the lock.
pub fn next(dossier: &Path, state: &Path, kind: Kind) -> Result<String> {
    let prefix = kind.prefix();
    let path = state.join(NUMBERS_FILE);
    let mut numbers = read_numbers(&path);
    let recorded = numbers.get(prefix).copied().unwrap_or(0);
    let number = recorded
        .max(highest_on_disk(dossier, prefix))
        .checked_add(1)
        .ok_or_else(|| {
            Error::new(
                Code::Parse,
                format!("the number of {prefix} is too large to count up from"),
            )
        })?;
    numbers.insert(prefix.to_string(), number);

    let mut text = serde_json::to_string(&numbers)
        .map_err(|error| Error::new(Code::Io, format!("{}: {error}", path.display())))?;
    text.push('\n');
    atomic::replace(&path, text.as_bytes(), None)?;
    Ok(format!("{prefix}-{number:03}"))
}

/// The numbers in ids.json. A missing or unreadable file, or one that is not a JSON object of
/// whole numbers, is an empty object. A byte-order mark at the start is skipped.
fn read_numbers(path: &Path) -> BTreeMap<String, u64> {
    let Ok(bytes) = std::fs::read(path) else {
        return BTreeMap::new();
    };
    let text = bytes.strip_prefix(BYTE_ORDER_MARK).unwrap_or(&bytes);
    serde_json::from_slice(text).unwrap_or_default()
}

/// The highest number among the entries whose ID has this prefix; 0 when there is none. Files
/// that fail to parse are not in `store::all`, so they are not counted.
fn highest_on_disk(dossier: &Path, prefix: &str) -> u64 {
    let (items, _warnings) = store::all(dossier);
    items
        .iter()
        .filter_map(|item| number_of(&item.entry.id(), prefix))
        .max()
        .unwrap_or(0)
}

/// The number of an ID with this prefix: `DEC-012` gives 12. None for another prefix, or when no
/// digit follows the `-`. A number too large for a u64 gives u64::MAX, so that the next ID is
/// refused rather than repeated.
fn number_of(id: &str, prefix: &str) -> Option<u64> {
    let (found, rest) = id.split_once('-')?;
    if found != prefix {
        return None;
    }
    let digits: String = rest.chars().take_while(char::is_ascii_digit).collect();
    if digits.is_empty() {
        return None;
    }
    Some(digits.parse().unwrap_or(u64::MAX))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;
    use tempfile::TempDir;

    /// A dossier folder with its four kind folders and no state folder yet, in a temporary folder
    /// whose name has a space and a non-ASCII letter. Returns the temporary folder, the dossier
    /// and its state folder.
    fn dossier() -> (TempDir, PathBuf, PathBuf) {
        let temp = tempfile::tempdir().expect("temporary folder");
        let dossier = temp.path().join("Prüf stand").join("DOSSIER");
        for kind in Kind::all() {
            fs::create_dir_all(dossier.join(kind.folder())).expect("kind folder");
        }
        let state = dossier.join(".state");
        (temp, dossier, state)
    }

    /// Writes an entry file in the folder of its kind, with the given heading and literal text
    /// in the form of format.md §1. The title is invented.
    fn put(dossier: &Path, kind: Kind, file: &str, heading: &str) {
        let text = format!(
            "<!-- dossier-format: 1 -->\n# {heading}\n\n- kind: {}\n- status: approved\n- created: 2026-11-20T09:00:00+01:00\n- author: tess\n\n## Text\n\nAn invented text for a test.\n",
            kind.name()
        );
        fs::write(dossier.join(kind.folder()).join(file), text).expect("entry file");
    }

    /// The text of ids.json.
    fn ids_json(state: &Path) -> String {
        fs::read_to_string(state.join(NUMBERS_FILE)).expect("ids.json")
    }

    #[test]
    fn the_first_decision_is_dec_001_and_the_next_is_dec_002() {
        let (_temp, dossier, state) = dossier();
        assert_eq!(
            next(&dossier, &state, Kind::Decision).expect("first ID"),
            "DEC-001"
        );
        assert_eq!(
            next(&dossier, &state, Kind::Decision).expect("second ID"),
            "DEC-002"
        );
        assert_eq!(ids_json(&state), "{\"DEC\":2}\n");
    }

    #[test]
    fn an_entry_on_disk_counts_and_a_number_without_an_entry_does_not() {
        let (_temp, dossier, state) = dossier();
        put(
            &dossier,
            Kind::Decision,
            "DEC-007-invented.md",
            "DEC-007 · Invented",
        );
        put(
            &dossier,
            Kind::Decision,
            "misc.md",
            "DEC · Without a number",
        );
        assert_eq!(
            next(&dossier, &state, Kind::Decision).expect("ID"),
            "DEC-008"
        );
    }

    #[test]
    fn a_number_given_out_still_counts_after_its_entry_is_deleted() {
        let (_temp, dossier, state) = dossier();
        put(
            &dossier,
            Kind::Decision,
            "DEC-007-invented.md",
            "DEC-007 · Invented",
        );
        assert_eq!(
            next(&dossier, &state, Kind::Decision).expect("ID"),
            "DEC-008"
        );
        put(
            &dossier,
            Kind::Decision,
            "DEC-008-invented.md",
            "DEC-008 · Invented",
        );
        assert_eq!(
            next(&dossier, &state, Kind::Decision).expect("ID"),
            "DEC-009"
        );
        assert_eq!(ids_json(&state), "{\"DEC\":9}\n");

        fs::remove_file(dossier.join("decisions").join("DEC-007-invented.md"))
            .expect("delete DEC-007");
        fs::remove_file(dossier.join("decisions").join("DEC-008-invented.md"))
            .expect("delete DEC-008");
        assert_eq!(
            next(&dossier, &state, Kind::Decision).expect("ID"),
            "DEC-010"
        );
    }

    #[test]
    fn each_kind_counts_its_own_numbers() {
        let (_temp, dossier, state) = dossier();
        put(
            &dossier,
            Kind::Assumption,
            "ASM-050-invented.md",
            "ASM-050 · Invented",
        );
        assert_eq!(
            next(&dossier, &state, Kind::Decision).expect("decision"),
            "DEC-001"
        );
        assert_eq!(
            next(&dossier, &state, Kind::Assumption).expect("assumption"),
            "ASM-051"
        );
        assert_eq!(
            next(&dossier, &state, Kind::Finding).expect("finding"),
            "FND-001"
        );
        assert_eq!(
            next(&dossier, &state, Kind::Lesson).expect("lesson"),
            "LES-001"
        );
        assert_eq!(
            next(&dossier, &state, Kind::Decision).expect("decision"),
            "DEC-002"
        );
        assert_eq!(
            ids_json(&state),
            "{\"ASM\":51,\"DEC\":2,\"FND\":1,\"LES\":1}\n"
        );
    }

    #[test]
    fn an_unreadable_ids_json_counts_as_empty_and_the_entries_still_count() {
        let (_temp, dossier, state) = dossier();
        fs::create_dir_all(&state).expect("state folder");
        fs::write(state.join(NUMBERS_FILE), "{\"DEC\": not a number").expect("ids.json");
        put(
            &dossier,
            Kind::Decision,
            "DEC-004-invented.md",
            "DEC-004 · Invented",
        );
        assert_eq!(
            next(&dossier, &state, Kind::Decision).expect("ID"),
            "DEC-005"
        );
        assert_eq!(ids_json(&state), "{\"DEC\":5}\n");
    }

    #[test]
    fn a_byte_order_mark_in_ids_json_is_skipped() {
        let (_temp, dossier, state) = dossier();
        fs::create_dir_all(&state).expect("state folder");
        let mut bytes = BYTE_ORDER_MARK.to_vec();
        bytes.extend_from_slice(b"{\"DEC\":12}\n");
        fs::write(state.join(NUMBERS_FILE), bytes).expect("ids.json");
        assert_eq!(
            next(&dossier, &state, Kind::Decision).expect("ID"),
            "DEC-013"
        );
    }

    #[test]
    fn a_number_past_999_keeps_all_its_digits() {
        let (_temp, dossier, state) = dossier();
        fs::create_dir_all(&state).expect("state folder");
        fs::write(state.join(NUMBERS_FILE), "{\"DEC\":999}\n").expect("ids.json");
        assert_eq!(
            next(&dossier, &state, Kind::Decision).expect("ID"),
            "DEC-1000"
        );
        assert_eq!(
            next(&dossier, &state, Kind::Decision).expect("ID"),
            "DEC-1001"
        );
    }

    #[test]
    fn a_number_too_large_to_count_up_is_an_error_and_nothing_is_written() {
        let (_temp, dossier, state) = dossier();
        put(
            &dossier,
            Kind::Decision,
            "DEC-huge-invented.md",
            "DEC-99999999999999999999999 · Invented",
        );
        let error = next(&dossier, &state, Kind::Decision).expect_err("no ID");
        assert_eq!(error.code, Code::Parse);
        assert!(!state.join(NUMBERS_FILE).exists());
    }
}
