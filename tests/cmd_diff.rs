//! Runs the built program for `diff`: the lines added and removed since a snapshot (text and
//! JSON), the latest snapshot by default and a given number, "no difference" when the file and the
//! snapshot agree, and exit 1 for an entry without snapshots, a missing number and an unknown
//! entry. Each test runs in its own project folder, with the clock fixed by `common::Project`.

mod common;

use common::Project;
use predicates::str::contains;
use serde_json::{Value, json};

/// Invented texts and times. The fixed clock of `common::Project` is the creation of DEC-001.
const FIRST_CHANGE: &str = "2026-11-21T09:00:00+01:00";
const SECOND_CHANGE: &str = "2026-11-21T11:00:00+01:00";
const DECISION: &str = "The last three points of every speedline are run transient.";
const REASON: &str = "Steady runs converged where the transient run already showed rotating stall.";

/// Anna decides DEC-001; it is approved at once, on the fixed clock.
fn anna_decides(project: &Project) {
    project.stdout("anna", &["decide", DECISION, "--reason", REASON]);
}

/// A project with DEC-001, and `set --field machine=x` on FIRST_CHANGE. Snapshot 1 is the file
/// as it was before that change.
fn one_change() -> Project {
    let project = Project::new();
    anna_decides(&project);
    project
        .run_at(
            "anna",
            FIRST_CHANGE,
            &["set", "DEC-001", "--field", "machine=x"],
        )
        .success();
    project
}

/// The history line (without its `- ` list marker) that `set` writes for a change of the machine
/// field on `when`, by anna.
fn machine_changed_line(when: &str) -> String {
    format!("history: {when} anna: changed — field machine")
}

#[test]
fn after_set_the_added_field_and_the_history_line_are_shown_with_plus() {
    let project = one_change();

    let expected = format!(
        "+ - machine: x\n+ - {}\n",
        machine_changed_line(FIRST_CHANGE)
    );
    // Every form of the ID names the same entry.
    for id in ["DEC-001", "dec-001", "P-001:DEC-001"] {
        assert_eq!(project.stdout("", &["diff", id]), expected, "{id}");
    }
}

#[test]
fn json_gives_the_added_lines_and_the_snapshot_number() {
    let project = one_change();

    let stdout = project.stdout("", &["diff", "DEC-001", "--json"]);
    let envelope: Value = serde_json::from_str(stdout.trim()).expect("one line of JSON");

    assert_eq!(envelope["schema"], "dossier.diff.v1");
    assert_eq!(envelope["ok"], true);
    assert_eq!(envelope["warnings"], json!([]));
    assert_eq!(
        envelope["data"],
        json!({
            "id": "DEC-001",
            "n": 1,
            "removed": [],
            "added": [
                "- machine: x",
                format!("- {}", machine_changed_line(FIRST_CHANGE)),
            ],
        })
    );
}

#[test]
fn a_changed_value_is_removed_then_added_since_the_latest_snapshot() {
    let project = one_change();
    project
        .run_at(
            "anna",
            SECOND_CHANGE,
            &["set", "DEC-001", "--field", "machine=y"],
        )
        .success();

    // The latest snapshot is number 2: the file as it was before the second change.
    assert_eq!(
        project.stdout("", &["diff", "DEC-001"]),
        format!(
            "- - machine: x\n+ - machine: y\n+ - {}\n",
            machine_changed_line(SECOND_CHANGE)
        )
    );
}

#[test]
fn a_given_number_compares_the_file_with_that_snapshot() {
    let project = one_change();
    project
        .run_at(
            "anna",
            SECOND_CHANGE,
            &["set", "DEC-001", "--field", "machine=y"],
        )
        .success();

    // Snapshot 1 is the file before the first change: it has no machine line yet.
    assert_eq!(
        project.stdout("", &["diff", "DEC-001", "1"]),
        format!(
            "+ - machine: y\n+ - {}\n+ - {}\n",
            machine_changed_line(FIRST_CHANGE),
            machine_changed_line(SECOND_CHANGE)
        )
    );
}

#[test]
fn a_snapshot_equal_to_the_file_gives_no_difference() {
    let project = Project::new();
    anna_decides(&project);
    let text =
        std::fs::read_to_string(project.entry_file("DEC-001")).expect("the entry file can be read");
    project.write("DOSSIER/.state/history/DEC-001/000001.md", &text);

    assert_eq!(
        project.stdout("", &["diff", "DEC-001", "1"]),
        "no difference\n"
    );
}

#[test]
fn an_entry_without_snapshots_is_exit_1() {
    let project = Project::new();
    anna_decides(&project);

    project
        .run("", &["diff", "DEC-001"])
        .code(1)
        .stderr(contains("error: no snapshots of DEC-001"));
    project
        .run("", &["diff", "DEC-001", "--json"])
        .code(1)
        .stdout(contains("\"NOT_FOUND\""));
}

#[test]
fn a_number_without_a_snapshot_and_an_unknown_entry_are_exit_1() {
    let project = one_change();

    project
        .run("", &["diff", "DEC-001", "9"])
        .code(1)
        .stderr(contains("error: no snapshot 9 of DEC-001"));
    project
        .run("", &["diff", "DEC-999"])
        .code(1)
        .stderr(contains("error: no entry DEC-999"));
}
