//! The register: the projects this user works on, and where each one is kept (format.md §12).

use crate::atomic::{self, Seen};
use crate::error::{Code, Error, Result};
use crate::hash::sha256_hex;
use serde::Deserialize;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

/// One project of the register.
pub struct Project {
    pub code: String,
    pub server: String,
    pub folder: String,
    pub local: Option<String>,
}

/// The environment variable that names the register file itself.
const OVERRIDE: &str = "DOSSIER_REGISTER";

/// The folder of the register inside the user's configuration folder.
const FOLDER: &str = "dossier";

/// The register file inside `FOLDER`.
const FILE: &str = "projects.toml";

/// projects.toml as written: format.md §12. Unknown keys are errors.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct File {
    #[serde(default)]
    project: Vec<ProjectFile>,
}

/// One `[[project]]` table.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ProjectFile {
    code: String,
    server: String,
    folder: String,
    #[serde(default)]
    local: Option<String>,
}

/// The register as read from its file: its projects, and the file's length and checksum, which
/// the write checks (None when there is no file).
struct Register {
    projects: Vec<Project>,
    seen: Option<Seen>,
}

/// The register file (format.md §12). `DOSSIER_REGISTER` overrides it.
pub fn file() -> Result<PathBuf> {
    if let Some(path) = non_empty_var(OVERRIDE) {
        return Ok(PathBuf::from(path));
    }
    let folder = if cfg!(windows) {
        required("APPDATA")?
    } else if let Some(config) = non_empty_var("XDG_CONFIG_HOME") {
        PathBuf::from(config)
    } else {
        required("HOME")?.join(".config")
    };
    Ok(folder.join(FOLDER).join(FILE))
}

/// Every project of the register, in the order of the file. A missing file is an empty list.
pub fn load() -> Result<Vec<Project>> {
    Ok(read_register(&file()?)?.projects)
}

/// Replaces the project with the same code (ignoring case), or adds it.
pub fn put(project: &Project) -> Result<()> {
    put_at(&file()?, project)
}

/// Removes the project with `code` (ignoring case). True when there was one.
pub fn remove(code: &str) -> Result<bool> {
    remove_at(&file()?, code)
}

/// `put` for the register file at `path`.
fn put_at(path: &Path, project: &Project) -> Result<()> {
    let register = read_register(path)?;
    let mut projects: Vec<Project> = register
        .projects
        .into_iter()
        .filter(|known| !same(&known.code, &project.code))
        .collect();
    projects.push(Project {
        code: project.code.clone(),
        server: project.server.clone(),
        folder: project.folder.clone(),
        local: project.local.clone(),
    });
    write_register(path, projects, &register.seen)
}

/// `remove` for the register file at `path`. Nothing is written when no project matches.
fn remove_at(path: &Path, code: &str) -> Result<bool> {
    let register = read_register(path)?;
    let known = register.projects.len();
    let kept: Vec<Project> = register
        .projects
        .into_iter()
        .filter(|project| !same(&project.code, code))
        .collect();
    if kept.len() == known {
        return Ok(false);
    }
    write_register(path, kept, &register.seen)?;
    Ok(true)
}

/// The register in the file at `path`. A missing file is an empty register.
fn read_register(path: &Path) -> Result<Register> {
    let bytes = match std::fs::read(path) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(Register {
                projects: Vec::new(),
                seen: None,
            });
        }
        Err(error) => {
            return Err(Error::new(Code::Io, format!("{}: {error}", path.display())));
        }
    };
    Ok(Register {
        projects: parse(&bytes, path)?,
        seen: Some(Seen {
            len: bytes.len() as u64,
            hash: sha256_hex(&bytes),
        }),
    })
}

/// Writes `projects` sorted by code (ignoring case) to the file at `path`. The write fails with
/// Code::Changed when the file is no longer the one `expected` describes.
fn write_register(path: &Path, mut projects: Vec<Project>, expected: &Option<Seen>) -> Result<()> {
    projects.sort_by_key(|project| project.code.to_lowercase());
    atomic::replace(path, render(&projects).as_bytes(), Some(expected))
}

/// The projects in `bytes`, which are the content of `path`.
fn parse(bytes: &[u8], path: &Path) -> Result<Vec<Project>> {
    let text = std::str::from_utf8(bytes).map_err(|error| config_error(path, error))?;
    // format.md §2: a byte-order mark at the start is accepted.
    let text = text.strip_prefix('\u{feff}').unwrap_or(text);
    let file: File = toml::from_str(text).map_err(|error| config_error(path, error))?;
    Ok(file
        .project
        .into_iter()
        .map(|project| Project {
            code: project.code,
            server: project.server,
            folder: project.folder,
            local: project.local,
        })
        .collect())
}

/// The text of the register: one `[[project]]` table per project, a blank line between tables.
fn render(projects: &[Project]) -> String {
    let tables: Vec<String> = projects.iter().map(table).collect();
    tables.join("\n")
}

/// One `[[project]]` table, ending with a line break. `local` is left out when there is none.
fn table(project: &Project) -> String {
    let mut lines = vec![
        "[[project]]".to_string(),
        format!("code = {}", quoted(&project.code)),
        format!("server = {}", quoted(&project.server)),
        format!("folder = {}", quoted(&project.folder)),
    ];
    if let Some(local) = &project.local {
        lines.push(format!("local = {}", quoted(local)));
    }
    lines.push(String::new());
    lines.join("\n")
}

/// A TOML basic string holding `text`: quotes, backslashes and control characters escaped.
fn quoted(text: &str) -> String {
    let mut out = String::from("\"");
    for c in text.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            c if c.is_control() => out.push_str(&format!("\\u{:04X}", u32::from(c))),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// Codes are compared ignoring case.
fn same(a: &str, b: &str) -> bool {
    a.to_lowercase() == b.to_lowercase()
}

/// An environment variable that is set and not empty.
fn non_empty_var(name: &str) -> Option<OsString> {
    std::env::var_os(name).filter(|value| !value.is_empty())
}

/// The folder named by an environment variable that must be set. Missing: Code::Config.
fn required(name: &str) -> Result<PathBuf> {
    non_empty_var(name).map(PathBuf::from).ok_or_else(|| {
        Error::new(
            Code::Config,
            format!("the environment variable {name} is not set, so the register of projects cannot be found; set {OVERRIDE} to its file"),
        )
    })
}

fn config_error(path: &Path, error: impl std::fmt::Display) -> Error {
    Error::new(Code::Config, format!("{}: {error}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn project(code: &str, server: &str, folder: &str, local: Option<&str>) -> Project {
        Project {
            code: code.into(),
            server: server.into(),
            folder: folder.into(),
            local: local.map(Into::into),
        }
    }

    /// The fields of each project, in order, for comparison.
    fn rows(projects: &[Project]) -> Vec<(String, String, String, Option<String>)> {
        projects
            .iter()
            .map(|p| {
                (
                    p.code.clone(),
                    p.server.clone(),
                    p.folder.clone(),
                    p.local.clone(),
                )
            })
            .collect()
    }

    #[test]
    fn projects_are_kept_per_user_and_survive_rewriting() {
        let _guard = crate::ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let dir = tempfile::tempdir().unwrap();
        // A folder with a space and a non-ASCII letter, as a user's folder may have.
        let path = dir.path().join("Konfig ü").join("projects.toml");
        // SAFETY: ENV_LOCK is held, so no other unit test reads or changes the environment now.
        unsafe { std::env::set_var("DOSSIER_REGISTER", &path) };
        assert_eq!(file().unwrap(), path);
        assert!(load().unwrap().is_empty());

        put(&project("P-002", "anna@sim02", "/projects/P-002", None)).unwrap();
        put(&project(
            "P-001",
            "anna@sim01",
            "/projects/P-001",
            Some(r"C:\Users\anna\Projekte ü\P-001"),
        ))
        .unwrap();
        assert_eq!(
            rows(&load().unwrap()),
            rows(&[
                project(
                    "P-001",
                    "anna@sim01",
                    "/projects/P-001",
                    Some(r"C:\Users\anna\Projekte ü\P-001"),
                ),
                project("P-002", "anna@sim02", "/projects/P-002", None),
            ])
        );

        // The same code in other letter case replaces the project.
        put(&project("p-001", "anna@sim03", "/srv/P-001", None)).unwrap();
        assert_eq!(
            rows(&load().unwrap()),
            rows(&[
                project("p-001", "anna@sim03", "/srv/P-001", None),
                project("P-002", "anna@sim02", "/projects/P-002", None),
            ])
        );

        assert!(remove("P-001").unwrap());
        assert!(!remove("P-001").unwrap());
        assert_eq!(
            rows(&load().unwrap()),
            rows(&[project("P-002", "anna@sim02", "/projects/P-002", None)])
        );

        // SAFETY: as above, ENV_LOCK is still held.
        unsafe { std::env::remove_var("DOSSIER_REGISTER") };
    }

    #[test]
    fn a_missing_file_is_an_empty_register_and_remove_does_not_create_it() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("dossier").join("projects.toml");
        assert!(read_register(&path).unwrap().projects.is_empty());
        assert!(!remove_at(&path, "P-001").unwrap());
        assert!(!path.exists());
    }

    #[test]
    fn each_project_is_one_table_and_a_blank_line_separates_the_tables() {
        let text = render(&[
            project(
                "P-001",
                "anna@sim01",
                "/projects/P-001",
                Some(r"C:\Users\anna\Projekte ü\P-001"),
            ),
            project("P-002", "anna@sim01", "/projects/P-002", None),
        ]);
        assert_eq!(
            text,
            r#"[[project]]
code = "P-001"
server = "anna@sim01"
folder = "/projects/P-001"
local = "C:\\Users\\anna\\Projekte ü\\P-001"

[[project]]
code = "P-002"
server = "anna@sim01"
folder = "/projects/P-002"
"#
        );
    }

    #[test]
    fn quotes_backslashes_and_non_ascii_survive_a_rewrite() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("projects.toml");
        let local = r#"D:\Daten "neu"\Projekt Ü ß"#;
        put_at(
            &path,
            &project("P-001", r#"anna@"sim 01""#, "/srv/Projekt ü", Some(local)),
        )
        .unwrap();
        put_at(&path, &project("P-002", "anna@sim02", "/srv/x", None)).unwrap();
        assert_eq!(
            rows(&read_register(&path).unwrap().projects),
            rows(&[
                project("P-001", r#"anna@"sim 01""#, "/srv/Projekt ü", Some(local)),
                project("P-002", "anna@sim02", "/srv/x", None),
            ])
        );
    }

    #[test]
    fn a_broken_file_is_a_config_error_that_names_it() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("projects.toml");
        std::fs::write(&path, "[[project]]\ncode = \"P-001\"\n").unwrap();
        let error = read_register(&path).err().unwrap();
        assert_eq!(error.code, Code::Config);
        assert!(error.message.contains("projects.toml"), "{}", error.message);

        std::fs::write(
            &path,
            "[[project]]\ncode = \"P-001\"\nserver = \"a\"\nfolder = \"b\"\ncolour = \"red\"\n",
        )
        .unwrap();
        let error = read_register(&path).err().unwrap();
        assert_eq!(error.code, Code::Config);
    }

    #[test]
    fn a_file_changed_since_it_was_read_is_not_overwritten() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("projects.toml");
        put_at(&path, &project("P-001", "anna@sim01", "/p", None)).unwrap();
        let register = read_register(&path).unwrap();
        std::fs::write(&path, "# someone else's edit\n").unwrap();
        let error = write_register(&path, Vec::new(), &register.seen).unwrap_err();
        assert_eq!(error.code, Code::Changed);
        assert_eq!(
            std::fs::read_to_string(&path).unwrap(),
            "# someone else's edit\n"
        );
    }
}
