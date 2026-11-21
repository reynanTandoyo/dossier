//! The exchange with the customer or other parties (format.md §17, exchange.md): one block per
//! received (RCV) or delivered (DLV) record.

use crate::blockfile::BlockFile;
use crate::error::{Code, Error, Result};
use std::path::Path;

/// The file of a dossier that holds the exchange, `exchange.md`.
pub const FILE: &str = "exchange.md";

/// The item name of the exchange file in snapshots and the journal.
pub const ITEM: &str = "EXCHANGE";

/// The exchange file of a project and its bytes as read; a missing file is
/// BlockFile::empty("Exchange of <code>"), None.
///
/// A file that cannot be read is Code::Io. A file that is not UTF-8 text is Code::Parse. A file
/// that does not parse, or has a newer format, keeps the error code of `BlockFile::parse`, and
/// its message starts with the path of the file.
pub fn load(dossier: &Path, code: &str) -> Result<(BlockFile, Option<Vec<u8>>)> {
    let path = dossier.join(FILE);
    let bytes = match std::fs::read(&path) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok((BlockFile::empty(&format!("Exchange of {code}")), None));
        }
        Err(error) => {
            return Err(Error::new(Code::Io, format!("{}: {error}", path.display())));
        }
    };
    let text = std::str::from_utf8(&bytes)
        .map_err(|_| Error::new(Code::Parse, format!("{}: not UTF-8 text", path.display())))?;
    let file = BlockFile::parse(text).map_err(|error| {
        Error::new(error.code, format!("{}: {}", path.display(), error.message))
    })?;
    Ok((file, Some(bytes)))
}

/// The highest number of RCV or DLV keys with this prefix, 0 when there is none.
pub fn highest(file: &BlockFile, prefix: &str) -> u64 {
    file.keys()
        .iter()
        .filter_map(|key| number_of_key(key, prefix))
        .max()
        .unwrap_or(0)
}

/// The number in a key `<prefix>-<digits>`: `RCV-002` gives 2 for the prefix `RCV`. None for any
/// other key. A number too large for a u64 gives u64::MAX, so that the next number is refused
/// rather than repeated (as in ids.rs).
fn number_of_key(key: &str, prefix: &str) -> Option<u64> {
    let digits = key.strip_prefix(prefix)?.strip_prefix('-')?;
    if digits.is_empty() || !digits.bytes().all(|byte| byte.is_ascii_digit()) {
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

    /// Three invented records in the form of format.md §17: two received, one delivered.
    const RECORDS: &str = r#"<!-- dossier-format: 1 -->
# Exchange of P-001

## RCV-001

- date: 2026-11-02
- from: Customer A
- files: inbox/spec-rev2.pdf@sha256:9f2c0b7e4d1a8c3f9f2c0b7e4d1a8c3f9f2c0b7e4d1a8c3f9f2c0b7e4d1a8c3f
- history: 2026-11-02T10:00:00+01:00 anna: received

## RCV-002

- date: 2026-11-10
- from: Customer A
- note: revised spec
- history: 2026-11-10T09:30:00+01:00 anna: received

## DLV-001

- date: 2026-11-18
- to: Customer A
- based-on: T-042
- history: 2026-11-18T16:00:00+01:00 anna: delivered
"#;

    /// A dossier folder in a temporary folder whose name has a space and a non-ASCII letter.
    fn dossier() -> (TempDir, PathBuf) {
        let temp = tempfile::tempdir().expect("temporary folder");
        let dossier = temp.path().join("Prüf stand").join("DOSSIER");
        fs::create_dir_all(&dossier).expect("dossier folder");
        (temp, dossier)
    }

    #[test]
    fn highest_is_the_largest_number_of_each_prefix_in_the_example() {
        let file = BlockFile::parse(RECORDS).expect("the example parses");
        assert_eq!(highest(&file, "RCV"), 2);
        assert_eq!(highest(&file, "DLV"), 1);
        assert_eq!(highest(&file, "XYZ"), 0);
    }

    #[test]
    fn a_missing_file_is_empty_and_nothing_is_written() {
        let (_temp, dossier) = dossier();
        let (file, bytes) = load(&dossier, "P-001").expect("load");
        assert_eq!(
            file.text(),
            "<!-- dossier-format: 1 -->\n# Exchange of P-001\n\n"
        );
        assert!(bytes.is_none());
        assert!(file.keys().is_empty());
        assert_eq!(highest(&file, "RCV"), 0);
        assert!(!dossier.join(FILE).exists());
    }

    #[test]
    fn an_existing_file_is_read_with_the_bytes_as_read() {
        let (_temp, dossier) = dossier();
        fs::write(dossier.join(FILE), RECORDS).expect("exchange.md");
        let (file, bytes) = load(&dossier, "P-001").expect("load");
        assert_eq!(bytes.as_deref(), Some(RECORDS.as_bytes()));
        assert_eq!(file.text(), RECORDS);
        assert_eq!(highest(&file, "RCV"), 2);
        assert_eq!(highest(&file, "DLV"), 1);
    }

    #[test]
    fn a_byte_order_mark_is_kept_and_the_records_are_still_found() {
        let (_temp, dossier) = dossier();
        let text = format!("\u{feff}{RECORDS}");
        fs::write(dossier.join(FILE), &text).expect("exchange.md");
        let (file, bytes) = load(&dossier, "P-001").expect("load");
        assert_eq!(bytes.as_deref(), Some(text.as_bytes()));
        assert_eq!(file.text(), text);
        assert_eq!(highest(&file, "RCV"), 2);
    }

    #[test]
    fn only_keys_of_the_form_prefix_dash_digits_count() {
        let text = "<!-- dossier-format: 1 -->\n# Exchange of P-001\n\n## RCV-\n\n## RCV-x3\n\n## RCV-3a\n\n## RCV-4 later\n\n## RCVX-9\n\n## rcv-8\n\n## DLV-2\n\n## RCV-007\n";
        let file = BlockFile::parse(text).expect("parses");
        assert_eq!(highest(&file, "RCV"), 7);
        assert_eq!(highest(&file, "DLV"), 2);
    }

    #[test]
    fn a_number_too_large_for_a_u64_is_the_highest_and_is_not_repeated() {
        let file = BlockFile::parse("# Exchange of P-001\n\n## RCV-99999999999999999999999\n")
            .expect("parses");
        assert_eq!(highest(&file, "RCV"), u64::MAX);
    }

    #[test]
    fn a_file_that_cannot_be_used_is_an_error_that_names_the_file() {
        let (_temp, dossier) = dossier();
        let path = dossier.join(FILE);

        fs::write(&path, "<!-- dossier-format: 1 -->\nno heading\n").expect("exchange.md");
        let error = load(&dossier, "P-001").expect_err("no heading");
        assert_eq!(error.code, Code::Parse);
        assert_eq!(
            error.message,
            format!("{}: line 2: no heading", path.display())
        );

        fs::write(&path, "<!-- dossier-format: 2 -->\n# Exchange of P-001\n").expect("exchange.md");
        let error = load(&dossier, "P-001").expect_err("too new");
        assert_eq!(error.code, Code::TooNew);
        assert!(error.message.starts_with(&path.display().to_string()));

        fs::write(&path, [0xFF, 0xFE, b'#', b' ', b'x']).expect("exchange.md");
        let error = load(&dossier, "P-001").expect_err("not UTF-8");
        assert_eq!(error.code, Code::Parse);

        fs::remove_file(&path).expect("remove exchange.md");
        fs::create_dir(&path).expect("a folder in place of the file");
        let error = load(&dossier, "P-001").expect_err("a folder is not readable");
        assert_eq!(error.code, Code::Io);
    }
}
