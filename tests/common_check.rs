//! Checks the shared test project of tests/common/mod.rs: a ready project with two people, and a
//! stand-in program written where the program looks for it.

mod common;

use common::Project;

#[test]
fn a_new_project_lists_anna_and_ben() {
    let project = Project::new();
    let listed = project.stdout("", &["people", "list"]);
    assert!(listed.contains("anna"), "people list shows anna: {listed}");
    assert!(listed.contains("ben"), "people list shows ben: {listed}");
}

#[test]
fn a_stand_in_todo_is_written_where_the_program_looks() {
    let project = Project::new();
    let path = project.write_stand_in(
        "todo",
        "@echo off\r\necho stand-in todo\r\n",
        "#!/bin/sh\necho stand-in todo\n",
    );
    assert!(path.exists(), "the stand-in exists at {}", path.display());
    assert_eq!(path, project.stand_in("todo"));
}

#[test]
fn an_unknown_command_exits_with_two() {
    let project = Project::new();
    project.run("anna", &["no-such-command"]).code(2);
}
