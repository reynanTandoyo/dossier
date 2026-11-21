//! Runs the built program for `review-task`: the task as `todo show` gives it, the entries
//! recorded for the task, the drafting rules, and nothing changed on disk. Each test runs in its
//! own temporary project folder, with the clock fixed at 2026-11-20T10:00:00+01:00.

mod common;

use common::Project;
use predicates::str::contains;
use std::fs;
use std::path::Path;

/// The envelope the stand-in for todo prints for `todo show T-121 --json`.
const SHOW_T121: &str = r#"{"schema":"todo.show.v1","ok":true,"data":{"id":"T-121","title":"review-task: the material for drafting lessons","status":"open","priority":"P0","difficulty":1,"fields":{"machine":"axial compressor"},"steps":["Give Args","Run the tests"],"notes":[{"date":"2026-11-20","agent":"anna-claude","text":"Started on the material."}]},"warnings":[]}"#;

/// Writes the stand-in for todo: it prints `envelope` and exits with 0, whatever it is given.
fn write_todo(project: &Project, envelope: &str) {
    let windows = format!("@echo off\r\necho {envelope}\r\nexit /b 0\r\n");
    let unix = format!("#!/bin/sh\nprintf '%s\\n' '{envelope}'\nexit 0\n");
    project.write_stand_in("todo", &windows, &unix);
}

/// Every file under `folder` with its bytes, sorted by the path below `folder`.
fn tree(folder: &Path) -> Vec<(String, Vec<u8>)> {
    let mut files = Vec::new();
    collect(folder, folder, &mut files);
    files.sort();
    files
}

/// Adds the files below `folder` to `files`, with paths relative to `base`.
fn collect(base: &Path, folder: &Path, files: &mut Vec<(String, Vec<u8>)>) {
    for entry in fs::read_dir(folder).expect("a folder to read") {
        let path = entry.expect("a folder entry").path();
        if path.is_dir() {
            collect(base, &path, files);
        } else {
            let key = path
                .strip_prefix(base)
                .expect("a path below the base")
                .display()
                .to_string();
            files.push((key, fs::read(&path).expect("a file to read")));
        }
    }
}

#[test]
fn review_task_prints_the_task_its_entries_and_the_drafting_rules() {
    let project = Project::new();
    write_todo(&project, SHOW_T121);
    // The task is written in lower case: the match ignores case.
    project.stdout(
        "anna",
        &[
            "found",
            "The mixing plane converged at lower mass flow.",
            "--task",
            "t-121",
        ],
    );
    project.stdout(
        "ben-codex",
        &[
            "learn",
            "Steady runs need a transient check near stall.",
            "--valid-for",
            "Transonic front stages.",
            "--evidence",
            "Runs 12 and 13.",
            "--confidence",
            "simulated",
            "--task",
            "T-121",
        ],
    );
    project
        .run("anna", &["review-task", "T-121"])
        .success()
        .stdout(contains("# Review of T-121"))
        .stdout(contains(
            "title: review-task: the material for drafting lessons",
        ))
        .stdout(contains("status: open"))
        .stdout(contains("machine: axial compressor"))
        .stdout(contains("1. Give Args"))
        .stdout(contains("2. Run the tests"))
        .stdout(contains("2026-11-20 anna-claude: Started on the material."))
        .stdout(contains("## Entries recorded for this task"))
        .stdout(contains(
            "FND-001 · finding · The mixing plane converged at lower mass flow.",
        ))
        .stdout(contains(
            "LES-001 · lesson · [draft] · Steady runs need a transient check near stall.",
        ))
        .stdout(contains("## Drafting rules"))
        .stdout(contains(
            "1. Propose a lesson only for something that would change how a later project is done.",
        ))
        .stdout(contains("10. Record what had to be assumed."))
        .stdout(contains("dossier learn \"<statement>\""))
        .stdout(contains("dossier found --dead-end"));
}

#[test]
fn review_task_changes_nothing_on_disk() {
    let project = Project::new();
    write_todo(&project, SHOW_T121);
    project.stdout(
        "anna",
        &["found", "The mixing plane converged.", "--task", "T-121"],
    );
    let before = tree(project.root());
    project.stdout("anna", &["review-task", "T-121"]);
    assert_eq!(tree(project.root()), before);
}

#[test]
fn review_task_says_none_when_no_entry_has_the_task() {
    let project = Project::new();
    write_todo(&project, SHOW_T121);
    project.stdout(
        "anna",
        &["found", "A finding of another task.", "--task", "T-999"],
    );
    let text = project.stdout("anna", &["review-task", "T-121"]);
    assert!(
        text.contains("## Entries recorded for this task\nnone\n"),
        "{text}"
    );
    assert!(!text.contains("A finding of another task."), "{text}");
}

#[test]
fn review_task_without_todo_exits_with_5() {
    let project = Project::new();
    // The exit code is what matters; the words differ by system (on Windows a missing .cmd
    // stand-in is run through cmd.exe, which reports the missing file instead).
    project
        .run("anna", &["review-task", "T-121"])
        .code(5)
        .stderr(contains("todo"));
}

#[test]
fn review_task_json_gives_the_task_the_entries_and_the_rules() {
    let project = Project::new();
    write_todo(&project, SHOW_T121);
    project.stdout(
        "anna",
        &["found", "The mixing plane converged.", "--task", "T-121"],
    );
    project
        .run("anna", &["--json", "review-task", "T-121"])
        .success()
        .stdout(contains("\"schema\":\"dossier.review-task.v1\""))
        .stdout(contains("\"id\":\"FND-001\""))
        .stdout(contains("\"rules\":["));
}
