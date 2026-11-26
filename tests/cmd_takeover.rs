//! Runs the built program for `takeover` through a stand-in for todo: a person takes over a task
//! held by a colleague (claim with --force, the note, the journal line, the progress); an open task
//! is claimed without --force; an LLM is refused without --said-by and with it the note ends with
//! the person; a task already held by the writer changes nothing; the files a colleague took for
//! the task are named with the take command; a failing todo writes no journal line. Each test runs
//! in its own temporary project folder (common::Project), with the clock fixed at
//! 2026-11-20T10:00:00+01:00.

mod common;

use common::Project;
use predicates::str::contains;

/// The fixed clock of these tests (the same as common::Project uses).
const NOW: &str = "2026-11-20T10:00:00+01:00";

/// The answer of todo's `show` for T-7: held by `owner`, or open when None.
fn show_envelope(owner: Option<&str>) -> String {
    let (status, owner) = match owner {
        Some(name) => ("claimed", serde_json::json!(name)),
        None => ("open", serde_json::Value::Null),
    };
    serde_json::json!({
        "schema": "todo.show.v1",
        "ok": true,
        "data": {"id": "T-7", "title": "Speedline check", "status": status, "owner": owner},
        "warnings": []
    })
    .to_string()
}

/// Writes the todo stand-in of `project`: `show` answers with T-7 held by `owner`; every other
/// command is recorded in `.test/calls.log` (its arguments) and answered with the envelope
/// `answer`, and exits with `code`.
fn stand_in(project: &Project, owner: Option<&str>, answer: &str, code: u8) {
    let show = show_envelope(owner);
    project.write_stand_in(
        "todo",
        &format!(
            "@echo off\r\n>>\"%~dp0calls.log\" echo %*\r\nif \"%~1\"==\"show\" (\r\necho {show}\r\n) else (\r\necho {answer}\r\nexit /b {code}\r\n)\r\n"
        ),
        &format!(
            "#!/bin/sh\nprintf '%s\\n' \"$*\" >> \"$(dirname \"$0\")/calls.log\"\nif [ \"$1\" = show ]; then\nprintf '%s\\n' '{show}'\nelse\nprintf '%s\\n' '{answer}'\nexit {code}\nfi\n"
        ),
    );
}

/// The answer of todo's claim, note and release commands when they succeed.
fn ok_envelope() -> String {
    serde_json::json!({
        "schema": "todo.claim.v1",
        "ok": true,
        "data": {"id": "T-7"},
        "warnings": []
    })
    .to_string()
}

/// The lines of the todo stand-in's calls, one per call, as the stand-in recorded them.
fn calls(project: &Project) -> String {
    std::fs::read_to_string(project.root().join(".test").join("calls.log")).unwrap_or_default()
}

/// The journal records of takeovers, oldest first.
fn takeovers(project: &Project) -> Vec<dossier::journal::Record> {
    project
        .entry_journal()
        .into_iter()
        .filter(|record| record.command == "takeover")
        .collect()
}

#[test]
fn a_person_takes_over_a_task_held_by_a_colleague_and_the_journal_has_the_line() {
    let project = Project::new();
    stand_in(&project, Some("ben"), &ok_envelope(), 0);

    let text = project.stdout("anna", &["takeover", "T-7"]);

    assert_eq!(
        text.lines().next(),
        Some("T-7 taken over from ben"),
        "{text}"
    );
    // The progress of brief --task follows the takeover, after a blank line.
    assert!(text.contains("\n\nTask T-7 · Speedline check"), "{text}");

    let log = calls(&project);
    assert!(log.contains("claim T-7 --agent anna --force"), "{log}");
    assert!(log.contains("note T-7 "), "{log}");
    assert!(log.contains("--agent anna"), "{log}");

    let records = takeovers(&project);
    assert_eq!(records.len(), 1, "{records:?}");
    let record = &records[0];
    assert_eq!(record.id, "T-7");
    assert_eq!(record.event, "takeover");
    assert_eq!(record.name, "anna");
    assert_eq!(record.person, "anna");
    assert!(!record.llm);
    assert_eq!(record.detail, "takeover — anna took over from ben");
}

#[test]
fn an_open_task_is_claimed_without_force_and_the_note_says_from_nobody() {
    let project = Project::new();
    stand_in(&project, None, &ok_envelope(), 0);

    let text = project.stdout("anna", &["takeover", "T-7"]);

    assert_eq!(
        text.lines().next(),
        Some("T-7 taken over from nobody"),
        "{text}"
    );
    let log = calls(&project);
    assert!(log.contains("claim T-7 --agent anna --json"), "{log}");
    assert!(!log.contains("--force"), "{log}");
    let records = takeovers(&project);
    assert_eq!(records.len(), 1, "{records:?}");
    assert_eq!(records[0].detail, "takeover — anna took over from nobody");
}

#[test]
fn an_llm_without_said_by_is_refused_and_todo_is_not_called() {
    let project = Project::new();
    stand_in(&project, Some("ben"), &ok_envelope(), 0);

    project
        .run("anna-claude", &["takeover", "T-7"])
        .failure()
        .code(1)
        .stderr(contains(
            "ask your person; with their yes: dossier takeover T-7 --said-by <person>",
        ));

    assert!(calls(&project).is_empty(), "todo was called");
    assert!(takeovers(&project).is_empty());
}

#[test]
fn an_llm_with_said_by_writes_the_person_at_the_end_of_the_note() {
    let project = Project::new();
    stand_in(&project, Some("ben"), &ok_envelope(), 0);

    project
        .run("anna-claude", &["takeover", "T-7", "--said-by", "anna"])
        .success();

    let log = calls(&project);
    assert!(
        log.contains("claim T-7 --agent anna-claude --force"),
        "{log}"
    );
    let records = takeovers(&project);
    assert_eq!(records.len(), 1, "{records:?}");
    let record = &records[0];
    assert_eq!(record.name, "anna-claude");
    assert_eq!(record.person, "anna");
    assert!(record.llm);
    assert_eq!(
        record.detail,
        "takeover — anna-claude took over from ben (said by anna)"
    );

    // A name that is not in the settings is refused, as decide refuses it.
    project
        .run(
            "anna-claude",
            &["takeover", "T-7", "--said-by", "nobody-here"],
        )
        .failure()
        .code(1)
        .stderr(contains("no person nobody-here in dossier.toml"));
    assert_eq!(takeovers(&project).len(), 1);
}

#[test]
fn a_task_the_writer_holds_is_already_yours_and_nothing_changes() {
    let project = Project::new();
    stand_in(&project, Some("anna"), &ok_envelope(), 0);

    let text = project.stdout("anna", &["takeover", "T-7"]);

    assert_eq!(text.trim(), "T-7 is already yours", "{text}");
    assert!(!calls(&project).contains("claim"), "{}", calls(&project));
    assert!(takeovers(&project).is_empty());
}

#[test]
fn files_a_colleague_took_for_the_task_are_named_with_the_take_command() {
    let project = Project::new();
    project.write("runs/case-a.dat", "invented data");
    project.write("runs/case b.dat", "invented data");
    stand_in(&project, Some("ben"), &ok_envelope(), 0);
    project
        .run(
            "ben",
            &[
                "take",
                "runs/case-a.dat",
                "runs/case b.dat",
                "--task",
                "T-7",
            ],
        )
        .success();

    let text = project.stdout("anna", &["takeover", "T-7"]);

    let a = dossier::paths::shown("runs/case-a.dat");
    let b = dossier::paths::shown("runs/case b.dat");
    assert!(
        text.contains(&format!(
            "{a}: taken by ben on {NOW} and not returned; changes on their PC are not on the server"
        )),
        "{text}"
    );
    assert!(
        text.contains(&format!(
            "{b}: taken by ben on {NOW} and not returned; changes on their PC are not on the server"
        )),
        "{text}"
    );
    // The take command names every file once; a path with a space is in double quotes.
    let command = format!("take them with: dossier take {a} \"{b}\" --task T-7 --force");
    assert_eq!(text.matches("take them with:").count(), 1, "{text}");
    assert!(text.contains(&command), "{text}");
    // Nothing was copied or taken: the taken file of ben is still ben's.
    assert!(project.read("DOSSIER/taken.md").contains("by ben"));
}

#[test]
fn a_failing_todo_writes_no_journal_line() {
    let project = Project::new();
    let refused = serde_json::json!({
        "schema": "todo.claim.v1",
        "ok": false,
        "error": {"code": "FILES_BUSY", "message": "T-7 is busy in todo"},
        "warnings": []
    })
    .to_string();
    stand_in(&project, Some("ben"), &refused, 1);

    project
        .run("anna", &["takeover", "T-7"])
        .failure()
        .code(1)
        .stderr(contains("T-7 is busy in todo"));

    assert!(takeovers(&project).is_empty());
}
