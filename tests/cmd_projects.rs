//! Runs the built program for `projects`: the list of the register and the removal of one project
//! from it. The register lies in each test's temporary folder (DOSSIER_REGISTER), and the projects
//! are added with `connect --code`, which runs no ssh.

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
