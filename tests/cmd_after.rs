//! Runs the built program for `after`: a Claude Code PostToolUse payload of a shell tool on standard
//! input. After a `todo` command that claimed, finished or gave back a task, it prints one JSON
//! line whose additionalContext is the dossier's context for that moment (R-AGENT-5). Every other
//! payload, text that is not a payload, and a run that fails print nothing and exit 0. The payloads
//! are invented, and each test runs in its own temporary project folder with the clock fixed at
//! 2026-11-20T10:00:00+01:00 (common::Project), with a stand-in for todo where a test needs one.

mod common;

use common::Project;
use predicates::str::is_empty;
use serde_json::{Value, json};

/// The envelope the stand-in for todo prints for `todo show T-001 --json`.
const SHOW_T001: &str = r#"{"schema":"todo.show.v1","ok":true,"data":{"id":"T-001","title":"after: the invented task","status":"open","priority":"P2","difficulty":1,"fields":{"machine":"invented machine"},"steps":["Run the invented check"],"notes":[]},"warnings":[]}"#;

/// The title of the finding that every test with a claimed or finished task records for T-001.
const FINDING: &str = "The invented mixer converged at low flow.";

/// Writes the stand-in for todo: it prints `envelope` and exits with 0, whatever it is given.
fn write_todo(project: &Project, envelope: &str) {
    let windows = format!("@echo off\r\necho {envelope}\r\nexit /b 0\r\n");
    let unix = format!("#!/bin/sh\nprintf '%s\\n' '{envelope}'\nexit 0\n");
    project.write_stand_in("todo", &windows, &unix);
}

/// A Claude Code PostToolUse payload of the Bash tool that ran `command` and printed `stdout`.
fn shell(command: &str, stdout: &str) -> String {
    json!({
        "session_id": "invented-session",
        "hook_event_name": "PostToolUse",
        "tool_name": "Bash",
        "tool_input": {"command": command},
        "tool_response": {"stdout": stdout, "stderr": "", "interrupted": false},
    })
    .to_string()
}

/// Runs `after` in the project folder with `payload` on standard input.
fn run_after(project: &Project, payload: &str) -> assert_cmd::assert::Assert {
    project
        .command("anna", "2026-11-20T10:00:00+01:00")
        .arg("after")
        .write_stdin(payload)
        .assert()
}

/// The additionalContext of the one JSON line that `after` printed. Panics when the output is not
/// exactly one such line.
fn context_of(stdout: &[u8]) -> String {
    let text = String::from_utf8(stdout.to_vec()).expect("the output is UTF-8");
    assert_eq!(text.matches('\n').count(), 1, "not one line: {text}");
    assert!(text.ends_with('\n'), "no line break: {text}");
    let value: Value = serde_json::from_str(text.trim_end()).expect("a JSON line");
    assert_eq!(
        value["hookSpecificOutput"]["hookEventName"], "PostToolUse",
        "{text}"
    );
    assert_eq!(
        value.as_object().map(|object| object.len()),
        Some(1),
        "{text}"
    );
    value["hookSpecificOutput"]["additionalContext"]
        .as_str()
        .expect("an additionalContext text")
        .to_string()
}

#[test]
fn a_claimed_task_gives_its_brief_as_one_json_line() {
    let project = Project::new();
    write_todo(&project, SHOW_T001);
    project.stdout("anna", &["found", FINDING, "--task", "T-001"]);
    let payload = shell(
        "todo next --claim --brief --agent haiku",
        "T-001 · P2 · claimed · haiku · after: the invented task\n",
    );
    let output = run_after(&project, &payload).success();
    let context = context_of(&output.get_output().stdout);
    assert!(context.contains(FINDING), "{context}");
    assert!(context.contains("T-001"), "{context}");
}

#[test]
fn a_finished_task_gives_its_review_and_the_usage_reminder() {
    let project = Project::new();
    write_todo(&project, SHOW_T001);
    project.stdout("anna", &["found", FINDING, "--task", "T-001"]);
    let payload = shell("todo done T-001 --commit HEAD --agent haiku", "");
    let output = run_after(&project, &payload).success();
    let context = context_of(&output.get_output().stdout);
    assert!(context.contains("# Review of T-001"), "{context}");
    assert!(context.contains(FINDING), "{context}");
    assert!(context.contains("No usage record for T-001"), "{context}");
}

#[test]
fn the_usage_reminder_is_gone_once_a_usage_record_exists() {
    let project = Project::new();
    write_todo(&project, SHOW_T001);
    project.stdout(
        "anna",
        &[
            "used",
            "ExampleSolver",
            "--version",
            "2406",
            "--task",
            "T-001",
        ],
    );
    let payload = shell("todo done T-001 --commit HEAD --agent haiku", "");
    let output = run_after(&project, &payload).success();
    let context = context_of(&output.get_output().stdout);
    assert!(context.contains("# Review of T-001"), "{context}");
    assert!(!context.contains("No usage record"), "{context}");
}

#[test]
fn a_given_back_task_gives_the_text_for_recording_what_went_wrong() {
    let project = Project::new();
    write_todo(&project, SHOW_T001);
    let payload = shell("todo stuck T-001 --note \"no data\" --agent haiku", "");
    let output = run_after(&project, &payload).success();
    let context = context_of(&output.get_output().stdout);
    assert!(context.starts_with("T-001 was given back."), "{context}");
    assert!(context.contains("--dead-end"), "{context}");
    assert!(context.contains("--task T-001"), "{context}");
}

#[test]
fn a_command_that_is_no_todo_event_prints_nothing() {
    let project = Project::new();
    write_todo(&project, SHOW_T001);
    run_after(&project, &shell("git status", ""))
        .success()
        .stdout(is_empty());
    run_after(&project, &shell("todo next --brief", ""))
        .success()
        .stdout(is_empty());
}

#[test]
fn an_empty_payload_and_text_that_is_not_json_print_nothing() {
    let project = Project::new();
    run_after(&project, "").success().stdout(is_empty());
    run_after(&project, "this is not a payload")
        .success()
        .stdout(is_empty());
}

#[test]
fn a_claim_with_no_todo_stand_in_still_exits_0_with_nothing_printed() {
    let project = Project::new();
    let payload = shell("todo claim T-001", "");
    run_after(&project, &payload).success().stdout(is_empty());
}

#[test]
fn a_finished_task_without_todo_still_exits_0_and_gives_the_usage_reminder() {
    let project = Project::new();
    // No stand-in for todo: review-task fails, and the usage reminder is still given.
    let payload = shell("todo done T-001 --commit HEAD --agent haiku", "");
    let output = run_after(&project, &payload).success();
    let context = context_of(&output.get_output().stdout);
    assert_eq!(
        context,
        "No usage record for T-001: if software was used, record it with dossier used <software> --version <v> --task T-001",
        "{context}"
    );
}
