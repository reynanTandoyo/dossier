//! Runs the built program for `where`: the project folder, the DOSSIER folder, the state folder and
//! the code, from the project folder and from a subfolder of it, with and without --json, and the
//! refusal in a folder with no dossier. Each test runs in its own temporary folder, with the clock
//! fixed.

mod common;

use common::Project;
use predicates::str::contains;
use std::path::Path;

/// The fixed clock of these tests.
const NOW: &str = "2026-11-20T10:00:00+01:00";

/// The four lines `where` prints for the dossier of the project folder `root`.
fn expected_text(root: &Path, code: &str) -> String {
    let dossier = root.join("DOSSIER");
    format!(
        "project: {}\ndossier: {}\nstate: {}\ncode: {code}\n",
        root.display(),
        dossier.display(),
        dossier.join(".state").display(),
    )
}

/// The program run in `folder` with `args`, with no name and the clock fixed. Its standard
/// output as text, after it succeeded.
fn stdout_in(project: &Project, folder: &Path, args: &[&str]) -> String {
    let stdout = project
        .command("", NOW)
        .current_dir(folder)
        .args(args)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    String::from_utf8(stdout).unwrap()
}

#[test]
fn where_prints_the_project_dossier_state_and_code() {
    let project = Project::new();
    let text = project.stdout("", &["where"]);
    assert_eq!(text, expected_text(project.root(), "P-001"));
    assert!(text.contains("code: P-001"));
    assert!(text.contains("DOSSIER"));
}

#[test]
fn where_as_json_gives_the_same_four_values() {
    let project = Project::new();
    let root = project.root();
    let dossier = root.join("DOSSIER");
    let value: serde_json::Value =
        serde_json::from_str(&project.stdout("", &["where", "--json"])).unwrap();
    assert_eq!(value["schema"], "dossier.where.v1");
    assert_eq!(value["ok"], true);
    assert_eq!(value["data"]["project"], root.display().to_string());
    assert_eq!(value["data"]["dossier"], dossier.display().to_string());
    assert_eq!(
        value["data"]["state"],
        dossier.join(".state").display().to_string()
    );
    assert_eq!(value["data"]["code"], "P-001");
}

#[test]
fn where_from_a_subfolder_gives_the_same_dossier_line() {
    let project = Project::new();
    let deeper = project.root().join("sub").join("deeper");
    std::fs::create_dir_all(&deeper).unwrap();
    let from_project = project.stdout("", &["where"]);
    let from_deeper = stdout_in(&project, &deeper, &["where"]);
    assert_eq!(from_deeper, from_project);
    let dossier_line = format!("dossier: {}\n", project.root().join("DOSSIER").display());
    assert!(from_deeper.contains(dossier_line.as_str()));
}

#[test]
fn where_in_a_folder_with_no_dossier_is_refused_with_code_4() {
    let folder = tempfile::tempdir().unwrap();
    assert_cmd::Command::cargo_bin("dossier")
        .unwrap()
        .current_dir(folder.path())
        .env("DOSSIER_NOW", NOW)
        .env_remove("DOSSIER_AGENT")
        .env_remove("TODO_AGENT")
        .arg("where")
        .assert()
        .code(4)
        .stdout("")
        .stderr(contains("error: no DOSSIER folder found here or above"))
        .stderr(contains("dossier init --code <code>"));
}

#[test]
fn where_as_json_in_a_folder_with_no_dossier_names_no_dossier() {
    let folder = tempfile::tempdir().unwrap();
    assert_cmd::Command::cargo_bin("dossier")
        .unwrap()
        .current_dir(folder.path())
        .env("DOSSIER_NOW", NOW)
        .env_remove("DOSSIER_AGENT")
        .env_remove("TODO_AGENT")
        .args(["where", "--json"])
        .assert()
        .code(4)
        .stdout(contains("\"ok\":false"))
        .stdout(contains("\"code\":\"NO_DOSSIER\""));
}

#[test]
fn where_prints_a_folder_name_with_a_space_and_an_umlaut_as_it_is() {
    let dir = tempfile::Builder::new()
        .prefix("Projekt Äpfel ")
        .tempdir()
        .unwrap();
    let project = Project { dir };
    project
        .command("", NOW)
        .args(["init", "--code", "P-001"])
        .assert()
        .success();
    assert_eq!(
        project.stdout("", &["where"]),
        expected_text(project.root(), "P-001")
    );
}
