//! The version check of a command run on the server (R-XOS-9, format.md §31): a PC with another
//! format is refused before the command runs, and the side to update is named; a PC with another
//! program version and the same format gets a warning after the command ran. Each test runs in
//! its own temporary project, with the clock fixed.

mod common;

use common::Project;
use dossier::remote::version::{CLIENT_FORMAT, CLIENT_VERSION};

/// The fixed clock of these tests.
const NOW: &str = "2026-11-20T10:00:00+01:00";

/// The program version of the server program: this build.
const SERVER_VERSION: &str = env!("CARGO_PKG_VERSION");

/// The program run in the project folder with `args`, with the two variables of a PC set as
/// `variables` (the two are removed first, so a test never sees the variables of its runner).
fn run_as_pc(
    project: &Project,
    variables: &[(&str, &str)],
    args: &[&str],
) -> assert_cmd::assert::Assert {
    let mut command = project.command("", NOW);
    command.env_remove(CLIENT_VERSION).env_remove(CLIENT_FORMAT);
    for (name, value) in variables {
        command.env(name, value);
    }
    command.args(args).assert()
}

/// Bytes as text; the program writes UTF-8.
fn text(bytes: &[u8]) -> String {
    String::from_utf8(bytes.to_vec()).expect("the output is UTF-8")
}

#[test]
fn a_pc_with_another_format_is_refused_before_the_command_runs() {
    let project = Project::new();
    let output = run_as_pc(&project, &[(CLIENT_FORMAT, "2")], &["where"])
        .code(4)
        .get_output()
        .clone();
    assert_eq!(text(&output.stdout), "");
    assert_eq!(
        text(&output.stderr),
        "error: the PC uses dossier format 2, the server format 1: update dossier on the server\n"
    );
}

#[test]
fn a_refused_pc_format_with_json_has_the_code_too_new() {
    let project = Project::new();
    let output = run_as_pc(&project, &[(CLIENT_FORMAT, "2")], &["where", "--json"])
        .code(4)
        .get_output()
        .clone();
    assert_eq!(text(&output.stderr), "");
    let value: serde_json::Value =
        serde_json::from_str(&text(&output.stdout)).expect("one JSON line");
    assert_eq!(value["schema"], "dossier.where.v1");
    assert_eq!(value["ok"], false);
    assert_eq!(value["error"]["code"], "TOO_NEW");
    assert_eq!(
        value["error"]["message"],
        "the PC uses dossier format 2, the server format 1: update dossier on the server"
    );
}

#[test]
fn a_pc_with_another_program_version_and_the_same_format_gets_a_warning() {
    let project = Project::new();
    let output = run_as_pc(
        &project,
        &[(CLIENT_FORMAT, "1"), (CLIENT_VERSION, "0.0.1")],
        &["where"],
    )
    .success()
    .get_output()
    .clone();
    assert!(text(&output.stdout).contains("code: P-001"));
    assert_eq!(
        text(&output.stderr),
        format!("warning: dossier on the PC is 0.0.1, on the server {SERVER_VERSION}\n")
    );
}

#[test]
fn without_the_variables_there_is_no_warning() {
    let project = Project::new();
    let output = run_as_pc(&project, &[], &["where"])
        .success()
        .get_output()
        .clone();
    assert!(text(&output.stdout).contains("code: P-001"));
    assert_eq!(text(&output.stderr), "");
}

#[test]
fn an_empty_variable_counts_as_not_set() {
    let project = Project::new();
    run_as_pc(
        &project,
        &[(CLIENT_FORMAT, ""), (CLIENT_VERSION, "0.0.1")],
        &["where"],
    )
    .success()
    .stderr("");
    run_as_pc(
        &project,
        &[(CLIENT_FORMAT, "1"), (CLIENT_VERSION, "")],
        &["where"],
    )
    .success()
    .stderr("");
}
