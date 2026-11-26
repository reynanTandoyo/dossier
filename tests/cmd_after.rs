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

/// The problem of the investigations that the tests open with `problem`.
const PROBLEM: &str = "The invented mixer stalls at high flow.";

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
    assert!(
        context.contains(
            "or open an investigation: dossier problem \"<what goes wrong>\" --task T-001"
        ),
        "{context}"
    );
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

#[test]
fn a_finished_task_names_its_open_investigations_until_one_is_solved() {
    let project = Project::new();
    write_todo(&project, SHOW_T001);
    project.stdout("anna", &["problem", PROBLEM, "--task", "T-001"]);
    project.stdout(
        "anna",
        &["problem", "The invented pump runs dry.", "--task", "T-002"],
    );
    let payload = shell("todo done T-001 --commit HEAD --agent haiku", "");
    let output = run_after(&project, &payload).success();
    let context = context_of(&output.get_output().stdout);
    assert!(
        context.contains("INV-001 is still open: record the result with dossier tried INV-001"),
        "{context}"
    );
    assert!(context.contains("dossier solved INV-001"), "{context}");
    assert!(!context.contains("INV-002"), "{context}");

    project.stdout(
        "anna",
        &["solved", "INV-001", "The invented valve was stuck open."],
    );
    let output = run_after(&project, &payload).success();
    let context = context_of(&output.get_output().stdout);
    assert!(!context.contains("is still open"), "{context}");
}

#[test]
fn an_open_investigation_with_an_attempt_of_the_task_is_named_once() {
    let project = Project::new();
    write_todo(&project, SHOW_T001);
    project.stdout("anna", &["problem", PROBLEM, "--task", "T-002"]);
    project.stdout(
        "anna",
        &[
            "tried",
            "INV-001",
            "The invented gasket was replaced.",
            "--failed",
            "--task",
            "T-001",
        ],
    );
    project.stdout(
        "anna",
        &[
            "tried",
            "INV-001",
            "The invented gasket was tightened.",
            "--unclear",
            "--task",
            "T-001",
        ],
    );
    let payload = shell("todo done T-001 --commit HEAD --agent haiku", "");
    let output = run_after(&project, &payload).success();
    let context = context_of(&output.get_output().stdout);
    assert_eq!(
        context.matches("INV-001 is still open").count(),
        1,
        "{context}"
    );

    project.stdout(
        "anna",
        &["solved", "INV-001", "The invented gasket was the cause."],
    );
    let output = run_after(&project, &payload).success();
    let context = context_of(&output.get_output().stdout);
    assert!(!context.contains("is still open"), "{context}");
}

/// A temporary project folder made with `init --code P-001 --code-project` (format.md §19), with
/// the person anna added. The clock is the one of the other tests.
fn code_project() -> Project {
    let project = Project {
        dir: tempfile::TempDir::new().expect("a temporary project folder"),
    };
    project
        .command("anna", "2026-11-20T10:00:00+01:00")
        .args(["init", "--code", "P-001", "--code-project"])
        .assert()
        .success();
    project
        .command("anna", "2026-11-20T10:00:00+01:00")
        .args(["people", "add", "anna", "--llm", "anna-claude"])
        .assert()
        .success();
    project
}

#[test]
fn a_finished_task_in_a_code_project_gives_no_usage_reminder() {
    let project = code_project();
    write_todo(&project, SHOW_T001);
    project.stdout("anna", &["found", FINDING, "--task", "T-001"]);
    let payload = shell("todo done T-001 --commit HEAD --agent haiku", "");
    let output = run_after(&project, &payload).success();
    let context = context_of(&output.get_output().stdout);
    assert!(context.contains("# Review of T-001"), "{context}");
    assert!(!context.contains("No usage record"), "{context}");
}
