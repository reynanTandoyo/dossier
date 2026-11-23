//! Runs the built program for `return`: `--check` says whether each taken file is still as it was
//! taken (size, time and checksum) and writes nothing; `--done` records the new version and ends the
//! take; `--abandon` ends the take with no new version; only the person who took a file returns it,
//! unless `--force`. Each test runs in its own temporary project folder with the clock fixed by
//! `common::Project` (2026-11-20T10:00:00+01:00).

mod common;

use common::Project;
use dossier::blockfile::BlockFile;
use dossier::filesum;
use dossier::hash::sha256_hex;
use dossier::journal::Record;
use dossier::paths::shown;
use predicates::str::contains;
use std::time::{Duration, UNIX_EPOCH};

/// The fixed clock of `common::Project`, as it appears in history and journal lines.
const WHEN: &str = "2026-11-20T10:00:00+01:00";

/// The stored path of the taken file of most tests.
const PATH: &str = "runs/case-a.dat";

/// Another file of the tests.
const OTHER: &str = "runs/case-b.dat";

/// Invented contents of the file as it is taken.
const DATA: &str = "invented simulation data\n";

/// The same size as DATA, with other content.
const SAME_SIZE: &str = "invented simulation DATA\n";

/// A longer version of the file: another size and other content.
const LONGER: &str = "invented simulation data, second version\n";

/// The modification time the tests give a file (seconds since 1970); a later one differs only in
/// time.
const TAKEN_AT: u64 = 1_700_000_000;

/// The text of the file of taken files of the project.
fn taken_text(project: &Project) -> String {
    project.read("DOSSIER/taken.md")
}

/// The file of taken files, parsed.
fn taken_of(project: &Project) -> BlockFile {
    BlockFile::parse(&taken_text(project)).expect("the taken files parse")
}

/// The checksum of some text, as the record holds it (format.md §18).
fn checksum_of(text: &str) -> String {
    format!("sha256:{}", sha256_hex(text.as_bytes()))
}

/// The journal records of the taken files, oldest first.
fn taken_journal(project: &Project) -> Vec<Record> {
    project
        .entry_journal()
        .into_iter()
        .filter(|record| record.id == "TAKEN")
        .collect()
}

/// The last journal record of the taken files.
fn last_taken_record(project: &Project) -> Record {
    taken_journal(project)
        .pop()
        .expect("a journal line for the taken files")
}

/// Sets the modification time of a file of the project to `seconds` after 1970.
fn set_modified(project: &Project, relative: &str, seconds: u64) {
    let path = project.root().join(relative);
    let file = std::fs::OpenOptions::new()
        .write(true)
        .open(&path)
        .expect("open the test file");
    file.set_modified(UNIX_EPOCH + Duration::from_secs(seconds))
        .expect("set the modification time");
}

/// Writes `PATH` with DATA, gives it the fixed time and takes it as anna for T-001.
fn take_as_anna(project: &Project) {
    project.write(PATH, DATA);
    set_modified(project, PATH, TAKEN_AT);
    project.stdout("anna", &["take", PATH, "--task", "T-001"]);
}

#[test]
fn check_after_take_is_unchanged_and_writes_nothing() {
    let project = Project::new();
    take_as_anna(&project);
    let before = taken_text(&project);

    let output = project.stdout("anna", &["return", PATH, "--check"]);
    assert_eq!(output, "unchanged: 1 file\n");
    assert_eq!(taken_text(&project), before, "taken.md is unchanged");
    assert_eq!(
        taken_journal(&project).len(),
        1,
        "a check writes no journal line"
    );
}

#[test]
fn check_names_a_file_whose_content_changed_and_nothing_else() {
    let project = Project::new();
    take_as_anna(&project);
    // Same size, same time, other content: only the content differs.
    project.write(PATH, SAME_SIZE);
    set_modified(&project, PATH, TAKEN_AT);

    project
        .run("anna", &["return", PATH, "--check"])
        .code(1)
        .stderr(contains(format!("{}: content\n", shown(PATH))));
}

#[test]
fn check_names_size_time_and_content_when_all_three_differ() {
    let project = Project::new();
    take_as_anna(&project);
    project.write(PATH, LONGER);
    set_modified(&project, PATH, TAKEN_AT + 60);

    project
        .run("anna", &["return", PATH, "--check"])
        .code(1)
        .stderr(contains(format!("{}: size, time, content", shown(PATH))));
}

#[test]
fn check_names_a_file_whose_only_change_is_its_time() {
    let project = Project::new();
    take_as_anna(&project);
    set_modified(&project, PATH, TAKEN_AT + 60);

    project
        .run("anna", &["return", PATH, "--check"])
        .code(1)
        .stderr(contains(format!("{}: time\n", shown(PATH))));
}

#[test]
fn check_lists_only_the_files_that_changed() {
    let project = Project::new();
    take_as_anna(&project);
    project.write(OTHER, DATA);
    set_modified(&project, OTHER, TAKEN_AT);
    project.stdout("anna", &["take", OTHER, "--task", "T-001"]);
    project.write(OTHER, LONGER);
    set_modified(&project, OTHER, TAKEN_AT);

    let assert = project
        .run("anna", &["return", PATH, OTHER, "--check"])
        .code(1);
    let stderr = String::from_utf8_lossy(&assert.get_output().stderr).to_string();
    assert!(
        stderr.contains(&format!("{}: size, content", shown(OTHER))),
        "{stderr}"
    );
    assert!(!stderr.contains(&shown(PATH)), "{stderr}");
}

#[test]
fn check_counts_a_missing_file_as_changed() {
    let project = Project::new();
    take_as_anna(&project);
    std::fs::remove_file(project.root().join(PATH)).expect("remove the test file");

    project
        .run("anna", &["return", PATH, "--check"])
        .code(1)
        .stderr(contains(format!("{}: missing\n", shown(PATH))));
}

#[test]
fn check_refuses_a_file_that_is_not_taken() {
    let project = Project::new();
    project.write(OTHER, DATA);

    project
        .run("anna", &["return", OTHER, "--check"])
        .code(1)
        .stderr(contains(format!("{} is not taken", shown(OTHER))));
}

#[test]
fn done_records_the_new_version_and_ends_the_take() {
    let project = Project::new();
    take_as_anna(&project);
    project.write(PATH, LONGER);

    let output = project.stdout("anna", &["return", PATH, "--done"]);
    assert_eq!(
        output,
        format!(
            "returned: 1 file\n  {}  {}\n",
            shown(PATH),
            checksum_of(LONGER)
        )
    );

    let file = taken_of(&project);
    assert!(file.field(PATH, "taken").is_none(), "the take is ended");
    assert!(file.field(PATH, "forced").is_none());
    assert_eq!(file.field(PATH, "size"), Some(LONGER.len().to_string()));
    let stamp = filesum::stamp(&project.root().join(PATH)).expect("the stamp");
    assert_eq!(file.field(PATH, "modified"), Some(stamp.modified));
    assert_eq!(file.field(PATH, "checksum"), Some(checksum_of(LONGER)));
    assert_eq!(
        file.history(PATH),
        [
            format!("{WHEN} anna: taken — T-001 {}", checksum_of(DATA)),
            format!("{WHEN} anna: returned — {}", checksum_of(LONGER)),
        ]
    );

    let record = last_taken_record(&project);
    assert_eq!(record.name, "anna");
    assert_eq!(record.event, "returned");
    assert_eq!(record.detail, PATH);
}

#[test]
fn a_check_after_done_refuses_because_the_file_is_no_longer_taken() {
    let project = Project::new();
    take_as_anna(&project);
    project.write(PATH, LONGER);
    project.stdout("anna", &["return", PATH, "--done"]);

    project
        .run("anna", &["return", PATH, "--check"])
        .code(1)
        .stderr(contains(format!("{} is not taken", shown(PATH))));
}

#[test]
fn a_second_done_is_refused_and_nothing_is_written() {
    let project = Project::new();
    take_as_anna(&project);
    project.write(PATH, LONGER);
    project.stdout("anna", &["return", PATH, "--done"]);
    let before = taken_text(&project);

    project
        .run("anna", &["return", PATH, "--done"])
        .code(1)
        .stderr(contains(format!("{} is not taken", shown(PATH))));
    assert_eq!(taken_text(&project), before, "taken.md is unchanged");
    assert_eq!(
        taken_journal(&project).len(),
        2,
        "no journal line for the refusal"
    );
}

#[test]
fn ben_cannot_return_annas_file_without_force() {
    let project = Project::new();
    take_as_anna(&project);
    project.write(PATH, LONGER);
    let before = taken_text(&project);

    for mode in ["--done", "--abandon"] {
        project
            .run("ben", &["return", PATH, mode])
            .code(1)
            .stderr(contains(format!(
                "{} is taken by anna for T-001 since {WHEN}; only they return it, or add --force",
                shown(PATH)
            )));
    }
    assert_eq!(taken_text(&project), before, "taken.md is unchanged");
}

#[test]
fn ben_with_force_returns_annas_file_and_the_history_says_who() {
    let project = Project::new();
    take_as_anna(&project);
    project.write(PATH, LONGER);

    let output = project.stdout("ben", &["return", PATH, "--done", "--force"]);
    assert_eq!(
        output,
        format!(
            "returned: 1 file\n  {}  {}\n",
            shown(PATH),
            checksum_of(LONGER)
        )
    );

    let file = taken_of(&project);
    assert!(file.field(PATH, "taken").is_none());
    assert_eq!(
        file.history(PATH)[1],
        format!("{WHEN} ben: returned — {}", checksum_of(LONGER))
    );
    assert_eq!(last_taken_record(&project).name, "ben");
}

#[test]
fn abandon_ends_the_take_and_keeps_the_old_checksum() {
    let project = Project::new();
    take_as_anna(&project);
    project.write(PATH, LONGER);

    let output = project.stdout("anna", &["return", PATH, "--abandon"]);
    assert_eq!(
        output,
        format!(
            "abandoned: 1 file\n  {}  {}\n",
            shown(PATH),
            checksum_of(DATA)
        )
    );

    let file = taken_of(&project);
    assert!(file.field(PATH, "taken").is_none(), "the take is ended");
    assert_eq!(file.field(PATH, "size"), Some(DATA.len().to_string()));
    assert_eq!(file.field(PATH, "checksum"), Some(checksum_of(DATA)));
    assert_eq!(file.history(PATH)[1], format!("{WHEN} anna: abandoned"));

    let record = last_taken_record(&project);
    assert_eq!(record.event, "abandoned");
    assert_eq!(record.detail, PATH);
}

#[test]
fn abandon_needs_no_file_on_disk() {
    let project = Project::new();
    take_as_anna(&project);
    std::fs::remove_file(project.root().join(PATH)).expect("remove the test file");

    let output = project.stdout("anna", &["return", PATH, "--abandon"]);
    assert_eq!(
        output,
        format!(
            "abandoned: 1 file\n  {}  {}\n",
            shown(PATH),
            checksum_of(DATA)
        )
    );
}

#[test]
fn done_of_a_missing_file_is_refused_and_nothing_is_written() {
    let project = Project::new();
    take_as_anna(&project);
    std::fs::remove_file(project.root().join(PATH)).expect("remove the test file");
    let before = taken_text(&project);

    project
        .run("anna", &["return", PATH, "--done"])
        .code(1)
        .stderr(contains(format!("{} does not exist", shown(PATH))));
    assert_eq!(taken_text(&project), before, "taken.md is unchanged");
}

#[test]
fn two_modes_or_no_mode_is_a_usage_error_with_exit_2() {
    let project = Project::new();
    take_as_anna(&project);

    project
        .run("anna", &["return", PATH, "--check", "--done"])
        .code(2)
        .stderr(contains("give exactly one of --check, --done or --abandon"));
    project
        .run("anna", &["return", PATH])
        .code(2)
        .stderr(contains("give exactly one of --check, --done or --abandon"));
    assert_eq!(taken_journal(&project).len(), 1, "nothing is written");
}

#[test]
fn json_output_of_done_gives_the_event_and_each_file() {
    let project = Project::new();
    take_as_anna(&project);
    project.write(PATH, LONGER);

    let output = project.stdout("anna", &["return", PATH, "--done", "--json"]);
    let value: serde_json::Value =
        serde_json::from_str(output.trim_end()).expect("one line of JSON");
    assert_eq!(value["schema"], "dossier.return.v1");
    assert_eq!(value["ok"], true);
    assert_eq!(value["data"]["event"], "returned");
    assert_eq!(value["data"]["files"][0]["path"], PATH);
    assert_eq!(value["data"]["files"][0]["checksum"], checksum_of(LONGER));
}

#[test]
fn a_path_with_a_space_and_a_non_ascii_letter_is_checked_and_returned() {
    let project = Project::new();
    let path = "Läufe/Lauf 01 — Entwurf.dat";
    project.write(path, DATA);
    project.stdout("anna", &["take", path, "--task", "T-001"]);

    assert_eq!(
        project.stdout("anna", &["return", path, "--check"]),
        "unchanged: 1 file\n"
    );
    project.write(path, LONGER);
    let output = project.stdout("anna", &["return", path, "--done"]);
    assert_eq!(
        output,
        format!(
            "returned: 1 file\n  {}  {}\n",
            shown(path),
            checksum_of(LONGER)
        )
    );
    assert_eq!(taken_of(&project).keys(), [path]);
}
