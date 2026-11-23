//! Runs the built program for `solved` and `gave-up`: a solved investigation keeps its attempt
//! lines byte for byte and names the attempts that led to the solution; an unknown attempt is
//! refused with nothing written; a closed investigation is closed again by neither command;
//! `gave-up` sets its reason; `tried` opens a solved investigation again; an LLM may run both; a
//! decision and a team dossier are refused. Each test runs in its own temporary project with the
//! clock fixed at 2026-11-20T10:00:00+01:00.

mod common;

use common::Project;
use dossier::entry::{Entry, Kind, Status};
use predicates::str::contains;
use std::fs;

/// The clock of every run in these tests: the clock of `common::Project`.
const NOW: &str = "2026-11-20T10:00:00+01:00";

/// The problem of the investigation that the tests open with `problem`.
const PROBLEM: &str = "The density residual grows from iteration 40 on.";

/// The text of the entry file with `id`.
fn entry_text(project: &Project, id: &str) -> String {
    fs::read_to_string(project.entry_file(id)).expect("the entry file")
}

/// The entry with `id`, parsed.
fn entry_of(project: &Project, id: &str) -> Entry {
    Entry::parse(&entry_text(project, id)).expect("the entry parses")
}

/// The Attempts section of an entry text, from its heading up to the next section.
fn attempts_block(text: &str) -> &str {
    let after = text.split("## Attempts").nth(1).unwrap_or_default();
    after.split("## ").next().unwrap_or_default()
}

/// Runs the program as `agent` with the clock `now`, asserts success, and returns its standard
/// output.
fn stdout_at(project: &Project, agent: &str, now: &str, args: &[&str]) -> String {
    let output = project
        .run_at(agent, now, args)
        .success()
        .get_output()
        .stdout
        .clone();
    String::from_utf8(output).expect("the standard output is UTF-8")
}

/// Opens INV-001 as a person, with the problem `PROBLEM`.
fn open_investigation(project: &Project) {
    let stdout = project.stdout("anna", &["problem", PROBLEM]);
    assert_eq!(stdout, "INV-001: added (open)\n");
}

/// Records a failed attempt on INV-001 as anna at `now`, with the text `text`.
fn tried_failed(project: &Project, now: &str, text: &str) {
    stdout_at(
        project,
        "anna",
        now,
        &["tried", "INV-001", text, "--failed"],
    );
}

#[test]
fn solved_after_two_attempts_sets_the_fields_and_the_solution_and_keeps_the_attempt_lines() {
    let project = Project::new();
    open_investigation(&project);
    tried_failed(
        &project,
        "2026-11-20T10:05:00+01:00",
        "The CFL number is too high",
    );
    stdout_at(
        &project,
        "anna",
        "2026-11-20T10:10:00+01:00",
        &[
            "tried",
            "INV-001",
            "The flux at the outlet has the wrong sign",
            "--worked",
        ],
    );
    let before = entry_text(&project, "INV-001");

    let stdout = stdout_at(
        &project,
        "anna",
        "2026-11-20T10:20:00+01:00",
        &[
            "solved",
            "INV-001",
            "The outlet flux had the wrong sign",
            "--attempt",
            "1",
            "--attempt",
            "2",
        ],
    );
    assert_eq!(stdout, "INV-001: solved\n");

    let after = entry_text(&project, "INV-001");
    // The block gains one blank line before the Solution heading; the attempt lines do not change.
    assert_eq!(
        attempts_block(&after).trim_end(),
        attempts_block(&before).trim_end(),
        "the attempt lines are kept byte for byte"
    );
    let entry = entry_of(&project, "INV-001");
    assert_eq!(entry.kind(), Some(Kind::Investigation));
    assert_eq!(entry.status(), Some(Status::Approved));
    assert_eq!(entry.field("state").as_deref(), Some("solved"));
    assert_eq!(
        entry.field("closed").as_deref(),
        Some("2026-11-20T10:20:00+01:00 by anna")
    );
    assert_eq!(entry.field("given-up-reason"), None);
    assert_eq!(
        entry.section("Solution").as_deref(),
        Some("The outlet flux had the wrong sign\n\nAttempts: 1, 2")
    );
    assert_eq!(
        entry.history().last().map(String::as_str),
        Some("2026-11-20T10:20:00+01:00 anna: solved")
    );
}

#[test]
fn a_solution_without_attempt_numbers_has_no_attempts_line() {
    let project = Project::new();
    open_investigation(&project);

    project.stdout(
        "anna",
        &["solved", "INV-001", "A restart from the converged solution"],
    );

    let entry = entry_of(&project, "INV-001");
    assert_eq!(
        entry.section("Solution").as_deref(),
        Some("A restart from the converged solution")
    );
    assert_eq!(entry.field("state").as_deref(), Some("solved"));
}

#[test]
fn an_unknown_attempt_is_refused_and_nothing_is_written() {
    let project = Project::new();
    open_investigation(&project);
    tried_failed(
        &project,
        "2026-11-20T10:05:00+01:00",
        "The CFL number is too high",
    );
    let before = entry_text(&project, "INV-001");

    project
        .run(
            "anna",
            &["solved", "INV-001", "A solution", "--attempt", "9"],
        )
        .code(1)
        .stderr(contains("no attempt 9 in INV-001"));
    assert_eq!(entry_text(&project, "INV-001"), before);
}

#[test]
fn an_investigation_that_is_closed_is_not_closed_again() {
    let project = Project::new();
    open_investigation(&project);
    project.stdout(
        "anna",
        &["solved", "INV-001", "The outlet flux had the wrong sign"],
    );
    let solved = entry_text(&project, "INV-001");

    project
        .run("anna", &["solved", "INV-001", "Something else"])
        .code(1)
        .stderr(contains(
            "only an open investigation can be closed; INV-001 is solved",
        ));
    project
        .run("anna", &["gave-up", "INV-001", "--reason", "out of time"])
        .code(1)
        .stderr(contains(
            "only an open investigation can be closed; INV-001 is solved",
        ));
    assert_eq!(entry_text(&project, "INV-001"), solved);
}

#[test]
fn gave_up_sets_its_fields_and_a_closed_one_is_not_closed_again() {
    let project = Project::new();
    open_investigation(&project);

    let stdout = stdout_at(
        &project,
        "anna",
        "2026-11-20T10:30:00+01:00",
        &["gave-up", "INV-001", "--reason", "out of time"],
    );
    assert_eq!(stdout, "INV-001: given up\n");

    let entry = entry_of(&project, "INV-001");
    assert_eq!(entry.status(), Some(Status::Approved));
    assert_eq!(entry.field("state").as_deref(), Some("given up"));
    assert_eq!(
        entry.field("closed").as_deref(),
        Some("2026-11-20T10:30:00+01:00 by anna")
    );
    assert_eq!(
        entry.field("given-up-reason").as_deref(),
        Some("out of time")
    );
    assert_eq!(
        entry.history().last().map(String::as_str),
        Some("2026-11-20T10:30:00+01:00 anna: given up — out of time")
    );

    project
        .run("anna", &["solved", "INV-001", "A solution"])
        .code(1)
        .stderr(contains(
            "only an open investigation can be closed; INV-001 is given up",
        ));
}

#[test]
fn tried_after_solved_opens_it_again_and_says_so() {
    let project = Project::new();
    open_investigation(&project);
    tried_failed(
        &project,
        "2026-11-20T10:05:00+01:00",
        "The CFL number is too high",
    );
    stdout_at(
        &project,
        "anna",
        "2026-11-20T10:10:00+01:00",
        &[
            "solved",
            "INV-001",
            "The outlet flux had the wrong sign",
            "--attempt",
            "1",
        ],
    );

    let stdout = stdout_at(
        &project,
        "anna",
        "2026-11-20T10:15:00+01:00",
        &[
            "tried",
            "INV-001",
            "Restarted from the converged solution",
            "--worked",
        ],
    );
    assert_eq!(
        stdout,
        "INV-001: attempt 2 recorded (worked), opened again\n"
    );

    let entry = entry_of(&project, "INV-001");
    assert_eq!(entry.field("state").as_deref(), Some("open"));
    assert_eq!(entry.field("closed"), None);
    assert_eq!(entry.field("given-up-reason"), None);
}

#[test]
fn an_llm_may_solve_one_investigation_and_give_up_another() {
    let project = Project::new();
    open_investigation(&project);
    let stdout = project.stdout("anna", &["problem", "The inlet pressure drifts."]);
    assert!(
        stdout.starts_with("INV-002: added (open)\n"),
        "the second investigation is INV-002: {stdout}"
    );

    let solved = stdout_at(
        &project,
        "anna-claude",
        "2026-11-20T10:20:00+01:00",
        &["solved", "INV-001", "The outlet flux had the wrong sign"],
    );
    assert_eq!(solved, "INV-001: solved\n");
    let given_up = stdout_at(
        &project,
        "anna-claude",
        "2026-11-20T10:25:00+01:00",
        &["gave-up", "INV-002", "--reason", "no time left"],
    );
    assert_eq!(given_up, "INV-002: given up\n");

    let first = entry_of(&project, "INV-001");
    assert_eq!(first.field("state").as_deref(), Some("solved"));
    assert_eq!(
        first.history().last().map(String::as_str),
        Some("2026-11-20T10:20:00+01:00 anna via anna-claude: solved")
    );
    let second = entry_of(&project, "INV-002");
    assert_eq!(second.field("state").as_deref(), Some("given up"));
    assert_eq!(
        second.history().last().map(String::as_str),
        Some("2026-11-20T10:25:00+01:00 anna via anna-claude: given up — no time left")
    );
}

#[test]
fn a_decision_is_refused_by_both_commands_and_nothing_is_written() {
    let project = Project::new();
    project.stdout(
        "anna",
        &[
            "decide",
            "Use the transient runs for the last three points.",
            "--reason",
            "Steady runs converged where the transient run already showed rotating stall.",
        ],
    );
    let before = entry_text(&project, "DEC-001");

    project
        .run("anna", &["solved", "DEC-001", "A solution"])
        .code(1)
        .stderr(contains(
            "solved closes investigations only; DEC-001 is of kind decision",
        ));
    project
        .run("anna", &["gave-up", "DEC-001", "--reason", "no"])
        .code(1)
        .stderr(contains(
            "gave-up closes investigations only; DEC-001 is of kind decision",
        ));
    assert_eq!(entry_text(&project, "DEC-001"), before);
}

#[test]
fn a_blank_multi_line_or_heading_text_is_refused_and_nothing_is_written() {
    let project = Project::new();
    open_investigation(&project);
    let before = entry_text(&project, "INV-001");

    project
        .run("anna", &["solved", "INV-001", "   "])
        .code(1)
        .stderr(contains("the solution must not be empty"));
    project
        .run("anna", &["solved", "INV-001", "Two\nlines"])
        .code(1)
        .stderr(contains("the solution must be one line"));
    project
        .run("anna", &["solved", "INV-001", "## Not a heading"])
        .code(1)
        .stderr(contains(
            "the solution must not start with the heading mark ##",
        ));
    project
        .run("anna", &["gave-up", "INV-001", "--reason", "   "])
        .code(1)
        .stderr(contains("--reason must not be empty"));
    project
        .run("anna", &["gave-up", "INV-001", "--reason", "## Not either"])
        .code(1)
        .stderr(contains("--reason must not start with the heading mark ##"));
    assert_eq!(entry_text(&project, "INV-001"), before);
}

#[test]
fn a_team_dossier_refuses_both_commands() {
    let team = Project {
        dir: tempfile::TempDir::new().expect("a temporary folder"),
    };
    team.command("anna", NOW)
        .args(["init", "--code", "P-001", "--team-dossier"])
        .assert()
        .success();

    team.run("anna", &["solved", "INV-001", "A solution"])
        .code(1)
        .stderr(contains("investigations belong to a project"));
    team.run("anna", &["gave-up", "INV-001", "--reason", "no"])
        .code(1)
        .stderr(contains("investigations belong to a project"));
}
