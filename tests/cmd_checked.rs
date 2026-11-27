//! Runs the built program for `checked`: a person says that the check of a lesson was shown to fail
//! on the mistake the lesson is about. An LLM carries it out with `--said-by <person>` (D57), and
//! the lesson gets `check-state: checked`, `--check` sets its check when it names none, and a
//! history line `checked` carries the `--by` text. An LLM without `--said-by` or for another
//! person, a worker session, an entry that is not a lesson, a lesson with no check and no
//! `--check`, and an empty `--by` are refused, and nothing is written in those cases. Each test
//! runs in its own temporary project folder, with the clock fixed by `common::Project`
//! (2026-11-20T10:00:00+01:00).

mod common;

use common::Project;
use dossier::entry::{Entry, Status};
use predicates::str::contains;

/// The fixed clock of `common::Project`, as it appears in the fields and history lines.
const NOW: &str = "2026-11-20T10:00:00+01:00";

/// Invented text for a decision, its reason, a lesson, and the check of the lesson.
const DECISION: &str = "Run the invented test rig with two inlet guide vanes.";
const REASON: &str = "An invented reason: the rig has room for two vanes.";
const STATEMENT: &str = "Steady runs with a mixing-plane interface overpredict the stall margin.";
const VALID_FOR: &str = "An invented test rig with a tip Mach number above 1.1.";
const EVIDENCE: &str = "An invented comparison of two speedlines, section 1.";
const CHECK: &str = "An invented unit test, rig_cap_is_enforced, in section 2.";
const BY: &str = "An invented run: the check fails on the old inlet cap.";

/// The text of the entry file with `id`.
fn text_of(project: &Project, id: &str) -> String {
    std::fs::read_to_string(project.entry_file(id)).expect("the entry file can be read")
}

/// The entry file with `id`, read and parsed.
fn entry_of(project: &Project, id: &str) -> Entry {
    Entry::parse(&text_of(project, id)).expect("the entry file parses")
}

/// Anna's LLM records a draft lesson, LES-001, with no check.
fn draft_lesson(project: &Project) {
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
}

/// Anna approves LES-001, which is a draft lesson made by her LLM.
fn approved_lesson(project: &Project) {
    draft_lesson(project);
    project
        .command("anna", NOW)
        .env("DOSSIER_ANSWERS", "a")
        .args(["approve", "LES-001"])
        .assert()
        .success();
}

/// Runs `checked` as `anna-claude` with `args`, at the fixed clock, without a terminal.
fn checked_as_llm(project: &Project, args: &[&str]) -> assert_cmd::assert::Assert {
    project
        .command("anna-claude", NOW)
        .arg("checked")
        .args(args)
        .assert()
}

#[test]
fn an_llm_without_said_by_is_refused_and_nothing_is_written() {
    let project = Project::new();
    draft_lesson(&project);
    let before = text_of(&project, "LES-001");

    checked_as_llm(&project, &["LES-001", "--by", BY])
        .failure()
        .code(1)
        .stderr(contains("--said-by anna"));

    assert_eq!(
        text_of(&project, "LES-001"),
        before,
        "the file is unchanged"
    );
}

#[test]
fn an_llm_with_said_by_checks_the_lesson_without_a_terminal_and_the_entry_and_journal_name_both() {
    let project = Project::new();
    draft_lesson(&project);

    checked_as_llm(
        &project,
        &["LES-001", "--by", BY, "--check", CHECK, "--said-by", "anna"],
    )
    .success()
    .stdout("LES-001: checked\n")
    .stderr("");

    let entry = entry_of(&project, "LES-001");
    assert_eq!(entry.field("check-state").as_deref(), Some("checked"));
    assert_eq!(
        entry.history().last().map(String::as_str),
        Some(format!("{NOW} anna via anna-claude: checked — {BY}").as_str())
    );

    let journal = project.entry_journal();
    let last = journal.last().expect("a journal line");
    assert_eq!(last.command, "checked");
    assert_eq!(last.id, "LES-001");
    assert_eq!(last.event, "checked");
    assert_eq!(last.detail, BY);
    assert_eq!(last.person, "anna");
    assert_eq!(last.name, "anna-claude");
    assert!(last.llm);
}

#[test]
fn an_llm_with_said_by_for_another_person_is_refused_and_nothing_is_written() {
    let project = Project::new();
    draft_lesson(&project);
    let before = text_of(&project, "LES-001");

    checked_as_llm(&project, &["LES-001", "--by", BY, "--said-by", "ben"])
        .failure()
        .code(1)
        .stderr(contains("acts for anna only"));

    assert_eq!(
        text_of(&project, "LES-001"),
        before,
        "the file is unchanged"
    );
}

#[test]
fn a_worker_session_checks_nothing_even_with_said_by() {
    let project = Project::new();
    draft_lesson(&project);
    let before = text_of(&project, "LES-001");

    project
        .command("anna-claude", NOW)
        .env("DOSSIER_ROLE", "worker")
        .args(["checked", "LES-001", "--by", BY, "--said-by", "anna"])
        .assert()
        .failure()
        .code(1)
        .stderr(contains("a worker decides nothing"));

    assert_eq!(
        text_of(&project, "LES-001"),
        before,
        "the file is unchanged"
    );
}

#[test]
fn check_sets_the_check_of_a_lesson_that_had_none() {
    let project = Project::new();
    draft_lesson(&project);
    assert_eq!(entry_of(&project, "LES-001").field("check"), None);

    checked_as_llm(
        &project,
        &["LES-001", "--by", BY, "--check", CHECK, "--said-by", "anna"],
    )
    .success()
    .stdout("LES-001: checked\n");

    let entry = entry_of(&project, "LES-001");
    assert_eq!(entry.field("check").as_deref(), Some(CHECK));
    assert_eq!(entry.field("check-state").as_deref(), Some("checked"));
}

#[test]
fn a_lesson_with_no_check_and_no_check_option_is_refused_and_nothing_is_written() {
    let project = Project::new();
    draft_lesson(&project);
    let before = text_of(&project, "LES-001");

    checked_as_llm(&project, &["LES-001", "--by", BY, "--said-by", "anna"])
        .failure()
        .code(1)
        .stderr(contains("LES-001 names no check: give --check"));

    assert_eq!(
        text_of(&project, "LES-001"),
        before,
        "the file is unchanged"
    );
}

#[test]
fn a_decision_is_refused_as_not_a_lesson_and_nothing_is_written() {
    let project = Project::new();
    project.stdout("anna", &["decide", DECISION, "--reason", REASON]);
    let before = text_of(&project, "DEC-001");

    checked_as_llm(
        &project,
        &["DEC-001", "--by", BY, "--check", CHECK, "--said-by", "anna"],
    )
    .failure()
    .code(1)
    .stderr(contains("DEC-001 is not a lesson"));

    assert_eq!(
        text_of(&project, "DEC-001"),
        before,
        "the file is unchanged"
    );
}

#[test]
fn after_checked_an_approved_lesson_is_not_shown_as_changed_after_approval() {
    let project = Project::new();
    approved_lesson(&project);

    checked_as_llm(
        &project,
        &["LES-001", "--by", BY, "--check", CHECK, "--said-by", "anna"],
    )
    .success();

    let entry = entry_of(&project, "LES-001");
    assert_eq!(entry.status(), Some(Status::Approved));
    assert_eq!(entry.field("check-state").as_deref(), Some("checked"));
    let shown = project.stdout("anna", &["show", "LES-001"]);
    assert!(
        !shown.contains("changed after approval"),
        "show marks the lesson: {shown}"
    );
}

#[test]
fn an_empty_by_is_refused_and_nothing_is_written() {
    let project = Project::new();
    draft_lesson(&project);
    let before = text_of(&project, "LES-001");

    checked_as_llm(&project, &["LES-001", "--by", "   ", "--said-by", "anna"])
        .failure()
        .code(1)
        .stderr(contains(
            "say with --by how the check was shown to catch the mistake",
        ));

    assert_eq!(
        text_of(&project, "LES-001"),
        before,
        "the file is unchanged"
    );
}
