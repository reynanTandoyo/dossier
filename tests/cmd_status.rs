//! Runs the built program for `status`: the project's tasks from todo (or one line without it),
//! the decisions in force, the newest findings, what is unsettled, `--since` and `--milestone`.
//! Each test runs in its own temporary project folder with the clock fixed; the entries and the
//! names are invented.

mod common;

use common::Project;
use predicates::str::contains;
use serde_json::Value;

/// The summary of todo with one milestone (invented). Its counts are the whole project's.
const SUMMARY: &str = r#"{"schema":"todo.summary.v1","ok":true,"data":{"milestones":[{"id":"T-130","title":"Context work","priority":"P0","status":"open","progress":{"done":2,"total":5}}],"counts":{"open":3,"claimed":1,"blocked":0,"done":2,"dropped":0,"total":6,"questions":0,"ideas":0}},"warnings":[]}"#;

/// The stand-in for todo: it answers `--version` with exit 0 and prints the summary for any
/// other call.
fn summary_stand_in() -> (String, String) {
    let windows = [
        "@echo off".to_string(),
        "if \"%~1\"==\"--version\" exit /b 0".to_string(),
        format!("echo {SUMMARY}"),
    ]
    .join("\r\n")
        + "\r\n";
    let unix = [
        "#!/bin/sh".to_string(),
        "if [ \"$1\" = \"--version\" ]; then exit 0; fi".to_string(),
        format!("printf '%s\\n' '{SUMMARY}'"),
    ]
    .join("\n")
        + "\n";
    (windows, unix)
}

/// Runs `status` (or any command) as `agent` at `now` and returns the standard output.
fn out_at(project: &Project, agent: &str, now: &str, args: &[&str]) -> String {
    let stdout = project
        .run_at(agent, now, args)
        .success()
        .get_output()
        .stdout
        .clone();
    String::from_utf8(stdout).expect("the standard output is UTF-8")
}

/// A project with a decision by anna (task T-042) and a draft note by ben, both recorded at the
/// fixed clock of the project (2026-11-20T10:00).
fn project_with_entries() -> Project {
    let project = Project::new();
    project
        .run(
            "anna",
            &[
                "decide",
                "Use transient runs for the last three speedline points",
                "--reason",
                "Steady runs converged where rotating stall already shows.",
                "--task",
                "T-042",
            ],
        )
        .success();
    project
        .run(
            "ben",
            &[
                "found",
                "The mass flow still oscillates with half the time step",
                "--later",
            ],
        )
        .success();
    project
}

#[test]
fn without_todo_the_screen_has_the_todo_line_a_decision_and_the_drafts_count() {
    let project = project_with_entries();
    let stdout = project.stdout("anna", &["status"]);
    assert_eq!(
        stdout,
        "Project P-001\n\
         tasks: todo is not installed\n\
         ## Decisions in force\n\
         DEC-001 · Use transient runs for the last three speedline points (anna, 2026-11-20)\n\
         ## Newest findings\n\
         none\n\
         ## Unsettled\n\
         1 drafts waiting\n"
    );
}

#[test]
fn with_a_stand_in_todo_the_tasks_part_shows_the_milestone_and_the_counts() {
    let project = project_with_entries();
    let (windows, unix) = summary_stand_in();
    project.write_stand_in("todo", &windows, &unix);
    let stdout = project.stdout("anna", &["status"]);
    assert_eq!(
        stdout,
        "Project P-001\n\
         ## Tasks\n\
         T-130 · Context work · 2/5 done\n\
         open 3, claimed 1, blocked 0, done 2\n\
         ## Decisions in force\n\
         DEC-001 · Use transient runs for the last three speedline points (anna, 2026-11-20)\n\
         ## Newest findings\n\
         none\n\
         ## Unsettled\n\
         1 drafts waiting\n"
    );
}

#[test]
fn since_last_shows_the_events_the_first_time_and_none_the_second() {
    let project = project_with_entries();
    let first = out_at(
        &project,
        "anna",
        "2026-11-21T09:00:00+01:00",
        &["status", "--since", "last"],
    );
    assert!(first.starts_with("Project P-001\n## Since the start (no earlier run)\n"));
    assert!(first.contains(
        "2026-11-20T10:00:00+01:00  anna  created  DEC-001  Use transient runs for the last three speedline points\n"
    ));
    assert!(first.contains(
        "2026-11-20T10:00:00+01:00  ben  created  FND-001  The mass flow still oscillates with half the time step\n"
    ));
    assert_eq!(
        project.read("DOSSIER/.state/seen-anna.txt"),
        "2026-11-21T09:00:00+01:00\n"
    );

    let second = out_at(
        &project,
        "anna",
        "2026-11-21T10:00:00+01:00",
        &["status", "--since", "last"],
    );
    assert_eq!(
        second,
        "Project P-001\n## Since 2026-11-21T09:00:00+01:00\nnone\n"
    );
    assert_eq!(
        project.read("DOSSIER/.state/seen-anna.txt"),
        "2026-11-21T10:00:00+01:00\n"
    );
}

#[test]
fn an_llm_since_last_shows_the_news_but_does_not_move_the_persons_marker() {
    let project = Project::new();
    // Anna runs first: her marker is written at 09:00.
    out_at(
        &project,
        "anna",
        "2026-11-21T09:00:00+01:00",
        &["status", "--since", "last"],
    );
    // Ben's LLM records a finding after that.
    project
        .run_at(
            "ben-codex",
            "2026-11-21T10:30:00+01:00",
            &["found", "Halving the time step changed nothing"],
        )
        .success();

    let first = out_at(
        &project,
        "anna-claude",
        "2026-11-21T11:00:00+01:00",
        &["status", "--since", "last"],
    );
    assert!(first.contains(
        "2026-11-21T10:30:00+01:00  ben via ben-codex  created  FND-001  Halving the time step changed nothing\n"
    ));
    assert_eq!(
        project.read("DOSSIER/.state/seen-anna.txt"),
        "2026-11-21T09:00:00+01:00\n"
    );

    // The LLM's run did not move the marker, so the same news is still new for it.
    let again = out_at(
        &project,
        "anna-claude",
        "2026-11-21T11:30:00+01:00",
        &["status", "--since", "last"],
    );
    assert!(again.contains("FND-001  Halving the time step changed nothing\n"));
}

#[test]
fn since_a_date_shows_only_the_records_on_or_after_it() {
    let project = project_with_entries();
    project
        .run_at(
            "ben",
            "2026-11-21T10:00:00+01:00",
            &["found", "Second observation"],
        )
        .success();
    let stdout = project.stdout("anna", &["status", "--since", "2026-11-21"]);
    assert_eq!(
        stdout,
        "Project P-001\n## Since 2026-11-21\n2026-11-21T10:00:00+01:00  ben  created  FND-002  Second observation\n"
    );
}

#[test]
fn since_last_without_a_name_is_a_usage_error_and_a_bad_date_is_refused() {
    let project = project_with_entries();
    project
        .run("", &["status", "--since", "last"])
        .code(2)
        .stderr(contains("needs a name"));
    project
        .run("anna", &["status", "--since", "yesterday"])
        .code(1)
        .stderr(contains("2026-11-20"));
}

#[test]
fn milestone_keeps_only_the_entries_of_that_task() {
    let project = Project::new();
    project
        .run(
            "anna",
            &[
                "decide",
                "Keep the rig",
                "--reason",
                "It is calibrated.",
                "--task",
                "T-042",
            ],
        )
        .success();
    project
        .run(
            "anna",
            &[
                "decide",
                "Use another rule",
                "--reason",
                "It is simpler.",
                "--task",
                "T-043",
            ],
        )
        .success();
    let stdout = project.stdout("anna", &["status", "--milestone", "t-042"]);
    assert!(
        stdout.contains("DEC-001 · Keep the rig (anna, 2026-11-20)\n"),
        "{stdout}"
    );
    assert!(!stdout.contains("DEC-002"), "{stdout}");
    assert!(
        !stdout.contains("open "),
        "the counts are for the whole project: {stdout}"
    );
}

#[test]
fn the_screen_is_cut_to_the_lines_of_the_settings_and_says_what_is_left_out() {
    let project = Project::new();
    for number in 1..=3 {
        let title = format!("Decision number {number}");
        project
            .run("anna", &["decide", title.as_str(), "--reason", "A reason."])
            .success();
    }
    let settings = project
        .read("DOSSIER/dossier.toml")
        .replace("status_lines = 60", "status_lines = 9");
    project.write("DOSSIER/dossier.toml", &settings);

    let stdout = project.stdout("anna", &["status"]);
    assert_eq!(
        stdout,
        "Project P-001\n\
         tasks: todo is not installed\n\
         ## Decisions in force\n\
         DEC-003 · Decision number 3 (anna, 2026-11-20)\n\
         2 more: dossier list\n\
         ## Newest findings\n\
         none\n\
         ## Unsettled\n\
         nothing waiting\n"
    );
    assert_eq!(stdout.lines().count(), 9);
}

#[test]
fn json_returns_the_same_parts_as_data() {
    let project = project_with_entries();
    let stdout = project.stdout("anna", &["status", "--json"]);
    let value: Value = serde_json::from_str(stdout.trim()).expect("one JSON object");
    assert_eq!(value["schema"], "dossier.status.v1");
    assert_eq!(value["ok"], true);
    assert_eq!(value["data"]["code"], "P-001");
    assert_eq!(value["data"]["tasks"], Value::Null);
    assert_eq!(value["data"]["decisions"], serde_json::json!(["DEC-001"]));
    assert_eq!(value["data"]["findings"], serde_json::json!([]));
    assert_eq!(
        value["data"]["unsettled"],
        serde_json::json!({"drafts": 1, "open_assumptions": 0, "changed_lessons": 0})
    );
}
