//! Runs the built program for `add`: each kind, sections and fields, text from a file or from
//! standard input, and the refusals. Each test runs in its own temporary project folder.

mod common;

use common::Project;
use dossier::entry::{Entry, Kind, Status};
use predicates::str::contains;
use std::path::Path;

/// The fixed clock of the tests (the same as the shared project of tests/common).
const NOW: &str = "2026-11-20T10:00:00+01:00";

/// The text of the entry file with `id`.
fn text_of(project: &Project, id: &str) -> String {
    std::fs::read_to_string(project.entry_file(id)).expect("the entry file exists")
}

/// Whether the kind folder of the project has no files in it.
fn folder_is_empty(folder: &Path) -> bool {
    std::fs::read_dir(folder)
        .expect("the kind folder")
        .next()
        .is_none()
}

#[test]
fn anna_adds_a_finding_with_a_section_and_the_file_is_approved() {
    let project = Project::new();
    let stdout = project.stdout(
        "anna",
        &[
            "add",
            "finding",
            "--title",
            "Halving the time step did not help",
            "--section",
            "Observation=The mass flow still oscillated by 0.4 %.",
        ],
    );
    assert_eq!(stdout, "FND-001: added (approved)\n");

    let path = project.entry_file("FND-001");
    assert!(path.starts_with(project.root().join("DOSSIER").join("findings")));
    let text = text_of(&project, "FND-001");
    let entry = Entry::parse(&text).expect("the file is an entry");
    assert_eq!(entry.kind(), Some(Kind::Finding));
    assert_eq!(entry.title(), "Halving the time step did not help");
    assert_eq!(entry.status(), Some(Status::Approved));
    assert_eq!(entry.field("author").as_deref(), Some("anna"));
    assert_eq!(entry.field("via"), None);
    assert_eq!(
        entry.section("Observation").as_deref(),
        Some("The mass flow still oscillated by 0.4 %.")
    );
}

#[test]
fn anna_claude_adds_a_lesson_and_it_is_a_draft() {
    let project = Project::new();
    let stdout = project.stdout(
        "anna-claude",
        &[
            "add",
            "lesson",
            "--title",
            "Steady runs overpredict stall margin",
            "--section",
            "Statement=Near the stability limit, steady runs converged where the transient run stalled.",
        ],
    );
    assert_eq!(stdout, "LES-001: added (draft, waiting for approval)\n");

    let text = text_of(&project, "LES-001");
    let entry = Entry::parse(&text).expect("the file is an entry");
    assert_eq!(entry.kind(), Some(Kind::Lesson));
    assert_eq!(entry.status(), Some(Status::Draft));
    assert_eq!(entry.field("author").as_deref(), Some("anna"));
    assert_eq!(entry.field("via").as_deref(), Some("anna-claude"));
    assert_eq!(entry.field("approved"), None);
    assert_eq!(
        entry.history(),
        vec![format!("{NOW} anna via anna-claude: created")]
    );
}

#[test]
fn text_from_standard_input_arrives_unchanged_with_quotes_backticks_and_dollars() {
    let project = Project::new();
    let body = "A note before the first heading is ignored.\n\n## Observation\n\nHe said \"stop\" at `12:00` and paid $5 for $HOME.\n\n## Evidence\n\nRun 12 ended with \"$?\" and `exit 0`.\n";
    project
        .command("anna", NOW)
        .args([
            "add",
            "finding",
            "--title",
            "Quotes, backticks and dollars",
            "--stdin",
        ])
        .write_stdin(body)
        .assert()
        .success()
        .stdout(contains("FND-001: added (approved)"));

    let text = text_of(&project, "FND-001");
    assert!(
        text.contains("\n## Observation\n\nHe said \"stop\" at `12:00` and paid $5 for $HOME.\n"),
        "the observation is kept as typed: {text}"
    );
    assert!(
        text.contains("\n## Evidence\n\nRun 12 ended with \"$?\" and `exit 0`.\n"),
        "the evidence is kept as typed: {text}"
    );
    assert!(
        !text.contains("A note before"),
        "text before a heading is ignored"
    );
}

#[test]
fn a_file_whose_name_has_a_space_and_a_non_ascii_letter_gives_the_body() {
    let project = Project::new();
    project.write(
        "notizen Übersicht.md",
        "## Observation\r\n\r\nThe inlet was checked.\r\n\r\n## Evidence\n\nLog file 3.\n",
    );
    project
        .run(
            "anna",
            &[
                "add",
                "finding",
                "--title",
                "Inlet check",
                "--from",
                "notizen Übersicht.md",
            ],
        )
        .success()
        .stdout(contains("FND-001: added (approved)"));

    let text = text_of(&project, "FND-001");
    assert!(text.contains("\n## Observation\n\nThe inlet was checked.\n"));
    assert!(text.contains("\n## Evidence\n\nLog file 3.\n"));
}

#[test]
fn from_and_stdin_together_are_a_usage_error() {
    let project = Project::new();
    project.write("body.md", "## Observation\n\nSeen.\n");
    project
        .command("anna", NOW)
        .args([
            "add", "finding", "--title", "Both", "--from", "body.md", "--stdin",
        ])
        .write_stdin("## Observation\n\nSeen too.\n")
        .assert()
        .code(2);
    assert!(folder_is_empty(
        &project.root().join("DOSSIER").join("findings")
    ));
}

#[test]
fn a_field_the_tool_writes_is_refused_and_nothing_is_written() {
    let project = Project::new();
    for key in [
        "status",
        "created",
        "author",
        "via",
        "approved",
        "valid-from",
        "valid-until",
        "history",
        "Status",
    ] {
        let field = format!("{key}=approved");
        project
            .run(
                "anna",
                &[
                    "add",
                    "finding",
                    "--title",
                    "Refused",
                    "--section",
                    "Observation=Seen.",
                    "--field",
                    field.as_str(),
                ],
            )
            .code(1)
            .stderr(contains("error: this field is written by the tool"));
    }
    assert!(folder_is_empty(
        &project.root().join("DOSSIER").join("findings")
    ));
}

#[test]
fn an_unknown_kind_fails_with_exit_1_and_names_the_four_kinds() {
    let project = Project::new();
    project
        .run("anna", &["add", "note", "--title", "Not a kind"])
        .code(1)
        .stderr(contains(
            "error: unknown kind note: use decision, assumption, finding or lesson",
        ));
}

#[test]
fn without_a_name_the_exit_code_is_2_and_nothing_is_written() {
    let project = Project::new();
    project
        .run(
            "",
            &[
                "add",
                "finding",
                "--title",
                "No name",
                "--section",
                "Observation=Seen.",
            ],
        )
        .code(2)
        .stderr(contains("this command needs a name"));
    assert!(folder_is_empty(
        &project.root().join("DOSSIER").join("findings")
    ));
}

#[test]
fn a_missing_title_is_a_usage_error() {
    let project = Project::new();
    project
        .run(
            "anna",
            &["add", "finding", "--section", "Observation=Seen."],
        )
        .code(2);
}

#[test]
fn a_field_without_equals_or_with_a_bad_key_is_refused() {
    let project = Project::new();
    for field in ["machine", "bad key=1", "=1", "Mäschine=1"] {
        project
            .run(
                "anna",
                &[
                    "add",
                    "finding",
                    "--title",
                    "Bad field",
                    "--section",
                    "Observation=Seen.",
                    "--field",
                    field,
                ],
            )
            .code(1)
            .stderr(contains("error: "));
    }
    assert!(folder_is_empty(
        &project.root().join("DOSSIER").join("findings")
    ));
}

#[test]
fn task_is_upper_cased_and_fields_are_written_with_their_values() {
    let project = Project::new();
    project
        .run(
            "anna",
            &[
                "add",
                "finding",
                "--title",
                "Task and fields",
                "--task",
                "t-042",
                "--field",
                "Machine=axial compressor",
                "--field",
                "analysis=steady RANS, speedline",
                "--section",
                "Observation=Seen.",
            ],
        )
        .success();

    let text = text_of(&project, "FND-001");
    let entry = Entry::parse(&text).expect("the file is an entry");
    assert_eq!(entry.field("task").as_deref(), Some("T-042"));
    assert_eq!(entry.field("machine").as_deref(), Some("axial compressor"));
    assert_eq!(
        entry.list("analysis"),
        vec!["steady RANS".to_string(), "speedline".to_string()]
    );
}

#[test]
fn a_task_given_twice_is_refused() {
    let project = Project::new();
    project
        .run(
            "anna",
            &[
                "add",
                "finding",
                "--title",
                "Task twice",
                "--task",
                "T-042",
                "--field",
                "task=T-043",
                "--section",
                "Observation=Seen.",
            ],
        )
        .code(1)
        .stderr(contains("the task is given twice"));
    assert!(folder_is_empty(
        &project.root().join("DOSSIER").join("findings")
    ));
}

#[test]
fn a_blank_title_is_refused() {
    let project = Project::new();
    project
        .run(
            "anna",
            &[
                "add",
                "finding",
                "--title",
                "   ",
                "--section",
                "Observation=Seen.",
            ],
        )
        .code(1)
        .stderr(contains("the title is empty"));
}

#[test]
fn json_output_gives_the_id_and_the_status() {
    let project = Project::new();
    project
        .run(
            "anna",
            &[
                "add",
                "finding",
                "--title",
                "As JSON",
                "--section",
                "Observation=Seen.",
                "--json",
            ],
        )
        .success()
        .stdout(contains("\"schema\":\"dossier.add.v1\""))
        .stdout(contains("\"id\":\"FND-001\""))
        .stdout(contains("\"status\":\"approved\""));
}

#[test]
fn a_dry_run_names_the_entry_and_writes_nothing() {
    let project = Project::new();
    project
        .run(
            "anna",
            &[
                "add",
                "finding",
                "--title",
                "Dry run",
                "--section",
                "Observation=Seen.",
                "--dry-run",
            ],
        )
        .success()
        .stdout(contains("FND-NEW: added (approved)"));
    assert!(folder_is_empty(
        &project.root().join("DOSSIER").join("findings")
    ));
}
