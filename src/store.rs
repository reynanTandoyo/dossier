//! Reading the entry files of a dossier.

use crate::entry::{Entry, Kind};
use crate::error::{Code, Error, Result};
use crate::paths;
use std::path::{Path, PathBuf};

/// One entry file as it was read: where it is, its exact bytes, and the parsed entry.
pub struct Item {
    pub path: PathBuf,
    pub raw: Vec<u8>,
    pub entry: Entry,
}

/// Every entry file of the dossier, sorted by ID: by the number after the prefix, an ID with no
/// number last, then by prefix. Only the files directly inside each kind's folder whose name ends
/// with `.md` are read, and a missing folder is empty. A file that cannot be read, is not UTF-8 or
/// fails to parse is skipped and gives the warning `<path>: <message>`; the message of a parse
/// error starts with `line <n>: `.
pub fn all(dossier: &Path) -> (Vec<Item>, Vec<String>) {
    let mut warnings = Vec::new();
    let mut found = Vec::new();
    for kind in Kind::all() {
        let folder = dossier.join(kind.folder());
        let listing = match std::fs::read_dir(&folder) {
            Ok(listing) => listing,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(error) => {
                warnings.push(format!("{}: {error}", folder.display()));
                continue;
            }
        };
        for dir_entry in listing {
            match dir_entry {
                Ok(dir_entry) => {
                    let path = dir_entry.path();
                    let name = dir_entry.file_name().to_string_lossy().into_owned();
                    if name.ends_with(".md") && path.is_file() {
                        found.push(path);
                    }
                }
                Err(error) => warnings.push(format!("{}: {error}", folder.display())),
            }
        }
    }
    // The order of the warnings does not depend on the order the system lists folders in.
    found.sort();

    let mut items = Vec::new();
    for path in found {
        match read_item(&path) {
            Ok(item) => items.push(item),
            Err(warning) => warnings.push(warning),
        }
    }
    items.sort_by_cached_key(|item| (id_key(&item.entry.id()), item.path.clone()));
    (items, warnings)
}

/// By ID, with or without a place prefix (`P-017:DEC-012`), any case. Code::NotFound. When two
/// files give the same ID, the first of them in the order of `all` is returned.
pub fn get(dossier: &Path, id: &str) -> Result<Item> {
    let wanted = id
        .rsplit_once(':')
        .map_or(id, |(_, rest)| rest)
        .to_uppercase();
    let (items, _) = all(dossier);
    items
        .into_iter()
        .find(|item| item.entry.id() == wanted)
        .ok_or_else(|| Error::new(Code::NotFound, format!("no entry {wanted}")))
}

/// The path of the file of an entry with this kind, ID and title: `<folder>/<ID>-<slug>.md`
/// (format.md §4).
pub fn path_for(dossier: &Path, kind: Kind, id: &str, title: &str) -> PathBuf {
    dossier
        .join(kind.folder())
        .join(format!("{id}-{}.md", paths::slug(title)))
}

/// Reads and parses one entry file. The error is the warning for that file.
fn read_item(path: &Path) -> std::result::Result<Item, String> {
    let raw = std::fs::read(path).map_err(|error| format!("{}: {error}", path.display()))?;
    let text = std::str::from_utf8(&raw).map_err(|_| format!("{}: not UTF-8", path.display()))?;
    let entry =
        Entry::parse(text).map_err(|error| format!("{}: {}", path.display(), error.message))?;
    Ok(Item {
        path: path.to_path_buf(),
        raw,
        entry,
    })
}

/// The sort key of an ID: the number after the prefix (an ID with no number sorts last), then
/// the prefix.
fn id_key(id: &str) -> (bool, u64, String) {
    let (prefix, rest) = id.split_once('-').unwrap_or((id, ""));
    let digits: String = rest.chars().take_while(char::is_ascii_digit).collect();
    match digits.parse::<u64>() {
        Ok(number) => (false, number, prefix.to_string()),
        Err(_) => (true, 0, prefix.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    /// A dossier folder with its four kind folders. It sits in a temporary folder whose name has
    /// a space and a non-ASCII letter, so that the paths of the tests have both.
    fn dossier() -> (TempDir, PathBuf) {
        let temp = tempfile::tempdir().expect("temporary folder");
        let dossier = temp.path().join("Prüf stand").join("DOSSIER");
        for kind in Kind::all() {
            fs::create_dir_all(dossier.join(kind.folder())).expect("kind folder");
        }
        (temp, dossier)
    }

    /// Writes a file at a path relative to the dossier, given with `/`, and returns its full path
    /// with the separators of this system. Folders on the way are created.
    fn put(dossier: &Path, relative: &str, bytes: &[u8]) -> PathBuf {
        let path = relative
            .split('/')
            .fold(dossier.to_path_buf(), |path, part| path.join(part));
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("folder");
        }
        fs::write(&path, bytes).expect("write test file");
        path
    }

    /// An entry file in the form of format.md §1, with an invented statement.
    fn entry_text(heading: &str, kind: &str) -> String {
        format!(
            "<!-- dossier-format: 1 -->\n# {heading}\n\n- kind: {kind}\n- status: approved\n- created: 2026-11-20T09:00:00+01:00\n- author: tess\n\n## Statement\n\nA statement written for a test.\n"
        )
    }

    #[test]
    fn all_reads_every_folder_in_id_order() {
        let (_temp, dossier) = dossier();
        let files = [
            (
                "decisions/DEC-1000-four-digits.md",
                "DEC-1000 · Four digits",
                "decision",
            ),
            (
                "decisions/DEC-999-three-digits.md",
                "DEC-999 · Three digits",
                "decision",
            ),
            ("decisions/DEC-011-größe.md", "DEC-011 · Größe", "decision"),
            (
                "decisions/DEC-002-second.md",
                "DEC-002 · Second",
                "decision",
            ),
            ("decisions/misc.md", "DEC · Without a number", "decision"),
            (
                "findings/FND-001-finding.md",
                "FND-001 · A finding",
                "finding",
            ),
            ("lessons/LES-001-lesson.md", "LES-001 · A lesson", "lesson"),
            (
                "assumptions/ASM-001-assumption.md",
                "ASM-001 · An assumption",
                "assumption",
            ),
        ];
        for (relative, heading, kind) in files {
            put(&dossier, relative, entry_text(heading, kind).as_bytes());
        }

        let (items, warnings) = all(&dossier);

        assert!(warnings.is_empty(), "{warnings:?}");
        let ids: Vec<String> = items.iter().map(|item| item.entry.id()).collect();
        assert_eq!(
            ids,
            [
                "ASM-001", "FND-001", "LES-001", "DEC-002", "DEC-011", "DEC-999", "DEC-1000", "DEC"
            ]
        );
        assert_eq!(items[3].path, dossier.join("decisions/DEC-002-second.md"));
        for item in &items {
            assert_eq!(
                item.raw,
                item.entry.text().as_bytes(),
                "{}",
                item.entry.id()
            );
        }
    }

    #[test]
    fn a_broken_file_gives_a_warning_and_the_others_still_load() {
        let (_temp, dossier) = dossier();
        let good = put(
            &dossier,
            "decisions/DEC-001-good.md",
            entry_text("DEC-001 · Good", "decision").as_bytes(),
        );
        let no_heading = put(
            &dossier,
            "decisions/DEC-002-broken.md",
            b"<!-- dossier-format: 1 -->\nthis file has no heading\n",
        );
        let not_utf8 = put(
            &dossier,
            "findings/FND-003-bytes.md",
            b"# FND-003 \xff\xfe\n",
        );

        let (items, warnings) = all(&dossier);

        let ids: Vec<String> = items.iter().map(|item| item.entry.id()).collect();
        assert_eq!(ids, ["DEC-001"]);
        assert_eq!(items[0].path, good);
        assert_eq!(
            warnings,
            [
                format!("{}: line 2: no heading", no_heading.display()),
                format!("{}: not UTF-8", not_utf8.display()),
            ]
        );
    }

    #[test]
    fn only_md_files_directly_inside_a_kind_folder_are_read() {
        let (_temp, dossier) = dossier();
        put(&dossier, "decisions/notes.txt", b"not an entry\n");
        put(
            &dossier,
            "decisions/DEC-003-leftover.md.tmp-4242",
            entry_text("DEC-003 · Leftover", "decision").as_bytes(),
        );
        put(
            &dossier,
            "decisions/archive/DEC-004-nested.md",
            entry_text("DEC-004 · Nested", "decision").as_bytes(),
        );
        put(&dossier, "dossier.toml", b"format = 1\ncode = \"P-001\"\n");
        fs::create_dir_all(dossier.join("decisions/DEC-006-folder.md")).expect("a folder");
        put(
            &dossier,
            "decisions/DEC-005-real.md",
            entry_text("DEC-005 · Real", "decision").as_bytes(),
        );

        let (items, warnings) = all(&dossier);

        let ids: Vec<String> = items.iter().map(|item| item.entry.id()).collect();
        assert_eq!(ids, ["DEC-005"]);
        assert!(warnings.is_empty(), "{warnings:?}");
    }

    #[test]
    fn a_missing_folder_or_dossier_is_empty_without_a_warning() {
        let (_temp, dossier) = dossier();
        fs::remove_dir(dossier.join("lessons")).expect("remove the lessons folder");
        put(
            &dossier,
            "decisions/DEC-001-only.md",
            entry_text("DEC-001 · Only", "decision").as_bytes(),
        );
        let (items, warnings) = all(&dossier);
        assert_eq!(items.len(), 1);
        assert!(warnings.is_empty(), "{warnings:?}");

        let missing = dossier.join("no-such-dossier");
        let (items, warnings) = all(&missing);
        assert!(items.is_empty());
        assert!(warnings.is_empty(), "{warnings:?}");
    }

    #[test]
    fn get_finds_an_entry_in_any_case_with_or_without_a_place_prefix() {
        let (_temp, dossier) = dossier();
        let decision = put(
            &dossier,
            "decisions/DEC-001-first.md",
            entry_text("DEC-001 · First", "decision").as_bytes(),
        );
        put(
            &dossier,
            "assumptions/ASM-003-second.md",
            entry_text("ASM-003 · Second", "assumption").as_bytes(),
        );

        for id in ["DEC-001", "dec-001", "P-001:DEC-001", "p-017:dec-001"] {
            let item = get(&dossier, id).expect("the entry is found");
            assert_eq!(item.entry.id(), "DEC-001", "{id}");
            assert_eq!(item.path, decision, "{id}");
        }
        let item = get(&dossier, "P-001:asm-003").expect("the assumption is found");
        assert_eq!(item.entry.title(), "Second");
    }

    #[test]
    fn an_unknown_id_is_not_found() {
        let (_temp, dossier) = dossier();
        put(
            &dossier,
            "decisions/DEC-001-first.md",
            entry_text("DEC-001 · First", "decision").as_bytes(),
        );

        for id in ["DEC-999", "p-001:dec-999"] {
            let error = get(&dossier, id).err().expect("no entry is found");
            assert_eq!(error.code, Code::NotFound, "{id}");
            assert_eq!(error.message, "no entry DEC-999", "{id}");
        }
    }

    #[test]
    fn path_for_names_the_file_by_kind_id_and_slug_and_all_finds_it_there() {
        let (_temp, dossier) = dossier();
        assert_eq!(
            path_for(
                &dossier,
                Kind::Finding,
                "FND-007",
                "Halving the time step: no help!"
            ),
            dossier
                .join("findings")
                .join("FND-007-halving-the-time-step-no-help.md")
        );
        assert_eq!(
            path_for(&dossier, Kind::Lesson, "LES-004", "   "),
            dossier.join("lessons").join("LES-004-entry.md")
        );

        let path = path_for(&dossier, Kind::Decision, "DEC-010", "Use the rig");
        fs::write(&path, entry_text("DEC-010 · Use the rig", "decision")).expect("write entry");
        let (items, warnings) = all(&dossier);
        assert!(warnings.is_empty(), "{warnings:?}");
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].path, path);
    }
}
