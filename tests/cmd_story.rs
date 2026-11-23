//! Runs the built program for `story`: the approved decisions oldest first by the time they were
//! created, each with its reason and task, a superseded decision marked with the one that replaced
//! it, a proposal and the other kinds left out, `--milestone` through a todo stand-in, the JSON
//! data, and the refusals without todo. Each test runs in its own temporary project folder with
//! the clock set per decision (Project::run_at); the decisions and tasks are invented.

mod common;

use common::Project;
use predicates::str::contains;
use serde_json::{Value, json};

/// The answer of todo to `list --milestone T-900 --status all` (invented): two tasks.
const LIST: &str = r#"{"schema":"todo.list.v1","ok":true,"data":{"tasks":[{"id":"T-901","title":"Invented first task","status":"done"},{"id":"T-902","title":"Invented second task","status":"doing"}]},"warnings":[]}"#;

/// The stand-in for todo. It appends the arguments of every call to calls.txt in the project
/// folder, then prints `list` as its answer. The JSON has no character the batch file reads as
/// syntax.
fn todo_stand_in(list: &str) -> (String, String) {
    let windows = [
        "@echo off".to_string(),
        "echo %* >> calls.txt".to_string(),
        format!("echo {list}"),
        "exit /b 0".to_string(),
    ]
    .join("\r\n")
        + "\r\n";
    let unix = [
        "#!/bin/sh".to_string(),
        "echo \"$@\" >> calls.txt".to_string(),
        format!("printf '%s\\n' '{list}'"),
    ]
    .join("\n")
        + "\n";
    (windows, unix)
}

/// A project with the stand-in of todo answering with `list`.
fn project_with_todo(list: &str) -> Project {
    let project = Project::new();
    let (windows, unix) = todo_stand_in(list);
    project.write_stand_in("todo", &windows, &unix);
    project
}

/// Records a decision as `agent` at `now` with a title. `extra` holds the options, such as
/// `--task T-901` or `--replaces DEC-001`.
fn decide_at(
    project: &Project,
    agent: &str,
    now: &str,
    decision: &str,
    reason: &str,
    title: &str,
    extra: &[&str],
) {
    let mut args = vec!["decide", decision, "--reason", reason, "--title", title];
    args.extend_from_slice(extra);
    project.run_at(agent, now, &args).success();
}

/// The output the story prints for `lines`: each line, then a line break at the end.
fn text(lines: &[&str]) -> String {
    format!("{}\n", lines.join("\n"))
}

/// Three decisions made at three times, out of ID order (invented): DEC-001 on 18 November
/// (task T-901), DEC-002 on 2 November (no task), DEC-003 on 10 November (task T-777, a reason of
/// two lines). An assumption and a finding of the same time are not decisions and are left out.
fn project_with_three_decisions() -> Project {
    let project = Project::new();
    decide_at(
        &project,
        "anna",
        "2026-11-18T09:30:00+01:00",
        "Keep the speedline runs at three points",
        "Steady runs converged where stall shows.",
        "Three speedline points",
        &["--task", "T-901"],
    );
    decide_at(
        &project,
        "anna",
        "2026-11-02T10:00:00+01:00",
        "Start the speedline from the steady runs",
        "Steady runs are cheap for the first survey.",
        "Steady first",
        &[],
    );
    decide_at(
        &project,
        "anna",
        "2026-11-10T14:15:00+01:00",
        "Use the finer mesh in the first stage",
        "Invented reason: the first stage needs the finer mesh.\nThe second line says why.",
        "Finer first stage",
        &["--task", "T-777"],
    );
    project
        .run_at(
            "anna",
            "2026-11-05T11:00:00+01:00",
            &[
                "assume",
                "The wall function is valid at this Reynolds number",
                "--reason",
                "Invented reason: the wall law was not checked.",
                "--title",
                "Wall function valid",
            ],
        )
        .success();
    project
        .run_at(
            "anna",
            "2026-11-06T11:00:00+01:00",
            &[
                "found",
                "The mass flow still oscillates with half the time step",
                "--title",
                "Mass flow oscillates",
            ],
        )
        .success();
    project
}

#[test]
fn decisions_come_out_oldest_first_with_their_reasons_and_tasks() {
    let project = project_with_three_decisions();
    let output = project.stdout("", &["story"]);
    assert_eq!(
        output,
        text(&[
            "2026-11-02  DEC-002 · Steady first",
            "  Steady runs are cheap for the first survey.",
            "",
            "2026-11-10  DEC-003 · Finer first stage",
            "  Invented reason: the first stage needs the finer mesh.",
            "  The second line says why.",
            "  task: T-777",
            "",
            "2026-11-18  DEC-001 · Three speedline points",
            "  Steady runs converged where stall shows.",
            "  task: T-901",
        ])
    );
}

#[test]
fn a_superseded_decision_stays_and_is_marked_with_the_one_that_replaced_it() {
    let project = Project::new();
    decide_at(
        &project,
        "anna",
        "2026-11-02T10:00:00+01:00",
        "Use the steady runs first",
        "Steady runs converged early.",
        "Steady first",
        &[],
    );
    decide_at(
        &project,
        "anna",
        "2026-11-19T16:00:00+01:00",
        "Use the transient runs first",
        "Transient runs show the stall.",
        "Transient first",
        &["--replaces", "DEC-001"],
    );
    let output = project.stdout("", &["story"]);
    assert_eq!(
        output,
        text(&[
            "2026-11-02  DEC-001 · Steady first [superseded by DEC-002]",
            "  Steady runs converged early.",
            "",
            "2026-11-19  DEC-002 · Transient first",
            "  Transient runs show the stall.",
        ])
    );
}

#[test]
fn a_proposal_is_not_listed_until_it_is_a_decision() {
    let project = Project::new();
    decide_at(
        &project,
        "anna",
        "2026-11-02T10:00:00+01:00",
        "Use the steady runs first",
        "Steady runs converged early.",
        "Steady first",
        &[],
    );
    project
        .run_at(
            "ben",
            "2026-11-12T10:00:00+01:00",
            &[
                "propose",
                "Try the coarse mesh first",
                "--reason",
                "Invented reason: a coarse mesh is quicker.",
                "--title",
                "Coarse mesh first",
            ],
        )
        .success();
    let output = project.stdout("", &["story"]);
    assert_eq!(
        output,
        text(&[
            "2026-11-02  DEC-001 · Steady first",
            "  Steady runs converged early.",
        ]),
        "the proposal DEC-002 is not a decision yet"
    );
}

#[test]
fn json_gives_each_decision_its_fields_and_null_when_it_has_none() {
    let project = Project::new();
    decide_at(
        &project,
        "anna",
        "2026-11-02T10:00:00+01:00",
        "Use the steady runs first",
        "Steady runs converged early.",
        "Steady first",
        &[],
    );
    decide_at(
        &project,
        "anna",
        "2026-11-19T16:00:00+01:00",
        "Use the transient runs first",
        "Transient runs show the stall.",
        "Transient first",
        &["--replaces", "DEC-001", "--task", "T-901"],
    );
    let stdout = project.stdout("", &["story", "--json"]);
    let envelope: Value = serde_json::from_str(stdout.trim()).expect("one JSON line");
    assert_eq!(envelope["schema"], "dossier.story.v1");
    assert_eq!(envelope["ok"], true);
    assert_eq!(
        envelope["data"]["decisions"],
        json!([
            {
                "id": "DEC-001",
                "title": "Steady first",
                "date": "2026-11-02",
                "reason": "Steady runs converged early.",
                "task": null,
                "superseded_by": "DEC-002"
            },
            {
                "id": "DEC-002",
                "title": "Transient first",
                "date": "2026-11-19",
                "reason": "Transient runs show the stall.",
                "task": "T-901",
                "superseded_by": null
            }
        ])
    );
}

#[test]
fn without_decisions_the_story_says_so() {
    let project = Project::new();
    assert_eq!(project.stdout("", &["story"]), "no decisions yet\n");
    let stdout = project.stdout("", &["story", "--json"]);
    let envelope: Value = serde_json::from_str(stdout.trim()).expect("one JSON line");
    assert_eq!(envelope["data"]["decisions"], json!([]));
}

#[test]
fn milestone_keeps_only_the_decisions_of_its_tasks_and_the_milestone_itself() {
    let project = project_with_todo(LIST);
    decide_at(
        &project,
        "anna",
        "2026-11-02T10:00:00+01:00",
        "Start with the steady runs",
        "Steady runs are cheap.",
        "Steady first",
        &["--task", "T-901"],
    );
    decide_at(
        &project,
        "anna",
        "2026-11-03T10:00:00+01:00",
        "Use the other mesh",
        "Invented reason of another task.",
        "Other mesh",
        &["--task", "T-777"],
    );
    decide_at(
        &project,
        "anna",
        "2026-11-04T10:00:00+01:00",
        "Use the finer mesh in the second task",
        "Finer mesh for the second task.",
        "Finer second",
        &["--task", "T-902"],
    );
    decide_at(
        &project,
        "anna",
        "2026-11-05T10:00:00+01:00",
        "Plan the whole milestone at once",
        "The milestone binds all its tasks.",
        "Milestone plan",
        &["--task", "T-900"],
    );
    decide_at(
        &project,
        "anna",
        "2026-11-06T10:00:00+01:00",
        "Keep the default settings",
        "Invented reason with no task.",
        "No task",
        &[],
    );
    let output = project.stdout("", &["story", "--milestone", "T-900"]);
    assert_eq!(
        output,
        text(&[
            "2026-11-02  DEC-001 · Steady first",
            "  Steady runs are cheap.",
            "  task: T-901",
            "",
            "2026-11-04  DEC-003 · Finer second",
            "  Finer mesh for the second task.",
            "  task: T-902",
            "",
            "2026-11-05  DEC-004 · Milestone plan",
            "  The milestone binds all its tasks.",
            "  task: T-900",
        ])
    );
    let calls = project.read("calls.txt");
    assert!(
        calls.contains("list --milestone T-900 --status all --json"),
        "todo is asked for the tasks of the milestone: {calls}"
    );
}

#[test]
fn milestone_without_todo_is_todo_missing() {
    let project = Project::new();
    decide_at(
        &project,
        "anna",
        "2026-11-02T10:00:00+01:00",
        "Start with the steady runs",
        "Steady runs are cheap.",
        "Steady first",
        &["--task", "T-901"],
    );
    project
        .run("", &["story", "--milestone", "T-900"])
        .failure()
        .code(5)
        .stderr(contains("story --milestone needs todo"));
    // Without --milestone the story needs no todo at all.
    assert!(
        project
            .stdout("", &["story"])
            .contains("DEC-001 · Steady first")
    );
}
