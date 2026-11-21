//! Runs the built program for `status`: the project's tags and tasks from todo (or one line without
//! it), who works on what, what waits, what is next, the decisions in force, the newest findings,
//! what is unsettled, `--since` and `--milestone`.
//! Each test runs in its own temporary project folder with the clock fixed; the entries and the
//! names are invented.

mod common;

use common::Project;
use predicates::str::contains;
use serde_json::Value;

/// The summary of todo with one milestone (invented). Its counts are the whole project's.
const SUMMARY: &str = r#"{"schema":"todo.summary.v1","ok":true,"data":{"milestones":[{"id":"T-130","title":"Context work","priority":"P0","status":"open","progress":{"done":2,"total":5}}],"counts":{"open":3,"claimed":1,"blocked":1,"done":2,"dropped":0,"total":7,"questions":1,"ideas":0}},"warnings":[]}"#;

/// The task list of todo without a milestone (invented, as todo's `list --json` gives it: its
/// tasks have no milestone field): one claimed, one blocked, and seven ready tasks, the first of
/// them of another milestone.
const LIST: &str = r#"{"schema":"todo.list.v1","ok":true,"data":{"tasks":[{"id":"T-201","title":"Cut the speedline runs","status":"claimed","owner":"ben-codex","ready":false,"priority":"P0"},{"id":"T-202","title":"Get the mesh from the customer","status":"blocked","owner":null,"ready":false,"priority":"P0"},{"id":"T-209","title":"Tidy the old notes","status":"open","owner":null,"ready":true,"priority":"P2"},{"id":"T-203","title":"Write the status tests","status":"open","owner":null,"ready":true,"priority":"P1"},{"id":"T-204","title":"Write the milestone tests","status":"open","owner":null,"ready":true,"priority":"P1"},{"id":"T-205","title":"Write the cut tests","status":"open","owner":null,"ready":true,"priority":"P1"},{"id":"T-206","title":"Write the JSON tests","status":"open","owner":null,"ready":true,"priority":"P1"},{"id":"T-207","title":"Review the status code","status":"open","owner":null,"ready":true,"priority":"P2"},{"id":"T-208","title":"Update the status help","status":"open","owner":null,"ready":true,"priority":"P2"}]},"warnings":[]}"#;

/// The task list of todo for `list --milestone T-130` (invented): the same tasks as LIST below
/// that milestone, without T-209, which belongs to another one.
const LIST_T130: &str = r#"{"schema":"todo.list.v1","ok":true,"data":{"tasks":[{"id":"T-201","title":"Cut the speedline runs","status":"claimed","owner":"ben-codex","ready":false,"priority":"P0"},{"id":"T-202","title":"Get the mesh from the customer","status":"blocked","owner":null,"ready":false,"priority":"P0"},{"id":"T-203","title":"Write the status tests","status":"open","owner":null,"ready":true,"priority":"P1"},{"id":"T-204","title":"Write the milestone tests","status":"open","owner":null,"ready":true,"priority":"P1"},{"id":"T-205","title":"Write the cut tests","status":"open","owner":null,"ready":true,"priority":"P1"},{"id":"T-206","title":"Write the JSON tests","status":"open","owner":null,"ready":true,"priority":"P1"},{"id":"T-207","title":"Review the status code","status":"open","owner":null,"ready":true,"priority":"P2"},{"id":"T-208","title":"Update the status help","status":"open","owner":null,"ready":true,"priority":"P2"}]},"warnings":[]}"#;

/// The open questions of todo (invented): one question.
const QUESTIONS: &str = r#"{"schema":"todo.list.v1","ok":true,"data":{"tasks":[{"id":"Q-001","title":"Is the wall function valid here?","status":"open","owner":null,"ready":false,"priority":"P1"}]},"warnings":[]}"#;

/// The answer of todo when the task list cannot be read (invented).
const REFUSED: &str = r#"{"schema":"todo.list.v1","ok":false,"error":{"code":"REFUSED","message":"no TODO.md in this folder"},"warnings":[]}"#;

/// The stand-in for todo. It answers `--version` with exit 0, and appends the arguments of every
/// other call to calls.txt in the project folder. `list --kind question` gives QUESTIONS,
/// `list --milestone <id>` gives LIST_T130, `list` gives `list` (and exits with `list_exit`), and
/// any other call gives SUMMARY.
fn todo_stand_in(list: &str, list_exit: u8) -> (String, String) {
    let windows = [
        "@echo off".to_string(),
        "if \"%~1\"==\"--version\" exit /b 0".to_string(),
        "echo %* >> calls.txt".to_string(),
        "if \"%~1\"==\"list\" if \"%~2\"==\"--kind\" goto question".to_string(),
        "if \"%~1\"==\"list\" if \"%~2\"==\"--milestone\" goto milestone".to_string(),
        "if \"%~1\"==\"list\" goto list".to_string(),
        format!("echo {SUMMARY}"),
        "exit /b 0".to_string(),
        ":question".to_string(),
        format!("echo {QUESTIONS}"),
        "exit /b 0".to_string(),
        ":milestone".to_string(),
        format!("echo {LIST_T130}"),
        "exit /b 0".to_string(),
        ":list".to_string(),
        format!("echo {list}"),
        format!("exit /b {list_exit}"),
    ]
    .join("\r\n")
        + "\r\n";
    let unix = [
        "#!/bin/sh".to_string(),
        "if [ \"$1\" = \"--version\" ]; then exit 0; fi".to_string(),
        "echo \"$@\" >> calls.txt".to_string(),
        "if [ \"$1\" = \"list\" ] && [ \"$2\" = \"--kind\" ]; then".to_string(),
        format!("  printf '%s\\n' '{QUESTIONS}'; exit 0"),
        "fi".to_string(),
        "if [ \"$1\" = \"list\" ] && [ \"$2\" = \"--milestone\" ]; then".to_string(),
        format!("  printf '%s\\n' '{LIST_T130}'; exit 0"),
        "fi".to_string(),
        "if [ \"$1\" = \"list\" ]; then".to_string(),
        format!("  printf '%s\\n' '{list}'; exit {list_exit}"),
        "fi".to_string(),
        format!("printf '%s\\n' '{SUMMARY}'"),
    ]
    .join("\n")
        + "\n";
    (windows, unix)
}

/// Writes the stand-in of `todo_stand_in` into the project.
fn with_todo(project: &Project, list: &str, list_exit: u8) {
    let (windows, unix) = todo_stand_in(list, list_exit);
    project.write_stand_in("todo", &windows, &unix);
}

/// The arguments of each call the stand-in got, one line per call, from calls.txt. The batch
/// file's `echo` leaves a space at the end of each line, so the ends are trimmed.
fn todo_calls(project: &Project) -> Vec<String> {
    project
        .read("calls.txt")
        .lines()
        .map(|line| line.trim_end().to_string())
        .collect()
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
         1 draft waiting\n"
    );
}

#[test]
fn with_a_stand_in_todo_the_screen_shows_the_tasks_who_works_on_what_what_waits_and_what_is_next() {
    let project = project_with_entries();
    with_todo(&project, LIST, 0);
    let stdout = project.stdout("anna", &["status"]);
    assert_eq!(
        stdout,
        "Project P-001\n\
         ## Tasks\n\
         T-130 · Context work · 2/5 done\n\
         open 3, claimed 1, blocked 1, done 2\n\
         ## Working now\n\
         T-201 · Cut the speedline runs (ben-codex)\n\
         ## Waiting\n\
         T-202 · Get the mesh from the customer (blocked)\n\
         Q-001 · Is the wall function valid here? (question)\n\
         ## Decisions in force\n\
         DEC-001 · Use transient runs for the last three speedline points (anna, 2026-11-20)\n\
         ## Newest findings\n\
         none\n\
         ## Unsettled\n\
         1 draft waiting\n\
         ## Next\n\
         T-209 · Tidy the old notes\n\
         T-203 · Write the status tests\n\
         T-204 · Write the milestone tests\n\
         T-205 · Write the cut tests\n\
         T-206 · Write the JSON tests\n"
    );
}

#[test]
fn the_tags_line_names_the_kinds_that_have_values_and_leaves_out_the_rest() {
    let project = Project::new();
    let settings = project
        .read("DOSSIER/dossier.toml")
        .replace("machine = []", "machine = [\"test rig\"]")
        .replace("software = []", "software = [\"Solver Z\", \"Mesher Y\"]");
    project.write("DOSSIER/dossier.toml", &settings);
    let stdout = project.stdout("anna", &["status"]);
    assert_eq!(
        stdout,
        "Project P-001\n\
         tags: machine test rig; software Solver Z, Mesher Y\n\
         tasks: todo is not installed\n\
         ## Decisions in force\n\
         none\n\
         ## Newest findings\n\
         none\n\
         ## Unsettled\n\
         nothing waiting\n"
    );
}

#[test]
fn a_failed_task_list_call_leaves_its_parts_out_and_the_questions_still_show() {
    let project = Project::new();
    with_todo(&project, REFUSED, 1);
    let stdout = project.stdout("anna", &["status"]);
    assert_eq!(
        stdout,
        "Project P-001\n\
         ## Tasks\n\
         T-130 · Context work · 2/5 done\n\
         open 3, claimed 1, blocked 1, done 2\n\
         ## Waiting\n\
         Q-001 · Is the wall function valid here? (question)\n\
         ## Decisions in force\n\
         none\n\
         ## Newest findings\n\
         none\n\
         ## Unsettled\n\
         nothing waiting\n"
    );
}

#[test]
fn milestone_limits_the_task_parts_to_the_tasks_of_that_milestone() {
    let project = Project::new();
    with_todo(&project, LIST, 0);
    let stdout = project.stdout("anna", &["status", "--milestone", "T-130"]);
    assert_eq!(
        stdout,
        "Project P-001\n\
         ## Tasks\n\
         T-130 · Context work · 2/5 done\n\
         ## Working now\n\
         T-201 · Cut the speedline runs (ben-codex)\n\
         ## Waiting\n\
         T-202 · Get the mesh from the customer (blocked)\n\
         Q-001 · Is the wall function valid here? (question)\n\
         ## Decisions in force\n\
         none\n\
         ## Newest findings\n\
         none\n\
         ## Unsettled\n\
         nothing waiting\n\
         ## Next\n\
         T-203 · Write the status tests\n\
         T-204 · Write the milestone tests\n\
         T-205 · Write the cut tests\n\
         T-206 · Write the JSON tests\n\
         T-207 · Review the status code\n"
    );
}

#[test]
fn the_task_parts_are_cut_to_the_lines_of_the_settings() {
    let project = Project::new();
    with_todo(&project, LIST, 0);
    let settings = project
        .read("DOSSIER/dossier.toml")
        .replace("status_lines = 60", "status_lines = 18");
    project.write("DOSSIER/dossier.toml", &settings);
    let stdout = project.stdout("anna", &["status"]);
    assert_eq!(
        stdout,
        "Project P-001\n\
         ## Tasks\n\
         T-130 · Context work · 2/5 done\n\
         open 3, claimed 1, blocked 1, done 2\n\
         ## Working now\n\
         T-201 · Cut the speedline runs (ben-codex)\n\
         ## Waiting\n\
         T-202 · Get the mesh from the customer (blocked)\n\
         1 more: todo list\n\
         ## Decisions in force\n\
         none\n\
         ## Newest findings\n\
         none\n\
         ## Unsettled\n\
         nothing waiting\n\
         ## Next\n\
         T-209 · Tidy the old notes\n\
         4 more: todo list\n"
    );
    assert_eq!(stdout.lines().count(), 18);
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
    assert_eq!(value["data"]["working"], Value::Null);
    assert_eq!(value["data"]["waiting"], Value::Null);
    assert_eq!(value["data"]["next"], Value::Null);
    assert_eq!(value["data"]["decisions"], serde_json::json!(["DEC-001"]));
    assert_eq!(value["data"]["findings"], serde_json::json!([]));
    assert_eq!(
        value["data"]["unsettled"],
        serde_json::json!({"drafts": 1, "open_assumptions": 0, "changed_lessons": 0})
    );
}

#[test]
fn json_gives_working_waiting_and_next_as_task_ids() {
    let project = project_with_entries();
    with_todo(&project, LIST, 0);
    let stdout = project.stdout("anna", &["status", "--json"]);
    let value: Value = serde_json::from_str(stdout.trim()).expect("one JSON object");
    assert_eq!(value["data"]["tasks"]["counts"]["claimed"], 1);
    assert_eq!(value["data"]["working"], serde_json::json!(["T-201"]));
    assert_eq!(
        value["data"]["waiting"],
        serde_json::json!(["T-202", "Q-001"])
    );
    assert_eq!(
        value["data"]["next"],
        serde_json::json!(["T-209", "T-203", "T-204", "T-205", "T-206"])
    );
}

#[test]
fn without_a_milestone_todo_is_asked_for_the_whole_list_and_the_questions() {
    let project = Project::new();
    with_todo(&project, LIST, 0);
    project.stdout("anna", &["status"]);
    let calls = todo_calls(&project);
    assert!(calls.iter().any(|call| call == "list --json"), "{calls:?}");
    assert!(
        calls
            .iter()
            .any(|call| call == "list --kind question --json"),
        "{calls:?}"
    );
}

#[test]
fn with_a_milestone_todo_is_asked_for_its_list_and_its_questions() {
    let project = Project::new();
    with_todo(&project, LIST, 0);
    project.stdout("anna", &["status", "--milestone", "t-130"]);
    let calls = todo_calls(&project);
    assert!(
        calls
            .iter()
            .any(|call| call == "list --milestone T-130 --json"),
        "{calls:?}"
    );
    assert!(
        calls
            .iter()
            .any(|call| call == "list --kind question --milestone T-130 --json"),
        "{calls:?}"
    );
}

/// Runs `status` as anna with SSH_CONNECTION set to `value`, as an ssh session sets it, and
/// returns the standard output.
fn status_over_ssh(project: &Project, value: &str, args: &[&str]) -> String {
    let stdout = project
        .command("anna", "2026-11-20T10:00:00+01:00")
        .env("SSH_CONNECTION", value)
        .args(args)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    String::from_utf8(stdout).expect("the standard output is UTF-8")
}

#[test]
fn over_ssh_the_first_line_names_the_project_folder_on_the_server() {
    let project = Project::new();
    let root = project.root().display().to_string();
    let expected = format!("Project P-001 on the server in {root}");
    let session = "192.0.2.10 50722 192.0.2.20 22";

    let screen = status_over_ssh(&project, session, &["status"]);
    assert_eq!(screen.lines().next(), Some(expected.as_str()), "{screen}");

    let since = status_over_ssh(&project, session, &["status", "--since", "2026-11-01"]);
    assert_eq!(since.lines().next(), Some(expected.as_str()), "{since}");

    // An empty SSH_CONNECTION is not a server session.
    let local = status_over_ssh(&project, "", &["status"]);
    assert_eq!(local.lines().next(), Some("Project P-001"), "{local}");
}
