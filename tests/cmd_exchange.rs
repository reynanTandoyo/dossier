//! Runs the built program for `exchange`: the received and delivered records in time order, the
//! files and what a delivery is based on, the flags that choose one kind, the JSON data, the
//! warning for a block that is not a record, and an empty exchange. Each test runs in its own
//! temporary project with the clock fixed at 2026-11-20T10:00:00+01:00.

mod common;

use common::Project;
use dossier::hash::sha256_hex;
use predicates::str::contains;
use serde_json::{Value, json};

/// Invented bytes of a file that came from the customer.
const SPEC: &str = "Invented customer spec, revision 3.\n";

/// Invented bytes of a file that went to the customer.
const REPORT: &str = "Invented report on run 1.\n";

/// The checksum that the program records for `bytes` (format.md §18).
fn checksum_of(bytes: &str) -> String {
    format!("sha256:{}", sha256_hex(bytes.as_bytes()))
}

/// Attaches `path` to `task` as anna, with the role `reference`.
fn attach_reference(project: &Project, task: &str, path: &str) {
    project.stdout("anna", &["attach", task, path, "--role", "reference"]);
}

/// A project with two records, written in the wrong order on purpose: DLV-001 (the report,
/// delivered on 2026-11-18, based on T-001) is written first, then RCV-001 (the spec, received on
/// 2026-11-10 with the note "revised spec"). The list must put RCV-001 first, by date.
fn two_records() -> Project {
    let project = Project::new();
    project.write("spec.pdf", SPEC);
    project.write("report.pdf", REPORT);
    attach_reference(&project, "T-001", "spec.pdf");
    project.stdout(
        "anna",
        &[
            "delivered",
            "report.pdf",
            "--to",
            "the customer",
            "--date",
            "2026-11-18",
            "--based-on",
            "T-001",
        ],
    );
    project.stdout(
        "anna",
        &[
            "received",
            "spec.pdf",
            "--from",
            "the customer",
            "--date",
            "2026-11-10",
            "--note",
            "revised spec",
        ],
    );
    project
}

/// The output of `exchange --json`, as its JSON value.
fn json_of(project: &Project, args: &[&str]) -> Value {
    let stdout = project.stdout("anna", args);
    serde_json::from_str(stdout.trim_end()).expect("one JSON line")
}

#[test]
fn records_are_listed_in_date_order_with_their_files_and_what_a_delivery_is_based_on() {
    let project = two_records();
    let stdout = project.stdout("anna", &["exchange"]);
    assert_eq!(
        stdout,
        "2026-11-10  RCV-001  from the customer\n  spec.pdf\n2026-11-18  DLV-001  to the customer\n  report.pdf\n  based on T-001\n2 records\n"
    );
}

#[test]
fn delivered_shows_only_the_delivered_record() {
    let project = two_records();
    let stdout = project.stdout("anna", &["exchange", "--delivered"]);
    assert_eq!(
        stdout,
        "2026-11-18  DLV-001  to the customer\n  report.pdf\n  based on T-001\n1 record\n"
    );
}

#[test]
fn received_shows_only_the_received_record() {
    let project = two_records();
    let stdout = project.stdout("anna", &["exchange", "--received"]);
    assert_eq!(
        stdout,
        "2026-11-10  RCV-001  from the customer\n  spec.pdf\n1 record\n"
    );
}

#[test]
fn both_flags_show_every_record_like_no_flag() {
    let project = two_records();
    let both = project.stdout("anna", &["exchange", "--received", "--delivered"]);
    assert_eq!(both, project.stdout("anna", &["exchange"]));
    assert!(both.ends_with("2 records\n"), "{both}");
}

#[test]
fn json_has_one_object_per_record_with_its_files_and_based_on() {
    let project = two_records();
    let value = json_of(&project, &["exchange", "--json"]);
    assert_eq!(value["ok"], true);
    assert_eq!(value["warnings"], json!([]));

    let data = value["data"].as_array().expect("the data is a list");
    assert_eq!(data.len(), 2);
    assert_eq!(
        data[0],
        json!({
            "key": "RCV-001",
            "date": "2026-11-10",
            "direction": "received",
            "who": "the customer",
            "files": [{"path": "spec.pdf", "checksum": checksum_of(SPEC)}],
            "based_on": [],
            "note": "revised spec",
        })
    );
    assert_eq!(
        data[1],
        json!({
            "key": "DLV-001",
            "date": "2026-11-18",
            "direction": "delivered",
            "who": "the customer",
            "files": [{"path": "report.pdf", "checksum": checksum_of(REPORT)}],
            "based_on": [{"task": "T-001"}],
            "note": null,
        })
    );
}

#[test]
fn json_with_delivered_has_only_the_delivered_record() {
    let project = two_records();
    let value = json_of(&project, &["exchange", "--delivered", "--json"]);
    let data = value["data"].as_array().expect("the data is a list");
    assert_eq!(data.len(), 1);
    assert_eq!(data[0]["key"], "DLV-001");
}

#[test]
fn a_path_based_on_and_names_with_spaces_and_non_ascii_letters_are_listed_by_their_path() {
    let project = Project::new();
    project.write("Prüf bericht.pdf", REPORT);
    project.write("results/run 1.csv", SPEC);
    project.stdout(
        "anna",
        &[
            "delivered",
            "Prüf bericht.pdf",
            "--to",
            "the customer",
            "--date",
            "2026-11-12",
            "--based-on",
            "results/run 1.csv",
        ],
    );

    let stdout = project.stdout("anna", &["exchange"]);
    assert_eq!(
        stdout,
        "2026-11-12  DLV-001  to the customer\n  Prüf bericht.pdf\n  based on results/run 1.csv\n1 record\n"
    );
    let value = json_of(&project, &["exchange", "--json"]);
    assert_eq!(
        value["data"][0]["based_on"],
        json!([{"path": "results/run 1.csv", "checksum": checksum_of(SPEC)}])
    );
}

#[test]
fn an_empty_exchange_lists_no_records_needs_no_name_and_writes_nothing() {
    let project = Project::new();
    let assert = project.run("", &["exchange"]).success();
    let stdout = String::from_utf8(assert.get_output().stdout.clone()).expect("UTF-8");
    assert_eq!(stdout, "0 records\n");
    assert!(
        !project.root().join("DOSSIER").join("exchange.md").exists(),
        "reading must not create exchange.md"
    );

    let value = json_of(&project, &["exchange", "--json"]);
    assert_eq!(value["data"], json!([]));
}

#[test]
fn records_of_one_date_keep_key_order_and_numbers_are_read_as_numbers() {
    let project = Project::new();
    project.write(
        "DOSSIER/exchange.md",
        "<!-- dossier-format: 1 -->\n# Exchange of P-001\n\n## RCV-10\n\n- date: 2026-11-10\n- from: the customer\n- history: 2026-11-10T09:00:00+01:00 anna: received\n\n## RCV-9\n\n- date: 2026-11-10\n- from: the customer\n- history: 2026-11-10T09:05:00+01:00 anna: received\n\n## DLV-1\n\n- date: 2026-11-10\n- to: the customer\n- history: 2026-11-10T16:00:00+01:00 anna: delivered\n",
    );
    let stdout = project.stdout("anna", &["exchange"]);
    assert_eq!(
        stdout,
        "2026-11-10  DLV-1  to the customer\n2026-11-10  RCV-9  from the customer\n2026-11-10  RCV-10  from the customer\n3 records\n"
    );
}

#[test]
fn a_block_that_is_not_a_record_is_left_out_with_a_warning() {
    let project = Project::new();
    project.write(
        "DOSSIER/exchange.md",
        "<!-- dossier-format: 1 -->\n# Exchange of P-001\n\n## NOTE-1\n\n- date: 2026-11-01\n\n## RCV-001\n\n- date: 2026-11-10\n- from: the customer\n- history: 2026-11-10T09:00:00+01:00 anna: received\n",
    );
    project
        .run("anna", &["exchange"])
        .success()
        .stdout("2026-11-10  RCV-001  from the customer\n1 record\n")
        .stderr(contains(
            "warning: DOSSIER/exchange.md: NOTE-1 is not a received or delivered record; it is left out",
        ));
}
