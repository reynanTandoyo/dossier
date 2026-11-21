//! Runs the built program for `delivered`: the DLV record with the checksum of each file and of
//! each file named with `--based-on`, the task and file warnings, the question a person answers,
//! the refusals of an LLM and of no terminal, and the dry run. Each test runs in its own temporary
//! project with the clock fixed at 2026-11-20T10:00:00+01:00.

mod common;

use common::Project;
use dossier::hash::sha256_hex;
use predicates::str::contains;

/// The fixed clock of these tests (the same as the other command tests).
const NOW: &str = "2026-11-20T10:00:00+01:00";

/// Invented bytes of a file that went to the customer.
const SPEC: &str = "Invented customer spec, revision 3.\n";

/// Invented bytes of a result file.
const RUN: &str = "Invented results of run 1.\n";

/// Invented bytes of a model file, and of the same file after it was changed.
const GEOMETRY: &str = "Invented geometry, first cut.\n";
const REVISED: &str = "Invented geometry, second cut.\n";

/// The checksum that the program records for `bytes` (format.md §18).
fn checksum_of(bytes: &str) -> String {
    format!("sha256:{}", sha256_hex(bytes.as_bytes()))
}

/// The exchange file of the project, as text.
fn exchange_of(project: &Project) -> String {
    project.read("DOSSIER/exchange.md")
}

/// Whether the exchange file exists in the project.
fn exchange_exists(project: &Project) -> bool {
    project.root().join("DOSSIER").join("exchange.md").exists()
}

/// Whether the journal has a change with this ID, event and detail, made by anyone.
fn journaled(project: &Project, id: &str, event: &str, detail: &str) -> bool {
    project
        .entry_journal()
        .iter()
        .any(|record| record.id == id && record.event == event && record.detail == detail)
}

/// Records an open assumption as anna: it is approved at once and stays open (R-WRITE-14).
fn assume_open(project: &Project, text: &str, affects: &str) {
    project.stdout(
        "anna",
        &[
            "assume",
            text,
            "--reason",
            "No measured profile was delivered.",
            "--affects",
            affects,
        ],
    );
}

/// Attaches `path` to `task` as anna, with the role `reference`.
fn attach_reference(project: &Project, task: &str, path: &str) {
    project.stdout("anna", &["attach", task, path, "--role", "reference"]);
}

#[test]
fn a_delivery_based_on_a_clean_task_is_written_without_a_question() {
    let project = Project::new();
    project.write("spec.pdf", SPEC);
    attach_reference(&project, "T-001", "spec.pdf");

    let stdout = project.stdout(
        "anna",
        &[
            "delivered",
            "spec.pdf",
            "--to",
            "the customer",
            "--based-on",
            "T-001",
        ],
    );
    assert_eq!(stdout, "DLV-001: delivered 1 files to the customer\n");
    assert_eq!(
        exchange_of(&project),
        format!(
            "<!-- dossier-format: 1 -->\n# Exchange of P-001\n\n## DLV-001\n\n- date: 2026-11-20\n- to: the customer\n- files: spec.pdf@{}\n- based-on: T-001\n- history: 2026-11-20T10:00:00+01:00 anna: delivered\n\n",
            checksum_of(SPEC)
        )
    );
    assert!(journaled(&project, "EXCHANGE", "delivered", "DLV-001"));
}

#[test]
fn a_file_named_with_based_on_is_stored_with_its_checksum_and_a_note_is_kept() {
    let project = Project::new();
    project.write("spec.pdf", SPEC);
    project.write("results/run1.csv", RUN);
    project.stdout("anna", &["received", "spec.pdf", "--from", "the customer"]);

    let stdout = project.stdout(
        "anna",
        &[
            "delivered",
            "spec.pdf",
            "--to",
            "the customer",
            "--based-on",
            "results/run1.csv",
            "--note",
            "first report",
        ],
    );
    assert_eq!(stdout, "DLV-001: delivered 1 files to the customer\n");

    let text = exchange_of(&project);
    let delivered = format!(
        "## DLV-001\n\n- date: 2026-11-20\n- to: the customer\n- files: spec.pdf@{}\n- based-on: results/run1.csv@{}\n- note: first report\n- history: 2026-11-20T10:00:00+01:00 anna: delivered\n\n",
        checksum_of(SPEC),
        checksum_of(RUN)
    );
    assert!(
        text.ends_with(&delivered),
        "the DLV record is at the end: {text}"
    );
    assert!(
        text.contains("## RCV-001\n"),
        "the received record stays: {text}"
    );
}

#[test]
fn the_second_record_is_dlv_002_and_the_date_is_the_one_given() {
    let project = Project::new();
    project.write("spec.pdf", SPEC);
    attach_reference(&project, "T-001", "spec.pdf");

    project.stdout(
        "anna",
        &[
            "delivered",
            "spec.pdf",
            "--to",
            "the customer",
            "--date",
            "2026-11-18",
        ],
    );
    assert!(
        exchange_of(&project).contains("## DLV-001\n\n- date: 2026-11-18\n"),
        "the date given is written"
    );

    let stdout = project.stdout(
        "anna",
        &[
            "delivered",
            "spec.pdf",
            "--to",
            "the customer",
            "--based-on",
            "T-001",
        ],
    );
    assert_eq!(stdout, "DLV-002: delivered 1 files to the customer\n");
    assert!(exchange_of(&project).contains("## DLV-002\n\n- date: 2026-11-20\n"));
}

#[test]
fn an_open_assumption_about_another_task_is_no_warning() {
    let project = Project::new();
    project.write("spec.pdf", SPEC);
    attach_reference(&project, "T-001", "spec.pdf");
    assume_open(&project, "The inlet profile is uniform.", "T-002");

    project.stdout(
        "anna",
        &[
            "delivered",
            "spec.pdf",
            "--to",
            "the customer",
            "--based-on",
            "T-001",
        ],
    );
    assert!(exchange_of(&project).contains("## DLV-001\n"));
}

#[test]
fn an_open_assumption_that_affects_the_task_is_asked_about_and_nothing_is_written_without_a_person()
{
    let project = Project::new();
    project.write("spec.pdf", SPEC);
    attach_reference(&project, "T-001", "spec.pdf");
    assume_open(&project, "The inlet profile is uniform.", "T-001");

    project
        .command("anna", NOW)
        .env_remove("DOSSIER_ANSWERS")
        .args([
            "delivered",
            "spec.pdf",
            "--to",
            "the customer",
            "--based-on",
            "T-001",
        ])
        .assert()
        .failure()
        .code(1)
        .stderr(contains("ASM-001 is open and affects T-001"));
    assert!(!exchange_exists(&project), "nothing is written");
}

#[test]
fn a_person_who_answers_no_delivers_nothing_and_a_yes_delivers() {
    let project = Project::new();
    project.write("spec.pdf", SPEC);
    attach_reference(&project, "T-001", "spec.pdf");
    assume_open(&project, "The inlet profile is uniform.", "T-001");
    let args = [
        "delivered",
        "spec.pdf",
        "--to",
        "the customer",
        "--based-on",
        "T-001",
    ];

    project
        .command("anna", NOW)
        .env("DOSSIER_ANSWERS", "no")
        .args(args)
        .assert()
        .success()
        .stdout("nothing changed\n")
        .stderr(contains("warning: ASM-001 is open and affects T-001"));
    assert!(!exchange_exists(&project), "a no writes nothing");

    project
        .command("anna", NOW)
        .env("DOSSIER_ANSWERS", "yes")
        .args(args)
        .assert()
        .success()
        .stdout("DLV-001: delivered 1 files to the customer\n");
    let text = exchange_of(&project);
    assert!(text.contains("## DLV-001\n"));
    assert!(text.contains("- based-on: T-001\n"));
    assert!(journaled(&project, "EXCHANGE", "delivered", "DLV-001"));
}

#[test]
fn an_llm_cannot_confirm_a_delivery_with_warnings_but_delivers_a_clean_one() {
    let project = Project::new();
    project.write("spec.pdf", SPEC);
    attach_reference(&project, "T-001", "spec.pdf");
    assume_open(&project, "The inlet profile is uniform.", "T-001");

    project
        .command("ben-codex", NOW)
        .env("DOSSIER_ANSWERS", "yes")
        .args([
            "delivered",
            "spec.pdf",
            "--to",
            "the customer",
            "--based-on",
            "T-001",
        ])
        .assert()
        .failure()
        .code(1)
        .stderr(contains("must confirm"))
        .stderr(contains("ASM-001 is open and affects T-001"));
    assert!(!exchange_exists(&project), "nothing is written");

    let stdout = project.stdout(
        "ben-codex",
        &["delivered", "spec.pdf", "--to", "the customer"],
    );
    assert_eq!(stdout, "DLV-001: delivered 1 files to the customer\n");
}

#[test]
fn results_made_from_a_file_that_changed_are_outdated_and_warned_about_for_the_task() {
    let project = Project::new();
    project.write("spec.pdf", SPEC);
    project.write("model/geometry.prt", GEOMETRY);
    project.write("results/run1.csv", RUN);
    project.stdout(
        "anna",
        &["attach", "T-001", "model/geometry.prt", "--role", "model"],
    );
    project.stdout(
        "anna",
        &[
            "attach",
            "T-001",
            "results/run1.csv",
            "--role",
            "result",
            "--from",
            "model/geometry.prt",
        ],
    );
    project.write("model/geometry.prt", REVISED);

    project
        .command("anna", NOW)
        .env("DOSSIER_ANSWERS", "no")
        .args([
            "delivered",
            "spec.pdf",
            "--to",
            "the customer",
            "--based-on",
            "T-001",
        ])
        .assert()
        .success()
        .stdout("nothing changed\n")
        .stderr(contains("warning: T-001: results/run1.csv is outdated"));
    assert!(!exchange_exists(&project));
}

#[test]
fn a_file_named_that_is_outdated_in_a_file_list_is_warned_about() {
    let project = Project::new();
    project.write("spec.pdf", SPEC);
    project.write("model/geometry.prt", GEOMETRY);
    project.write("results/run1.csv", RUN);
    project.stdout(
        "anna",
        &["attach", "T-001", "model/geometry.prt", "--role", "model"],
    );
    project.stdout(
        "anna",
        &[
            "attach",
            "T-001",
            "results/run1.csv",
            "--role",
            "result",
            "--from",
            "model/geometry.prt",
        ],
    );
    project.write("model/geometry.prt", REVISED);

    project
        .command("anna", NOW)
        .env("DOSSIER_ANSWERS", "no")
        .args([
            "delivered",
            "spec.pdf",
            "--to",
            "the customer",
            "--based-on",
            "results/run1.csv",
        ])
        .assert()
        .success()
        .stdout("nothing changed\n")
        .stderr(contains(
            "warning: results/run1.csv is outdated (files of T-001)",
        ));
}

#[test]
fn an_entry_of_the_task_that_rests_on_a_wrong_assumption_is_warned_about() {
    let project = Project::new();
    project.write("spec.pdf", SPEC);
    attach_reference(&project, "T-001", "spec.pdf");
    // An assumption that affects nothing named, so only the entry gives the warning.
    project.stdout(
        "anna",
        &[
            "assume",
            "The inlet profile is uniform.",
            "--reason",
            "No measured profile was delivered.",
        ],
    );
    project.stdout(
        "anna",
        &[
            "found",
            "The stall margin moves with the inlet profile.",
            "--task",
            "T-001",
        ],
    );
    project.stdout("anna", &["set", "FND-001", "--field", "rests-on=ASM-001"]);

    project
        .command("anna", NOW)
        .env("DOSSIER_ANSWERS", "no")
        .args([
            "delivered",
            "spec.pdf",
            "--to",
            "the customer",
            "--based-on",
            "T-001",
        ])
        .assert()
        .success()
        .stdout("nothing changed\n")
        .stderr(contains(
            "warning: T-001: FND-001 rests on a wrong assumption: ASM-001",
        ));
    assert!(!exchange_exists(&project));
}

#[test]
fn a_task_named_in_lower_case_is_recorded_in_upper_case_with_its_warnings_in_json() {
    let project = Project::new();
    project.write("spec.pdf", SPEC);
    attach_reference(&project, "T-001", "spec.pdf");
    assume_open(&project, "The inlet profile is uniform.", "T-001");

    let stdout = project
        .command("anna", NOW)
        .env("DOSSIER_ANSWERS", "yes")
        .args([
            "delivered",
            "spec.pdf",
            "--to",
            "the customer",
            "--based-on",
            "t-001",
            "--json",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let value: serde_json::Value = serde_json::from_slice(&stdout).expect("one line of JSON");
    assert_eq!(value["schema"], "dossier.delivered.v1");
    assert_eq!(value["ok"], true);
    assert_eq!(value["data"]["id"], "DLV-001");
    assert_eq!(value["data"]["based_on"][0]["task"], "T-001");
    assert_eq!(
        value["data"]["warnings"][0],
        "ASM-001 is open and affects T-001"
    );
    assert!(exchange_of(&project).contains("- based-on: T-001\n"));
}

#[test]
fn a_refused_option_writes_nothing() {
    let project = Project::new();
    project.write("spec.pdf", SPEC);
    project.write("results/run1.csv", RUN);

    let cases: [(&[&str], &str); 4] = [
        (
            &[
                "delivered",
                "spec.pdf",
                "--to",
                "the customer",
                "--based-on",
                "results/missing.csv",
            ],
            "results/missing.csv does not exist",
        ),
        (
            &[
                "delivered",
                "spec.pdf",
                "--to",
                "the customer",
                "--based-on",
                "results",
            ],
            "is a folder",
        ),
        (
            &["delivered", "spec.pdf", "--to", " "],
            "--to needs who the files went to",
        ),
        (
            &[
                "delivered",
                "spec.pdf",
                "--to",
                "the customer",
                "--date",
                "20.11.2026",
            ],
            "--date needs a date",
        ),
    ];
    for (args, message) in cases {
        project
            .run("anna", args)
            .failure()
            .code(1)
            .stderr(contains(message));
    }
    assert!(!exchange_exists(&project), "no refused command writes");
}

#[test]
fn a_dry_run_names_the_record_and_writes_nothing() {
    let project = Project::new();
    project.write("spec.pdf", SPEC);

    let stdout = project.stdout(
        "anna",
        &["delivered", "spec.pdf", "--to", "the customer", "--dry-run"],
    );
    assert!(stdout.starts_with("DLV-NEW: delivered 1 files to the customer"));
    assert!(!exchange_exists(&project), "a dry run writes nothing");
}
