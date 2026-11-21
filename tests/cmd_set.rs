//! Runs the built program for `set`: only the named fields and sections change, with one history
//! line added; an LLM's change to an approved lesson takes it back to draft, and a person's change
//! to one is recorded as its new approved text. Each test runs in its own temporary project folder
//! with the clock fixed by `common::Project` (2026-11-20T10:00:00+01:00).

mod common;

use common::Project;
use dossier::approval::{self, Mark};
use dossier::entry::{Entry, Status};
use predicates::str::contains;

/// The fixed clock of `common::Project`, as it appears in history lines.
const WHEN: &str = "2026-11-20T10:00:00+01:00";

/// Invented text for a finding and a lesson used by the tests.
const OBSERVATION: &str = "The mass flow still oscillated by 0.4 % with half the time step.";
const EVIDENCE: &str = "Runs 12 and 13, monitor plots in results.";
const STATEMENT: &str = "Steady mixing-plane runs overpredict the stall margin.";
const VALID_FOR: &str = "An invented test rig with a tip Mach number above 1.1.";
const LESSON_EVIDENCE: &str = "An invented comparison of two speedlines, section 1.";

/// The text of the entry file with `id`.
fn text_of(project: &Project, id: &str) -> String {
    std::fs::read_to_string(project.entry_file(id)).expect("the entry file can be read")
}

/// The entry with `id`, parsed.
fn entry_of(project: &Project, id: &str) -> Entry {
    Entry::parse(&text_of(project, id)).expect("the file is an entry")
}

/// A finding written by anna, a person: approved at once. Its machine is axial compressor.
fn anna_finds(project: &Project) {
    project.stdout(
        "anna",
        &[
            "found",
            OBSERVATION,
            "--evidence",
            EVIDENCE,
            "--machine",
            "axial compressor",
        ],
    );
}

/// A lesson written by anna, a person: approved at once.
fn anna_learns(project: &Project) {
    project.stdout(
        "anna",
        &[
            "learn",
            STATEMENT,
            "--valid-for",
            VALID_FOR,
            "--evidence",
            LESSON_EVIDENCE,
            "--confidence",
            "simulated",
        ],
    );
}

/// Whether the state folder of the dossier holds a snapshot with this name.
fn snapshot_exists(project: &Project, id: &str, n: &str) -> bool {
    project
        .root()
        .join("DOSSIER")
        .join(".state")
        .join("history")
        .join(id)
        .join(format!("{n}.md"))
        .exists()
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

#[test]
fn anna_changes_a_field_of_a_finding_and_only_that_line_and_one_history_line_differ() {
    let project = Project::new();
    anna_finds(&project);
    let before = text_of(&project, "FND-001");

    project
        .run("anna", &["set", "FND-001", "--field", "machine=fan"])
        .success()
        .stdout("FND-001: changed\n");

    let after = text_of(&project, "FND-001");
    let history = format!("- history: {WHEN} anna: changed — field machine\n");
    assert_eq!(after.matches(&history).count(), 1);
    // Taking away the one history line and the one field line gives back the old text.
    assert_eq!(
        after.replacen(&history, "", 1),
        before.replacen("- machine: axial compressor\n", "- machine: fan\n", 1)
    );
    assert_eq!(
        entry_of(&project, "FND-001").field("machine").as_deref(),
        Some("fan")
    );
    assert_eq!(
        entry_of(&project, "FND-001").status(),
        Some(Status::Approved)
    );
}

#[test]
fn the_text_before_the_change_is_kept_as_snapshot_one() {
    let project = Project::new();
    anna_finds(&project);
    let before = text_of(&project, "FND-001");

    project.stdout("anna", &["set", "FND-001", "--field", "machine=fan"]);

    assert!(snapshot_exists(&project, "FND-001", "000001"));
    let snapshot = project.read("DOSSIER/.state/history/FND-001/000001.md");
    assert_eq!(snapshot, before);
}

#[test]
fn a_section_changes_and_the_other_sections_keep_their_bytes() {
    let project = Project::new();
    anna_finds(&project);
    let before = text_of(&project, "FND-001");

    project
        .run(
            "anna",
            &["set", "FND-001", "--section", "Evidence=Runs 14 to 16."],
        )
        .success()
        .stdout("FND-001: changed\n");

    let after = text_of(&project, "FND-001");
    let history = format!("- history: {WHEN} anna: changed — section Evidence\n");
    assert_eq!(after.matches(&history).count(), 1);
    // Taking away the one history line and the new body gives back the old text.
    assert_eq!(
        after
            .replacen(&history, "", 1)
            .replacen("Runs 14 to 16.", EVIDENCE, 1),
        before
    );
}

#[test]
fn a_field_given_an_empty_value_is_removed() {
    let project = Project::new();
    anna_finds(&project);

    project
        .run("anna", &["set", "FND-001", "--field", "machine="])
        .success()
        .stdout("FND-001: changed\n");

    let entry = entry_of(&project, "FND-001");
    assert_eq!(entry.field("machine"), None);
    assert!(text_of(&project, "FND-001").contains(&format!(
        "- history: {WHEN} anna: changed — field machine\n"
    )));
}

#[test]
fn naming_the_value_an_entry_already_has_changes_nothing() {
    let project = Project::new();
    anna_finds(&project);
    let before = text_of(&project, "FND-001");

    project
        .run(
            "anna",
            &["set", "FND-001", "--field", "machine=axial compressor"],
        )
        .success()
        .stdout("FND-001: no change\n");

    assert_eq!(text_of(&project, "FND-001"), before);
    assert!(!snapshot_exists(&project, "FND-001", "000001"));
}

#[test]
fn anna_claude_changing_an_approved_lesson_makes_it_a_draft() {
    let project = Project::new();
    anna_learns(&project);
    assert!(recorded_checksums(&project).contains_key("LES-001"));

    project
        .run(
            "anna-claude",
            &[
                "set",
                "LES-001",
                "--section",
                "Evidence=An invented comparison of two speedlines, section 2.",
            ],
        )
        .success()
        .stdout("LES-001: changed (back to draft, waiting for approval)\n");

    let entry = entry_of(&project, "LES-001");
    assert_eq!(entry.status(), Some(Status::Draft));
    assert_eq!(entry.field("approved"), None);
    assert_eq!(entry.field("valid-from"), None);
    // The lesson was written by anna, so there is no via field; the history line names the LLM.
    assert_eq!(entry.field("author").as_deref(), Some("anna"));
    assert_eq!(entry.field("via"), None);
    assert_eq!(
        entry.section("Evidence").as_deref(),
        Some("An invented comparison of two speedlines, section 2.")
    );
    let text = text_of(&project, "LES-001");
    assert!(text.contains(&format!(
        "- history: {WHEN} anna via anna-claude: changed — section Evidence, back to draft\n"
    )));
    // The approved text stays in the snapshot, and the checksum of the approval is forgotten.
    assert!(snapshot_exists(&project, "LES-001", "000001"));
    assert!(!recorded_checksums(&project).contains_key("LES-001"));
}

#[test]
fn an_llm_changing_a_draft_lesson_changes_it_and_it_stays_a_draft() {
    let project = Project::new();
    project
        .run(
            "anna-claude",
            &[
                "learn",
                STATEMENT,
                "--valid-for",
                VALID_FOR,
                "--evidence",
                LESSON_EVIDENCE,
                "--confidence",
                "guess",
            ],
        )
        .success()
        .stdout("LES-001: added (draft, waiting for approval)\n");

    project
        .run("anna-claude", &["set", "LES-001", "--field", "machine=fan"])
        .success()
        .stdout("LES-001: changed\n");

    assert_eq!(entry_of(&project, "LES-001").status(), Some(Status::Draft));
}

#[test]
fn a_persons_change_to_an_approved_lesson_stays_approved_and_is_recorded() {
    let project = Project::new();
    anna_learns(&project);

    project
        .run("anna", &["set", "LES-001", "--field", "machine=fan"])
        .success()
        .stdout("LES-001: changed\n");

    assert_eq!(
        entry_of(&project, "LES-001").status(),
        Some(Status::Approved)
    );
    let text = text_of(&project, "LES-001");
    let state = project.root().join("DOSSIER").join(".state");
    assert!(matches!(
        approval::mark(&state, "LES-001", text.as_bytes()),
        Mark::Unchanged
    ));
}

#[test]
fn a_field_the_tool_writes_is_refused() {
    let project = Project::new();
    anna_finds(&project);
    let before = text_of(&project, "FND-001");

    for key in ["history=x", "status=approved", "Approved=2026-11-20"] {
        project
            .run("anna", &["set", "FND-001", "--field", key])
            .failure()
            .code(1)
            .stderr(contains("this field is written by the tool"));
    }
    assert_eq!(text_of(&project, "FND-001"), before);
}

#[test]
fn a_set_with_nothing_to_change_is_a_usage_error() {
    let project = Project::new();
    anna_finds(&project);

    project
        .run("anna", &["set", "FND-001"])
        .failure()
        .code(2)
        .stderr(contains("nothing to change"));
}

#[test]
fn a_section_line_that_starts_a_section_is_refused() {
    let project = Project::new();
    anna_finds(&project);
    let before = text_of(&project, "FND-001");

    project
        .run(
            "anna",
            &["set", "FND-001", "--section", "Evidence=first\n## Other"],
        )
        .failure()
        .code(1)
        .stderr(contains("cannot start with ##"));
    assert_eq!(text_of(&project, "FND-001"), before);
}

#[test]
fn an_unknown_id_is_not_found() {
    let project = Project::new();
    anna_finds(&project);

    project
        .run("anna", &["set", "FND-099", "--field", "machine=fan"])
        .failure()
        .code(1)
        .stderr(contains("no entry FND-099"));
    assert!(!snapshot_exists(&project, "FND-001", "000001"));
}

#[test]
fn a_dry_run_by_an_llm_changes_nothing() {
    let project = Project::new();
    anna_learns(&project);
    let before = text_of(&project, "LES-001");
    let checksums = recorded_checksums(&project);

    project
        .run(
            "anna-claude",
            &[
                "--dry-run",
                "set",
                "LES-001",
                "--section",
                "Evidence=An invented comparison, section 3.",
            ],
        )
        .success();

    assert_eq!(text_of(&project, "LES-001"), before);
    assert_eq!(recorded_checksums(&project), checksums);
    assert!(!snapshot_exists(&project, "LES-001", "000001"));
}
