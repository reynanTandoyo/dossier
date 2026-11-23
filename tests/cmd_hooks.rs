//! Runs the built program for `hooks install` and `hooks uninstall`: the Claude Code hooks of the
//! dossier in .claude/settings.json, and the name of the LLM that Claude's writes are for in
//! .claude/settings.local.json (D34), of the current folder. Each test runs in its own temporary
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

/// The personal settings file of the project folder, relative to it.
const LOCAL: &str = ".claude/settings.local.json";

/// The backup of the personal settings file, relative to the project folder.
const LOCAL_BACKUP: &str = ".claude/settings.local.json.bak";

/// A settings file with another key and another hook, and no dossier hook.
const OTHER: &str = "{\n  \"model\": \"invented-model\",\n  \"hooks\": {\n    \"SessionStart\": [\n      {\"hooks\": [{\"type\": \"command\", \"command\": \"echo start\"}]}\n    ]\n  }\n}\n";

/// A personal settings file with another key and another variable, and an older agent name.
const LOCAL_OTHER: &str = "{\n  \"model\": \"invented-model\",\n  \"env\": {\n    \"DOSSIER_AGENT\": \"old - claude\",\n    \"INVENTED_VARIABLE\": \"1\"\n  }\n}\n";

/// The settings file of the project folder as parsed JSON.
fn settings(project: &Project) -> Value {
    serde_json::from_str(&project.read(SETTINGS)).unwrap()
}

/// The personal settings file of the project folder as parsed JSON.
fn local_settings(project: &Project) -> Value {
    serde_json::from_str(&project.read(LOCAL)).unwrap()
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
            "would add the dossier hooks to .claude/settings.json and set DOSSIER_AGENT in .claude/settings.local.json; add --yes to write them",
        ))
        .stdout(contains("+ SessionStart: dossier brief"))
        .stdout(contains(
            "+ PreToolUse (Edit|Write|MultiEdit): dossier guard",
        ))
        .stdout(contains(
            "+ PostToolUse (Bash|PowerShell): dossier after",
        ))
        .stdout(contains("+ env DOSSIER_AGENT: anna - claude"));
    assert!(!exists(&project, ".claude"));
    assert!(hook_records(&project).is_empty());
}

#[test]
fn install_with_yes_writes_the_hooks_and_the_agent_of_claude() {
    let project = Project::new();
    project
        .run("anna", &["hooks", "install", "--yes"])
        .success()
        .stdout(contains(
            "hooks installed in .claude/settings.json and DOSSIER_AGENT set in .claude/settings.local.json",
        ));
    assert_eq!(
        settings(&project),
        json!({"hooks": {
            "SessionStart": [
                {"hooks": [{"type": "command", "command": "dossier brief"}]}
            ],
            "PreToolUse": [{
                "matcher": "Edit|Write|MultiEdit",
                "hooks": [{"type": "command", "command": "dossier guard"}]
            }],
            "PostToolUse": [{
                "matcher": "Bash|PowerShell",
                "hooks": [{"type": "command", "command": "dossier after", "timeout": 30}]
            }]
        }})
    );
    assert_eq!(
        local_settings(&project),
        json!({"env": {"DOSSIER_AGENT": "anna - claude"}})
    );
    // There were no files before, so there are no copies of them.
    assert!(!exists(&project, BACKUP));
    assert!(!exists(&project, LOCAL_BACKUP));
}

#[test]
fn a_second_install_changes_nothing_and_makes_no_copy() {
    let project = Project::new();
    project
        .run("anna", &["hooks", "install", "--yes"])
        .success();
    let once = project.read(SETTINGS);
    let once_local = project.read(LOCAL);
    project
        .run("anna", &["hooks", "install", "--yes"])
        .success()
        .stdout(contains("nothing to change"))
        .stdout(contains(
            "DOSSIER_AGENT is set in .claude/settings.local.json",
        ));
    assert_eq!(project.read(SETTINGS), once);
    assert_eq!(project.read(LOCAL), once_local);
    assert!(!exists(&project, BACKUP));
    assert!(!exists(&project, LOCAL_BACKUP));
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
    // The personal settings file did not exist, so it has no copy.
    assert_eq!(
        local_settings(&project),
        json!({"env": {"DOSSIER_AGENT": "anna - claude"}})
    );
    assert!(!exists(&project, LOCAL_BACKUP));
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
        .stdout(contains(
            "hooks removed from .claude/settings.json and DOSSIER_AGENT removed from .claude/settings.local.json",
        ));
    assert_eq!(
        settings(&project),
        serde_json::from_str::<Value>(OTHER).unwrap()
    );
    // The agent name goes too. The file keeps an empty object, as the settings file does.
    assert_eq!(local_settings(&project), json!({}));
}

#[test]
fn uninstall_removes_the_agent_and_keeps_the_other_local_settings_with_a_copy() {
    let project = Project::new();
    project.write(LOCAL, LOCAL_OTHER);
    project
        .run("anna", &["hooks", "uninstall", "--yes"])
        .success()
        .stdout(contains(
            "DOSSIER_AGENT removed from .claude/settings.local.json",
        ));
    assert_eq!(
        local_settings(&project),
        json!({"model": "invented-model", "env": {"INVENTED_VARIABLE": "1"}})
    );
    assert_eq!(project.read(LOCAL_BACKUP), LOCAL_OTHER);
    assert!(!exists(&project, BACKUP));
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
    assert_eq!(records[0].detail, "hooks installed; DOSSIER_AGENT set");
    assert_eq!(records[1].detail, "hooks removed; DOSSIER_AGENT removed");
}

#[test]
fn uninstall_without_dossier_hooks_changes_nothing_and_creates_no_file() {
    let project = Project::new();
    project
        .run("anna", &["hooks", "uninstall", "--yes"])
        .success()
        .stdout(contains(
            "there are no dossier hooks in .claude/settings.json and there is no DOSSIER_AGENT in .claude/settings.local.json: nothing to change",
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
    assert!(!exists(&project, LOCAL));
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
    assert!(!exists(&project, LOCAL));
    assert!(hook_records(&project).is_empty());
}

#[test]
fn a_personal_settings_file_that_is_not_json_refuses_the_whole_change() {
    let project = Project::new();
    project.write(LOCAL, "not json");
    project
        .run("anna", &["hooks", "install", "--yes"])
        .code(4)
        .stderr(contains("the settings are not valid JSON"));
    assert_eq!(project.read(LOCAL), "not json");
    // The hooks file would have changed, but nothing is written when one file is refused.
    assert!(!exists(&project, SETTINGS));
    assert!(!exists(&project, LOCAL_BACKUP));
    assert!(hook_records(&project).is_empty());
}

#[test]
fn a_dry_run_with_yes_writes_nothing() {
    let project = Project::new();
    project
        .run("anna", &["hooks", "install", "--yes", "--dry-run"])
        .success()
        .stdout(contains(
            "dry run: would write .claude/settings.json and .claude/settings.local.json",
        ))
        .stdout(contains("+ SessionStart: dossier brief"))
        .stdout(contains("+ PostToolUse (Bash|PowerShell): dossier after"));
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
        .stdout(contains("\"files\":["))
        .stdout(contains("\"added\":["))
        .stdout(contains("\"SessionStart: dossier brief\""))
        .stdout(contains("\"PostToolUse (Bash|PowerShell): dossier after\""))
        .stdout(contains("\"env DOSSIER_AGENT: anna - claude\""));
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
    assert!(folder.join(".claude").join("settings.local.json").is_file());
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

#[test]
fn install_keeps_the_other_local_settings_and_replaces_an_older_agent_with_a_copy() {
    let project = Project::new();
    project.write(LOCAL, LOCAL_OTHER);
    project
        .run("anna", &["hooks", "install", "--yes"])
        .success()
        .stdout(contains(
            "DOSSIER_AGENT set in .claude/settings.local.json (the old file is kept as .claude/settings.local.json.bak)",
        ))
        .stdout(contains("+ env DOSSIER_AGENT: anna - claude"))
        .stdout(contains("- env DOSSIER_AGENT: old - claude"));
    assert_eq!(
        local_settings(&project),
        json!({
            "model": "invented-model",
            "env": {"DOSSIER_AGENT": "anna - claude", "INVENTED_VARIABLE": "1"}
        })
    );
    assert_eq!(project.read(LOCAL_BACKUP), LOCAL_OTHER);
    assert!(!exists(&project, BACKUP));
}

#[test]
fn install_with_both_old_files_keeps_both_copies() {
    let project = Project::new();
    project.write(SETTINGS, OTHER);
    project.write(LOCAL, LOCAL_OTHER);
    project
        .run("anna", &["hooks", "install", "--yes"])
        .success()
        .stdout(contains(
            "(the old files are kept as .claude/settings.json.bak and .claude/settings.local.json.bak)",
        ));
    assert_eq!(project.read(BACKUP), OTHER);
    assert_eq!(project.read(LOCAL_BACKUP), LOCAL_OTHER);
}

#[test]
fn when_the_hooks_are_already_in_place_only_the_agent_is_written() {
    let project = Project::new();
    project
        .run("anna", &["hooks", "install", "--yes"])
        .success();
    project.write(LOCAL, "{}\n");
    project
        .run("anna", &["hooks", "install", "--yes"])
        .success()
        .stdout(contains("DOSSIER_AGENT set in .claude/settings.local.json"));
    assert_eq!(
        local_settings(&project),
        json!({"env": {"DOSSIER_AGENT": "anna - claude"}})
    );
    // The hooks file was not written again, so it has no copy.
    assert!(!exists(&project, BACKUP));
    let records = hook_records(&project);
    assert_eq!(records.len(), 2);
    assert_eq!(records[1].detail, "DOSSIER_AGENT set");
}

#[test]
fn without_a_dossier_the_agent_flag_names_the_writer() {
    // A folder with no dossier: hooks install does not need one.
    let project = Project {
        dir: tempfile::TempDir::new().unwrap(),
    };
    project
        .run("", &["hooks", "install", "--yes", "--agent", "tony"])
        .success()
        .stdout(contains(
            "hooks installed in .claude/settings.json and DOSSIER_AGENT set in .claude/settings.local.json",
        ));
    assert_eq!(
        local_settings(&project),
        json!({"env": {"DOSSIER_AGENT": "tony - claude"}})
    );
    assert!(hook_records(&project).is_empty());
}

#[test]
fn the_agent_is_named_for_the_person_of_an_llm_or_a_person_who_writes() {
    let project = Project::new();
    // anna-claude is the LLM that the project registers for anna.
    project
        .run("anna-claude", &["hooks", "install", "--yes"])
        .success();
    assert_eq!(
        local_settings(&project),
        json!({"env": {"DOSSIER_AGENT": "anna - claude"}})
    );
    project.run("ben", &["hooks", "install", "--yes"]).success();
    assert_eq!(
        local_settings(&project),
        json!({"env": {"DOSSIER_AGENT": "ben - claude"}})
    );
}
