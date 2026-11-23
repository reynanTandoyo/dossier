//! Runs the built program for `tried`: attempts are numbered 1, 2, 3 in the order they are recorded
//! and change nothing else in the file; an attempt with every part has them all in the line of
//! format.md §28; an LLM's attempt ends with `via`; exactly one verdict is required; a finding, a
//! draft investigation, a missing usage record, a blank or multi-line text and a team dossier are
//! refused with nothing written; a given up investigation is opened again; `search` finds the text
//! of an attempt. Each test runs in its own temporary project with the clock fixed at
//! 2026-11-20T10:00:00+01:00.

mod common;

use common::Project;
use dossier::attempts;
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

/// The text with the Attempts section and the history lines of `tried` left out, and the blank
/// line before the section too: what `tried` must leave exactly as it was.
fn without_attempts(text: &str) -> String {
    let head = text.split("## Attempts").next().unwrap_or_default();
    head.split_inclusive('\n')
        .filter(|line| !line.contains(": tried — "))
        .collect::<String>()
        .trim_end()
        .to_string()
}

#[test]
fn three_attempts_are_numbered_one_two_three_and_change_nothing_else() {
    let project = Project::new();
    open_investigation(&project);
    let before = entry_text(&project, "INV-001");

    let first = stdout_at(
        &project,
        "anna",
        "2026-11-20T10:05:00+01:00",
        &["tried", "INV-001", "The CFL number is too high", "--failed"],
    );
    assert_eq!(first, "INV-001: attempt 1 recorded (failed)\n");
    let second = stdout_at(
        &project,
        "anna",
        "2026-11-20T10:10:00+01:00",
        &[
            "tried",
            "INV-001",
            "The mesh is too coarse at the leading edge",
            "--failed",
        ],
    );
    assert_eq!(second, "INV-001: attempt 2 recorded (failed)\n");
    let third = stdout_at(
        &project,
        "anna",
        "2026-11-20T10:15:00+01:00",
        &[
            "tried",
            "INV-001",
            "The flux at the outlet has the wrong sign",
            "--worked",
        ],
    );
    assert_eq!(third, "INV-001: attempt 3 recorded (worked)\n");

    let after = entry_text(&project, "INV-001");
    assert_eq!(
        without_attempts(&after),
        before.trim_end(),
        "only the history lines and the Attempts section were added"
    );
    let entry = Entry::parse(&after).expect("the entry parses");
    assert_eq!(entry.kind(), Some(Kind::Investigation));
    assert_eq!(entry.status(), Some(Status::Approved));
    let numbers: Vec<u32> = attempts::of(&entry)
        .iter()
        .map(|attempt| attempt.n)
        .collect();
    assert_eq!(numbers, [1, 2, 3]);
    let lines = [
        "1. [failed] The CFL number is too high (2026-11-20T10:05:00+01:00 anna)",
        "2. [failed] The mesh is too coarse at the leading edge (2026-11-20T10:10:00+01:00 anna)",
        "3. [worked] The flux at the outlet has the wrong sign (2026-11-20T10:15:00+01:00 anna)",
    ]
    .join("\n");
    assert_eq!(entry.section("Attempts").as_deref(), Some(lines.as_str()));
    assert_eq!(entry.field("state").as_deref(), Some("open"));
    let history = entry.history();
    assert_eq!(
        history[2..],
        [
            "2026-11-20T10:05:00+01:00 anna: tried — 1 failed",
            "2026-11-20T10:10:00+01:00 anna: tried — 2 failed",
            "2026-11-20T10:15:00+01:00 anna: tried — 3 worked",
        ]
    );
}

#[test]
fn an_attempt_with_every_part_has_them_all_in_the_line_of_format_section_28() {
    let project = Project::new();
    open_investigation(&project);
    project.stdout(
        "anna",
        &[
            "used",
            "ExampleSolver",
            "--version",
            "2406",
            "--task",
            "T-012",
        ],
    );

    let stdout = project.stdout(
        "anna",
        &[
            "tried",
            "INV-001",
            "The CFL number is too high",
            "--failed",
            "--did",
            "CFL 5 to 1",
            "--result",
            "diverges at iteration 180",
            "--run",
            "use-001",
            "--task",
            "t-012",
        ],
    );
    assert_eq!(stdout, "INV-001: attempt 1 recorded (failed)\n");

    let entry = entry_of(&project, "INV-001");
    assert_eq!(
        entry.section("Attempts").as_deref(),
        Some(
            "1. [failed] The CFL number is too high — did: CFL 5 to 1 — result: diverges at iteration 180 — run: USE-001 — task: T-012 (2026-11-20T10:00:00+01:00 anna)"
        )
    );
    let attempt = &attempts::of(&entry)[0];
    assert_eq!(attempt.did.as_deref(), Some("CFL 5 to 1"));
    assert_eq!(attempt.result.as_deref(), Some("diverges at iteration 180"));
    assert_eq!(attempt.run.as_deref(), Some("USE-001"));
    assert_eq!(attempt.task.as_deref(), Some("T-012"));
}

#[test]
fn no_verdict_or_two_verdicts_is_a_usage_error_and_writes_nothing() {
    let project = Project::new();
    open_investigation(&project);
    let before = entry_text(&project, "INV-001");

    project
        .run("anna", &["tried", "INV-001", "No verdict given"])
        .code(2)
        .stderr(contains("give exactly one verdict"));
    project
        .run(
            "anna",
            &["tried", "INV-001", "Two verdicts", "--failed", "--worked"],
        )
        .code(2)
        .stderr(contains("give exactly one verdict"));
    assert_eq!(entry_text(&project, "INV-001"), before);
}

#[test]
fn a_run_that_names_a_finding_or_a_missing_entry_is_refused_and_nothing_is_written() {
    let project = Project::new();
    open_investigation(&project);
    project.stdout("anna", &["found", "The mass flow still oscillates."]);
    let before = entry_text(&project, "INV-001");

    project
        .run(
            "anna",
            &[
                "tried",
                "INV-001",
                "Halved the time step",
                "--failed",
                "--run",
                "FND-001",
            ],
        )
        .code(1)
        .stderr(contains(
            "--run must name a usage record; FND-001 is of kind finding",
        ));
    project
        .run(
            "anna",
            &[
                "tried",
                "INV-001",
                "Halved the time step",
                "--failed",
                "--run",
                "USE-099",
            ],
        )
        .code(1)
        .stderr(contains("no entry USE-099"));
    project
        .run("anna", &["tried", "INV-099", "A hypothesis", "--failed"])
        .code(1)
        .stderr(contains("no entry INV-099"));
    assert_eq!(entry_text(&project, "INV-001"), before);
}

#[test]
fn an_llm_attempt_ends_with_via_and_its_history_names_the_llm() {
    let project = Project::new();
    open_investigation(&project);

    let stdout = stdout_at(
        &project,
        "anna-claude",
        NOW,
        &[
            "tried",
            "INV-001",
            "Restarted from the converged solution",
            "--worked",
        ],
    );
    assert_eq!(stdout, "INV-001: attempt 1 recorded (worked)\n");

    let entry = entry_of(&project, "INV-001");
    assert_eq!(
        entry.section("Attempts").as_deref(),
        Some(
            "1. [worked] Restarted from the converged solution (2026-11-20T10:00:00+01:00 anna via anna-claude)"
        )
    );
    assert_eq!(attempts::of(&entry)[0].who, "anna via anna-claude");
    assert_eq!(
        entry.history().last().map(String::as_str),
        Some("2026-11-20T10:00:00+01:00 anna via anna-claude: tried — 1 worked")
    );
}

#[test]
fn tried_on_a_finding_is_refused_and_the_finding_is_not_changed() {
    let project = Project::new();
    project.stdout("anna", &["found", "The mass flow still oscillates."]);
    let before = entry_text(&project, "FND-001");

    project
        .run(
            "anna",
            &["tried", "FND-001", "Halved the time step", "--failed"],
        )
        .code(1)
        .stderr(contains(
            "tried records attempts on investigations only; FND-001 is of kind finding",
        ));
    assert_eq!(entry_text(&project, "FND-001"), before);
}

#[test]
fn an_investigation_that_is_not_approved_takes_no_attempt() {
    let project = Project::new();
    open_investigation(&project);
    let draft =
        entry_text(&project, "INV-001").replace("- status: approved\n", "- status: draft\n");
    fs::write(project.entry_file("INV-001"), &draft).expect("the entry file");

    project
        .run("anna", &["tried", "INV-001", "A hypothesis", "--failed"])
        .code(1)
        .stderr(contains(
            "only an approved investigation takes an attempt; INV-001 is draft",
        ));
    assert_eq!(entry_text(&project, "INV-001"), draft);
}

#[test]
fn a_blank_or_multi_line_text_or_a_heading_mark_is_refused_and_nothing_is_written() {
    let project = Project::new();
    open_investigation(&project);
    let before = entry_text(&project, "INV-001");

    project
        .run("anna", &["tried", "INV-001", "   ", "--failed"])
        .code(1)
        .stderr(contains("the attempt must not be empty"));
    project
        .run("anna", &["tried", "INV-001", "Two\nlines", "--failed"])
        .code(1)
        .stderr(contains("the attempt must be one line"));
    project
        .run(
            "anna",
            &["tried", "INV-001", "## Not a heading", "--failed"],
        )
        .code(1)
        .stderr(contains(
            "the attempt must not start with the heading mark ##",
        ));
    project
        .run(
            "anna",
            &[
                "tried",
                "INV-001",
                "A hypothesis",
                "--failed",
                "--did",
                "## Not either",
            ],
        )
        .code(1)
        .stderr(contains("--did must not start with the heading mark ##"));
    assert_eq!(entry_text(&project, "INV-001"), before);
}

#[test]
fn an_attempt_on_a_given_up_investigation_opens_it_again_and_says_so() {
    let project = Project::new();
    open_investigation(&project);
    let given_up = entry_text(&project, "INV-001").replace(
        "- state: open\n",
        "- state: given up\n- closed: 2026-11-20T09:50:00+01:00 by anna\n- given-up-reason: out of time\n",
    );
    fs::write(project.entry_file("INV-001"), &given_up).expect("the entry file");

    let stdout = project.stdout(
        "anna",
        &[
            "tried",
            "INV-001",
            "Restarted from the converged solution",
            "--worked",
        ],
    );
    assert_eq!(
        stdout,
        "INV-001: attempt 1 recorded (worked), opened again\n"
    );

    let entry = entry_of(&project, "INV-001");
    assert_eq!(entry.field("state").as_deref(), Some("open"));
    assert_eq!(entry.field("closed"), None);
    assert_eq!(entry.field("given-up-reason"), None);
    let history = entry.history();
    assert_eq!(
        history[history.len() - 2..],
        [
            "2026-11-20T10:00:00+01:00 anna: reopened",
            "2026-11-20T10:00:00+01:00 anna: tried — 1 worked",
        ]
    );
}

#[test]
fn search_finds_the_text_of_an_attempt_and_names_the_attempt_in_the_result() {
    let project = Project::new();
    open_investigation(&project);
    project.stdout(
        "anna",
        &["tried", "INV-001", "The CFL number is too high", "--failed"],
    );
    project.stdout(
        "anna",
        &[
            "tried",
            "INV-001",
            "The flux at the outlet has the wrong sign",
            "--worked",
        ],
    );

    let stdout = project.stdout("anna", &["search", "outlet"]);
    let line = stdout
        .lines()
        .find(|line| line.starts_with("INV-001"))
        .expect("the investigation is found");
    assert!(
        line.contains("— 2. [worked] The flux at the outlet has the wrong sign"),
        "the result names the attempt: {line}"
    );
}

#[test]
fn a_team_dossier_refuses_an_attempt() {
    let team = Project {
        dir: tempfile::TempDir::new().expect("a temporary folder"),
    };
    team.command("anna", NOW)
        .args(["init", "--code", "P-001", "--team-dossier"])
        .assert()
        .success();

    team.run("anna", &["tried", "INV-001", "A hypothesis", "--failed"])
        .code(1)
        .stderr(contains("investigations belong to a project"));
    assert!(
        !team.root().join("DOSSIER").join("investigations").exists(),
        "a refused team write creates no folder"
    );
}
