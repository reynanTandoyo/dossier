//! Runs the built program for `files`: the file lists of the tasks, grouped by role with their
//! marks, narrowed by --task, --role and words, and as they were on a day with --on (R-FILE-11,
//! R-LIFE-9). Each test runs in its own temporary project folder with the clock fixed by
//! `common::Project` (2026-11-20T10:00:00+01:00). The modification dates of the files are the
//! real ones, so the tests take them from the lists.

mod common;

use common::Project;
use dossier::blockfile::BlockFile;
use dossier::time;
use predicates::str::contains;
use serde_json::json;

/// Invented contents of the files of the tests.
const MODEL: &str = "invented model data\n";
const RESULT: &str = "time;value\n0;1\n1;2\n";

/// The day (YYYY-MM-DD) that the list of `task` records as the modification of `path`.
fn modified_day(project: &Project, task: &str, path: &str) -> String {
    let text = project.read(&format!("DOSSIER/files/{task}.md"));
    let file = BlockFile::parse(&text).expect("the list parses");
    let stamp = file.field(path, "modified").expect("a modification time");
    time::date(&stamp).to_string()
}

/// Writes the invented model and attaches it to T-001 as a model with the note "first mesh".
fn attach_model(project: &Project) {
    project.write("models/wing-v3.step", MODEL);
    project.stdout(
        "anna",
        &[
            "attach",
            "T-001",
            "models/wing-v3.step",
            "--role",
            "model",
            "--note",
            "first mesh",
        ],
    );
}

/// Writes the invented result and attaches it to `task` as a result of the model.
fn attach_result(project: &Project, task: &str, path: &str) {
    project.write(path, RESULT);
    project.stdout("anna", &["attach", task, path, "--role", "result"]);
}

#[test]
fn with_a_task_the_list_is_grouped_by_role_in_the_order_of_the_roles() {
    let project = Project::new();
    project.write("notes/readme.txt", "invented notes\n");
    project.stdout(
        "anna",
        &["attach", "T-001", "notes/readme.txt", "--role", "other"],
    );
    attach_result(&project, "T-001", "results/run1.csv");
    attach_model(&project);
    project.write("data/brief.txt", "invented brief\n");
    project.stdout(
        "anna",
        &["attach", "T-002", "data/brief.txt", "--role", "description"],
    );

    let output = project.stdout("anna", &["files", "--task", "T-001"]);
    let model = modified_day(&project, "T-001", "models/wing-v3.step");
    let result = modified_day(&project, "T-001", "results/run1.csv");
    let other = modified_day(&project, "T-001", "notes/readme.txt");
    // The roles follow filelist::ROLES (result before other), not the alphabet; T-002 is left out.
    assert_eq!(
        output,
        format!(
            "model\n  models/wing-v3.step  first mesh  {model}\n\
             result\n  results/run1.csv  {result}\n\
             other\n  notes/readme.txt  {other}\n"
        )
    );
}

#[test]
fn role_finds_the_files_of_that_role_in_every_list() {
    let project = Project::new();
    attach_model(&project);
    attach_result(&project, "T-001", "results/run1.csv");
    attach_result(&project, "T-002", "results/run2.csv");

    let output = project.stdout("anna", &["files", "--role", "RESULT"]);
    let first = modified_day(&project, "T-001", "results/run1.csv");
    let second = modified_day(&project, "T-002", "results/run2.csv");
    assert_eq!(
        output,
        format!("T-001  results/run1.csv  {first}\nT-002  results/run2.csv  {second}\n")
    );
}

#[test]
fn a_file_changed_since_it_was_attached_is_marked_and_the_others_are_not() {
    let project = Project::new();
    attach_model(&project);
    attach_result(&project, "T-001", "results/run1.csv");
    project.write("results/run1.csv", "time;value\n0;1\n1;2\n2;3\n");

    let output = project.stdout("anna", &["files", "--task", "T-001"]);
    let model = modified_day(&project, "T-001", "models/wing-v3.step");
    let result = modified_day(&project, "T-001", "results/run1.csv");
    assert_eq!(
        output,
        format!(
            "model\n  models/wing-v3.step  first mesh  {model}\n\
             result\n  results/run1.csv  {result}  [changed since attached]\n"
        )
    );
}

#[test]
fn on_shows_the_lists_as_they_were_and_detached_files_only_with_all() {
    let project = Project::new();
    attach_model(&project);
    // Attached at the fixed clock (20 Nov); on 22 Nov the path is moved, which detaches the old one.
    project.write("models/wing-v4.step", "invented model data, version 4\n");
    project
        .run_at(
            "anna",
            "2026-11-22T09:00:00+01:00",
            &[
                "attach",
                "T-001",
                "models/wing-v3.step",
                "models/wing-v4.step",
                "--moved",
            ],
        )
        .success();
    let v3 = modified_day(&project, "T-001", "models/wing-v3.step");
    let v4 = modified_day(&project, "T-001", "models/wing-v4.step");

    assert_eq!(
        project.stdout("anna", &["files", "--on", "2026-11-19"]),
        "no files\n"
    );
    assert_eq!(
        project.stdout("anna", &["files", "--on", "2026-11-20"]),
        format!("T-001  models/wing-v3.step  first mesh  {v3}\n")
    );
    assert_eq!(
        project.stdout("anna", &["files", "--on", "2026-11-21"]),
        format!("T-001  models/wing-v3.step  first mesh  {v3}\n")
    );
    assert_eq!(
        project.stdout("anna", &["files", "--on", "2026-11-22"]),
        format!("T-001  models/wing-v4.step  first mesh  {v4}\n")
    );
    assert_eq!(
        project.stdout("anna", &["files"]),
        format!("T-001  models/wing-v4.step  first mesh  {v4}\n")
    );
    assert_eq!(
        project.stdout("anna", &["files", "--all"]),
        format!(
            "T-001  models/wing-v3.step  first mesh  {v3}  [detached]\n\
             T-001  models/wing-v4.step  first mesh  {v4}\n"
        )
    );
}

#[test]
fn words_must_each_be_in_the_path_or_the_note_ignoring_case() {
    let project = Project::new();
    attach_model(&project);
    attach_result(&project, "T-001", "results/run1.csv");

    let output = project.stdout("anna", &["files", "WING", "Mesh"]);
    let model = modified_day(&project, "T-001", "models/wing-v3.step");
    assert_eq!(
        output,
        format!("T-001  models/wing-v3.step  first mesh  {model}\n")
    );
}

#[test]
fn json_names_each_file_with_its_marks_and_with_on_the_marks_are_null() {
    let project = Project::new();
    attach_model(&project);
    attach_result(&project, "T-001", "results/run1.csv");
    project.write("results/run1.csv", "changed\n");

    let output = project.stdout("anna", &["files", "--task", "T-001", "--json"]);
    let value: serde_json::Value =
        serde_json::from_str(output.trim_end()).expect("one line of JSON");
    assert_eq!(value["schema"], "dossier.files.v1");
    assert_eq!(value["ok"], true);
    let rows = &value["data"];
    assert_eq!(rows[0]["task"], "T-001");
    assert_eq!(rows[0]["path"], "models/wing-v3.step");
    assert_eq!(rows[0]["role"], "model");
    assert_eq!(rows[0]["note"], "first mesh");
    assert_eq!(rows[0]["marks"], json!([]));
    assert_eq!(rows[1]["path"], "results/run1.csv");
    assert_eq!(rows[1]["role"], "result");
    assert!(rows[1]["note"].is_null());
    assert_eq!(rows[1]["marks"], json!(["changed since attached"]));
    let modified = rows[1]["modified"].as_str().expect("a modification time");
    assert!(time::valid(modified), "{modified}");

    let dated = project.stdout(
        "anna",
        &["files", "--task", "T-001", "--on", "2026-11-20", "--json"],
    );
    let value: serde_json::Value =
        serde_json::from_str(dated.trim_end()).expect("one line of JSON");
    assert_eq!(value["data"][0]["path"], "models/wing-v3.step");
    assert!(value["data"][0]["marks"].is_null());
}

#[test]
fn a_task_without_files_and_a_project_without_lists_say_no_files() {
    let project = Project::new();
    assert_eq!(
        project.stdout("anna", &["files", "--task", "T-009"]),
        "T-009: no files\n"
    );
    assert_eq!(project.stdout("anna", &["files"]), "no files\n");
}

#[test]
fn a_path_with_spaces_and_non_ascii_is_shown_with_slashes() {
    let project = Project::new();
    project.write("Modelle ü/Lauf 1.sim", MODEL);
    project.stdout(
        "anna",
        &["attach", "T-001", "Modelle ü/Lauf 1.sim", "--role", "model"],
    );
    let day = modified_day(&project, "T-001", "Modelle ü/Lauf 1.sim");
    assert_eq!(
        project.stdout("anna", &["files", "--task", "T-001"]),
        format!("model\n  Modelle ü/Lauf 1.sim  {day}\n")
    );
}

#[test]
fn an_unknown_role_or_a_bad_day_is_refused() {
    let project = Project::new();
    attach_model(&project);
    project
        .run("anna", &["files", "--role", "drawing"])
        .failure()
        .stderr(contains("drawing is not a role"));
    project
        .run("anna", &["files", "--on", "2026-02-31"])
        .failure()
        .stderr(contains("--on needs a date"));
}

#[test]
fn a_list_that_cannot_be_read_is_a_warning_unless_its_task_is_asked_for() {
    let project = Project::new();
    attach_model(&project);
    project.write("DOSSIER/files/T-003.md", "not a block file\n");

    project
        .run("anna", &["files"])
        .success()
        .stdout(contains("T-001  models/wing-v3.step  first mesh"))
        .stderr(contains(
            "warning: DOSSIER/files/T-003.md: line 1: no heading",
        ));
    project
        .run("anna", &["files", "--task", "T-003"])
        .failure()
        .stderr(contains("no heading"));
}
