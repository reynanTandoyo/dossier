//! Runs the built program for `which`: the tasks whose file list has a path, with its role and
//! marks, what those blocks were made from, the blocks made from the path, and the entries whose
//! `code` field points at it (R-FILE-15, R-CODE-2). Each test runs in its own temporary project
//! folder with the clock fixed by `common::Project` (2026-11-20T10:00:00+01:00).

mod common;

use common::Project;
use predicates::str::contains;
use serde_json::json;
use std::fs;

/// Invented contents of the files of the tests.
const MODEL: &str = "invented model data\n";
const RESULT: &str = "time;value\n0;1\n1;2\n";

/// Writes the invented model and attaches it to `task` as a model, with the note "first mesh".
fn attach_model(project: &Project, task: &str) {
    project.write("models/wing-v3.step", MODEL);
    project.stdout(
        "anna",
        &[
            "attach",
            task,
            "models/wing-v3.step",
            "--role",
            "model",
            "--note",
            "first mesh",
        ],
    );
}

/// Writes the invented result and attaches it to `task` as a result made from the model.
fn attach_result_of_model(project: &Project, task: &str, path: &str) {
    project.write(path, RESULT);
    project.stdout(
        "anna",
        &[
            "attach",
            task,
            path,
            "--role",
            "result",
            "--from",
            "models/wing-v3.step",
        ],
    );
}

#[test]
fn a_path_that_no_list_or_entry_has_is_not_attached_to_any_task() {
    let project = Project::new();
    project.write("notes/loose.txt", "invented notes\n");

    // Reading needs no name, so the command runs without one.
    assert_eq!(
        project.stdout("", &["which", "notes/loose.txt"]),
        "not attached to any task\n"
    );
    let output = project.stdout("", &["which", "notes/loose.txt", "--json"]);
    let value: serde_json::Value =
        serde_json::from_str(output.trim_end()).expect("one line of JSON");
    assert_eq!(value["schema"], "dossier.which.v1");
    assert_eq!(value["ok"], true);
    assert_eq!(
        value["data"],
        json!({
            "path": "notes/loose.txt",
            "tasks": [],
            "made_into": [],
            "entries": [],
        })
    );
}

#[test]
fn a_model_names_the_result_made_from_it_and_a_result_names_its_model() {
    let project = Project::new();
    attach_model(&project, "T-001");
    attach_result_of_model(&project, "T-001", "results/run1.csv");

    assert_eq!(
        project.stdout("anna", &["which", "models/wing-v3.step"]),
        "T-001  model\nmade into: T-001 results/run1.csv\n"
    );
    assert_eq!(
        project.stdout("anna", &["which", "results/run1.csv"]),
        "T-001  result\n  made from: models/wing-v3.step\n"
    );
}

#[test]
fn marks_of_each_block_are_shown_and_a_result_whose_model_changed_is_outdated() {
    let project = Project::new();
    attach_model(&project, "T-001");
    attach_result_of_model(&project, "T-001", "results/run1.csv");
    project.write("models/wing-v3.step", "invented model data, revision 2\n");

    assert_eq!(
        project.stdout("anna", &["which", "models/wing-v3.step"]),
        "T-001  model  [changed since attached]\nmade into: T-001 results/run1.csv\n"
    );
    assert_eq!(
        project.stdout("anna", &["which", "results/run1.csv"]),
        "T-001  result  [outdated]\n  made from: models/wing-v3.step\n"
    );
}

#[test]
fn a_detached_block_keeps_what_it_was_made_from_and_is_marked_detached() {
    let project = Project::new();
    attach_model(&project, "T-001");
    attach_result_of_model(&project, "T-001", "results/run1.csv");
    project.stdout("anna", &["detach", "T-001", "results/run1.csv"]);

    assert_eq!(
        project.stdout("anna", &["which", "results/run1.csv"]),
        "T-001  result  [detached]\n  made from: models/wing-v3.step\n"
    );
}

#[test]
fn a_file_in_two_lists_is_named_with_each_task_and_its_role_and_made_into_across_tasks() {
    let project = Project::new();
    attach_model(&project, "T-001");
    project.stdout(
        "anna",
        &[
            "attach",
            "T-002",
            "models/wing-v3.step",
            "--role",
            "reference",
        ],
    );
    attach_result_of_model(&project, "T-002", "results/run2.csv");

    assert_eq!(
        project.stdout("anna", &["which", "models/wing-v3.step"]),
        "T-001  model\nT-002  reference\nmade into: T-002 results/run2.csv\n"
    );
}

#[test]
fn an_entry_whose_code_points_at_the_path_is_named_with_its_marks() {
    let project = Project::new();
    project.stdout(
        "anna",
        &[
            "found",
            "The tip mesh is coarse",
            "--title",
            "Coarse tip mesh",
        ],
    );
    project.stdout(
        "anna",
        &[
            "set",
            "FND-001",
            "--field",
            "code=solver/io.py::MeshReader.read",
        ],
    );
    project.stdout(
        "anna",
        &[
            "found",
            "The old reader is slow",
            "--title",
            "Slow reader",
            "--later",
        ],
    );
    project.stdout("anna", &["set", "FND-002", "--field", "code=solver/io.py"]);
    project.stdout(
        "anna",
        &[
            "found",
            "A longer name of the file",
            "--title",
            "Other file",
        ],
    );
    project.stdout(
        "anna",
        &["set", "FND-003", "--field", "code=solver/io.pyx::Reader"],
    );

    assert_eq!(
        project.stdout("anna", &["which", "solver/io.py"]),
        "not attached to any task\n\
         entries: FND-001 Coarse tip mesh\n\
         entries: FND-002 Slow reader  [draft]\n"
    );
}

#[test]
fn an_entry_whose_code_points_at_a_folder_is_named_for_the_files_in_it_and_for_the_folder() {
    let project = Project::new();
    project.stdout(
        "anna",
        &[
            "found",
            "The seal leaks at high pressure",
            "--title",
            "Seal leak",
        ],
    );
    project.stdout("anna", &["set", "FND-001", "--field", "code=src/pump"]);

    assert_eq!(
        project.stdout("anna", &["which", "src/pump/seal.rs"]),
        "not attached to any task\nentries: FND-001 Seal leak\n"
    );
    assert_eq!(
        project.stdout("anna", &["which", "src/pump"]),
        "not attached to any task\nentries: FND-001 Seal leak\n"
    );
}

#[test]
fn an_entry_whose_code_names_a_name_that_only_starts_like_the_path_is_not_named() {
    let project = Project::new();
    project.stdout(
        "anna",
        &["found", "The pump is loud", "--title", "Loud pump"],
    );
    project.stdout("anna", &["set", "FND-001", "--field", "code=src/pum"]);

    assert_eq!(
        project.stdout("anna", &["which", "src/pump/seal.rs"]),
        "not attached to any task\n"
    );
}

#[test]
fn an_entry_whose_code_names_a_file_and_symbol_is_named_for_that_file_only() {
    let project = Project::new();
    project.stdout(
        "anna",
        &["found", "The leak model is wrong", "--title", "Leak model"],
    );
    project.stdout(
        "anna",
        &[
            "set",
            "FND-001",
            "--field",
            "code=src/pump/seal.rs::leak_model",
        ],
    );

    assert_eq!(
        project.stdout("anna", &["which", "src/pump/seal.rs"]),
        "not attached to any task\nentries: FND-001 Leak model\n"
    );
    assert_eq!(
        project.stdout("anna", &["which", "src/pump"]),
        "not attached to any task\n"
    );
}

#[test]
fn json_gives_the_same_parts_as_the_text() {
    let project = Project::new();
    attach_model(&project, "T-001");
    attach_result_of_model(&project, "T-001", "results/run1.csv");
    project.stdout(
        "anna",
        &[
            "found",
            "The model mesh is coarse",
            "--title",
            "Coarse mesh",
        ],
    );
    project.stdout(
        "anna",
        &[
            "set",
            "FND-001",
            "--field",
            "code=models/wing-v3.step::Mesh",
        ],
    );

    let output = project.stdout("anna", &["which", "models/wing-v3.step", "--json"]);
    let value: serde_json::Value =
        serde_json::from_str(output.trim_end()).expect("one line of JSON");
    assert_eq!(value["schema"], "dossier.which.v1");
    assert_eq!(value["ok"], true);
    assert_eq!(
        value["data"],
        json!({
            "path": "models/wing-v3.step",
            "tasks": [{
                "task": "T-001",
                "role": "model",
                "marks": [],
                "made_from": [],
            }],
            "made_into": [{"task": "T-001", "path": "results/run1.csv"}],
            "entries": [{"id": "FND-001", "title": "Coarse mesh", "marks": []}],
        })
    );
}

#[test]
fn a_path_with_spaces_and_non_ascii_letters_works() {
    let project = Project::new();
    project.write("Modelle ü/Lauf 1.sim", MODEL);
    project.stdout(
        "anna",
        &["attach", "T-001", "Modelle ü/Lauf 1.sim", "--role", "model"],
    );
    project.write("Ergebnisse/Lauf 1.csv", RESULT);
    project.stdout(
        "anna",
        &[
            "attach",
            "T-001",
            "Ergebnisse/Lauf 1.csv",
            "--role",
            "result",
            "--from",
            "Modelle ü/Lauf 1.sim",
        ],
    );

    assert_eq!(
        project.stdout("anna", &["which", "Modelle ü/Lauf 1.sim"]),
        "T-001  model\nmade into: T-001 Ergebnisse/Lauf 1.csv\n"
    );
    assert_eq!(
        project.stdout("anna", &["which", "Ergebnisse/Lauf 1.csv"]),
        "T-001  result\n  made from: Modelle ü/Lauf 1.sim\n"
    );
}

#[test]
fn a_folder_made_from_is_named_by_its_path_without_a_checksum() {
    let project = Project::new();
    project.write("results/out/a.csv", "1\n");
    project.write("results/out/b.csv", "2\n");
    project.stdout(
        "anna",
        &["attach", "T-001", "results/out", "--role", "result"],
    );
    project.write("reports/summary.csv", "total\n");
    project.stdout(
        "anna",
        &[
            "attach",
            "T-001",
            "reports/summary.csv",
            "--role",
            "report",
            "--from",
            "results/out",
        ],
    );

    assert_eq!(
        project.stdout("anna", &["which", "results/out"]),
        "T-001  result\nmade into: T-001 reports/summary.csv\n"
    );
}

#[test]
fn a_path_outside_the_project_folder_is_refused() {
    let project = Project::new();
    project
        .run("", &["which", "../elsewhere.step"])
        .failure()
        .code(1)
        .stderr(contains("the path is outside the project folder"));
}

/// Writes the invented approved decision DEC-001, whose code pointer is the folder `src/pump`.
fn decide_about_pump(project: &Project) {
    project.stdout(
        "anna",
        &[
            "decide",
            "Keep the seal gap at 0.2 mm.",
            "--reason",
            "Invented tests leaked above it.",
            "--title",
            "Seal gap",
        ],
    );
    project.stdout("anna", &["set", "DEC-001", "--field", "code=src/pump"]);
}

#[test]
fn binding_prints_only_the_approved_entry_in_force_with_its_first_decision_line() {
    let project = Project::new();
    decide_about_pump(&project);
    project.stdout(
        "anna-claude",
        &[
            "learn",
            "The pump seal wears quickly.",
            "--valid-for",
            "An invented rig.",
            "--evidence",
            "An invented test.",
            "--confidence",
            "guess",
            "--title",
            "Draft lesson",
        ],
    );
    project.stdout("anna", &["set", "LES-001", "--field", "code=src/pump"]);

    assert_eq!(
        project.stdout("", &["which", "src/pump/seal.rs", "--binding"]),
        "DEC-001 · decision · Seal gap\n  Keep the seal gap at 0.2 mm.\n"
    );
    assert_eq!(
        project.stdout("", &["which", "src/valve/gate.rs", "--binding"]),
        ""
    );
}

#[test]
fn binding_leaves_out_a_superseded_decision() {
    let project = Project::new();
    decide_about_pump(&project);
    project.stdout(
        "anna",
        &[
            "decide",
            "Use the new seal.",
            "--reason",
            "It replaces the old one.",
            "--title",
            "New seal",
        ],
    );
    project.stdout("anna", &["supersede", "DEC-001", "--by", "DEC-002"]);

    assert_eq!(
        project.stdout("", &["which", "src/pump/seal.rs", "--binding"]),
        ""
    );
}

/// Sets `valid-until` of the entry `id` to `stamp` by hand, as a person would edit its file: the
/// tool writes that field itself, so `set` refuses it. The line is put under `valid-from`.
fn end_by_hand(project: &Project, id: &str, stamp: &str) {
    let path = project.entry_file(id);
    let text = fs::read_to_string(&path).expect("the entry file");
    let from_line = text
        .lines()
        .find(|line| line.starts_with("- valid-from: "))
        .expect("a valid-from line")
        .to_string();
    let edited = text.replacen(
        &format!("{from_line}\n"),
        &format!("{from_line}\n- valid-until: {stamp}\n"),
        1,
    );
    fs::write(&path, edited).expect("the entry is changed by hand");
}

#[test]
fn binding_leaves_out_an_approved_entry_whose_valid_until_is_before_today() {
    let project = Project::new();
    decide_about_pump(&project);
    end_by_hand(&project, "DEC-001", "2026-11-19T10:00:00+01:00");
    project.stdout(
        "anna",
        &[
            "decide",
            "Replace the seal every 5000 hours.",
            "--reason",
            "Invented wear data.",
            "--title",
            "Seal wear",
        ],
    );
    project.stdout("anna", &["set", "DEC-002", "--field", "code=src/pump"]);

    assert_eq!(
        project.stdout("", &["which", "src/pump/seal.rs", "--binding"]),
        "DEC-002 · decision · Seal wear\n  Replace the seal every 5000 hours.\n"
    );
}

#[test]
fn binding_json_gives_the_path_and_each_entry_with_its_kind_and_first_line() {
    let project = Project::new();
    decide_about_pump(&project);
    project.stdout(
        "anna",
        &[
            "found",
            "The seal leaks above 40 bar.",
            "--title",
            "Seal leak",
        ],
    );
    project.stdout("anna", &["set", "FND-001", "--field", "code=src/pump"]);

    let output = project.stdout("", &["which", "src/pump/seal.rs", "--binding", "--json"]);
    let value: serde_json::Value =
        serde_json::from_str(output.trim_end()).expect("one line of JSON");
    assert_eq!(value["schema"], "dossier.which.v1");
    assert_eq!(value["ok"], true);
    assert_eq!(
        value["data"],
        json!({
            "path": "src/pump/seal.rs",
            "entries": [
                {
                    "id": "DEC-001",
                    "kind": "decision",
                    "title": "Seal gap",
                    "text": "Keep the seal gap at 0.2 mm.",
                    "linked": [],
                },
                {
                    "id": "FND-001",
                    "kind": "finding",
                    "title": "Seal leak",
                    "text": "The seal leaks above 40 bar.",
                    "linked": [],
                },
            ],
        })
    );
}

#[test]
fn binding_with_session_leaves_out_what_the_session_was_shown_before() {
    let project = Project::new();
    decide_about_pump(&project);
    let shown = |session: &str| {
        project.stdout(
            "",
            &[
                "which",
                "src/pump/seal.rs",
                "--binding",
                "--session",
                session,
            ],
        )
    };

    let first = "DEC-001 · decision · Seal gap\n  Keep the seal gap at 0.2 mm.\n";
    assert_eq!(shown("s1"), first);
    assert_eq!(shown("s1"), "");
    assert_eq!(shown("s2"), first);
}

/// Makes the approved finding FND-001 "Seal leak" (no code pointer) and links it to DEC-001, the
/// decision of `decide_about_pump`.
fn link_seal_leak_to_pump_decision(project: &Project) {
    project.stdout(
        "anna",
        &[
            "found",
            "The seal leaks above 40 bar.",
            "--title",
            "Seal leak",
        ],
    );
    project.stdout("anna", &["link", "DEC-001", "FND-001"]);
}

#[test]
fn binding_shows_the_approved_entry_linked_under_the_decision() {
    let project = Project::new();
    decide_about_pump(&project);
    link_seal_leak_to_pump_decision(&project);

    assert_eq!(
        project.stdout("", &["which", "src/pump/seal.rs", "--binding"]),
        "DEC-001 · decision · Seal gap\n  Keep the seal gap at 0.2 mm.\n  linked: FND-001 · Seal leak\n"
    );
}

#[test]
fn binding_json_gives_each_entry_its_linked_entries() {
    let project = Project::new();
    decide_about_pump(&project);
    link_seal_leak_to_pump_decision(&project);

    let output = project.stdout("", &["which", "src/pump/seal.rs", "--binding", "--json"]);
    let value: serde_json::Value =
        serde_json::from_str(output.trim_end()).expect("one line of JSON");
    assert_eq!(value["schema"], "dossier.which.v1");
    assert_eq!(value["ok"], true);
    assert_eq!(
        value["data"],
        json!({
            "path": "src/pump/seal.rs",
            "entries": [{
                "id": "DEC-001",
                "kind": "decision",
                "title": "Seal gap",
                "text": "Keep the seal gap at 0.2 mm.",
                "linked": [{"id": "FND-001", "title": "Seal leak", "type": "related"}],
            }],
        })
    );
}

#[test]
fn binding_names_the_type_of_each_link_in_text_and_json() {
    let project = Project::new();
    project.stdout(
        "anna",
        &[
            "decide",
            "Keep the pump gap at 0.2 mm.",
            "--reason",
            "Invented tests leaked above it.",
            "--title",
            "Pump gap",
            "--code",
            "src/pump.rs",
        ],
    );
    project.stdout(
        "anna",
        &[
            "decide",
            "Keep the pump gap at 0.3 mm.",
            "--reason",
            "Invented second test.",
            "--title",
            "Wider pump gap",
        ],
    );
    project.stdout(
        "anna",
        &["set", "DEC-001", "--field", "contradicts=DEC-002"],
    );
    project.write("src/pump.rs", "invented pump code\n");

    assert_eq!(
        project.stdout("", &["which", "src/pump.rs", "--binding"]),
        "DEC-001 · decision · Pump gap\n  Keep the pump gap at 0.2 mm.\n  contradicts: DEC-002 · Wider pump gap\n"
    );
    let output = project.stdout("", &["which", "src/pump.rs", "--binding", "--json"]);
    let value: serde_json::Value =
        serde_json::from_str(output.trim_end()).expect("one line of JSON");
    assert_eq!(
        value["data"]["entries"][0]["linked"],
        json!([{"id": "DEC-002", "title": "Wider pump gap", "type": "contradicts"}])
    );
}

#[test]
fn binding_leaves_out_a_linked_entry_that_is_retired() {
    let project = Project::new();
    decide_about_pump(&project);
    link_seal_leak_to_pump_decision(&project);
    project.stdout(
        "anna",
        &["retire", "FND-001", "--reason", "Invented reason."],
    );

    assert_eq!(
        project.stdout("", &["which", "src/pump/seal.rs", "--binding"]),
        "DEC-001 · decision · Seal gap\n  Keep the seal gap at 0.2 mm.\n"
    );
}

#[test]
fn a_linked_entry_that_is_listed_as_an_entry_of_its_own_is_not_repeated_as_linked() {
    let project = Project::new();
    decide_about_pump(&project);
    link_seal_leak_to_pump_decision(&project);
    project.stdout("anna", &["set", "FND-001", "--field", "code=src/pump"]);

    assert_eq!(
        project.stdout("", &["which", "src/pump/seal.rs", "--binding"]),
        "DEC-001 · decision · Seal gap\n  Keep the seal gap at 0.2 mm.\n\
         FND-001 · finding · Seal leak\n  The seal leaks above 40 bar.\n"
    );
}

#[test]
fn binding_with_session_shows_the_linked_entry_with_its_decision_only_the_first_time() {
    let project = Project::new();
    decide_about_pump(&project);
    link_seal_leak_to_pump_decision(&project);
    let shown = |session: &str| {
        project.stdout(
            "",
            &[
                "which",
                "src/pump/seal.rs",
                "--binding",
                "--session",
                session,
            ],
        )
    };

    assert_eq!(
        shown("s1"),
        "DEC-001 · decision · Seal gap\n  Keep the seal gap at 0.2 mm.\n  linked: FND-001 · Seal leak\n"
    );
    assert_eq!(shown("s1"), "");
}

#[test]
fn session_without_binding_is_a_usage_error() {
    let project = Project::new();
    project
        .run("", &["which", "src/pump/seal.rs", "--session", "s1"])
        .failure()
        .code(2)
        .stderr(contains("--binding"));
}
