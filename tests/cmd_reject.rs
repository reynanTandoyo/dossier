//! Runs the built program for `reject`: a person turns a draft down. The entry is kept with status
//! `rejected` and the reason; an approved entry and an LLM are refused, and nothing is written in
//! those cases. Each test runs in its own project folder, with the clock fixed by `common::Project`.

mod common;

use common::Project;
use dossier::entry::{Entry, Status};
use predicates::str::contains;

/// The fixed clock of `common::Project`.
const NOW: &str = "2026-11-20T10:00:00+01:00";

/// Invented text for a lesson, and the reason given for rejecting it.
const STATEMENT: &str = "Steady runs with a mixing-plane interface overpredict the stall margin.";
const VALID_FOR: &str = "An invented test rig with a tip Mach number above 1.1.";
const EVIDENCE: &str = "An invented comparison of two speedlines, section 1.";
const REASON: &str = "An invented reason: the same proposal was already in the dossier.";

/// Runs `learn` as anna-claude with a complete lesson: a draft, with the ID LES-001.
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
        .success()
        .stdout(contains("LES-001: added (draft"));
}

/// Runs `learn` as anna with a complete lesson: a person's entry, approved at once, LES-001.
fn approved_lesson(project: &Project) {
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
        .success()
        .stdout(contains("LES-001: added (approved"));
}

/// The entry file with `id`, read and parsed.
fn entry_of(project: &Project, id: &str) -> Entry {
    let text = std::fs::read_to_string(project.entry_file(id)).expect("the entry file is read");
    Entry::parse(&text).expect("the entry parses")
}

/// The bytes of the entry file with `id`.
fn bytes_of(project: &Project, id: &str) -> Vec<u8> {
    std::fs::read(project.entry_file(id)).expect("the entry file is read")
}

#[test]
fn anna_rejects_a_draft_and_the_entry_is_kept_with_status_rejected_and_the_reason() {
    let project = Project::new();
    draft_lesson(&project);
    let path_before = project.entry_file("LES-001");
    project
        .run("anna", &["reject", "LES-001", "--reason", REASON])
        .success()
        .stdout("LES-001: rejected\n");

    // The entry is kept where it was, with its statement and its author.
    assert_eq!(project.entry_file("LES-001"), path_before);
    let entry = entry_of(&project, "LES-001");
    assert_eq!(entry.status(), Some(Status::Rejected));
    assert_eq!(entry.field("rejected-reason").as_deref(), Some(REASON));
    assert!(entry.section("Statement").is_some());
    assert_eq!(entry.field("author").as_deref(), Some("anna"));
    assert_eq!(entry.field("via").as_deref(), Some("anna-claude"));
    // A rejected draft was never approved.
    assert_eq!(entry.field("approved"), None);
    assert_eq!(entry.field("valid-from"), None);
    assert_eq!(
        entry.history().last().map(String::as_str),
        Some(format!("{NOW} anna: rejected — {REASON}").as_str())
    );
    // The change is journalled, and the draft as it was is kept as a snapshot.
    assert!(
        project
            .read("DOSSIER/.state/journal.jsonl")
            .contains("\"event\":\"rejected\"")
    );
    assert!(
        project
            .root()
            .join("DOSSIER/.state/history/LES-001/000001.md")
            .exists()
    );
}

#[test]
fn a_reject_without_a_reason_keeps_no_reason_field_and_its_history_line_has_no_detail() {
    let project = Project::new();
    draft_lesson(&project);
    project
        .run("anna", &["reject", "LES-001"])
        .success()
        .stdout("LES-001: rejected\n");

    let entry = entry_of(&project, "LES-001");
    assert_eq!(entry.status(), Some(Status::Rejected));
    assert_eq!(entry.field("rejected-reason"), None);
    assert_eq!(
        entry.history().last().map(String::as_str),
        Some(format!("{NOW} anna: rejected").as_str())
    );
}

#[test]
fn an_approved_entry_cannot_be_rejected_and_its_file_stays_as_it_was() {
    let project = Project::new();
    approved_lesson(&project);
    let before = bytes_of(&project, "LES-001");
    project
        .run("anna", &["reject", "LES-001", "--reason", REASON])
        .code(1)
        .stderr(contains(
            "error: only a draft can be rejected; LES-001 is approved",
        ));

    assert_eq!(bytes_of(&project, "LES-001"), before);
    assert_eq!(
        entry_of(&project, "LES-001").status(),
        Some(Status::Approved)
    );
}

#[test]
fn a_rejected_entry_cannot_be_rejected_again() {
    let project = Project::new();
    draft_lesson(&project);
    project
        .run("anna", &["reject", "LES-001"])
        .success()
        .stdout("LES-001: rejected\n");
    let before = bytes_of(&project, "LES-001");
    project
        .run("anna", &["reject", "LES-001", "--reason", REASON])
        .code(1)
        .stderr(contains(
            "error: only a draft can be rejected; LES-001 is rejected",
        ));

    assert_eq!(bytes_of(&project, "LES-001"), before);
}

#[test]
fn an_llm_cannot_reject_and_nothing_is_written() {
    let project = Project::new();
    draft_lesson(&project);
    let before = bytes_of(&project, "LES-001");
    project
        .run("anna-claude", &["reject", "LES-001", "--reason", REASON])
        .code(1)
        .stderr(contains(
            "error: reject is for a person; anna-claude is an LLM",
        ));
    project
        .command("anna-claude", NOW)
        .args(["reject", "LES-001", "--json"])
        .assert()
        .code(1)
        .stdout(contains("\"code\":\"NEEDS_PERSON\""));

    assert_eq!(bytes_of(&project, "LES-001"), before);
    assert_eq!(entry_of(&project, "LES-001").status(), Some(Status::Draft));
}

#[test]
fn json_output_gives_the_id_and_the_new_status() {
    let project = Project::new();
    draft_lesson(&project);
    let output = project
        .command("anna", NOW)
        .args(["reject", "LES-001", "--reason", REASON, "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let value: serde_json::Value =
        serde_json::from_slice(output.trim_ascii_end()).expect("one JSON line");
    assert_eq!(value["schema"], "dossier.reject.v1");
    assert_eq!(value["ok"], true);
    assert_eq!(
        value["data"],
        serde_json::json!({"id": "LES-001", "status": "rejected"})
    );
}

#[test]
fn an_unknown_id_is_not_found_and_nothing_is_written() {
    let project = Project::new();
    draft_lesson(&project);
    let before = bytes_of(&project, "LES-001");
    project
        .run("anna", &["reject", "LES-099"])
        .code(1)
        .stderr(contains("error: no entry LES-099"));

    assert_eq!(bytes_of(&project, "LES-001"), before);
}

#[test]
fn reject_needs_an_id() {
    let project = Project::new();
    project.run("anna", &["reject"]).code(2);
}
