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
//! (R-LIFE-6, R-LIFE-7, R-CTX-4). The last part adds the open investigations under their heading,
//! after the open assumptions, with their number of attempts and the last attempt (R-INV-5), and
//! their place in the order in which the line limit cuts. The digest part adds the approved digests
//! in force under `## Digests`, the decisions and findings they sum up left out, one count line per
//! digest after them, and the rule that a digest line is never cut (R-CTX-15). The last part adds
//! `--budget <lines>`, the line limit of one call for the project brief and for `--task`, at least
//! 5 lines (R-CTX-6).

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

/// Opens an investigation as anna with the title `title`; its problem is invented.
fn open_investigation(project: &Project, title: &str) {
    project
        .run(
            "anna",
            &[
                "problem",
                "The density residual grows after 40 iterations.",
                "--title",
                title,
            ],
        )
        .success();
}

/// Records one attempt of the investigation `id` as anna: the hypothesis `text` and a verdict flag.
fn try_it(project: &Project, id: &str, text: &str, verdict: &str) {
    project.run("anna", &["tried", id, text, verdict]).success();
}

#[test]
fn an_open_investigation_is_listed_with_its_attempts_and_its_last_one() {
    let project = Project::new();
    open_investigation(&project, "Residual blows up");
    try_it(
        &project,
        "INV-001",
        "The CFL number is too high",
        "--failed",
    );
    try_it(
        &project,
        "INV-001",
        "The mesh is too coarse at the leading edge",
        "--failed",
    );

    let text = brief(&project, &[], None);

    assert_eq!(
        text.lines().collect::<Vec<&str>>(),
        vec![
            "Project P-001",
            "## Open investigations",
            "INV-001 · Residual blows up — 2 attempts, last: 2. failed: The mesh is too coarse at the leading edge",
        ],
        "{text}"
    );
}

#[test]
fn a_solved_or_given_up_investigation_is_left_out_and_one_without_an_attempt_says_so() {
    let project = Project::new();
    open_investigation(&project, "Solved one");
    try_it(
        &project,
        "INV-001",
        "The flux has the wrong sign",
        "--worked",
    );
    project
        .run(
            "anna",
            &["solved", "INV-001", "The outlet flux had the wrong sign."],
        )
        .success();
    open_investigation(&project, "Given up one");
    project
        .run(
            "anna",
            &["gave-up", "INV-002", "--reason", "No data to test it with."],
        )
        .success();
    open_investigation(&project, "Open one");

    let text = brief(&project, &[], None);

    assert_eq!(
        text.lines().collect::<Vec<&str>>(),
        vec![
            "Project P-001",
            "## Open investigations",
            "INV-003 · Open one — no attempt yet",
        ],
        "{text}"
    );
}

#[test]
fn the_open_investigations_come_after_the_open_assumptions_and_before_the_findings() {
    let project = project_with_entries();
    open_investigation(&project, "Residual blows up");

    let text = brief(&project, &[], None);

    let position = |heading: &str| {
        text.lines()
            .position(|line| line == heading)
            .unwrap_or_else(|| panic!("no {heading} in:\n{text}"))
    };
    assert!(
        position("## Open assumptions") < position("## Open investigations"),
        "{text}"
    );
    assert!(
        position("## Open investigations") < position("## Findings"),
        "{text}"
    );
}

#[test]
fn the_json_data_names_the_open_investigations_only() {
    let project = Project::new();
    open_investigation(&project, "Residual blows up");
    try_it(
        &project,
        "INV-001",
        "The CFL number is too high",
        "--failed",
    );
    open_investigation(&project, "Solved one");
    project
        .run(
            "anna",
            &["solved", "INV-002", "The flux had the wrong sign."],
        )
        .success();

    let text = brief(&project, &["--json"], None);

    let envelope: serde_json::Value = serde_json::from_str(text.trim()).expect("one JSON line");
    assert_eq!(
        envelope["data"]["investigations"],
        serde_json::json!(["INV-001"]),
        "{text}"
    );
}

#[test]
fn a_brief_without_open_investigations_has_no_such_heading() {
    let project = project_with_entries();

    let text = brief(&project, &[], None);
    assert!(!text.contains("## Open investigations"), "{text}");

    let envelope: serde_json::Value =
        serde_json::from_str(brief(&project, &["--json"], None).trim()).expect("one JSON line");
    assert_eq!(
        envelope["data"]["investigations"],
        serde_json::json!([]),
        "{envelope}"
    );
}

#[test]
fn open_investigations_are_cut_before_the_proposals_and_the_cut_is_said() {
    let project = Project::new();
    for number in 1..=60 {
        open_investigation(&project, &format!("Investigation {number}"));
    }
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

    // 60 investigations and 1 proposal make 64 lines with the project line and the two headings.
    // Five investigations are cut, the oldest, and the left-out line takes the place of one of them.
    let lines: Vec<&str> = text.lines().collect();
    assert_eq!(lines.len(), 60, "{text}");
    assert_eq!(
        lines.last(),
        Some(&"5 entries left out: dossier list"),
        "{text}"
    );
    assert!(lines.contains(&"DEC-001 · Blade mesh size (ben)"), "{text}");
    assert!(
        lines.contains(&"INV-060 · Investigation 60 — no attempt yet"),
        "{text}"
    );
    assert!(
        lines.contains(&"INV-006 · Investigation 6 — no attempt yet"),
        "{text}"
    );
    assert!(!text.contains("INV-005 ·"), "{text}");
}

/// Writes a digest of `milestone` as the file the `digest` command writes (format.md §32): the
/// command is not built yet, so the test writes the file itself. `status` is `approved` or
/// `draft`, `tasks` the comma-separated tasks of the milestone, and `summary` the lines of the
/// Summary section. An approved digest is in force from NOW.
fn write_digest(
    project: &Project,
    id: &str,
    status: &str,
    milestone: &str,
    tasks: &str,
    summary: &str,
) {
    let approval = if status == "approved" {
        format!("- approved: {NOW} by anna\n- valid-from: {NOW}\n")
    } else {
        String::new()
    };
    let text = format!(
        "<!-- dossier-format: 1 -->\n\
         # {id} · Summary of {milestone}\n\
         \n\
         - kind: digest\n\
         - status: {status}\n\
         - created: {NOW}\n\
         - author: anna-claude\n\
         {approval}\
         - milestone: {milestone}\n\
         - tasks: {tasks}\n\
         - history: {NOW} anna-claude: created\n\
         \n\
         ## Summary\n\
         \n\
         {summary}\n"
    );
    let name = format!("{id}-summary-of-{}.md", milestone.to_lowercase());
    project.write(&format!("DOSSIER/digests/{name}"), &text);
}

/// Writes an approved decision of `task` numbered `number`, as the file `decide` writes. Many of
/// them are written this way, since one `decide` run each would be slow.
fn write_decision(project: &Project, number: usize, task: &str) {
    let id = format!("DEC-{number:03}");
    let text = format!(
        "<!-- dossier-format: 1 -->\n\
         # {id} · Decision {number}\n\
         \n\
         - kind: decision\n\
         - status: approved\n\
         - created: {NOW}\n\
         - author: anna\n\
         - approved: {NOW} by anna\n\
         - valid-from: {NOW}\n\
         - task: {task}\n\
         - history: {NOW} anna: created\n\
         - history: {NOW} anna: approved\n\
         \n\
         ## Decision\n\
         \n\
         Decision number {number} of the invented project.\n\
         \n\
         ## Reason\n\
         \n\
         Reason number {number} of the invented project.\n"
    );
    project.write(
        &format!("DOSSIER/decisions/{id}-decision-{number}.md"),
        &text,
    );
}

#[test]
fn a_digest_prints_its_summary_in_place_of_the_decisions_and_findings_of_its_tasks() {
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
                "--task",
                "T-010",
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
                "--task",
                "T-010",
            ],
        )
        .success();
    project
        .run(
            "anna",
            &[
                "found",
                "The transient run held its margin.",
                "--title",
                "Transient run holds",
                "--task",
                "T-011",
            ],
        )
        .success();
    project
        .run(
            "anna",
            &[
                "found",
                "The surge margin was measured at the rig.",
                "--title",
                "Surge margin of the rig",
                "--task",
                "T-012",
            ],
        )
        .success();
    write_digest(
        &project,
        "DIG-001",
        "approved",
        "T-010",
        "T-010, T-011",
        "Decided: run the first speedline plan.\nFound: the steady run stalls at low flow.",
    );

    let text = brief(&project, &[], None);

    let lines: Vec<&str> = text.lines().collect();
    assert!(lines.contains(&"## Digests"), "{text}");
    assert!(
        lines.contains(&"DIG-001 · T-010 · Summary of T-010"),
        "{text}"
    );
    assert!(
        lines.contains(&"  Decided: run the first speedline plan."),
        "{text}"
    );
    assert!(
        lines.contains(&"  Found: the steady run stalls at low flow."),
        "{text}"
    );
    // The decision and the two findings of T-010 and T-011 are summed up; the finding of T-012 is
    // not in the digest and stays.
    assert!(
        lines.contains(&"3 entries of T-010 are summed up in DIG-001: dossier show DIG-001"),
        "{text}"
    );
    assert!(!text.contains("DEC-001"), "{text}");
    assert!(!text.contains("FND-001"), "{text}");
    assert!(!text.contains("FND-002"), "{text}");
    assert!(!lines.contains(&"## Decisions in force"), "{text}");
    assert!(
        lines.contains(&"FND-003 · Surge margin of the rig"),
        "{text}"
    );
    let digests_at = text.find("## Digests").expect("the digests heading");
    let findings_at = text.find("## Findings").expect("the findings heading");
    assert!(digests_at < findings_at, "{text}");
}

#[test]
fn a_decision_that_stands_and_an_open_assumption_of_a_summed_task_are_still_printed() {
    let project = Project::new();
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
                "T-010",
                "--standing",
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
                "--task",
                "T-010",
            ],
        )
        .success();
    write_digest(
        &project,
        "DIG-001",
        "approved",
        "T-010",
        "T-010",
        "Decided: the transient points run.",
    );

    let text = brief(&project, &[], None);

    let lines: Vec<&str> = text.lines().collect();
    assert!(
        lines.contains(&"DEC-001 · Transient speedline points (anna, 2026-11-20)"),
        "{text}"
    );
    assert!(
        lines.contains(&"  Run the last speedline points transient."),
        "{text}"
    );
    assert!(lines.contains(&"## Open assumptions"), "{text}");
    assert!(lines.contains(&"ASM-001 · Uniform inlet profile"), "{text}");
    // Nothing is summed up: the digest's heading and summary still print.
    assert!(!text.contains("summed up"), "{text}");
}

#[test]
fn a_draft_digest_changes_nothing() {
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
                "--task",
                "T-010",
            ],
        )
        .success();
    write_digest(
        &project,
        "DIG-001",
        "draft",
        "T-010",
        "T-010",
        "Decided: run the first plan.",
    );

    let text = brief(&project, &[], None);

    let lines: Vec<&str> = text.lines().collect();
    assert!(
        lines.contains(&"DEC-001 · First plan (anna, 2026-11-20)"),
        "{text}"
    );
    assert!(lines.contains(&"  It was the first plan."), "{text}");
    assert!(!text.contains("## Digests"), "{text}");
    assert!(!text.contains("summed up"), "{text}");
    assert!(
        lines.contains(&"1 draft is waiting: dossier drafts"),
        "{text}"
    );
}

#[test]
fn with_500_decisions_of_which_480_are_summed_up_the_brief_keeps_its_limit() {
    let project = Project::new();
    for number in 1..=500 {
        let task = if number <= 480 { "T-010" } else { "T-020" };
        write_decision(&project, number, task);
    }
    write_digest(
        &project,
        "DIG-001",
        "approved",
        "T-010",
        "T-010",
        "Decided: 480 invented decisions.\nFound: nothing new.",
    );

    let text = brief(&project, &[], None);

    let lines: Vec<&str> = text.lines().collect();
    assert!(lines.len() <= 60, "{} lines:\n{text}", lines.len());
    assert!(
        lines.contains(&"480 entries of T-010 are summed up in DIG-001: dossier show DIG-001"),
        "{text}"
    );
    assert!(
        lines.contains(&"DEC-500 · Decision 500 (anna, 2026-11-20)"),
        "{text}"
    );
    assert!(!text.contains("DEC-001 ·"), "{text}");
}

#[test]
fn a_digest_line_is_never_cut_to_fit_the_limit() {
    let project = Project::new();
    for number in 1..=70 {
        write_decision(&project, number, "T-020");
    }
    write_digest(
        &project,
        "DIG-001",
        "approved",
        "T-010",
        "T-010",
        "Decided: the first plan.\nFound: the steady run stalls.",
    );

    let text = brief(&project, &[], None);

    // The digest takes 4 lines and the project line, the decisions heading and the left-out line
    // take 3 more: 53 of the 70 decisions fit with the rest in 60 lines.
    let lines: Vec<&str> = text.lines().collect();
    assert_eq!(lines.len(), 60, "{text}");
    assert_eq!(
        lines.last(),
        Some(&"17 entries left out: dossier list"),
        "{text}"
    );
    assert!(lines.contains(&"## Digests"), "{text}");
    assert!(
        lines.contains(&"DIG-001 · T-010 · Summary of T-010"),
        "{text}"
    );
    assert!(lines.contains(&"  Found: the steady run stalls."), "{text}");
}

#[test]
fn two_digests_of_two_milestones_each_get_a_line_naming_its_milestone_and_digest() {
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
                "--task",
                "T-010",
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
                "--task",
                "T-011",
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
                "It was the second plan.",
                "--title",
                "Second plan",
                "--task",
                "T-020",
            ],
        )
        .success();
    write_digest(
        &project,
        "DIG-001",
        "approved",
        "T-010",
        "T-010, T-011",
        "Decided: run the first plan.",
    );
    write_digest(
        &project,
        "DIG-002",
        "approved",
        "T-020",
        "T-020",
        "Decided: run the second plan.",
    );
    // A digest whose tasks have no entries sums up nothing, so it gets no line.
    write_digest(
        &project,
        "DIG-003",
        "approved",
        "T-030",
        "T-030",
        "Decided: nothing yet.",
    );

    let text = brief(&project, &[], None);

    let lines: Vec<&str> = text.lines().collect();
    assert!(
        lines.contains(&"2 entries of T-010 are summed up in DIG-001: dossier show DIG-001"),
        "{text}"
    );
    assert!(
        lines.contains(&"1 entry of T-020 is summed up in DIG-002: dossier show DIG-002"),
        "{text}"
    );
    assert!(!text.contains("in DIG-003"), "{text}");
    assert!(!text.contains("dossier list --task"), "{text}");
    for left_out in ["DEC-001", "FND-001", "DEC-002"] {
        assert!(!text.contains(left_out), "{left_out} is summed up: {text}");
    }
}

#[test]
fn the_json_data_names_the_digests_and_the_entries_they_sum_up() {
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
                "--task",
                "T-010",
            ],
        )
        .success();
    write_digest(
        &project,
        "DIG-001",
        "approved",
        "T-010",
        "T-010",
        "Decided: run the first plan.",
    );

    let text = brief(&project, &[], None);
    assert!(
        text.contains("1 entry of T-010 is summed up in DIG-001: dossier show DIG-001"),
        "{text}"
    );

    let text = brief(&project, &["--json"], None);
    let envelope: serde_json::Value = serde_json::from_str(text.trim()).expect("one JSON line");
    assert_eq!(
        envelope["data"]["digests"],
        serde_json::json!(["DIG-001"]),
        "{text}"
    );
    assert_eq!(envelope["data"]["summed_up"], 1, "{text}");
    assert_eq!(
        envelope["data"]["decisions"],
        serde_json::json!([]),
        "{text}"
    );
}

/// Decides `count` invented decisions as anna, numbered from 1, each with a title, a decision and
/// a reason.
fn decide_numbered(project: &Project, count: usize) {
    for number in 1..=count {
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
}

#[test]
fn budget_limits_the_project_brief_for_one_call_and_says_what_is_left_out() {
    let project = Project::new();
    decide_numbered(&project, 30);

    let text = brief(&project, &["--budget", "10"], None);

    let lines: Vec<&str> = text.lines().collect();
    assert_eq!(lines.len(), 10, "{text}");
    assert_eq!(lines.first(), Some(&"Project P-001"), "{text}");
    assert_eq!(lines.get(1), Some(&"## Decisions in force"), "{text}");
    assert_eq!(
        lines.last(),
        Some(&"23 entries left out: dossier list"),
        "{text}"
    );
    // 1 project line, 1 heading, 7 decisions (the newest) and the left-out line make 10.
    assert!(
        lines.contains(&"DEC-030 · Decision 30 (anna, 2026-11-20)"),
        "{text}"
    );
    assert!(
        lines.contains(&"DEC-024 · Decision 24 (anna, 2026-11-20)"),
        "{text}"
    );
    assert!(!text.contains("DEC-023 ·"), "{text}");
}

#[test]
fn budget_of_200_prints_every_decision_in_full_where_the_default_would_shorten_them() {
    let project = Project::new();
    decide_numbered(&project, 30);

    let text = brief(&project, &["--budget", "200"], None);

    let lines: Vec<&str> = text.lines().collect();
    // 1 project line, 1 heading, and for each decision its title line and its two body lines.
    assert_eq!(lines.len(), 2 + 30 * 3, "{text}");
    assert!(
        lines.contains(&"  Decision number 1 of the invented project."),
        "{text}"
    );
    assert!(
        lines.contains(&"  Reason number 1 of the invented project."),
        "{text}"
    );
    assert!(!text.contains("left out"), "{text}");

    // The default limit of 60 lines shortens the oldest decisions to their title lines.
    let default = brief(&project, &[], None);
    assert!(
        !default.contains("  Reason number 1 of the invented project."),
        "{default}"
    );
    assert!(
        default.contains("  Reason number 30 of the invented project."),
        "{default}"
    );
}

#[test]
fn budget_limits_brief_task_for_one_call_and_its_first_line_stays() {
    let project = Project::new();
    for number in 1..=12 {
        let decision = format!("Decision number {number} for the invented task.");
        let reason = format!("Reason number {number} for the invented task.");
        let title = format!("Task decision {number}");
        project
            .run(
                "anna",
                &[
                    "decide",
                    decision.as_str(),
                    "--reason",
                    reason.as_str(),
                    "--task",
                    "T-001",
                    "--title",
                    title.as_str(),
                ],
            )
            .success();
    }
    let data = serde_json::json!({"id": "T-001", "title": "Speedline check", "status": "open"});
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

    let default = brief(&project, &["--task", "T-001"], None);
    assert!(default.lines().count() > 8, "{default}");

    let text = brief(&project, &["--task", "T-001", "--budget", "8"], None);

    let lines: Vec<&str> = text.lines().collect();
    assert_eq!(lines.len(), 8, "{text}");
    assert_eq!(
        lines.first(),
        Some(&"Task T-001 · Speedline check"),
        "{text}"
    );
    assert!(
        lines
            .last()
            .is_some_and(|line| line.ends_with("more left out")),
        "{text}"
    );
}

#[test]
fn a_budget_below_five_lines_is_a_usage_error_and_five_is_allowed() {
    let project = Project::new();

    project
        .command("anna", NOW)
        .args(["brief", "--budget", "4"])
        .assert()
        .failure()
        .code(2)
        .stderr(contains("--budget needs at least 5 lines, not 4"));
    project
        .command("anna", NOW)
        .args(["brief", "--budget", "2", "--json"])
        .assert()
        .failure()
        .code(2)
        .stdout(contains("\"code\":\"USAGE\""));

    let text = brief(&project, &["--budget", "5"], None);
    assert!(text.lines().count() <= 5, "{text}");
}
