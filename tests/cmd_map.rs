//! Runs the built program for `map`: `map set` describes a folder or package, `map` prints the
//! map sorted by path with the detached folders left out, and `map remove` takes a folder out of
//! the map and keeps its history. Each test runs in its own temporary project folder with the
//! clock fixed by `common::Project` (2026-11-20T10:00:00+01:00). Folder names are invented.

mod common;

use common::Project;
use predicates::str::contains;

/// The fixed clock of `common::Project`, as it appears in history lines.
const WHEN: &str = "2026-11-20T10:00:00+01:00";

/// A later clock, for the detach time.
const LATER: &str = "2026-11-21T09:00:00+01:00";

/// The purpose and start of the first folder, and the purpose of the second, which has a space
/// and a non-ASCII letter in its name.
const SOLVER: &str = "time integration and the linear solver";
const SOLVER_START: &str = "src/solver/mod.rs";
const RUNS: &str = "test runs and their checks";

/// The map file of the project as text.
fn map_text(project: &Project) -> String {
    project.read("DOSSIER/map.md")
}

/// Two folders with files in them, so that they exist.
fn make_folders(project: &Project) {
    project.write("src/solver/mod.rs", "// invented\n");
    project.write("tools/Prüf läufe/README.md", "# invented\n");
}

/// Describes the two folders as anna. The second one, with the space and the letter, is set
/// first, so that the sorted print differs from the order of setting.
fn describe_both(project: &Project) {
    make_folders(project);
    assert_eq!(
        project
            .stdout("anna", &["map", "set", "tools/Prüf läufe", RUNS])
            .trim_end(),
        "tools/Prüf läufe: set"
    );
    assert_eq!(
        project
            .stdout(
                "anna",
                &["map", "set", "src/solver", SOLVER, "--start", SOLVER_START],
            )
            .trim_end(),
        "src/solver: set"
    );
}

#[test]
fn without_a_map_the_print_says_how_to_start_one() {
    let project = Project::new();
    project.run("anna", &["map"]).success().stdout(contains(
        "no map yet; add one with: dossier map set <path> \"<what it is for>\"",
    ));
    assert!(!project.root().join("DOSSIER/map.md").exists());
}

#[test]
fn two_folders_are_set_and_printed_sorted_by_path() {
    let project = Project::new();
    describe_both(&project);

    assert_eq!(
        project.stdout("anna", &["map"]),
        "src/solver  time integration and the linear solver (start: src/solver/mod.rs)\n\
         tools/Prüf läufe  test runs and their checks\n"
    );
}

#[test]
fn a_changed_folder_shows_its_new_purpose_and_the_other_block_stays_as_written() {
    let project = Project::new();
    describe_both(&project);
    let runs_block = "## tools/Prüf läufe\n\n- purpose: test runs and their checks\n";
    assert!(map_text(&project).contains(runs_block));

    assert_eq!(
        project
            .stdout(
                "anna",
                &[
                    "map",
                    "set",
                    "src/solver",
                    "time integration only",
                    "--start",
                    ""
                ],
            )
            .trim_end(),
        "src/solver: changed"
    );

    assert_eq!(
        project.stdout("anna", &["map"]),
        "src/solver  time integration only\n\
         tools/Prüf läufe  test runs and their checks\n"
    );
    let text = map_text(&project);
    assert!(text.contains(runs_block), "the other block changed: {text}");
    assert!(text.contains(&format!(
        "- history: {WHEN} anna: changed — field purpose, field start\n"
    )));
}

#[test]
fn a_removed_folder_leaves_the_print_and_its_block_keeps_the_history() {
    let project = Project::new();
    describe_both(&project);

    assert_eq!(
        project
            .stdout("anna", &["map", "remove", "src/solver"])
            .trim_end(),
        "src/solver: detached"
    );

    assert_eq!(
        project.stdout("anna", &["map"]),
        "tools/Prüf läufe  test runs and their checks\n"
    );
    let text = map_text(&project);
    assert!(text.contains(&format!("## src/solver\n\n- purpose: {SOLVER}\n")));
    assert!(text.contains(&format!("- detached: {WHEN} by anna\n")));
    assert!(text.contains(&format!("- history: {WHEN} anna: detached\n")));
}

#[test]
fn a_detached_folder_is_used_again_with_set() {
    let project = Project::new();
    describe_both(&project);
    project.stdout("anna", &["map", "remove", "src/solver"]);

    project
        .run_at(
            "anna",
            LATER,
            &["map", "set", "src/solver", SOLVER, "--start", SOLVER_START],
        )
        .success()
        .stdout(contains("src/solver: set"));

    let text = map_text(&project);
    assert!(
        !text.contains("- detached:"),
        "the mark is still there: {text}"
    );
    assert!(text.contains(&format!("- history: {LATER} anna: set\n")));
    assert_eq!(
        project.stdout("anna", &["map"]),
        "src/solver  time integration and the linear solver (start: src/solver/mod.rs)\n\
         tools/Prüf läufe  test runs and their checks\n"
    );
}

#[test]
fn a_set_that_changes_nothing_writes_nothing() {
    let project = Project::new();
    describe_both(&project);
    let before = map_text(&project);

    project
        .run(
            "anna",
            &["map", "set", "src/solver", SOLVER, "--start", SOLVER_START],
        )
        .success()
        .stdout(contains("src/solver: no change"));
    assert_eq!(map_text(&project), before);
}

#[test]
fn a_folder_is_described_from_inside_a_subfolder_by_a_path_relative_to_it() {
    let project = Project::new();
    make_folders(&project);
    project
        .command("anna", WHEN)
        .current_dir(project.root().join("src"))
        .args(["map", "set", "solver", SOLVER])
        .assert()
        .success()
        .stdout(contains("src/solver: set"));
    assert!(map_text(&project).contains("## src/solver\n"));
}

#[test]
fn the_map_as_data_lists_the_folders_with_their_purpose_and_start() {
    let project = Project::new();
    describe_both(&project);

    let output = project.stdout("anna", &["--json", "map"]);
    let value: serde_json::Value = serde_json::from_str(output.trim()).expect("one JSON line");
    assert_eq!(value["schema"], "dossier.map.v1");
    assert_eq!(value["ok"], true);
    assert_eq!(
        value["data"],
        serde_json::json!([
            {"path": "src/solver", "purpose": SOLVER, "start": SOLVER_START},
            {"path": "tools/Prüf läufe", "purpose": RUNS, "start": null},
        ])
    );
}

#[test]
fn each_change_is_in_the_journal_with_who_and_when() {
    let project = Project::new();
    describe_both(&project);
    project
        .run_at("anna", LATER, &["map", "remove", "src/solver"])
        .success();

    let records: Vec<_> = project
        .entry_journal()
        .into_iter()
        .filter(|record| record.id == "MAP")
        .collect();
    let events: Vec<(&str, &str, &str, &str)> = records
        .iter()
        .map(|record| {
            (
                record.time.as_str(),
                record.person.as_str(),
                record.event.as_str(),
                record.detail.as_str(),
            )
        })
        .collect();
    assert_eq!(
        events,
        [
            (WHEN, "anna", "set", "tools/Prüf läufe"),
            (WHEN, "anna", "set", "src/solver"),
            (LATER, "anna", "detached", "src/solver"),
        ]
    );
    assert!(records.iter().all(|record| record.command == "map"));
}

#[test]
fn an_llm_writes_its_name_in_the_history_line() {
    let project = Project::new();
    make_folders(&project);
    project
        .run("anna-claude", &["map", "set", "src/solver", SOLVER])
        .success();
    assert!(map_text(&project).contains(&format!("- history: {WHEN} anna via anna-claude: set\n")));
}

#[test]
fn a_folder_that_does_not_exist_is_refused_and_nothing_is_written() {
    let project = Project::new();
    project
        .run("anna", &["map", "set", "src/missing", SOLVER])
        .failure()
        .code(1)
        .stderr(contains("no folder or file at src/missing"));
    assert!(!project.root().join("DOSSIER/map.md").exists());
}

#[test]
fn a_path_outside_the_project_folder_is_refused() {
    let project = Project::new();
    project
        .run("anna", &["map", "set", "../elsewhere", SOLVER])
        .failure()
        .code(1)
        .stderr(contains("outside the project folder"));
}

#[test]
fn the_project_folder_itself_and_an_empty_purpose_are_refused() {
    let project = Project::new();
    make_folders(&project);
    project
        .run("anna", &["map", "set", ".", SOLVER])
        .failure()
        .code(1)
        .stderr(contains("not the project folder itself"));
    project
        .run("anna", &["map", "set", "src/solver", "  "])
        .failure()
        .code(1)
        .stderr(contains("a folder needs a purpose"));
    assert!(!project.root().join("DOSSIER/map.md").exists());
}

#[test]
fn removing_a_folder_that_is_not_in_the_map_is_refused() {
    let project = Project::new();
    describe_both(&project);
    project
        .run("anna", &["map", "remove", "src/unknown"])
        .failure()
        .code(1)
        .stderr(contains("the map has no src/unknown"));
}

#[test]
fn a_write_without_a_name_is_refused() {
    let project = Project::new();
    make_folders(&project);
    project
        .run("", &["map", "set", "src/solver", SOLVER])
        .failure()
        .code(2)
        .stderr(contains("this command needs a name"));
}
