//! Runs the built program for `sweep`: two approved entries whose titles share their rare words
//! are one pair, with the rules for judging it, and an unrelated entry is not named; a link
//! between the two leaves no pair; a draft is never paired; `--limit` cuts the pairs; `--json`
//! gives the IDs, kinds, titles and scores; and nothing is written. Each test runs in its own
//! temporary project folder with the clock fixed by `common::Project` (2026-11-20T10:00:00+01:00).

mod common;

use common::Project;
use std::fs;

/// Invented titles. The first two share the rare words labyrinth, seal, clearance and millimetres;
/// the third is about something else.
const LABYRINTH_ONE: &str = "Labyrinth seal clearance fixed at two millimetres";
const LABYRINTH_TWO: &str = "Labyrinth seal clearance kept at two millimetres for the rig";
const LABYRINTH_THREE: &str =
    "Labyrinth seal clearance checked at two millimetres after the rebuild";
const COOLANT: &str = "Coolant manifold pressure logged every ten seconds";

/// Invented reasons, one for the labyrinth decisions and one for the coolant decision.
const LABYRINTH_REASON: &str = "An invented reason: the tighter gap leaked less in the rig.";
const COOLANT_REASON: &str = "An invented reason: the logged values drifted slowly over the shift.";

/// Invented parts of a lesson for the LLM draft.
const VALID_FOR: &str = "An invented test rig with a tip Mach number above 1.1.";
const EVIDENCE: &str = "An invented comparison of two speedlines, section 1.";

/// The first line of the rules printed after the pairs.
const RULES_START: &str = "Judge each pair with your person; change nothing without their yes:\n";

/// Decides an approved decision as anna, with its reason.
fn decide(project: &Project, title: &str, reason: &str) {
    project.stdout("anna", &["decide", title, "--reason", reason]);
}

/// Runs `learn` as anna-claude with a complete lesson: a draft, not in force.
fn draft_lesson(project: &Project, statement: &str) {
    project
        .run(
            "anna-claude",
            &[
                "learn",
                statement,
                "--valid-for",
                VALID_FOR,
                "--evidence",
                EVIDENCE,
                "--confidence",
                "simulated",
            ],
        )
        .success();
}

/// The number of pair lines in the output: each pair has two, and the decision kind is in both.
fn pair_lines(stdout: &str) -> usize {
    stdout
        .lines()
        .filter(|line| line.contains(" · decision · "))
        .count()
}

/// The journal, as its text.
fn journal_text(project: &Project) -> String {
    project.read("DOSSIER/.state/journal.jsonl")
}

#[test]
fn two_decisions_with_the_same_rare_words_are_one_pair_and_the_third_is_not_named() {
    let project = Project::new();
    decide(&project, LABYRINTH_ONE, LABYRINTH_REASON);
    decide(&project, LABYRINTH_TWO, LABYRINTH_REASON);
    decide(&project, COOLANT, COOLANT_REASON);

    let stdout = project.stdout("", &["sweep"]);
    let head = format!(
        "1 pair of entries in force looks alike:\n\n\
         DEC-001 · decision · {LABYRINTH_ONE}\n\
         DEC-002 · decision · {LABYRINTH_TWO}\n\n\
         {RULES_START}"
    );
    assert!(stdout.starts_with(&head), "{stdout}");
    assert!(
        stdout
            .contains("- The same lesson under other conditions (Valid for) is not a duplicate.\n"),
        "{stdout}"
    );
    assert!(!stdout.contains("DEC-003"), "{stdout}");
}

#[test]
fn a_link_between_the_two_leaves_no_pair() {
    let project = Project::new();
    decide(&project, LABYRINTH_ONE, LABYRINTH_REASON);
    decide(&project, LABYRINTH_TWO, LABYRINTH_REASON);
    assert_eq!(
        project.stdout("", &["sweep"]).lines().next(),
        Some("1 pair of entries in force looks alike:")
    );

    project.stdout("anna", &["link", "DEC-001", "DEC-002"]);

    assert_eq!(
        project.stdout("", &["sweep"]),
        "no entries in force look alike\n"
    );
}

#[test]
fn a_draft_that_looks_like_a_decision_is_not_paired() {
    let project = Project::new();
    decide(&project, LABYRINTH_ONE, LABYRINTH_REASON);
    decide(&project, LABYRINTH_TWO, LABYRINTH_REASON);
    draft_lesson(
        &project,
        "Labyrinth seal clearance at two millimetres closes the leak path",
    );

    let stdout = project.stdout("", &["sweep"]);
    assert!(
        stdout.starts_with("1 pair of entries in force looks alike:\n"),
        "{stdout}"
    );
    assert!(!stdout.contains("LES-001"), "{stdout}");
    assert_eq!(pair_lines(&stdout), 2, "{stdout}");
}

#[test]
fn limit_cuts_the_pairs_and_the_count_line_follows_the_cut() {
    let project = Project::new();
    decide(&project, LABYRINTH_ONE, LABYRINTH_REASON);
    decide(&project, LABYRINTH_TWO, LABYRINTH_REASON);
    decide(&project, LABYRINTH_THREE, LABYRINTH_REASON);

    let all = project.stdout("", &["sweep"]);
    assert!(
        all.starts_with("3 pairs of entries in force look alike:\n"),
        "{all}"
    );
    assert_eq!(pair_lines(&all), 6, "{all}");

    let one = project.stdout("", &["sweep", "--limit", "1"]);
    assert!(
        one.starts_with("1 pair of entries in force looks alike:\n"),
        "{one}"
    );
    assert_eq!(pair_lines(&one), 2, "{one}");
}

#[test]
fn json_gives_the_ids_kinds_titles_and_a_score_of_each_pair() {
    let project = Project::new();
    decide(&project, LABYRINTH_ONE, LABYRINTH_REASON);
    decide(&project, LABYRINTH_TWO, LABYRINTH_REASON);

    let stdout = project.stdout("", &["--json", "sweep"]);
    let value: serde_json::Value = serde_json::from_str(&stdout).expect("one JSON line");
    let pairs = value["data"]["pairs"].as_array().expect("a list of pairs");

    assert_eq!(pairs.len(), 1, "{stdout}");
    assert_eq!(pairs[0]["a"]["id"], "DEC-001", "{stdout}");
    assert_eq!(pairs[0]["a"]["kind"], "decision", "{stdout}");
    assert_eq!(pairs[0]["a"]["title"], LABYRINTH_ONE, "{stdout}");
    assert_eq!(pairs[0]["b"]["id"], "DEC-002", "{stdout}");
    assert_eq!(pairs[0]["b"]["kind"], "decision", "{stdout}");
    assert_eq!(pairs[0]["b"]["title"], LABYRINTH_TWO, "{stdout}");
    assert!(pairs[0]["score"].as_u64().is_some(), "{stdout}");
}

#[test]
fn sweep_writes_nothing_and_the_journal_keeps_its_lines() {
    let project = Project::new();
    decide(&project, LABYRINTH_ONE, LABYRINTH_REASON);
    decide(&project, LABYRINTH_TWO, LABYRINTH_REASON);
    decide(&project, COOLANT, COOLANT_REASON);
    let journal_before = journal_text(&project);
    let lines_before = journal_before.lines().count();
    let first_before = fs::read(project.entry_file("DEC-001")).expect("the entry file can be read");
    let second_before =
        fs::read(project.entry_file("DEC-002")).expect("the entry file can be read");

    project.stdout("", &["sweep"]);

    let journal_after = journal_text(&project);
    assert_eq!(journal_after.lines().count(), lines_before);
    assert_eq!(journal_after, journal_before);
    assert_eq!(
        fs::read(project.entry_file("DEC-001")).expect("the entry file can be read"),
        first_before
    );
    assert_eq!(
        fs::read(project.entry_file("DEC-002")).expect("the entry file can be read"),
        second_before
    );
}
