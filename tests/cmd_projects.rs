//! Runs the built program for `projects`: the list of the register, the answer of each project's
//! server with `--check`, and the removal of one project from it. The register lies in each test's
//! temporary folder (DOSSIER_REGISTER), and the projects are added with `connect --code`, which
//! runs no ssh. The ssh of `--check` is a stand-in that answers for anna@good-server only.

mod common;

use common::Project;
use predicates::str::contains;

/// The fixed clock of the runs (the same as the helpers use).
const NOW: &str = "2026-11-20T10:00:00+01:00";

/// Records `code` on `target` in the register only, with no pointer file.
fn connect_register_only(project: &Project, target: &str, code: &str) {
    project
        .run("", &["connect", target, "--code", code, "--no-folder"])
        .success();
}

#[test]
fn an_empty_register_says_how_to_add_a_project() {
    let project = Project::new();
    project.run("", &["projects"]).success().stdout(contains(
        "no projects; add one with: dossier connect <user@server>:<folder>",
    ));
}

#[test]
fn projects_lists_both_codes_in_the_order_of_the_register() {
    let project = Project::new();
    connect_register_only(&project, "anna@sim01:/projects/P-018", "P-018");
    connect_register_only(&project, "anna@sim01:/projects/P-017", "P-017");

    let text = project.stdout("", &["projects"]);
    assert_eq!(
        text.lines().collect::<Vec<_>>(),
        vec![
            "P-017  anna@sim01:/projects/P-017  -",
            "P-018  anna@sim01:/projects/P-018  -",
        ]
    );
}

#[test]
fn a_connected_folder_with_spaces_and_non_ascii_letters_is_listed_with_its_local_folder() {
    let project = Project::new();
    project.write("Projekt Übersicht/.keep", "");
    let folder = project.root().join("Projekt Übersicht");
    project
        .command("", NOW)
        .current_dir(&folder)
        .args([
            "connect",
            "anna@sim01:/projekte/Größe P-019",
            "--code",
            "P-019",
        ])
        .assert()
        .success();

    let text = project.stdout("", &["projects"]);
    let line = text.lines().next().unwrap_or_default();
    assert!(
        line.starts_with("P-019  anna@sim01:/projekte/Größe P-019  "),
        "{line}"
    );
    assert!(line.contains("Projekt Übersicht"), "{line}");
    assert!(!line.ends_with("  -"), "{line}");
}

#[test]
fn json_output_is_the_list_of_code_server_folder_and_local() {
    let project = Project::new();
    connect_register_only(&project, "anna@sim01:/projects/P-017", "P-017");

    let text = project.stdout("", &["projects", "--json"]);
    let answer: serde_json::Value = serde_json::from_str(text.trim()).unwrap();
    assert_eq!(answer["schema"], "dossier.projects.v1");
    assert_eq!(answer["ok"], true);
    assert_eq!(
        answer["data"],
        serde_json::json!([{
            "code": "P-017",
            "server": "anna@sim01",
            "folder": "/projects/P-017",
            "local": null,
        }])
    );
}

#[test]
fn remove_forgets_one_project_and_the_list_keeps_the_other() {
    let project = Project::new();
    connect_register_only(&project, "anna@sim01:/projects/P-017", "P-017");
    connect_register_only(&project, "anna@sim01:/projects/P-018", "P-018");

    project
        .run("", &["projects", "remove", "P-017"])
        .success()
        .stdout(contains("removed P-017"));

    let text = project.stdout("", &["projects"]);
    assert_eq!(
        text.lines().collect::<Vec<_>>(),
        vec!["P-018  anna@sim01:/projects/P-018  -"]
    );
}

#[test]
fn removing_a_project_twice_fails_with_exit_1() {
    let project = Project::new();
    connect_register_only(&project, "anna@sim01:/projects/P-017", "P-017");

    project.run("", &["projects", "remove", "P-017"]).success();
    project
        .run("", &["projects", "remove", "P-017"])
        .code(1)
        .stderr(contains("error: no project P-017"));
}

#[test]
fn removing_an_unknown_project_as_json_names_not_found() {
    let project = Project::new();
    project
        .run("", &["projects", "remove", "P-099", "--json"])
        .code(1)
        .stdout(contains("\"ok\":false"))
        .stdout(contains("\"code\":\"NOT_FOUND\""));
}

#[test]
fn remove_without_a_code_is_a_usage_error() {
    let project = Project::new();
    project
        .run("", &["projects", "remove"])
        .code(2)
        .stderr(contains("<CODE>"));
}

/// The stand-in ssh on Windows: it appends its arguments to ssh-calls.txt in the project folder,
/// and exits 0 for anna@good-server (the 5th argument) and 255 for any other server.
const SSH_WINDOWS: &str = concat!(
    "@echo off\r\n",
    "echo %* >> ssh-calls.txt\r\n",
    "if \"%~5\"==\"anna@good-server\" exit /b 0\r\n",
    "exit /b 255\r\n",
);

/// The same stand-in as a shell script, for the other systems.
const SSH_UNIX: &str = concat!(
    "#!/bin/sh\n",
    "echo \"$*\" >> ssh-calls.txt\n",
    "[ \"$5\" = anna@good-server ] && exit 0\n",
    "exit 255\n",
);

/// A project folder with the stand-in ssh written, and two projects connected with --code and
/// --no-folder: P-017 on anna@good-server, which answers, and P-018 on anna@dead-server, which
/// does not.
fn two_projects_with_a_stand_in_ssh() -> Project {
    let project = Project::new();
    project.write_stand_in("ssh", SSH_WINDOWS, SSH_UNIX);
    connect_register_only(&project, "anna@good-server:/projects/P-017", "P-017");
    connect_register_only(&project, "anna@dead-server:/projects/P-018", "P-018");
    project
}

/// The output lines of `projects` with `args`, run with DOSSIER_SSH pointing at a file that does
/// not exist. The run must succeed.
fn lines_with_no_ssh(project: &Project, args: &[&str]) -> Vec<String> {
    let missing = project.root().join("no-such-ssh");
    let stdout = project
        .command("", NOW)
        .env("DOSSIER_SSH", &missing)
        .args(args)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    String::from_utf8(stdout)
        .expect("the standard output is UTF-8")
        .lines()
        .map(str::to_string)
        .collect()
}

#[test]
fn check_shows_which_server_answers_and_which_does_not() {
    let project = two_projects_with_a_stand_in_ssh();
    let text = project.stdout("", &["projects", "--check"]);
    assert_eq!(
        text.lines().collect::<Vec<_>>(),
        vec![
            "P-017  anna@good-server:/projects/P-017  -  answers",
            "P-018  anna@dead-server:/projects/P-018  -  no answer",
        ]
    );
}

#[test]
fn check_runs_ssh_once_per_project_in_order_with_the_connect_timeout_and_true() {
    let project = two_projects_with_a_stand_in_ssh();
    project.stdout("", &["projects", "--check"]);
    // Rust quotes the arguments it passes to a .cmd file, and echo leaves a trailing space; neither
    // is part of the arguments, so both are dropped before the comparison.
    let calls: Vec<String> = project
        .read("ssh-calls.txt")
        .lines()
        .map(|line| line.replace('"', "").trim().to_string())
        .collect();
    assert_eq!(
        calls,
        vec![
            "-o BatchMode=yes -o ConnectTimeout=10 anna@good-server true",
            "-o BatchMode=yes -o ConnectTimeout=10 anna@dead-server true",
        ]
    );
}

#[test]
fn json_with_check_adds_answers_to_each_project() {
    let project = two_projects_with_a_stand_in_ssh();
    let text = project.stdout("", &["projects", "--check", "--json"]);
    let answer: serde_json::Value = serde_json::from_str(text.trim()).unwrap();
    assert_eq!(answer["ok"], true);
    assert_eq!(
        answer["data"],
        serde_json::json!([
            {
                "code": "P-017",
                "server": "anna@good-server",
                "folder": "/projects/P-017",
                "local": null,
                "answers": true,
            },
            {
                "code": "P-018",
                "server": "anna@dead-server",
                "folder": "/projects/P-018",
                "local": null,
                "answers": false,
            },
        ])
    );
}

#[test]
fn projects_without_check_adds_no_answer_and_succeeds_with_no_ssh_program() {
    let project = Project::new();
    connect_register_only(&project, "anna@good-server:/projects/P-017", "P-017");
    assert_eq!(
        lines_with_no_ssh(&project, &["projects"]),
        vec!["P-017  anna@good-server:/projects/P-017  -"]
    );
}

#[test]
fn projects_without_check_never_runs_the_ssh_program() {
    let project = two_projects_with_a_stand_in_ssh();
    project.stdout("", &["projects"]);
    assert!(!project.root().join("ssh-calls.txt").exists());
}

#[test]
fn check_with_no_ssh_program_says_no_answer_for_each_project() {
    let project = Project::new();
    connect_register_only(&project, "anna@good-server:/projects/P-017", "P-017");
    assert_eq!(
        lines_with_no_ssh(&project, &["projects", "--check"]),
        vec!["P-017  anna@good-server:/projects/P-017  -  no answer"]
    );
}

#[test]
fn check_with_a_subcommand_is_a_usage_error_and_removes_nothing() {
    let project = Project::new();
    connect_register_only(&project, "anna@good-server:/projects/P-017", "P-017");
    project
        .run("", &["projects", "--check", "remove", "P-017"])
        .code(2)
        .stderr(contains("--check only lists the projects"));
    assert_eq!(
        project
            .stdout("", &["projects"])
            .lines()
            .collect::<Vec<_>>(),
        vec!["P-017  anna@good-server:/projects/P-017  -"]
    );
}
