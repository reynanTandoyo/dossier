//! Runs the built program for `brief`: the approved decisions in full, the open assumptions,
//! findings and lessons by title, the drafts line, the marks of a lesson changed by hand and of
//! a dead end, the line limit with 70 decisions, the same output on every run, the server line
//! of remote mode, and `--task` through a stand-in for todo. Phase 2 adds the map first, the key
//! values, the proposals waiting, the `rests on` marks and the heading of what rests on a wrong
//! assumption, `--on <date>` and the order in which the line limit cuts. Each test runs in its own
//! temporary project folder (common::Project), with
//! the clock fixed at 2026-11-20T10:00:00+01:00. Phase 3 adds the team's lessons and how-tos whose
//! tags match the project's, and the news line (R-CTX-1, R-READ-11). The later part adds the
//! `[due for review]` mark of every entry printed, and the changed mark of team how-tos
//! (R-LIFE-6, R-LIFE-7, R-CTX-4).

mod common;

use common::Project;
use predicates::str::contains;
use std::fs;
use std::path::{Path, PathBuf};

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

/// Describes a folder of the code as anna: writes a file in it, so that the folder exists.
fn describe(project: &Project, folder: &str, purpose: &str) {
    project.write(&format!("{folder}/file.txt"), "");
    project
        .run("anna", &["map", "set", folder, purpose])
        .success();
}

/// Finds the assumption `id` wrong as a person at a terminal (DOSSIER_ANSWERS is set), at `now`.
fn refute(project: &Project, now: &str, id: &str) {
    project
        .command("anna", now)
        .env("DOSSIER_ANSWERS", "")
        .args(["refute", id, "--by", "anna, from the rig measurements"])
        .assert()
        .success();
}

/// A project with one decision (DEC-001, task T-042) and one assumption (ASM-001, affecting
/// T-042), as anna on 2026-11-20.
fn project_with_a_decision_resting_on_an_assumption() -> Project {
    let project = Project::new();
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
                "--affects",
                "T-042",
            ],
        )
        .success();
    project
        .run(
            "anna",
            &[
                "decide",
                "Run the last speedline points transient.",
                "--reason",
                "Steady runs stall first.",
                "--title",
                "Transient speedline points",
                "--task",
                "T-042",
            ],
        )
        .success();
    project
}

#[test]
fn the_map_comes_first_after_the_project_lines() {
    let project = project_with_entries();
    describe(
        &project,
        "src/solver",
        "time integration and the linear solver",
    );
    describe(&project, "src/io", "reading the input decks");

    let text = brief(&project, &[], None);

    assert_eq!(
        text.lines().take(4).collect::<Vec<&str>>(),
        vec![
            "Project P-001",
            "## Map",
            "src/io  reading the input decks",
            "src/solver  time integration and the linear solver",
        ],
        "{text}"
    );
    assert_eq!(text.lines().nth(4), Some("## Decisions in force"), "{text}");
}

#[test]
fn the_map_prints_at_most_25_lines_and_says_how_many_are_left_out() {
    let project = Project::new();
    for number in 1..=27 {
        let folder = format!("pkg{number:02}");
        describe(&project, &folder, &format!("the part number {number}"));
    }

    let text = brief(&project, &[], None);

    let lines: Vec<&str> = text.lines().collect();
    let heading = lines
        .iter()
        .position(|line| *line == "## Map")
        .expect("a map heading");
    assert_eq!(lines.len(), heading + 26, "{text}");
    assert_eq!(lines[heading + 1], "pkg01  the part number 1", "{text}");
    assert_eq!(lines[heading + 24], "pkg24  the part number 24", "{text}");
    assert_eq!(
        lines[heading + 25],
        "3 more folders in the map: dossier map",
        "{text}"
    );
}

#[test]
fn key_values_come_first_then_by_name() {
    let project = Project::new();
    project
        .run("anna", &["value", "set", "design.speed", "3900 rpm"])
        .success();
    project
        .run(
            "anna",
            &[
                "value",
                "set",
                "design.gas",
                "air, ideal gas",
                "--dimensionless",
            ],
        )
        .success();
    project
        .run(
            "anna",
            &["value", "set", "design.massflow", "12.4 kg/s", "--key"],
        )
        .success();

    let text = brief(&project, &[], None);

    assert_eq!(
        text.lines().collect::<Vec<&str>>(),
        vec![
            "Project P-001",
            "## Key values",
            "design.massflow = 12.4 kg/s",
            "design.gas = air, ideal gas",
            "design.speed = 3900 rpm",
        ],
        "{text}"
    );
}

#[test]
fn at_most_15_values_are_printed_with_the_key_value_first() {
    let project = Project::new();
    project
        .run("anna", &["value", "set", "design.key", "1 kg", "--key"])
        .success();
    for number in 1..=16 {
        let name = format!("design.v{number:02}");
        project
            .run("anna", &["value", "set", name.as_str(), "2 m"])
            .success();
    }

    let text = brief(&project, &[], None);

    let lines: Vec<&str> = text.lines().collect();
    assert_eq!(lines.len(), 17, "{text}");
    assert_eq!(lines[2], "design.key = 1 kg", "{text}");
    assert_eq!(lines[16], "design.v14 = 2 m", "{text}");
}

#[test]
fn a_proposal_waits_under_its_own_heading_with_its_author_and_is_no_draft() {
    let project = Project::new();
    project
        .run(
            "ben-codex",
            &[
                "propose",
                "Mesh the blades with 1 mm cells.",
                "--reason",
                "The tip gap needs fine cells.",
                "--title",
                "Blade mesh size",
            ],
        )
        .success();

    let text = brief(&project, &[], None);

    assert_eq!(
        text.lines().collect::<Vec<&str>>(),
        vec![
            "Project P-001",
            "## Proposals waiting",
            "DEC-001 · Blade mesh size (ben)",
        ],
        "{text}"
    );
}

#[test]
fn an_entry_resting_on_a_wrong_assumption_is_marked_and_the_assumption_is_not_open() {
    let project = project_with_a_decision_resting_on_an_assumption();
    refute(&project, NOW, "ASM-001");

    let text = brief(&project, &[], None);

    assert!(
        text.lines().any(|line| line
            == "DEC-001 · Transient speedline points (anna, 2026-11-20) [rests on ASM-001]"),
        "{text}"
    );
    assert!(!text.contains("## Open assumptions"), "{text}");
}

#[test]
fn on_a_past_day_an_assumption_was_open_and_the_mark_appears_from_the_day_it_was_found_wrong() {
    let project = project_with_a_decision_resting_on_an_assumption();
    refute(&project, "2026-11-22T09:00:00+01:00", "ASM-001");

    let before = brief(&project, &["--on", "2026-11-21"], None);
    assert!(before.contains("## Open assumptions"), "{before}");
    assert!(
        before
            .lines()
            .any(|line| line == "ASM-001 · Uniform inlet profile"),
        "{before}"
    );
    assert!(
        before
            .lines()
            .any(|line| line == "DEC-001 · Transient speedline points (anna, 2026-11-20)"),
        "{before}"
    );

    let after = brief(&project, &["--on", "2026-11-22"], None);
    assert!(!after.contains("## Open assumptions"), "{after}");
    assert!(
        after.lines().any(|line| line
            == "DEC-001 · Transient speedline points (anna, 2026-11-20) [rests on ASM-001]"),
        "{after}"
    );
}

#[test]
fn on_a_day_before_a_decision_leaves_it_out_and_its_own_day_keeps_it() {
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

    let before = brief(&project, &["--on", "2026-11-19"], None);
    assert!(!before.contains("DEC-001"), "{before}");
    assert!(!before.contains("## Decisions in force"), "{before}");

    let on = brief(&project, &["--on", "2026-11-20"], None);
    assert!(
        on.lines()
            .any(|line| line == "DEC-001 · First plan (anna, 2026-11-20)"),
        "{on}"
    );
}

#[test]
fn on_leaves_out_proposals_and_drafts_and_gives_the_values_of_that_day() {
    let project = Project::new();
    project
        .run(
            "anna",
            &["value", "set", "design.massflow", "12.1 kg/s", "--key"],
        )
        .success();
    project
        .run_at(
            "anna",
            "2026-11-25T09:00:00+01:00",
            &["value", "set", "design.massflow", "12.4 kg/s"],
        )
        .success();
    project
        .run(
            "ben-codex",
            &[
                "propose",
                "Use 1 mm cells.",
                "--reason",
                "The tip gap needs them.",
                "--title",
                "Mesh cells",
            ],
        )
        .success();
    project
        .run(
            "ben-codex",
            &[
                "learn",
                "A draft statement.",
                "--valid-for",
                "Nothing yet.",
                "--evidence",
                "Nothing yet.",
                "--confidence",
                "guess",
                "--title",
                "Draft lesson",
            ],
        )
        .success();

    let now = brief(&project, &[], None);
    assert!(now.contains("## Proposals waiting"), "{now}");
    assert!(now.contains("1 draft is waiting"), "{now}");

    let on = brief(&project, &["--on", "2026-11-21"], None);
    assert!(
        on.lines().any(|line| line == "design.massflow = 12.1 kg/s"),
        "{on}"
    );
    assert!(!on.contains("waiting"), "{on}");
    assert!(!on.contains("Mesh cells"), "{on}");
    assert!(!on.contains("Draft lesson"), "{on}");

    let later = brief(&project, &["--on", "2026-11-26"], None);
    assert!(
        later
            .lines()
            .any(|line| line == "design.massflow = 12.4 kg/s"),
        "{later}"
    );
}

#[test]
fn on_is_refused_with_task_and_a_day_must_be_a_date() {
    let project = Project::new();

    project
        .command("anna", NOW)
        .args(["brief", "--on", "2026-11-20", "--task", "T-042"])
        .assert()
        .failure()
        .code(2);
    project
        .command("anna", NOW)
        .args(["brief", "--on", "20.11.2026"])
        .assert()
        .failure()
        .code(1)
        .stderr(contains("--on needs a date"));
}

#[test]
fn the_json_data_names_the_day_and_the_new_parts() {
    let project = Project::new();
    project
        .run(
            "ben-codex",
            &[
                "propose",
                "Mesh the blades with 1 mm cells.",
                "--reason",
                "The tip gap needs fine cells.",
                "--title",
                "Blade mesh size",
            ],
        )
        .success();
    project
        .run(
            "anna",
            &["value", "set", "design.massflow", "12.4 kg/s", "--key"],
        )
        .success();

    let text = brief(&project, &["--json"], None);
    let envelope: serde_json::Value = serde_json::from_str(text.trim()).expect("one JSON line");
    assert_eq!(envelope["data"]["on"], serde_json::Value::Null, "{text}");
    assert_eq!(
        envelope["data"]["proposals"],
        serde_json::json!(["DEC-001"]),
        "{text}"
    );
    assert_eq!(
        envelope["data"]["values"],
        serde_json::json!(["design.massflow"]),
        "{text}"
    );

    let text = brief(&project, &["--json", "--on", "2026-11-20"], None);
    let envelope: serde_json::Value = serde_json::from_str(text.trim()).expect("one JSON line");
    assert_eq!(envelope["data"]["on"], "2026-11-20", "{text}");
    assert_eq!(
        envelope["data"]["proposals"],
        serde_json::json!([]),
        "{text}"
    );
}

#[test]
fn proposals_are_cut_to_fit_the_limit_before_the_decisions() {
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
    for number in 1..=60 {
        let title = format!("Proposal {number}");
        project
            .run(
                "ben-codex",
                &[
                    "propose",
                    "A proposed decision.",
                    "--reason",
                    "It may be needed.",
                    "--title",
                    title.as_str(),
                ],
            )
            .success();
    }

    let text = brief(&project, &[], None);

    let lines: Vec<&str> = text.lines().collect();
    assert_eq!(lines.len(), 60, "{text}");
    assert_eq!(
        lines.last(),
        Some(&"5 proposals left out: dossier list --status proposed"),
        "{text}"
    );
    assert!(
        lines.contains(&"DEC-001 · First plan (anna, 2026-11-20)"),
        "{text}"
    );
    assert!(lines.contains(&"DEC-056 · Proposal 55 (ben)"), "{text}");
    assert!(!text.contains("DEC-057"), "{text}");
}

#[test]
fn the_map_and_the_values_are_cut_before_any_decision_and_the_decisions_are_shortened_first() {
    let project = Project::new();
    for number in 1..=40 {
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
    for number in 1..=25 {
        describe(
            &project,
            &format!("pkg{number:02}"),
            &format!("part {number}"),
        );
    }
    for number in 1..=15 {
        let name = format!("design.v{number:02}");
        project
            .run("anna", &["value", "set", name.as_str(), "2 m"])
            .success();
    }

    let text = brief(&project, &[], None);

    let lines: Vec<&str> = text.lines().collect();
    assert_eq!(lines.len(), 44, "{text}");
    assert_eq!(
        lines[lines.len() - 2],
        "15 values left out: dossier values",
        "{text}"
    );
    assert_eq!(
        lines.last(),
        Some(&"the map is left out: dossier map"),
        "{text}"
    );
    assert!(
        lines.contains(&"DEC-001 · Decision 1 (anna, 2026-11-20)"),
        "{text}"
    );
    assert!(
        lines.contains(&"DEC-040 · Decision 40 (anna, 2026-11-20)"),
        "{text}"
    );
    assert!(!text.contains("Reason number"), "{text}");
}

/// A project with the file data/model.sim attached to T-001 (as anna), a decision DEC-001 for
/// T-001 and an assumption ASM-001 that affects T-001, as anna on 2026-11-20.
fn project_with_a_file_and_a_decision_resting_on_an_assumption() -> Project {
    let project = Project::new();
    project.write("data/model.sim", "invented model");
    project
        .run(
            "anna",
            &["attach", "T-001", "data/model.sim", "--role", "model"],
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
                "--affects",
                "T-001",
            ],
        )
        .success();
    project
        .run(
            "anna",
            &[
                "decide",
                "Run the last speedline points transient.",
                "--reason",
                "Steady runs stall first.",
                "--title",
                "Transient speedline points",
                "--task",
                "T-001",
            ],
        )
        .success();
    project
}

#[test]
fn what_rests_on_a_wrong_assumption_is_listed_under_its_own_heading() {
    let project = project_with_a_file_and_a_decision_resting_on_an_assumption();
    refute(&project, NOW, "ASM-001");

    let text = brief(&project, &[], None);

    assert_eq!(
        text.lines().collect::<Vec<&str>>(),
        vec![
            "Project P-001",
            "## Decisions in force",
            "DEC-001 · Transient speedline points (anna, 2026-11-20) [rests on ASM-001]",
            "  Run the last speedline points transient.",
            "  Steady runs stall first.",
            "## Rests on a wrong assumption",
            "DEC-001 · Transient speedline points [rests on ASM-001]",
            "T-001 data/model.sim [rests on ASM-001]",
        ],
        "{text}"
    );
    let envelope =
        serde_json::from_str::<serde_json::Value>(brief(&project, &["--json"], None).trim())
            .expect("one JSON line");
    assert_eq!(
        envelope["data"]["rests"],
        serde_json::json!(["DEC-001", "T-001 data/model.sim"]),
    );
}

#[test]
fn a_brief_with_no_such_marks_has_no_such_heading() {
    let project = project_with_entries();
    let text = brief(&project, &[], None);
    assert!(!text.contains("## Rests on a wrong assumption"), "{text}");

    project
        .run(
            "anna",
            &[
                "assume",
                "The gas is ideal.",
                "--reason",
                "No gas data yet.",
                "--title",
                "Ideal gas",
            ],
        )
        .success();
    refute(&project, NOW, "ASM-002");

    let text = brief(&project, &[], None);
    assert!(!text.contains("## Rests on a wrong assumption"), "{text}");
}

#[test]
fn a_detached_file_that_rests_on_a_wrong_assumption_is_not_listed() {
    let project = project_with_a_file_and_a_decision_resting_on_an_assumption();
    refute(&project, NOW, "ASM-001");
    project
        .run("anna", &["detach", "T-001", "data/model.sim"])
        .success();

    let text = brief(&project, &[], None);

    assert!(
        !text
            .lines()
            .any(|line| line.starts_with("T-001 data/model.sim")),
        "{text}"
    );
    assert!(
        text.lines()
            .any(|line| line == "DEC-001 · Transient speedline points [rests on ASM-001]"),
        "{text}"
    );
}

#[test]
fn on_leaves_out_what_rests_on_a_wrong_assumption() {
    let project = project_with_a_file_and_a_decision_resting_on_an_assumption();
    refute(&project, NOW, "ASM-001");

    let now = brief(&project, &[], None);
    assert!(now.contains("## Rests on a wrong assumption"), "{now}");

    let on = brief(&project, &["--on", "2026-11-20"], None);
    assert!(!on.contains("## Rests on a wrong assumption"), "{on}");
    assert!(
        on.lines().any(|line| line
            == "DEC-001 · Transient speedline points (anna, 2026-11-20) [rests on ASM-001]"),
        "{on}"
    );
    assert!(
        !on.lines()
            .any(|line| line.starts_with("T-001 data/model.sim")),
        "{on}"
    );
}

#[test]
fn what_rests_on_a_wrong_assumption_is_cut_after_the_proposals_and_says_how_many_are_left_out() {
    let project = Project::new();
    let mut paths = Vec::new();
    for number in 1..=60 {
        let path = format!("data/part{number:02}.sim");
        project.write(&path, "invented part");
        paths.push(path);
    }
    let mut args = vec!["attach", "T-001"];
    args.extend(paths.iter().map(String::as_str));
    args.extend(["--role", "model"]);
    project.run("anna", &args).success();
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
                "--affects",
                "T-001",
            ],
        )
        .success();
    project
        .run(
            "ben-codex",
            &[
                "propose",
                "Mesh the blades with 1 mm cells.",
                "--reason",
                "The tip gap needs fine cells.",
                "--title",
                "Blade mesh size",
            ],
        )
        .success();
    refute(&project, NOW, "ASM-001");

    let text = brief(&project, &[], None);

    // 60 files rest on the assumption and 1 proposal waits: the proposal is cut first, then 4
    // lines of the files, so 56 file lines and 2 left-out lines make the 60 lines.
    let lines: Vec<&str> = text.lines().collect();
    assert_eq!(lines.len(), 60, "{text}");
    assert_eq!(lines[1], "## Rests on a wrong assumption", "{text}");
    assert_eq!(
        lines
            .iter()
            .filter(|line| line.starts_with("T-001 data/"))
            .count(),
        56,
        "{text}"
    );
    assert!(!text.contains("Blade mesh size"), "{text}");
    assert!(
        lines.contains(&"1 proposal left out: dossier list --status proposed"),
        "{text}"
    );
    assert_eq!(
        lines.last(),
        Some(&"4 lines resting on a wrong assumption left out: dossier list and dossier files"),
        "{text}"
    );
}

/// Invented text for the parts of a team lesson.
const VALID_FOR: &str = "An invented rig with a tip Mach number above 1.1.";
const EVIDENCE: &str = "An invented comparison of two speedlines, section 1.";

/// A team dossier made for a project test, and the project names it. `dossier` is the team's
/// DOSSIER folder; `_folder` keeps the temporary folder that holds it alive.
struct Team {
    _folder: tempfile::TempDir,
    dossier: PathBuf,
}

/// Makes the team dossier with `init --team-dossier --code TEAM` run as anna in a second temporary
/// folder, makes anna its member, and names it in the settings of `project` (with `/` as the
/// separator, so that the TOML text needs no escapes).
fn with_team(project: &Project) -> Team {
    let folder = tempfile::tempdir().expect("a temporary team folder");
    project
        .command("anna", NOW)
        .current_dir(folder.path())
        .args(["init", "--code", "TEAM", "--team-dossier"])
        .assert()
        .success();
    let dossier = folder.path().join("DOSSIER");
    dossier::config::set_list(&dossier, "members", &["anna".to_string()])
        .expect("the member is set");
    let stored = dossier.display().to_string().replace('\\', "/");
    dossier::config::set_value(
        &project.root().join("DOSSIER"),
        "team",
        &format!("\"{stored}\""),
    )
    .expect("the team is named");
    Team {
        _folder: folder,
        dossier,
    }
}

/// Runs `args` against the team dossier as `agent` at the time `now`, from the project folder.
fn in_team(
    project: &Project,
    team: &Team,
    agent: &str,
    now: &str,
    args: &[&str],
) -> assert_cmd::assert::Assert {
    project
        .command(agent, now)
        .arg("--dossier")
        .arg(&team.dossier)
        .args(args)
        .assert()
}

/// Writes a lesson into the team dossier with `learn`: approved when anna writes it, a draft when
/// her LLM does (`agent` "anna - claude"). Returns the assertion of the command.
fn learn_in_team(
    project: &Project,
    team: &Team,
    agent: &str,
    now: &str,
    statement: &str,
    title: &str,
    machine: &str,
) -> assert_cmd::assert::Assert {
    in_team(
        project,
        team,
        agent,
        now,
        &[
            "learn",
            statement,
            "--valid-for",
            VALID_FOR,
            "--evidence",
            EVIDENCE,
            "--confidence",
            "measured",
            "--title",
            title,
            "--machine",
            machine,
        ],
    )
}

/// Writes an approved how-to of the team for `software` as its file (format.md §22): the `howto`
/// command is not built yet, so the test writes what the command would write.
fn write_how_to(team: &Team, id: &str, title: &str, software: &str) {
    let text = format!(
        "<!-- dossier-format: 1 -->\n\
         # {id} · {title}\n\
         \n\
         - kind: howto\n\
         - status: approved\n\
         - created: {NOW}\n\
         - author: anna\n\
         - approved: {NOW} by anna\n\
         - valid-from: {NOW}\n\
         - software: {software}\n\
         - history: {NOW} anna: created\n\
         - history: {NOW} anna: approved\n\
         \n\
         ## Purpose\n\
         \n\
         An invented purpose.\n\
         \n\
         ## Steps\n\
         \n\
         An invented step.\n"
    );
    let folder = team.dossier.join("howtos");
    fs::create_dir_all(&folder).expect("the howtos folder is created");
    let slug = title.to_lowercase().replace(' ', "-");
    fs::write(folder.join(format!("{id}-{slug}.md")), text).expect("the how-to is written");
}

/// The file of entry `id` in `folder` of a dossier: the first file whose name starts with the ID
/// and a dash. Panics when there is none.
fn entry_file_in(dossier: &Path, folder: &str, id: &str) -> PathBuf {
    let prefix = format!("{id}-");
    fs::read_dir(dossier.join(folder))
        .expect("the folder is read")
        .map(|entry| entry.expect("a folder entry").path())
        .find(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with(&prefix))
        })
        .unwrap_or_else(|| panic!("no {id} in {folder}"))
}

/// Sets the project's tags: the machine "axial compressor" and the software "ExampleSolver".
fn tag_project(project: &Project) {
    let path = "DOSSIER/dossier.toml";
    let text = project
        .read(path)
        .replace("machine = []\n", "machine = [\"axial compressor\"]\n")
        .replace("software = []\n", "software = [\"ExampleSolver\"]\n");
    assert!(text.contains("machine = [\"axial compressor\"]"), "{text}");
    project.write(path, &text);
}

/// `brief` as anna at the time `now`, its standard output.
fn brief_now(project: &Project, now: &str) -> String {
    let mut command = project.command("anna", now);
    command.env_remove("SSH_CONNECTION");
    let output = command
        .arg("brief")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    String::from_utf8(output).expect("the standard output is UTF-8")
}

/// `status --since last` as anna at `now`: the marker of anna is written then.
fn mark_seen(project: &Project, now: &str) {
    project
        .run_at("anna", now, &["status", "--since", "last"])
        .success();
}

#[test]
fn a_team_lesson_and_how_to_with_matching_tags_are_listed_under_their_heading() {
    let project = Project::new();
    tag_project(&project);
    let team = with_team(&project);
    learn_in_team(
        &project,
        &team,
        "anna",
        NOW,
        "Steady runs overpredict the stall margin.",
        "Stall margin",
        "axial compressor",
    )
    .success();
    learn_in_team(
        &project,
        &team,
        "anna",
        NOW,
        "Pump curves drift with wear.",
        "Pump wear",
        "pump",
    )
    .success();
    learn_in_team(
        &project,
        &team,
        "anna - claude",
        NOW,
        "A draft of the team that nobody approved.",
        "Draft lesson",
        "axial compressor",
    )
    .success();
    write_how_to(&team, "HOW-001", "Restart the solver", "ExampleSolver");
    write_how_to(&team, "HOW-002", "Other solver", "OtherSolver");

    let text = brief_now(&project, NOW);

    let lines: Vec<&str> = text.lines().collect();
    assert!(lines.contains(&"## Team lessons and how-tos"), "{text}");
    assert!(lines.contains(&"team:LES-001 · Stall margin"), "{text}");
    assert!(
        lines.contains(&"team:HOW-001 · Restart the solver"),
        "{text}"
    );
    for left in ["LES-002", "LES-003", "HOW-002"] {
        assert!(!text.contains(left), "{left} must not be listed: {text}");
    }
}

#[test]
fn at_most_ten_team_lines_are_printed_and_the_rest_is_said() {
    let project = Project::new();
    tag_project(&project);
    let team = with_team(&project);
    for number in 1..=11 {
        learn_in_team(
            &project,
            &team,
            "anna",
            NOW,
            &format!("Lesson number {number} about the stall margin."),
            &format!("Lesson {number}"),
            "axial compressor",
        )
        .success();
    }

    let text = brief_now(&project, NOW);

    assert_eq!(
        text.lines()
            .filter(|line| line.starts_with("team:LES-"))
            .count(),
        10,
        "{text}"
    );
    assert!(
        text.lines()
            .any(|line| line == "1 team entry left out: dossier list --team"),
        "{text}"
    );
}

#[test]
fn the_news_line_appears_after_an_approval_and_not_before() {
    let project = Project::new();
    mark_seen(&project, "2026-11-21T09:00:00+01:00");

    let before = brief_now(&project, "2026-11-21T09:30:00+01:00");
    assert!(!before.contains("dossier status --since last"), "{before}");

    project
        .run_at(
            "anna",
            "2026-11-21T10:00:00+01:00",
            &[
                "found",
                "The mass flow oscillates again.",
                "--title",
                "Oscillation again",
            ],
        )
        .success();
    let after = brief_now(&project, "2026-11-21T11:00:00+01:00");
    assert_eq!(
        after.lines().last(),
        Some("1 new entry since 2026-11-21: dossier status --since last"),
        "{after}"
    );

    // The person's run of `status --since last` moves the marker, and the news is then old.
    mark_seen(&project, "2026-11-21T11:30:00+01:00");
    let again = brief_now(&project, "2026-11-21T12:00:00+01:00");
    assert!(!again.contains("dossier status --since last"), "{again}");
}

#[test]
fn an_approval_in_the_team_counts_as_news_and_a_draft_does_not() {
    let project = Project::new();
    let team = with_team(&project);
    mark_seen(&project, "2026-11-21T09:00:00+01:00");
    learn_in_team(
        &project,
        &team,
        "anna",
        "2026-11-21T10:00:00+01:00",
        "The team's statement.",
        "Team lesson",
        "axial compressor",
    )
    .success();
    learn_in_team(
        &project,
        &team,
        "anna - claude",
        "2026-11-21T10:15:00+01:00",
        "A team draft.",
        "Team draft",
        "axial compressor",
    )
    .success();
    project
        .run_at(
            "anna",
            "2026-11-21T10:30:00+01:00",
            &[
                "found",
                "A finding of the project.",
                "--title",
                "Project finding",
            ],
        )
        .success();

    let text = brief_now(&project, "2026-11-21T11:00:00+01:00");

    assert_eq!(
        text.lines().last(),
        Some("2 new entries since 2026-11-21: dossier status --since last"),
        "{text}"
    );
}

#[test]
fn without_a_marker_there_is_no_news_line() {
    let project = Project::new();
    project
        .run_at(
            "anna",
            "2026-11-21T10:00:00+01:00",
            &[
                "found",
                "A finding of the project.",
                "--title",
                "Project finding",
            ],
        )
        .success();

    let text = brief_now(&project, "2026-11-21T11:00:00+01:00");

    assert!(!text.contains("dossier status --since last"), "{text}");
}

#[test]
fn a_team_lesson_changed_by_hand_after_its_approval_is_marked() {
    let project = Project::new();
    tag_project(&project);
    let team = with_team(&project);
    learn_in_team(
        &project,
        &team,
        "anna",
        NOW,
        "Steady runs overpredict the stall margin.",
        "Stall margin",
        "axial compressor",
    )
    .success();
    let path = entry_file_in(&team.dossier, "lessons", "LES-001");
    let text = fs::read_to_string(&path).expect("the lesson is read");
    fs::write(&path, format!("{text}\nA line added by hand.\n")).expect("the lesson is changed");

    let text = brief_now(&project, NOW);

    assert!(
        text.lines()
            .any(|line| line == "team:LES-001 · Stall margin [changed after approval]"),
        "{text}"
    );
}

#[test]
fn the_json_data_names_the_team_lines_and_the_news() {
    let project = Project::new();
    tag_project(&project);
    let team = with_team(&project);
    mark_seen(&project, "2026-11-21T09:00:00+01:00");
    learn_in_team(
        &project,
        &team,
        "anna",
        "2026-11-21T10:00:00+01:00",
        "Steady runs overpredict the stall margin.",
        "Stall margin",
        "axial compressor",
    )
    .success();
    let mut command = project.command("anna", "2026-11-21T11:00:00+01:00");
    command.env_remove("SSH_CONNECTION");
    let output = command
        .args(["--json", "brief"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let value: serde_json::Value = serde_json::from_slice(&output).expect("the JSON parses");
    assert_eq!(
        value["data"]["team"],
        serde_json::json!(["LES-001"]),
        "{value}"
    );
    assert_eq!(
        value["data"]["news"],
        serde_json::json!({"count": 1, "since": "2026-11-21"}),
        "{value}"
    );
}

#[test]
fn on_a_day_the_team_lines_are_those_in_force_and_no_news_is_given() {
    let project = Project::new();
    tag_project(&project);
    let team = with_team(&project);
    mark_seen(&project, "2026-11-21T09:00:00+01:00");
    learn_in_team(
        &project,
        &team,
        "anna",
        NOW,
        "Steady runs overpredict the stall margin.",
        "Stall margin",
        "axial compressor",
    )
    .success();
    project
        .run_at(
            "anna",
            "2026-11-21T10:00:00+01:00",
            &[
                "found",
                "A finding of the project.",
                "--title",
                "Project finding",
            ],
        )
        .success();

    let mut command = project.command("anna", "2026-11-22T10:00:00+01:00");
    command.env_remove("SSH_CONNECTION");
    let text = String::from_utf8(
        command
            .args(["brief", "--on", "2026-11-20"])
            .assert()
            .success()
            .get_output()
            .stdout
            .clone(),
    )
    .expect("the standard output is UTF-8");

    assert!(
        text.lines()
            .any(|line| line == "team:LES-001 · Stall margin"),
        "{text}"
    );
    assert!(!text.contains("dossier status --since last"), "{text}");
}

#[test]
fn a_team_that_cannot_be_reached_leaves_the_brief_without_a_team_part() {
    let project = Project::new();
    tag_project(&project);
    let stored = project
        .root()
        .join("NOWHERE")
        .join("DOSSIER")
        .display()
        .to_string()
        .replace('\\', "/");
    dossier::config::set_value(
        &project.root().join("DOSSIER"),
        "team",
        &format!("\"{stored}\""),
    )
    .expect("the team is named");

    let text = brief_now(&project, NOW);

    assert!(text.starts_with("Project P-001\n"), "{text}");
    assert!(!text.contains("## Team lessons and how-tos"), "{text}");
}

/// Sets the `review` date of entry `id` as anna at the fixed clock, a person's change.
fn set_review(project: &Project, id: &str, review: &str) {
    let field = format!("review={review}");
    project
        .run("anna", &["set", id, "--field", field.as_str()])
        .success();
}

#[test]
fn entries_the_brief_prints_are_marked_when_their_review_date_has_passed() {
    let project = project_with_entries();
    for id in ["DEC-001", "ASM-001", "FND-001", "LES-001"] {
        set_review(&project, id, "2026-11-19");
    }

    let text = brief_now(&project, NOW);

    let lines: Vec<&str> = text.lines().collect();
    for line in [
        "DEC-001 · Transient speedline points (anna, 2026-11-20) [due for review]",
        "ASM-001 · Uniform inlet profile [due for review]",
        "FND-001 · Steady run stalls at low flow [due for review]",
        "LES-001 · Mixing-plane stall margin [due for review]",
    ] {
        assert!(lines.contains(&line), "{line} is missing: {text}");
    }
}

#[test]
fn a_review_date_of_today_or_later_gives_no_due_mark() {
    let project = project_with_entries();
    set_review(&project, "DEC-001", "2026-11-20");
    set_review(&project, "LES-001", "2026-12-01");

    let text = brief_now(&project, NOW);

    assert!(!text.contains("[due for review]"), "{text}");
    assert!(
        text.lines()
            .any(|line| line == "LES-001 · Mixing-plane stall margin"),
        "{text}"
    );
}

#[test]
fn a_lesson_changed_by_hand_and_past_its_review_date_has_the_due_mark_before_the_changed_mark() {
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
    set_review(&project, "LES-001", "2026-11-19");
    let path = project.entry_file("LES-001");
    let text = fs::read_to_string(&path).expect("the lesson file");
    fs::write(&path, text.replace("overpredict", "underpredict")).expect("change it by hand");

    let text = brief_now(&project, NOW);

    assert!(
        text.lines().any(|line| line
            == "LES-001 · Stall margin lesson [due for review] [changed after approval]"),
        "{text}"
    );
}

/// A draft how-to of the team for `software` (format.md §22), written as the `howto` command would
/// write it, for `approve` to approve.
fn write_draft_how_to(team: &Team, id: &str, title: &str, software: &str) {
    let text = format!(
        "<!-- dossier-format: 1 -->\n\
         # {id} · {title}\n\
         \n\
         - kind: howto\n\
         - status: draft\n\
         - created: {NOW}\n\
         - author: anna\n\
         - software: {software}\n\
         - history: {NOW} anna: created\n\
         \n\
         ## Purpose\n\
         \n\
         An invented purpose.\n\
         \n\
         ## Steps\n\
         \n\
         An invented step.\n"
    );
    let folder = team.dossier.join("howtos");
    fs::create_dir_all(&folder).expect("the howtos folder is created");
    let slug = title.to_lowercase().replace(' ', "-");
    fs::write(folder.join(format!("{id}-{slug}.md")), text).expect("the how-to is written");
}

/// Approves the draft `id` of the team dossier as anna, answering `a` to its question.
fn approve_in_team(project: &Project, team: &Team, id: &str) {
    project
        .command("anna", NOW)
        .arg("--dossier")
        .arg(&team.dossier)
        .env("DOSSIER_ANSWERS", "a")
        .args(["approve", id])
        .assert()
        .success();
}

#[test]
fn a_team_how_to_changed_by_hand_after_its_approval_is_marked() {
    let project = Project::new();
    tag_project(&project);
    let team = with_team(&project);
    write_draft_how_to(&team, "HOW-001", "Restart the solver", "ExampleSolver");
    approve_in_team(&project, &team, "HOW-001");
    let path = entry_file_in(&team.dossier, "howtos", "HOW-001");
    let text = fs::read_to_string(&path).expect("the how-to is read");
    fs::write(&path, format!("{text}\nA line added by hand.\n")).expect("the how-to is changed");

    let text = brief_now(&project, NOW);

    assert!(
        text.lines()
            .any(|line| line == "team:HOW-001 · Restart the solver [changed after approval]"),
        "{text}"
    );
}

#[test]
fn a_team_lesson_and_how_to_past_their_review_date_are_marked_due_for_review() {
    let project = Project::new();
    tag_project(&project);
    let team = with_team(&project);
    learn_in_team(
        &project,
        &team,
        "anna",
        NOW,
        "Steady runs overpredict the stall margin.",
        "Stall margin",
        "axial compressor",
    )
    .success();
    in_team(
        &project,
        &team,
        "anna",
        NOW,
        &["set", "LES-001", "--field", "review=2026-11-19"],
    )
    .success();
    write_how_to(&team, "HOW-001", "Restart the solver", "ExampleSolver");
    let path = entry_file_in(&team.dossier, "howtos", "HOW-001");
    let text = fs::read_to_string(&path).expect("the how-to is read");
    fs::write(
        &path,
        text.replace(
            "- software: ExampleSolver\n",
            "- software: ExampleSolver\n- review: 2026-11-19\n",
        ),
    )
    .expect("the review date is set");

    let text = brief_now(&project, NOW);

    let lines: Vec<&str> = text.lines().collect();
    assert!(
        lines.contains(&"team:LES-001 · Stall margin [due for review]"),
        "{text}"
    );
    assert!(
        lines.contains(&"team:HOW-001 · Restart the solver [due for review]"),
        "{text}"
    );
}
