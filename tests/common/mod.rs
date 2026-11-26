//! Shared helpers for the command tests: a temporary project folder with two people, the
//! program run in it with a fixed clock, and stand-in programs for `todo` and `ssh`.
//! Every test file that needs them declares `mod common;`.

#![allow(dead_code)]

use std::path::{Path, PathBuf};

/// The fixed clock of every run that does not say otherwise (25 characters, with a T).
const NOW: &str = "2026-11-20T10:00:00+01:00";

/// The journal IDs of the settings events: the records about dossier.toml, the state of the
/// dossier folder, AGENTS.md, the hooks and the skill. They are not entries (format.md §11), so
/// tests that count entry events leave them out.
pub const SETTINGS_IDS: [&str; 5] = ["DOSSIER", "SETTINGS", "AGENTS", "HOOKS", "SKILL"];

/// Whether `id` is one of `SETTINGS_IDS`.
pub fn is_settings_id(id: &str) -> bool {
    SETTINGS_IDS.contains(&id)
}

/// A temporary project folder. `dossier init --code P-001` has been run in it, and the people
/// anna (LLM anna-claude) and ben (LLM ben-codex) are added. The folder is removed on drop.
pub struct Project {
    pub dir: tempfile::TempDir,
}

impl Project {
    /// A temporary project folder with `dossier init --code P-001` run in it as anna, the people
    /// anna (LLM anna-claude) and ben (LLM ben-codex) added as anna, DOSSIER_NOW fixed. The
    /// journal line of init therefore always names anna, never the machine's user.
    pub fn new() -> Project {
        let project = Project {
            dir: tempfile::TempDir::new().expect("a temporary project folder"),
        };
        project
            .command("anna", NOW)
            .args(["init", "--code", "P-001"])
            .assert()
            .success();
        project
            .command("anna", NOW)
            .args(["people", "add", "anna", "--llm", "anna-claude"])
            .assert()
            .success();
        project
            .command("anna", NOW)
            .args(["people", "add", "ben", "--llm", "ben-codex"])
            .assert()
            .success();
        project
    }

    /// The journal records of the project's dossier that are about entries, oldest first: the
    /// settings events (`SETTINGS_IDS`) are left out.
    pub fn entry_journal(&self) -> Vec<dossier::journal::Record> {
        let state = self.root().join("DOSSIER").join(".state");
        dossier::journal::read(&state)
            .unwrap_or_else(|error| panic!("reading the journal: {error:?}"))
            .into_iter()
            .filter(|record| !is_settings_id(&record.id))
            .collect()
    }

    /// The project folder: the folder that holds DOSSIER.
    pub fn root(&self) -> &Path {
        self.dir.path()
    }

    /// The program run in the project folder as `agent` ("" = no name), with the fixed clock.
    pub fn run(&self, agent: &str, args: &[&str]) -> assert_cmd::assert::Assert {
        self.run_at(agent, NOW, args)
    }

    /// The program run in the project folder as `agent` ("" = no name), with the clock `now`.
    pub fn run_at(&self, agent: &str, now: &str, args: &[&str]) -> assert_cmd::assert::Assert {
        self.command(agent, now).args(args).assert()
    }

    /// Runs the program as `agent`, asserts success, and returns its standard output.
    pub fn stdout(&self, agent: &str, args: &[&str]) -> String {
        let output = self.run(agent, args).success().get_output().stdout.clone();
        String::from_utf8(output).expect("the standard output is UTF-8")
    }

    /// The text of a file, relative to the project folder.
    pub fn read(&self, relative: &str) -> String {
        let path = self.root().join(relative);
        std::fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("reading {}: {error}", path.display()))
    }

    /// Writes a file relative to the project folder, creating its folders as needed.
    pub fn write(&self, relative: &str, text: &str) {
        let path = self.root().join(relative);
        if let Some(folder) = path.parent() {
            std::fs::create_dir_all(folder)
                .unwrap_or_else(|error| panic!("creating {}: {error}", folder.display()));
        }
        std::fs::write(&path, text)
            .unwrap_or_else(|error| panic!("writing {}: {error}", path.display()));
    }

    /// The entry file with `id`: the first file in DOSSIER/<folder>/ whose name starts with the
    /// ID followed by `-`, folders and files in name order. Panics when there is none.
    pub fn entry_file(&self, id: &str) -> PathBuf {
        let dossier = self.root().join("DOSSIER");
        let prefix = format!("{id}-");
        for folder in sorted_children(&dossier, true) {
            for file in sorted_children(&folder, false) {
                let name = file
                    .file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or("");
                if name.starts_with(&prefix) {
                    return file;
                }
            }
        }
        panic!("no entry file for {id} in {}", dossier.display());
    }

    /// The prepared program, not yet run, so that a test can add variables such as
    /// DOSSIER_ANSWERS or EDITOR. Run in the project folder, with DOSSIER_NOW set to `now`,
    /// DOSSIER_REGISTER and the stand-ins of DOSSIER_TODO, DOSSIER_SSH and DOSSIER_SCP,
    /// DOSSIER_AGENT set to `agent`, and DOSSIER_AGENT, USERNAME and USER removed when it is
    /// empty; TODO_AGENT and DOSSIER_ROLE are always removed, so that a test run is never a
    /// worker's unless the test sets DOSSIER_ROLE itself. `run_at` is
    /// `command(agent, now).args(args).assert()`.
    pub fn command(&self, agent: &str, now: &str) -> assert_cmd::Command {
        let mut command =
            assert_cmd::Command::cargo_bin("dossier").expect("the dossier program is built");
        command
            .current_dir(self.root())
            .env("DOSSIER_NOW", now)
            .env(
                "DOSSIER_REGISTER",
                self.root().join(".test").join("projects.toml"),
            )
            .env("DOSSIER_TODO", self.stand_in("todo"))
            .env("DOSSIER_SSH", self.stand_in("ssh"))
            .env("DOSSIER_SCP", self.stand_in("scp"))
            .env_remove("TODO_AGENT")
            .env_remove("DOSSIER_ROLE")
            // A test run inside an ssh session must not look like remote mode (R-REMOTE-12).
            .env_remove("SSH_CONNECTION");
        if agent.is_empty() {
            // The operating-system user is a name too, so a run without a name has none.
            command
                .env_remove("DOSSIER_AGENT")
                .env_remove("USERNAME")
                .env_remove("USER");
        } else {
            command.env("DOSSIER_AGENT", agent);
        }
        command
    }

    /// Where DOSSIER_TODO, DOSSIER_SSH and DOSSIER_SCP point for `program`:
    /// <project>/.test/<program>, with .cmd on Windows. Nothing is there until a test writes it (see `write_stand_in`).
    pub fn stand_in(&self, program: &str) -> PathBuf {
        let folder = self.root().join(".test");
        if cfg!(windows) {
            folder.join(format!("{program}.cmd"))
        } else {
            folder.join(program)
        }
    }

    /// Writes a stand-in program where `stand_in` points: the Windows text on Windows, the Unix
    /// text elsewhere (made executable), and returns its path.
    pub fn write_stand_in(&self, program: &str, windows: &str, unix: &str) -> PathBuf {
        let path = self.stand_in(program);
        let folder = self.root().join(".test");
        std::fs::create_dir_all(&folder)
            .unwrap_or_else(|error| panic!("creating {}: {error}", folder.display()));
        let text = if cfg!(windows) { windows } else { unix };
        std::fs::write(&path, text)
            .unwrap_or_else(|error| panic!("writing {}: {error}", path.display()));
        make_executable(&path);
        path
    }
}

/// The children of `folder` in name order: its subfolders when `folders` is true, else its files.
/// Names that start with a dot are skipped, as a shell glob `*` skips them.
fn sorted_children(folder: &Path, folders: bool) -> Vec<PathBuf> {
    let entries = std::fs::read_dir(folder)
        .unwrap_or_else(|error| panic!("reading {}: {error}", folder.display()));
    let mut children: Vec<PathBuf> = entries
        .map(|entry| {
            entry
                .unwrap_or_else(|error| panic!("reading {}: {error}", folder.display()))
                .path()
        })
        .filter(|path| {
            let hidden = path
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with('.'));
            !hidden
                && if folders {
                    path.is_dir()
                } else {
                    path.is_file()
                }
        })
        .collect();
    children.sort();
    children
}

/// Makes a Unix stand-in program executable (mode 0o755).
#[cfg(unix)]
fn make_executable(path: &Path) {
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755))
        .unwrap_or_else(|error| panic!("making {} executable: {error}", path.display()));
}

/// Windows programs need no mode; a .cmd file runs as it is.
#[cfg(not(unix))]
fn make_executable(_path: &Path) {}
