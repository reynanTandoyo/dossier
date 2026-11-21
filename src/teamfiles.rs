//! The files of a team dossier that hold lists (format.md §20).

use crate::blockfile::BlockFile;
use crate::error::{Code, Error, Result};
use std::path::Path;

/// projects.md of the team dossier and its raw bytes, when it exists.
pub fn projects(team: &Path, code: &str) -> Result<(BlockFile, Option<Vec<u8>>)> {
    load(team, "projects.md", &format!("Projects of {code}"))
}

/// usage-index.md of the team dossier and its raw bytes, when it exists.
pub fn usage(team: &Path, code: &str) -> Result<(BlockFile, Option<Vec<u8>>)> {
    load(team, "usage-index.md", &format!("Usage of {code}"))
}

/// Reads a block file of the team dossier the way `values::load` reads its file: a missing file is
/// `BlockFile::empty(title)` and None. Every error names the file's path, as `exchange::load` does.
fn load(team: &Path, name: &str, title: &str) -> Result<(BlockFile, Option<Vec<u8>>)> {
    let path = team.join(name);
    let bytes = match std::fs::read(&path) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok((BlockFile::empty(title), None));
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;
    use tempfile::TempDir;

    /// projects.md of an invented team: two projects, the second in a folder whose name has a
    /// space and a non-ASCII letter.
    const PROJECTS: &str = r#"<!-- dossier-format: 1 -->
# Projects of TEAM-1

## P-001

- folder: /projects/P-001
- history: 2026-11-02T10:00:00+01:00 anna: added

## P-002

- folder: /projects/P-002 Prüfstand
- history: 2026-11-10T09:30:00+01:00 ben: added
"#;

    /// usage-index.md of the same invented team: one use of each project, the first with a
    /// how-to and a list of analysis words.
    const USAGE: &str = r#"<!-- dossier-format: 1 -->
# Usage of TEAM-1

## P-001:USE-003

- date: 2026-12-02
- code: P-001
- software: ExampleSolver
- version: 2406
- analysis: steady RANS, speedline
- machine: axial compressor
- howto: HOW-002
- person: anna
- history: 2026-12-02T10:15:00+01:00 anna: added

## P-002:USE-001

- date: 2026-12-05
- code: P-002
- software: ExampleSolver
- version: 2312
- person: ben
- history: 2026-12-05T08:40:00+01:00 ben: added
"#;

    /// A team dossier folder in a temporary folder whose name has a space and a non-ASCII letter.
    fn team() -> (TempDir, PathBuf) {
        let temp = tempfile::tempdir().expect("temporary folder");
        let team = temp.path().join("Prüf stand").join("DOSSIER");
        fs::create_dir_all(&team).expect("team dossier folder");
        (temp, team)
    }

    #[test]
    fn a_missing_projects_file_is_empty_with_its_title_and_nothing_is_written() {
        let (_temp, team) = team();
        let (file, bytes) = projects(&team, "TEAM-1").expect("projects");
        assert_eq!(
            file.text(),
            "<!-- dossier-format: 1 -->\n# Projects of TEAM-1\n\n"
        );
        assert_eq!(file.text(), BlockFile::empty("Projects of TEAM-1").text());
        assert!(file.keys().is_empty());
        assert!(bytes.is_none());
        assert!(!team.join("projects.md").exists());
    }

    #[test]
    fn a_missing_usage_index_is_empty_with_its_title_and_nothing_is_written() {
        let (_temp, team) = team();
        let (file, bytes) = usage(&team, "TEAM-1").expect("usage");
        assert_eq!(
            file.text(),
            "<!-- dossier-format: 1 -->\n# Usage of TEAM-1\n\n"
        );
        assert!(file.keys().is_empty());
        assert!(bytes.is_none());
        assert!(!team.join("usage-index.md").exists());
    }

    #[test]
    fn the_projects_example_parses_and_its_blocks_are_the_projects() {
        let (_temp, team) = team();
        fs::write(team.join("projects.md"), PROJECTS).expect("projects.md");
        let (file, bytes) = projects(&team, "TEAM-1").expect("projects");
        assert_eq!(file.text(), PROJECTS);
        assert_eq!(bytes.as_deref(), Some(PROJECTS.as_bytes()));
        assert_eq!(file.keys(), ["P-001", "P-002"]);
        assert_eq!(
            file.field("P-001", "folder").as_deref(),
            Some("/projects/P-001")
        );
        assert_eq!(
            file.field("P-002", "folder").as_deref(),
            Some("/projects/P-002 Prüfstand")
        );
        assert_eq!(
            file.history("P-002"),
            ["2026-11-10T09:30:00+01:00 ben: added"]
        );
    }

    #[test]
    fn the_usage_example_parses_and_its_blocks_are_keyed_by_project_and_use() {
        let (_temp, team) = team();
        fs::write(team.join("usage-index.md"), USAGE).expect("usage-index.md");
        let (file, bytes) = usage(&team, "TEAM-1").expect("usage");
        assert_eq!(file.text(), USAGE);
        assert_eq!(bytes.as_deref(), Some(USAGE.as_bytes()));
        assert_eq!(file.keys(), ["P-001:USE-003", "P-002:USE-001"]);
        assert_eq!(
            file.field("P-001:USE-003", "software").as_deref(),
            Some("ExampleSolver")
        );
        assert_eq!(
            file.field("P-001:USE-003", "howto").as_deref(),
            Some("HOW-002")
        );
        assert_eq!(file.field("P-002:USE-001", "howto"), None);
        assert_eq!(
            file.field("P-002:USE-001", "person").as_deref(),
            Some("ben")
        );
    }

    #[test]
    fn a_file_that_cannot_be_used_is_an_error_that_names_the_file() {
        let (_temp, team) = team();
        let path = team.join("projects.md");

        fs::write(&path, "<!-- dossier-format: 1 -->\nno heading\n").expect("projects.md");
        let error = projects(&team, "TEAM-1").expect_err("no heading");
        assert_eq!(error.code, Code::Parse);
        assert_eq!(
            error.message,
            format!("{}: line 2: no heading", path.display())
        );

        fs::write(&path, "<!-- dossier-format: 2 -->\n# Projects of TEAM-1\n")
            .expect("projects.md");
        let error = projects(&team, "TEAM-1").expect_err("too new");
        assert_eq!(error.code, Code::TooNew);
        assert!(error.message.starts_with(&path.display().to_string()));

        fs::write(&path, [0xFF, 0xFE, b'#', b' ', b'x']).expect("projects.md");
        let error = projects(&team, "TEAM-1").expect_err("not UTF-8");
        assert_eq!(error.code, Code::Parse);
        assert!(error.message.starts_with(&path.display().to_string()));

        fs::remove_file(&path).expect("remove projects.md");
        fs::create_dir(&path).expect("a folder in place of the file");
        let error = projects(&team, "TEAM-1").expect_err("a folder is not readable");
        assert_eq!(error.code, Code::Io);
        assert!(error.message.starts_with(&path.display().to_string()));

        let usage_path = team.join("usage-index.md");
        fs::write(&usage_path, "<!-- dossier-format: 1 -->\nno heading\n").expect("usage-index.md");
        let error = usage(&team, "TEAM-1").expect_err("no heading");
        assert_eq!(error.code, Code::Parse);
        assert_eq!(
            error.message,
            format!("{}: line 2: no heading", usage_path.display())
        );
    }
}
