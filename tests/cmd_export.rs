//! Runs the built program for `export`: the approved decisions, findings, investigations and usage
//! records as one Markdown document in time order, with their sections (a draft is left out), `--to`
//! (a file outside the dossier, with a name that has spaces and non-ASCII letters, replaced when it
//! exists), `--milestone` (the tasks todo lists under it), the refusals, and the journal left as it
//! was. Each test runs in its own temporary project folder with the clock fixed; the milestone, the
//! tasks, the entries and their texts are invented.

mod common;

use common::Project;
use serde_json::{Value, json};

/// The time the document is exported at (the day after the last entry).
const EXPORT_AT: &str = "2026-11-24T08:00:00+01:00";

/// The answer of todo to `list --milestone T-900 --status all` (invented): one task, T-901.
const LIST: &str = r#"{"schema":"todo.list.v1","ok":true,"data":{"tasks":[{"id":"T-901","title":"Invented first task","status":"done","owner":null,"ready":false,"priority":"P1"}]},"warnings":[]}"#;

/// The stand-in of todo: it answers every call with `json`, after `--version` (exit 0).
fn todo_stand_in(project: &Project, json: &str) {
    let windows =
        format!("@echo off\r\nif \"%~1\"==\"--version\" exit /b 0\r\necho {json}\r\nexit /b 0\r\n");
    let unix = format!(
        "#!/bin/sh\nif [ \"$1\" = \"--version\" ]; then exit 0; fi\nprintf '%s\\n' '{json}'\n"
    );
    project.write_stand_in("todo", &windows, &unix);
}

/// The standard output of a run as `agent` at `now`, asserted to succeed.
fn stdout_at(project: &Project, agent: &str, now: &str, args: &[&str]) -> String {
    let output = project
        .run_at(agent, now, args)
        .success()
        .get_output()
        .stdout
        .clone();
    String::from_utf8(output).expect("the standard output is UTF-8")
}

/// The document `export` prints at EXPORT_AT.
fn document(project: &Project) -> String {
    stdout_at(project, "anna", EXPORT_AT, &["export"])
}

/// A project with a problem (INV-001), a decision (DEC-001), a lesson drafted by an LLM (LES-001, a
/// draft that must not be exported), a finding (FND-001) and a usage record (USE-001), each written
/// at its own time and all for the task T-901 (invented).
fn project_with_entries() -> Project {
    let project = Project::new();
    project
        .run_at(
            "anna",
            "2026-11-19T09:00:00+01:00",
            &[
                "problem",
                "The density residual grows after iteration 40",
                "--task",
                "T-901",
            ],
        )
        .success();
    project
        .run_at(
            "anna",
            "2026-11-20T10:00:00+01:00",
            &[
                "decide",
                "Keep three speedline points",
                "--reason",
                "Steady runs converged where stall shows.",
                "--rejected",
                "A finer mesh: same result, three times the cost.",
                "--title",
                "Three speedline points",
                "--task",
                "T-901",
            ],
        )
        .success();
    project
        .run_at(
            "anna-claude",
            "2026-11-21T09:00:00+01:00",
            &[
                "learn",
                "Steady mixing-plane runs overpredict the stall margin",
                "--valid-for",
                "Transonic front stages",
                "--evidence",
                "Report section 4.2",
                "--confidence",
                "simulated",
                "--title",
                "Mixing plane stall margin",
                "--task",
                "T-901",
            ],
        )
        .success();
    project
        .run_at(
            "anna",
            "2026-11-22T11:00:00+01:00",
            &[
                "found",
                "The mass flow still oscillates by 0.4 percent with half the time step",
                "--evidence",
                "Runs 12 and 13",
                "--title",
                "Halving the time step",
                "--task",
                "T-901",
            ],
        )
        .success();
    project
        .run_at(
            "anna",
            "2026-11-23T14:00:00+01:00",
            &[
                "used",
                "ExampleSolver",
                "--version",
                "2406",
                "--task",
                "T-901",
                "--param",
                "massflow=12.4",
                "--deviations",
                "The last three points run transient",
                "--title",
                "ExampleSolver speedline run",
            ],
        )
        .success();
    project
}

#[test]
fn the_approved_entries_come_in_time_order_with_their_sections_and_a_draft_does_not() {
    let project = project_with_entries();
    let journal = project.read("DOSSIER/.state/journal.jsonl");
    let text = document(&project);
    assert_eq!(text, EXPECTED, "the document");
    assert_eq!(project.read("DOSSIER/.state/journal.jsonl"), journal);
}

/// The document of `project_with_entries` (invented), exported at EXPORT_AT.
const EXPECTED: &str = r"# P-001: decisions, findings and runs

exported 2026-11-24

## INV-001 · The density residual grows after iteration 40

investigation, 2026-11-19, anna, task T-901

### Problem

The density residual grows after iteration 40

## DEC-001 · Three speedline points

decision, 2026-11-20, anna, task T-901

### Decision

Keep three speedline points

### Reason

Steady runs converged where stall shows.

### Rejected

A finer mesh: same result, three times the cost.

## FND-001 · Halving the time step

finding, 2026-11-22, anna, task T-901

### Observation

The mass flow still oscillates by 0.4 percent with half the time step

### Evidence

Runs 12 and 13

## USE-001 · ExampleSolver speedline run

usage, 2026-11-23, anna, task T-901, software ExampleSolver, version 2406

### Parameters

massflow = 12.4

### Deviations

The last three points run transient
";

#[test]
fn the_entries_are_listed_in_a_document_that_has_a_line_for_each_kind_of_entry() {
    let project = project_with_entries();
    let text = document(&project);
    let order: Vec<usize> = ["## INV-001", "## DEC-001", "## FND-001", "## USE-001"]
        .iter()
        .map(|heading| text.find(heading).expect(heading))
        .collect();
    assert!(order.windows(2).all(|pair| pair[0] < pair[1]), "{text}");
    assert!(!text.contains("LES-001"), "{text}");
    assert!(!text.contains("Mixing plane"), "{text}");
}

#[test]
fn to_writes_the_document_outside_the_dossier_and_says_how_many_entries() {
    let project = project_with_entries();
    let document = document(&project);
    let journal = project.read("DOSSIER/.state/journal.jsonl");
    let file = "Bericht Übersicht/export Ä.md";
    project.write(file, "an old document\n");
    let text = stdout_at(&project, "anna", EXPORT_AT, &["export", "--to", file]);
    assert_eq!(
        text,
        "exported 4 entries to Bericht Übersicht/export Ä.md\n"
    );
    assert_eq!(project.read(file), document);
    assert_eq!(project.read("DOSSIER/.state/journal.jsonl"), journal);
}

#[test]
fn json_gives_the_count_and_the_file_and_no_document_text() {
    let project = project_with_entries();
    let out = stdout_at(
        &project,
        "anna",
        EXPORT_AT,
        &["export", "--to", "export.md", "--json"],
    );
    let value: Value = serde_json::from_str(out.trim_end()).expect("one JSON line");
    assert_eq!(value["schema"], "dossier.export.v1");
    assert_eq!(value["ok"], true);
    assert_eq!(value["data"], json!({"entries": 4, "file": "export.md"}));
    assert_eq!(value["warnings"], json!([]));
}

#[test]
fn an_empty_dossier_gives_the_title_and_the_export_day_only() {
    let project = Project::new();
    assert_eq!(
        stdout_at(&project, "anna", EXPORT_AT, &["export"]),
        "# P-001: decisions, findings and runs\n\nexported 2026-11-24\n"
    );
}

#[test]
fn export_needs_no_name() {
    let project = project_with_entries();
    assert_eq!(
        stdout_at(&project, "", EXPORT_AT, &["export"]),
        document(&project)
    );
}

#[test]
fn milestone_keeps_the_entries_of_the_tasks_todo_lists_under_it() {
    let project = project_with_entries();
    project
        .run_at(
            "anna",
            "2026-11-23T15:00:00+01:00",
            &[
                "decide",
                "Use the other mesh for the invented case",
                "--reason",
                "Invented reason of another task.",
                "--title",
                "Other mesh",
                "--task",
                "T-777",
            ],
        )
        .success();
    todo_stand_in(&project, LIST);
    assert!(document(&project).contains("## DEC-002"));

    let text = stdout_at(
        &project,
        "anna",
        EXPORT_AT,
        &["export", "--milestone", "T-900"],
    );
    for heading in ["## INV-001", "## DEC-001", "## FND-001", "## USE-001"] {
        assert!(text.contains(heading), "{heading} in {text}");
    }
    assert!(!text.contains("DEC-002"), "{text}");
}

#[test]
fn milestone_without_todo_is_refused_with_todo_missing() {
    let project = project_with_entries();
    project
        .run_at("anna", EXPORT_AT, &["export", "--milestone", "T-900"])
        .code(5)
        .stderr(predicates::str::contains("error:"));
}

#[test]
fn a_file_inside_the_dossier_is_refused_and_nothing_is_written() {
    let project = project_with_entries();
    let journal = project.read("DOSSIER/.state/journal.jsonl");
    project
        .run_at(
            "anna",
            EXPORT_AT,
            &["export", "--to", "DOSSIER/exported.md"],
        )
        .code(1)
        .stderr(predicates::str::contains("inside the dossier"));
    assert!(!project.root().join("DOSSIER/exported.md").exists());
    assert_eq!(project.read("DOSSIER/.state/journal.jsonl"), journal);
}

#[test]
fn dry_run_says_what_it_would_export_and_writes_no_file() {
    let project = project_with_entries();
    let text = stdout_at(
        &project,
        "anna",
        EXPORT_AT,
        &["export", "--to", "export.md", "--dry-run"],
    );
    assert_eq!(text, "dry run: would export 4 entries to export.md\n");
    assert!(!project.root().join("export.md").exists());
}
