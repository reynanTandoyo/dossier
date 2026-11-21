//! Runs the built program for `reopen`: an approved or rejected entry goes back to draft, and a
//! retired entry is in force again as approved, which only a person can do. A draft and a
//! superseded entry cannot be reopened. Refusals leave the file, its snapshots and the journal as
//! they were. Each test runs in its own temporary project folder (common::Project), with the
//! clock fixed at 2026-11-20T10:00:00+01:00.

mod common;

use common::Project;
use dossier::approval::{self, Mark};
use dossier::entry::{Entry, Status};
use predicates::str::contains;

/// The fixed clock of `common::Project`, as it appears in the fields and history lines.
const NOW: &str = "2026-11-20T10:00:00+01:00";

/// Invented text for a lesson, a finding, decisions and the reasons used by the tests.
const STATEMENT: &str = "Invented: the probe reads steady pressure at the outlet of every run.";
const VALID_FOR: &str = "An invented test rig with a tip Mach number above 1.1.";
const EVIDENCE: &str = "An invented comparison of two invented speedlines, section 1.";
const OBSERVATION: &str = "Invented: the inlet profile stayed uniform in all three runs.";
const DECISION: &str = "Run the invented test rig with two inlet guide vanes.";
const OTHER_DECISION: &str = "Run the invented test rig with three inlet guide vanes.";
const REASON: &str = "An invented reason: the rig was dismantled in the second quarter.";

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
    project
        .run(
            "anna",
            &["decide", DECISION, "--reason", "Because the test says so."],
        )
        .success()
        .stdout(contains("DEC-001: added (approved"));
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

/// Anna records an approved finding: FND-001.
fn anna_finds(project: &Project) {
    project
        .run("anna", &["found", OBSERVATION, "--evidence", EVIDENCE])
        .success()
        .stdout("FND-001: added (approved)\n");
}

#[test]
fn anna_reopens_a_rejected_lesson_and_it_is_a_draft_again() {
    let project = Project::new();
    claude_drafts_a_lesson(&project);
    project
        .run("anna", &["reject", "LES-001", "--reason", REASON])
        .success()
        .stdout("LES-001: rejected\n");
    let path_before = project.entry_file("LES-001");

    project
        .run("anna", &["reopen", "LES-001"])
        .success()
        .stdout("LES-001: reopened (draft)\n");

    // The file stays where it was. The rejection fields and the approval fields are gone, and
    // the statement and the authors are kept.
    assert_eq!(project.entry_file("LES-001"), path_before);
    let entry = entry_of(&project, "LES-001");
    assert_eq!(entry.status(), Some(Status::Draft));
    assert_eq!(entry.field("rejected-reason"), None);
    assert_eq!(entry.field("approved"), None);
    assert_eq!(entry.field("valid-from"), None);
    assert_eq!(entry.field("author").as_deref(), Some("anna"));
    assert_eq!(entry.field("via").as_deref(), Some("anna-claude"));
    assert!(entry.section("Statement").is_some());
    assert_eq!(
        entry.history().last().map(String::as_str),
        Some(format!("{NOW} anna: reopened — back to draft").as_str())
    );
    // The change is journalled, and the rejected text is kept as snapshot 2 (snapshot 1 is the
    // draft as it was before the rejection).
    let journal = project.read("DOSSIER/.state/journal.jsonl");
    assert!(journal.contains("\"event\":\"reopened\""));
    assert!(journal.contains("\"detail\":\"back to draft\""));
    let snapshot = std::fs::read_to_string(
        state_of(&project)
            .join("history")
            .join("LES-001")
            .join("000002.md"),
    )
    .expect("snapshot 2 is kept");
    assert!(snapshot.contains("- status: rejected"), "{snapshot}");
}

#[test]
fn an_approved_lesson_goes_back_to_draft_and_its_checksum_is_forgotten() {
    let project = Project::new();
    anna_learns(&project);
    let state = state_of(&project);
    let before = bytes_of(&project, "LES-001");
    assert!(matches!(
        approval::mark(&state, "LES-001", &before),
        Mark::Unchanged
    ));

    project
        .run("anna", &["reopen", "LES-001"])
        .success()
        .stdout("LES-001: reopened (draft)\n");

    assert!(!recorded_checksums(&project).contains_key("LES-001"));
    let after = bytes_of(&project, "LES-001");
    assert!(matches!(
        approval::mark(&state, "LES-001", &after),
        Mark::NotRecorded
    ));
    let entry = entry_of(&project, "LES-001");
    assert_eq!(entry.status(), Some(Status::Draft));
    assert_eq!(entry.field("approved"), None);
    assert_eq!(entry.field("valid-from"), None);
}

#[test]
fn an_approved_decision_goes_back_to_draft_and_the_writer_is_named_with_the_llm() {
    let project = Project::new();
    anna_decides(&project);

    project
        .run("anna-claude", &["reopen", "DEC-001"])
        .success()
        .stdout("DEC-001: reopened (draft)\n");

    let entry = entry_of(&project, "DEC-001");
    assert_eq!(entry.status(), Some(Status::Draft));
    assert_eq!(entry.field("approved"), None);
    assert_eq!(entry.field("valid-from"), None);
    assert_eq!(
        entry.history().last().map(String::as_str),
        Some(format!("{NOW} anna via anna-claude: reopened — back to draft").as_str())
    );
}

#[test]
fn a_retired_finding_is_approved_again_when_anna_runs_it_and_anna_claude_is_refused() {
    let project = Project::new();
    anna_finds(&project);
    project
        .run("anna", &["retire", "FND-001", "--reason", REASON])
        .success()
        .stdout("FND-001: retired\n");
    let before = bytes_of(&project, "FND-001");

    project
        .run("anna-claude", &["reopen", "FND-001"])
        .code(1)
        .stderr(contains(
            "error: reopening a retired entry is for a person; anna-claude is an LLM",
        ));
    project
        .command("anna-claude", NOW)
        .args(["reopen", "FND-001", "--json"])
        .assert()
        .code(1)
        .stdout(contains("\"code\":\"NEEDS_PERSON\""));
    assert_eq!(bytes_of(&project, "FND-001"), before);
    assert_eq!(
        entry_of(&project, "FND-001").status(),
        Some(Status::Retired)
    );

    project
        .run("anna", &["reopen", "FND-001"])
        .success()
        .stdout("FND-001: reopened (approved)\n");

    // The retired finding is in force again: the closing fields are gone, the approval stays.
    let entry = entry_of(&project, "FND-001");
    assert_eq!(entry.status(), Some(Status::Approved));
    assert_eq!(entry.field("valid-until"), None);
    assert_eq!(entry.field("retired-reason"), None);
    assert_eq!(entry.field("valid-from").as_deref(), Some(NOW));
    assert_eq!(
        entry.history().last().map(String::as_str),
        Some(format!("{NOW} anna: reopened — in force again").as_str())
    );
}

#[test]
fn a_retired_lesson_is_approved_again_and_its_checksum_is_recorded_again() {
    let project = Project::new();
    anna_learns(&project);
    project
        .run("anna", &["retire", "LES-001", "--reason", REASON])
        .success()
        .stdout("LES-001: retired\n");
    assert!(!recorded_checksums(&project).contains_key("LES-001"));

    project
        .run("anna", &["reopen", "LES-001"])
        .success()
        .stdout("LES-001: reopened (approved)\n");

    assert!(recorded_checksums(&project).contains_key("LES-001"));
    let bytes = bytes_of(&project, "LES-001");
    assert!(matches!(
        approval::mark(&state_of(&project), "LES-001", &bytes),
        Mark::Unchanged
    ));
    assert_eq!(
        entry_of(&project, "LES-001").status(),
        Some(Status::Approved)
    );
}

#[test]
fn a_draft_cannot_be_reopened_and_the_file_stays_as_it_was() {
    let project = Project::new();
    claude_drafts_a_lesson(&project);
    let before = bytes_of(&project, "LES-001");

    project
        .run("anna", &["reopen", "LES-001"])
        .code(1)
        .stderr(contains("error: a draft entry cannot be reopened"));
    project
        .command("anna", NOW)
        .args(["reopen", "LES-001", "--json"])
        .assert()
        .code(1)
        .stdout(contains("\"code\":\"REFUSED\""));

    assert_eq!(bytes_of(&project, "LES-001"), before);
    assert!(
        !project
            .read("DOSSIER/.state/journal.jsonl")
            .contains("\"event\":\"reopened\"")
    );
}

#[test]
fn a_superseded_decision_cannot_be_reopened_and_the_file_stays_as_it_was() {
    let project = Project::new();
    anna_decides(&project);
    project
        .run(
            "anna",
            &[
                "decide",
                OTHER_DECISION,
                "--reason",
                "A second invented reason.",
            ],
        )
        .success()
        .stdout(contains("DEC-002: added (approved"));
    project
        .run("anna", &["supersede", "DEC-001", "--by", "DEC-002"])
        .success();
    let before = bytes_of(&project, "DEC-001");

    project
        .run("anna", &["reopen", "DEC-001"])
        .code(1)
        .stderr(contains("error: a superseded entry cannot be reopened"));

    assert_eq!(bytes_of(&project, "DEC-001"), before);
}

#[test]
fn a_dry_run_writes_nothing_and_keeps_the_recorded_checksum() {
    let project = Project::new();
    anna_learns(&project);
    let before = bytes_of(&project, "LES-001");

    project
        .command("anna", NOW)
        .args(["reopen", "LES-001", "--dry-run"])
        .assert()
        .success()
        .stdout("LES-001: reopened (draft)\n")
        .stderr(contains("dry run: would write DOSSIER/lessons/LES-001-"));

    assert_eq!(bytes_of(&project, "LES-001"), before);
    assert!(recorded_checksums(&project).contains_key("LES-001"));
    assert!(matches!(
        approval::mark(&state_of(&project), "LES-001", &before),
        Mark::Unchanged
    ));
    assert!(
        !project
            .read("DOSSIER/.state/journal.jsonl")
            .contains("\"event\":\"reopened\"")
    );
}

#[test]
fn an_unknown_id_is_not_found_and_nothing_is_written() {
    let project = Project::new();
    anna_decides(&project);
    let before = bytes_of(&project, "DEC-001");

    project
        .run("anna", &["reopen", "DEC-099"])
        .code(1)
        .stderr(contains("error: no entry DEC-099"));

    assert_eq!(bytes_of(&project, "DEC-001"), before);
}

#[test]
fn reopen_needs_an_id() {
    let project = Project::new();
    anna_decides(&project);
    project.run("anna", &["reopen"]).code(2);
}

#[test]
fn json_output_gives_the_id_and_the_new_status() {
    let project = Project::new();
    anna_decides(&project);
    let output = project
        .command("anna", NOW)
        .args(["reopen", "dec-001", "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let value: serde_json::Value =
        serde_json::from_slice(output.trim_ascii_end()).expect("one JSON line");
    assert_eq!(value["schema"], "dossier.reopen.v1");
    assert_eq!(value["ok"], true);
    assert_eq!(
        value["data"],
        serde_json::json!({"id": "DEC-001", "status": "draft"})
    );
}
