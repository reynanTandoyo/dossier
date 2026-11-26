//! Runs the built program for `approve`: a person at a terminal approves or rejects drafts, one
//! answer per draft. An LLM cannot approve without `--said-by <person>` (D57), and nothing is asked
//! without a terminal. Each test runs in its own project folder, with the clock fixed by
//! `common::Project`.

mod common;

use common::Project;
use dossier::approval::{self, Mark};
use dossier::entry::{Entry, Status};
use predicates::str::contains;
use std::path::{Path, PathBuf};

/// The fixed clock of `common::Project`.
const NOW: &str = "2026-11-20T10:00:00+01:00";

/// Invented text for the parts of a lesson.
const STATEMENT: &str = "Steady runs with a mixing-plane interface overpredict the stall margin.";
const VALID_FOR: &str = "An invented test rig with a tip Mach number above 1.1.";
const EVIDENCE: &str = "An invented comparison of two speedlines, section 1.";

/// The stand-in editor that appends one line to the file it is given (as in tests/cmd_edit.rs).
const APPENDS_A_LINE: (&str, &str) = (
    "@echo off\r\necho Added by the editor.>> %1\r\n",
    "#!/bin/sh\necho \"Added by the editor.\" >> \"$1\"\n",
);

/// The stand-in editor that saves the file as it was (as in tests/cmd_edit.rs).
const CHANGES_NOTHING: (&str, &str) = ("@echo off\r\n", "#!/bin/sh\nexit 0\n");

/// The stand-in editor that ends with an error (exit code 1) and saves nothing.
const EXITS_WITH_ERROR: (&str, &str) = ("@echo off\r\nexit /b 1\r\n", "#!/bin/sh\nexit 1\n");

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

/// The text of the entry file with `id`.
fn text_of(project: &Project, id: &str) -> String {
    std::fs::read_to_string(project.entry_file(id)).expect("the entry file is read")
}

/// The entry file with `id`, read and parsed.
fn entry_of(project: &Project, id: &str) -> Entry {
    Entry::parse(&text_of(project, id)).expect("the entry parses")
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
            "error: approve carries out a person's decision: ask your person, and with their yes run it again with --said-by anna",
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
fn an_llm_without_said_by_is_refused_and_nothing_is_written() {
    let project = Project::new();
    draft_lesson(&project);
    approve_with(&project, "anna-claude", "a", &["LES-001"])
        .code(1)
        .stderr(contains("--said-by anna"));

    assert_eq!(entry_of(&project, "LES-001").status(), Some(Status::Draft));
    assert!(!project.root().join("DOSSIER/.state/approved.json").exists());
}

#[test]
fn an_llm_with_said_by_approves_its_drafts_without_a_terminal_or_a_question() {
    let project = Project::new();
    draft_lesson(&project);
    draft_lesson(&project);
    // No answers are prepared and no terminal is attached, so nothing can be asked: the drafts are
    // approved as the person decided (D57).
    project
        .command("anna-claude", NOW)
        .args(["approve", "LES-001", "LES-002", "--said-by", "anna"])
        .assert()
        .success()
        .stdout("LES-001: approved\nLES-002: approved\n")
        .stderr("");

    for id in ["LES-001", "LES-002"] {
        let entry = entry_of(&project, id);
        assert_eq!(entry.status(), Some(Status::Approved));
        assert_eq!(
            entry.field("approved").as_deref(),
            Some(format!("{NOW} by anna").as_str())
        );
        assert_eq!(entry.field("author").as_deref(), Some("anna"));
        assert_eq!(entry.field("via").as_deref(), Some("anna-claude"));
        assert_eq!(
            entry.history().last().map(String::as_str),
            Some(format!("{NOW} anna via anna-claude: approved").as_str())
        );
    }
    let journal = project.entry_journal();
    let newest = journal.last().expect("the approvals are journalled");
    assert_eq!(newest.id, "LES-002");
    assert_eq!(newest.event, "approved");
    assert_eq!(newest.person, "anna");
    assert_eq!(newest.name, "anna-claude");
    assert!(newest.llm);
    assert!(
        project
            .read("DOSSIER/.state/approved.json")
            .contains("LES-002")
    );
}

#[test]
fn an_llm_with_said_by_for_another_person_is_refused_and_nothing_is_written() {
    let project = Project::new();
    draft_lesson(&project);
    project
        .command("anna-claude", NOW)
        .args(["approve", "LES-001", "--said-by", "ben"])
        .assert()
        .code(1)
        .stderr(contains("acts for anna only"));

    assert_eq!(entry_of(&project, "LES-001").status(), Some(Status::Draft));
    assert!(!project.root().join("DOSSIER/.state/approved.json").exists());
}

#[test]
fn a_worker_session_decides_nothing_even_with_said_by() {
    let project = Project::new();
    draft_lesson(&project);
    draft_lesson(&project);
    project
        .command("anna-claude", NOW)
        .env("DOSSIER_ROLE", "worker")
        .args(["approve", "LES-001", "LES-002", "--said-by", "anna"])
        .assert()
        .code(1)
        .stderr(contains("a worker decides nothing"));

    for id in ["LES-001", "LES-002"] {
        assert_eq!(entry_of(&project, id).status(), Some(Status::Draft));
    }
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

/// Runs `approve` with `ids` as anna, answering `answers`, with `editor` as the editor (VISUAL).
/// The editor's temporary file goes into the project's `.test/tmp` folder, as in tests/cmd_edit.rs.
fn approve_with_editor(
    project: &Project,
    answers: &str,
    editor: &Path,
    ids: &[&str],
) -> assert_cmd::assert::Assert {
    let tmp = project.root().join(".test").join("tmp");
    std::fs::create_dir_all(&tmp).expect("the temporary folder of the test");
    project
        .command("anna", NOW)
        .env("DOSSIER_ANSWERS", answers)
        .env("VISUAL", editor)
        .env("TMPDIR", &tmp)
        .env("TMP", &tmp)
        .env("TEMP", &tmp)
        .arg("approve")
        .args(ids)
        .assert()
}

#[test]
fn an_edit_before_approval_is_kept_and_its_history_line_comes_before_the_approval() {
    let project = Project::new();
    draft_lesson(&project);
    let (windows, unix) = APPENDS_A_LINE;
    let editor = project.write_stand_in("editor", windows, unix);

    approve_with_editor(&project, "e|a", &editor, &["LES-001"])
        .success()
        .stdout("LES-001: approved\n");

    let entry = entry_of(&project, "LES-001");
    assert_eq!(entry.status(), Some(Status::Approved));
    assert_eq!(
        entry.section("Evidence").as_deref(),
        Some("An invented comparison of two speedlines, section 1.\nAdded by the editor.")
    );
    assert_eq!(
        entry.history(),
        [
            format!("{NOW} anna via anna-claude: created"),
            format!("{NOW} anna: changed — edited before approval"),
            format!("{NOW} anna: approved"),
        ]
    );
    // The checksum is that of the edited text, the text that was approved.
    let state = project.root().join("DOSSIER").join(".state");
    let text = text_of(&project, "LES-001");
    assert!(matches!(
        approval::mark(&state, "LES-001", text.as_bytes()),
        Mark::Unchanged
    ));
}

#[test]
fn an_edit_followed_by_a_skip_is_kept_and_the_draft_stays_a_draft() {
    let project = Project::new();
    draft_lesson(&project);
    let (windows, unix) = APPENDS_A_LINE;
    let editor = project.write_stand_in("editor", windows, unix);

    approve_with_editor(&project, "e|s", &editor, &["LES-001"])
        .success()
        .stdout("LES-001: skipped\n");

    let entry = entry_of(&project, "LES-001");
    assert_eq!(entry.status(), Some(Status::Draft));
    assert_eq!(
        entry.section("Evidence").as_deref(),
        Some("An invented comparison of two speedlines, section 1.\nAdded by the editor.")
    );
    assert_eq!(
        entry.history(),
        [
            format!("{NOW} anna via anna-claude: created"),
            format!("{NOW} anna: changed — edited before approval"),
        ]
    );
    assert!(!project.root().join("DOSSIER/.state/approved.json").exists());
}

#[test]
fn an_editor_that_saves_the_draft_unchanged_is_asked_again_and_leaves_no_change_line() {
    let project = Project::new();
    draft_lesson(&project);
    let (windows, unix) = CHANGES_NOTHING;
    let editor = project.write_stand_in("editor", windows, unix);

    approve_with_editor(&project, "e|r", &editor, &["LES-001"])
        .success()
        .stdout("LES-001: rejected\n");

    let entry = entry_of(&project, "LES-001");
    assert_eq!(entry.status(), Some(Status::Rejected));
    assert_eq!(
        entry.history(),
        [
            format!("{NOW} anna via anna-claude: created"),
            format!("{NOW} anna: rejected"),
        ]
    );
}

#[test]
fn a_failed_editor_keeps_the_answers_given_before_it_and_the_draft_is_asked_again() {
    let project = Project::new();
    draft_lesson(&project);
    draft_lesson(&project);
    let (windows, unix) = EXITS_WITH_ERROR;
    let editor = project.write_stand_in("editor", windows, unix);
    let before = std::fs::read(project.entry_file("LES-002")).expect("the entry file is read");

    approve_with_editor(&project, "a|e|s", &editor, &["LES-001", "LES-002"])
        .success()
        .stdout("LES-001: approved\nLES-002: skipped\n")
        .stderr(contains("warning: LES-002: the editor ended with"))
        .stderr(contains("; nothing was changed; the draft is unchanged"));

    assert_eq!(
        entry_of(&project, "LES-001").status(),
        Some(Status::Approved)
    );
    assert_eq!(
        std::fs::read(project.entry_file("LES-002")).expect("the entry file is read"),
        before
    );
    assert_eq!(entry_of(&project, "LES-002").status(), Some(Status::Draft));
    let state = project.read("DOSSIER/.state/approved.json");
    assert!(state.contains("LES-001"));
    assert!(!state.contains("LES-002"));
}

/// Writes `replacement` as the text that the stand-in editor saves: a copy of a file in the
/// project's `.test` folder (as in tests/cmd_edit.rs).
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
fn an_edit_the_tool_refuses_keeps_the_draft_and_asks_again() {
    let project = Project::new();
    draft_lesson(&project);
    let before = text_of(&project, "LES-001");
    let changed = before.replace("- status: draft", "- status: approved");
    assert_ne!(changed, before, "the example has a draft status line");
    let editor = copying_editor(&project, &changed);

    approve_with_editor(&project, "e|r", &editor, &["LES-001"])
        .success()
        .stdout("LES-001: rejected\n")
        .stderr(contains(
            "warning: LES-001: these lines are written by the tool: status; the draft is unchanged",
        ));

    let entry = entry_of(&project, "LES-001");
    assert_eq!(entry.status(), Some(Status::Rejected));
    assert_eq!(
        entry.history(),
        [
            format!("{NOW} anna via anna-claude: created"),
            format!("{NOW} anna: rejected"),
        ]
    );
}

#[test]
fn approve_needs_at_least_one_id() {
    let project = Project::new();
    project.run("anna", &["approve"]).code(2);
}
