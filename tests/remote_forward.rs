//! Runs the built program for commands typed in a connected folder (R-REMOTE-1): each command
//! goes to the server through ssh with the versions of the PC (R-XOS-9, format.md §31) and the
//! folder typed below the connected folder (R-XOS-3). The ssh program is a stand-in that appends
//! its arguments to ssh-calls.txt in the project folder, so no test runs a real ssh. The
//! register lies in each test's temporary folder (DOSSIER_REGISTER).

mod common;

use common::Project;
use std::path::Path;

/// The fixed clock of the runs (the same as the helpers use).
const NOW: &str = "2026-11-20T10:00:00+01:00";

/// The server and folder that the connected project folder points at.
const TARGET: &str = "anna@server:/projects/p1";

/// The program version of this build, as the PC sends it.
const VERSION: &str = env!("CARGO_PKG_VERSION");

/// The file in the project folder where the stand-in writes the arguments of each call.
const RECORD: &str = "ssh-calls.txt";

/// The stand-in ssh for Windows and for the other systems: each writes its arguments as one line
/// to `record` (an absolute path, so that the folder the command runs in does not matter), and
/// exits with 0.
fn stand_in_texts(record: &Path) -> (String, String) {
    let windows = format!(
        "@echo off\r\necho %* >> \"{}\"\r\nexit /b 0\r\n",
        record.display()
    );
    let unix = format!(
        "#!/bin/sh\nprintf '%s\\n' \"$*\" >> '{}'\nexit 0\n",
        record.display()
    );
    (windows, unix)
}

/// A project folder with the stand-in ssh written, and its folder connected to TARGET with the
/// code P-001 (the register gets the project too).
fn connected_project() -> Project {
    let project = Project::new();
    let (windows, unix) = stand_in_texts(&project.root().join(RECORD));
    project.write_stand_in("ssh", &windows, &unix);
    project
        .command("", NOW)
        .args(["connect", "--code", "P-001", TARGET])
        .assert()
        .success();
    project
}

/// `dossier <args>` run in `folder`, which must succeed (the stand-in exits with 0).
fn run_in(project: &Project, folder: &Path, args: &[&str]) {
    project
        .command("", NOW)
        .current_dir(folder)
        .args(args)
        .assert()
        .success();
}

/// The one line that the stand-in recorded: the arguments of the one ssh call that the command
/// made. Panics when the number of calls is not one.
fn the_one_call(project: &Project) -> String {
    let text = std::fs::read_to_string(project.root().join(RECORD))
        .unwrap_or_else(|error| panic!("reading {RECORD}: {error}"));
    let lines: Vec<&str> = text.lines().collect();
    assert_eq!(lines.len(), 1, "one ssh call expected: {text}");
    lines[0].to_string()
}

#[test]
fn a_command_in_the_connected_folder_sends_the_versions_and_no_subfolder() {
    let project = connected_project();
    run_in(&project, project.root(), &["list"]);

    let call = the_one_call(&project);
    assert!(
        call.contains(&format!(
            "cd '/projects/p1' && DOSSIER_CLIENT_VERSION='{VERSION}' DOSSIER_CLIENT_FORMAT='1' 'dossier' 'list'"
        )),
        "{call}"
    );
}

#[test]
fn a_command_in_a_subfolder_runs_there_when_it_exists_and_sends_the_versions() {
    let project = connected_project();
    project.write("sim/run 1/.keep", "");
    run_in(
        &project,
        &project.root().join("sim").join("run 1"),
        &["list"],
    );

    let call = the_one_call(&project);
    assert!(
        call.contains("cd '/projects/p1/sim/run 1' 2>/dev/null || cd '/projects/p1'"),
        "{call}"
    );
    assert!(
        call.contains(&format!(
            "DOSSIER_CLIENT_VERSION='{VERSION}' DOSSIER_CLIENT_FORMAT='1' 'dossier' 'list'"
        )),
        "{call}"
    );
}

#[test]
fn todo_in_the_connected_folder_runs_without_the_client_variables() {
    let project = connected_project();
    run_in(&project, project.root(), &["todo", "list"]);

    let call = the_one_call(&project);
    assert!(
        call.contains("cd '/projects/p1' && 'todo' 'list'"),
        "{call}"
    );
    assert!(!call.contains("DOSSIER_CLIENT"), "{call}");
}

#[test]
fn project_option_from_a_subfolder_runs_in_the_project_folder_with_the_versions() {
    let project = connected_project();
    project.write("sim/run 1/.keep", "");
    run_in(
        &project,
        &project.root().join("sim").join("run 1"),
        &["--project", "P-001", "list"],
    );

    let call = the_one_call(&project);
    assert!(
        call.contains(&format!(
            "cd '/projects/p1' && DOSSIER_CLIENT_VERSION='{VERSION}' DOSSIER_CLIENT_FORMAT='1' 'dossier' 'list'"
        )),
        "{call}"
    );
    assert!(!call.contains("sim"), "{call}");
}
