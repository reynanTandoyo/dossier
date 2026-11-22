//! The list of documents brought in from finished projects (format.md §25).

use crate::blockfile::BlockFile;
use crate::error::{Code, Error, Result};
use std::path::Path;

/// The file of a dossier that lists the documents, `imports.md`.
const FILE: &str = "imports.md";

/// The imports file of the dossier and its raw bytes, when it exists. A missing file is
/// BlockFile::empty("Documents of <code>") and None.
pub fn load(dossier: &Path, code: &str) -> Result<(BlockFile, Option<Vec<u8>>)> {
    let bytes = match std::fs::read(dossier.join(FILE)) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok((BlockFile::empty(&format!("Documents of {code}")), None));
        }
        Err(error) => return Err(error.into()),
    };
    let text = std::str::from_utf8(&bytes)
        .map_err(|_| Error::new(Code::Parse, format!("{FILE}: not UTF-8")))?;
    let file = BlockFile::parse(text)
        .map_err(|error| Error::new(error.code, format!("{FILE}: {}", error.message)))?;
    Ok((file, Some(bytes)))
}

/// The state of the document `stored` in the imports file, when it is listed: `reviewing`,
/// `done` or `nothing` (format.md §25). A listed document with no state field reads as an empty
/// text, as `values` reads a missing value. None when the document is not listed.
pub fn state(file: &BlockFile, stored: &str) -> Option<String> {
    let stored = stored.trim();
    file.keys()
        .iter()
        .any(|key| key == stored)
        .then(|| file.field(stored, "state").unwrap_or_default())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    /// The imports file of an invented project: one document done, one being reviewed whose name
    /// has a space and a non-ASCII letter, and one with nothing to take from it.
    const IMPORTS: &str = "<!-- dossier-format: 1 -->\n# Documents of P-003\n\n## docs/spec-rev1.pdf\n\n- state: done\n- history: 2026-11-03T09:00:00+01:00 anna: changed — state reviewing\n- history: 2026-11-04T16:30:00+01:00 anna: changed — state done\n\n## docs/Prüfstand notes.docx\n\n- state: reviewing\n- history: 2026-11-05T10:00:00+01:00 ben: changed — state reviewing\n\n## old/minutes.txt\n\n- state: nothing\n- history: 2026-11-06T11:20:00+01:00 ben: changed — state nothing\n";

    /// A dossier folder with the given imports file, or none at all when `text` is None. The
    /// folder's name has a space and a non-ASCII letter.
    fn dossier_with(text: Option<&str>) -> TempDir {
        let dir = tempfile::Builder::new()
            .prefix("dossier Prüfung ")
            .tempdir()
            .expect("a temporary folder");
        if let Some(text) = text {
            fs::write(dir.path().join(FILE), text).expect("writes the imports file");
        }
        dir
    }

    #[test]
    fn a_missing_file_is_empty() {
        let dir = dossier_with(None);
        let (file, bytes) = load(dir.path(), "P-003").expect("loads");
        assert!(bytes.is_none());
        assert!(file.keys().is_empty());
        assert_eq!(
            file.text(),
            "<!-- dossier-format: 1 -->\n# Documents of P-003\n\n"
        );
        assert_eq!(state(&file, "docs/spec-rev1.pdf"), None);
    }

    #[test]
    fn a_reviewed_document_has_its_state() {
        let dir = dossier_with(Some(IMPORTS));
        let (file, bytes) = load(dir.path(), "P-003").expect("loads");
        assert_eq!(bytes.as_deref(), Some(IMPORTS.as_bytes()));
        assert_eq!(state(&file, "docs/spec-rev1.pdf").as_deref(), Some("done"));
        assert_eq!(
            state(&file, "docs/Prüfstand notes.docx").as_deref(),
            Some("reviewing")
        );
        assert_eq!(state(&file, "old/minutes.txt").as_deref(), Some("nothing"));
    }

    #[test]
    fn a_document_that_is_not_listed_has_no_state() {
        let dir = dossier_with(Some(IMPORTS));
        let (file, _) = load(dir.path(), "P-003").expect("loads");
        assert_eq!(state(&file, "docs/never-listed.pdf"), None);
        assert_eq!(state(&file, "docs/spec-rev"), None);
    }

    #[test]
    fn a_listed_document_without_a_state_reads_as_empty() {
        let text = "<!-- dossier-format: 1 -->\n# Documents of P-003\n\n## docs/draft.pdf\n\n- history: 2026-11-07T08:00:00+01:00 anna: changed — listed\n";
        let dir = dossier_with(Some(text));
        let (file, _) = load(dir.path(), "P-003").expect("loads");
        assert_eq!(state(&file, "docs/draft.pdf").as_deref(), Some(""));
    }

    #[test]
    fn a_file_that_is_not_utf8_is_a_parse_error() {
        let dir = dossier_with(None);
        fs::write(dir.path().join(FILE), [0x23, 0x20, 0xff, 0xfe, 0x0a]).expect("writes");
        let error = load(dir.path(), "P-003").expect_err("refused");
        assert_eq!(error.code, Code::Parse);
    }

    #[test]
    fn a_file_too_new_for_this_program_is_refused() {
        let text = "<!-- dossier-format: 2 -->\n# Documents of P-003\n";
        let dir = dossier_with(Some(text));
        let error = load(dir.path(), "P-003").expect_err("refused");
        assert_eq!(error.code, Code::TooNew);
    }
}
