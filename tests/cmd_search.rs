//! Runs the built program for `search`: a word in a title is found, every word must be present,
//! drafts are found only with `--drafts` and are marked, `--kind` narrows, title matches come
//! first, `--full` prints whole entries, and no match prints `0 matches`. Each test runs in its
//! own temporary project with the clock fixed at 2026-11-20T10:00:00+01:00.

mod common;

use common::Project;
use predicates::str::contains;
use std::fs;

/// A later time than the fixed clock, for entries that must sort after the others.
const LATER: &str = "2026-11-21T09:00:00+01:00";

/// The IDs at the start of the result lines of `stdout`, in order. A result line has two spaces
/// after its ID; the count line has none.
fn result_ids(stdout: &str) -> Vec<String> {
    stdout
        .lines()
        .filter(|line| line.contains("  "))
        .filter_map(|line| line.split("  ").next())
        .map(str::to_string)
        .collect()
}

#[test]
fn a_word_in_the_title_is_found_whatever_its_case() {
    let project = Project::new();
    project
        .run(
            "anna",
            &[
                "decide",
                "Use transient runs for the last three speedline points",
                "--reason",
                "Steady runs converged where the transient run already showed stall.",
            ],
        )
        .success();

    let expected = "DEC-001  Use transient runs for the last three speedline points  — \
        # DEC-001 · Use transient runs for the last three speedline points\n\
        1 match\n";
    assert_eq!(project.stdout("", &["search", "speedline"]), expected);
    assert_eq!(project.stdout("", &["search", "SPEEDLINE"]), expected);
}

#[test]
fn two_words_must_both_be_present() {
    let project = Project::new();
    project
        .run(
            "anna",
            &[
                "decide",
                "Use transient runs for the last three speedline points",
                "--reason",
                "Rotating stall was visible in the transient run.",
            ],
        )
        .success();
    project
        .run(
            "anna",
            &["found", "Mixing-plane runs converge at high back pressure"],
        )
        .success();

    let both = project.stdout("", &["search", "transient", "stall"]);
    assert_eq!(result_ids(&both), ["DEC-001"]);
    assert!(both.ends_with("1 match\n"), "{both}");

    project
        .run("", &["search", "transient", "mixing"])
        .success()
        .stdout("0 matches\n");
}

#[test]
fn a_draft_is_found_only_with_drafts_and_is_marked() {
    let project = Project::new();
    project
        .run(
            "anna",
            &[
                "found",
                "Compressor stall seen in the monitor plot",
                "--later",
            ],
        )
        .success();

    project
        .run("", &["search", "stall"])
        .success()
        .stdout("0 matches\n");

    let expected = "FND-001  [draft] Compressor stall seen in the monitor plot  — \
        # FND-001 · Compressor stall seen in the monitor plot\n\
        1 match\n";
    assert_eq!(
        project.stdout("", &["search", "stall", "--drafts"]),
        expected
    );
}

#[test]
fn kind_narrows_the_results() {
    let project = Project::new();
    project
        .run(
            "anna",
            &[
                "decide",
                "Keep the stall margin check in every speedline",
                "--reason",
                "The margin is the design limit.",
            ],
        )
        .success();
    project
        .run("anna", &["found", "Stall started at the lowest mass flow"])
        .success();

    let all = project.stdout("", &["search", "stall"]);
    assert_eq!(result_ids(&all).len(), 2, "{all}");

    let decisions = project.stdout("", &["search", "stall", "--kind", "decision"]);
    assert_eq!(result_ids(&decisions), ["DEC-001"]);

    let findings = project.stdout("", &["search", "stall", "--kind", "finding"]);
    assert_eq!(result_ids(&findings), ["FND-001"]);
}

#[test]
fn title_matches_come_first_then_tags_then_the_rest_newest_first() {
    let project = Project::new();
    // A title match, the older one first.
    project
        .run(
            "anna",
            &["found", "Bearing temperature drifted during the first run"],
        )
        .success();
    project
        .run_at(
            "anna",
            LATER,
            &["found", "Bearing noise rose in the second run"],
        )
        .success();
    // A word in a tag field only.
    project
        .run_at(
            "anna",
            "2026-11-21T10:00:00+01:00",
            &[
                "found",
                "Rig vibration stayed low",
                "--machine",
                "bearing rig",
            ],
        )
        .success();
    // A word in the text only, and the newest of all.
    project
        .run_at(
            "anna",
            "2026-11-21T11:00:00+01:00",
            &[
                "assume",
                "Seal leakage is negligible",
                "--reason",
                "The bearing housing is sealed",
            ],
        )
        .success();

    let stdout = project.stdout("", &["search", "bearing"]);
    assert_eq!(
        result_ids(&stdout),
        ["FND-002", "FND-001", "FND-003", "ASM-001"],
        "{stdout}"
    );
    assert!(stdout.ends_with("4 matches\n"), "{stdout}");
}

#[test]
fn full_prints_whole_entries_separated_by_a_line_of_dashes() {
    let project = Project::new();
    project
        .run(
            "anna",
            &[
                "decide",
                "Use transient runs for the last three speedline points",
                "--reason",
                "Rotating stall was visible in the transient run.",
            ],
        )
        .success();
    project
        .run_at(
            "anna",
            LATER,
            &[
                "found",
                "Speedline monitor plots agree with the transient run",
            ],
        )
        .success();

    let first = fs::read_to_string(project.entry_file("FND-001")).expect("the finding file");
    let second = fs::read_to_string(project.entry_file("DEC-001")).expect("the decision file");
    let expected = format!("{first}---\n{second}2 matches\n");
    assert_eq!(
        project.stdout("", &["search", "speedline", "--full"]),
        expected
    );
}

#[test]
fn no_match_prints_zero_matches_and_exits_zero() {
    let project = Project::new();
    project
        .run(
            "anna",
            &["found", "The mass flow still oscillated by 0.4 %"],
        )
        .success();

    project
        .run("", &["search", "zebra"])
        .success()
        .code(0)
        .stdout("0 matches\n");
    project
        .run("", &["search", "zebra", "--full"])
        .success()
        .stdout("0 matches\n");
}

#[test]
fn a_blank_word_is_refused_as_a_usage_error() {
    let project = Project::new();
    project
        .run("", &["search", "   "])
        .code(2)
        .stderr(contains("needs at least one word that is not blank"));
}

#[test]
fn json_gives_the_id_title_status_and_line_of_each_result() {
    let project = Project::new();
    project
        .run(
            "anna",
            &[
                "decide",
                "Use transient runs for the last three speedline points",
                "--reason",
                "Rotating stall was visible in the transient run.",
            ],
        )
        .success();
    project
        .run_at(
            "anna",
            LATER,
            &["found", "Speedline plot kept for review", "--later"],
        )
        .success();

    let stdout = project.stdout("", &["--json", "search", "speedline", "--drafts"]);
    let value: serde_json::Value = serde_json::from_str(&stdout).expect("one JSON line");
    assert_eq!(value["schema"], "dossier.search.v1");
    assert_eq!(value["ok"], true);
    let expected = serde_json::json!([
        {
            "id": "FND-001",
            "title": "Speedline plot kept for review",
            "status": "draft",
            "line": "# FND-001 · Speedline plot kept for review",
        },
        {
            "id": "DEC-001",
            "title": "Use transient runs for the last three speedline points",
            "status": "approved",
            "line": "# DEC-001 · Use transient runs for the last three speedline points",
        },
    ]);
    assert_eq!(value["data"], expected);
}

#[test]
fn an_approved_lesson_changed_by_hand_is_marked() {
    let project = Project::new();
    project
        .run(
            "anna",
            &[
                "learn",
                "Steady runs overpredict the stall margin",
                "--valid-for",
                "Transonic front stages",
                "--evidence",
                "Report section 4.2",
                "--confidence",
                "measured",
            ],
        )
        .success();

    let heading = "LES-001  Steady runs overpredict the stall margin  — \
        # LES-001 · Steady runs overpredict the stall margin";
    assert_eq!(
        project.stdout("", &["search", "stall"]),
        format!("{heading}\n1 match\n")
    );

    let path = project.entry_file("LES-001");
    let mut text = fs::read_to_string(&path).expect("the lesson file");
    text.push_str("\nChecked again by hand.\n");
    fs::write(&path, text).expect("the lesson file is changed");

    let marked = "LES-001  [changed after approval] Steady runs overpredict the stall margin  — \
        # LES-001 · Steady runs overpredict the stall margin\n\
        1 match\n";
    assert_eq!(project.stdout("", &["search", "stall"]), marked);
}
