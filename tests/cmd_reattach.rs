//! Runs the built program for `reattach`: after an intended change, a file of a task's list gets
//! its size, time and checksum again, so that the `changed since attached` mark goes (R-FILE-13);
//! the history line says what changed. A file that is not attached to the task, or is detached
//! from it, and a file that is gone from the disk are refused, and the list does not change. Each
//! test runs in its own temporary project folder with the clock fixed by `common::Project`
//! (2026-11-20T10:00:00+01:00).

mod common;

use common::Project;
use dossier::blockfile::BlockFile;
use dossier::filesum;
use dossier::journal::Record;
use dossier::paths::shown;
use predicates::str::contains;

/// The fixed clock of `common::Project`, as it appears in history and journal lines.
const WHEN: &str = "2026-11-20T10:00:00+01:00";

/// The invented contents of the model file of the tests, and the same file after an intended
/// change (a different size), and after a change of the same size.
const MODEL: &str = "invented model data\n";
const REVISED: &str = "invented model data, revised\n";
const SAME_SIZE: &str = "xnvented model data\n";

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

/// The lines of a list that a reattach does not change: everything except the size, modified
/// time, checksum and history lines.
fn kept_lines(text: &str) -> Vec<&str> {
    const CHANGED: [&str; 4] = ["- size:", "- modified:", "- checksum:", "- history:"];
    text.lines()
        .filter(|line| !CHANGED.iter().any(|prefix| line.starts_with(prefix)))
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

/// The checksum of a file of the project on the disk now, as the list records it.
fn checksum_on_disk(project: &Project, path: &str) -> String {
    filesum::checksum(&project.root().join(path)).expect("the checksum of the test file")
}

#[test]
fn reattach_records_an_intended_change_and_the_mark_goes() {
    let project = Project::new();
    attach_model(&project);
    let before = list_text(&project);
    project.write(MODEL_PATH, REVISED);

    let files = project.stdout("anna", &["files", "--task", "T-001"]);
    assert!(files.contains("[changed since attached]"), "{files}");

    // The mark names who acted, as detach does: the LLM's name here.
    let output = project.stdout("anna-claude", &["reattach", "T-001", MODEL_PATH]);
    assert_eq!(
        output,
        format!(
            "T-001: reattached {}: size {} to {}\n",
            shown(MODEL_PATH),
            MODEL.len(),
            REVISED.len()
        )
    );

    let list = list_of(&project);
    assert_eq!(list.keys(), [MODEL_PATH]);
    assert_eq!(
        list.field(MODEL_PATH, "size"),
        Some(REVISED.len().to_string())
    );
    assert_eq!(
        list.field(MODEL_PATH, "checksum"),
        Some(checksum_on_disk(&project, MODEL_PATH))
    );
    assert!(list.field(MODEL_PATH, "modified").is_some());
    assert_eq!(list.field(MODEL_PATH, "role").as_deref(), Some("model"));
    assert_eq!(
        list.field(MODEL_PATH, "note").as_deref(),
        Some("first mesh")
    );
    assert!(list.field(MODEL_PATH, "detached").is_none());
    assert_eq!(
        list.history(MODEL_PATH),
        [
            format!("{WHEN} anna: attached"),
            format!(
                "{WHEN} anna via anna-claude: reattached — size {} to {}",
                MODEL.len(),
                REVISED.len()
            ),
        ]
    );

    // Only the size, time and checksum lines and the history line change: every other line stays.
    assert_eq!(kept_lines(&list_text(&project)), kept_lines(&before));

    let files = project.stdout("anna", &["files", "--task", "T-001"]);
    assert!(!files.contains("changed since attached"), "{files}");

    let journal = list_journal(&project);
    let last = journal.last().expect("the reattach is in the journal");
    assert_eq!(last.event, "reattached");
    assert_eq!(
        last.detail,
        format!("{MODEL_PATH}: size {} to {}", MODEL.len(), REVISED.len())
    );
    assert_eq!(last.command, "reattach");
    assert_eq!(last.name, "anna-claude");
    assert_eq!(last.person, "anna");
    assert!(last.llm);
}

#[test]
fn reattach_of_a_file_with_the_same_size_says_the_checksum_changed() {
    let project = Project::new();
    attach_model(&project);
    project.write(MODEL_PATH, SAME_SIZE);

    let output = project.stdout("anna", &["reattach", "T-001", MODEL_PATH]);
    assert_eq!(
        output,
        format!(
            "T-001: reattached {}: checksum changed\n",
            shown(MODEL_PATH)
        )
    );

    let list = list_of(&project);
    assert_eq!(
        list.field(MODEL_PATH, "size"),
        Some(MODEL.len().to_string())
    );
    assert_eq!(
        list.field(MODEL_PATH, "checksum"),
        Some(checksum_on_disk(&project, MODEL_PATH))
    );
    assert_eq!(
        list.history(MODEL_PATH).last().map(String::as_str),
        Some(format!("{WHEN} anna: reattached — checksum changed").as_str())
    );
}

#[test]
fn reattach_of_an_unchanged_file_says_unchanged_and_changes_no_other_line() {
    let project = Project::new();
    attach_model(&project);
    let before = list_text(&project);

    let output = project.stdout("anna", &["reattach", "T-001", MODEL_PATH]);
    assert_eq!(
        output,
        format!("T-001: reattached {}: unchanged\n", shown(MODEL_PATH))
    );

    assert_eq!(kept_lines(&list_text(&project)), kept_lines(&before));
    assert_eq!(
        list_of(&project)
            .history(MODEL_PATH)
            .last()
            .map(String::as_str),
        Some(format!("{WHEN} anna: reattached — unchanged").as_str())
    );
}

#[test]
fn a_folder_gets_its_number_of_files_again_and_the_history_says_so() {
    let project = Project::new();
    project.write("results/run1/a.csv", "1\n");
    project.write("results/run1/b.csv", "2\n");
    project.stdout(
        "anna",
        &["attach", "T-001", "results/run1", "--role", "result"],
    );
    project.write("results/run1/c.csv", "3\n");

    let output = project.stdout("anna", &["reattach", "T-001", "results/run1"]);
    assert_eq!(
        output,
        format!(
            "T-001: reattached {}: files 2 to 3\n",
            shown("results/run1")
        )
    );

    let list = list_of(&project);
    assert_eq!(list.field("results/run1", "files").as_deref(), Some("3"));
    assert!(list.field("results/run1", "checksum").is_none());
    assert_eq!(
        list.history("results/run1").last().map(String::as_str),
        Some(format!("{WHEN} anna: reattached — files 2 to 3").as_str())
    );
    let files = project.stdout("anna", &["files", "--task", "T-001"]);
    assert!(!files.contains("changed since attached"), "{files}");
}

#[test]
fn a_path_with_spaces_and_non_ascii_letters_can_be_reattached() {
    let project = Project::new();
    let path = "Modelle ü/Lauf 1.step";
    project.write(path, MODEL);
    project.stdout("anna", &["attach", "T-001", path, "--role", "model"]);
    project.write(path, REVISED);

    project.stdout("anna", &["reattach", "T-001", path]);

    assert_eq!(
        list_of(&project).field(path, "size"),
        Some(REVISED.len().to_string())
    );
    let files = project.stdout("anna", &["files", "--task", "T-001"]);
    assert!(!files.contains("changed since attached"), "{files}");
}

#[test]
fn a_file_not_attached_or_detached_is_refused_and_the_list_does_not_change() {
    let project = Project::new();
    attach_model(&project);
    project.write("models/never-attached.step", MODEL);
    project.stdout("anna", &["detach", "T-001", MODEL_PATH]);
    let before = list_text(&project);
    let journal_before = list_journal(&project).len();

    project
        .run("anna", &["reattach", "T-001", "models/never-attached.step"])
        .failure()
        .code(1)
        .stderr(contains("is not attached to T-001"));
    project
        .run("anna", &["reattach", "T-001", MODEL_PATH])
        .failure()
        .code(1)
        .stderr(contains("is detached from T-001; attach it again first"));
    project
        .run("anna", &["reattach", "T-002", MODEL_PATH])
        .failure()
        .code(1)
        .stderr(contains("is not attached to T-002"));

    assert_eq!(list_text(&project), before);
    assert!(!project.root().join("DOSSIER/files/T-002.md").exists());
    assert_eq!(list_journal(&project).len(), journal_before);
}

#[test]
fn a_file_that_is_gone_from_the_disk_is_refused_and_the_list_does_not_change() {
    let project = Project::new();
    attach_model(&project);
    let before = list_text(&project);
    let journal_before = list_journal(&project).len();
    std::fs::remove_file(project.root().join(MODEL_PATH)).expect("the model is removed on disk");

    project
        .run("anna", &["reattach", "T-001", MODEL_PATH])
        .failure()
        .code(1)
        .stderr(contains("does not exist"));

    assert_eq!(list_text(&project), before);
    assert_eq!(list_journal(&project).len(), journal_before);
}

#[test]
fn reattach_needs_a_name_and_writes_nothing_without_one() {
    let project = Project::new();
    attach_model(&project);
    project.write(MODEL_PATH, REVISED);
    let before = list_text(&project);

    project
        .run("", &["reattach", "T-001", MODEL_PATH])
        .failure()
        .code(2)
        .stderr(contains("this command needs a name"));

    assert_eq!(list_text(&project), before);
}

#[test]
fn json_output_names_the_reattached_path_and_what_changed() {
    let project = Project::new();
    attach_model(&project);
    project.write(MODEL_PATH, REVISED);

    let output = project.stdout("anna", &["reattach", "T-001", MODEL_PATH, "--json"]);
    let value: serde_json::Value =
        serde_json::from_str(output.trim_end()).expect("one line of JSON");
    assert_eq!(value["schema"], "dossier.reattach.v1");
    assert_eq!(value["ok"], true);
    assert_eq!(value["data"]["task"], "T-001");
    assert_eq!(value["data"]["event"], "reattached");
    assert_eq!(value["data"]["path"], MODEL_PATH);
    assert_eq!(
        value["data"]["detail"],
        format!("size {} to {}", MODEL.len(), REVISED.len())
    );
}
