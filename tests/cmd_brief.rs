//! Runs the built program for `brief`: the approved decisions in full, the open assumptions,
//! findings and lessons by title, the drafts line, the marks of a lesson changed by hand and of
//! a dead end, the line limit with 70 decisions, the same output on every run, the server line
//! of remote mode, and `--task` through a stand-in for todo. Each test runs in its own temporary
//! project folder (common::Project), with the clock fixed at 2026-11-20T10:00:00+01:00.

mod common;

use common::Project;
use predicates::str::contains;
use std::fs;

/// The fixed clock of these tests (the same as common::Project uses).
const NOW: &str = "2026-11-20T10:00:00+01:00";

/// An ssh connection string as sshd sets SSH_CONNECTION (invented addresses).
const SSH: &str = "10.0.0.5 50000 10.0.0.9 22";

/// Runs `brief` with `extra` arguments as anna at the fixed clock and returns the standard
/// output. The variable SSH_CONNECTION is removed, unless `ssh` gives it a value.
fn brief(project: &Project, extra: &[&str], ssh: Option<&str>) -> String {
    let mut command = project.command("anna", NOW);
    command.env_remove("SSH_CONNECTION");
    if let Some(connection) = ssh {
        command.env("SSH_CONNECTION", connection);
    }
    let output = command
        .arg("brief")
        .args(extra)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    String::from_utf8(output).expect("the standard output is UTF-8")
}

/// A project with one approved decision, one open assumption, one approved finding, one
/// approved lesson and one draft lesson written by ben-codex.
fn project_with_entries() -> Project {
    let project = Project::new();
    project
        .run(
            "anna",
            &[
                "decide",
                "Run the last speedline points transient.",
                "--reason",
                "Steady runs stall before the transient run does.",
                "--title",
                "Transient speedline points",
            ],
        )
        .success();
    project
        .run(
            "anna",
            &[
                "assume",
                "The inlet profile is uniform.",
                "--reason",
                "No measured profile was delivered.",
                "--title",
                "Uniform inlet profile",
            ],
        )
        .success();
    project
        .run(
            "anna",
            &[
                "found",
                "The steady run stalled at the lowest mass flow.",
                "--title",
                "Steady run stalls at low flow",
            ],
        )
        .success();
    project
        .run(
            "anna",
            &[
                "learn",
                "Steady mixing-plane runs overpredict the stall margin.",
                "--valid-for",
                "Transonic front stages.",
                "--evidence",
                "Speedline comparison, section 4.2.",
                "--confidence",
                "simulated",
                "--title",
                "Mixing-plane stall margin",
            ],
        )
        .success();
    project
        .run(
            "ben-codex",
            &[
                "learn",
                "A coarse mesh overpredicts the stall margin.",
                "--valid-for",
                "Coarse meshes only.",
                "--evidence",
                "An invented mesh study.",
                "--confidence",
                "guess",
                "--title",
                "Coarse mesh margin draft",
            ],
        )
        .success();
    project
}

#[test]
fn the_brief_gives_decisions_in_full_and_the_other_approved_entries_by_title() {
    let project = project_with_entries();

    let text = brief(&project, &[], None);

    assert_eq!(
        text.lines().collect::<Vec<&str>>(),
        vec![
            "Project P-001",
            "## Decisions in force",
            "DEC-001 · Transient speedline points (anna, 2026-11-20)",
            "  Run the last speedline points transient.",
            "  Steady runs stall before the transient run does.",
            "## Open assumptions",
            "ASM-001 · Uniform inlet profile",
            "## Findings",
            "FND-001 · Steady run stalls at low flow",
            "## Lessons",
            "LES-001 · Mixing-plane stall margin",
            "1 draft is waiting: dossier drafts",
        ]
    );
}

#[test]
fn a_draft_and_a_superseded_decision_are_never_printed() {
    let project = Project::new();
    project
        .run(
            "anna",
            &[
                "decide",
                "Run the first speedline plan.",
                "--reason",
                "It was the first plan.",
                "--title",
                "First plan",
            ],
        )
        .success();
    project
        .run(
            "anna",
            &[
                "decide",
                "Run the second speedline plan.",
                "--reason",
                "It replaces the first plan.",
                "--title",
                "Second plan",
            ],
        )
        .success();
    project
        .run("anna", &["supersede", "DEC-001", "--by", "DEC-002"])
        .success();
    project
        .run(
            "ben-codex",
            &[
                "learn",
                "An unapproved statement.",
                "--valid-for",
                "Nothing yet.",
                "--evidence",
                "Nothing yet.",
                "--confidence",
                "guess",
                "--title",
                "Unapproved lesson title",
            ],
        )
        .success();
    project
        .run(
            "anna-claude",
            &[
                "learn",
                "Another unapproved statement.",
                "--valid-for",
                "Nothing yet.",
                "--evidence",
                "Nothing yet.",
                "--confidence",
                "guess",
                "--title",
                "Second unapproved lesson",
            ],
        )
        .success();

    let text = brief(&project, &[], None);

    assert!(
        text.contains("DEC-002 · Second plan (anna, 2026-11-20)"),
        "{text}"
    );
    assert!(!text.contains("First plan"), "{text}");
    assert!(!text.contains("Run the first speedline plan."), "{text}");
    assert!(!text.contains("Unapproved lesson title"), "{text}");
    assert!(!text.contains("Second unapproved lesson"), "{text}");
    assert!(
        text.contains("2 drafts are waiting: dossier drafts"),
        "{text}"
    );
}

#[test]
fn with_70_decisions_the_brief_has_at_most_60_lines_and_ends_with_the_left_out_line() {
    let project = Project::new();
    for number in 1..=70 {
        let decision = format!("Decision number {number} of the invented project.");
        let reason = format!("Reason number {number} of the invented project.");
        let title = format!("Decision {number}");
        project
            .run(
                "anna",
                &[
                    "decide",
                    decision.as_str(),
                    "--reason",
                    reason.as_str(),
                    "--title",
                    title.as_str(),
                ],
            )
            .success();
    }

    let text = brief(&project, &[], None);

    let lines: Vec<&str> = text.lines().collect();
    assert!(lines.len() <= 60, "{} lines:\n{text}", lines.len());
    assert_eq!(
        lines.last(),
        Some(&"13 entries left out: dossier list"),
        "{text}"
    );
    // All 70 are shortened to their title lines (72 lines). 13 must go: 1 project line, 1
    // heading, 57 decisions and the left-out line make 60. The newest 57 are kept.
    assert!(
        text.contains("DEC-070 · Decision 70 (anna, 2026-11-20)"),
        "{text}"
    );
    assert!(
        text.contains("DEC-014 · Decision 14 (anna, 2026-11-20)"),
        "{text}"
    );
    assert!(!text.contains("DEC-013 ·"), "{text}");
    assert!(!text.contains("Reason number 70"), "{text}");
}

#[test]
fn two_runs_give_identical_output() {
    let project = project_with_entries();

    let first = brief(&project, &[], None);
    let second = brief(&project, &[], None);

    assert_eq!(first, second);
}

#[test]
fn an_approved_lesson_changed_by_hand_is_marked() {
    let project = Project::new();
    project
        .run(
            "anna",
            &[
                "learn",
                "Steady runs overpredict the stall margin.",
                "--valid-for",
                "Transonic front stages.",
                "--evidence",
                "Speedline comparison, section 4.2.",
                "--confidence",
                "simulated",
                "--title",
                "Stall margin lesson",
            ],
        )
        .success();
    project
        .run(
            "anna",
            &[
                "learn",
                "Steady runs converge at low mass flow.",
                "--valid-for",
                "Subsonic rear stages.",
                "--evidence",
                "Speedline comparison, section 5.",
                "--confidence",
                "experience",
                "--title",
                "Unchanged lesson",
            ],
        )
        .success();
    let path = project.entry_file("LES-001");
    let text = fs::read_to_string(&path).expect("the lesson file");
    fs::write(&path, text.replace("overpredict", "underpredict")).expect("change it by hand");

    let text = brief(&project, &[], None);

    assert!(
        text.lines()
            .any(|line| line == "LES-001 · Stall margin lesson [changed after approval]"),
        "{text}"
    );
    assert!(
        text.lines()
            .any(|line| line == "LES-002 · Unchanged lesson"),
        "{text}"
    );
}

#[test]
fn a_dead_end_finding_is_marked() {
    let project = Project::new();
    project
        .run(
            "anna",
            &[
                "found",
                "Halving the time step kept the oscillation.",
                "--dead-end",
                "--title",
                "Smaller time step",
            ],
        )
        .success();

    let text = brief(&project, &[], None);

    assert!(
        text.lines()
            .any(|line| line == "FND-001 · Smaller time step (dead end)"),
        "{text}"
    );
}

#[test]
fn in_remote_mode_the_first_line_names_the_project_and_the_server_folder() {
    let project = project_with_entries();

    let text = brief(&project, &[], Some(SSH));

    let expected = format!(
        "Project P-001 on the server in {}",
        project.root().display()
    );
    assert_eq!(text.lines().next(), Some(expected.as_str()), "{text}");
    assert!(!text.lines().any(|line| line == "Project P-001"), "{text}");
    assert!(
        text.contains("DEC-001 · Transient speedline points"),
        "{text}"
    );
}

#[test]
fn brief_task_needs_todo_and_says_so() {
    let project = Project::new();

    project
        .command("anna", NOW)
        .env_remove("SSH_CONNECTION")
        .args(["brief", "--task", "T-132"])
        .assert()
        .failure()
        .code(5)
        .stderr(contains("brief --task needs todo"));
}

#[test]
fn brief_task_prints_the_task_from_todo_and_its_recorded_decision() {
    let project = Project::new();
    project
        .run(
            "anna",
            &[
                "decide",
                "Run the first speedline plan.",
                "--reason",
                "It was the first plan.",
                "--task",
                "T-132",
                "--title",
                "Speedline plan",
            ],
        )
        .success();
    let data = serde_json::json!({"id": "T-132", "title": "Speedline check", "status": "open"});
    let envelope = serde_json::json!({
        "schema": "todo.show.v1",
        "ok": true,
        "data": data,
        "warnings": []
    })
    .to_string();
    project.write_stand_in(
        "todo",
        &format!("@echo off\r\necho {envelope}\r\n"),
        &format!("#!/bin/sh\nprintf '%s\\n' '{envelope}'\n"),
    );

    let text = brief(&project, &["--task", "T-132"], None);
    assert_eq!(
        text.lines().next(),
        Some("Task T-132 · Speedline check"),
        "{text}"
    );
    assert!(
        text.contains("DEC-001 · decision · Speedline plan"),
        "{text}"
    );

    let text = brief(&project, &["--task", "T-132"], Some(SSH));
    let expected = format!(
        "Project P-001 on the server in {}",
        project.root().display()
    );
    assert_eq!(text.lines().next(), Some(expected.as_str()), "{text}");
    assert_eq!(
        text.lines().nth(1),
        Some("Task T-132 · Speedline check"),
        "{text}"
    );
}
