//! Scans the folder of a finished project for documents to bring in (R-IMPORT-2).
//!
//! The scan reads names, sizes and times, never the contents of a file. Symbolic links and
//! junctions are left out and never followed, and the folders named DOSSIER, .git and target
//! are not entered at any level.

use crate::error::{Code, Error, Result};
use crate::filesum;
use chrono::{DateTime, FixedOffset};
use std::cmp::Reverse;
use std::path::{Path, PathBuf};

/// One document found in a folder.
pub struct Doc {
    pub stored: String,
    pub size: u64,
    pub modified: String,
    pub kind: String,
}

/// The folders the scan does not enter: the dossier, version control and build output.
const SKIPPED: [&str; 3] = ["DOSSIER", ".git", "target"];

/// The folder tree to `depth` levels as indented lines `<name>/` and `<name>  <size>`.
/// Folders come first, then files, each group in plain string order; two spaces of indent per
/// level; sizes in bytes. Depth 1 lists the entries of `root`, depth 2 also the entries of its
/// folders, and so on. A folder on the last level is listed but not opened; depth 0 lists none.
pub fn tree(root: &Path, depth: usize) -> Result<Vec<String>> {
    let mut lines = Vec::new();
    tree_into(root, 0, depth, &mut lines)?;
    Ok(lines)
}

/// Documents found under root (reports .pdf .docx .doc .odt .md .txt, presentations .pptx
/// .ppt, spreadsheets .xlsx .xls .csv, scripts .py .java .m .sh .bat .ps1), newest first.
/// `stored` is the path relative to root with `/`. Equal times are ordered by `stored`. The
/// extension is matched in any case.
pub fn documents(root: &Path) -> Result<Vec<Doc>> {
    let mut found = Vec::new();
    // The folders still to read, each with its stored path ("" for root).
    let mut pending = vec![(root.to_path_buf(), String::new())];
    while let Some((folder, stored)) = pending.pop() {
        for child in children(&folder)? {
            match child {
                Child::Folder { name, path } => pending.push((path, join(&stored, &name))),
                Child::File { name, path, .. } => {
                    let Some(kind) = kind_of(&name) else {
                        continue;
                    };
                    let stamp = filesum::stamp(&path)?;
                    found.push(Doc {
                        stored: join(&stored, &name),
                        size: stamp.size,
                        modified: stamp.modified,
                        kind: kind.to_string(),
                    });
                }
            }
        }
    }
    found.sort_by_cached_key(|doc| (Reverse(instant(&doc.modified)), doc.stored.clone()));
    Ok(found)
}

/// An entry of a folder that the scan looks at.
enum Child {
    /// A folder that is entered.
    Folder { name: String, path: PathBuf },
    /// A regular file, with its size in bytes.
    File {
        name: String,
        path: PathBuf,
        size: u64,
    },
}

impl Child {
    fn name(&self) -> &str {
        match self {
            Child::Folder { name, .. } | Child::File { name, .. } => name.as_str(),
        }
    }

    fn is_file(&self) -> bool {
        matches!(self, Child::File { .. })
    }
}

/// The folders and regular files directly inside `folder`, folders first and each group in plain
/// string order. Symbolic links, other entries and the SKIPPED folders are left out.
fn children(folder: &Path) -> Result<Vec<Child>> {
    let mut found = Vec::new();
    let entries = std::fs::read_dir(folder).map_err(|error| io_error(folder, error))?;
    for entry in entries {
        let entry = entry.map_err(|error| io_error(folder, error))?;
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().into_owned();
        let metadata = std::fs::symlink_metadata(&path).map_err(|error| io_error(&path, error))?;
        if metadata.is_dir() {
            if !SKIPPED.contains(&name.as_str()) {
                found.push(Child::Folder { name, path });
            }
        } else if metadata.is_file() {
            found.push(Child::File {
                name,
                path,
                size: metadata.len(),
            });
        }
    }
    found.sort_by(|a, b| {
        a.is_file()
            .cmp(&b.is_file())
            .then_with(|| a.name().cmp(b.name()))
    });
    Ok(found)
}

/// Adds the lines of the entries of `folder`, which lies `level` levels below root. Its entries
/// are listed only while `level` is less than `depth`, so a folder on the last level is not read.
fn tree_into(folder: &Path, level: usize, depth: usize, lines: &mut Vec<String>) -> Result<()> {
    if level >= depth {
        return Ok(());
    }
    let indent = "  ".repeat(level);
    for child in children(folder)? {
        match child {
            Child::Folder { name, path } => {
                lines.push(format!("{indent}{name}/"));
                tree_into(&path, level + 1, depth, lines)?;
            }
            Child::File { name, size, .. } => lines.push(format!("{indent}{name}  {size}")),
        }
    }
    Ok(())
}

/// The kind of a document by its extension, in any case, or None when it is not a document.
fn kind_of(name: &str) -> Option<&'static str> {
    let extension = Path::new(name).extension()?.to_str()?.to_ascii_lowercase();
    Some(match extension.as_str() {
        "pdf" | "docx" | "doc" | "odt" | "md" | "txt" => "report",
        "pptx" | "ppt" => "presentation",
        "xlsx" | "xls" | "csv" => "spreadsheet",
        "py" | "java" | "m" | "sh" | "bat" | "ps1" => "script",
        _ => return None,
    })
}

/// The stored path of `name` inside the folder with stored path `stored` ("" for root).
fn join(stored: &str, name: &str) -> String {
    if stored.is_empty() {
        name.to_string()
    } else {
        format!("{stored}/{name}")
    }
}

/// The instant of a time stamp (format.md §7), or None when it does not parse. Comparing
/// instants rather than text keeps the order right across a change of daylight saving time.
fn instant(stamp: &str) -> Option<DateTime<FixedOffset>> {
    DateTime::parse_from_rfc3339(stamp).ok()
}

/// A failure on `path` as Code::Io, with the path in the message.
fn io_error(path: &Path, error: std::io::Error) -> Error {
    Error::new(Code::Io, format!("{}: {error}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, SystemTime};

    /// Writes `bytes` to `path`, making its folders first.
    fn write(path: &Path, bytes: &[u8]) {
        std::fs::create_dir_all(path.parent().expect("a folder for the file"))
            .expect("create the folders");
        std::fs::write(path, bytes).expect("write a test file");
    }

    /// Sets the modification time of `path` to `seconds` after the Unix epoch.
    fn set_time(path: &Path, seconds: u64) {
        let when = SystemTime::UNIX_EPOCH + Duration::from_secs(seconds);
        std::fs::OpenOptions::new()
            .write(true)
            .open(path)
            .expect("open a test file")
            .set_modified(when)
            .expect("set the modification time");
    }

    /// The stored paths of `docs`, in order.
    fn stored(docs: &[Doc]) -> Vec<String> {
        docs.iter().map(|doc| doc.stored.clone()).collect()
    }

    /// Makes `folder` with a document and a sub-folder holding another, both inside a folder the
    /// scan must not enter.
    fn make_skipped(folder: &Path) {
        write(&folder.join("inside.md"), b"entry");
        write(&folder.join("deeper").join("inside.py"), b"print(1)");
    }

    #[test]
    fn tree_lists_folders_then_files_in_plain_string_order() {
        let dir = tempfile::tempdir().expect("temporary folder");
        let root = dir.path();
        std::fs::create_dir_all(root.join("beta")).expect("folder");
        std::fs::create_dir_all(root.join("Alpha")).expect("folder");
        write(&root.join("c.txt"), &[0u8; 1234]);
        write(&root.join("a.txt"), b"0123456789");
        write(&root.join("B.txt"), b"abc");

        // Plain string order: the upper-case B comes before the lower-case a.
        assert_eq!(
            tree(root, 1).expect("tree"),
            ["Alpha/", "beta/", "B.txt  3", "a.txt  10", "c.txt  1234"]
        );
    }

    #[test]
    fn tree_indents_two_spaces_per_level_and_stops_at_the_depth() {
        let dir = tempfile::tempdir().expect("temporary folder");
        let root = dir.path();
        write(&root.join("root.txt"), b"abc");
        write(&root.join("top").join("side.txt"), b"ab");
        write(&root.join("top").join("mid").join("low.txt"), b"1234567");

        assert!(tree(root, 0).expect("tree").is_empty());
        assert_eq!(tree(root, 1).expect("tree"), ["top/", "root.txt  3"]);
        // mid/ is on the last level: listed, not opened.
        assert_eq!(
            tree(root, 2).expect("tree"),
            ["top/", "  mid/", "  side.txt  2", "root.txt  3"]
        );
        assert_eq!(
            tree(root, 3).expect("tree"),
            [
                "top/",
                "  mid/",
                "    low.txt  7",
                "  side.txt  2",
                "root.txt  3"
            ]
        );
    }

    #[test]
    fn tree_skips_dossier_git_and_target_at_every_level() {
        let dir = tempfile::tempdir().expect("temporary folder");
        let root = dir.path();
        make_skipped(&root.join("DOSSIER"));
        make_skipped(&root.join(".git"));
        make_skipped(&root.join("target"));
        make_skipped(&root.join("sub").join("target"));
        make_skipped(&root.join("sub").join("DOSSIER"));
        write(&root.join("targets.txt"), b"abc");
        write(&root.join("sub").join("keep.txt"), b"ab");

        assert_eq!(
            tree(root, 9).expect("tree"),
            ["sub/", "  keep.txt  2", "targets.txt  3"]
        );
    }

    #[test]
    fn tree_names_with_spaces_and_non_ascii_work() {
        let dir = tempfile::tempdir().expect("temporary folder");
        let root = dir.path();
        write(&root.join("Prüf stand").join("Ergebnis ü.txt"), b"abcd");

        assert_eq!(
            tree(root, 2).expect("tree"),
            ["Prüf stand/", "  Ergebnis ü.txt  4"]
        );
    }

    #[test]
    fn tree_of_a_missing_folder_is_an_io_error_naming_it() {
        let dir = tempfile::tempdir().expect("temporary folder");
        let missing = dir.path().join("missing");
        let error = tree(&missing, 1).expect_err("a missing folder has no tree");
        assert_eq!(error.code, Code::Io);
        assert!(
            error.message.starts_with(&missing.display().to_string()),
            "{}",
            error.message
        );
    }

    #[test]
    fn documents_find_the_four_kinds_by_extension_in_any_case() {
        let dir = tempfile::tempdir().expect("temporary folder");
        let root = dir.path();
        for name in [
            "report.PDF",
            "notes.md",
            "plan.txt",
            "letter.DOCX",
            "deck.pptx",
            "old.PPT",
            "data.XLSX",
            "table.csv",
            "fit.py",
            "kernel.java",
            "model.m",
            "run.ps1",
            "build.bat",
            "go.sh",
            "image.png",
            "model.stl",
            "archive.zip",
            "notes.md.bak",
            "README",
        ] {
            write(&root.join(name), b"x");
        }

        let mut listed: Vec<String> = documents(root)
            .expect("documents")
            .iter()
            .map(|doc| format!("{} {}", doc.stored, doc.kind))
            .collect();
        listed.sort();
        let mut expected = [
            "report.PDF report",
            "notes.md report",
            "plan.txt report",
            "letter.DOCX report",
            "deck.pptx presentation",
            "old.PPT presentation",
            "data.XLSX spreadsheet",
            "table.csv spreadsheet",
            "fit.py script",
            "kernel.java script",
            "model.m script",
            "run.ps1 script",
            "build.bat script",
            "go.sh script",
        ];
        expected.sort();
        assert_eq!(listed, expected);
    }

    #[test]
    fn documents_store_paths_relative_to_root_with_slashes() {
        let dir = tempfile::tempdir().expect("temporary folder");
        let root = dir.path();
        let nested = root
            .join("Projekt final")
            .join("Berichte ü")
            .join("Schluss bericht.docx");
        write(&nested, b"abc");
        write(&root.join("Angebot.doc"), b"abc");
        set_time(&nested, 1_700_000_000);
        set_time(&root.join("Angebot.doc"), 1_700_000_000);

        let docs = documents(root).expect("documents");
        assert_eq!(
            stored(&docs),
            [
                "Angebot.doc",
                "Projekt final/Berichte ü/Schluss bericht.docx"
            ]
        );
        assert!(docs.iter().all(|doc| !doc.stored.contains('\\')));
    }

    #[test]
    fn documents_are_newest_first_then_by_stored_path() {
        let dir = tempfile::tempdir().expect("temporary folder");
        let root = dir.path();
        let files = [
            ("old.pdf", 1_000),
            ("b.md", 2_000),
            ("a.md", 2_000),
            ("new.txt", 3_000),
            ("sub/c.md", 2_000),
        ];
        for (name, seconds) in files {
            let path = root.join(name);
            write(&path, b"x");
            set_time(&path, seconds);
        }

        assert_eq!(
            stored(&documents(root).expect("documents")),
            ["new.txt", "a.md", "b.md", "sub/c.md", "old.pdf"]
        );
    }

    #[test]
    fn documents_skip_dossier_git_and_target_at_every_level() {
        let dir = tempfile::tempdir().expect("temporary folder");
        let root = dir.path();
        make_skipped(&root.join("DOSSIER"));
        make_skipped(&root.join(".git"));
        make_skipped(&root.join("target"));
        make_skipped(&root.join("sub").join("target"));
        make_skipped(&root.join("sub").join("DOSSIER"));
        write(&root.join("targets.md"), b"abc");
        write(&root.join("sub").join("keep.py"), b"ab");

        let mut names = stored(&documents(root).expect("documents"));
        names.sort();
        assert_eq!(names, ["sub/keep.py", "targets.md"]);
    }

    #[test]
    fn documents_report_size_in_bytes_and_never_read_the_contents() {
        let dir = tempfile::tempdir().expect("temporary folder");
        let root = dir.path();
        // Not UTF-8 and not text: a reader that parsed the file would fail or differ.
        write(&root.join("scan.pdf"), &[0xff, 0xfe, 0x00, 0x80]);
        write(&root.join("empty.md"), b"");

        let docs = documents(root).expect("documents");
        let mut sizes: Vec<String> = docs
            .iter()
            .map(|doc| format!("{} {}", doc.stored, doc.size))
            .collect();
        sizes.sort();
        assert_eq!(sizes, ["empty.md 0", "scan.pdf 4"]);
        assert!(
            docs.iter().all(|doc| crate::time::valid(&doc.modified)),
            "times must be in the format of format.md section 7"
        );
    }

    #[test]
    fn documents_of_a_folder_without_documents_are_empty() {
        let dir = tempfile::tempdir().expect("temporary folder");
        let root = dir.path();
        write(&root.join("image.png"), b"x");
        write(&root.join("nested").join("archive.zip"), b"x");
        assert!(documents(root).expect("documents").is_empty());
    }

    #[test]
    fn documents_of_a_missing_folder_is_an_io_error_naming_it() {
        let dir = tempfile::tempdir().expect("temporary folder");
        let missing = dir.path().join("missing");
        let error = documents(&missing)
            .err()
            .expect("a missing folder has no documents");
        assert_eq!(error.code, Code::Io);
        assert!(
            error.message.starts_with(&missing.display().to_string()),
            "{}",
            error.message
        );
    }

    // Making a link on Windows needs Developer Mode or an elevated shell, so the test is ignored
    // there by default. Run it with `-- --ignored` where links are allowed.
    #[cfg_attr(
        windows,
        ignore = "making links on Windows needs Developer Mode or an elevated shell"
    )]
    #[test]
    fn symbolic_links_are_left_out_and_never_followed() {
        let dir = tempfile::tempdir().expect("temporary folder");
        let root = dir.path();
        write(&root.join("real").join("notes.md"), b"abcd");
        let folder_link = root.join("link folder");
        let file_link = root.join("link.md");
        let broken_link = root.join("broken.pdf");
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(root.join("real"), &folder_link).expect("link folder");
            std::os::unix::fs::symlink(root.join("real").join("notes.md"), &file_link)
                .expect("link file");
            std::os::unix::fs::symlink(root.join("missing.pdf"), &broken_link)
                .expect("broken link");
        }
        #[cfg(windows)]
        {
            std::os::windows::fs::symlink_dir(root.join("real"), &folder_link)
                .expect("link folder");
            std::os::windows::fs::symlink_file(root.join("real").join("notes.md"), &file_link)
                .expect("link file");
            std::os::windows::fs::symlink_file(root.join("missing.pdf"), &broken_link)
                .expect("broken link");
        }

        assert_eq!(tree(root, 3).expect("tree"), ["real/", "  notes.md  4"]);
        assert_eq!(
            stored(&documents(root).expect("documents")),
            ["real/notes.md"]
        );
    }

    /// Junctions need no special rights, so this runs on Windows without Developer Mode.
    #[cfg(windows)]
    #[test]
    fn junctions_are_left_out_on_windows() {
        let dir = tempfile::tempdir().expect("temporary folder");
        let root = dir.path();
        write(&root.join("real").join("notes.md"), b"abcd");
        let status = std::process::Command::new("cmd")
            .args(["/C", "mklink", "/J"])
            .arg(root.join("junction"))
            .arg(root.join("real"))
            .status()
            .expect("run cmd");
        assert!(status.success(), "mklink /J failed");

        assert_eq!(tree(root, 3).expect("tree"), ["real/", "  notes.md  4"]);
        assert_eq!(
            stored(&documents(root).expect("documents")),
            ["real/notes.md"]
        );
    }
}
