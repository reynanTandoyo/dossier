//! Runs the built program for `digest <milestone>`: the material for summing up a milestone (the
//! title, goal and tasks from todo, the approved entries of its tasks under their headings, the
//! rules for the summary), the JSON data, the journal left as it was, the refusals without todo
//! and the `--write` arguments that are not built yet (R-CTX-14).
//! Each test runs in its own temporary project folder with the clock fixed; the milestone, the
//! tasks and the entries are invented.

mod common;

use common::Project;
use predicates::str::contains;
use serde_json::{Value, json};

/// The answer of todo to `show T-900` (invented): the title and status at the top of the data, the
/// goal at the top too.
const SHOW: &str = r#"{"schema":"todo.show.v1","ok":true,"data":{"id":"T-900","title":"Invented milestone of the digest tests","status":"done","goal":"Invented goal: the digest tests run on it"},"warnings":[]}"#;

/// The answer of todo to `show T-900` (invented) with the goal under `fields`, as todo may give it.
const SHOW_FIELDS: &str = r#"{"schema":"todo.show.v1","ok":true,"data":{"id":"T-900","title":"Invented milestone of the digest tests","status":"done","fields":{"goal":"Invented goal under fields"}},"warnings":[]}"#;

/// The answer of todo to `list --milestone T-900 --status all` (invented): one task done and one
/// dropped.
const LIST: &str = r#"{"schema":"todo.list.v1","ok":true,"data":{"tasks":[{"id":"T-901","title":"Invented first task","status":"done","owner":"anna-claude","ready":false,"priority":"P1"},{"id":"T-902","title":"Invented second task","status":"dropped","owner":null,"ready":false,"priority":"P2"}]},"warnings":[]}"#;

/// The answer of todo to `list --milestone T-900 --status all` when the milestone has no tasks.
const EMPTY_LIST: &str = r#"{"schema":"todo.list.v1","ok":true,"data":{"tasks":[]},"warnings":[]}"#;

/// The stand-in for todo. It answers `--version` with exit 0, appends the arguments of every other
/// call to calls.txt in the project folder, and answers `show` with `show` and any other call with
/// `list`. The JSON of the answers has no character that the batch file would read as syntax.
fn todo_stand_in(show: &str, list: &str) -> (String, String) {
    let windows = [
        "@echo off".to_string(),
        "if \"%~1\"==\"--version\" exit /b 0".to_string(),
        "echo %* >> calls.txt".to_string(),
        "if \"%~1\"==\"show\" goto show".to_string(),
        format!("echo {list}"),
        "exit /b 0".to_string(),
        ":show".to_string(),
        format!("echo {show}"),
        "exit /b 0".to_string(),
    ]
    .join("\r\n")
        + "\r\n";
    let unix = [
        "#!/bin/sh".to_string(),
        "if [ \"$1\" = \"--version\" ]; then exit 0; fi".to_string(),
        "echo \"$@\" >> calls.txt".to_string(),
        "if [ \"$1\" = \"show\" ]; then".to_string(),
        format!("  printf '%s\\n' '{show}'; exit 0"),
        "fi".to_string(),
        format!("printf '%s\\n' '{list}'"),
    ]
    .join("\n")
        + "\n";
    (windows, unix)
}

/// A project with the stand-in of todo answering with `show` and `list`, and no entries.
fn project_with_todo(show: &str, list: &str) -> Project {
    let project = Project::new();
    let (windows, unix) = todo_stand_in(show, list);
    project.write_stand_in("todo", &windows, &unix);
    project
}

/// A project with the stand-in of todo answering with SHOW and LIST, and the entries of the
/// digest tests (invented). The milestone T-900 has the tasks T-901 and T-902. Recorded as anna
/// (a person): DEC-001 for T-901, DEC-002 for T-777 (another task), ASM-001 (open) for T-901,
/// FND-001 for T-901, FND-002 a dead end for T-902, INV-001 (open) for T-902, and FND-003 for
/// T-901 as a draft (`--later`), which is not approved.
fn project_with_entries() -> Project {
    let project = project_with_todo(SHOW, LIST);
    project
        .run(
            "anna",
            &[
                "decide",
                "Keep the speedline runs at three points",
                "--reason",
                "Steady runs converged where stall shows.",
                "--title",
                "Three speedline points",
                "--task",
                "T-901",
            ],
        )
        .success();
    project
        .run(
            "anna",
            &[
                "decide",
                "Use the other mesh for the invented case",
                "--reason",
                "Invented reason of another task.",
                "--title",
                "Other mesh",
                "--task",
                "T-777",
            ],
        )
        .success();
    project
        .run(
            "anna",
            &[
                "assume",
                "The wall function is valid at this Reynolds number",
                "--reason",
                "Invented reason: the wall law was not checked.",
                "--title",
                "Wall function valid",
                "--task",
                "T-901",
            ],
        )
        .success();
    project
        .run(
            "anna",
            &[
                "found",
                "The mass flow still oscillates with half the time step",
                "--title",
                "Mass flow oscillates",
                "--task",
                "T-901",
            ],
        )
        .success();
    project
        .run(
            "anna",
            &[
                "found",
                "The first run diverges at iteration 40",
                "--dead-end",
                "--title",
                "Run diverges",
                "--task",
                "T-902",
            ],
        )
        .success();
    project
        .run(
            "anna",
            &[
                "problem",
                "The residuals stall after iteration 300",
                "--title",
                "Residuals stall",
                "--task",
                "T-902",
            ],
        )
        .success();
    project
        .run(
            "anna",
            &[
                "found",
                "A draft note of the first task",
                "--later",
                "--title",
                "Draft note",
                "--task",
                "T-901",
            ],
        )
        .success();
    project
}

/// The text of `digest T-900` for the project of `project_with_entries`.
fn expected_material() -> String {
    [
        "# Digest of T-900 · Invented milestone of the digest tests",
        "goal: Invented goal: the digest tests run on it",
        "## Tasks",
        "T-901 · Invented first task (done)",
        "T-902 · Invented second task (dropped)",
        "## Decisions",
        "DEC-001 · Three speedline points",
        "  Keep the speedline runs at three points",
        "## Assumptions",
        "ASM-001 · Wall function valid [open]",
        "  The wall function is valid at this Reynolds number",
        "## Findings",
        "FND-001 · Mass flow oscillates",
        "  The mass flow still oscillates with half the time step",
        "## Dead ends",
        "FND-002 · Run diverges",
        "  The first run diverges at iteration 40",
        "## Investigations",
        "INV-001 · Residuals stall [open]",
        "  The residuals stall after iteration 300",
        "## Rules for the summary",
        "1. At most 15 lines.",
        "2. What was decided and still binds.",
        "3. What was found.",
        "4. What was assumed and is still open.",
        "5. What did not work.",
        "Write it with: dossier digest T-900 --write --stdin",
    ]
    .join("\n")
        + "\n"
}

#[test]
fn the_material_lists_the_tasks_the_approved_entries_of_them_and_the_rules() {
    let project = project_with_entries();
    let stdout = project.stdout("anna", &["digest", "T-900"]);
    assert_eq!(stdout, expected_material());
}

#[test]
fn an_entry_of_another_task_and_a_draft_are_not_listed() {
    let project = project_with_entries();
    let value: Value = serde_json::from_str(
        project
            .stdout("anna", &["digest", "T-900", "--json"])
            .trim(),
    )
    .expect("the JSON parses");
    let entries = value["data"]["entries"]
        .as_array()
        .expect("the entries are a list");
    assert!(!entries.iter().any(|id| id == "DEC-002"), "{entries:?}");
    assert!(!entries.iter().any(|id| id == "FND-003"), "{entries:?}");
    let stdout = project.stdout("anna", &["digest", "T-900"]);
    assert!(!stdout.contains("Other mesh"), "{stdout}");
    assert!(!stdout.contains("Draft note"), "{stdout}");
}

#[test]
fn json_gives_the_milestone_its_tasks_the_entry_ids_and_the_rules() {
    let project = project_with_entries();
    let stdout = project.stdout("anna", &["digest", "T-900", "--json"]);
    let value: Value = serde_json::from_str(stdout.trim()).expect("the JSON parses");
    assert_eq!(value["schema"], "dossier.digest.v1");
    assert_eq!(value["ok"], true);
    assert_eq!(value["warnings"], json!([]));
    let data = &value["data"];
    assert_eq!(data["milestone"], "T-900");
    assert_eq!(data["title"], "Invented milestone of the digest tests");
    assert_eq!(data["goal"], "Invented goal: the digest tests run on it");
    assert_eq!(data["status"], "done");
    assert_eq!(
        data["tasks"],
        json!([
            {"id": "T-901", "title": "Invented first task", "status": "done"},
            {"id": "T-902", "title": "Invented second task", "status": "dropped"},
        ])
    );
    assert_eq!(
        data["entries"],
        json!(["DEC-001", "ASM-001", "FND-001", "FND-002", "INV-001"])
    );
    assert_eq!(
        data["rules"],
        json!([
            "At most 15 lines.",
            "What was decided and still binds.",
            "What was found.",
            "What was assumed and is still open.",
            "What did not work.",
        ])
    );
}

/// The arguments of each call the stand-in of todo got so far, one line per call; none when it
/// has not been called.
fn todo_calls(project: &Project) -> Vec<String> {
    std::fs::read_to_string(project.root().join("calls.txt"))
        .unwrap_or_default()
        .lines()
        .map(|line| line.trim_end().to_string())
        .collect()
}

#[test]
fn digest_changes_nothing_and_asks_todo_for_the_milestone_and_its_tasks() {
    let project = project_with_entries();
    let journal = project.read("DOSSIER/.state/journal.jsonl");
    let before = todo_calls(&project).len();
    project.stdout("anna", &["digest", "T-900"]);
    assert_eq!(project.read("DOSSIER/.state/journal.jsonl"), journal);
    let calls = todo_calls(&project);
    assert_eq!(
        calls[before..],
        [
            "show T-900 --json".to_string(),
            "list --milestone T-900 --status all --json".to_string(),
        ]
    );
}

#[test]
fn the_goal_may_be_under_fields_and_a_heading_without_entries_is_left_out() {
    let project = project_with_todo(SHOW_FIELDS, EMPTY_LIST);
    let stdout = project.stdout("anna", &["digest", "T-900"]);
    let expected = [
        "# Digest of T-900 · Invented milestone of the digest tests",
        "goal: Invented goal under fields",
        "## Rules for the summary",
        "1. At most 15 lines.",
        "2. What was decided and still binds.",
        "3. What was found.",
        "4. What was assumed and is still open.",
        "5. What did not work.",
        "Write it with: dossier digest T-900 --write --stdin",
    ]
    .join("\n")
        + "\n";
    assert_eq!(stdout, expected);
}

#[test]
fn without_todo_the_digest_is_refused_with_todo_missing() {
    let project = Project::new();
    project
        .run("anna", &["digest", "T-900"])
        .code(5)
        .stderr(contains("todo is not installed"));
}

#[test]
fn write_needs_stdin_and_with_stdin_the_digest_entry_is_not_built_yet() {
    let project = Project::new();
    project
        .run("anna", &["digest", "T-900", "--write"])
        .code(2)
        .stderr(contains("needs --stdin"));
    project
        .run("anna", &["digest", "T-900", "--write", "--stdin"])
        .code(2)
        .stderr(contains("not built yet"));
}

#[test]
fn stdin_and_title_need_write() {
    let project = Project::new();
    project
        .run("anna", &["digest", "T-900", "--stdin"])
        .code(2)
        .stderr(contains("--write"));
    project
        .run("anna", &["digest", "T-900", "--title", "Invented title"])
        .code(2)
        .stderr(contains("--write"));
}
