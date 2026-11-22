//! Runs the built program for `reviewed`: a person at a terminal confirms that an entry still
//! holds. The entry gets `reviewed` with the time and the person, its review date is set to
//! `--next` or removed, and a history line `reviewed` names the next date. An LLM and a missing
//! terminal are refused, a date that is not YYYY-MM-DD is refused, and nothing is written in
//! those cases. Each test runs in its own temporary project folder, with the clock fixed by
//! `common::Project` (2026-11-20T10:00:00+01:00).

mod common;

use common::Project;
use dossier::entry::{Entry, Status};
use predicates::str::contains;

/// The fixed clock of `common::Project`, as it appears in the fields and history lines.
const NOW: &str = "2026-11-20T10:00:00+01:00";

/// Invented text for a decision, its reason, and a lesson.
const DECISION: &str = "Run the invented test rig with two inlet guide vanes.";
const REASON: &str = "An invented reason: the rig has room for two vanes.";
const STATEMENT: &str = "Steady runs with a mixing-plane interface overpredict the stall margin.";
const VALID_FOR: &str = "An invented test rig with a tip Mach number above 1.1.";
const EVIDENCE: &str = "An invented comparison of two speedlines, section 1.";

/// A review date that has passed at the fixed clock.
const PAST: &str = "2026-10-01";

/// The text of the entry file with `id`.
fn text_of(project: &Project, id: &str) -> String {
    std::fs::read_to_string(project.entry_file(id)).expect("the entry file can be read")
}

/// The entry file with `id`, read and parsed.
fn entry_of(project: &Project, id: &str) -> Entry {
    Entry::parse(&text_of(project, id)).expect("the entry file parses")
}

/// Anna records an approved decision, DEC-001, whose review date has passed.
fn decision_due_for_review(project: &Project) {
    project.stdout("anna", &["decide", DECISION, "--reason", REASON]);
    let review = format!("review={PAST}");
    project.stdout("anna", &["set", "DEC-001", "--field", review.as_str()]);
}

/// Runs `reviewed` with `args` as `agent`, at the fixed clock, with a terminal. DOSSIER_ANSWERS
/// makes the run a terminal run; `reviewed` never asks anything, so the empty answer is unused.
fn reviewed_as(project: &Project, agent: &str, args: &[&str]) -> assert_cmd::assert::Assert {
    project
        .command(agent, NOW)
        .env("DOSSIER_ANSWERS", "")
        .arg("reviewed")
        .args(args)
        .assert()
}

#[test]
fn a_person_confirms_an_entry_whose_review_date_has_passed_and_the_date_is_removed() {
    let project = Project::new();
    decision_due_for_review(&project);
    let before = entry_of(&project, "DEC-001");
    assert_eq!(before.field("review").as_deref(), Some(PAST));

    reviewed_as(&project, "ben", &["DEC-001"])
        .success()
        .stdout("DEC-001: reviewed\n");

    let entry = entry_of(&project, "DEC-001");
    assert_eq!(entry.field("review"), None);
    assert_eq!(
        entry.field("reviewed").as_deref(),
        Some(format!("{NOW} by ben").as_str())
    );
    assert_eq!(
        entry.history().last().map(String::as_str),
        Some("2026-11-20T10:00:00+01:00 ben: reviewed")
    );
    // The confirmation changes no part of the decision itself.
    assert_eq!(entry.status(), Some(Status::Approved));
    assert_eq!(entry.field("approved"), before.field("approved"));
    assert_eq!(entry.section("Decision"), before.section("Decision"));
    assert_eq!(entry.section("Reason"), before.section("Reason"));
}

#[test]
fn next_sets_a_new_review_date_and_the_history_names_it() {
    let project = Project::new();
    decision_due_for_review(&project);

    reviewed_as(&project, "ben", &["DEC-001", "--next", "2027-05-20"])
        .success()
        .stdout("DEC-001: reviewed, next review 2027-05-20\n");

    let entry = entry_of(&project, "DEC-001");
    assert_eq!(entry.field("review").as_deref(), Some("2027-05-20"));
    assert_eq!(
        entry.field("reviewed").as_deref(),
        Some(format!("{NOW} by ben").as_str())
    );
    assert_eq!(
        entry.history().last().map(String::as_str),
        Some("2026-11-20T10:00:00+01:00 ben: reviewed — next 2027-05-20")
    );
}

#[test]
fn the_json_data_names_the_confirmation_and_the_next_date() {
    let project = Project::new();
    decision_due_for_review(&project);

    reviewed_as(
        &project,
        "ben",
        &["DEC-001", "--next", "2027-05-20", "--json"],
    )
    .success()
    .stdout(contains("\"schema\":\"dossier.reviewed.v1\""))
    .stdout(contains("\"review\":\"2027-05-20\""))
    .stdout(contains(format!("\"reviewed\":\"{NOW} by ben\"")));

    reviewed_as(&project, "ben", &["DEC-001", "--json"])
        .success()
        .stdout(contains("\"review\":null"));
}

#[test]
fn confirming_an_approved_lesson_is_not_taken_for_a_change_by_hand() {
    let project = Project::new();
    project.stdout(
        "anna-claude",
        &[
            "learn",
            STATEMENT,
            "--valid-for",
            VALID_FOR,
            "--evidence",
            EVIDENCE,
            "--confidence",
            "simulated",
        ],
    );
    project
        .command("anna", NOW)
        .env("DOSSIER_ANSWERS", "a")
        .args(["approve", "LES-001"])
        .assert()
        .success();

    reviewed_as(&project, "anna", &["LES-001", "--next", "2027-01-15"]).success();

    // The approved text is the reviewed text, so show does not mark the lesson as changed.
    let text = text_of(&project, "LES-001");
    assert!(text.contains("- review: 2027-01-15\n"));
    assert_eq!(project.stdout("anna", &["show", "LES-001"]), text);
}

#[test]
fn an_llm_cannot_confirm_and_nothing_is_written() {
    let project = Project::new();
    decision_due_for_review(&project);
    let before = text_of(&project, "DEC-001");

    reviewed_as(&project, "anna-claude", &["DEC-001"])
        .failure()
        .code(1)
        .stderr(contains(
            "error: reviewed is for a person; anna-claude is an LLM",
        ));
    reviewed_as(&project, "anna-claude", &["DEC-001", "--json"])
        .code(1)
        .stdout(contains("\"code\":\"NEEDS_PERSON\""));

    assert_eq!(text_of(&project, "DEC-001"), before);
}

#[test]
fn without_a_terminal_a_person_is_refused_and_nothing_is_written() {
    let project = Project::new();
    decision_due_for_review(&project);
    let before = text_of(&project, "DEC-001");

    project
        .command("ben", NOW)
        .env_remove("DOSSIER_ANSWERS")
        .args(["reviewed", "DEC-001", "--json"])
        .assert()
        .code(1)
        .stdout(contains("\"code\":\"NEEDS_TERMINAL\""));

    assert_eq!(text_of(&project, "DEC-001"), before);
}

#[test]
fn a_next_date_must_be_a_real_date_of_the_form_year_month_day() {
    let project = Project::new();
    decision_due_for_review(&project);
    let before = text_of(&project, "DEC-001");

    for bad in ["2027-1-5", "2027-02-30", "20-11-2027", "tomorrow"] {
        reviewed_as(&project, "ben", &["DEC-001", "--next", bad])
            .failure()
            .code(1)
            .stderr(contains("error: --next needs a date such as 2027-11-20"));
    }

    assert_eq!(text_of(&project, "DEC-001"), before);
}
