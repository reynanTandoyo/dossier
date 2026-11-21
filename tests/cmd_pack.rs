//! Runs the built program for `pack` and `unpack`: the archive holds the whole dossier with its
//! state folder, unpacking restores it identically, and the refusals hold. Each test runs in its
//! own temporary project with the clock fixed at 2026-11-20T10:00:00+01:00.

mod common;

use common::Project;
use predicates::str::contains;
use std::fs;
use std::path::{Path, PathBuf};

/// Two entries: a decision by anna and a finding by anna-claude, so that the journal has lines.
fn two_entries(project: &Project) {
    project
        .run(
            "anna",
            &[
                "decide",
                "Use the transient runs for the stall margin.",
                "--reason",
                "The steady runs converge where the transient run already stalls.",
            ],
        )
        .success();
    project
        .run(
            "anna-claude",
            &[
                "found",
                "The mass flow still oscillated by 0.4 %.",
                "--evidence",
                "Runs 12 and 13, monitor plots.",
            ],
        )
        .success();
}

/// The files that must come back identical: dossier.toml, every entry file and the journal.
fn identical_files(project: &Project) -> Vec<PathBuf> {
    let mut files = vec![
        PathBuf::from("DOSSIER").join("dossier.toml"),
        PathBuf::from("DOSSIER")
            .join(".state")
            .join("journal.jsonl"),
    ];
    for id in ["DEC-001", "FND-001"] {
        let entry = project.entry_file(id);
        let relative = entry
            .strip_prefix(project.root())
            .expect("the entry is inside the project")
            .to_path_buf();
        files.push(relative);
    }
    files
}

/// The bytes of `relative` below `folder`.
fn bytes_in(folder: &Path, relative: &Path) -> Vec<u8> {
    let path = folder.join(relative);
    fs::read(&path).unwrap_or_else(|error| panic!("reading {}: {error}", path.display()))
}

#[test]
fn pack_after_two_entries_writes_the_archive_file() {
    let project = Project::new();
    two_entries(&project);
    project
        .run("anna", &["pack", "--to", "backup.tar"])
        .success()
        .stdout(contains("packed "))
        .stdout(contains("into backup.tar"));
    assert!(project.root().join("backup.tar").is_file());
}

#[test]
fn pack_without_a_name_also_works() {
    let project = Project::new();
    two_entries(&project);
    project
        .run("", &["pack", "--to", "backup.tar"])
        .success()
        .stdout(contains("packed "));
    assert!(project.root().join("backup.tar").is_file());
}

#[test]
fn unpack_restores_the_dossier_byte_for_byte() {
    let project = Project::new();
    two_entries(&project);
    project
        .run("anna", &["pack", "--to", "backup.tar"])
        .success();
    project
        .run("", &["unpack", "backup.tar", "--into", "restored"])
        .success()
        .stdout(contains("unpacked "))
        .stdout(contains("restored/DOSSIER"))
        .stdout(contains("check it with: dossier check"));

    let restored = project.root().join("restored");
    for relative in identical_files(&project) {
        assert_eq!(
            bytes_in(&restored, &relative),
            bytes_in(project.root(), &relative),
            "{} differs after unpack",
            relative.display()
        );
    }
}

#[test]
fn the_restored_dossier_has_the_state_folder_and_passes_check() {
    let project = Project::new();
    two_entries(&project);
    project
        .run("anna", &["pack", "--to", "backup.tar"])
        .success();
    project
        .run("", &["unpack", "backup.tar", "--into", "restored"])
        .success();

    let dossier = project.root().join("restored").join("DOSSIER");
    assert!(dossier.join(".state").join("ids.json").is_file());
    assert!(dossier.join("decisions").is_dir());
    assert!(dossier.join("findings").is_dir());
    // The restored dossier is read by the same program, from its own folder.
    assert_cmd::Command::cargo_bin("dossier")
        .expect("the dossier program is built")
        .current_dir(project.root().join("restored"))
        .env_remove("DOSSIER_AGENT")
        .env_remove("TODO_AGENT")
        .arg("check")
        .assert()
        .success()
        .stdout(contains("DOSSIER: ok (0 problems)"));
}

#[test]
fn pack_to_an_existing_file_is_refused_and_the_file_is_kept() {
    let project = Project::new();
    two_entries(&project);
    project.write("backup.tar", "an earlier backup");
    project
        .run("anna", &["pack", "--to", "backup.tar"])
        .code(1)
        .stderr(contains("error: backup.tar already exists"));
    assert_eq!(project.read("backup.tar"), "an earlier backup");
}

#[test]
fn unpack_into_a_folder_that_has_a_dossier_is_refused_and_changes_nothing() {
    let project = Project::new();
    two_entries(&project);
    project
        .run("anna", &["pack", "--to", "backup.tar"])
        .success();
    let before = bytes_in(project.root(), Path::new("DOSSIER/dossier.toml"));
    project
        .run("", &["unpack", "backup.tar", "--into", "."])
        .code(1)
        .stderr(contains("exists and is not empty"));
    assert_eq!(
        bytes_in(project.root(), Path::new("DOSSIER/dossier.toml")),
        before
    );
}

#[test]
fn unpack_of_a_missing_archive_is_refused() {
    let project = Project::new();
    project
        .run("", &["unpack", "missing.tar", "--into", "restored"])
        .code(1)
        .stderr(contains("no such archive"));
    assert!(!project.root().join("restored").exists());
}

#[test]
fn pack_without_to_is_a_usage_error() {
    let project = Project::new();
    project.run("anna", &["pack"]).code(2);
    assert!(!project.root().join("backup.tar").exists());
}

#[test]
fn pack_as_json_names_the_archive_and_the_file_count() {
    let project = Project::new();
    two_entries(&project);
    project
        .run("anna", &["pack", "--to", "backup.tar", "--json"])
        .success()
        .stdout(contains("\"schema\":\"dossier.pack.v1\""))
        .stdout(contains("\"ok\":true"))
        .stdout(contains("\"archive\":\"backup.tar\""));
}
