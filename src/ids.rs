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
    let mut numbers = read_numbers(&path)?;
    let number = give_out(&mut numbers, prefix, highest_on_disk(dossier, kind)?)?;
    write_numbers(&path, &numbers)?;
    Ok(format!("{prefix}-{number:03}"))
}

/// The next number for a prefix that is not an entry kind (RCV, DLV): 1 + max(ids.json, on_disk).
/// Writes ids.json; call under the lock. Returns e.g. "RCV-001".
pub fn next_prefix(state: &Path, prefix: &str, on_disk: u64) -> Result<String> {
    let path = state.join(NUMBERS_FILE);
    let mut numbers = read_numbers(&path)?;
    let number = give_out(&mut numbers, prefix, on_disk)?;
    write_numbers(&path, &numbers)?;
    Ok(format!("{prefix}-{number:03}"))
}

/// Gives out the next number of a prefix and records it in `numbers`: 1 + max(the recorded
/// number, on_disk). A number that cannot be counted up from is Code::Parse.
fn give_out(numbers: &mut BTreeMap<String, u64>, prefix: &str, on_disk: u64) -> Result<u64> {
    let recorded = numbers.get(prefix).copied().unwrap_or(0);
    let number = recorded.max(on_disk).checked_add(1).ok_or_else(|| {
        Error::new(
            Code::Parse,
            format!("the number of {prefix} is too large to count up from"),
        )
    })?;
    numbers.insert(prefix.to_string(), number);
    Ok(number)
}

/// Replaces ids.json with the numbers as one JSON line. `atomic::replace` leaves the old file
/// in place when the write fails.
fn write_numbers(path: &Path, numbers: &BTreeMap<String, u64>) -> Result<()> {
    let mut text = serde_json::to_string(numbers)
        .map_err(|error| Error::new(Code::Io, format!("{}: {error}", path.display())))?;
    text.push('\n');
    atomic::replace(path, text.as_bytes(), None)
}

/// The numbers in ids.json. A missing file is an empty object. A file that cannot be read, or
/// that is not a JSON object of whole numbers, is Code::Parse, and nothing is written over it. A
/// byte-order mark at the start is skipped.
fn read_numbers(path: &Path) -> Result<BTreeMap<String, u64>> {
    let bytes = match std::fs::read(path) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(BTreeMap::new());
        }
        Err(_) => return Err(not_valid(path)),
    };
    let text = bytes.strip_prefix(BYTE_ORDER_MARK).unwrap_or(&bytes);
    serde_json::from_slice(text).map_err(|_| not_valid(path))
}

/// The error for an ids.json that cannot be used.
fn not_valid(path: &Path) -> Error {
    Error::new(
        Code::Parse,
        format!(
            "{}: not a valid ids.json; fix it before adding entries",
            path.display()
        ),
    )
}

/// The highest number of a kind on disk, 0 when there is none. It is the highest number in the ID
/// of each entry that parses (`store::all`) and in the name of each file of the kind's folder,
/// whether or not that file parses.
fn highest_on_disk(dossier: &Path, kind: Kind) -> Result<u64> {
    let prefix = kind.prefix();
    let (items, _warnings) = store::all(dossier);
    let in_ids = items
        .iter()
        .filter_map(|item| number_of_id(&item.entry.id(), prefix))
        .max()
        .unwrap_or(0);
    Ok(in_ids.max(highest_in_file_names(dossier, kind)?))
}

/// The highest number in the names of the files directly inside the folder of a kind (format.md
/// §4), 0 when there is none. A folder that cannot be listed is an error, since its numbers
/// would be missed.
fn highest_in_file_names(dossier: &Path, kind: Kind) -> Result<u64> {
    let folder = dossier.join(kind.folder());
    let listing = match std::fs::read_dir(&folder) {
        Ok(listing) => listing,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(0),
        Err(error) => return Err(io_error(&folder, error)),
    };
    let mut highest = 0;
    for dir_entry in listing {
        let dir_entry = dir_entry.map_err(|error| io_error(&folder, error))?;
        let name = dir_entry.file_name().to_string_lossy().into_owned();
        if let Some(number) = number_in_file_name(&name, kind.prefix()) {
            highest = highest.max(number);
        }
    }
    Ok(highest)
}

/// A failure on `path` as Code::Io, with the path in the message.
fn io_error(path: &Path, error: std::io::Error) -> Error {
    Error::new(Code::Io, format!("{}: {error}", path.display()))
}

/// The number in the name of an entry file: `DEC-012-title.md` gives 12, and the prefix may be in
/// any case. None when the name does not end in `.md`, does not start with the prefix and a `-`,
/// or has no digit after the `-`.
fn number_in_file_name(name: &str, prefix: &str) -> Option<u64> {
    let stem = name.strip_suffix(".md")?;
    let head = stem.get(..prefix.len())?;
    if !head.eq_ignore_ascii_case(prefix) {
        return None;
    }
    leading_number(stem.get(prefix.len()..)?.strip_prefix('-')?)
}

/// The number of an ID with this prefix: `DEC-012` gives 12. None for another prefix, or when no
/// digit follows the `-`.
fn number_of_id(id: &str, prefix: &str) -> Option<u64> {
    let (found, rest) = id.split_once('-')?;
    if found != prefix {
        return None;
    }
    leading_number(rest)
}

/// The number made by the digits at the start of `text`; None when there is no digit. A number
/// too large for a u64 gives u64::MAX, so that the next ID is refused rather than repeated.
fn leading_number(text: &str) -> Option<u64> {
    let digits: String = text.chars().take_while(char::is_ascii_digit).collect();
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

    /// The message for an ids.json that cannot be used.
    fn not_valid_message(path: &Path) -> String {
        format!(
            "{}: not a valid ids.json; fix it before adding entries",
            path.display()
        )
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
    fn an_ids_json_that_is_not_a_map_of_numbers_is_an_error_and_is_kept() {
        let (_temp, dossier, state) = dossier();
        fs::create_dir_all(&state).expect("state folder");
        let path = state.join(NUMBERS_FILE);
        put(
            &dossier,
            Kind::Decision,
            "DEC-004-invented.md",
            "DEC-004 · Invented",
        );
        for text in [
            "",
            "[]",
            "null",
            "{\"DEC\": -1}",
            "{\"DEC\": \"seven\"}",
            "{\"DEC\": 4",
        ] {
            fs::write(&path, text).expect("ids.json");
            let error = next(&dossier, &state, Kind::Decision).expect_err("no ID");
            assert_eq!(error.code, Code::Parse, "{text}");
            assert_eq!(error.message, not_valid_message(&path), "{text}");
            assert_eq!(ids_json(&state), text, "{text}");
        }

        fs::write(&path, "{\"DEC\":4}\n").expect("fixed ids.json");
        assert_eq!(
            next(&dossier, &state, Kind::Decision).expect("ID"),
            "DEC-005"
        );
        assert_eq!(ids_json(&state), "{\"DEC\":5}\n");
    }

    #[test]
    fn an_ids_json_that_cannot_be_read_is_an_error() {
        let (_temp, dossier, state) = dossier();
        let path = state.join(NUMBERS_FILE);
        fs::create_dir_all(&path).expect("a folder where ids.json should be");
        let error = next(&dossier, &state, Kind::Decision).expect_err("no ID");
        assert_eq!(error.code, Code::Parse);
        assert_eq!(error.message, not_valid_message(&path));
        assert!(path.is_dir());
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

    #[test]
    fn a_file_name_blocks_its_number_even_when_its_entry_does_not_parse() {
        let (_temp, dossier, state) = dossier();
        fs::write(
            dossier.join("decisions").join("DEC-005-broken.md"),
            "<!-- dossier-format: 1 -->\nthis file has no heading\n",
        )
        .expect("broken entry");
        assert_eq!(
            next(&dossier, &state, Kind::Decision).expect("ID"),
            "DEC-006"
        );
        assert_eq!(ids_json(&state), "{\"DEC\":6}\n");
    }

    #[test]
    fn the_prefix_in_a_file_name_counts_in_any_case() {
        let (_temp, dossier, state) = dossier();
        let decisions = dossier.join("decisions");
        fs::write(decisions.join("dec-009-lower.md"), "not an entry\n").expect("lower case");
        fs::write(decisions.join("Dec-011.md"), "not an entry\n").expect("mixed case");
        assert_eq!(
            next(&dossier, &state, Kind::Decision).expect("ID"),
            "DEC-012"
        );
    }

    #[test]
    fn only_names_of_this_kind_in_its_folder_that_end_in_md_count() {
        let (_temp, dossier, state) = dossier();
        let decisions = dossier.join("decisions");
        fs::create_dir_all(decisions.join("archive")).expect("subfolder");
        for name in [
            "DEC-020.md.tmp-4242",
            "DEC-021-notes.txt",
            "DECISION-040.md",
            "DEC-x.md",
            "DEC-.md",
        ] {
            fs::write(decisions.join(name), "not an entry\n").expect(name);
        }
        fs::write(
            decisions.join("archive").join("DEC-050.md"),
            "not an entry\n",
        )
        .expect("nested file");
        fs::write(
            dossier.join("assumptions").join("ASM-060.md"),
            "not an entry\n",
        )
        .expect("other kind");
        assert_eq!(
            next(&dossier, &state, Kind::Decision).expect("ID"),
            "DEC-001"
        );
        assert_eq!(
            next(&dossier, &state, Kind::Assumption).expect("ID"),
            "ASM-061"
        );
    }

    #[test]
    fn a_kind_folder_that_cannot_be_listed_is_an_error_and_nothing_is_written() {
        let (_temp, dossier, state) = dossier();
        let decisions = dossier.join("decisions");
        fs::remove_dir(&decisions).expect("remove the decisions folder");
        fs::write(&decisions, "not a folder\n").expect("a file in its place");
        let error = next(&dossier, &state, Kind::Decision).expect_err("no ID");
        assert_eq!(error.code, Code::Io);
        assert!(!state.join(NUMBERS_FILE).exists());
    }

    #[test]
    fn the_first_rcv_is_rcv_001() {
        let (_temp, _dossier, state) = dossier();
        assert_eq!(next_prefix(&state, "RCV", 0).expect("first ID"), "RCV-001");
        assert_eq!(ids_json(&state), "{\"RCV\":1}\n");
    }

    #[test]
    fn a_number_on_disk_is_counted_up_from() {
        let (_temp, _dossier, state) = dossier();
        assert_eq!(next_prefix(&state, "RCV", 4).expect("first ID"), "RCV-005");
        assert_eq!(next_prefix(&state, "RCV", 4).expect("second ID"), "RCV-006");
        assert_eq!(ids_json(&state), "{\"RCV\":6}\n");
    }

    #[test]
    fn a_prefix_number_past_999_keeps_all_its_digits() {
        let (_temp, _dossier, state) = dossier();
        assert_eq!(next_prefix(&state, "DLV", 999).expect("ID"), "DLV-1000");
    }

    #[test]
    fn rcv_and_dlv_count_separately_and_do_not_disturb_dec() {
        let (_temp, dossier, state) = dossier();
        assert_eq!(
            next(&dossier, &state, Kind::Decision).expect("decision"),
            "DEC-001"
        );
        assert_eq!(next_prefix(&state, "RCV", 0).expect("received"), "RCV-001");
        assert_eq!(next_prefix(&state, "RCV", 0).expect("received"), "RCV-002");
        assert_eq!(next_prefix(&state, "DLV", 0).expect("delivered"), "DLV-001");
        assert_eq!(
            next(&dossier, &state, Kind::Decision).expect("decision"),
            "DEC-002"
        );
        assert_eq!(ids_json(&state), "{\"DEC\":2,\"DLV\":1,\"RCV\":2}\n");
    }

    #[test]
    fn a_prefix_number_too_large_to_count_up_is_an_error_and_nothing_is_written() {
        let (_temp, _dossier, state) = dossier();
        let error = next_prefix(&state, "RCV", u64::MAX).expect_err("no ID");
        assert_eq!(error.code, Code::Parse);
        assert_eq!(
            error.message,
            "the number of RCV is too large to count up from"
        );
        assert!(!state.join(NUMBERS_FILE).exists());
    }

    #[test]
    fn an_ids_json_that_is_broken_is_an_error_for_a_prefix_too_and_is_kept() {
        let (_temp, _dossier, state) = dossier();
        fs::create_dir_all(&state).expect("state folder");
        let path = state.join(NUMBERS_FILE);
        fs::write(&path, "{\"RCV\": \"two\"}\n").expect("ids.json");
        let error = next_prefix(&state, "RCV", 0).expect_err("no ID");
        assert_eq!(error.code, Code::Parse);
        assert_eq!(error.message, not_valid_message(&path));
        assert_eq!(ids_json(&state), "{\"RCV\": \"two\"}\n");
    }
}
