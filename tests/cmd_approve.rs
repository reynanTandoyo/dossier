//! Runs the built program for `approve`: a person at a terminal approves or rejects drafts, one
//! answer per draft. An LLM cannot approve, and nothing is asked without a terminal. Each test
//! runs in its own project folder, with the clock fixed by `common::Project`.

mod common;

use common::Project;
use dossier::entry::{Entry, Status};
use predicates::str::contains;

/// The fixed clock of `common::Project`.
const NOW: &str = "2026-11-20T10:00:00+01:00";

/// Invented text for the parts of a lesson.
const STATEMENT: &str = "Steady runs with a mixing-plane interface overpredict the stall margin.";
const VALID_FOR: &str = "An invented test rig with a tip Mach number above 1.1.";
const EVIDENCE: &str = "An invented comparison of two speedlines, section 1.";

/// Runs `learn` as anna-claude with a complete lesson: a draft, with the next lesson ID.
fn draft_lesson(project: &Project) {
    project
        .run(
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
        )
        .success();
}

/// Runs `approve` with `ids` as `agent`, answering `answers` (DOSSIER_ANSWERS, separated by `|`).
fn approve_with(
    project: &Project,
    agent: &str,
    answers: &str,
    ids: &[&str],
) -> assert_cmd::assert::Assert {
    project
        .command(agent, NOW)
        .env("DOSSIER_ANSWERS", answers)
        .arg("approve")
        .args(ids)
        .assert()
}

/// The entry file with `id`, read and parsed.
fn entry_of(project: &Project, id: &str) -> Entry {
    let text = std::fs::read_to_string(project.entry_file(id)).expect("the entry file is read");
    Entry::parse(&text).expect("the entry parses")
}

#[test]
fn a_person_approves_a_draft_and_the_entry_records_who_and_when() {
    let project = Project::new();
    draft_lesson(&project);
    approve_with(&project, "anna", "a", &["LES-001"])
        .success()
        .stdout("LES-001: approved\n")
        .stderr(contains("# LES-001 ·"));

    let entry = entry_of(&project, "LES-001");
    assert_eq!(entry.status(), Some(Status::Approved));
    assert_eq!(
        entry.field("approved").as_deref(),
        Some(format!("{NOW} by anna").as_str())
    );
    assert_eq!(entry.field("valid-from").as_deref(), Some(NOW));
    // The author and the LLM that drafted it are kept; the approval is a history line of its own.
    assert_eq!(entry.field("author").as_deref(), Some("anna"));
    assert_eq!(entry.field("via").as_deref(), Some("anna-claude"));
    assert_eq!(
        entry.history(),
        [
            format!("{NOW} anna via anna-claude: created"),
            format!("{NOW} anna: approved"),
        ]
    );
    // An approved lesson has its checksum recorded (R-LIFE-6).
    assert!(
        project
            .read("DOSSIER/.state/approved.json")
            .contains("LES-001")
    );
}

#[test]
fn answering_r_rejects_the_draft_and_records_no_approval() {
    let project = Project::new();
    draft_lesson(&project);
    approve_with(&project, "anna", "r", &["LES-001"])
        .success()
        .stdout("LES-001: rejected\n");

    let entry = entry_of(&project, "LES-001");
    assert_eq!(entry.status(), Some(Status::Rejected));
    assert_eq!(entry.field("approved"), None);
    assert_eq!(entry.field("valid-from"), None);
    assert_eq!(
        entry.history().last().map(String::as_str),
        Some(format!("{NOW} anna: rejected").as_str())
    );
    assert!(!project.root().join("DOSSIER/.state/approved.json").exists());
}

#[test]
fn an_answer_is_read_in_any_case_and_the_word_approve_works() {
    let project = Project::new();
    draft_lesson(&project);
    draft_lesson(&project);
    approve_with(&project, "anna", "A|Approve", &["LES-001", "LES-002"])
        .success()
        .stdout("LES-001: approved\nLES-002: approved\n");
    assert_eq!(
        entry_of(&project, "LES-002").status(),
        Some(Status::Approved)
    );
}

#[test]
fn any_other_answer_skips_the_draft_and_leaves_its_file_as_it_was() {
    let project = Project::new();
    draft_lesson(&project);
    let before = std::fs::read(project.entry_file("LES-001")).expect("the entry file is read");
    approve_with(&project, "anna", "s", &["LES-001"])
        .success()
        .stdout("LES-001: skipped\n");
    assert_eq!(
        std::fs::read(project.entry_file("LES-001")).expect("the entry file is read"),
        before
    );
    assert_eq!(entry_of(&project, "LES-001").status(), Some(Status::Draft));
}

#[test]
fn each_draft_gets_its_own_answer_in_the_order_given() {
    let project = Project::new();
    draft_lesson(&project);
    draft_lesson(&project);
    approve_with(&project, "anna", "r|a", &["LES-001", "LES-002"])
        .success()
        .stdout("LES-001: rejected\nLES-002: approved\n");
    assert_eq!(
        entry_of(&project, "LES-001").status(),
        Some(Status::Rejected)
    );
    assert_eq!(
        entry_of(&project, "LES-002").status(),
        Some(Status::Approved)
    );
}

#[test]
fn json_output_gives_the_ids_of_each_verdict() {
    let project = Project::new();
    draft_lesson(&project);
    draft_lesson(&project);
    draft_lesson(&project);
    let output = project
        .command("anna", NOW)
        .env("DOSSIER_ANSWERS", "a|r|s")
        .args(["approve", "LES-001", "LES-002", "LES-003", "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let value: serde_json::Value =
        serde_json::from_slice(output.trim_ascii_end()).expect("one JSON line");
    assert_eq!(value["schema"], "dossier.approve.v1");
    assert_eq!(value["ok"], true);
    assert_eq!(
        value["data"],
        serde_json::json!({
            "approved": ["LES-001"],
            "rejected": ["LES-002"],
            "skipped": ["LES-003"],
        })
    );
}

#[test]
fn an_llm_cannot_approve_and_nothing_is_written() {
    let project = Project::new();
    draft_lesson(&project);
    approve_with(&project, "anna-claude", "a", &["LES-001"])
        .code(1)
        .stderr(contains(
            "error: only a person can approve; anna-claude is an LLM",
        ));
    project
        .command("anna-claude", NOW)
        .env("DOSSIER_ANSWERS", "a")
        .args(["approve", "LES-001", "--json"])
        .assert()
        .code(1)
        .stdout(contains("\"code\":\"NEEDS_PERSON\""));

    assert_eq!(entry_of(&project, "LES-001").status(), Some(Status::Draft));
    assert!(!project.root().join("DOSSIER/.state/approved.json").exists());
}

#[test]
fn without_a_terminal_a_person_is_refused_and_nothing_is_asked_or_written() {
    let project = Project::new();
    draft_lesson(&project);
    project
        .command("anna", NOW)
        .env_remove("DOSSIER_ANSWERS")
        .args(["approve", "LES-001", "--json"])
        .assert()
        .code(1)
        .stdout(contains("\"code\":\"NEEDS_TERMINAL\""));

    assert_eq!(entry_of(&project, "LES-001").status(), Some(Status::Draft));
}

#[test]
fn a_lesson_without_valid_for_is_skipped_with_a_warning_and_not_asked_about() {
    let project = Project::new();
    project
        .run(
            "anna-claude",
            &[
                "learn",
                STATEMENT,
                "--valid-for",
                "",
                "--evidence",
                EVIDENCE,
                "--confidence",
                "simulated",
            ],
        )
        .success();
    approve_with(&project, "anna", "a", &["LES-001"])
        .success()
        .stdout("LES-001: skipped\n")
        .stderr(contains(
            "warning: LES-001 cannot be approved yet: missing section Valid for",
        ));

    assert_eq!(entry_of(&project, "LES-001").status(), Some(Status::Draft));
    assert!(!project.root().join("DOSSIER/.state/approved.json").exists());
}

#[test]
fn an_entry_that_is_not_a_draft_is_skipped_with_a_warning() {
    let project = Project::new();
    project
        .run(
            "anna",
            &[
                "learn",
                STATEMENT,
                "--valid-for",
                VALID_FOR,
                "--evidence",
                EVIDENCE,
                "--confidence",
                "measured",
            ],
        )
        .success();
    let before = std::fs::read(project.entry_file("LES-001")).expect("the entry file is read");
    approve_with(&project, "anna", "a", &["LES-001"])
        .success()
        .stdout("LES-001: skipped\n")
        .stderr(contains(
            "warning: LES-001 is approved, not a draft; skipped",
        ));
    assert_eq!(
        std::fs::read(project.entry_file("LES-001")).expect("the entry file is read"),
        before
    );
}

#[test]
fn a_finding_draft_is_approved_and_gets_no_checksum() {
    let project = Project::new();
    project
        .run(
            "anna-claude",
            &["found", "An invented observation for a test.", "--later"],
        )
        .success()
        .stdout(contains("FND-001: added (draft"));
    approve_with(&project, "anna", "a", &["FND-001"])
        .success()
        .stdout("FND-001: approved\n");

    let entry = entry_of(&project, "FND-001");
    assert_eq!(entry.status(), Some(Status::Approved));
    assert_eq!(
        entry.field("approved").as_deref(),
        Some(format!("{NOW} by anna").as_str())
    );
    // Only lessons have a checksum (R-LIFE-6).
    assert!(!project.root().join("DOSSIER/.state/approved.json").exists());
}

#[test]
fn an_unknown_id_stops_the_command_before_any_question() {
    let project = Project::new();
    draft_lesson(&project);
    approve_with(&project, "anna", "a", &["LES-001", "LES-099"])
        .code(1)
        .stderr(contains("error: no entry LES-099"));

    assert_eq!(entry_of(&project, "LES-001").status(), Some(Status::Draft));
    assert!(!project.root().join("DOSSIER/.state/approved.json").exists());
}

#[test]
fn an_id_given_twice_is_handled_once() {
    let project = Project::new();
    draft_lesson(&project);
    approve_with(&project, "anna", "a", &["LES-001", "les-001"])
        .success()
        .stdout("LES-001: approved\n")
        .stderr(contains(
            "warning: LES-001 is given more than once; it is handled once",
        ));
    assert_eq!(
        entry_of(&project, "LES-001").status(),
        Some(Status::Approved)
    );
}

#[test]
fn approve_needs_at_least_one_id() {
    let project = Project::new();
    project.run("anna", &["approve"]).code(2);
}
