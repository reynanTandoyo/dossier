//! Runs the built program for `runs`: the approved usage records newest first, the parameters that
//! changed from the previous run with the same model, the filters, `--compare` of two records, the
//! JSON data, and the refusals. Each test runs in its own temporary project folder with the clock
//! fixed by `common::Project` (2026-11-20T10:00:00+01:00) unless a test gives another time. The
//! invented runs are recorded with `used`.

mod common;

use common::Project;
use predicates::str::contains;
use serde_json::Value;

/// The fixed clock of `common::Project`, and a later day for the second run of a test.
const NOW: &str = "2026-11-20T10:00:00+01:00";
const LATER: &str = "2026-11-21T09:00:00+01:00";

/// Invented contents of the files of the runs.
const MODEL: &str = "invented model data\n";
const OTHER_MODEL: &str = "another invented model\n";
const RESULT: &str = "time;value\n0;1\n";

/// The lines of a listing, each with its own line break, as the program prints them.
fn lines(items: &[&str]) -> String {
    items.iter().map(|line| format!("{line}\n")).collect()
}

/// Writes the invented files of the runs: two models and the result.
fn write_files(project: &Project) {
    project.write("sim/m.sim", MODEL);
    project.write("sim/other.sim", OTHER_MODEL);
    project.write("out.csv", RESULT);
}

/// Records a run with `used` as `agent` at `now`: ExampleSolver 2406 for task T-001, the model
/// `model`, the parameters `params` (each `name=value`) and out.csv as its output. Every run
/// of a test gets the next number, USE-001, USE-002, and so on.
fn record(project: &Project, agent: &str, now: &str, model: &str, params: &[&str]) {
    let mut args = vec![
        "used",
        "ExampleSolver",
        "--version",
        "2406",
        "--task",
        "T-001",
        "--model",
        model,
        "--output",
        "out.csv",
    ];
    for param in params {
        args.push("--param");
        args.push(param);
    }
    project.run_at(agent, now, &args).success();
}

#[test]
fn runs_lists_the_newest_run_first_with_the_parameter_that_changed() {
    let project = Project::new();
    write_files(&project);
    record(&project, "anna", NOW, "sim/m.sim", &["p=1", "q=5"]);
    record(&project, "anna-claude", LATER, "sim/m.sim", &["p=2", "q=5"]);

    let out = project.stdout("anna", &["runs"]);

    // USE-002 changed p from 1 to 2; q did not change, so it is not listed.
    assert_eq!(
        out,
        lines(&[
            "2026-11-21  anna  USE-002  ExampleSolver 2406  sim/m.sim",
            "  changed: p 1 -> 2",
            "  outputs: out.csv",
            "2026-11-20  anna  USE-001  ExampleSolver 2406  sim/m.sim",
            "  outputs: out.csv",
        ])
    );
}

#[test]
fn compare_lists_the_fields_and_the_parameters_that_differ() {
    let project = Project::new();
    write_files(&project);
    record(&project, "anna", NOW, "sim/m.sim", &["p=1", "q=5"]);
    record(&project, "anna-claude", LATER, "sim/m.sim", &["p=2", "q=5"]);

    let out = project.stdout("anna", &["runs", "--compare", "USE-001", "USE-002"]);

    // The same software, version, task, model and checksum: those are not listed. The second run
    // was made through anna-claude, so its via differs; the times differ; p differs.
    assert_eq!(
        out,
        lines(&[
            "created: 2026-11-20T10:00:00+01:00 | 2026-11-21T09:00:00+01:00",
            "approved: 2026-11-20T10:00:00+01:00 by anna | 2026-11-21T09:00:00+01:00 by anna",
            "valid-from: 2026-11-20T10:00:00+01:00 | 2026-11-21T09:00:00+01:00",
            "via: - | anna-claude",
            "p: 1 | 2",
        ])
    );
}

#[test]
fn a_change_is_measured_against_the_previous_run_of_the_same_model_only() {
    let project = Project::new();
    write_files(&project);
    record(&project, "anna", NOW, "sim/m.sim", &["p=1"]);
    record(&project, "anna", LATER, "sim/other.sim", &["p=9"]);
    record(
        &project,
        "anna",
        "2026-11-21T10:00:00+01:00",
        "sim/m.sim",
        &["p=3"],
    );

    let out = project.stdout("anna", &["runs"]);

    // USE-003 is compared with USE-001, the earlier run of sim/m.sim, not with USE-002.
    assert_eq!(
        out,
        lines(&[
            "2026-11-21  anna  USE-003  ExampleSolver 2406  sim/m.sim",
            "  changed: p 1 -> 3",
            "  outputs: out.csv",
            "2026-11-21  anna  USE-002  ExampleSolver 2406  sim/other.sim",
            "  outputs: out.csv",
            "2026-11-20  anna  USE-001  ExampleSolver 2406  sim/m.sim",
            "  outputs: out.csv",
        ])
    );
}

#[test]
fn a_parameter_that_is_added_or_removed_is_a_change_with_a_dash() {
    let project = Project::new();
    write_files(&project);
    record(&project, "anna", NOW, "sim/m.sim", &["p=1"]);
    record(&project, "anna", LATER, "sim/m.sim", &["q=2"]);

    let out = project.stdout("anna", &["runs"]);

    assert_eq!(
        out,
        lines(&[
            "2026-11-21  anna  USE-002  ExampleSolver 2406  sim/m.sim",
            "  changed: p 1 -> -",
            "  changed: q - -> 2",
            "  outputs: out.csv",
            "2026-11-20  anna  USE-001  ExampleSolver 2406  sim/m.sim",
            "  outputs: out.csv",
        ])
    );
}

#[test]
fn a_run_without_a_model_shows_a_dash_and_has_no_changes() {
    let project = Project::new();
    project.write("out.csv", RESULT);
    for (now, param) in [(NOW, "p=1"), (LATER, "p=2")] {
        project
            .run_at(
                "anna",
                now,
                &[
                    "used",
                    "ExampleSolver",
                    "--version",
                    "2406",
                    "--task",
                    "T-001",
                    "--param",
                    param,
                ],
            )
            .success();
    }

    let out = project.stdout("anna", &["runs"]);

    assert_eq!(
        out,
        lines(&[
            "2026-11-21  anna  USE-002  ExampleSolver 2406  -",
            "2026-11-20  anna  USE-001  ExampleSolver 2406  -",
        ])
    );
}

#[test]
fn runs_recorded_at_the_same_time_list_the_later_record_first() {
    let project = Project::new();
    write_files(&project);
    record(&project, "anna", NOW, "sim/m.sim", &["p=1"]);
    record(&project, "anna", NOW, "sim/m.sim", &["p=2"]);

    let out = project.stdout("anna", &["runs"]);

    assert_eq!(
        out,
        lines(&[
            "2026-11-20  anna  USE-002  ExampleSolver 2406  sim/m.sim",
            "  changed: p 1 -> 2",
            "  outputs: out.csv",
            "2026-11-20  anna  USE-001  ExampleSolver 2406  sim/m.sim",
            "  outputs: out.csv",
        ])
    );
}

#[test]
fn the_filters_choose_by_task_software_and_model_ignoring_case() {
    let project = Project::new();
    write_files(&project);
    record(&project, "anna", NOW, "sim/m.sim", &["p=1"]);
    project
        .run_at(
            "anna",
            LATER,
            &[
                "used",
                "OtherSolver",
                "--version",
                "1",
                "--task",
                "T-002",
                "--model",
                "sim/other.sim",
                "--output",
                "out.csv",
            ],
        )
        .success();

    assert_eq!(
        project.stdout("anna", &["runs", "--task", "t-002"]),
        lines(&[
            "2026-11-21  anna  USE-002  OtherSolver 1  sim/other.sim",
            "  outputs: out.csv",
        ])
    );
    assert_eq!(
        project.stdout("anna", &["runs", "--software", "EXAMPLESOLVER"]),
        lines(&[
            "2026-11-20  anna  USE-001  ExampleSolver 2406  sim/m.sim",
            "  outputs: out.csv",
        ])
    );
    // A typed path with a backslash names the same stored path.
    assert_eq!(
        project.stdout("anna", &["runs", "--model", "sim\\other.sim"]),
        lines(&[
            "2026-11-21  anna  USE-002  OtherSolver 1  sim/other.sim",
            "  outputs: out.csv",
        ])
    );
    assert_eq!(
        project.stdout("anna", &["runs", "--task", "T-999"]),
        "no runs\n"
    );
}

#[test]
fn compare_refuses_an_entry_that_is_not_a_usage_record() {
    let project = Project::new();
    write_files(&project);
    record(&project, "anna", NOW, "sim/m.sim", &["p=1"]);
    project
        .run(
            "anna",
            &[
                "decide",
                "Use the invented rig for the speedline.",
                "--reason",
                "Invented reason.",
            ],
        )
        .success();

    project
        .run("anna", &["runs", "--compare", "DEC-001", "USE-001"])
        .failure()
        .code(1)
        .stderr(contains("error: DEC-001 is not a usage record"));
}

#[test]
fn compare_takes_two_ids_and_no_filter() {
    let project = Project::new();
    write_files(&project);
    record(&project, "anna", NOW, "sim/m.sim", &["p=1"]);
    record(&project, "anna", LATER, "sim/m.sim", &["p=2"]);

    project
        .run("anna", &["runs", "--compare", "USE-001"])
        .failure()
        .code(2);
    project
        .run(
            "anna",
            &["runs", "--compare", "USE-001", "USE-002", "--task", "T-001"],
        )
        .failure()
        .code(2);
}

#[test]
fn json_gives_one_object_per_run_with_its_changes() {
    let project = Project::new();
    write_files(&project);
    record(&project, "anna", NOW, "sim/m.sim", &["p=1"]);
    record(&project, "anna-claude", LATER, "sim/m.sim", &["p=2"]);

    let out = project.stdout("anna", &["--json", "runs"]);
    let value: Value = serde_json::from_str(out.trim_end()).expect("one JSON line");

    assert_eq!(value["schema"], "dossier.runs.v1");
    assert_eq!(value["ok"], true);
    let data = value["data"].as_array().expect("an array of runs");
    assert_eq!(data.len(), 2);
    assert_eq!(data[0]["id"], "USE-002");
    assert_eq!(data[0]["author"], "anna");
    assert_eq!(data[0]["via"], "anna-claude");
    assert_eq!(data[1]["via"], Value::Null);
    assert_eq!(data[0]["changed"][0]["name"], "p");
    assert_eq!(data[0]["changed"][0]["before"], "1");
    assert_eq!(data[0]["changed"][0]["after"], "2");
    assert_eq!(data[0]["outputs"][0], "out.csv");
    assert_eq!(data[1]["id"], "USE-001");
    assert_eq!(data[1]["changed"], serde_json::json!([]));
}

#[test]
fn a_project_without_runs_says_so() {
    let project = Project::new();

    assert_eq!(project.stdout("anna", &["runs"]), "no runs\n");
}

#[test]
fn a_broken_usage_file_gives_a_warning_and_the_list_still_works() {
    let project = Project::new();
    write_files(&project);
    record(&project, "anna", NOW, "sim/m.sim", &["p=1"]);
    project.write(
        "DOSSIER/usage/USE-009-broken.md",
        "<!-- dossier-format: 1 -->\nno heading here\n",
    );

    project
        .run("anna", &["runs"])
        .success()
        .stdout(contains("USE-001  ExampleSolver 2406  sim/m.sim"))
        .stderr(contains(
            "warning: DOSSIER/usage/USE-009-broken.md: line 2: no heading",
        ));
}
