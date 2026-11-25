//! The files of a task (format.md §17, files/<task>.md): one block per attached path.

use crate::blockfile::BlockFile;
use crate::config::Config;
use crate::error::{Code, Error, Result};
use crate::filesum;
use std::path::{Path, PathBuf};

/// The roles a file of a task can have (format.md §17); the settings may add more.
pub const ROLES: [&str; 10] = [
    "description",
    "geometry",
    "mesh",
    "model",
    "setup",
    "script",
    "result",
    "report",
    "reference",
    "other",
];

/// The task as the list names it: upper-cased, as in `FILES-<TASK>` and `files/<TASK>.md`.
fn name(task: &str) -> String {
    task.to_uppercase()
}

/// The item name of a task's file list in snapshots and the journal: "FILES-<TASK>", with the
/// task upper-cased.
pub fn item(task: &str) -> String {
    format!("FILES-{}", name(task))
}

/// The file of a task's list: dossier/files/<TASK>.md.
pub fn path(dossier: &Path, task: &str) -> PathBuf {
    dossier.join("files").join(format!("{}.md", name(task)))
}

/// The list of a task and its bytes as read; a missing file is BlockFile::empty("Files of
/// <TASK>"), None.
pub fn load(dossier: &Path, task: &str) -> Result<(BlockFile, Option<Vec<u8>>)> {
    let path = path(dossier, task);
    match std::fs::read(&path) {
        Ok(bytes) => {
            let text = std::str::from_utf8(&bytes).map_err(|_| {
                Error::new(Code::Parse, format!("{}: not UTF-8 text", path.display()))
            })?;
            let file = BlockFile::parse(text)?;
            Ok((file, Some(bytes)))
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            Ok((BlockFile::empty(&format!("Files of {}", name(task))), None))
        }
        Err(error) => Err(Error::new(Code::Io, format!("{}: {error}", path.display()))),
    }
}

/// The tasks that have a list (a `.md` file in dossier/files), sorted.
pub fn tasks(dossier: &Path) -> Vec<String> {
    let Ok(entries) = std::fs::read_dir(dossier.join("files")) else {
        return Vec::new();
    };
    let mut tasks: Vec<String> = entries
        .flatten()
        .filter(|entry| entry.file_type().is_ok_and(|kind| kind.is_file()))
        .filter_map(|entry| {
            let file = entry.file_name().into_string().ok()?;
            file.strip_suffix(".md").map(str::to_string)
        })
        .filter(|task| !task.is_empty())
        .collect();
    tasks.sort();
    tasks
}

/// Whether a role is one of ROLES or of the settings' roles, ignoring case.
pub fn valid_role(cfg: &Config, role: &str) -> bool {
    let role = role.to_lowercase();
    ROLES.iter().any(|known| known.to_lowercase() == role)
        || cfg.roles.iter().any(|known| known.to_lowercase() == role)
}

/// The fields describing `stored` now: type, size or files, modified, checksum.
pub fn describe(root: &Path, stored: &str) -> Result<Vec<(String, String)>> {
    let full = root.join(stored);
    let metadata = std::fs::metadata(&full).map_err(|error| match error.kind() {
        std::io::ErrorKind::NotFound => {
            Error::new(Code::NotFound, format!("{stored} does not exist"))
        }
        _ => Error::new(Code::Io, format!("{}: {error}", full.display())),
    })?;
    let stamp = filesum::stamp(&full)?;
    if metadata.is_dir() {
        return Ok(vec![
            pair("type", "folder"),
            pair("files", filesum::count_files(&full)?.to_string()),
            pair("modified", stamp.modified),
        ]);
    }
    if metadata.is_file() {
        return Ok(vec![
            pair("type", "file"),
            pair("size", stamp.size.to_string()),
            pair("modified", stamp.modified),
            pair("checksum", filesum::checksum(&full)?),
        ]);
    }
    Err(Error::new(
        Code::Io,
        format!("{stored} is neither a file nor a folder"),
    ))
}

/// Marks of a block: "detached", "missing", "changed since attached", "outdated" (a made-from
/// checksum differs from that file now), "rests on a wrong assumption".
///
/// A detached block has no other mark. A missing path cannot be changed, so it gets no
/// "changed since attached" mark.
pub fn marks(root: &Path, file: &BlockFile, key: &str) -> Vec<String> {
    if file.field(key, "detached").is_some_and(|at| !at.is_empty()) {
        return vec!["detached".to_string()];
    }
    let mut marks = Vec::new();
    let full = root.join(key);
    if std::fs::metadata(&full).is_err() {
        marks.push("missing");
    } else if changed(file, key, &full) {
        marks.push("changed since attached");
    }
    if outdated(root, file, key) {
        marks.push("outdated");
    }
    if file
        .field(key, "rests-on")
        .is_some_and(|ids| !ids.is_empty())
    {
        marks.push("rests on a wrong assumption");
    }
    marks.into_iter().map(str::to_string).collect()
}

/// A field with its name and value.
fn pair(field: &str, value: impl Into<String>) -> (String, String) {
    (field.to_string(), value.into())
}

/// Whether the file or folder at `full` differs from what the block recorded. A file is compared
/// by size first, and by checksum only when the size is the same. A folder is compared by its
/// number of files. A value that cannot be read gives no mark.
fn changed(file: &BlockFile, key: &str, full: &Path) -> bool {
    let Ok(metadata) = std::fs::metadata(full) else {
        return false;
    };
    if metadata.is_dir() {
        let Some(recorded) = file
            .field(key, "files")
            .and_then(|count| count.parse::<u64>().ok())
        else {
            return false;
        };
        return filesum::count_files(full).is_ok_and(|now| now != recorded);
    }
    let recorded_size = file
        .field(key, "size")
        .and_then(|size| size.parse::<u64>().ok());
    if recorded_size.is_some_and(|size| size != metadata.len()) {
        return true;
    }
    match file.field(key, "checksum") {
        Some(recorded) if !recorded.is_empty() => {
            filesum::checksum(full).is_ok_and(|now| now != recorded)
        }
        _ => false,
    }
}

/// Whether a made-from entry `<stored path>@<checksum>` of the block is outdated: the file is
/// gone, or its checksum is not the recorded one (format.md §17). An entry with no `@` has an
/// empty checksum, which no file has. A folder has no checksum and is not compared.
fn outdated(root: &Path, file: &BlockFile, key: &str) -> bool {
    let Some(list) = file.field(key, "made-from") else {
        return false;
    };
    list.split(',')
        .map(str::trim)
        .filter(|entry| !entry.is_empty())
        .any(|entry| {
            let (stored, recorded) = entry.rsplit_once('@').unwrap_or((entry, ""));
            let path = root.join(stored);
            match std::fs::metadata(&path) {
                Err(_) => true,
                Ok(metadata) if metadata.is_dir() => false,
                Ok(_) => filesum::checksum(&path).ok().as_deref() != Some(recorded),
            }
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Tags;

    /// The ABC digest of format.md §18 (the SHA-256 of the three bytes `abc`).
    const ABC: &str = "sha256:ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad";

    /// Settings with the given roles of the project; the other settings are not used here.
    fn config_with_roles(roles: &[&str]) -> Config {
        Config {
            format: 1,
            code: "P-017".to_string(),
            tags: Tags {
                machine: Vec::new(),
                analysis: Vec::new(),
                software: Vec::new(),
            },
            people: Vec::new(),
            deciders: Vec::new(),
            roles: roles.iter().map(|role| role.to_string()).collect(),
            topics: Vec::new(),
            code_project: false,
            brief_lines: 15,
            status_lines: 10,
            draft_days: 14,
            take_days: 14,
            lock_wait_seconds: 10,
            ssh_timeout_seconds: 10,
            kind: "project".to_string(),
            team: None,
            members: Vec::new(),
            names: Vec::new(),
            past: false,
            describes: None,
        }
    }

    /// Writes `bytes` to `root/stored`, making its folders first.
    fn write(root: &Path, stored: &str, bytes: &[u8]) {
        let path = root.join(stored);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).expect("folders of the test file");
        }
        std::fs::write(&path, bytes).expect("write the test file");
    }

    /// The list of task T-001 with a block for `stored`, as attach writes it: what describe gives
    /// now, then the `extra` fields.
    fn attached(root: &Path, stored: &str, extra: &[(&str, &str)]) -> BlockFile {
        let mut fields = describe(root, stored).expect("the path exists");
        fields.extend(extra.iter().map(|&(field, value)| pair(field, value)));
        let mut file = BlockFile::empty("Files of T-001");
        file.add_block(stored, &fields);
        file
    }

    #[test]
    fn item_and_path_use_the_upper_cased_task() {
        assert_eq!(item("t-001"), "FILES-T-001");
        assert_eq!(
            path(Path::new("dossier"), "t-001"),
            Path::new("dossier").join("files").join("T-001.md")
        );
    }

    #[test]
    fn describe_of_a_file_gives_type_size_modified_and_checksum() {
        let root = tempfile::tempdir().expect("temporary folder");
        write(root.path(), "data/model.sim", b"abc");
        let fields = describe(root.path(), "data/model.sim").expect("describe");
        let names: Vec<&str> = fields.iter().map(|(name, _)| name.as_str()).collect();
        assert_eq!(names, ["type", "size", "modified", "checksum"]);
        assert_eq!(fields[0].1, "file");
        assert_eq!(fields[1].1, "3");
        assert!(crate::time::valid(&fields[2].1), "{}", fields[2].1);
        assert_eq!(fields[3].1, ABC);
    }

    #[test]
    fn describe_of_a_file_with_spaces_and_non_ascii_in_its_path_works() {
        let root = tempfile::tempdir().expect("temporary folder");
        write(root.path(), "Modelle ü/Lauf 1.sim", b"abc");
        let fields = describe(root.path(), "Modelle ü/Lauf 1.sim").expect("describe");
        assert_eq!(fields[1], pair("size", "3"));
        assert_eq!(fields[3], pair("checksum", ABC));
    }

    #[test]
    fn describe_of_a_folder_gives_type_files_and_modified_without_checksum() {
        let root = tempfile::tempdir().expect("temporary folder");
        write(root.path(), "results/run 1/a.csv", b"1");
        write(root.path(), "results/run 1/b.csv", b"2");
        write(root.path(), "results/run 1/sub/c.csv", b"3");
        let fields = describe(root.path(), "results/run 1").expect("describe");
        let names: Vec<&str> = fields.iter().map(|(name, _)| name.as_str()).collect();
        assert_eq!(names, ["type", "files", "modified"]);
        assert_eq!(fields[0], pair("type", "folder"));
        assert_eq!(fields[1], pair("files", "3"));
        assert!(crate::time::valid(&fields[2].1), "{}", fields[2].1);
    }

    #[test]
    fn describe_of_a_missing_path_is_not_found_with_its_path() {
        let root = tempfile::tempdir().expect("temporary folder");
        let error = describe(root.path(), "data/missing.sim").expect_err("no such file");
        assert_eq!(error.code, Code::NotFound);
        assert_eq!(error.message, "data/missing.sim does not exist");
    }

    #[test]
    fn load_of_a_missing_list_is_an_empty_block_titled_for_the_task() {
        let dossier = tempfile::tempdir().expect("temporary folder");
        let (file, bytes) = load(dossier.path(), "t-001").expect("load");
        assert_eq!(
            file.text(),
            "<!-- dossier-format: 1 -->\n# Files of T-001\n\n"
        );
        assert!(file.keys().is_empty());
        assert!(bytes.is_none());
    }

    #[test]
    fn load_reads_the_list_and_gives_the_bytes_as_read() {
        let dossier = tempfile::tempdir().expect("temporary folder");
        let text = "<!-- dossier-format: 1 -->\n# Files of T-001\n\n## data/model.sim\n\n- role: model\n- type: file\n";
        write(dossier.path(), "files/T-001.md", text.as_bytes());
        let (file, bytes) = load(dossier.path(), "T-001").expect("load");
        assert_eq!(file.keys(), ["data/model.sim"]);
        assert_eq!(
            file.field("data/model.sim", "role").as_deref(),
            Some("model")
        );
        assert_eq!(bytes.as_deref(), Some(text.as_bytes()));
    }

    #[test]
    fn tasks_lists_the_lists_sorted_and_leaves_other_files_out() {
        let dossier = tempfile::tempdir().expect("temporary folder");
        write(dossier.path(), "files/T-002.md", b"x");
        write(dossier.path(), "files/T-001.md", b"x");
        write(dossier.path(), "files/notes.txt", b"x");
        assert_eq!(tasks(dossier.path()), ["T-001", "T-002"]);
        let empty = tempfile::tempdir().expect("temporary folder");
        assert!(tasks(empty.path()).is_empty());
    }

    #[test]
    fn valid_role_takes_the_roles_and_the_settings_roles_in_any_case() {
        let cfg = config_with_roles(&["postprocessing"]);
        assert!(valid_role(&cfg, "model"));
        assert!(valid_role(&cfg, "MODEL"));
        assert!(valid_role(&cfg, "PostProcessing"));
        assert!(!valid_role(&cfg, "drawing"));
        assert!(!valid_role(&config_with_roles(&[]), "postprocessing"));
    }

    #[test]
    fn an_unchanged_file_has_no_marks() {
        let root = tempfile::tempdir().expect("temporary folder");
        write(root.path(), "data/model.sim", b"abc");
        let file = attached(root.path(), "data/model.sim", &[("role", "model")]);
        assert!(marks(root.path(), &file, "data/model.sim").is_empty());
    }

    #[test]
    fn a_detached_block_is_marked_detached_and_nothing_else() {
        let root = tempfile::tempdir().expect("temporary folder");
        write(root.path(), "data/model.sim", b"abc");
        let file = attached(
            root.path(),
            "data/model.sim",
            &[("detached", "2026-11-21T09:00:00+01:00 by Anna")],
        );
        assert_eq!(marks(root.path(), &file, "data/model.sim"), ["detached"]);
    }

    #[test]
    fn a_block_whose_file_is_gone_is_marked_missing() {
        let root = tempfile::tempdir().expect("temporary folder");
        write(root.path(), "data/model.sim", b"abc");
        let file = attached(root.path(), "data/model.sim", &[]);
        std::fs::remove_file(root.path().join("data/model.sim")).expect("remove test file");
        assert_eq!(marks(root.path(), &file, "data/model.sim"), ["missing"]);
    }

    #[test]
    fn a_file_with_another_size_is_marked_changed_since_attached() {
        let root = tempfile::tempdir().expect("temporary folder");
        write(root.path(), "data/model.sim", b"abc");
        let file = attached(root.path(), "data/model.sim", &[]);
        write(root.path(), "data/model.sim", b"abcd");
        assert_eq!(
            marks(root.path(), &file, "data/model.sim"),
            ["changed since attached"]
        );
    }

    #[test]
    fn a_file_with_the_same_size_and_another_checksum_is_marked_changed() {
        let root = tempfile::tempdir().expect("temporary folder");
        write(root.path(), "data/model.sim", b"abc");
        let file = attached(root.path(), "data/model.sim", &[]);
        write(root.path(), "data/model.sim", b"xyz");
        assert_eq!(
            marks(root.path(), &file, "data/model.sim"),
            ["changed since attached"]
        );
    }

    #[test]
    fn a_folder_with_another_number_of_files_is_marked_changed() {
        let root = tempfile::tempdir().expect("temporary folder");
        write(root.path(), "results/run1/a.csv", b"1");
        write(root.path(), "results/run1/b.csv", b"2");
        let file = attached(root.path(), "results/run1", &[]);
        assert!(marks(root.path(), &file, "results/run1").is_empty());
        write(root.path(), "results/run1/c.csv", b"3");
        assert_eq!(
            marks(root.path(), &file, "results/run1"),
            ["changed since attached"]
        );
    }

    #[test]
    fn a_made_from_file_that_changed_makes_the_result_outdated() {
        let root = tempfile::tempdir().expect("temporary folder");
        write(root.path(), "data/geometry.prt", b"geo");
        write(root.path(), "results/run1.csv", b"res");
        let geometry = filesum::checksum(&root.path().join("data/geometry.prt"))
            .expect("checksum of the geometry");
        let made_from = format!("data/geometry.prt@{geometry}");
        let file = attached(
            root.path(),
            "results/run1.csv",
            &[("made-from", made_from.as_str())],
        );
        assert!(marks(root.path(), &file, "results/run1.csv").is_empty());

        write(root.path(), "data/geometry.prt", b"geo, revised");
        assert_eq!(marks(root.path(), &file, "results/run1.csv"), ["outdated"]);
    }

    #[test]
    fn one_changed_made_from_file_of_several_makes_the_result_outdated() {
        let root = tempfile::tempdir().expect("temporary folder");
        write(root.path(), "data/geometry.prt", b"geo");
        write(root.path(), "data/mesh.msh", b"mesh");
        write(root.path(), "results/run1.csv", b"res");
        let geometry = filesum::checksum(&root.path().join("data/geometry.prt"))
            .expect("checksum of the geometry");
        let mesh =
            filesum::checksum(&root.path().join("data/mesh.msh")).expect("checksum of the mesh");
        let made_from = format!("data/geometry.prt@{geometry}, data/mesh.msh@{mesh}");
        let file = attached(
            root.path(),
            "results/run1.csv",
            &[("made-from", made_from.as_str())],
        );
        assert!(marks(root.path(), &file, "results/run1.csv").is_empty());

        write(root.path(), "data/mesh.msh", b"mesh, refined");
        assert_eq!(marks(root.path(), &file, "results/run1.csv"), ["outdated"]);
    }

    #[test]
    fn a_made_from_file_that_is_gone_makes_the_result_outdated() {
        let root = tempfile::tempdir().expect("temporary folder");
        write(root.path(), "data/geometry.prt", b"geo");
        write(root.path(), "results/run1.csv", b"res");
        let geometry = filesum::checksum(&root.path().join("data/geometry.prt"))
            .expect("checksum of the geometry");
        let made_from = format!("data/geometry.prt@{geometry}");
        let file = attached(
            root.path(),
            "results/run1.csv",
            &[("made-from", made_from.as_str())],
        );
        std::fs::remove_file(root.path().join("data/geometry.prt")).expect("remove test file");
        assert_eq!(marks(root.path(), &file, "results/run1.csv"), ["outdated"]);
    }

    #[test]
    fn a_result_that_rests_on_a_wrong_assumption_is_marked_so() {
        let root = tempfile::tempdir().expect("temporary folder");
        write(root.path(), "results/run1.csv", b"res");
        let file = attached(root.path(), "results/run1.csv", &[("rests-on", "ASM-003")]);
        assert_eq!(
            marks(root.path(), &file, "results/run1.csv"),
            ["rests on a wrong assumption"]
        );
    }
}
