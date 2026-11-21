//! Runs the built program for `received`: the RCV record with the checksum of each file, the
//! attachment to a task, the role, the date, the refusals, and the dry run. Each test runs in its
//! own temporary project with the clock fixed at 2026-11-20T10:00:00+01:00.

mod common;

use common::Project;
use dossier::blockfile::BlockFile;
use dossier::hash::sha256_hex;
use predicates::str::contains;

/// Invented bytes of a file that came from the customer.
const SPEC: &str = "Invented customer spec, revision 3.\n";

/// Invented bytes of a second file from the customer.
const NOTES: &str = "Invented notes on the revision.\n";

/// The checksum that the program records for `bytes` (format.md §18).
fn checksum_of(bytes: &str) -> String {
    format!("sha256:{}", sha256_hex(bytes.as_bytes()))
}

/// The file list of `task`, parsed.
fn list_of(project: &Project, task: &str) -> BlockFile {
    let text = project.read(&format!("DOSSIER/files/{task}.md"));
    BlockFile::parse(&text).expect("the file list parses")
}

/// Whether the journal has a change with this ID, event and detail, made by anyone.
fn journaled(project: &Project, id: &str, event: &str, detail: &str) -> bool {
    project
        .entry_journal()
        .iter()
        .any(|record| record.id == id && record.event == event && record.detail == detail)
}

#[test]
fn the_first_record_is_rcv_001_with_the_checksum_and_the_file_is_attached() {
    let project = Project::new();
    project.write("spec.pdf", SPEC);
    let stdout = project.stdout(
        "anna",
        &[
            "received",
            "spec.pdf",
            "--from",
            "the customer",
            "--task",
            "T-001",
        ],
    );
    assert_eq!(stdout, "RCV-001: received 1 files from the customer\n");

    let checksum = checksum_of(SPEC);
    assert_eq!(
        project.read("DOSSIER/exchange.md"),
        format!(
            "<!-- dossier-format: 1 -->\n# Exchange of P-001\n\n## RCV-001\n\n- date: 2026-11-20\n- from: the customer\n- files: spec.pdf@{checksum}\n- history: 2026-11-20T10:00:00+01:00 anna: received\n\n"
        )
    );

    let list = list_of(&project, "T-001");
    assert!(
        project
            .read("DOSSIER/files/T-001.md")
            .starts_with("<!-- dossier-format: 1 -->\n# Files of T-001\n\n## spec.pdf\n\n")
    );
    assert_eq!(list.keys(), vec!["spec.pdf".to_string()]);
    assert_eq!(list.field("spec.pdf", "role").as_deref(), Some("reference"));
    assert_eq!(list.field("spec.pdf", "type").as_deref(), Some("file"));
    assert_eq!(list.field("spec.pdf", "size"), Some(SPEC.len().to_string()));
    assert_eq!(list.field("spec.pdf", "checksum"), Some(checksum));
    assert!(
        list.field("spec.pdf", "modified")
            .is_some_and(|modified| !modified.is_empty())
    );
    assert_eq!(
        list.history("spec.pdf"),
        vec!["2026-11-20T10:00:00+01:00 anna: attached".to_string()]
    );

    assert!(journaled(&project, "EXCHANGE", "received", "RCV-001"));
    assert!(journaled(&project, "FILES-T-001", "attached", "spec.pdf"));
}

#[test]
fn a_second_record_is_rcv_002_below_the_first_and_an_llm_is_named_with_via() {
    let project = Project::new();
    project.write("spec.pdf", SPEC);
    project.write("notes.txt", NOTES);
    project.stdout(
        "anna",
        &[
            "received",
            "spec.pdf",
            "--from",
            "the customer",
            "--task",
            "T-001",
        ],
    );
    let first = project.read("DOSSIER/exchange.md");

    let stdout = project.stdout(
        "ben-codex",
        &[
            "received",
            "notes.txt",
            "--from",
            "the customer",
            "--note",
            "revised notes",
        ],
    );
    assert_eq!(stdout, "RCV-002: received 1 files from the customer\n");

    let text = project.read("DOSSIER/exchange.md");
    let second = format!(
        "## RCV-002\n\n- date: 2026-11-20\n- from: the customer\n- files: notes.txt@{}\n- note: revised notes\n- history: 2026-11-20T10:00:00+01:00 ben via ben-codex: received\n\n",
        checksum_of(NOTES)
    );
    assert_eq!(text, format!("{first}{second}"));
    assert!(!project.read("DOSSIER/files/T-001.md").contains("notes.txt"));
}

#[test]
fn the_date_is_today_unless_a_date_is_given_and_a_bad_date_writes_nothing() {
    let project = Project::new();
    project.write("spec.pdf", SPEC);
    project.stdout("anna", &["received", "spec.pdf", "--from", "the customer"]);
    assert!(
        project
            .read("DOSSIER/exchange.md")
            .contains("## RCV-001\n\n- date: 2026-11-20\n")
    );

    for date in ["2026-13-40", "20.11.2026"] {
        project
            .run(
                "anna",
                &[
                    "received",
                    "spec.pdf",
                    "--from",
                    "the customer",
                    "--date",
                    date,
                ],
            )
            .failure()
            .code(1)
            .stderr(contains("--date needs a date"));
    }
    assert!(!project.read("DOSSIER/exchange.md").contains("RCV-002"));

    project.stdout(
        "anna",
        &[
            "received",
            "spec.pdf",
            "--from",
            "the customer",
            "--date",
            "2026-11-02",
        ],
    );
    assert!(
        project
            .read("DOSSIER/exchange.md")
            .contains("## RCV-002\n\n- date: 2026-11-02\n")
    );
}

#[test]
fn the_role_is_reference_unless_description_is_given_and_other_roles_are_refused() {
    let project = Project::new();
    project.write("notes.txt", NOTES);
    project.write("spec.pdf", SPEC);
    project.stdout(
        "anna",
        &[
            "received",
            "notes.txt",
            "--from",
            "the customer",
            "--task",
            "T-002",
            "--role",
            "description",
        ],
    );
    assert_eq!(
        list_of(&project, "T-002")
            .field("notes.txt", "role")
            .as_deref(),
        Some("description")
    );

    project
        .run(
            "anna",
            &[
                "received",
                "spec.pdf",
                "--from",
                "the customer",
                "--task",
                "T-003",
                "--role",
                "result",
            ],
        )
        .failure()
        .code(1)
        .stderr(contains("--role accepts description or reference"));
    assert!(!project.read("DOSSIER/exchange.md").contains("RCV-002"));
    assert!(!project.root().join("DOSSIER/files/T-003.md").exists());
}

#[test]
fn a_file_the_task_has_already_is_left_as_it_is() {
    let project = Project::new();
    project.write("spec.pdf", SPEC);
    project.stdout(
        "anna",
        &[
            "received",
            "spec.pdf",
            "--from",
            "the customer",
            "--task",
            "T-001",
        ],
    );
    let list = project.read("DOSSIER/files/T-001.md");

    let stdout = project.stdout(
        "anna",
        &[
            "received",
            "spec.pdf",
            "--from",
            "the customer",
            "--task",
            "T-001",
        ],
    );
    assert_eq!(stdout, "RCV-002: received 1 files from the customer\n");
    assert!(project.read("DOSSIER/exchange.md").contains("## RCV-002\n"));
    assert_eq!(project.read("DOSSIER/files/T-001.md"), list);
    assert_eq!(
        project
            .entry_journal()
            .iter()
            .filter(|record| record.id == "FILES-T-001")
            .count(),
        1
    );
}

#[test]
fn several_files_go_in_one_record_and_are_attached_in_one_change() {
    let project = Project::new();
    project.write("spec.pdf", SPEC);
    project.write("notes.txt", NOTES);
    let stdout = project.stdout(
        "anna",
        &[
            "received",
            "spec.pdf",
            "notes.txt",
            "--from",
            "the customer",
            "--task",
            "T-001",
        ],
    );
    assert_eq!(stdout, "RCV-001: received 2 files from the customer\n");

    let exchange = project.read("DOSSIER/exchange.md");
    assert!(exchange.contains(&format!(
        "- files: spec.pdf@{}, notes.txt@{}\n",
        checksum_of(SPEC),
        checksum_of(NOTES)
    )));
    assert_eq!(
        list_of(&project, "T-001").keys(),
        vec!["spec.pdf".to_string(), "notes.txt".to_string()]
    );
    assert!(journaled(
        &project,
        "FILES-T-001",
        "attached",
        "spec.pdf, notes.txt"
    ));
}

#[test]
fn a_missing_file_a_folder_or_a_path_outside_the_project_writes_nothing() {
    let project = Project::new();
    project.write("inbox/readme.txt", "not received\n");

    project
        .run(
            "anna",
            &["received", "missing.pdf", "--from", "the customer"],
        )
        .failure()
        .code(1)
        .stderr(contains("missing.pdf does not exist"));
    project
        .run("anna", &["received", "inbox", "--from", "the customer"])
        .failure()
        .code(1)
        .stderr(contains("is a folder"));
    project
        .run(
            "anna",
            &["received", "../outside.pdf", "--from", "the customer"],
        )
        .failure()
        .code(1)
        .stderr(contains("outside the project folder"));

    assert!(!project.root().join("DOSSIER/exchange.md").exists());
    assert!(!project.root().join("DOSSIER/files").exists());
}

#[test]
fn a_path_with_spaces_and_letters_outside_ascii_works() {
    let project = Project::new();
    project.write("Berichte Ü/Prüf Bericht.pdf", SPEC);
    project.stdout(
        "anna",
        &[
            "received",
            "Berichte Ü/Prüf Bericht.pdf",
            "--from",
            "the customer",
            "--task",
            "T-004",
        ],
    );
    assert!(project.read("DOSSIER/exchange.md").contains(&format!(
        "- files: Berichte Ü/Prüf Bericht.pdf@{}\n",
        checksum_of(SPEC)
    )));
    assert_eq!(
        list_of(&project, "T-004").keys(),
        vec!["Berichte Ü/Prüf Bericht.pdf".to_string()]
    );
}

#[test]
fn a_name_a_file_and_a_sender_are_needed() {
    let project = Project::new();
    project.write("spec.pdf", SPEC);

    project
        .run("", &["received", "spec.pdf", "--from", "the customer"])
        .failure()
        .code(2)
        .stderr(contains("this command needs a name"));
    project
        .run("anna", &["received", "spec.pdf"])
        .failure()
        .code(2);
    project
        .run("anna", &["received", "--from", "the customer"])
        .failure()
        .code(2);
    project
        .run("anna", &["received", "spec.pdf", "--from", "  "])
        .failure()
        .code(1)
        .stderr(contains("--from needs who sent the files"));

    assert!(!project.root().join("DOSSIER/exchange.md").exists());
}

#[test]
fn a_task_must_be_an_id_so_that_its_list_stays_in_the_files_folder() {
    let project = Project::new();
    project.write("spec.pdf", SPEC);
    project
        .run(
            "anna",
            &[
                "received",
                "spec.pdf",
                "--from",
                "the customer",
                "--task",
                "../T-001",
            ],
        )
        .failure()
        .code(1)
        .stderr(contains("--task needs a task ID"));

    assert!(!project.root().join("DOSSIER/exchange.md").exists());
    assert!(!project.root().join("DOSSIER/files").exists());
}

#[test]
fn a_dry_run_writes_nothing() {
    let project = Project::new();
    project.write("spec.pdf", SPEC);
    project
        .run(
            "anna",
            &[
                "received",
                "spec.pdf",
                "--from",
                "the customer",
                "--task",
                "T-001",
                "--dry-run",
            ],
        )
        .success()
        .stdout("RCV-NEW: received 1 files from the customer\n");

    assert!(!project.root().join("DOSSIER/exchange.md").exists());
    assert!(!project.root().join("DOSSIER/files").exists());
    assert!(
        !project
            .entry_journal()
            .iter()
            .any(|record| record.id == "EXCHANGE")
    );
}
