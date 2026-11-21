//! Runs the built program for `retire`: a person ends an approved entry with a reason and no
//! replacement. The entry is kept with status `retired`, `valid-until` and `retired-reason`. A
//! draft, a retired entry, an LLM and a blank reason are refused, and nothing is written in those
//! cases. Each test runs in its own temporary project folder, with the clock fixed by
//! `common::Project` (2026-11-20T10:00:00+01:00).

mod common;

use common::Project;
use dossier::approval::{self, Mark};
use dossier::entry::{Entry, Status};
use predicates::str::contains;

/// The fixed clock of `common::Project`, as it appears in the fields and history lines.
const NOW: &str = "2026-11-20T10:00:00+01:00";

/// Invented text for a decision, a lesson and the reasons used by the tests.
const DECISION: &str = "Run the invented test rig with two inlet guide vanes.";
const REASON: &str = "An invented reason: the rig was dismantled in the second quarter.";
const STATEMENT: &str = "Invented: the probe reads steady pressure at the outlet of every run.";
const VALID_FOR: &str = "An invented test rig with a tip Mach number above 1.1.";
const EVIDENCE: &str = "An invented comparison of two invented speedlines, section 1.";

/// The entry file with `id`, read and parsed.
fn entry_of(project: &Project, id: &str) -> Entry {
    let text = std::fs::read_to_string(project.entry_file(id)).expect("the entry file is read");
    Entry::parse(&text).expect("the entry parses")
}

/// The bytes of the entry file with `id`.
fn bytes_of(project: &Project, id: &str) -> Vec<u8> {
    std::fs::read(project.entry_file(id)).expect("the entry file is read")
}

/// The approved-lesson checksums of the project, as the JSON object in approved.json.
fn recorded_checksums(project: &Project) -> serde_json::Map<String, serde_json::Value> {
    let text = project.read("DOSSIER/.state/approved.json");
    let value: serde_json::Value =
        serde_json::from_str(&text).expect("approved.json is a JSON object");
    value
        .as_object()
        .cloned()
        .expect("approved.json is a JSON object")
}

/// The state folder of the dossier.
fn state_of(project: &Project) -> std::path::PathBuf {
    project.root().join("DOSSIER").join(".state")
}

/// Anna records an approved decision: DEC-001.
fn anna_decides(project: &Project) {
    project.stdout(
        "anna",
        &["decide", DECISION, "--reason", "Because the test says so."],
    );
}

/// Anna records an approved lesson: LES-001, with its checksum recorded.
fn anna_learns(project: &Project) {
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

/// Anna-claude drafts a lesson: LES-001, waiting for approval.
fn claude_drafts_a_lesson(project: &Project) {
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

#[test]
fn anna_retires_an_approved_decision_with_a_reason_and_it_is_closed_with_a_date() {
    let project = Project::new();
    anna_decides(&project);
    let path_before = project.entry_file("DEC-001");

    project
        .run("anna", &["retire", "DEC-001", "--reason", REASON])
        .success()
        .stdout("DEC-001: retired\n");

    // The file stays where it was, with its text and its author; the status and the closing
    // fields are changed.
    assert_eq!(project.entry_file("DEC-001"), path_before);
    let entry = entry_of(&project, "DEC-001");
    assert_eq!(entry.status(), Some(Status::Retired));
    assert_eq!(entry.field("retired-reason").as_deref(), Some(REASON));
    assert_eq!(entry.field("valid-until").as_deref(), Some(NOW));
    assert_eq!(entry.field("valid-from").as_deref(), Some(NOW));
    assert_eq!(entry.field("author").as_deref(), Some("anna"));
    assert!(entry.section("Decision").is_some());
    assert!(entry.section("Reason").is_some());
    assert_eq!(
        entry.history().last().map(String::as_str),
        Some(format!("{NOW} anna: retired — {REASON}").as_str())
    );
    // valid-until is written before retired-reason, as format.md §5 lists them.
    let text = bytes_of(&project, "DEC-001");
    let text = String::from_utf8(text).expect("the entry is UTF-8");
    assert!(text.contains(&format!(
        "- valid-until: {NOW}\n- retired-reason: {REASON}\n"
    )));
    // The change is journalled with its reason, and the file as it was is kept as a snapshot.
    let journal = project.read("DOSSIER/.state/journal.jsonl");
    assert!(journal.contains("\"event\":\"retired\""));
    assert!(journal.contains(&format!("\"detail\":\"{REASON}\"")));
    assert!(
        state_of(&project)
            .join("history")
            .join("DEC-001")
            .join("000001.md")
            .exists()
    );
}

#[test]
fn retiring_an_approved_lesson_forgets_its_recorded_checksum() {
    let project = Project::new();
    anna_learns(&project);
    let state = state_of(&project);
    let before = bytes_of(&project, "LES-001");
    assert!(matches!(
        approval::mark(&state, "LES-001", &before),
        Mark::Unchanged
    ));

    project
        .run("anna", &["retire", "LES-001", "--reason", REASON])
        .success()
        .stdout("LES-001: retired\n");

    // A retired lesson is no longer approved, so it has no recorded checksum.
    assert!(!recorded_checksums(&project).contains_key("LES-001"));
    let after = bytes_of(&project, "LES-001");
    assert!(matches!(
        approval::mark(&state, "LES-001", &after),
        Mark::NotRecorded
    ));
    assert_eq!(
        entry_of(&project, "LES-001").status(),
        Some(Status::Retired)
    );
}

#[test]
fn retiring_a_draft_is_refused_and_the_file_stays_as_it_was() {
    let project = Project::new();
    claude_drafts_a_lesson(&project);
    let before = bytes_of(&project, "LES-001");

    project
        .run("anna", &["retire", "LES-001", "--reason", REASON])
        .code(1)
        .stderr(contains(
            "error: only an approved entry can be retired; LES-001 is draft",
        ));

    assert_eq!(bytes_of(&project, "LES-001"), before);
    assert_eq!(entry_of(&project, "LES-001").status(), Some(Status::Draft));
}

#[test]
fn a_retired_entry_cannot_be_retired_again_and_the_file_stays_as_it_was() {
    let project = Project::new();
    anna_decides(&project);
    project
        .run("anna", &["retire", "DEC-001", "--reason", REASON])
        .success()
        .stdout("DEC-001: retired\n");
    let before = bytes_of(&project, "DEC-001");

    project
        .run(
            "anna",
            &["retire", "DEC-001", "--reason", "A second reason."],
        )
        .code(1)
        .stderr(contains(
            "error: only an approved entry can be retired; DEC-001 is retired",
        ));

    assert_eq!(bytes_of(&project, "DEC-001"), before);
}

#[test]
fn anna_claude_cannot_retire_and_nothing_is_written() {
    let project = Project::new();
    anna_decides(&project);
    let before = bytes_of(&project, "DEC-001");

    project
        .run("anna-claude", &["retire", "DEC-001", "--reason", REASON])
        .code(1)
        .stderr(contains(
            "error: retire is for a person; anna-claude is an LLM",
        ));
    project
        .command("anna-claude", NOW)
        .args(["retire", "DEC-001", "--reason", REASON, "--json"])
        .assert()
        .code(1)
        .stdout(contains("\"code\":\"NEEDS_PERSON\""));

    assert_eq!(bytes_of(&project, "DEC-001"), before);
    assert_eq!(
        entry_of(&project, "DEC-001").status(),
        Some(Status::Approved)
    );
}

#[test]
fn a_blank_reason_is_refused_and_nothing_is_written() {
    let project = Project::new();
    anna_decides(&project);
    let before = bytes_of(&project, "DEC-001");

    project
        .run("anna", &["retire", "DEC-001", "--reason", "   "])
        .code(1)
        .stderr(contains("error: the reason must not be empty"));

    assert_eq!(bytes_of(&project, "DEC-001"), before);
}

#[test]
fn retire_needs_a_reason() {
    let project = Project::new();
    anna_decides(&project);
    project.run("anna", &["retire", "DEC-001"]).code(2);
}

#[test]
fn an_unknown_id_is_not_found_and_nothing_is_written() {
    let project = Project::new();
    anna_decides(&project);
    let before = bytes_of(&project, "DEC-001");

    project
        .run("anna", &["retire", "DEC-099", "--reason", REASON])
        .code(1)
        .stderr(contains("error: no entry DEC-099"));

    assert_eq!(bytes_of(&project, "DEC-001"), before);
}

#[test]
fn a_dry_run_writes_nothing_and_keeps_the_recorded_checksum() {
    let project = Project::new();
    anna_learns(&project);
    let before = bytes_of(&project, "LES-001");

    project
        .command("anna", NOW)
        .args(["retire", "LES-001", "--reason", REASON, "--dry-run"])
        .assert()
        .success()
        .stdout(contains("LES-001: retired"));

    assert_eq!(bytes_of(&project, "LES-001"), before);
    assert!(recorded_checksums(&project).contains_key("LES-001"));
    assert!(matches!(
        approval::mark(&state_of(&project), "LES-001", &before),
        Mark::Unchanged
    ));
}

#[test]
fn json_output_gives_the_id_and_the_new_status() {
    let project = Project::new();
    anna_decides(&project);
    let output = project
        .command("anna", NOW)
        .args(["retire", "dec-001", "--reason", REASON, "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let value: serde_json::Value =
        serde_json::from_slice(output.trim_ascii_end()).expect("one JSON line");
    assert_eq!(value["schema"], "dossier.retire.v1");
    assert_eq!(value["ok"], true);
    assert_eq!(
        value["data"],
        serde_json::json!({"id": "DEC-001", "status": "retired"})
    );
}
