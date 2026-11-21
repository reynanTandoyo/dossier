//! Runs the built program for `hooks install` and `hooks uninstall`: the Claude Code hooks of the
//! dossier in .claude/settings.json of the current folder. Each test runs in its own temporary
//! project folder (common::Project, with the clock fixed and the people anna and ben), so that the
//! journal lines of a change can be read back. The other hooks and keys are invented.

mod common;

use common::Project;
use predicates::str::contains;
use serde_json::{Value, json};

/// The fixed clock of `common::Project`, as it appears in the journal.
const WHEN: &str = "2026-11-20T10:00:00+01:00";

/// The settings file of the project folder, relative to it.
const SETTINGS: &str = ".claude/settings.json";

/// The backup of the settings file, relative to the project folder.
const BACKUP: &str = ".claude/settings.json.bak";

/// A settings file with another key and another hook, and no dossier hook.
const OTHER: &str = "{\n  \"model\": \"invented-model\",\n  \"hooks\": {\n    \"SessionStart\": [\n      {\"hooks\": [{\"type\": \"command\", \"command\": \"echo start\"}]}\n    ]\n  }\n}\n";

/// The settings file of the project folder as parsed JSON.
fn settings(project: &Project) -> Value {
    serde_json::from_str(&project.read(SETTINGS)).unwrap()
}

/// Whether a file or folder of the project folder exists.
fn exists(project: &Project, relative: &str) -> bool {
    project.root().join(relative).exists()
}

/// The journal lines of the project's dossier about hooks, oldest first.
fn hook_records(project: &Project) -> Vec<dossier::journal::Record> {
    let state = project.root().join("DOSSIER").join(".state");
    dossier::journal::read(&state)
        .unwrap()
        .into_iter()
        .filter(|record| record.id == "HOOKS")
        .collect()
}

#[test]
fn install_without_yes_says_what_would_change_and_writes_nothing() {
    let project = Project::new();
    project
        .run("anna", &["hooks", "install"])
        .success()
        .stdout(contains(
            "would add the dossier hooks to .claude/settings.json; add --yes to write it",
        ))
        .stdout(contains("+ SessionStart: dossier brief"))
        .stdout(contains(
            "+ PreToolUse (Edit|Write|MultiEdit): dossier guard",
        ));
    assert!(!exists(&project, ".claude"));
    assert!(hook_records(&project).is_empty());
}

#[test]
fn install_with_yes_writes_both_dossier_hooks() {
    let project = Project::new();
    project
        .run("anna", &["hooks", "install", "--yes"])
        .success()
        .stdout(contains("hooks installed in .claude/settings.json"));
    assert_eq!(
        settings(&project),
        json!({"hooks": {
            "SessionStart": [
                {"hooks": [{"type": "command", "command": "dossier brief"}]}
            ],
            "PreToolUse": [{
                "matcher": "Edit|Write|MultiEdit",
                "hooks": [{"type": "command", "command": "dossier guard"}]
            }]
        }})
    );
    // There was no file before, so there is no copy of one.
    assert!(!exists(&project, BACKUP));
}

#[test]
fn a_second_install_changes_nothing_and_makes_no_copy() {
    let project = Project::new();
    project
        .run("anna", &["hooks", "install", "--yes"])
        .success();
    let once = project.read(SETTINGS);
    project
        .run("anna", &["hooks", "install", "--yes"])
        .success()
        .stdout(contains("nothing to change"));
    assert_eq!(project.read(SETTINGS), once);
    assert!(!exists(&project, BACKUP));
    assert_eq!(hook_records(&project).len(), 1);
}

#[test]
fn an_existing_hook_is_kept_and_the_old_file_is_copied_to_a_backup() {
    let project = Project::new();
    project.write(SETTINGS, OTHER);
    project
        .run("anna", &["hooks", "install", "--yes"])
        .success()
        .stdout(contains(
            "(the old file is kept as .claude/settings.json.bak)",
        ));
    assert_eq!(project.read(BACKUP), OTHER);
    let value = settings(&project);
    assert_eq!(value["model"], "invented-model");
    let session = value["hooks"]["SessionStart"].as_array().unwrap();
    assert_eq!(session.len(), 2, "{value}");
    assert!(session.contains(&json!({"hooks": [{"type": "command", "command": "echo start"}]})));
    assert!(session.contains(&json!({"hooks": [{"type": "command", "command": "dossier brief"}]})));
    assert_eq!(
        value["hooks"]["PreToolUse"][0]["hooks"][0]["command"],
        "dossier guard"
    );
}

#[test]
fn uninstall_with_yes_removes_only_the_dossier_hooks() {
    let project = Project::new();
    project.write(SETTINGS, OTHER);
    project
        .run("anna", &["hooks", "install", "--yes"])
        .success();
    project
        .run("anna", &["hooks", "uninstall", "--yes"])
        .success()
        .stdout(contains("hooks removed from .claude/settings.json"));
    assert_eq!(
        settings(&project),
        serde_json::from_str::<Value>(OTHER).unwrap()
    );
}

#[test]
fn install_and_uninstall_each_add_a_journal_line_with_the_agent_and_the_time() {
    let project = Project::new();
    project
        .run("anna", &["hooks", "install", "--yes"])
        .success();
    project
        .run("anna", &["hooks", "uninstall", "--yes"])
        .success();
    let records = hook_records(&project);
    assert_eq!(records.len(), 2);
    for record in &records {
        assert_eq!(record.time, WHEN);
        assert_eq!(record.name, "anna");
        assert_eq!(record.person, "anna");
        assert!(!record.llm);
        assert_eq!(record.command, "hooks");
        assert_eq!(record.id, "HOOKS");
        assert_eq!(record.event, "changed");
    }
    assert_eq!(records[0].detail, "hooks installed");
    assert_eq!(records[1].detail, "hooks removed");
}

#[test]
fn uninstall_without_dossier_hooks_changes_nothing_and_creates_no_file() {
    let project = Project::new();
    project
        .run("anna", &["hooks", "uninstall", "--yes"])
        .success()
        .stdout(contains(
            "there are no dossier hooks in .claude/settings.json: nothing to change",
        ));
    assert!(!exists(&project, ".claude"));
    assert!(hook_records(&project).is_empty());

    // A file that has no dossier hook is left as it is, layout and all, with no copy made.
    project.write(SETTINGS, OTHER);
    project
        .run("anna", &["hooks", "uninstall", "--yes"])
        .success();
    assert_eq!(project.read(SETTINGS), OTHER);
    assert!(!exists(&project, BACKUP));
}

#[test]
fn a_settings_file_that_is_not_json_is_refused_and_kept() {
    let project = Project::new();
    project.write(SETTINGS, "not json");
    project
        .run("anna", &["hooks", "install", "--yes"])
        .code(4)
        .stderr(contains("the settings are not valid JSON"));
    assert_eq!(project.read(SETTINGS), "not json");
    assert!(!exists(&project, BACKUP));
    assert!(hook_records(&project).is_empty());
}

#[test]
fn a_dry_run_with_yes_writes_nothing() {
    let project = Project::new();
    project
        .run("anna", &["hooks", "install", "--yes", "--dry-run"])
        .success()
        .stdout(contains("dry run: would write .claude/settings.json"))
        .stdout(contains("+ SessionStart: dossier brief"));
    assert!(!exists(&project, ".claude"));
    assert!(hook_records(&project).is_empty());
}

#[test]
fn json_output_gives_the_hook_lines_as_data_and_writes_nothing() {
    let project = Project::new();
    project
        .run("anna", &["hooks", "install", "--json"])
        .success()
        .stdout(contains("\"schema\":\"dossier.hooks.v1\""))
        .stdout(contains("\"changed\":true"))
        .stdout(contains("\"written\":false"))
        .stdout(contains("\"added\":["))
        .stdout(contains("\"SessionStart: dossier brief\""));
    assert!(!exists(&project, ".claude"));
}

#[test]
fn a_folder_with_spaces_and_non_ascii_letters_works_and_the_dossier_above_gets_the_line() {
    let project = Project::new();
    let folder = project.root().join("Projekt Übersicht");
    std::fs::create_dir_all(&folder).unwrap();
    project
        .command("anna", WHEN)
        .current_dir(&folder)
        .args(["hooks", "install", "--yes"])
        .assert()
        .success();
    assert!(folder.join(".claude").join("settings.json").is_file());
    assert_eq!(hook_records(&project).len(), 1);
}

#[test]
fn a_dossier_option_that_is_not_a_dossier_is_refused_before_anything_is_written() {
    let project = Project::new();
    std::fs::create_dir_all(project.root().join("plain")).unwrap();
    project
        .run("anna", &["hooks", "install", "--yes", "--dossier", "plain"])
        .code(4)
        .stderr(contains("is not a dossier folder"));
    assert!(!exists(&project, ".claude"));
}
