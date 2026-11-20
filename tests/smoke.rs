//! Runs the built program: its version, and a command that does not exist.

use assert_cmd::Command;

#[test]
fn version_prints_the_version() {
    let folder = tempfile::tempdir().unwrap();
    Command::cargo_bin("dossier")
        .unwrap()
        .current_dir(folder.path())
        .arg("--version")
        .assert()
        .success()
        .stdout(predicates::str::starts_with("dossier"));
}

#[test]
fn unknown_command_exits_with_two() {
    let folder = tempfile::tempdir().unwrap();
    Command::cargo_bin("dossier")
        .unwrap()
        .current_dir(folder.path())
        .arg("no-such-command")
        .assert()
        .code(2);
}
