//! Runs the built program for `attach`: files and folders are added to a task's file list with
//! their role, note, made-from entries, size or number of files, time and checksum; a path that
//! is attached already is refused; `--moved` detaches the old path and keeps its history. Each
//! test runs in its own temporary project folder with the clock fixed by `common::Project`
//! (2026-11-20T10:00:00+01:00).

mod common;

use common::Project;
use dossier::blockfile::BlockFile;
use dossier::hash::sha256_hex;
use dossier::journal::Record;
use dossier::paths::shown;
use predicates::str::contains;

/// The fixed clock of `common::Project`, as it appears in history and journal lines.
const WHEN: &str = "2026-11-20T10:00:00+01:00";

/// Invented contents of the files of the tests.
const MODEL: &str = "invented model data\n";
const RESULT: &str = "time;value\n0;1\n1;2\n";

/// The text of the file list of task T-001.
fn list_text(project: &Project) -> String {
    project.read("DOSSIER/files/T-001.md")
}

/// The file list of task T-001, parsed.
fn list_of(project: &Project) -> BlockFile {
    BlockFile::parse(&list_text(project)).expect("the file list parses")
}

/// The checksum of some text, as the list records it (format.md §18).
fn checksum_of(text: &str) -> String {
    format!("sha256:{}", sha256_hex(text.as_bytes()))
}

/// The journal records of the file list of task T-001, oldest first.
fn list_journal(project: &Project) -> Vec<Record> {
    project
        .entry_journal()
        .into_iter()
        .filter(|record| record.id == "FILES-T-001")
        .collect()
}

/// Writes the invented model file and attaches it as a model with a note, as anna.
fn attach_model(project: &Project) {
    project.write("models/wing-v3.step", MODEL);
    project.stdout(
        "anna",
        &[
            "attach",
            "T-001",
            "models/wing-v3.step",
            "--role",
            "model",
            "--note",
            "first mesh",
        ],
    );
}

#[test]
fn attach_records_the_role_note_size_time_and_checksum_of_each_file() {
    let project = Project::new();
    attach_model(&project);
    project.write("results/run1.csv", RESULT);
    let output = project.stdout(
        "anna-claude",
        &[
            "attach",
            "T-001",
            "results/run1.csv",
            "--role",
            "result",
            "--from",
            "models/wing-v3.step",
        ],
    );
    assert_eq!(
        output,
        format!(
            "T-001: attached 1 file\n  {}: result, made from {}\n",
            shown("results/run1.csv"),
            shown("models/wing-v3.step")
        )
    );

    let list = list_of(&project);
    let model = "models/wing-v3.step";
    let result = "results/run1.csv";
    assert_eq!(list.keys(), [model, result]);

    assert_eq!(list.field(model, "role").as_deref(), Some("model"));
    assert_eq!(list.field(model, "type").as_deref(), Some("file"));
    assert_eq!(list.field(model, "note").as_deref(), Some("first mesh"));
    assert_eq!(list.field(model, "size"), Some(MODEL.len().to_string()));
    let modified = list.field(model, "modified").expect("a modification time");
    assert!(dossier::time::valid(&modified), "{modified}");
    assert_eq!(list.field(model, "checksum"), Some(checksum_of(MODEL)));
    assert!(list.field(model, "made-from").is_none());
    assert_eq!(
        list.history(model),
        ["2026-11-20T10:00:00+01:00 anna: attached"]
    );

    assert_eq!(list.field(result, "role").as_deref(), Some("result"));
    assert_eq!(list.field(result, "size"), Some(RESULT.len().to_string()));
    assert_eq!(list.field(result, "checksum"), Some(checksum_of(RESULT)));
    assert_eq!(
        list.field(result, "made-from"),
        Some(format!("{model}@{}", checksum_of(MODEL)))
    );
    assert!(list.field(result, "note").is_none());
    assert_eq!(
        list.history(result),
        ["2026-11-20T10:00:00+01:00 anna via anna-claude: attached"]
    );

    let journal = list_journal(&project);
    assert_eq!(journal.len(), 2);
    assert_eq!(journal[0].name, "anna");
    assert!(!journal[0].llm);
    assert_eq!(journal[0].event, "attached");
    assert_eq!(journal[0].detail, model);
    assert_eq!(journal[1].name, "anna-claude");
    assert!(journal[1].llm);
    assert_eq!(journal[1].time, WHEN);
    assert_eq!(journal[1].detail, result);
}

#[test]
fn a_folder_is_recorded_with_its_number_of_files_and_no_checksum() {
    let project = Project::new();
    project.write("results/run1/a.csv", "1");
    project.write("results/run1/sub/b.csv", "2");
    project.stdout(
        "anna",
        &["attach", "T-001", "results/run1", "--role", "result"],
    );

    let list = list_of(&project);
    assert_eq!(
        list.field("results/run1", "type").as_deref(),
        Some("folder")
    );
    assert_eq!(list.field("results/run1", "files").as_deref(), Some("2"));
    assert!(list.field("results/run1", "checksum").is_none());
    assert!(list.field("results/run1", "size").is_none());
}

#[test]
fn a_file_attached_already_is_refused_and_the_list_does_not_change() {
    let project = Project::new();
    attach_model(&project);
    let before = list_text(&project);

    project
        .run(
            "anna",
            &["attach", "T-001", "models/wing-v3.step", "--role", "model"],
        )
        .failure()
        .code(1)
        .stderr(contains("already attached; use dossier reattach"));

    assert_eq!(list_text(&project), before);
    assert_eq!(list_journal(&project).len(), 1);
}

#[test]
fn attach_moved_detaches_the_old_path_and_keeps_its_history() {
    let project = Project::new();
    attach_model(&project);
    project.write("results/run1.csv", RESULT);
    project.stdout(
        "anna-claude",
        &[
            "attach",
            "T-001",
            "results/run1.csv",
            "--role",
            "result",
            "--from",
            "models/wing-v3.step",
            "--note",
            "run 1 of the speedline",
        ],
    );
    std::fs::rename(
        project.root().join("results").join("run1.csv"),
        project.root().join("results").join("run1-final.csv"),
    )
    .expect("the result is renamed on disk");

    let old = "results/run1.csv";
    let new = "results/run1-final.csv";
    let model = "models/wing-v3.step";
    let output = project.stdout("anna", &["attach", "T-001", old, new, "--moved"]);
    assert_eq!(
        output,
        format!("T-001: moved {} to {}\n", shown(old), shown(new))
    );

    let list = list_of(&project);
    assert_eq!(list.keys(), [model, old, new]);

    // The old path stays in the list, detached, with its history.
    assert_eq!(list.field(old, "detached"), Some(format!("{WHEN} by anna")));
    assert_eq!(list.field(old, "role").as_deref(), Some("result"));
    assert_eq!(
        list.history(old),
        [
            "2026-11-20T10:00:00+01:00 anna via anna-claude: attached".to_string(),
            format!("{WHEN} anna: moved — to {new}"),
        ]
    );

    // The new path has the role, note and made-from of the old one, and says where it came from.
    assert_eq!(list.field(new, "role").as_deref(), Some("result"));
    assert_eq!(
        list.field(new, "note").as_deref(),
        Some("run 1 of the speedline")
    );
    assert_eq!(
        list.field(new, "made-from"),
        Some(format!("{model}@{}", checksum_of(MODEL)))
    );
    assert_eq!(list.field(new, "moved-from").as_deref(), Some(old));
    assert_eq!(list.field(new, "checksum"), Some(checksum_of(RESULT)));
    assert!(list.field(new, "detached").is_none());
    assert_eq!(
        list.history(new),
        [format!("{WHEN} anna: moved — from {old}")]
    );

    let journal = list_journal(&project);
    let last = journal.last().expect("the move is in the journal");
    assert_eq!(last.event, "moved");
    assert_eq!(last.detail, format!("{old} to {new}"));
}

#[test]
fn attach_moved_refuses_a_path_that_is_not_in_the_list() {
    let project = Project::new();
    attach_model(&project);
    project
        .run(
            "anna",
            &[
                "attach",
                "T-001",
                "models/never-attached.step",
                "models/other.step",
                "--moved",
            ],
        )
        .failure()
        .code(1)
        .stderr(contains("is not attached to T-001"));
}

#[test]
fn a_path_outside_the_project_is_refused_and_no_list_is_written() {
    let project = Project::new();
    project
        .run(
            "anna",
            &[
                "attach",
                "T-001",
                "../elsewhere/wing.step",
                "--role",
                "model",
            ],
        )
        .failure()
        .code(1)
        .stderr(contains("the path is outside the project folder"));
    assert!(!project.root().join("DOSSIER/files/T-001.md").exists());
}

#[test]
fn a_missing_path_is_refused_and_no_list_is_written() {
    let project = Project::new();
    project
        .run(
            "anna",
            &["attach", "T-001", "models/missing.step", "--role", "model"],
        )
        .failure()
        .code(1)
        .stderr(contains("models/missing.step does not exist"));
    assert!(!project.root().join("DOSSIER/files/T-001.md").exists());
}

#[test]
fn a_role_must_be_known_and_is_kept_in_the_spelling_of_the_list() {
    let project = Project::new();
    project.write("models/wing-v3.step", MODEL);
    project
        .run(
            "anna",
            &[
                "attach",
                "T-001",
                "models/wing-v3.step",
                "--role",
                "drawing",
            ],
        )
        .failure()
        .code(1)
        .stderr(contains(
            "the roles are: description, geometry, mesh, model, setup, script, result, report, reference, other",
        ));

    project.stdout(
        "anna",
        &["attach", "T-001", "models/wing-v3.step", "--role", "Model"],
    );
    assert_eq!(
        list_of(&project)
            .field("models/wing-v3.step", "role")
            .as_deref(),
        Some("model")
    );
}

#[test]
fn a_missing_role_and_moved_with_a_role_are_usage_errors() {
    let project = Project::new();
    project.write("models/wing-v3.step", MODEL);
    project
        .run("anna", &["attach", "T-001", "models/wing-v3.step"])
        .failure()
        .code(2)
        .stderr(contains("--role"));
    project
        .run(
            "anna",
            &[
                "attach",
                "T-001",
                "models/wing-v3.step",
                "models/wing-v4.step",
                "--moved",
                "--role",
                "model",
            ],
        )
        .failure()
        .code(2);
    assert!(!project.root().join("DOSSIER/files/T-001.md").exists());
}

#[test]
fn a_detached_file_is_attached_again_with_its_fields_refreshed() {
    let project = Project::new();
    project.write("models/wing-v3.step", MODEL);
    project.write(
        "DOSSIER/files/T-001.md",
        "<!-- dossier-format: 1 -->\n# Files of T-001\n\n## models/wing-v3.step\n\n- role: model\n- type: file\n- note: old note\n- size: 4\n- modified: 2026-11-01T09:00:00+01:00\n- checksum: sha256:0000\n- detached: 2026-11-10T09:00:00+01:00 by anna\n- history: 2026-11-01T09:00:00+01:00 anna: attached\n- history: 2026-11-10T09:00:00+01:00 anna: detached\n\n",
    );

    project.stdout(
        "anna",
        &["attach", "T-001", "models/wing-v3.step", "--role", "model"],
    );

    let list = list_of(&project);
    let model = "models/wing-v3.step";
    assert!(list.field(model, "detached").is_none());
    assert_eq!(list.field(model, "size"), Some(MODEL.len().to_string()));
    assert_eq!(list.field(model, "checksum"), Some(checksum_of(MODEL)));
    // The note was not given this time, so the attach gives the block none.
    assert!(list.field(model, "note").is_none());
    assert_eq!(
        list.history(model),
        vec![
            "2026-11-01T09:00:00+01:00 anna: attached",
            "2026-11-10T09:00:00+01:00 anna: detached",
            "2026-11-20T10:00:00+01:00 anna: attached",
        ]
    );
}

#[test]
fn json_output_names_each_file_with_its_role_and_what_it_was_made_from() {
    let project = Project::new();
    attach_model(&project);
    project.write("results/run1.csv", RESULT);
    let output = project.stdout(
        "anna",
        &[
            "attach",
            "T-001",
            "results/run1.csv",
            "--role",
            "result",
            "--from",
            "models/wing-v3.step",
            "--json",
        ],
    );
    let value: serde_json::Value =
        serde_json::from_str(output.trim_end()).expect("one line of JSON");
    assert_eq!(value["schema"], "dossier.attach.v1");
    assert_eq!(value["ok"], true);
    assert_eq!(value["data"]["task"], "T-001");
    assert_eq!(value["data"]["files"][0]["path"], "results/run1.csv");
    assert_eq!(value["data"]["files"][0]["role"], "result");
    assert_eq!(
        value["data"]["files"][0]["made_from"][0],
        format!("models/wing-v3.step@{}", checksum_of(MODEL))
    );
}

#[test]
fn two_paths_that_differ_only_in_case_give_a_warning_and_the_attach_still_happens() {
    let project = Project::new();
    project.write("Results/a.csv", RESULT);
    project.write("results/a.csv", RESULT);
    project.stdout(
        "anna",
        &["attach", "T-001", "Results/a.csv", "--role", "result"],
    );

    project
        .run(
            "anna",
            &["attach", "T-001", "results/a.csv", "--role", "result"],
        )
        .success()
        .stdout(contains("T-001: attached 1 file"))
        .stderr(contains("differ only in upper and lower case"));
    assert_eq!(list_of(&project).keys().len(), 2);
}

#[cfg(unix)]
#[test]
fn a_name_that_windows_cannot_have_gives_a_warning() {
    let project = Project::new();
    project.write("notes/summary.", "invented notes\n");
    project
        .run(
            "anna",
            &["attach", "T-001", "notes/summary.", "--role", "report"],
        )
        .success()
        .stderr(contains("which Windows cannot have"));
    assert_eq!(list_of(&project).keys(), ["notes/summary."]);
}
