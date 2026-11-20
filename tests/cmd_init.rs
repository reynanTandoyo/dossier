//! Runs the built program for `init`: the folders and the settings of a new project dossier, and
//! the refusals. Each test runs in its own temporary folder.

use assert_cmd::Command;
use predicates::str::contains;
use std::path::Path;

/// `init` with `args` run in `folder`, not yet checked. The agent names are removed from the
/// environment, and the clock is fixed.
fn init(folder: &Path, args: &[&str]) -> assert_cmd::assert::Assert {
    Command::cargo_bin("dossier")
        .unwrap()
        .current_dir(folder)
        .env("DOSSIER_NOW", "2026-11-21T09:00:00+01:00")
        .env_remove("DOSSIER_AGENT")
        .env_remove("TODO_AGENT")
        .arg("init")
        .args(args)
        .assert()
}

/// The text of dossier.toml in the dossier of `folder`.
fn settings(folder: &Path) -> String {
    std::fs::read_to_string(folder.join("DOSSIER").join("dossier.toml")).unwrap()
}

#[test]
fn init_creates_the_folders_and_a_settings_file_with_the_code() {
    let folder = tempfile::tempdir().unwrap();
    init(folder.path(), &["--code", "P-001"])
        .success()
        .stdout(contains("created DOSSIER for P-001"))
        .stdout(contains("next: dossier people add <person> --llm <name>"));
    let dossier = folder.path().join("DOSSIER");
    for name in ["decisions", "assumptions", "findings", "lessons", ".state"] {
        assert!(dossier.join(name).is_dir(), "{name} is missing");
    }
    assert!(settings(folder.path()).contains("code = \"P-001\""));
    let cfg = dossier::config::load(&dossier).unwrap();
    assert_eq!(cfg.code, "P-001");
    assert_eq!(cfg.format, 1);
}

#[test]
fn a_second_init_is_refused_and_changes_nothing() {
    let folder = tempfile::tempdir().unwrap();
    init(folder.path(), &["--code", "P-001"]).success();
    let before = settings(folder.path());
    init(folder.path(), &["--code", "P-002"])
        .code(1)
        .stderr(contains("error: DOSSIER already exists here"));
    assert_eq!(settings(folder.path()), before);
}

#[test]
fn a_file_named_dossier_is_refused_too() {
    let folder = tempfile::tempdir().unwrap();
    std::fs::write(folder.path().join("DOSSIER"), "not a folder").unwrap();
    init(folder.path(), &["--code", "P-001"])
        .code(1)
        .stderr(contains("error: DOSSIER already exists here"));
    assert!(folder.path().join("DOSSIER").is_file());
}

#[test]
fn a_code_with_a_space_is_refused_and_nothing_is_created() {
    let folder = tempfile::tempdir().unwrap();
    init(folder.path(), &["--code", "P 001"])
        .code(1)
        .stderr(contains(
            "error: a project code has only letters, digits and -, such as P-017",
        ));
    assert!(!folder.path().join("DOSSIER").exists());
}

#[test]
fn an_empty_code_is_refused() {
    let folder = tempfile::tempdir().unwrap();
    init(folder.path(), &["--code", ""])
        .code(1)
        .stderr(contains("a project code has only letters, digits and -"));
    assert!(!folder.path().join("DOSSIER").exists());
}

#[test]
fn a_code_with_a_non_ascii_letter_is_refused() {
    let folder = tempfile::tempdir().unwrap();
    init(folder.path(), &["--code", "P-Ü01"]).code(1);
    assert!(!folder.path().join("DOSSIER").exists());
}

#[test]
fn letters_and_digits_in_either_case_with_hyphens_are_accepted() {
    let folder = tempfile::tempdir().unwrap();
    init(folder.path(), &["--code", "abc-17-X"]).success();
    assert!(settings(folder.path()).contains("code = \"abc-17-X\""));
}

#[test]
fn init_without_a_code_is_a_usage_error() {
    let folder = tempfile::tempdir().unwrap();
    init(folder.path(), &[]).code(2);
    assert!(!folder.path().join("DOSSIER").exists());
}

#[test]
fn init_works_in_a_folder_with_spaces_and_non_ascii_characters() {
    let folder = tempfile::tempdir().unwrap();
    let project = folder.path().join("Projekt Übersicht");
    std::fs::create_dir(&project).unwrap();
    init(&project, &["--code", "P-001"]).success();
    assert!(project.join("DOSSIER").join("dossier.toml").is_file());
}

#[test]
fn init_as_json_names_the_code_and_the_dossier() {
    let folder = tempfile::tempdir().unwrap();
    init(folder.path(), &["--code", "P-001", "--json"])
        .success()
        .stdout(contains("\"schema\":\"dossier.init.v1\""))
        .stdout(contains("\"ok\":true"))
        .stdout(contains("\"code\":\"P-001\""));
}

#[test]
fn a_refused_init_as_json_gives_the_error_name() {
    let folder = tempfile::tempdir().unwrap();
    init(folder.path(), &["--code", "P 001", "--json"])
        .code(1)
        .stdout(contains("\"ok\":false"))
        .stdout(contains("\"code\":\"BAD_VALUE\""));
}

#[cfg(unix)]
#[test]
fn on_linux_the_folders_and_the_settings_are_group_writable() {
    use std::os::unix::fs::PermissionsExt;
    let folder = tempfile::tempdir().unwrap();
    init(folder.path(), &["--code", "P-001"])
        .success()
        .stdout(contains(
            "group: files are writable by the group of this folder",
        ));
    let dossier = folder.path().join("DOSSIER");
    let mode = |path: &Path| std::fs::metadata(path).unwrap().permissions().mode() & 0o7777;
    assert_eq!(mode(&dossier), 0o2775);
    for name in ["decisions", "assumptions", "findings", "lessons", ".state"] {
        assert_eq!(mode(&dossier.join(name)), 0o2775, "{name}");
    }
    assert_eq!(mode(&dossier.join("dossier.toml")), 0o664);
}

#[cfg(not(unix))]
#[test]
fn on_windows_no_group_line_is_written() {
    use predicates::prelude::PredicateBooleanExt;
    let folder = tempfile::tempdir().unwrap();
    init(folder.path(), &["--code", "P-001"])
        .success()
        .stdout(contains("group:").not());
}
