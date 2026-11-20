//! Runs the built program for `todo`: the arguments, the output and the exit code pass through to
//! the program `todo`, which runs in the project folder. Each test runs in its own temporary
//! project folder, with DOSSIER_TODO pointing at a stand-in (or at nothing).

mod common;

use common::Project;
use predicates::str::contains;

/// The stand-in for todo: it prints its arguments and exits with 3.
const PRINTS_AND_EXITS_3: (&str, &str) = (
    "@echo off\r\necho %*\r\nexit /b 3\r\n",
    "#!/bin/sh\necho \"$@\"\nexit 3\n",
);

/// The stand-in for todo: it writes its arguments to ran.txt in its current folder and exits
/// with 0.
const WRITES_RAN_TXT: (&str, &str) = (
    "@echo off\r\necho %*> ran.txt\r\nexit /b 0\r\n",
    "#!/bin/sh\necho \"$@\" > ran.txt\nexit 0\n",
);

/// The clock of the runs (todo does not write, so the value only has to be valid).
const NOW: &str = "2026-11-20T10:00:00+01:00";

#[test]
fn todo_gets_the_arguments_and_its_output_and_exit_code_come_back() {
    let project = Project::new();
    let (windows, unix) = PRINTS_AND_EXITS_3;
    project.write_stand_in("todo", windows, unix);
    project
        .run("", &["todo", "list", "--wide"])
        .code(3)
        .stdout(contains("list --wide"));
}

#[test]
fn todo_without_a_program_exits_with_5() {
    let project = Project::new();
    project
        .run("", &["todo", "list"])
        .code(5)
        .stderr(contains("todo is not installed or not on PATH"));
}

#[test]
fn todo_runs_in_the_project_folder_even_from_a_folder_below_it() {
    let project = Project::new();
    let (windows, unix) = WRITES_RAN_TXT;
    project.write_stand_in("todo", windows, unix);
    // A folder with a space and a non-ASCII letter, as project folders may have.
    let below = project.root().join("Übersicht Teil");
    std::fs::create_dir_all(&below).expect("create the folder below the project");
    project
        .command("", NOW)
        .current_dir(&below)
        .args(["todo", "list"])
        .assert()
        .success()
        .stdout("");
    assert!(
        project.root().join("ran.txt").is_file(),
        "todo ran in the project folder"
    );
    assert!(
        !below.join("ran.txt").exists(),
        "todo did not run in the folder below"
    );
    assert!(project.read("ran.txt").contains("list"));
}
