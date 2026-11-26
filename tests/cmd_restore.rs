//! Runs the built program for `restore`: a person at a terminal, or an LLM with `--said-by <person>`
//! (D57), brings a snapshot back. The entry gets the snapshot's field back, keeps every earlier
//! history line and gains a `restored` line; the text before the restore is kept as a snapshot too.
//! An LLM without its person's `--said-by`, a wrong answer, an unknown snapshot, a missing terminal
//! and a dry run change nothing. Each test runs in its own temporary project, with the clock fixed
//! by `common::Project`.

mod common;

use common::Project;
use dossier::{journal, snapshot};
use predicates::str::contains;
use std::path::PathBuf;

/// The fixed clock of `common::Project`, when the entry is created.
const NOW: &str = "2026-11-20T10:00:00+01:00";
/// When the field is changed, and when the snapshot is brought back.
const CHANGE_TIME: &str = "2026-11-21T09:00:00+01:00";
const RESTORE_TIME: &str = "2026-11-22T09:00:00+01:00";

/// Invented texts; the tests look for them in the entry files.
const OBSERVATION: &str = "An invented observation: the probe read 3.2 kPa where the rig gave 2.9.";
const EVIDENCE: &str = "An invented run sheet, run 7.";
const OLD_TITLE: &str = "Probe reading off by a tenth";
const NEW_TITLE: &str = "Probe reading off by a little";
const STATEMENT: &str = "An invented lesson: the probe needs a warm-up of ten minutes.";
const VALID_FOR: &str = "The probe model used in the invented test rig.";

/// The state folder of the project.
fn state_of(project: &Project) -> PathBuf {
    project.root().join("DOSSIER").join(".state")
}

/// The text of the entry file of `id`.
fn text_of(project: &Project, id: &str) -> String {
    std::fs::read_to_string(project.entry_file(id)).expect("the entry file is read")
}

/// The history lines of an entry file, in file order, with the `- history: ` prefix removed.
fn history_of(text: &str) -> Vec<String> {
    text.lines()
        .filter_map(|line| line.strip_prefix("- history: "))
        .map(str::to_string)
        .collect()
}

/// Runs `found` as anna with machine rig-a, approved at once, as FND-001 with the title OLD_TITLE.
fn found_with_machine(project: &Project) {
    project
        .run(
            "anna",
            &[
                "found",
                OBSERVATION,
                "--evidence",
                EVIDENCE,
                "--title",
                OLD_TITLE,
                "--machine",
                "rig-a",
            ],
        )
        .success()
        .stdout("FND-001: added (approved)\n");
}

/// Changes the machine of FND-001 to fan as anna on CHANGE_TIME; snapshot 1 keeps rig-a.
fn machine_changed_to_fan(project: &Project) {
    project
        .run_at(
            "anna",
            CHANGE_TIME,
            &["set", "FND-001", "--field", "machine=fan"],
        )
        .success();
}

#[test]
fn anna_brings_back_snapshot_1_and_the_history_keeps_every_line_and_gains_restored() {
    let project = Project::new();
    found_with_machine(&project);
    machine_changed_to_fan(&project);
    assert!(text_of(&project, "FND-001").contains("- machine: fan\n"));

    project
        .command("anna", RESTORE_TIME)
        .env("DOSSIER_ANSWERS", "yes")
        .args(["restore", "FND-001", "1"])
        .assert()
        .success()
        .stdout("FND-001: restored to snapshot 1\n");

    let text = text_of(&project, "FND-001");
    assert!(text.contains("- machine: rig-a\n"), "the old field is back");
    assert!(!text.contains("machine: fan"), "the changed field is gone");
    assert!(text.contains(EVIDENCE), "the sections are the snapshot's");
    assert_eq!(
        history_of(&text),
        [
            format!("{NOW} anna: created"),
            format!("{NOW} anna: approved"),
            format!("{CHANGE_TIME} anna: changed — field machine"),
            format!("{RESTORE_TIME} anna: restored — snapshot 1"),
        ]
    );
}

#[test]
fn the_text_before_the_restore_is_kept_as_a_snapshot_and_the_journal_says_restored() {
    let project = Project::new();
    found_with_machine(&project);
    machine_changed_to_fan(&project);
    project
        .command("anna", RESTORE_TIME)
        .env("DOSSIER_ANSWERS", "yes")
        .args(["restore", "FND-001", "1"])
        .assert()
        .success();

    let snaps = snapshot::list(&state_of(&project), "FND-001").expect("the snapshots are listed");
    assert_eq!(snaps.len(), 2, "the change and the restore each saved one");
    assert_eq!(snaps[1].n, 2);
    assert_eq!(snaps[1].command, "restore");
    assert_eq!(snaps[1].name, "anna");
    let before_restore = snapshot::load(&state_of(&project), "FND-001", 2).expect("snapshot 2");
    assert!(String::from_utf8_lossy(&before_restore).contains("- machine: fan\n"));

    let records = journal::read(&state_of(&project)).expect("the journal is read");
    let restored: Vec<_> = records
        .iter()
        .filter(|record| record.event == "restored")
        .collect();
    assert_eq!(restored.len(), 1);
    let record = restored[0];
    assert_eq!(record.id, "FND-001");
    assert_eq!(record.detail, "snapshot 1");
    assert_eq!(record.command, "restore");
    assert_eq!(record.name, "anna");
    assert_eq!(record.person, "anna");
    assert!(!record.llm);
    assert_eq!(record.time, RESTORE_TIME);
}

#[test]
fn a_restored_title_moves_the_file_back_to_the_name_it_had() {
    let project = Project::new();
    found_with_machine(&project);
    machine_changed_to_fan(&project);
    project
        .run_at(
            "anna",
            "2026-11-21T12:00:00+01:00",
            &["retitle", "FND-001", NEW_TITLE],
        )
        .success()
        .stdout("FND-001: retitled\n");
    let renamed = project
        .root()
        .join("DOSSIER")
        .join("findings")
        .join(format!("FND-001-{}.md", slug_of(NEW_TITLE)));
    assert!(renamed.exists(), "the retitle renamed the file");

    project
        .command("anna", RESTORE_TIME)
        .env("DOSSIER_ANSWERS", "yes")
        .args(["restore", "FND-001", "1"])
        .assert()
        .success();

    let old_name = project
        .root()
        .join("DOSSIER")
        .join("findings")
        .join(format!("FND-001-{}.md", slug_of(OLD_TITLE)));
    assert!(old_name.exists(), "the file has the name of the old title");
    assert!(!renamed.exists(), "the file with the new title is gone");
    assert!(text_of(&project, "FND-001").contains(OLD_TITLE));
}

#[test]
fn restoring_an_approved_lesson_is_not_marked_as_changed_after_approval() {
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
        .success()
        .stdout("LES-001: added (approved)\n");
    project
        .run_at(
            "anna",
            CHANGE_TIME,
            &["set", "LES-001", "--field", "machine=fan"],
        )
        .success();
    assert!(
        !project
            .stdout("anna", &["show", "LES-001"])
            .contains("changed after approval")
    );

    project
        .command("anna", RESTORE_TIME)
        .env("DOSSIER_ANSWERS", "yes")
        .args(["restore", "LES-001", "1"])
        .assert()
        .success()
        .stdout("LES-001: restored to snapshot 1\n");

    let text = text_of(&project, "LES-001");
    assert!(text.contains("status: approved"));
    assert!(!text.contains("machine: fan"));
    assert!(
        !project
            .stdout("anna", &["show", "LES-001"])
            .contains("changed after approval")
    );
}

#[test]
fn an_llm_without_said_by_cannot_restore_and_the_file_stays() {
    let project = Project::new();
    found_with_machine(&project);
    machine_changed_to_fan(&project);
    let path = project.entry_file("FND-001");
    let before = std::fs::read(&path).expect("the entry file is read");

    project
        .command("anna-claude", RESTORE_TIME)
        .env("DOSSIER_ANSWERS", "yes")
        .args(["restore", "FND-001", "1"])
        .assert()
        .code(1)
        .stderr(contains("--said-by anna"));

    assert_eq!(
        std::fs::read(&path).expect("the entry file is read"),
        before
    );
}

#[test]
fn an_llm_with_said_by_restores_the_snapshot_without_a_terminal_or_a_question() {
    let project = Project::new();
    found_with_machine(&project);
    machine_changed_to_fan(&project);

    // No answers are prepared and no terminal is attached, so nothing can be asked (D57).
    project
        .command("anna-claude", RESTORE_TIME)
        .args(["restore", "FND-001", "1", "--said-by", "anna"])
        .assert()
        .success()
        .stdout("FND-001: restored to snapshot 1\n")
        .stderr("");

    let text = text_of(&project, "FND-001");
    assert!(
        text.contains("- machine: rig-a\n"),
        "the snapshot's field is back"
    );
    assert!(!text.contains("machine: fan"), "the changed field is gone");
    let history = history_of(&text);
    let last = history.last().expect("a history line");
    assert!(
        last.contains("anna via anna-claude: restored"),
        "the newest history line names the person and the LLM: {last}"
    );

    let journal = project.entry_journal();
    let record = journal.last().expect("a journal line");
    assert_eq!(record.command, "restore");
    assert_eq!(record.id, "FND-001");
    assert_eq!(record.event, "restored");
    assert_eq!(record.detail, "snapshot 1");
    assert_eq!(record.person, "anna");
    assert_eq!(record.name, "anna-claude");
    assert!(record.llm);
}

#[test]
fn an_llm_with_said_by_for_another_person_restores_nothing() {
    let project = Project::new();
    found_with_machine(&project);
    machine_changed_to_fan(&project);
    let path = project.entry_file("FND-001");
    let before = std::fs::read(&path).expect("the entry file is read");

    project
        .command("anna-claude", RESTORE_TIME)
        .args(["restore", "FND-001", "1", "--said-by", "ben"])
        .assert()
        .code(1)
        .stderr(contains("acts for anna only"));

    assert_eq!(
        std::fs::read(&path).expect("the entry file is read"),
        before
    );
    assert!(
        !project
            .read("DOSSIER/.state/journal.jsonl")
            .contains("restored")
    );
}

#[test]
fn a_worker_session_restores_nothing_even_with_said_by() {
    let project = Project::new();
    found_with_machine(&project);
    machine_changed_to_fan(&project);
    let path = project.entry_file("FND-001");
    let before = std::fs::read(&path).expect("the entry file is read");

    project
        .command("anna-claude", RESTORE_TIME)
        .env("DOSSIER_ROLE", "worker")
        .args(["restore", "FND-001", "1", "--said-by", "anna"])
        .assert()
        .code(1)
        .stderr(contains("a worker decides nothing"));

    assert_eq!(
        std::fs::read(&path).expect("the entry file is read"),
        before
    );
}

#[test]
fn any_answer_but_yes_says_nothing_changed_and_the_file_stays() {
    let project = Project::new();
    found_with_machine(&project);
    machine_changed_to_fan(&project);
    let path = project.entry_file("FND-001");
    let before = std::fs::read(&path).expect("the entry file is read");

    project
        .command("anna", RESTORE_TIME)
        .env("DOSSIER_ANSWERS", "no")
        .args(["restore", "FND-001", "1"])
        .assert()
        .success()
        .stdout("nothing changed\n");

    assert_eq!(
        std::fs::read(&path).expect("the entry file is read"),
        before
    );
    assert!(
        !project
            .read("DOSSIER/.state/journal.jsonl")
            .contains("restored")
    );
}

#[test]
fn an_unknown_snapshot_is_refused_before_the_question() {
    let project = Project::new();
    found_with_machine(&project);
    machine_changed_to_fan(&project);
    let path = project.entry_file("FND-001");
    let before = std::fs::read(&path).expect("the entry file is read");

    project
        .command("anna", RESTORE_TIME)
        .env("DOSSIER_ANSWERS", "yes")
        .args(["restore", "FND-001", "9"])
        .assert()
        .code(1)
        .stderr(contains("error: no snapshot 9 of FND-001"));

    assert_eq!(
        std::fs::read(&path).expect("the entry file is read"),
        before
    );
}

#[test]
fn without_a_terminal_restore_needs_a_person_at_a_terminal() {
    let project = Project::new();
    found_with_machine(&project);
    machine_changed_to_fan(&project);

    project
        .run_at("anna", RESTORE_TIME, &["restore", "FND-001", "1"])
        .code(1)
        .stderr(contains(
            "error: this command must be run by a person at a terminal",
        ));

    assert!(text_of(&project, "FND-001").contains("machine: fan"));
}

#[test]
fn a_dry_run_says_what_it_would_write_and_writes_nothing() {
    let project = Project::new();
    found_with_machine(&project);
    machine_changed_to_fan(&project);
    let before = text_of(&project, "FND-001");

    project
        .command("anna", RESTORE_TIME)
        .env("DOSSIER_ANSWERS", "yes")
        .args(["--dry-run", "restore", "FND-001", "1"])
        .assert()
        .success()
        .stdout("FND-001: restored to snapshot 1\n")
        .stderr(contains("warning: dry run: would write DOSSIER/findings/"));

    assert_eq!(text_of(&project, "FND-001"), before);
    assert!(
        !project
            .read("DOSSIER/.state/journal.jsonl")
            .contains("restored")
    );
}

/// The slug of a title as a file name has it (format.md §4), for the titles of these tests.
fn slug_of(title: &str) -> String {
    dossier::paths::slug(title)
}
