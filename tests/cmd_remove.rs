//! Runs the built program for `remove`: a person at a terminal, or an LLM with `--said-by <person>`
//! (D57), deletes an entry for good. The file is gone, the journal has a `removed` line without the
//! text, no snapshot of the text remains, and the ID is not reused. An LLM without its person's
//! `--said-by`, a missing `--yes`, a wrong answer and a missing terminal change nothing. Each test
//! runs in its own temporary project, with the clock fixed by `common::Project`.

mod common;

use common::Project;
use dossier::journal;
use predicates::str::contains;
use std::path::{Path, PathBuf};

/// The fixed clock of `common::Project`.
const NOW: &str = "2026-11-20T10:00:00+01:00";

/// Invented text for a finding that is to be removed; the test looks for it in the dossier.
const OBSERVATION: &str = "An invented observation: the probe read 3.2 kPa where the rig gave 2.9.";
const EVIDENCE: &str = "An invented run sheet, run 7.";

/// Runs `found` as anna: a finding, approved at once, with the ID FND-001.
fn approved_finding(project: &Project) {
    project
        .run("anna", &["found", OBSERVATION, "--evidence", EVIDENCE])
        .success()
        .stdout("FND-001: added (approved)\n");
}

/// Changes FND-001 once as anna, so that a snapshot of its text exists in the state folder.
fn changed_once(project: &Project) {
    project
        .run("anna", &["set", "FND-001", "--field", "machine=fan"])
        .success();
}

/// The state folder of the project.
fn state_of(project: &Project) -> PathBuf {
    project.root().join("DOSSIER").join(".state")
}

/// Whether any file under `folder`, in any subfolder, contains `text`.
fn folder_mentions(folder: &Path, text: &str) -> bool {
    let listing = std::fs::read_dir(folder).expect("the folder is read");
    listing.flatten().any(|entry| {
        let path = entry.path();
        if path.is_dir() {
            folder_mentions(&path, text)
        } else {
            let bytes = std::fs::read(&path).expect("the file is read");
            String::from_utf8_lossy(&bytes).contains(text)
        }
    })
}

#[test]
fn anna_removes_a_finding_for_good_and_no_copy_of_its_text_remains() {
    let project = Project::new();
    approved_finding(&project);
    changed_once(&project);
    let path = project.entry_file("FND-001");
    let snapshots = state_of(&project).join("history").join("FND-001");
    assert!(path.exists());
    assert!(snapshots.exists(), "the change saved a snapshot");
    assert!(folder_mentions(
        &project.root().join("DOSSIER"),
        OBSERVATION
    ));

    project
        .command("anna", NOW)
        .env("DOSSIER_ANSWERS", "FND-001")
        .args(["remove", "FND-001", "--yes"])
        .assert()
        .success()
        .stdout("FND-001: removed\n");

    // The file is gone, and so are its snapshots.
    assert!(!path.exists(), "the entry file is deleted");
    assert!(!snapshots.exists(), "the snapshot folder is deleted");
    // The text is nowhere in the dossier any more.
    assert!(!folder_mentions(
        &project.root().join("DOSSIER"),
        OBSERVATION
    ));

    // The journal has the removal with no detail, and it holds no text.
    let records = journal::read(&state_of(&project)).expect("the journal is read");
    let removed: Vec<_> = records
        .iter()
        .filter(|record| record.event == "removed")
        .collect();
    assert_eq!(removed.len(), 1);
    let record = removed[0];
    assert_eq!(record.id, "FND-001");
    assert_eq!(record.detail, "");
    assert_eq!(record.name, "anna");
    assert_eq!(record.person, "anna");
    assert!(!record.llm);
    assert_eq!(record.command, "remove");
    assert_eq!(record.time, NOW);
    assert!(
        !project
            .read("DOSSIER/.state/journal.jsonl")
            .contains("probe read")
    );
}

#[test]
fn the_next_finding_gets_a_higher_number_and_the_removed_id_is_not_reused() {
    let project = Project::new();
    approved_finding(&project);
    project
        .command("anna", NOW)
        .env("DOSSIER_ANSWERS", "FND-001")
        .args(["remove", "FND-001", "--yes"])
        .assert()
        .success()
        .stdout("FND-001: removed\n");

    project
        .run(
            "anna",
            &[
                "found",
                "A second invented observation.",
                "--evidence",
                EVIDENCE,
            ],
        )
        .success()
        .stdout("FND-002: added (approved)\n");
    assert!(project.entry_file("FND-002").exists());
}

#[test]
fn an_llm_cannot_remove_an_entry_and_the_file_stays() {
    let project = Project::new();
    approved_finding(&project);
    let path = project.entry_file("FND-001");
    let before = std::fs::read(&path).expect("the entry file is read");

    project
        .command("anna-claude", NOW)
        .env("DOSSIER_ANSWERS", "FND-001")
        .args(["remove", "FND-001", "--yes"])
        .assert()
        .code(1)
        .stderr(contains("--said-by anna"));

    assert_eq!(
        std::fs::read(&path).expect("the entry file is read"),
        before
    );
    assert!(
        !project
            .read("DOSSIER/.state/journal.jsonl")
            .contains("removed")
    );
}

#[test]
fn remove_without_yes_is_a_usage_error_and_deletes_nothing() {
    let project = Project::new();
    approved_finding(&project);
    let path = project.entry_file("FND-001");

    project
        .command("anna", NOW)
        .env("DOSSIER_ANSWERS", "FND-001")
        .args(["remove", "FND-001"])
        .assert()
        .code(2)
        .stderr(contains(
            "error: remove deletes the entry for good; add --yes",
        ));

    assert!(path.exists());
}

#[test]
fn a_wrong_answer_says_nothing_changed_and_the_file_stays() {
    let project = Project::new();
    approved_finding(&project);
    let path = project.entry_file("FND-001");

    project
        .command("anna", NOW)
        .env("DOSSIER_ANSWERS", "FND-002")
        .args(["remove", "FND-001", "--yes"])
        .assert()
        .success()
        .stdout("nothing changed\n");

    assert!(path.exists());
    assert!(
        !project
            .read("DOSSIER/.state/journal.jsonl")
            .contains("removed")
    );
}

#[test]
fn the_answer_and_the_id_may_be_given_in_any_case() {
    let project = Project::new();
    approved_finding(&project);

    project
        .command("anna", NOW)
        .env("DOSSIER_ANSWERS", "Fnd-001")
        .args(["remove", "fnd-001", "--yes"])
        .assert()
        .success()
        .stdout("FND-001: removed\n");

    assert!(!state_of(&project).join("history").join("FND-001").exists());
}

#[test]
fn without_a_terminal_remove_needs_a_person_at_a_terminal() {
    let project = Project::new();
    approved_finding(&project);
    let path = project.entry_file("FND-001");

    project
        .run("anna", &["remove", "FND-001", "--yes"])
        .code(1)
        .stderr(contains(
            "error: this command must be run by a person at a terminal",
        ));

    assert!(path.exists());
}

#[test]
fn an_unknown_id_is_refused() {
    let project = Project::new();
    project
        .command("anna", NOW)
        .env("DOSSIER_ANSWERS", "FND-009")
        .args(["remove", "FND-009", "--yes"])
        .assert()
        .code(1);
}

#[test]
fn an_llm_without_said_by_is_refused_and_nothing_is_removed() {
    let project = Project::new();
    approved_finding(&project);
    let path = project.entry_file("FND-001");
    let before = std::fs::read(&path).expect("the entry file is read");

    project
        .command("anna-claude", NOW)
        .args(["remove", "FND-001", "--yes"])
        .assert()
        .code(1)
        .stderr(contains("--said-by anna"));

    assert_eq!(
        std::fs::read(&path).expect("the entry file is read"),
        before
    );
    assert!(
        !project
            .read("DOSSIER/.state/journal.jsonl")
            .contains("removed")
    );
}

#[test]
fn an_llm_with_said_by_removes_the_entry_without_a_terminal_or_a_question() {
    let project = Project::new();
    approved_finding(&project);
    let path = project.entry_file("FND-001");

    // No answers are prepared and no terminal is attached, so nothing can be asked (D57). The
    // entry file is gone after the removal, so its history cannot be read; the journal is checked.
    project
        .command("anna-claude", NOW)
        .args(["remove", "FND-001", "--yes", "--said-by", "anna"])
        .assert()
        .success()
        .stdout("FND-001: removed\n")
        .stderr("");

    assert!(!path.exists(), "the entry file is deleted");
    let journal = project.entry_journal();
    let last = journal.last().expect("a journal line");
    assert_eq!(last.command, "remove");
    assert_eq!(last.id, "FND-001");
    assert_eq!(last.event, "removed");
    assert_eq!(last.person, "anna");
    assert_eq!(last.name, "anna-claude");
    assert!(last.llm);
}

#[test]
fn an_llm_with_said_by_for_another_person_is_refused_and_nothing_is_removed() {
    let project = Project::new();
    approved_finding(&project);
    let path = project.entry_file("FND-001");
    let before = std::fs::read(&path).expect("the entry file is read");

    project
        .command("anna-claude", NOW)
        .args(["remove", "FND-001", "--yes", "--said-by", "ben"])
        .assert()
        .code(1)
        .stderr(contains("acts for anna only"));

    assert_eq!(
        std::fs::read(&path).expect("the entry file is read"),
        before
    );
}

#[test]
fn a_worker_session_removes_nothing_even_with_said_by() {
    let project = Project::new();
    approved_finding(&project);
    let path = project.entry_file("FND-001");
    let before = std::fs::read(&path).expect("the entry file is read");

    project
        .command("anna-claude", NOW)
        .env("DOSSIER_ROLE", "worker")
        .args(["remove", "FND-001", "--yes", "--said-by", "anna"])
        .assert()
        .code(1)
        .stderr(contains("a worker decides nothing"));

    assert_eq!(
        std::fs::read(&path).expect("the entry file is read"),
        before
    );
}

#[test]
fn an_llm_with_said_by_but_without_yes_is_a_usage_error_and_deletes_nothing() {
    let project = Project::new();
    approved_finding(&project);
    let path = project.entry_file("FND-001");

    project
        .command("anna-claude", NOW)
        .args(["remove", "FND-001", "--said-by", "anna"])
        .assert()
        .code(2)
        .stderr(contains(
            "error: remove deletes the entry for good; add --yes",
        ));

    assert!(path.exists());
}
