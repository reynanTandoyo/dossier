//! Runs the built program for `help <command>`: the description and the examples of a command of
//! the first group, a command without examples, the JSON data, and an unknown topic. `help` needs
//! no dossier, so each test runs in an empty temporary folder.

use assert_cmd::Command;
use predicates::str::contains;
use serde_json::Value;
use std::path::Path;

/// `dossier help <args>` run in `folder`.
fn help(folder: &Path, args: &[&str]) -> assert_cmd::assert::Assert {
    Command::cargo_bin("dossier")
        .unwrap()
        .current_dir(folder)
        .arg("help")
        .args(args)
        .assert()
}

#[test]
fn help_decide_prints_the_description_and_three_example_lines() {
    let folder = tempfile::tempdir().unwrap();
    let output = help(folder.path(), &["decide"])
        .success()
        .get_output()
        .stdout
        .clone();
    let text = String::from_utf8(output).expect("the standard output is UTF-8");
    let lines: Vec<&str> = text.lines().collect();
    assert_eq!(lines.len(), 6, "{text}");
    assert_eq!(lines[0], "decide: Records a decision and reason");
    assert_eq!(lines[1], "Examples (PowerShell):");
    assert_eq!(lines[4], "Example (Linux shell):");
    for example in [lines[2], lines[3], lines[5]] {
        assert!(
            example.trim_start().starts_with("dossier decide"),
            "{example}"
        );
    }
}

#[test]
fn help_of_a_command_without_examples_prints_only_its_description() {
    let folder = tempfile::tempdir().unwrap();
    help(folder.path(), &["pack"])
        .success()
        .stdout("pack: Writes a dossier archive file\n");
}

#[test]
fn help_as_json_gives_the_command_description_and_its_three_examples() {
    let folder = tempfile::tempdir().unwrap();
    let output = help(folder.path(), &["decide", "--json"])
        .success()
        .get_output()
        .stdout
        .clone();
    let value: Value = serde_json::from_slice(&output).expect("the output is JSON");
    assert_eq!(value["schema"], "dossier.help.v1");
    assert_eq!(value["data"]["command"], "decide");
    assert_eq!(
        value["data"]["description"],
        "Records a decision and reason"
    );
    let examples = value["data"]["examples"].as_array().expect("an array");
    assert_eq!(examples.len(), 3);
    for example in examples {
        assert!(example.as_str().unwrap_or("").starts_with("dossier decide"));
    }
}

#[test]
fn help_nosuch_is_a_usage_error() {
    let folder = tempfile::tempdir().unwrap();
    help(folder.path(), &["nosuch"])
        .code(2)
        .stderr(contains("unknown help topic 'nosuch'"));
}
