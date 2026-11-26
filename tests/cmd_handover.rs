//! Runs the built program for `handover`: the note `handover — <text>` is added in todo and the
//! task is released there, the journal records the event; a writer who holds files taken for the
//! task is refused and todo is not called; a todo that fails writes no journal line. todo is a
//! stand-in that records its arguments in calls.txt of the project folder. Each test runs in its
//! own temporary project folder with the clock fixed by `common::Project` (2026-11-20T10:00:00+01:00).

mod common;

use common::Project;
use dossier::journal::Record;
use dossier::paths::shown;
use predicates::str::contains;

/// The fixed clock of `common::Project`, as it appears in history lines.
const WHEN: &str = "2026-11-20T10:00:00+01:00";

/// The task of the tests (invented).
const TASK: &str = "T-001";

/// The note of the tests (invented).
const NOTE: &str = "Done: mesh; next: run 3";

/// The stored path of the file that a writer takes in the refusal test.
const PATH: &str = "runs/case-a.dat";

/// The answer of a todo that accepts a call (invented).
const OK_ANSWER: &str = r#"{"ok":true,"data":{}}"#;

/// The answer of a todo that refuses a call (invented).
const REFUSED_ANSWER: &str =
    r#"{"ok":false,"error":{"code":"NOT_FOUND","message":"no task T-001 in TODO.md"}}"#;

/// The stand-in for todo: appends its arguments to calls.txt in the project folder, prints
/// `answer` and exits with `code`. The answers have no character that the batch file reads as
/// syntax. On Windows the arguments are written by `cmd /u`, which keeps the dash of a note; a plain
/// `echo` writes it in the console code page, where it is lost.
fn todo_stand_in(project: &Project, answer: &str, code: u8) {
    let windows = format!(
        "@echo off\r\ncmd /u /c \"echo %* >> calls.txt\"\r\necho {answer}\r\nexit /b {code}\r\n"
    );
    let unix =
        format!("#!/bin/sh\necho \"$@\" >> calls.txt\nprintf '%s\\n' '{answer}'\nexit {code}\n");
    project.write_stand_in("todo", &windows, &unix);
}

/// The calls that the todo stand-in recorded, one line per call, in order. The file is UTF-16 on
/// Windows (see `todo_stand_in`) and UTF-8 elsewhere.
fn todo_calls(project: &Project) -> Vec<String> {
    let bytes = std::fs::read(project.root().join("calls.txt")).unwrap_or_default();
    let text = if cfg!(windows) {
        let (pairs, _) = bytes.as_chunks::<2>();
        let units: Vec<u16> = pairs.iter().map(|pair| u16::from_le_bytes(*pair)).collect();
        String::from_utf16_lossy(&units)
    } else {
        String::from_utf8_lossy(&bytes).into_owned()
    };
    text.trim_start_matches('\u{feff}')
        .lines()
        .map(|line| line.trim_end().to_string())
        .collect()
}

/// The journal records of the task, oldest first.
fn handover_journal(project: &Project) -> Vec<Record> {
    project
        .entry_journal()
        .into_iter()
        .filter(|record| record.id == TASK)
        .collect()
}

#[test]
fn handover_adds_the_note_releases_the_task_and_journals_it() {
    let project = Project::new();
    todo_stand_in(&project, OK_ANSWER, 0);

    let output = project.stdout("anna", &["handover", TASK, "--note", NOTE]);
    assert_eq!(output, format!("{TASK} handed over: handover — {NOTE}\n"));

    let calls = todo_calls(&project);
    assert_eq!(calls.len(), 2, "{calls:?}");
    assert!(calls[0].starts_with("note T-001"), "{calls:?}");
    assert!(
        calls[0].contains(&format!("handover — {NOTE}")),
        "{calls:?}"
    );
    assert!(calls[0].contains("--agent anna"), "{calls:?}");
    assert!(calls[1].starts_with("release T-001"), "{calls:?}");
    assert!(calls[1].contains("--agent anna"), "{calls:?}");

    let journal = handover_journal(&project);
    assert_eq!(journal.len(), 1);
    assert_eq!(journal[0].event, "handover");
    assert_eq!(journal[0].detail, NOTE);
    assert_eq!(journal[0].name, "anna");
    assert_eq!(journal[0].person, "anna");
    assert!(!journal[0].llm);
    assert_eq!(journal[0].time, WHEN);
}

#[test]
fn todo_gets_the_todo_agent_name_and_the_journal_keeps_the_writer() {
    let project = Project::new();
    todo_stand_in(&project, OK_ANSWER, 0);

    project
        .command("anna", WHEN)
        .env("TODO_AGENT", "anna-worker")
        .args(["handover", TASK, "--note", NOTE])
        .assert()
        .success();

    let calls = todo_calls(&project);
    assert_eq!(calls.len(), 2, "{calls:?}");
    assert!(calls[0].contains("--agent anna-worker"), "{calls:?}");
    assert!(calls[1].contains("--agent anna-worker"), "{calls:?}");
    let journal = handover_journal(&project);
    assert_eq!(journal[0].name, "anna");
}

#[test]
fn an_empty_todo_agent_is_not_a_name_and_the_writer_is_given_to_todo() {
    let project = Project::new();
    todo_stand_in(&project, OK_ANSWER, 0);

    project
        .command("anna", WHEN)
        .env("TODO_AGENT", "")
        .args(["handover", TASK, "--note", NOTE])
        .assert()
        .success();

    let calls = todo_calls(&project);
    assert!(calls[0].contains("--agent anna"), "{calls:?}");
    assert!(!calls[0].contains("--agent anna-worker"), "{calls:?}");
}

#[test]
fn a_writer_holding_a_file_for_the_task_is_refused_and_todo_is_not_called() {
    let project = Project::new();
    todo_stand_in(&project, OK_ANSWER, 0);
    project.write(PATH, "invented simulation data\n");
    project.stdout("anna", &["take", PATH, "--task", TASK]);

    project
        .run("anna", &["handover", TASK, "--note", NOTE])
        .code(1)
        .stderr(contains(format!(
            "{TASK}: return the files you took first:\n  {} (taken {WHEN})",
            shown(PATH)
        )))
        .stderr(contains(format!(
            "dossier return {} --done (or --abandon to drop your changes)",
            shown(PATH)
        )));

    assert!(todo_calls(&project).is_empty(), "todo was not called");
    assert!(handover_journal(&project).is_empty());
}

#[test]
fn a_todo_that_refuses_makes_handover_fail_and_writes_no_journal_line() {
    let project = Project::new();
    todo_stand_in(&project, REFUSED_ANSWER, 1);

    project
        .run("anna", &["handover", TASK, "--note", NOTE])
        .code(1)
        .stderr(contains("no task T-001 in TODO.md"));

    let calls = todo_calls(&project);
    assert_eq!(
        calls.len(),
        1,
        "release is not called after a refused note: {calls:?}"
    );
    assert!(handover_journal(&project).is_empty());
}
