//! Runs the built program for `history`: an entry's history lines and snapshots (text and JSON),
//! the entries of a task in time order (text and JSON), a task with nothing recorded, an unknown
//! ID as exit 1, and a broken file shown as a warning with its path relative to the project.
//! Each test runs in its own project folder, with the clock fixed by `common::Project`.

mod common;

use common::Project;
use predicates::str::contains;
use serde_json::{Value, json};

/// Invented texts. The fixed clock of `common::Project` is CREATED.
const CREATED: &str = "2026-11-20T10:00:00+01:00";
const CHANGE_TIME: &str = "2026-11-21T09:00:00+01:00";
const DECISION_TITLE: &str = "Run the last three speedline points transient";
const DECISION: &str = "The last three points of every speedline are run transient.";
const REASON: &str = "Steady runs converged where the transient run already showed rotating stall.";
const ASSUMPTION_TIME: &str = "2026-11-21T09:00:00+01:00";
const ASSUMPTION_TITLE: &str = "Inlet profile is uniform";
const ASSUMPTION: &str = "The total pressure at the inlet is uniform over the span.";
const ASSUMPTION_REASON: &str = "No measured profile was delivered.";
const FINDING_TIME: &str = "2026-11-22T09:00:00+01:00";
const FINDING_TITLE: &str = "Halving the time step did not remove the oscillation";
const FINDING: &str = "The mass flow still oscillated with half the time step.";
const OTHER_TIME: &str = "2026-11-21T12:00:00+01:00";
const OTHER_FINDING: &str = "An invented finding that belongs to another task.";
const OTHER_TITLE: &str = "A finding of another task";

/// The history line of the creation, by anna on the fixed clock, and of the approval.
fn created_line() -> String {
    format!("{CREATED} anna: created")
}

fn approved_line() -> String {
    format!("{CREATED} anna: approved")
}

/// The history line of the change made by `set` on CHANGE_TIME.
fn changed_line() -> String {
    format!("{CHANGE_TIME} anna: changed — field machine")
}

/// A project with DEC-001 for task T-042, written by anna and approved at once, and ASM-001,
/// written by ben-codex on ASSUMPTION_TIME; FND-001 belongs to T-043 and FND-002 to T-042. IDs
/// and times do not agree in order, so that the listing has to sort by time.
fn task_project() -> Project {
    let project = Project::new();
    project
        .run(
            "anna",
            &[
                "decide",
                DECISION,
                "--reason",
                REASON,
                "--title",
                DECISION_TITLE,
                "--task",
                "T-042",
            ],
        )
        .success();
    project
        .run_at(
            "ben-codex",
            ASSUMPTION_TIME,
            &[
                "assume",
                ASSUMPTION,
                "--reason",
                ASSUMPTION_REASON,
                "--title",
                ASSUMPTION_TITLE,
                "--task",
                "T-042",
            ],
        )
        .success();
    project
        .run_at(
            "anna",
            OTHER_TIME,
            &[
                "found",
                OTHER_FINDING,
                "--title",
                OTHER_TITLE,
                "--task",
                "T-043",
            ],
        )
        .success();
    project
        .run_at(
            "anna",
            FINDING_TIME,
            &[
                "found",
                FINDING,
                "--title",
                FINDING_TITLE,
                "--task",
                "T-042",
            ],
        )
        .success();
    project
}

#[test]
fn after_add_and_set_an_entry_shows_created_approved_and_changed_and_one_snapshot() {
    let project = Project::new();
    let decision = format!("Decision={DECISION}");
    let reason = format!("Reason={REASON}");
    project.stdout(
        "anna",
        &[
            "add",
            "decision",
            "--title",
            DECISION_TITLE,
            "--section",
            decision.as_str(),
            "--section",
            reason.as_str(),
        ],
    );
    project
        .run_at(
            "anna",
            CHANGE_TIME,
            &["set", "DEC-001", "--field", "machine=pump"],
        )
        .success();

    let expected = format!(
        "{}\n{}\n{}\n\nsnapshots:\nn 1  {CHANGE_TIME}  anna  set\n",
        created_line(),
        approved_line(),
        changed_line()
    );
    for id in ["DEC-001", "dec-001", "P-001:DEC-001"] {
        assert_eq!(project.stdout("", &["history", id]), expected, "{id}");
    }
}

#[test]
fn json_gives_the_history_lines_and_the_snapshot_of_an_entry() {
    let project = task_project();
    project
        .run_at(
            "anna",
            CHANGE_TIME,
            &["set", "DEC-001", "--field", "machine=pump"],
        )
        .success();

    let stdout = project.stdout("", &["history", "DEC-001", "--json"]);
    let envelope: Value = serde_json::from_str(stdout.trim()).expect("one line of JSON");

    assert_eq!(envelope["schema"], "dossier.history.v1");
    assert_eq!(envelope["ok"], true);
    assert_eq!(envelope["warnings"], json!([]));
    assert_eq!(
        envelope["data"],
        json!({
            "id": "DEC-001",
            "history": [created_line(), approved_line(), changed_line()],
            "snapshots": [
                {"n": 1, "time": CHANGE_TIME, "name": "anna", "command": "set"}
            ],
        })
    );
}

#[test]
fn an_entry_without_snapshots_says_none() {
    let project = task_project();

    let stdout = project.stdout("", &["history", "FND-002"]);

    assert_eq!(
        stdout,
        format!(
            "{FINDING_TIME} anna: created\n{FINDING_TIME} anna: approved\n\nsnapshots:\nnone\n"
        )
    );
}

#[test]
fn a_task_lists_its_entries_oldest_first_and_leaves_out_the_other_tasks() {
    let project = task_project();

    // The target is given in lower case: the task is compared ignoring case.
    let stdout = project.stdout("", &["history", "t-042"]);

    assert_eq!(
        stdout,
        format!(
            "{CREATED}  anna  DEC-001  {DECISION_TITLE}\n\
             {ASSUMPTION_TIME}  ben  ASM-001  {ASSUMPTION_TITLE}\n\
             {FINDING_TIME}  anna  FND-002  {FINDING_TITLE}\n"
        )
    );
}

#[test]
fn json_gives_the_entries_of_a_task_in_time_order() {
    let project = task_project();

    let stdout = project.stdout("", &["history", "T-042", "--json"]);
    let envelope: Value = serde_json::from_str(stdout.trim()).expect("one line of JSON");

    assert_eq!(envelope["schema"], "dossier.history.v1");
    assert_eq!(envelope["ok"], true);
    assert_eq!(
        envelope["data"],
        json!({
            "task": "T-042",
            "entries": [
                {"id": "DEC-001", "title": DECISION_TITLE, "author": "anna", "created": CREATED},
                {"id": "ASM-001", "title": ASSUMPTION_TITLE, "author": "ben", "created": ASSUMPTION_TIME},
                {"id": "FND-002", "title": FINDING_TITLE, "author": "anna", "created": FINDING_TIME},
            ],
        })
    );
}

#[test]
fn a_task_with_nothing_recorded_says_so() {
    let project = task_project();

    assert_eq!(
        project.stdout("", &["history", "T-999"]),
        "nothing recorded for T-999\n"
    );
    let stdout = project.stdout("", &["history", "T-999", "--json"]);
    let envelope: Value = serde_json::from_str(stdout.trim()).expect("one line of JSON");
    assert_eq!(envelope["data"], json!({"task": "T-999", "entries": []}));
}

#[test]
fn an_unknown_entry_is_exit_1_and_as_json_names_not_found() {
    let project = task_project();

    project
        .run("", &["history", "DEC-999"])
        .code(1)
        .stderr(contains("error: no entry DEC-999"));
    project
        .run("", &["history", "DEC-999", "--json"])
        .code(1)
        .stdout(contains("\"NOT_FOUND\""));
}

#[test]
fn a_broken_file_is_a_warning_with_its_path_shown_from_the_project_folder() {
    let project = task_project();
    project.write(
        "DOSSIER/decisions/DEC-009-broken.md",
        "<!-- dossier-format: 1 -->\nthis file has no heading\n",
    );

    project
        .run("", &["history", "T-042"])
        .success()
        .stderr(contains(
            "warning: DOSSIER/decisions/DEC-009-broken.md: line 2: no heading",
        ));
}
