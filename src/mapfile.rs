//! The map of a code project (format.md §17, map.md): one block per described folder or package.

use crate::blockfile::BlockFile;
use crate::error::{Code, Error, Result};
use std::path::Path;

/// The file of a dossier that holds the map, `map.md`.
pub const FILE: &str = "map.md";

/// The item name of the map file in snapshots and the journal.
pub const ITEM: &str = "MAP";

/// The map file of a project and its bytes as read; a missing file is
/// BlockFile::empty("Map of <code>"), None.
pub fn load(dossier: &Path, code: &str) -> Result<(BlockFile, Option<Vec<u8>>)> {
    let path = dossier.join(FILE);
    let bytes = match std::fs::read(&path) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok((BlockFile::empty(&format!("Map of {code}")), None));
        }
        Err(error) => return Err(error.into()),
    };
    let text = std::str::from_utf8(&bytes)
        .map_err(|_| Error::new(Code::Parse, format!("{} is not UTF-8 text", path.display())))?;
    let file = BlockFile::parse(text).map_err(|error| {
        Error::new(error.code, format!("{}: {}", path.display(), error.message))
    })?;
    Ok((file, Some(bytes)))
}

/// The map as lines `<path>  <purpose>[ (start: <start>)]`, sorted by path, detached left out.
pub fn lines(file: &BlockFile) -> Vec<String> {
    let mut keys: Vec<String> = file
        .keys()
        .into_iter()
        .filter(|key| !key.is_empty())
        .collect();
    keys.sort();
    keys.dedup();
    keys.iter()
        .filter(|key| present(file, key, "detached").is_none())
        .map(|key| line(file, key))
        .collect()
}

/// The text of a field of a block when it is not empty: format.md §5 leaves an empty field out.
fn present(file: &BlockFile, key: &str, name: &str) -> Option<String> {
    file.field(key, name).filter(|value| !value.is_empty())
}

/// One line of the map: the path, then `  <purpose>` when there is a purpose, then
/// ` (start: <start>)` when there is a start.
fn line(file: &BlockFile, key: &str) -> String {
    let mut text = match present(file, key, "purpose") {
        Some(purpose) => format!("{key}  {purpose}"),
        None => key.to_string(),
    };
    if let Some(start) = present(file, key, "start") {
        text = format!("{text} (start: {start})");
    }
    text
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A map with two described folders, one detached folder, and the blocks not in path order.
    const EXAMPLE: &str = r#"<!-- dossier-format: 1 -->
# Map of P-001

## src/solver

- purpose: time integration and the linear solver
- start: src/solver/mod.rs
- history: 2026-11-20T10:00:00+01:00 anna via anna-claude: set

## old/mesher

- purpose: former mesher, not used any more
- detached: 2026-11-21T09:00:00+01:00 by anna
- history: 2026-11-20T10:00:00+01:00 anna via anna-claude: set
- history: 2026-11-21T09:00:00+01:00 anna: detached

## src/io/readers

- purpose: reading the input decks
- history: 2026-11-20T10:05:00+01:00 anna via anna-claude: set
"#;

    #[test]
    fn lines_list_the_folders_sorted_by_path_and_leave_out_the_detached_one() {
        let file = BlockFile::parse(EXAMPLE).expect("the example parses");
        assert_eq!(
            lines(&file),
            [
                "src/io/readers  reading the input decks",
                "src/solver  time integration and the linear solver (start: src/solver/mod.rs)",
            ]
        );
    }

    #[test]
    fn a_folder_without_purpose_gives_its_path_and_its_start() {
        let file = BlockFile::parse(
            "# Map of P-001\n\n## tools/Prüf läufe\n\n- start: tools/Prüf läufe/README.md\n\n## docs\n\n- purpose: the manuals\n",
        )
        .expect("the example parses");
        assert_eq!(
            lines(&file),
            [
                "docs  the manuals",
                "tools/Prüf läufe (start: tools/Prüf läufe/README.md)",
            ]
        );
    }

    #[test]
    fn a_missing_map_file_is_an_empty_map_without_bytes() {
        let dossier = tempfile::tempdir().expect("temporary folder");
        let (file, bytes) = load(dossier.path(), "P-001").expect("loads");
        assert_eq!(
            file.text(),
            "<!-- dossier-format: 1 -->\n# Map of P-001\n\n"
        );
        assert!(bytes.is_none());
        assert!(lines(&file).is_empty());
    }

    #[test]
    fn an_existing_map_file_is_read_with_its_bytes_in_a_folder_with_a_space_and_a_non_ascii_letter()
    {
        let dossier = tempfile::Builder::new()
            .prefix("Projekt Größe ")
            .tempdir()
            .expect("temporary folder");
        std::fs::write(dossier.path().join(FILE), EXAMPLE).expect("the map is written");
        let (file, bytes) = load(dossier.path(), "P-001").expect("loads");
        assert_eq!(file.text(), EXAMPLE);
        assert_eq!(bytes.as_deref(), Some(EXAMPLE.as_bytes()));
    }

    #[test]
    fn a_map_file_that_is_not_text_or_needs_a_newer_program_is_refused() {
        let dossier = tempfile::tempdir().expect("temporary folder");
        std::fs::write(dossier.path().join(FILE), [0xff, 0xfe, b'#', b' '])
            .expect("the map is written");
        let error = load(dossier.path(), "P-001").expect_err("not UTF-8");
        assert_eq!(error.code, Code::Parse);

        std::fs::write(
            dossier.path().join(FILE),
            "<!-- dossier-format: 2 -->\n# Map of P-001\n",
        )
        .expect("the map is written");
        let error = load(dossier.path(), "P-001").expect_err("too new");
        assert_eq!(error.code, Code::TooNew);
        assert_eq!(error.exit(), 4);
    }
}
