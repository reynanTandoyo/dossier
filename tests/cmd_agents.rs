//! Runs the built program for `agents` and `help agents`: the rules block in AGENTS.md, and the
//! rules text. No dossier is needed; each test runs in its own temporary folder.

use assert_cmd::Command;
use predicates::str::{contains, starts_with};
use std::path::Path;

/// The program run in `folder` with `args`, not yet checked.
fn dossier(folder: &Path, args: &[&str]) -> assert_cmd::assert::Assert {
    Command::cargo_bin("dossier")
        .unwrap()
        .current_dir(folder)
        .args(args)
        .assert()
}

/// The text of AGENTS.md in `folder`.
fn agents_md(folder: &Path) -> String {
    std::fs::read_to_string(folder.join("AGENTS.md")).unwrap()
}

#[test]
fn help_agents_prints_the_heading_of_the_rules() {
    let folder = tempfile::tempdir().unwrap();
    dossier(folder.path(), &["help", "agents"])
        .success()
        .stdout(starts_with("# Working with the dossier\n"));
}

#[test]
fn help_agents_as_json_gives_the_rules_text() {
    let folder = tempfile::tempdir().unwrap();
    dossier(folder.path(), &["help", "agents", "--json"])
        .success()
        .stdout(contains("\"schema\":\"dossier.help.v1\""))
        .stdout(contains("\"rules\":\"# Working with the dossier"));
}

#[test]
fn check_fails_before_install_and_names_the_command() {
    let folder = tempfile::tempdir().unwrap();
    dossier(folder.path(), &["agents", "check"])
        .code(1)
        .stderr(contains("run: dossier agents install --yes"));
    assert!(!folder.path().join("AGENTS.md").exists());
}

#[test]
fn install_without_yes_changes_nothing() {
    let folder = tempfile::tempdir().unwrap();
    dossier(folder.path(), &["agents", "install"])
        .success()
        .stdout(contains(
            "would write the rules block to AGENTS.md; add --yes",
        ));
    assert!(!folder.path().join("AGENTS.md").exists());
}

#[test]
fn install_with_dry_run_writes_nothing() {
    let folder = tempfile::tempdir().unwrap();
    dossier(folder.path(), &["agents", "install", "--yes", "--dry-run"])
        .success()
        .stdout(contains(
            "dry run: would write the rules block to AGENTS.md",
        ));
    assert!(!folder.path().join("AGENTS.md").exists());
}

#[test]
fn install_with_yes_writes_the_block_and_check_then_succeeds() {
    let folder = tempfile::tempdir().unwrap();
    dossier(folder.path(), &["agents", "install", "--yes"])
        .success()
        .stdout(contains("AGENTS.md: rules block written"));
    assert!(agents_md(folder.path()).contains("<!-- dossier:begin -->"));
    assert!(agents_md(folder.path()).contains("<!-- dossier:end -->"));
    dossier(folder.path(), &["agents", "check"])
        .success()
        .stdout(contains("AGENTS.md: rules block is current"));
}

#[test]
fn check_as_json_reports_current() {
    let folder = tempfile::tempdir().unwrap();
    dossier(folder.path(), &["agents", "install", "--yes"]).success();
    dossier(folder.path(), &["agents", "check", "--json"])
        .success()
        .stdout(contains("\"current\":true"));
}

#[test]
fn a_line_written_before_install_is_still_there_afterwards() {
    let folder = tempfile::tempdir().unwrap();
    std::fs::write(
        folder.path().join("AGENTS.md"),
        "# Project\n\nKeep this line.\n",
    )
    .unwrap();
    dossier(folder.path(), &["agents", "install", "--yes"]).success();
    let text = agents_md(folder.path());
    assert!(text.starts_with("# Project\n\nKeep this line.\n"));
    assert!(text.contains("<!-- dossier:begin -->"));
    dossier(folder.path(), &["agents", "check"]).success();
}

#[test]
fn installing_twice_changes_nothing_the_second_time() {
    let folder = tempfile::tempdir().unwrap();
    dossier(folder.path(), &["agents", "install", "--yes"]).success();
    let once = agents_md(folder.path());
    dossier(folder.path(), &["agents", "install", "--yes"]).success();
    assert_eq!(agents_md(folder.path()), once);
}

#[test]
fn agents_without_an_action_is_a_usage_error() {
    let folder = tempfile::tempdir().unwrap();
    dossier(folder.path(), &["agents"]).code(2);
}

#[test]
fn unknown_help_topic_is_a_usage_error() {
    let folder = tempfile::tempdir().unwrap();
    dossier(folder.path(), &["help", "nothing"])
        .code(2)
        .stderr(contains("unknown help topic 'nothing'"));
}

#[test]
fn help_without_a_topic_lists_every_command_in_order() {
    let folder = tempfile::tempdir().unwrap();
    let output = Command::cargo_bin("dossier")
        .unwrap()
        .current_dir(folder.path())
        .arg("help")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let names: Vec<&str> = stdout
        .lines()
        .map(|line| line.split_whitespace().next().unwrap())
        .collect();
    assert_eq!(
        names,
        [
            "init",
            "where",
            "people",
            "add",
            "decide",
            "found",
            "learn",
            "assume",
            "set",
            "retitle",
            "link",
            "edit",
            "show",
            "list",
            "search",
            "history",
            "diff",
            "restore",
            "log",
            "review-task",
            "drafts",
            "approve",
            "reject",
            "supersede",
            "retire",
            "remove",
            "reopen",
            "brief",
            "status",
            "check",
            "agents",
            "help",
            "pack",
            "unpack",
            "connect",
            "projects",
            "todo",
        ]
    );
    let first = format!("  {:<12}  Creates the dossier folder here\n", "init");
    assert!(stdout.starts_with(&first), "{stdout}");
}
