//! Runs the built program for `log`: the events of the whole dossier, newest first, one line each
//! with the time, the person (and `via` the LLM), the event, the entry and its title; the filters
//! narrow them; `--limit` caps the lines; an entry that is gone shows `(removed)`; a settings event
//! (init's DOSSIER record) shows its detail where the title goes; `--json` gives the records with
//! their title; a bad kind or date is refused. Each test runs in its own temporary project
//! (common::Project), with the clock fixed at 2026-11-20T10:00:00+01:00 unless a test gives another
//! time. Project::new runs init as anna, so every project has the settings event of init; the tests
//! that count entry events leave it out (see `entries_only`).

mod common;

use common::{Project, is_settings_id};
use predicates::str::contains;
use serde_json::Value;
use std::fs;

/// Invented texts for the decision of anna (on the fixed clock) and the finding of ben-codex.
const DECISION_TITLE: &str = "Use transient runs for the last three speedline points";
const REASON: &str = "Steady runs converged where the transient run already showed rotating stall.";
const DECISION_TIME: &str = "2026-11-20T10:00:00+01:00";
const FINDING_TITLE: &str = "Mixing-plane runs converge at high back pressure";
const FINDING_TIME: &str = "2026-11-21T10:00:00+01:00";

/// The time of the init event that Project::new writes: the fixed clock.
const INIT_TIME: &str = "2026-11-20T10:00:00+01:00";

/// Invented texts for the lesson that ben-codex drafts and anna approves.
const LESSON_TITLE: &str = "Mixing plane margin";
const STATEMENT: &str = "Steady mixing-plane runs overpredict the stall margin.";
const VALID_FOR: &str = "An invented test rig with a tip Mach number above 1.1.";
const EVIDENCE: &str = "An invented comparison of two speedlines, section 1.";
const LESSON_TIME: &str = "2026-11-22T10:00:00+01:00";
const APPROVAL_TIME: &str = "2026-11-23T09:00:00+01:00";

/// A project with DEC-001, a decision that anna made on the fixed clock, and FND-001, a finding
/// that ben-codex wrote a day later.
fn two_events() -> Project {
    let project = Project::new();
    project
        .run(
            "anna",
            &[
                "decide",
                DECISION_TITLE,
                "--reason",
                REASON,
                "--title",
                DECISION_TITLE,
            ],
        )
        .success();
    project
        .run_at(
            "ben-codex",
            FINDING_TIME,
            &["found", FINDING_TITLE, "--title", FINDING_TITLE],
        )
        .success();
    project
}

/// The lines of `stdout` that are events: every line before the count line.
fn event_lines(stdout: &str) -> Vec<&str> {
    stdout.lines().filter(|line| line.contains("  ")).collect()
}

/// Whether an event line is a settings event. Its fields are separated by two spaces:
/// `<time>  <person>[ via <name>]  <event>  <ID>  <title>`, so the ID is the fourth field.
fn is_settings_line(line: &str) -> bool {
    line.split("  ").nth(3).is_some_and(is_settings_id)
}

/// The output of `log` with the settings events left out: the entry event lines, then a count
/// line that counts those alone. The same text as `log` gives when the settings events are not
/// there.
fn entries_only(stdout: &str) -> String {
    let events: Vec<&str> = event_lines(stdout)
        .into_iter()
        .filter(|line| !is_settings_line(line))
        .collect();
    let count = if events.len() == 1 {
        "1 event".to_string()
    } else {
        format!("{} events", events.len())
    };
    let mut text: String = events.iter().map(|line| format!("{line}\n")).collect();
    text.push_str(&count);
    text.push('\n');
    text
}

#[test]
fn two_events_show_newest_first_with_time_person_llm_entry_and_title() {
    let project = two_events();

    let expected = format!(
        "{FINDING_TIME}  ben via ben-codex  created  FND-001  {FINDING_TITLE}\n\
         {DECISION_TIME}  anna  created  DEC-001  {DECISION_TITLE}\n\
         2 events\n"
    );
    assert_eq!(entries_only(&project.stdout("", &["log"])), expected);
}

#[test]
fn the_person_filter_ignores_case_and_leaves_one_line() {
    let project = two_events();

    let stdout = entries_only(&project.stdout("", &["log", "--person", "ANNA"]));
    assert_eq!(event_lines(&stdout).len(), 1, "{stdout}");
    assert!(stdout.contains("DEC-001"), "{stdout}");
    assert!(!stdout.contains("FND-001"), "{stdout}");
    assert!(stdout.ends_with("1 event\n"), "{stdout}");
}

#[test]
fn the_kind_filter_takes_a_name_or_a_prefix_in_any_case() {
    let project = two_events();

    for kind in ["finding", "FND", "Finding"] {
        let stdout = project.stdout("", &["log", "--kind", kind]);
        assert_eq!(event_lines(&stdout).len(), 1, "{kind}: {stdout}");
        assert!(stdout.contains("FND-001"), "{kind}: {stdout}");
    }
    project
        .run("", &["log", "--kind", "lesson"])
        .success()
        .stdout("0 events\n");
}

#[test]
fn since_and_until_are_inclusive_dates_of_the_time_stamp() {
    let project = two_events();

    let since = project.stdout("", &["log", "--since", "2026-11-21"]);
    assert!(
        since.contains("FND-001") && !since.contains("DEC-001"),
        "{since}"
    );

    project
        .run("", &["log", "--since", "2026-11-22"])
        .success()
        .stdout("0 events\n");

    let until = project.stdout("", &["log", "--until", "2026-11-20"]);
    assert!(
        until.contains("DEC-001") && !until.contains("FND-001"),
        "{until}"
    );

    let one_day = project.stdout(
        "",
        &["log", "--since", "2026-11-20", "--until", "2026-11-20"],
    );
    assert!(
        one_day.contains("DEC-001") && !one_day.contains("FND-001"),
        "{one_day}"
    );
}

#[test]
fn the_event_filter_is_exact_and_finds_an_approval() {
    let project = two_events();
    project
        .run_at(
            "ben-codex",
            LESSON_TIME,
            &[
                "learn",
                STATEMENT,
                "--valid-for",
                VALID_FOR,
                "--evidence",
                EVIDENCE,
                "--confidence",
                "simulated",
                "--title",
                LESSON_TITLE,
            ],
        )
        .success();
    project
        .command("anna", APPROVAL_TIME)
        .env("DOSSIER_ANSWERS", "a")
        .args(["approve", "LES-001"])
        .assert()
        .success();

    let created = entries_only(&project.stdout("", &["log", "--event", "created"]));
    assert_eq!(event_lines(&created).len(), 3, "{created}");
    assert!(created.ends_with("3 events\n"), "{created}");

    let approved = project.stdout("", &["log", "--event", "approved"]);
    let expected = format!("{APPROVAL_TIME}  anna  approved  LES-001  {LESSON_TITLE}\n1 event\n");
    assert_eq!(approved, expected);

    project
        .run("", &["log", "--event", "CREATED"])
        .success()
        .stdout("0 events\n");
}

#[test]
fn limit_caps_the_lines_and_says_how_many_more_matched() {
    let project = two_events();

    // Every matched event counts, the settings event of init too: the number of the others is
    // taken from the listing without a limit.
    let all = project.stdout("", &["log"]);
    let total = event_lines(&all).len();
    assert!(total > 1, "{all}");

    let expected = format!(
        "{FINDING_TIME}  ben via ben-codex  created  FND-001  {FINDING_TITLE}\n\
         1 event\n\
         ({} more; use --limit)\n",
        total - 1
    );
    assert_eq!(project.stdout("", &["log", "--limit", "1"]), expected);
}

#[test]
fn an_entry_that_is_gone_shows_removed_in_place_of_its_title() {
    let project = two_events();
    fs::remove_file(project.entry_file("DEC-001")).expect("remove the decision file");

    let expected = format!(
        "{FINDING_TIME}  ben via ben-codex  created  FND-001  {FINDING_TITLE}\n\
         {DECISION_TIME}  anna  created  DEC-001  (removed)\n\
         2 events\n"
    );
    assert_eq!(entries_only(&project.stdout("", &["log"])), expected);
}

#[test]
fn a_settings_event_shows_its_detail_in_place_of_the_title() {
    let project = Project::new();

    let stdout = project.stdout("", &["log"]);
    let init_line = format!("{INIT_TIME}  anna  created  DOSSIER  code P-001\n");
    assert!(stdout.contains(&init_line), "{stdout}");
    assert!(!stdout.contains("(removed)"), "{stdout}");

    let value: Value =
        serde_json::from_str(project.stdout("", &["--json", "log"]).trim()).expect("one JSON line");
    let data = value["data"].as_array().expect("a list of records");
    let init = data
        .iter()
        .find(|record| record["id"] == "DOSSIER")
        .expect("the init record");
    assert_eq!(init["title"], Value::Null);
    assert_eq!(init["detail"], "code P-001");
    assert_eq!(init["person"], "anna");
    assert_eq!(init["llm"], false);
}

#[test]
fn json_gives_the_records_newest_first_with_their_titles() {
    let project = two_events();

    let stdout = project.stdout("", &["--json", "log"]);
    let value: Value = serde_json::from_str(stdout.trim()).expect("one JSON line");
    assert_eq!(value["schema"], "dossier.log.v1");
    assert_eq!(value["ok"], true);
    assert_eq!(value["warnings"], serde_json::json!([]));

    // The settings event of init is left out: this test is about the two entry events.
    let data: Vec<Value> = value["data"]
        .as_array()
        .expect("a list of records")
        .iter()
        .filter(|record| !is_settings_id(record["id"].as_str().unwrap_or("")))
        .cloned()
        .collect();
    assert_eq!(data.len(), 2);
    assert_eq!(data[0]["id"], "FND-001");
    assert_eq!(data[0]["time"], FINDING_TIME);
    assert_eq!(data[0]["person"], "ben");
    assert_eq!(data[0]["name"], "ben-codex");
    assert_eq!(data[0]["llm"], true);
    assert_eq!(data[0]["title"], FINDING_TITLE);
    assert_eq!(data[1]["id"], "DEC-001");
    assert_eq!(data[1]["person"], "anna");
    assert_eq!(data[1]["llm"], false);
    assert_eq!(data[1]["title"], DECISION_TITLE);
}

#[test]
fn an_unknown_kind_or_a_date_that_is_not_one_is_refused() {
    let project = two_events();

    project
        .run("", &["log", "--kind", "idea"])
        .failure()
        .code(1)
        .stderr(contains("idea"));
    project
        .run("", &["log", "--since", "yesterday"])
        .failure()
        .code(1)
        .stderr(contains("yesterday"));
}

#[test]
fn an_empty_journal_says_zero_events() {
    let project = Project::new();
    // Project::new has written the init event; without the journal there are no events at all.
    fs::remove_file(
        project
            .root()
            .join("DOSSIER")
            .join(".state")
            .join("journal.jsonl"),
    )
    .expect("remove the journal");

    project.run("", &["log"]).success().stdout("0 events\n");
}
