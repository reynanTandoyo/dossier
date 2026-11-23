//! Runs the built program for `status`: the project's tags and tasks from todo (or one line without
//! it), who works on what, what waits, what is next, the decisions in force, the newest findings,
//! the open assumptions, what rests on a wrong assumption, what is unsettled (proposals, the
//! counts), the map, the key values, the last exchange, `--since`, `--milestone` and `--on`
//! (R-CTX-16, R-EXCH-5, R-LIFE-9, R-INV-5).
//! Each test runs in its own temporary project folder with the clock fixed; the entries and the
//! names are invented.

mod common;

use common::Project;
use dossier::blockfile::BlockFile;
use predicates::str::contains;
use serde_json::{Value, json};

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

/// The marker of `person` as its file holds it, with its line break; None when there is no file.
fn marker_of(project: &Project, person: &str) -> Option<String> {
    let path = project
        .root()
        .join("DOSSIER")
        .join(".state")
        .join(format!("seen-{person}.txt"));
    std::fs::read_to_string(path).ok()
}

#[test]
fn since_leaves_out_the_settings_and_the_other_records_that_are_not_entries() {
    let project = Project::new();
    // The first run of `--since last` has no earlier run: the settings records of init and of
    // `people add` are not news either.
    assert_eq!(
        out_at(
            &project,
            "anna",
            "2026-11-21T09:00:00+01:00",
            &["status", "--since", "last"]
        ),
        "Project P-001\n## Since the start (no earlier run)\nnone\n"
    );

    project
        .run(
            "anna",
            &["decide", "Use transient runs", "--reason", "Converged."],
        )
        .success();
    project
        .run("ben", &["found", "The mass flow still oscillates"])
        .success();
    // A value and the map are journal records too, with IDs that are not entries (format.md §11).
    project
        .run("anna", &["value", "set", "design.massflow", "12.4 kg/s"])
        .success();
    project.write("src/solver/mod.rs", "solver\n");
    project
        .run("anna", &["map", "set", "src/solver", "time integration"])
        .success();

    let stdout = project.stdout("anna", &["status", "--since", "2026-11-01"]);
    assert_eq!(
        stdout,
        "Project P-001\n\
         ## Since 2026-11-01\n\
         2026-11-20T10:00:00+01:00  ben  created  FND-001  The mass flow still oscillates\n\
         2026-11-20T10:00:00+01:00  anna  created  DEC-001  Use transient runs\n"
    );
    assert!(!stdout.contains("(removed)"), "{stdout}");
    let data = status_data(&project, &["--since", "2026-11-01"]);
    assert_eq!(
        data["events"].as_array().map(|events| events
            .iter()
            .map(|event| event["id"].clone())
            .collect::<Vec<_>>()),
        Some(vec![json!("FND-001"), json!("DEC-001")])
    );
}

#[test]
fn keep_with_since_last_leaves_the_marker_where_it_is() {
    let project = Project::new();
    out_at(
        &project,
        "anna",
        "2026-11-21T09:00:00+01:00",
        &["status", "--since", "last"],
    );
    assert_eq!(
        marker_of(&project, "anna").as_deref(),
        Some("2026-11-21T09:00:00+01:00\n")
    );
    project
        .run_at(
            "ben",
            "2026-11-21T09:30:00+01:00",
            &["found", "The mass flow still oscillates"],
        )
        .success();

    // With --keep the news since 09:00 is shown and the marker stays at 09:00.
    let kept = out_at(
        &project,
        "anna",
        "2026-11-21T10:00:00+01:00",
        &["status", "--since", "last", "--keep"],
    );
    assert_eq!(
        kept,
        "Project P-001\n## Since 2026-11-21T09:00:00+01:00\n\
         2026-11-21T09:30:00+01:00  ben  created  FND-001  The mass flow still oscillates\n"
    );
    assert_eq!(
        marker_of(&project, "anna").as_deref(),
        Some("2026-11-21T09:00:00+01:00\n")
    );

    // The next run without --keep shows the same news, and moves the marker to 11:00.
    let again = out_at(
        &project,
        "anna",
        "2026-11-21T11:00:00+01:00",
        &["status", "--since", "last"],
    );
    assert_eq!(again, kept);
    assert_eq!(
        marker_of(&project, "anna").as_deref(),
        Some("2026-11-21T11:00:00+01:00\n")
    );
}

#[test]
fn keep_needs_since() {
    let project = Project::new();
    project
        .run("anna", &["status", "--keep"])
        .code(2)
        .stderr(contains("--since"));
    assert_eq!(marker_of(&project, "anna"), None);
}

#[test]
fn since_a_date_never_moves_the_marker() {
    let project = Project::new();
    project.stdout("anna", &["status", "--since", "2026-11-01"]);
    assert_eq!(marker_of(&project, "anna"), None);

    out_at(
        &project,
        "anna",
        "2026-11-21T09:00:00+01:00",
        &["status", "--since", "last"],
    );
    out_at(
        &project,
        "anna",
        "2026-11-21T10:00:00+01:00",
        &["status", "--since", "2026-11-01"],
    );
    assert_eq!(
        marker_of(&project, "anna").as_deref(),
        Some("2026-11-21T09:00:00+01:00\n")
    );
}

#[test]
fn an_llm_named_with_a_dash_reads_its_persons_marker_and_never_moves_it() {
    let project = Project::new();
    out_at(
        &project,
        "anna",
        "2026-11-21T09:00:00+01:00",
        &["status", "--since", "last"],
    );
    project
        .run_at(
            "ben-codex",
            "2026-11-21T10:30:00+01:00",
            &["found", "Halving the time step changed nothing"],
        )
        .success();

    // The LLM reads anna's marker (09:00), not "the start", and leaves it where it is.
    let first = out_at(
        &project,
        "anna - claude",
        "2026-11-21T11:00:00+01:00",
        &["status", "--since", "last"],
    );
    assert_eq!(
        first,
        "Project P-001\n## Since 2026-11-21T09:00:00+01:00\n\
         2026-11-21T10:30:00+01:00  ben via ben-codex  created  FND-001  Halving the time step changed nothing\n"
    );
    assert_eq!(
        marker_of(&project, "anna").as_deref(),
        Some("2026-11-21T09:00:00+01:00\n")
    );
    let again = out_at(
        &project,
        "anna - claude",
        "2026-11-21T11:30:00+01:00",
        &["status", "--since", "last"],
    );
    assert_eq!(again, first);
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
    assert_eq!(value["data"]["investigations"], serde_json::json!([]));
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

/// The fixed clock of `common::Project`.
const NOW: &str = "2026-11-20T10:00:00+01:00";
/// The time of the change of design.massflow in `dated_project`.
const CHANGE: &str = "2026-12-01T09:00:00+01:00";
/// A time after every change of `dated_project`: the project as it is now.
const LATER: &str = "2026-12-02T09:00:00+01:00";

/// The data of `status --json` run as anna with the arguments given.
fn status_data(project: &Project, args: &[&str]) -> Value {
    let mut all = vec!["status", "--json"];
    all.extend_from_slice(args);
    let stdout = project.stdout("anna", &all);
    let value: Value = serde_json::from_str(stdout.trim()).expect("one JSON object");
    value["data"].clone()
}

#[test]
fn key_values_show_the_key_one_first_with_its_source_and_date_and_at_most_fifteen() {
    let project = Project::new();
    for number in 1..=15 {
        let name = format!("test.v{number:02}");
        project.stdout("anna", &["value", "set", name.as_str(), "1 m"]);
    }
    project.stdout(
        "anna",
        &[
            "value",
            "set",
            "design.massflow",
            "12.4 kg/s",
            "--key",
            "--source",
            "spec rev3",
        ],
    );
    let mut expected = String::from(
        "Project P-001\n\
         ## Key values\n\
         design.massflow  12.4 kg/s  (spec rev3, 2026-11-20)\n",
    );
    for number in 1..=14 {
        expected.push_str(&format!("test.v{number:02}  1 m  (2026-11-20)\n"));
    }
    expected.push_str(
        "tasks: todo is not installed\n\
         ## Decisions in force\n\
         none\n\
         ## Newest findings\n\
         none\n\
         ## Unsettled\n\
         nothing waiting\n",
    );
    assert_eq!(project.stdout("anna", &["status"]), expected);

    let data = status_data(&project, &[]);
    assert_eq!(data["values"].as_array().map(Vec::len), Some(15));
    assert_eq!(data["values"][0]["name"], "design.massflow");
    assert_eq!(data["values"][0]["key"], true);
    assert_eq!(data["values"][0]["unit"], "kg/s");
    assert_eq!(data["values"][1]["name"], "test.v01");
}

#[test]
fn a_proposal_waits_in_unsettled_and_is_listed_as_data() {
    let project = Project::new();
    project.stdout(
        "ben",
        &[
            "propose",
            "Use the steady runs",
            "--reason",
            "They converged.",
        ],
    );
    assert_eq!(
        project.stdout("anna", &["status"]),
        "Project P-001\n\
         tasks: todo is not installed\n\
         ## Decisions in force\n\
         none\n\
         ## Newest findings\n\
         none\n\
         ## Unsettled\n\
         1 proposal waiting\n"
    );
    assert_eq!(status_data(&project, &[])["proposals"], json!(["DEC-001"]));
}

/// A project where DEC-001 (task T-042) and the file data/model.sim of T-042 rest on ASM-001, which
/// anna refutes at the fixed clock: the assumption affects T-042 and has no task of its own.
fn project_resting_on_a_wrong_assumption() -> Project {
    let project = Project::new();
    project.write("data/model.sim", "model\n");
    project.stdout(
        "anna",
        &["attach", "T-042", "data/model.sim", "--role", "model"],
    );
    project.stdout(
        "anna",
        &[
            "assume",
            "The inlet profile is uniform",
            "--reason",
            "No measured profile was delivered.",
            "--affects",
            "T-042",
        ],
    );
    project.stdout(
        "anna",
        &[
            "decide",
            "Use transient runs for the last three speedline points",
            "--reason",
            "Steady runs converged where rotating stall already shows.",
            "--task",
            "T-042",
        ],
    );
    project
        .command("anna", "2026-11-20T10:00:00+01:00")
        .env("DOSSIER_ANSWERS", "")
        .args([
            "refute",
            "ASM-001",
            "--by",
            "anna, from the rig measurements",
        ])
        .assert()
        .success();
    project
}

#[test]
fn what_rests_on_a_wrong_assumption_is_counted_in_unsettled_and_listed_as_data() {
    let project = project_resting_on_a_wrong_assumption();
    let stdout = project.stdout("anna", &["status"]);
    assert!(
        stdout.ends_with("## Unsettled\n2 entries and files rest on a wrong assumption\n"),
        "{stdout}"
    );
    assert_eq!(
        status_data(&project, &[])["rests_on"],
        json!([
            {"kind": "entry", "id": "DEC-001", "rests_on": ["ASM-001"]},
            {"kind": "file", "task": "T-042", "path": "data/model.sim", "rests_on": ["ASM-001"]}
        ])
    );
}

#[test]
fn a_detached_file_that_rests_on_a_wrong_assumption_is_not_counted() {
    let project = project_resting_on_a_wrong_assumption();
    project.stdout("anna", &["detach", "T-042", "data/model.sim"]);
    let stdout = project.stdout("anna", &["status"]);
    assert!(
        stdout.ends_with("## Unsettled\n1 entry or file rests on a wrong assumption\n"),
        "{stdout}"
    );
    assert_eq!(
        status_data(&project, &[])["rests_on"],
        json!([{"kind": "entry", "id": "DEC-001", "rests_on": ["ASM-001"]}])
    );
}

#[test]
fn the_last_received_and_the_last_delivered_record_show_in_their_own_part() {
    let project = Project::new();
    project.write("spec.pdf", "spec\n");
    project.write("spec-rev2.pdf", "spec rev 2\n");
    project.stdout("anna", &["received", "spec.pdf", "--from", "the customer"]);
    project
        .run_at(
            "anna",
            "2026-11-22T09:00:00+01:00",
            &["received", "spec-rev2.pdf", "--from", "the customer"],
        )
        .success();
    project.stdout("anna", &["delivered", "spec.pdf", "--to", "the customer"]);

    assert_eq!(
        project.stdout("anna", &["status"]),
        "Project P-001\n\
         tasks: todo is not installed\n\
         ## Decisions in force\n\
         none\n\
         ## Newest findings\n\
         none\n\
         ## Unsettled\n\
         nothing waiting\n\
         ## Exchange\n\
         received 2026-11-22 RCV-002 from the customer\n\
         delivered 2026-11-20 DLV-001 to the customer\n"
    );
    let data = status_data(&project, &[]);
    assert_eq!(
        data["exchange"]["received"],
        json!({"id": "RCV-002", "date": "2026-11-22", "who": "the customer"})
    );
    assert_eq!(
        data["exchange"]["delivered"],
        json!({"id": "DLV-001", "date": "2026-11-20", "who": "the customer"})
    );
}

/// A project with the history that `--on` reads, all of it invented:
/// - design.massflow is 12.1 kg/s from 2026-11-20 (source spec rev2), and 12.4 kg/s from
///   2026-12-01 (source spec rev3);
/// - DEC-001 (task T-042) is approved on 2026-11-20;
/// - FND-001 (task T-042) is found on 2026-11-22;
/// - the customer sends spec.pdf on 2026-11-21: RCV-001.
fn dated_project() -> Project {
    let project = Project::new();
    project.write("spec.pdf", "spec\n");
    project.stdout(
        "anna",
        &[
            "value",
            "set",
            "design.massflow",
            "12.1 kg/s",
            "--key",
            "--source",
            "spec rev2",
        ],
    );
    project.stdout(
        "anna",
        &[
            "decide",
            "Use transient runs for the last three speedline points",
            "--reason",
            "Steady runs converged where rotating stall already shows.",
            "--task",
            "T-042",
        ],
    );
    project
        .run_at(
            "anna",
            "2026-11-21T09:00:00+01:00",
            &["received", "spec.pdf", "--from", "the customer"],
        )
        .success();
    project
        .run_at(
            "ben",
            "2026-11-22T09:00:00+01:00",
            &[
                "found",
                "Halving the time step did not remove the oscillation",
                "--task",
                "T-042",
            ],
        )
        .success();
    project
        .run_at(
            "anna",
            CHANGE,
            &[
                "value",
                "set",
                "design.massflow",
                "12.4 kg/s",
                "--key",
                "--source",
                "spec rev3",
            ],
        )
        .success();
    project
}

#[test]
fn on_shows_the_decisions_findings_values_and_exchange_as_they_were_on_that_day() {
    let project = dated_project();
    assert_eq!(
        project.stdout("anna", &["status", "--on", "2026-11-21"]),
        "Project P-001\n\
         as of 2026-11-21 for decisions, findings, values and exchange; the rest is now\n\
         ## Key values\n\
         design.massflow  12.1 kg/s  (2026-11-20)\n\
         tasks: todo is not installed\n\
         ## Decisions in force\n\
         DEC-001 · Use transient runs for the last three speedline points (anna, 2026-11-20)\n\
         ## Newest findings\n\
         none\n\
         ## Unsettled\n\
         nothing waiting\n\
         ## Exchange\n\
         received 2026-11-21 RCV-001 from the customer\n\
         nothing delivered\n"
    );
    assert_eq!(
        project.stdout("anna", &["status", "--on", "2026-11-23"]),
        "Project P-001\n\
         as of 2026-11-23 for decisions, findings, values and exchange; the rest is now\n\
         ## Key values\n\
         design.massflow  12.1 kg/s  (2026-11-20)\n\
         tasks: todo is not installed\n\
         ## Decisions in force\n\
         DEC-001 · Use transient runs for the last three speedline points (anna, 2026-11-20)\n\
         ## Newest findings\n\
         FND-001 · Halving the time step did not remove the oscillation (ben, 2026-11-22)\n\
         ## Unsettled\n\
         nothing waiting\n\
         ## Exchange\n\
         received 2026-11-21 RCV-001 from the customer\n\
         nothing delivered\n"
    );
}

#[test]
fn on_a_day_before_the_first_record_shows_nothing_in_force() {
    let project = dated_project();
    assert_eq!(
        project.stdout("anna", &["status", "--on", "2026-11-19"]),
        "Project P-001\n\
         as of 2026-11-19 for decisions, findings, values and exchange; the rest is now\n\
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
fn without_on_the_screen_is_the_project_as_it_is_now() {
    let project = dated_project();
    assert_eq!(
        out_at(&project, "anna", LATER, &["status"]),
        "Project P-001\n\
         ## Key values\n\
         design.massflow  12.4 kg/s  (spec rev3, 2026-12-01)\n\
         tasks: todo is not installed\n\
         ## Decisions in force\n\
         DEC-001 · Use transient runs for the last three speedline points (anna, 2026-11-20)\n\
         ## Newest findings\n\
         FND-001 · Halving the time step did not remove the oscillation (ben, 2026-11-22)\n\
         ## Unsettled\n\
         nothing waiting\n\
         ## Exchange\n\
         received 2026-11-21 RCV-001 from the customer\n\
         nothing delivered\n"
    );
}

#[test]
fn the_json_of_on_gives_the_day_and_the_parts_as_they_were() {
    let project = dated_project();
    let data = status_data(&project, &["--on", "2026-11-21"]);
    assert_eq!(data["on"], "2026-11-21");
    assert_eq!(data["values"][0]["name"], "design.massflow");
    assert_eq!(data["values"][0]["value"], "12.1");
    assert_eq!(data["values"][0]["unit"], "kg/s");
    assert_eq!(data["values"][0]["source"], Value::Null);
    assert_eq!(data["values"][0]["valid_from"], "2026-11-20T10:00:00+01:00");
    assert_eq!(data["decisions"], json!(["DEC-001"]));
    assert_eq!(data["findings"], json!([]));
    assert_eq!(
        data["exchange"]["received"],
        json!({"id": "RCV-001", "date": "2026-11-21", "who": "the customer"})
    );
    assert_eq!(data["exchange"]["delivered"], Value::Null);
    assert_eq!(status_data(&project, &[])["on"], Value::Null);
}

#[test]
fn proposals_and_marks_show_on_a_past_day_too_as_they_are_now() {
    let project = project_resting_on_a_wrong_assumption();
    project.stdout(
        "ben",
        &[
            "propose",
            "Use the steady runs",
            "--reason",
            "They converged.",
        ],
    );
    let stdout = project.stdout("anna", &["status", "--on", "2026-11-20"]);
    assert!(
        stdout.ends_with(
            "## Unsettled\n1 proposal waiting\n2 entries and files rest on a wrong assumption\n"
        ),
        "{stdout}"
    );
}

#[test]
fn on_takes_a_date_and_does_not_go_with_since() {
    let project = dated_project();
    project
        .run("anna", &["status", "--on", "2026-13-01"])
        .failure()
        .code(1)
        .stderr(contains("--on takes a date such as 2026-11-21"));
    project
        .run("anna", &["status", "--on", "2026-11-21", "--since", "last"])
        .failure()
        .code(2);
}

#[test]
fn a_values_file_that_cannot_be_read_is_a_warning_and_the_rest_of_the_screen_shows() {
    let project = project_with_entries();
    project.write(
        "DOSSIER/values.md",
        "<!-- dossier-format: 2 -->\n# Values of P-001\n",
    );
    project
        .command("anna", NOW)
        .arg("status")
        .assert()
        .success()
        .stderr(contains("warning: values.md"))
        .stdout(contains("## Decisions in force\nDEC-001 ·"));
}

#[test]
fn the_map_is_listed_after_the_project_lines_and_not_with_a_day_or_since() {
    let project = Project::new();
    project.write("src/solver/mod.rs", "solver\n");
    project
        .run("anna", &["map", "set", "src/solver", "time integration"])
        .success();
    assert_eq!(
        project.stdout("anna", &["status"]),
        "Project P-001\n\
         ## Map\n\
         src/solver  time integration\n\
         tasks: todo is not installed\n\
         ## Decisions in force\n\
         none\n\
         ## Newest findings\n\
         none\n\
         ## Unsettled\n\
         nothing waiting\n"
    );
    assert!(
        !project
            .stdout("anna", &["status", "--on", "2026-11-20"])
            .contains("## Map")
    );
    assert!(
        !project
            .stdout("anna", &["status", "--since", "2026-11-01"])
            .contains("## Map")
    );
}

#[test]
fn the_map_names_at_most_ten_lines_and_says_how_many_folders_are_left_out() {
    let project = Project::new();
    for number in 1..=11 {
        let folder = format!("folder-{number:02}");
        project.write(&format!("{folder}/notes.txt"), "notes\n");
        let purpose = format!("part {number:02}");
        project
            .run("anna", &["map", "set", folder.as_str(), purpose.as_str()])
            .success();
    }
    let stdout = project.stdout("anna", &["status"]);
    assert!(
        stdout.starts_with("Project P-001\n## Map\nfolder-01  part 01\n"),
        "{stdout}"
    );
    assert!(
        stdout.contains("folder-09  part 09\n2 more: dossier map\ntasks: todo is not installed\n"),
        "{stdout}"
    );
    assert!(!stdout.contains("folder-10"), "{stdout}");
    let data = status_data(&project, &[]);
    assert_eq!(data["map"].as_array().map(Vec::len), Some(10));
    assert_eq!(data["map"][9], "2 more: dossier map");
}

#[test]
fn open_assumptions_and_what_rests_on_a_wrong_assumption_are_listed_under_the_decisions() {
    let project = project_resting_on_a_wrong_assumption();
    project.stdout(
        "anna",
        &[
            "assume",
            "The wall function is valid",
            "--reason",
            "It was used in the last project.",
            "--affects",
            "T-042",
        ],
    );
    assert_eq!(
        project.stdout("anna", &["status"]),
        "Project P-001\n\
         tasks: todo is not installed\n\
         ## Decisions in force\n\
         DEC-001 · Use transient runs for the last three speedline points (anna, 2026-11-20)\n\
         ## Open assumptions\n\
         ASM-002 · The wall function is valid\n\
         ## Rests on a wrong assumption\n\
         DEC-001 · Use transient runs for the last three speedline points [rests on ASM-001]\n\
         T-042 data/model.sim [rests on ASM-001]\n\
         ## Newest findings\n\
         none\n\
         ## Unsettled\n\
         1 open assumption\n\
         2 entries and files rest on a wrong assumption\n"
    );
    let data = status_data(&project, &[]);
    assert_eq!(data["open_assumptions"], json!(["ASM-002"]));
    assert_eq!(data["rests"], json!(["DEC-001", "T-042 data/model.sim"]));
    assert_eq!(data["map"], json!([]));
}

#[test]
fn the_open_assumptions_are_cut_before_the_decisions_when_the_screen_is_too_long() {
    let project = Project::new();
    for number in 1..=3 {
        let title = format!("Decision number {number}");
        project
            .run("anna", &["decide", title.as_str(), "--reason", "A reason."])
            .success();
    }
    for number in 1..=3 {
        let title = format!("Assumption number {number}");
        project
            .run("anna", &["assume", title.as_str(), "--reason", "A reason."])
            .success();
    }
    let settings = project
        .read("DOSSIER/dossier.toml")
        .replace("status_lines = 60", "status_lines = 12");
    project.write("DOSSIER/dossier.toml", &settings);
    assert_eq!(
        project.stdout("anna", &["status"]),
        "Project P-001\n\
         tasks: todo is not installed\n\
         ## Decisions in force\n\
         DEC-003 · Decision number 3 (anna, 2026-11-20)\n\
         DEC-002 · Decision number 2 (anna, 2026-11-20)\n\
         DEC-001 · Decision number 1 (anna, 2026-11-20)\n\
         ## Open assumptions\n\
         3 more: dossier list --kind assumption\n\
         ## Newest findings\n\
         none\n\
         ## Unsettled\n\
         3 open assumptions\n"
    );
}

/// Writes `deciders = ["anna"]` into dossier.toml after the `code` line, so that anna may decline.
fn only_anna_decides(project: &Project) {
    let text = project.read("DOSSIER/dossier.toml");
    let mut edited = String::new();
    for line in text.split_inclusive('\n') {
        edited.push_str(line);
        if line.starts_with("code = ") {
            edited.push_str("deciders = [\"anna\"]\n");
        }
    }
    assert!(edited.contains("deciders = [\"anna\"]\n"), "{edited}");
    project.write("DOSSIER/dossier.toml", &edited);
}

#[test]
fn a_proposal_that_rests_on_a_refuted_assumption_is_listed_with_its_mark_and_counted() {
    let project = Project::new();
    project.stdout(
        "ben",
        &[
            "propose",
            "Use the steady runs",
            "--reason",
            "They converged.",
        ],
    );
    project.stdout(
        "anna",
        &[
            "assume",
            "The inlet profile is uniform",
            "--reason",
            "No measured profile was delivered.",
            "--affects",
            "DEC-001",
        ],
    );
    project
        .command("anna", NOW)
        .env("DOSSIER_ANSWERS", "")
        .args([
            "refute",
            "ASM-001",
            "--by",
            "anna, from the rig measurements",
        ])
        .assert()
        .success();
    assert_eq!(
        project.stdout("anna", &["status"]),
        "Project P-001\n\
         tasks: todo is not installed\n\
         ## Decisions in force\n\
         none\n\
         ## Rests on a wrong assumption\n\
         DEC-001 · Use the steady runs [proposed] [rests on ASM-001]\n\
         ## Newest findings\n\
         none\n\
         ## Unsettled\n\
         1 proposal waiting\n\
         1 entry or file rests on a wrong assumption\n"
    );
    let data = status_data(&project, &[]);
    assert_eq!(data["proposals"], json!(["DEC-001"]));
    assert_eq!(data["rests"], json!(["DEC-001"]));
}

#[test]
fn an_entry_that_is_rejected_declined_or_retired_leaves_the_list_and_the_count() {
    let project = Project::new();
    only_anna_decides(&project);
    project.stdout(
        "anna",
        &[
            "decide",
            "Use the transient runs",
            "--reason",
            "Steady runs converged where rotating stall already shows.",
        ],
    );
    project.stdout(
        "ben",
        &[
            "propose",
            "Use the steady runs",
            "--reason",
            "They converged.",
        ],
    );
    project.stdout(
        "ben",
        &["found", "The mass flow still oscillates", "--later"],
    );
    project.stdout(
        "anna",
        &[
            "assume",
            "The inlet profile is uniform",
            "--reason",
            "No measured profile was delivered.",
            "--affects",
            "DEC-001, DEC-002, FND-001",
        ],
    );
    project
        .command("anna", NOW)
        .env("DOSSIER_ANSWERS", "")
        .args([
            "refute",
            "ASM-001",
            "--by",
            "anna, from the rig measurements",
        ])
        .assert()
        .success();
    assert_eq!(
        project.stdout("anna", &["status"]),
        "Project P-001\n\
         tasks: todo is not installed\n\
         ## Decisions in force\n\
         DEC-001 · Use the transient runs (anna, 2026-11-20)\n\
         ## Rests on a wrong assumption\n\
         DEC-001 · Use the transient runs [rests on ASM-001]\n\
         FND-001 · The mass flow still oscillates [draft] [rests on ASM-001]\n\
         DEC-002 · Use the steady runs [proposed] [rests on ASM-001]\n\
         ## Newest findings\n\
         none\n\
         ## Unsettled\n\
         1 draft waiting\n\
         1 proposal waiting\n\
         3 entries and files rest on a wrong assumption\n"
    );

    project
        .run("anna", &["reject", "FND-001", "--reason", "Not needed."])
        .success();
    project
        .run(
            "anna",
            &["retire", "DEC-001", "--reason", "The plan changed."],
        )
        .success();
    project
        .command("anna", NOW)
        .env("DOSSIER_ANSWERS", "")
        .args([
            "decline",
            "DEC-002",
            "--reason",
            "The inlet profile was measured later.",
        ])
        .assert()
        .success();
    assert_eq!(
        project.stdout("anna", &["status"]),
        "Project P-001\n\
         tasks: todo is not installed\n\
         ## Decisions in force\n\
         none\n\
         ## Newest findings\n\
         none\n\
         ## Unsettled\n\
         nothing waiting\n"
    );
    let data = status_data(&project, &[]);
    assert_eq!(data["rests"], json!([]));
    assert_eq!(data["rests_on"], json!([]));
}

/// A team dossier made in a folder of its own, with anna as its only member, linked from the
/// project (`team` in its settings). Returns the folder, which keeps the team alive, and the team's
/// DOSSIER folder.
fn team_of(project: &Project) -> (tempfile::TempDir, std::path::PathBuf) {
    let folder = tempfile::TempDir::new().expect("a temporary folder for the team");
    assert_cmd::Command::cargo_bin("dossier")
        .expect("the dossier program is built")
        .current_dir(folder.path())
        .env("DOSSIER_NOW", NOW)
        .env("DOSSIER_AGENT", "anna")
        .env(
            "DOSSIER_REGISTER",
            project.root().join(".test").join("projects.toml"),
        )
        .env_remove("TODO_AGENT")
        .args(["init", "--team-dossier", "--code", "TEAM"])
        .assert()
        .success();
    let team = folder.path().join("DOSSIER");
    dossier::config::set_list(&team, "members", &["anna".to_string()]).expect("the members");
    let stored = team.display().to_string().replace('\\', "/");
    link_team(project, &stored);
    // A team write from the project needs the project's confidential terms first (R-CONF-5).
    project.stdout("anna", &["confidential", "add", "NT-999"]);
    (folder, team)
}

/// Writes `team = "<stored>"` into the settings of the project.
fn link_team(project: &Project, stored: &str) {
    dossier::config::set_value(
        &project.root().join("DOSSIER"),
        "team",
        &format!("\"{stored}\""),
    )
    .expect("the team link");
}

/// `learn` of a team lesson with this statement, run as `agent` at `now` (`--team`).
fn team_lesson(project: &Project, agent: &str, now: &str, statement: &str) {
    project
        .run_at(
            agent,
            now,
            &[
                "learn",
                statement,
                "--valid-for",
                "Transonic front stages.",
                "--evidence",
                "Report section 4.2.",
                "--confidence",
                "measured",
                "--team",
            ],
        )
        .success();
}

/// anna approves the team's draft `id` at `now`, answering `a` at the terminal (`--team`).
fn approve_team(project: &Project, id: &str, now: &str) {
    project
        .command("anna", now)
        .env("DOSSIER_ANSWERS", "a")
        .args(["approve", id, "--team"])
        .assert()
        .success();
}

#[test]
fn a_team_lesson_approved_after_the_marker_appears_under_team() {
    let project = Project::new();
    let (_folder, _team) = team_of(&project);
    team_lesson(
        &project,
        "anna - claude",
        "2026-11-21T09:00:00+01:00",
        "Steady mixing-plane runs overpredict stall margin",
    );
    // anna's first run at 09:30 writes her marker; the draft is not news.
    let first = out_at(
        &project,
        "anna",
        "2026-11-21T09:30:00+01:00",
        &["status", "--since", "last"],
    );
    assert!(first.ends_with("## Team\nnone\n"), "{first}");

    // anna approves the draft after the marker; a second draft stays a draft.
    approve_team(&project, "LES-001", "2026-11-21T10:00:00+01:00");
    team_lesson(
        &project,
        "anna - claude",
        "2026-11-21T10:30:00+01:00",
        "A second draft that nobody approves",
    );
    let second = out_at(
        &project,
        "anna",
        "2026-11-21T11:00:00+01:00",
        &["status", "--since", "last"],
    );
    assert_eq!(
        second,
        "Project P-001\n\
         ## Since 2026-11-21T09:30:00+01:00\n\
         ## Project P-001\n\
         none\n\
         ## Team\n\
         2026-11-21T10:00:00+01:00  anna  approved  LES-001  Steady mixing-plane runs overpredict stall margin\n"
    );
}

#[test]
fn since_a_date_lists_the_team_approvals_and_leaves_out_drafts_and_rejections() {
    let project = Project::new();
    let (_folder, _team) = team_of(&project);
    team_lesson(
        &project,
        "anna",
        "2026-11-21T09:00:00+01:00",
        "Created approved by a person",
    );
    team_lesson(
        &project,
        "anna - claude",
        "2026-11-21T09:10:00+01:00",
        "A draft by an LLM",
    );
    team_lesson(
        &project,
        "anna - claude",
        "2026-11-21T09:20:00+01:00",
        "A draft that is rejected",
    );
    project
        .run_at(
            "anna",
            "2026-11-21T09:30:00+01:00",
            &["reject", "LES-003", "--reason", "Not checked.", "--team"],
        )
        .success();
    assert_eq!(
        project.stdout("anna", &["status", "--since", "2026-11-21"]),
        "Project P-001\n\
         ## Since 2026-11-21\n\
         ## Project P-001\n\
         none\n\
         ## Team\n\
         2026-11-21T09:00:00+01:00  anna  created  LES-001  Created approved by a person\n"
    );

    approve_team(&project, "LES-002", "2026-11-21T10:00:00+01:00");
    assert_eq!(
        project.stdout("anna", &["status", "--since", "2026-11-21"]),
        "Project P-001\n\
         ## Since 2026-11-21\n\
         ## Project P-001\n\
         none\n\
         ## Team\n\
         2026-11-21T10:00:00+01:00  anna  approved  LES-002  A draft by an LLM\n\
         2026-11-21T09:00:00+01:00  anna  created  LES-001  Created approved by a person\n"
    );
    let data = status_data(&project, &["--since", "2026-11-21"]);
    assert_eq!(data["team_events"][0]["id"], "LES-002");
    assert_eq!(data["team_events"][0]["event"], "approved");
    assert_eq!(data["team_events"][1]["id"], "LES-001");
    assert_eq!(data["team_events"].as_array().map(Vec::len), Some(2));
    // Without a team dossier in the project there is no team part in the data.
    assert_eq!(status_data(&project, &[])["team_events"], Value::Null);
}

#[test]
fn the_team_part_names_at_most_twenty_lines_and_counts_the_rest() {
    let project = Project::new();
    let (_folder, _team) = team_of(&project);
    for number in 1..=21 {
        let statement = format!("Team lesson number {number:02}");
        team_lesson(&project, "anna", "2026-11-21T09:00:00+01:00", &statement);
    }
    let stdout = project.stdout("anna", &["status", "--since", "2026-11-21"]);
    let team: Vec<&str> = stdout
        .lines()
        .skip_while(|line| *line != "## Team")
        .skip(1)
        .collect();
    assert_eq!(team.len(), 20, "{stdout}");
    assert_eq!(
        team[0], "2026-11-21T09:00:00+01:00  anna  created  LES-021  Team lesson number 21",
        "{stdout}"
    );
    assert_eq!(
        team[18], "2026-11-21T09:00:00+01:00  anna  created  LES-003  Team lesson number 03",
        "{stdout}"
    );
    assert_eq!(team[19], "2 more: dossier log --team", "{stdout}");
    // The data is not cut: every piece of news is in it.
    let data = status_data(&project, &["--since", "2026-11-21"]);
    assert_eq!(data["team_events"].as_array().map(Vec::len), Some(21));
}

#[test]
fn a_team_that_cannot_be_reached_is_a_warning_and_its_part_is_left_out() {
    let project = Project::new();
    link_team(&project, "NOWHERE/DOSSIER");
    project
        .run("anna", &["status", "--since", "2026-11-21"])
        .success()
        .stderr(contains(
            "warning: the team dossier NOWHERE/DOSSIER cannot be reached",
        ));
    assert_eq!(
        project.stdout("anna", &["status", "--since", "2026-11-21"]),
        "Project P-001\n## Since 2026-11-21\nnone\n"
    );
}

/// Sets the `review` date of entry `id` as anna at the fixed clock, a person's change.
fn set_review(project: &Project, id: &str, review: &str) {
    let field = format!("review={review}");
    project
        .run("anna", &["set", id, "--field", field.as_str()])
        .success();
}

/// A draft how-to (format.md §22) written as the `howto` command would write it.
const DRAFT_HOW_TO: &str = "<!-- dossier-format: 1 -->\n\
     # HOW-001 · Restart the solver\n\
     \n\
     - kind: howto\n\
     - status: draft\n\
     - created: 2026-11-20T10:00:00+01:00\n\
     - author: anna\n\
     - software: ExampleSolver\n\
     - history: 2026-11-20T10:00:00+01:00 anna: created\n\
     \n\
     ## Purpose\n\
     \n\
     An invented purpose.\n\
     \n\
     ## Steps\n\
     \n\
     An invented step.\n";

/// A draft tool (format.md §22) written as the `tool` command would write it.
const DRAFT_TOOL: &str = "<!-- dossier-format: 1 -->\n\
     # TOOL-001 · Export the mesh\n\
     \n\
     - kind: tool\n\
     - status: draft\n\
     - created: 2026-11-20T10:00:00+01:00\n\
     - author: anna\n\
     - history: 2026-11-20T10:00:00+01:00 anna: created\n\
     \n\
     ## Purpose\n\
     \n\
     An invented purpose.\n";

#[test]
fn entries_the_screen_names_are_marked_when_their_review_date_has_passed() {
    let project = project_with_entries();
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
    for id in ["DEC-001", "ASM-001", "FND-002"] {
        set_review(&project, id, "2026-11-19");
    }

    let stdout = project.stdout("anna", &["status"]);

    let lines: Vec<&str> = stdout.lines().collect();
    assert!(
        lines.contains(
            &"DEC-001 · Use transient runs for the last three speedline points (anna, 2026-11-20) [due for review]"
        ),
        "{stdout}"
    );
    assert!(
        lines.contains(&"ASM-001 · Uniform inlet profile [due for review]"),
        "{stdout}"
    );
    assert!(
        lines.contains(
            &"FND-002 · Steady run stalls at low flow (anna, 2026-11-20) [due for review]"
        ),
        "{stdout}"
    );
}

#[test]
fn a_review_date_of_today_or_later_gives_no_due_mark_in_the_screen() {
    let project = project_with_entries();
    set_review(&project, "DEC-001", "2026-11-20");

    let stdout = project.stdout("anna", &["status"]);

    assert!(!stdout.contains("[due for review]"), "{stdout}");
}

#[test]
fn a_how_to_and_a_tool_changed_by_hand_after_their_approval_are_counted_as_changed() {
    let project = Project::new();
    project.write("DOSSIER/howtos/HOW-001-restart-the-solver.md", DRAFT_HOW_TO);
    project.write("DOSSIER/tools/TOOL-001-export-the-mesh.md", DRAFT_TOOL);
    project
        .command("anna", NOW)
        .env("DOSSIER_ANSWERS", "a|a")
        .args(["approve", "HOW-001", "TOOL-001"])
        .assert()
        .success();
    for path in [
        "DOSSIER/howtos/HOW-001-restart-the-solver.md",
        "DOSSIER/tools/TOOL-001-export-the-mesh.md",
    ] {
        let text = project.read(path);
        project.write(path, &format!("{text}\nA line added by hand.\n"));
    }

    let stdout = project.stdout("anna", &["status"]);
    assert!(
        stdout
            .lines()
            .any(|line| line == "2 approved lessons, how-tos or tools changed by hand"),
        "{stdout}"
    );

    let value = status_json(&project);
    assert_eq!(value["data"]["unsettled"]["changed_lessons"], 2);
}

/// Opens an investigation as anna with the title `title`, for `task` when one is given (R-INV-2).
fn open_problem(project: &Project, title: &str, task: Option<&str>) {
    let mut args = vec![
        "problem",
        "The density residual grows from iteration 40 on.",
        "--title",
        title,
    ];
    if let Some(task) = task {
        args.extend(["--task", task]);
    }
    project.stdout("anna", &args);
}

#[test]
fn an_open_investigation_waits_with_its_attempts_and_a_solved_one_does_not() {
    let project = Project::new();
    open_problem(&project, "Residual grows", None);
    project.stdout(
        "anna",
        &["tried", "INV-001", "The CFL number is too high", "--failed"],
    );
    project.stdout(
        "anna",
        &[
            "tried",
            "INV-001",
            "The outlet flux has the wrong sign",
            "--worked",
        ],
    );
    open_problem(&project, "Coarse mesh", None);
    project.stdout("anna", &["solved", "INV-002", "The mesh was refined"]);

    assert_eq!(
        project.stdout("anna", &["status"]),
        "Project P-001\n\
         tasks: todo is not installed\n\
         ## Waiting\n\
         INV-001 · Residual grows (investigation, 2 attempts)\n\
         ## Decisions in force\n\
         none\n\
         ## Newest findings\n\
         none\n\
         ## Unsettled\n\
         nothing waiting\n"
    );
    assert_eq!(
        status_data(&project, &[])["investigations"],
        json!(["INV-001"])
    );
}

#[test]
fn an_open_investigation_waits_after_the_tasks_that_wait_when_todo_is_there() {
    let project = Project::new();
    with_todo(&project, LIST, 0);
    open_problem(&project, "Residual grows", None);
    project.stdout(
        "anna",
        &["tried", "INV-001", "The CFL number is too high", "--failed"],
    );
    assert_eq!(
        project.stdout("anna", &["status"]),
        "Project P-001\n\
         ## Tasks\n\
         T-130 · Context work · 2/5 done\n\
         open 3, claimed 1, blocked 1, done 2\n\
         ## Working now\n\
         T-201 · Cut the speedline runs (ben-codex)\n\
         ## Waiting\n\
         T-202 · Get the mesh from the customer (blocked)\n\
         Q-001 · Is the wall function valid here? (question)\n\
         INV-001 · Residual grows (investigation, 1 attempt)\n\
         ## Decisions in force\n\
         none\n\
         ## Newest findings\n\
         none\n\
         ## Unsettled\n\
         nothing waiting\n\
         ## Next\n\
         T-209 · Tidy the old notes\n\
         T-203 · Write the status tests\n\
         T-204 · Write the milestone tests\n\
         T-205 · Write the cut tests\n\
         T-206 · Write the JSON tests\n"
    );
}

#[test]
fn milestone_limits_the_open_investigations_to_its_tasks() {
    let project = Project::new();
    open_problem(&project, "Residual grows", Some("T-042"));
    project.stdout(
        "anna",
        &["tried", "INV-001", "The CFL number is too high", "--failed"],
    );
    open_problem(&project, "Coarse mesh", Some("T-043"));

    assert_eq!(
        project.stdout("anna", &["status", "--milestone", "t-042"]),
        "Project P-001\n\
         tasks: todo is not installed\n\
         ## Waiting\n\
         INV-001 · Residual grows (investigation, 1 attempt)\n\
         ## Decisions in force\n\
         none\n\
         ## Newest findings\n\
         none\n\
         ## Unsettled\n\
         nothing waiting\n"
    );
    assert_eq!(
        status_data(&project, &["--milestone", "t-042"])["investigations"],
        json!(["INV-001"])
    );
}

#[test]
fn a_cut_waiting_part_names_the_investigation_list_when_only_investigations_are_in_it() {
    let project = Project::new();
    open_problem(&project, "Residual grows", None);
    open_problem(&project, "Coarse mesh", None);
    let settings = project
        .read("DOSSIER/dossier.toml")
        .replace("status_lines = 60", "status_lines = 10");
    project.write("DOSSIER/dossier.toml", &settings);

    assert_eq!(
        project.stdout("anna", &["status"]),
        "Project P-001\n\
         tasks: todo is not installed\n\
         ## Waiting\n\
         2 more: dossier list --kind investigation --state open\n\
         ## Decisions in force\n\
         none\n\
         ## Newest findings\n\
         none\n\
         ## Unsettled\n\
         nothing waiting\n"
    );
}

/// Takes a file of invented contents as `person` for `task`, at the fixed clock of the project.
fn take_file(project: &Project, person: &str, path: &str, task: &str) {
    project.write(path, "invented simulation data\n");
    project
        .run(person, &["take", path, "--task", task])
        .success();
}

#[test]
fn a_taken_file_is_listed_under_working_now_also_without_todo() {
    let project = Project::new();
    take_file(&project, "anna", "runs/case-a.dat", "T-130");
    assert_eq!(
        project.stdout("anna", &["status"]),
        "Project P-001\n\
         tasks: todo is not installed\n\
         ## Working now\n\
         runs/case-a.dat · taken by anna for T-130 since 2026-11-20\n\
         ## Decisions in force\n\
         none\n\
         ## Newest findings\n\
         none\n\
         ## Unsettled\n\
         nothing waiting\n"
    );
}

#[test]
fn a_taken_file_is_listed_after_the_claimed_tasks() {
    let project = Project::new();
    with_todo(&project, LIST, 0);
    take_file(&project, "anna", "runs/case-a.dat", "T-130");
    assert_eq!(
        project.stdout("anna", &["status"]),
        "Project P-001\n\
         ## Tasks\n\
         T-130 · Context work · 2/5 done\n\
         open 3, claimed 1, blocked 1, done 2\n\
         ## Working now\n\
         T-201 · Cut the speedline runs (ben-codex)\n\
         runs/case-a.dat · taken by anna for T-130 since 2026-11-20\n\
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
         T-209 · Tidy the old notes\n\
         T-203 · Write the status tests\n\
         T-204 · Write the milestone tests\n\
         T-205 · Write the cut tests\n\
         T-206 · Write the JSON tests\n"
    );
}

#[test]
fn milestone_lists_only_the_taken_files_whose_task_is_in_it() {
    let project = Project::new();
    with_todo(&project, LIST, 0);
    take_file(&project, "anna", "runs/case-a.dat", "T-201");
    take_file(&project, "ben", "runs/case-b.dat", "T-209");
    assert_eq!(
        project.stdout("anna", &["status", "--milestone", "T-130"]),
        "Project P-001\n\
         ## Tasks\n\
         T-130 · Context work · 2/5 done\n\
         ## Working now\n\
         T-201 · Cut the speedline runs (ben-codex)\n\
         runs/case-a.dat · taken by anna for T-201 since 2026-11-20\n\
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
fn a_returned_file_leaves_working_now() {
    let project = Project::new();
    take_file(&project, "anna", "runs/case-a.dat", "T-130");
    assert!(
        project.stdout("anna", &["status"]).contains(
            "## Working now\nruns/case-a.dat · taken by anna for T-130 since 2026-11-20\n"
        )
    );
    // `return --done` ends the take by removing the `taken` field of the block (format.md §30);
    // it is not built yet, so the field is removed here the way it would be.
    let mut file = BlockFile::parse(&project.read("DOSSIER/taken.md")).expect("the file parses");
    file.set_field("runs/case-a.dat", "taken", "");
    project.write("DOSSIER/taken.md", &file.text());
    assert_eq!(
        project.stdout("anna", &["status"]),
        "Project P-001\n\
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
fn json_gives_each_taken_file_with_its_person_and_task() {
    let project = Project::new();
    take_file(&project, "anna", "runs/case-a.dat", "T-130");
    take_file(&project, "ben", "runs/case-b.dat", "T-209");
    assert_eq!(
        status_json(&project)["data"]["taken"],
        json!([
            {"path": "runs/case-a.dat", "person": "anna", "task": "T-130"},
            {"path": "runs/case-b.dat", "person": "ben", "task": "T-209"},
        ])
    );
}

#[test]
fn a_taken_file_that_cannot_be_read_is_a_warning_and_the_rest_of_the_screen_shows() {
    let project = Project::new();
    std::fs::write(project.root().join("DOSSIER/taken.md"), [0xff, 0xfe, b'#'])
        .expect("writes the file");
    project
        .command("anna", NOW)
        .arg("status")
        .assert()
        .success()
        .stderr(contains("warning: taken.md"))
        .stdout(contains("## Unsettled\nnothing waiting\n"));
    assert_eq!(status_json(&project)["data"]["taken"], Value::Null);
}

/// `status --json` as anna, parsed.
fn status_json(project: &Project) -> Value {
    let stdout = project.stdout("anna", &["status", "--json"]);
    serde_json::from_str(stdout.trim()).expect("one JSON object")
}

/// A decision of anna for `task` (invented text); `standing` adds `standing: yes`, which keeps it
/// out of any digest (format.md §32).
fn decision_for(project: &Project, title: &str, task: &str, standing: bool) {
    let mut args = vec![
        "add",
        "decision",
        "--title",
        title,
        "--section",
        "Decision=Invented decision text.",
        "--section",
        "Reason=Invented reason.",
        "--task",
        task,
    ];
    if standing {
        args.extend(["--field", "standing=yes"]);
    }
    project.stdout("anna", &args);
}

/// A digest of `agent` for `milestone` whose `tasks` field is `tasks` (invented Summary).
fn digest_for(project: &Project, agent: &str, title: &str, milestone: &str, tasks: &str) {
    let milestone_field = format!("milestone={milestone}");
    let tasks_field = format!("tasks={tasks}");
    project.stdout(
        agent,
        &[
            "add",
            "digest",
            "--title",
            title,
            "--field",
            &milestone_field,
            "--field",
            &tasks_field,
            "--section",
            "Summary=Invented summary of the milestone.",
        ],
    );
}

#[test]
fn a_digest_in_force_replaces_the_decisions_and_findings_it_covers_and_a_standing_decision_stays() {
    let project = Project::new();
    decision_for(
        &project,
        "Use transient runs for the speedline",
        "T-056",
        false,
    );
    decision_for(&project, "Keep the inlet at the rig values", "T-056", true);
    project.stdout(
        "anna",
        &[
            "found",
            "The speedline is flat at the last point",
            "--task",
            "T-057",
        ],
    );
    project.stdout("anna", &["found", "The mesh has no inlet layer"]);
    digest_for(
        &project,
        "anna",
        "Milestone T-056 digests",
        "T-056",
        "T-056, T-057",
    );

    assert_eq!(
        project.stdout("anna", &["status"]),
        "Project P-001\n\
         tasks: todo is not installed\n\
         ## Digests\n\
         DIG-001 · T-056 · Milestone T-056 digests\n\
         2 entries of T-056 are summed up in DIG-001: dossier show DIG-001\n\
         ## Decisions in force\n\
         DEC-002 · Keep the inlet at the rig values (anna, 2026-11-20)\n\
         ## Newest findings\n\
         FND-002 · The mesh has no inlet layer (anna, 2026-11-20)\n\
         ## Unsettled\n\
         nothing waiting\n"
    );
    let data = status_data(&project, &[]);
    assert_eq!(data["digests"], json!(["DIG-001"]));
    assert_eq!(data["decisions"], json!(["DEC-002"]));
    assert_eq!(data["findings"], json!(["FND-002"]));
}

#[test]
fn milestone_lists_only_the_digest_of_that_milestone() {
    let project = Project::new();
    decision_for(
        &project,
        "Use the coarse mesh for the screening",
        "T-040",
        false,
    );
    decision_for(
        &project,
        "Use transient runs for the speedline",
        "T-056",
        false,
    );
    digest_for(
        &project,
        "anna",
        "Milestone T-040 digests",
        "T-040",
        "T-040",
    );
    digest_for(
        &project,
        "anna",
        "Milestone T-056 digests",
        "T-056",
        "T-056, T-057",
    );

    assert_eq!(
        project.stdout("anna", &["status"]),
        "Project P-001\n\
         tasks: todo is not installed\n\
         ## Digests\n\
         DIG-002 · T-056 · Milestone T-056 digests\n\
         1 entry of T-056 is summed up in DIG-002: dossier show DIG-002\n\
         DIG-001 · T-040 · Milestone T-040 digests\n\
         1 entry of T-040 is summed up in DIG-001: dossier show DIG-001\n\
         ## Decisions in force\n\
         none\n\
         ## Newest findings\n\
         none\n\
         ## Unsettled\n\
         nothing waiting\n"
    );
    assert_eq!(
        project.stdout("anna", &["status", "--milestone", "t-056"]),
        "Project P-001\n\
         tasks: todo is not installed\n\
         ## Digests\n\
         DIG-002 · T-056 · Milestone T-056 digests\n\
         1 entry of T-056 is summed up in DIG-002: dossier show DIG-002\n\
         ## Decisions in force\n\
         none\n\
         ## Newest findings\n\
         none\n\
         ## Unsettled\n\
         nothing waiting\n"
    );
    assert_eq!(
        status_data(&project, &["--milestone", "T-056"])["digests"],
        json!(["DIG-002"])
    );
}

#[test]
fn a_draft_digest_is_not_listed_and_covers_nothing() {
    let project = Project::new();
    decision_for(
        &project,
        "Use transient runs for the speedline",
        "T-056",
        false,
    );
    digest_for(
        &project,
        "ben-codex",
        "Milestone T-056 digests",
        "T-056",
        "T-056",
    );

    assert_eq!(
        project.stdout("anna", &["status"]),
        "Project P-001\n\
         tasks: todo is not installed\n\
         ## Decisions in force\n\
         DEC-001 · Use transient runs for the speedline (anna, 2026-11-20)\n\
         ## Newest findings\n\
         none\n\
         ## Unsettled\n\
         1 draft waiting\n"
    );
    let data = status_data(&project, &[]);
    assert_eq!(data["digests"], json!([]));
    assert_eq!(data["decisions"], json!(["DEC-001"]));
}
