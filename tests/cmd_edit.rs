//! Runs the built program for `edit`: the editor changes the entry under the lock, and the lines
//! the tool writes itself are refused when the editor changes them. The editors are stand-in
//! programs written by each test. Each test runs in its own temporary project folder with the
//! clock fixed by `common::Project` (2026-11-20T10:00:00+01:00).

mod common;

use common::Project;
use dossier::approval::{self, Mark};
use dossier::entry::{Entry, Status};
use predicates::str::contains;
use std::path::{Path, PathBuf};

/// The fixed clock of `common::Project`, as it appears in history lines.
const WHEN: &str = "2026-11-20T10:00:00+01:00";

/// The clock of the runs, as `common::Project::command` takes it.
const NOW: &str = "2026-11-20T10:00:00+01:00";

/// Invented text for a finding and a lesson used by the tests.
const OBSERVATION: &str = "The mass flow still oscillated by 0.4 % with half the time step.";
const EVIDENCE: &str = "Runs 12 and 13, monitor plots in results.";
const STATEMENT: &str = "Steady mixing-plane runs overpredict the stall margin.";
const VALID_FOR: &str = "An invented test rig with a tip Mach number above 1.1.";
const LESSON_EVIDENCE: &str = "An invented comparison of two speedlines, section 1.";

/// The stand-in editor that appends one line to the file it is given (the task's text).
const APPENDS_A_LINE: (&str, &str) = (
    "@echo off\r\necho Added by the editor.>> %1\r\n",
    "#!/bin/sh\necho \"Added by the editor.\" >> \"$1\"\n",
);

/// The stand-in editor that saves the file as it was.
const CHANGES_NOTHING: (&str, &str) = ("@echo off\r\n", "#!/bin/sh\nexit 0\n");

/// The text of the entry file with `id`.
fn text_of(project: &Project, id: &str) -> String {
    std::fs::read_to_string(project.entry_file(id)).expect("the entry file can be read")
}

/// The entry with `id`, parsed.
fn entry_of(project: &Project, id: &str) -> Entry {
    Entry::parse(&text_of(project, id)).expect("the file is an entry")
}

/// The temporary folder that the edit runs use (TMP, TEMP and TMPDIR point at it), so that a
/// test can see whether the editor's file was removed.
fn temporary_folder(project: &Project) -> PathBuf {
    project.root().join(".test").join("tmp")
}

/// Runs `edit <id>` as `agent`, with `editor` as the editor and the temporary folder above.
fn edit(project: &Project, agent: &str, id: &str, editor: &Path) -> assert_cmd::assert::Assert {
    let tmp = temporary_folder(project);
    std::fs::create_dir_all(&tmp).expect("the temporary folder of the test");
    project
        .command(agent, NOW)
        .env("EDITOR", editor)
        .env_remove("VISUAL")
        .env("TMPDIR", &tmp)
        .env("TMP", &tmp)
        .env("TEMP", &tmp)
        .args(["edit", id])
        .assert()
}

/// Whether the temporary folder of the test is empty: the editor's file was removed.
fn temporary_folder_is_empty(project: &Project) -> bool {
    std::fs::read_dir(temporary_folder(project))
        .expect("the temporary folder")
        .next()
        .is_none()
}

/// A finding written by anna, a person: approved at once.
fn anna_finds(project: &Project) {
    project.stdout("anna", &["found", OBSERVATION, "--evidence", EVIDENCE]);
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

/// Writes a stand-in editor that copies `replacement` over the file it is given, and returns its
/// path. The replacement is written into the test's own folder first.
fn copying_editor(project: &Project, replacement: &str) -> PathBuf {
    let source = project.root().join(".test").join("replacement.md");
    project.write(".test/replacement.md", replacement);
    let source = source.display();
    project.write_stand_in(
        "editor",
        &format!("@echo off\r\ncopy /Y \"{source}\" \"%~1\" >nul\r\n"),
        &format!("#!/bin/sh\ncp \"{source}\" \"$1\"\n"),
    )
}

#[test]
fn an_editor_change_is_written_with_one_history_line_and_the_temporary_file_is_removed() {
    let project = Project::new();
    anna_finds(&project);
    let (windows, unix) = APPENDS_A_LINE;
    let editor = project.write_stand_in("editor", windows, unix);
    let before = text_of(&project, "FND-001");

    edit(&project, "anna", "FND-001", &editor)
        .success()
        .stdout("FND-001: changed\n");

    let after = text_of(&project, "FND-001");
    assert_ne!(after, before);
    let entry = entry_of(&project, "FND-001");
    assert_eq!(
        entry.section("Evidence").as_deref(),
        Some("Runs 12 and 13, monitor plots in results.\nAdded by the editor.")
    );
    assert_eq!(entry.status(), Some(Status::Approved));
    assert!(after.contains(&format!("- history: {WHEN} anna: changed — edited\n")));
    assert!(temporary_folder_is_empty(&project));
}

#[test]
fn an_editor_that_saves_the_text_as_it_was_gives_no_change_and_writes_nothing() {
    let project = Project::new();
    anna_finds(&project);
    let (windows, unix) = CHANGES_NOTHING;
    let editor = project.write_stand_in("editor", windows, unix);
    let before = text_of(&project, "FND-001");
    let journal = project.read("DOSSIER/.state/journal.jsonl");

    edit(&project, "anna", "FND-001", &editor)
        .success()
        .stdout("FND-001: no change\n");

    assert_eq!(text_of(&project, "FND-001"), before);
    assert_eq!(project.read("DOSSIER/.state/journal.jsonl"), journal);
    assert!(temporary_folder_is_empty(&project));
}

#[test]
fn an_editor_that_changes_the_status_line_is_refused_and_nothing_is_written() {
    let project = Project::new();
    anna_finds(&project);
    let before = text_of(&project, "FND-001");
    let changed = before.replace("- status: approved", "- status: draft");
    assert_ne!(changed, before, "the example has an approved status line");
    let editor = copying_editor(&project, &changed);

    edit(&project, "anna", "FND-001", &editor)
        .failure()
        .code(1)
        .stderr(contains("these lines are written by the tool: status"));

    assert_eq!(text_of(&project, "FND-001"), before);
    assert!(temporary_folder_is_empty(&project));
}

#[test]
fn an_editor_that_changes_the_id_or_adds_a_history_line_is_refused() {
    let project = Project::new();
    anna_finds(&project);
    let before = text_of(&project, "FND-001");
    let changed = before.replace("# FND-001", "# FND-009").replace(
        "- kind: finding\n",
        &format!("- kind: finding\n- history: {WHEN} ben: changed — forged\n"),
    );
    assert_ne!(changed, before, "the example has a kind line");
    let editor = copying_editor(&project, &changed);

    edit(&project, "anna", "FND-001", &editor)
        .failure()
        .code(1)
        .stderr(contains("these lines are written by the tool: ID, history"));

    assert_eq!(text_of(&project, "FND-001"), before);
}

#[test]
fn an_editor_that_leaves_no_heading_is_a_parse_error_and_nothing_is_written() {
    let project = Project::new();
    anna_finds(&project);
    let before = text_of(&project, "FND-001");
    let editor = copying_editor(&project, "no heading here\n");

    edit(&project, "anna", "FND-001", &editor)
        .failure()
        .code(4)
        .stderr(contains("no heading"));

    assert_eq!(text_of(&project, "FND-001"), before);
    assert!(temporary_folder_is_empty(&project));
}

#[test]
fn an_editor_that_cannot_run_is_an_io_error_and_nothing_is_written() {
    let project = Project::new();
    anna_finds(&project);
    let before = text_of(&project, "FND-001");
    let missing = project.root().join(".test").join("no-such-editor");

    edit(&project, "anna", "FND-001", &missing)
        .failure()
        .code(4)
        .stderr(contains("cannot run the editor"));

    assert_eq!(text_of(&project, "FND-001"), before);
    assert!(temporary_folder_is_empty(&project));
}

#[test]
fn edit_without_a_name_is_a_usage_error_and_runs_no_editor() {
    let project = Project::new();
    anna_finds(&project);
    let before = text_of(&project, "FND-001");
    let missing = project.root().join(".test").join("no-such-editor");

    project
        .command("", NOW)
        .env("EDITOR", &missing)
        .env_remove("VISUAL")
        .args(["edit", "FND-001"])
        .assert()
        .failure()
        .code(2)
        .stderr(contains("this command needs a name"));

    assert_eq!(text_of(&project, "FND-001"), before);
}

#[test]
fn an_llm_edit_of_an_approved_lesson_takes_it_back_to_draft() {
    let project = Project::new();
    anna_learns(&project);
    let state = project.root().join("DOSSIER").join(".state");
    let (windows, unix) = APPENDS_A_LINE;
    let editor = project.write_stand_in("editor", windows, unix);

    edit(&project, "anna-claude", "LES-001", &editor)
        .success()
        .stdout("LES-001: changed (back to draft, waiting for approval)\n");

    let entry = entry_of(&project, "LES-001");
    assert_eq!(entry.status(), Some(Status::Draft));
    assert_eq!(entry.field("approved"), None);
    assert_eq!(entry.field("valid-from"), None);
    assert_eq!(
        entry.section("Evidence").as_deref(),
        Some("An invented comparison of two speedlines, section 1.\nAdded by the editor.")
    );
    let text = text_of(&project, "LES-001");
    assert!(text.contains(&format!(
        "- history: {WHEN} anna via anna-claude: changed — edited, back to draft\n"
    )));
    assert!(matches!(
        approval::mark(&state, "LES-001", text.as_bytes()),
        Mark::NotRecorded
    ));
    assert!(temporary_folder_is_empty(&project));
}

#[test]
fn a_persons_edit_of_an_approved_lesson_stays_approved_and_is_recorded() {
    let project = Project::new();
    anna_learns(&project);
    let state = project.root().join("DOSSIER").join(".state");
    let (windows, unix) = APPENDS_A_LINE;
    let editor = project.write_stand_in("editor", windows, unix);

    edit(&project, "anna", "LES-001", &editor)
        .success()
        .stdout("LES-001: changed\n");

    assert_eq!(
        entry_of(&project, "LES-001").status(),
        Some(Status::Approved)
    );
    let text = text_of(&project, "LES-001");
    assert!(matches!(
        approval::mark(&state, "LES-001", text.as_bytes()),
        Mark::Unchanged
    ));
}
