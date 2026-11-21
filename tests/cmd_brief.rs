//! Runs the built program for `brief`: the approved decisions in full, the open assumptions,
//! findings and lessons by title, the drafts line, the marks of a lesson changed by hand and of
//! a dead end, the line limit with 70 decisions, the same output on every run, the server line
//! of remote mode, and `--task` through a stand-in for todo. Phase 2 adds the map first, the key
//! values, the proposals waiting, the `rests on` marks and the heading of what rests on a wrong
//! assumption, `--on <date>` and the order in which the line limit cuts. Each test runs in its own
//! temporary project folder (common::Project), with
//! the clock fixed at 2026-11-20T10:00:00+01:00.

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
