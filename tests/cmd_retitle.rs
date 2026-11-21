//! Runs the built program for `retitle`: the title changes, the file moves to the slug of the new
//! title, the ID stays, and a history line says what changed. An LLM's change to an approved lesson
//! takes it back to draft, as `set` does. Each test runs in its own temporary project folder with
//! the clock fixed by `common::Project` (2026-11-20T10:00:00+01:00).

mod common;

use common::Project;
use dossier::approval::{self, Mark};
use dossier::entry::{Entry, Status};
use predicates::str::contains;

/// The fixed clock of `common::Project`, as it appears in history lines.
const WHEN: &str = "2026-11-20T10:00:00+01:00";

/// Invented text for a decision and a lesson used by the tests.
const DECISION: &str = "Run the invented test rig with two inlet guide vanes.";
const REASON: &str = "An invented reason: the first rig build had one vane.";
const STATEMENT: &str = "Invented: the probe reads steady pressure at the outlet of every run.";
const VALID_FOR: &str = "An invented test rig with a tip Mach number above 1.1.";
const LESSON_EVIDENCE: &str = "An invented comparison of two invented speedlines, section 1.";

/// The entry with `id`, parsed from its file.
fn entry_of(project: &Project, id: &str) -> Entry {
    let text = std::fs::read_to_string(project.entry_file(id)).expect("the entry file can be read");
    Entry::parse(&text).expect("the file is an entry")
}

/// The number of files in the decisions folder.
fn decision_files(project: &Project) -> usize {
    std::fs::read_dir(project.root().join("DOSSIER").join("decisions"))
        .expect("the decisions folder")
        .count()
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

/// A decision written by anna, a person: approved at once, with the title given.
fn anna_decides(project: &Project, title: &str) {
    project.stdout(
        "anna",
        &["decide", DECISION, "--reason", REASON, "--title", title],
    );
}

/// A lesson written by anna, a person: approved at once, with the title given.
fn anna_learns(project: &Project, title: &str) {
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
            "--title",
            title,
        ],
    );
}

#[test]
fn anna_retitles_a_decision_the_old_file_goes_the_id_stays_and_the_history_has_a_changed_line() {
    let project = Project::new();
    anna_decides(&project, "Two inlet guide vanes");
    let old = project.entry_file("DEC-001");
    assert!(old.ends_with("DEC-001-two-inlet-guide-vanes.md"));

    project
        .run("anna", &["retitle", "DEC-001", "Three inlet guide vanes"])
        .success()
        .stdout("DEC-001: retitled\n");

    assert!(!old.exists(), "the old file is gone");
    assert_eq!(
        decision_files(&project),
        1,
        "the file was renamed, not copied"
    );
    let text = project.read("DOSSIER/decisions/DEC-001-three-inlet-guide-vanes.md");
    assert!(text.contains("# DEC-001 · Three inlet guide vanes\n"));
    let entry = entry_of(&project, "DEC-001");
    assert_eq!(entry.id(), "DEC-001");
    assert_eq!(entry.title(), "Three inlet guide vanes");
    assert_eq!(entry.status(), Some(Status::Approved));
    assert!(text.contains(&format!("- history: {WHEN} anna: changed — title\n")));
    assert!(snapshot_exists(&project, "DEC-001", "000001"));
}

#[test]
fn a_title_that_differs_only_in_case_keeps_the_file_name() {
    let project = Project::new();
    anna_decides(&project, "two inlet guide vanes");
    let before = project.entry_file("DEC-001");

    project
        .run("anna", &["retitle", "DEC-001", "Two Inlet Guide Vanes"])
        .success()
        .stdout("DEC-001: retitled\n");

    assert_eq!(project.entry_file("DEC-001"), before);
    assert_eq!(
        entry_of(&project, "DEC-001").title(),
        "Two Inlet Guide Vanes"
    );
    assert_eq!(decision_files(&project), 1);
}

#[test]
fn a_title_with_umlauts_gives_an_ascii_file_name() {
    let project = Project::new();
    anna_decides(&project, "Two inlet guide vanes");

    project
        .run("anna", &["retitle", "DEC-001", "Übersicht der Läufe"])
        .success()
        .stdout("DEC-001: retitled\n");

    project.read("DOSSIER/decisions/DEC-001-bersicht-der-l-ufe.md");
    assert_eq!(entry_of(&project, "DEC-001").title(), "Übersicht der Läufe");
}

#[test]
fn an_llm_retitling_an_approved_lesson_takes_it_back_to_draft() {
    let project = Project::new();
    anna_learns(&project, "Old lesson title");
    assert!(recorded_checksums(&project).contains_key("LES-001"));

    project
        .run("anna-claude", &["retitle", "LES-001", "New lesson title"])
        .success()
        .stdout("LES-001: retitled (back to draft, waiting for approval)\n");

    let entry = entry_of(&project, "LES-001");
    assert_eq!(entry.status(), Some(Status::Draft));
    assert_eq!(entry.field("approved"), None);
    assert_eq!(entry.field("valid-from"), None);
    assert_eq!(entry.title(), "New lesson title");
    let text = project.read("DOSSIER/lessons/LES-001-new-lesson-title.md");
    assert!(text.contains(&format!(
        "- history: {WHEN} anna via anna-claude: changed — title, back to draft\n"
    )));
    // The approved text stays in the snapshot, and the checksum of the approval is forgotten.
    assert!(snapshot_exists(&project, "LES-001", "000001"));
    assert!(!recorded_checksums(&project).contains_key("LES-001"));
}

#[test]
fn a_persons_retitle_of_an_approved_lesson_stays_approved_and_is_recorded() {
    let project = Project::new();
    anna_learns(&project, "Old lesson title");

    project
        .run("anna", &["retitle", "LES-001", "New lesson title"])
        .success()
        .stdout("LES-001: retitled\n");

    let entry = entry_of(&project, "LES-001");
    assert_eq!(entry.status(), Some(Status::Approved));
    let text = std::fs::read_to_string(project.entry_file("LES-001")).expect("the lesson file");
    let state = project.root().join("DOSSIER").join(".state");
    assert!(matches!(
        approval::mark(&state, "LES-001", text.as_bytes()),
        Mark::Unchanged
    ));
}

#[test]
fn the_same_title_changes_nothing() {
    let project = Project::new();
    anna_decides(&project, "Two inlet guide vanes");
    let before = std::fs::read_to_string(project.entry_file("DEC-001")).expect("the file");

    project
        .run("anna", &["retitle", "DEC-001", "  Two inlet guide vanes  "])
        .success()
        .stdout("DEC-001: no change\n");

    assert_eq!(
        std::fs::read_to_string(project.entry_file("DEC-001")).expect("the file"),
        before
    );
    assert!(!snapshot_exists(&project, "DEC-001", "000001"));
}

#[test]
fn a_blank_title_is_refused_and_nothing_changes() {
    let project = Project::new();
    anna_decides(&project, "Two inlet guide vanes");
    let before = project.entry_file("DEC-001");

    project
        .run("anna", &["retitle", "DEC-001", "   "])
        .failure()
        .code(1)
        .stderr(contains("the title must not be empty"));

    assert_eq!(project.entry_file("DEC-001"), before);
    assert!(!snapshot_exists(&project, "DEC-001", "000001"));
}

#[test]
fn an_unknown_id_is_not_found() {
    let project = Project::new();
    anna_decides(&project, "Two inlet guide vanes");

    project
        .run("anna", &["retitle", "DEC-099", "Something else"])
        .failure()
        .code(1)
        .stderr(contains("no entry DEC-099"));
    assert_eq!(decision_files(&project), 1);
}

#[test]
fn a_dry_run_names_the_rename_and_changes_nothing() {
    let project = Project::new();
    anna_decides(&project, "Two inlet guide vanes");
    let before = project.entry_file("DEC-001");

    project
        .run(
            "anna",
            &["--dry-run", "retitle", "DEC-001", "Three inlet guide vanes"],
        )
        .success()
        .stdout("DEC-001: retitled\n")
        .stderr(contains("dry run: would rename"));

    assert_eq!(project.entry_file("DEC-001"), before);
    assert_eq!(
        entry_of(&project, "DEC-001").title(),
        "Two inlet guide vanes"
    );
    assert!(!snapshot_exists(&project, "DEC-001", "000001"));
}

#[test]
fn a_dry_run_by_an_llm_leaves_an_approved_lesson_approved() {
    let project = Project::new();
    anna_learns(&project, "Old lesson title");
    let checksums = recorded_checksums(&project);

    project
        .run(
            "anna-claude",
            &["--dry-run", "retitle", "LES-001", "New lesson title"],
        )
        .success()
        .stdout("LES-001: retitled (back to draft, waiting for approval)\n");

    assert_eq!(
        entry_of(&project, "LES-001").status(),
        Some(Status::Approved)
    );
    assert_eq!(recorded_checksums(&project), checksums);
}

#[test]
fn a_retitle_without_a_name_is_a_usage_error() {
    let project = Project::new();
    anna_decides(&project, "Two inlet guide vanes");

    project
        .run("", &["retitle", "DEC-001", "Three inlet guide vanes"])
        .failure()
        .code(2)
        .stderr(contains("this command needs a name"));
    assert_eq!(decision_files(&project), 1);
}

#[test]
fn a_retitle_without_a_title_is_a_usage_error() {
    let project = Project::new();
    anna_decides(&project, "Two inlet guide vanes");

    project
        .run("anna", &["retitle", "DEC-001"])
        .failure()
        .code(2);
}
