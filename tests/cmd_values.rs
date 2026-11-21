//! Runs the built program for `values`: the values in force, with a prefix, on a past day and as
//! data (R-VALUE-4, R-LIFE-9). Each test runs in its own project folder, with the clock fixed by
//! `common::Project` (2026-11-20T10:00:00+01:00) unless it gives other times.

mod common;

use common::Project;
use predicates::str::contains;
use serde_json::{Value, json};

/// The fixed clock of `common::Project`.
const NOW: &str = "2026-11-20T10:00:00+01:00";

/// A later time, for a change made after the first one.
const LATER: &str = "2026-12-01T09:00:00+01:00";

/// Runs `values` with `args` as anna and returns its standard output.
fn values(project: &Project, args: &[&str]) -> String {
    let mut all = vec!["values"];
    all.extend_from_slice(args);
    project.stdout("anna", &all)
}

#[test]
fn key_values_come_first_then_by_name_each_with_its_source_and_date_and_a_count() {
    let project = Project::new();
    project
        .run(
            "anna",
            &[
                "value",
                "set",
                "design.massflow",
                "12.4 kg/s",
                "--source",
                "spec rev3",
                "--key",
            ],
        )
        .success();
    project
        .run(
            "anna",
            &[
                "value",
                "set",
                "design.gas",
                "air, ideal gas",
                "--dimensionless",
            ],
        )
        .success();
    assert_eq!(
        values(&project, &[]),
        "design.massflow  12.4 kg/s  (spec rev3, 2026-11-20)\n\
         design.gas  air, ideal gas  (2026-11-20)\n\
         2 values\n"
    );
}

#[test]
fn a_prefix_narrows_the_list_and_a_name_that_matches_nothing_gives_zero() {
    let project = Project::new();
    project
        .run(
            "anna",
            &[
                "value",
                "set",
                "design.massflow",
                "12.4 kg/s",
                "--source",
                "spec rev3",
            ],
        )
        .success();
    project
        .run(
            "anna",
            &[
                "value",
                "set",
                "design.gas",
                "air, ideal gas",
                "--dimensionless",
            ],
        )
        .success();
    assert_eq!(
        values(&project, &["design.m"]),
        "design.massflow  12.4 kg/s  (spec rev3, 2026-11-20)\n1 value\n"
    );
    assert_eq!(
        values(&project, &["design.g"]),
        "design.gas  air, ideal gas  (2026-11-20)\n1 value\n"
    );
    assert_eq!(
        values(&project, &["design"]),
        "design.gas  air, ideal gas  (2026-11-20)\n\
         design.massflow  12.4 kg/s  (spec rev3, 2026-11-20)\n\
         2 values\n"
    );
    assert_eq!(values(&project, &["pressure"]), "0 values\n");
}

#[test]
fn a_project_without_values_lists_none() {
    let project = Project::new();
    assert_eq!(values(&project, &[]), "0 values\n");
}

#[test]
fn on_a_past_day_shows_the_value_then_in_force_and_leaves_out_later_values() {
    let project = Project::new();
    project
        .run(
            "anna",
            &[
                "value",
                "set",
                "design.massflow",
                "12.1 kg/s",
                "--source",
                "spec rev2",
            ],
        )
        .success();
    project
        .run_at(
            "anna",
            LATER,
            &["value", "set", "design.massflow", "12.4 kg/s"],
        )
        .success();
    project
        .run_at(
            "anna",
            LATER,
            &[
                "value",
                "set",
                "design.gas",
                "air, ideal gas",
                "--dimensionless",
            ],
        )
        .success();

    // Today: the new value, with its source (kept, as the change named none) and its date.
    assert_eq!(
        values(&project, &[]),
        "design.gas  air, ideal gas  (2026-12-01)\n\
         design.massflow  12.4 kg/s  (spec rev2, 2026-12-01)\n\
         2 values\n"
    );
    // Before the change: the old value. Its source is left out, because a later set line could
    // have changed it; design.gas did not exist yet and is left out.
    assert_eq!(
        values(&project, &["--on", "2026-11-25"]),
        "design.massflow  12.1 kg/s  (2026-11-20)\n1 value\n"
    );
    assert_eq!(
        values(&project, &["--on", "2026-12-01"]),
        "design.gas  air, ideal gas  (2026-12-01)\n\
         design.massflow  12.4 kg/s  (spec rev2, 2026-12-01)\n\
         2 values\n"
    );
    assert_eq!(values(&project, &["--on", "2026-11-19"]), "0 values\n");
}

#[test]
fn the_json_gives_the_fields_of_each_value_and_the_value_and_unit_apart_on_a_past_day() {
    let project = Project::new();
    project
        .run(
            "anna",
            &[
                "value",
                "set",
                "design.massflow",
                "12.1 kg/s",
                "--source",
                "spec rev2",
                "--key",
            ],
        )
        .success();
    project
        .run_at(
            "anna",
            LATER,
            &["value", "set", "design.massflow", "12.4 kg/s"],
        )
        .success();

    let now: Value = serde_json::from_str(&project.stdout("anna", &["values", "--json"]))
        .expect("the JSON parses");
    assert_eq!(now["schema"], "dossier.values.v1");
    assert_eq!(now["ok"], true);
    assert_eq!(now["warnings"], json!([]));
    assert_eq!(
        now["data"],
        json!([{
            "name": "design.massflow",
            "value": "12.4",
            "unit": "kg/s",
            "source": "spec rev2",
            "valid_from": LATER,
            "key": true,
            "proposed": null,
        }])
    );

    let past: Value =
        serde_json::from_str(&project.stdout("anna", &["values", "--on", "2026-11-25", "--json"]))
            .expect("the JSON parses");
    assert_eq!(
        past["data"],
        json!([{
            "name": "design.massflow",
            "value": "12.1",
            "unit": "kg/s",
            "source": null,
            "valid_from": NOW,
            "key": true,
            "proposed": null,
        }])
    );
}

#[test]
fn a_waiting_proposal_is_shown_today_but_not_on_a_past_day() {
    let project = Project::new();
    project
        .run(
            "anna",
            &[
                "value",
                "set",
                "design.massflow",
                "12.4 kg/s",
                "--source",
                "spec rev3",
            ],
        )
        .success();
    project
        .run(
            "ben-codex",
            &["value", "set", "design.massflow", "12.9 kg/s"],
        )
        .success();

    assert_eq!(
        values(&project, &[]),
        "design.massflow  12.4 kg/s  (spec rev3, 2026-11-20)  \
         [proposed: 12.9 kg/s by ben-codex at 2026-11-20T10:00:00+01:00]\n\
         1 value\n"
    );
    assert_eq!(
        values(&project, &["--on", "2026-11-20"]),
        "design.massflow  12.4 kg/s  (spec rev3, 2026-11-20)\n1 value\n"
    );
}

#[test]
fn a_proposal_for_a_value_not_yet_in_force_is_not_listed() {
    let project = Project::new();
    project
        .run("ben-codex", &["value", "set", "design.speed", "3900 rpm"])
        .success();
    assert_eq!(values(&project, &[]), "0 values\n");
}

#[test]
fn an_on_date_that_is_not_a_date_is_refused() {
    let project = Project::new();
    project
        .run("anna", &["values", "--on", "2026-13-40"])
        .failure()
        .code(1)
        .stderr(contains("--on needs a date such as 2026-11-21"));
}
