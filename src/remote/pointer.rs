//! The pointer file: a project folder's record of the server folder its dossier is kept in.

use crate::error::Result;
use serde::Deserialize;
use std::path::{Path, PathBuf};

/// The name of the pointer file in a connected folder (format.md §12).
const FILE_NAME: &str = ".dossier-connect";

/// Where the dossier of a project lives on a server.
pub struct Pointer {
    pub server: String,
    pub folder: String,
    pub code: String,
}

/// The pointer file as read: format.md §12. Unknown keys are errors.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PointerFile {
    server: String,
    folder: String,
    code: String,
}

/// The pointer file found walking up from `start`, and the folder it is in. `start` itself is
/// checked first. A file that does not parse counts as not found, so the walk goes on upwards.
pub fn find(start: &Path) -> Option<(PathBuf, Pointer)> {
    start.ancestors().find_map(|folder| {
        let text = std::fs::read_to_string(folder.join(FILE_NAME)).ok()?;
        let file = parse(&text)?;
        Some((
            folder.to_path_buf(),
            Pointer {
                server: file.server,
                folder: file.folder,
                code: file.code,
            },
        ))
    })
}

/// Writes the pointer file into `folder`: three TOML lines, `server`, `folder` and `code`, each
/// a quoted string, with LF line endings.
pub fn write(folder: &Path, pointer: &Pointer) -> Result<()> {
    let text = format!(
        "server = {}\nfolder = {}\ncode = {}\n",
        quote(&pointer.server),
        quote(&pointer.folder),
        quote(&pointer.code),
    );
    crate::atomic::replace(&folder.join(FILE_NAME), text.as_bytes(), None)
}

/// The pointer in `text`, or None when the text does not parse.
fn parse(text: &str) -> Option<PointerFile> {
    // format.md §2: a byte-order mark at the start is accepted, as in the settings file.
    let text = text.strip_prefix('\u{feff}').unwrap_or(text);
    toml::from_str(text).ok()
}

/// `text` as a TOML basic string: in double quotes, with `\` and `"` escaped and control
/// characters written as escapes. Other characters, such as non-ASCII letters, stay as they are.
fn quote(text: &str) -> String {
    let mut quoted = String::with_capacity(text.len() + 2);
    quoted.push('"');
    for c in text.chars() {
        match c {
            '"' => quoted.push_str("\\\""),
            '\\' => quoted.push_str("\\\\"),
            '\u{8}' => quoted.push_str("\\b"),
            '\t' => quoted.push_str("\\t"),
            '\n' => quoted.push_str("\\n"),
            '\u{c}' => quoted.push_str("\\f"),
            '\r' => quoted.push_str("\\r"),
            c if c.is_control() => quoted.push_str(&format!("\\u{:04X}", c as u32)),
            c => quoted.push(c),
        }
    }
    quoted.push('"');
    quoted
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pointer() -> Pointer {
        Pointer {
            server: "anna@sim01".into(),
            folder: "/projects/P-017".into(),
            code: "P-017".into(),
        }
    }

    /// The three values of a pointer, to compare without `PartialEq`.
    fn values(pointer: &Pointer) -> (&str, &str, &str) {
        (&pointer.server, &pointer.folder, &pointer.code)
    }

    #[test]
    fn the_file_has_three_lf_terminated_lines() {
        let folder = tempfile::tempdir().unwrap();
        write(folder.path(), &pointer()).unwrap();
        let bytes = std::fs::read(folder.path().join(".dossier-connect")).unwrap();
        assert_eq!(
            bytes,
            b"server = \"anna@sim01\"\nfolder = \"/projects/P-017\"\ncode = \"P-017\"\n"
        );
    }

    #[test]
    fn found_from_a_subfolder_with_the_same_values_and_folder() {
        let root = tempfile::tempdir().unwrap();
        write(root.path(), &pointer()).unwrap();
        let sub = root.path().join("results").join("monitors");
        std::fs::create_dir_all(&sub).unwrap();
        let (folder, found) = find(&sub).unwrap();
        assert_eq!(folder, root.path());
        assert_eq!(values(&found), values(&pointer()));
    }

    #[test]
    fn found_in_the_folder_itself() {
        let root = tempfile::tempdir().unwrap();
        write(root.path(), &pointer()).unwrap();
        let (folder, found) = find(root.path()).unwrap();
        assert_eq!(folder, root.path());
        assert_eq!(values(&found), values(&pointer()));
    }

    #[test]
    fn writing_again_replaces_the_values() {
        let root = tempfile::tempdir().unwrap();
        write(root.path(), &pointer()).unwrap();
        let other = Pointer {
            server: "ben@sim02".into(),
            folder: "/work/P-020".into(),
            code: "P-020".into(),
        };
        write(root.path(), &other).unwrap();
        let (_, found) = find(root.path()).unwrap();
        assert_eq!(values(&found), values(&other));
    }

    #[test]
    fn a_project_folder_with_spaces_and_non_ascii_names_works() {
        let root = tempfile::tempdir().unwrap();
        let project = root.path().join("Projekt Übersicht").join("Größe");
        write(&project, &pointer()).unwrap();
        let sub = project.join("notizen");
        std::fs::create_dir_all(&sub).unwrap();
        let (folder, found) = find(&sub).unwrap();
        assert_eq!(folder, project);
        assert_eq!(values(&found), values(&pointer()));
    }

    #[test]
    fn quotes_backslashes_and_control_characters_survive() {
        let root = tempfile::tempdir().unwrap();
        let tricky = Pointer {
            server: "anna@sim01".into(),
            folder: "C:\\Projekte\\\"P-017\" Übersicht\\tab\tend\u{1}".into(),
            code: "P-017".into(),
        };
        write(root.path(), &tricky).unwrap();
        let (_, found) = find(root.path()).unwrap();
        assert_eq!(values(&found), values(&tricky));
    }

    #[test]
    fn no_file_gives_none() {
        let root = tempfile::tempdir().unwrap();
        let sub = root.path().join("sub");
        std::fs::create_dir_all(&sub).unwrap();
        assert!(find(&sub).is_none());
    }

    #[test]
    fn a_broken_file_gives_none() {
        let root = tempfile::tempdir().unwrap();
        let broken = [
            "server = \"anna@sim01\"\n",
            "server = \"anna@sim01\"\nfolder = \"/projects/P-017\"\ncode = \"P-017\"\nextra = \"x\"\n",
            "server = anna@sim01\nfolder = \"/projects/P-017\"\ncode = \"P-017\"\n",
            "this is not toml",
        ];
        for text in broken {
            std::fs::write(root.path().join(".dossier-connect"), text).unwrap();
            assert!(find(root.path()).is_none(), "{text}");
        }
    }

    #[test]
    fn a_broken_file_counts_as_not_found_so_the_walk_goes_on() {
        let root = tempfile::tempdir().unwrap();
        write(root.path(), &pointer()).unwrap();
        let sub = root.path().join("sub");
        std::fs::create_dir_all(&sub).unwrap();
        std::fs::write(sub.join(".dossier-connect"), "not toml").unwrap();
        let (folder, found) = find(&sub).unwrap();
        assert_eq!(folder, root.path());
        assert_eq!(values(&found), values(&pointer()));
    }

    #[test]
    fn a_byte_order_mark_is_accepted() {
        let root = tempfile::tempdir().unwrap();
        let text =
            "\u{feff}server = \"anna@sim01\"\nfolder = \"/projects/P-017\"\ncode = \"P-017\"\n";
        std::fs::write(root.path().join(".dossier-connect"), text).unwrap();
        let (_, found) = find(root.path()).unwrap();
        assert_eq!(values(&found), values(&pointer()));
    }
}
