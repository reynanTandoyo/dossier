//! Runs the built program for `detach`: a file of a task's list is marked detached, its block and
//! history stay in the list, and only the detached mark and its history line are new (R-FILE-13).
//! Each test runs in its own temporary project folder with the clock fixed by `common::Project`
//! (2026-11-20T10:00:00+01:00).

mod common;

use common::Project;
use dossier::blockfile::BlockFile;
use dossier::journal::Record;
use dossier::paths::shown;
use predicates::str::contains;

/// The fixed clock of `common::Project`, as it appears in history and journal lines.
const WHEN: &str = "2026-11-20T10:00:00+01:00";

/// The invented contents of the model file of the tests.
const MODEL: &str = "invented model data\n";

/// The stored path of the model file of the tests.
const MODEL_PATH: &str = "models/wing-v3.step";

/// The text of the file list of task T-001.
fn list_text(project: &Project) -> String {
    project.read("DOSSIER/files/T-001.md")
}

/// The file list of task T-001, parsed.
fn list_of(project: &Project) -> BlockFile {
    BlockFile::parse(&list_text(project)).expect("the file list parses")
}

/// The journal records of the file list of task T-001, oldest first.
fn list_journal(project: &Project) -> Vec<Record> {
    project
        .entry_journal()
        .into_iter()
        .filter(|record| record.id == "FILES-T-001")
        .collect()
}

/// Writes the invented model file and attaches it to T-001 as a model with a note, as anna.
fn attach_model(project: &Project) {
    project.write(MODEL_PATH, MODEL);
    project.stdout(
        "anna",
        &[
            "attach",
            "T-001",
            MODEL_PATH,
            "--role",
            "model",
            "--note",
            "first mesh",
        ],
    );
}

#[test]
fn detach_marks_the_block_detached_and_keeps_its_fields_and_history() {
    let project = Project::new();
    attach_model(&project);
    let before = list_text(&project);

    let output = project.stdout("anna-claude", &["detach", "T-001", MODEL_PATH]);
    assert_eq!(output, format!("T-001: detached {}\n", shown(MODEL_PATH)));

    // The mark names who acted, as attach and map do: the LLM's name here.
    let list = list_of(&project);
    assert_eq!(list.keys(), [MODEL_PATH]);
    assert_eq!(
        list.field(MODEL_PATH, "detached"),
        Some(format!("{WHEN} by anna-claude"))
    );
    assert_eq!(list.field(MODEL_PATH, "role").as_deref(), Some("model"));
    assert_eq!(
        list.field(MODEL_PATH, "note").as_deref(),
        Some("first mesh")
    );
    assert_eq!(
        list.history(MODEL_PATH),
        [
            format!("{WHEN} anna: attached"),
            format!("{WHEN} anna via anna-claude: detached"),
        ]
    );

    // Only the detached mark and the detach history line are new: every other line stays.
    let added = [
        format!("- detached: {WHEN} by anna-claude"),
        format!("- history: {WHEN} anna via anna-claude: detached"),
    ];
    let mut after: Vec<String> = list_text(&project).lines().map(str::to_string).collect();
    after.retain(|line| !added.contains(line));
    let before: Vec<String> = before.lines().map(str::to_string).collect();
    assert_eq!(after, before);

    let journal = list_journal(&project);
    let last = journal.last().expect("the detach is in the journal");
    assert_eq!(last.event, "detached");
    assert_eq!(last.detail, MODEL_PATH);
    assert_eq!(last.command, "detach");
    assert_eq!(last.name, "anna-claude");
    assert_eq!(last.person, "anna");
    assert!(last.llm);
}

#[test]
fn detaching_a_file_again_is_refused_and_the_list_does_not_change() {
    let project = Project::new();
    attach_model(&project);
    project.stdout("anna", &["detach", "T-001", MODEL_PATH]);
    let before = list_text(&project);
    let journal_before = list_journal(&project).len();

    project
        .run("anna", &["detach", "T-001", MODEL_PATH])
        .failure()
        .code(1)
        .stderr(contains("is already detached from T-001"));

    assert_eq!(list_text(&project), before);
    assert_eq!(list_journal(&project).len(), journal_before);
}

#[test]
fn a_file_that_is_not_attached_to_the_task_is_refused_and_no_list_is_written() {
    let project = Project::new();
    attach_model(&project);
    let before = list_text(&project);

    project
        .run("anna", &["detach", "T-001", "models/never-attached.step"])
        .failure()
        .code(1)
        .stderr(contains("is not attached to T-001"));
    project
        .run("anna", &["detach", "T-002", MODEL_PATH])
        .failure()
        .code(1)
        .stderr(contains("is not attached to T-002"));

    assert_eq!(list_text(&project), before);
    assert!(!project.root().join("DOSSIER/files/T-002.md").exists());
    assert_eq!(list_journal(&project).len(), 1);
}

#[test]
fn a_file_that_is_gone_from_the_disk_can_still_be_detached() {
    let project = Project::new();
    attach_model(&project);
    std::fs::remove_file(project.root().join(MODEL_PATH)).expect("the model is removed on disk");

    project.stdout("anna", &["detach", "T-001", MODEL_PATH]);

    assert_eq!(
        list_of(&project).field(MODEL_PATH, "detached"),
        Some(format!("{WHEN} by anna"))
    );
}

#[test]
fn a_detached_file_attached_again_keeps_both_events_in_its_history() {
    let project = Project::new();
    attach_model(&project);
    project.stdout("anna", &["detach", "T-001", MODEL_PATH]);

    project.stdout("anna", &["attach", "T-001", MODEL_PATH, "--role", "model"]);

    let list = list_of(&project);
    assert!(list.field(MODEL_PATH, "detached").is_none());
    assert_eq!(
        list.history(MODEL_PATH),
        [
            format!("{WHEN} anna: attached"),
            format!("{WHEN} anna: detached"),
            format!("{WHEN} anna: attached"),
        ]
    );
}

#[test]
fn a_path_with_spaces_and_non_ascii_letters_can_be_detached() {
    let project = Project::new();
    let path = "Modelle ü/Lauf 1.step";
    project.write(path, MODEL);
    project.stdout("anna", &["attach", "T-001", path, "--role", "model"]);

    project.stdout("anna", &["detach", "T-001", path]);

    assert_eq!(
        list_of(&project).field(path, "detached"),
        Some(format!("{WHEN} by anna"))
    );
}

#[test]
fn detach_needs_a_name_and_writes_nothing_without_one() {
    let project = Project::new();
    attach_model(&project);
    let before = list_text(&project);

    project
        .run("", &["detach", "T-001", MODEL_PATH])
        .failure()
        .code(2)
        .stderr(contains("this command needs a name"));

    assert_eq!(list_text(&project), before);
}

#[test]
fn json_output_names_the_detached_path() {
    let project = Project::new();
    attach_model(&project);

    let output = project.stdout("anna", &["detach", "T-001", MODEL_PATH, "--json"]);
    let value: serde_json::Value =
        serde_json::from_str(output.trim_end()).expect("one line of JSON");
    assert_eq!(value["schema"], "dossier.detach.v1");
    assert_eq!(value["ok"], true);
    assert_eq!(value["data"]["task"], "T-001");
    assert_eq!(value["data"]["event"], "detached");
    assert_eq!(value["data"]["path"], MODEL_PATH);
    assert_eq!(value["data"]["detached"][0], MODEL_PATH);
}
