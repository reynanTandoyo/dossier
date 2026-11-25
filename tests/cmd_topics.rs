//! Runs the built program for `topics` (docs/design.md section 21.2, R-TOPIC-4): each topic of the
//! project's list with the number of approved entries that name it, then the approved decisions,
//! findings and lessons that name no topic. Also `topics add`, which a person runs, or an LLM with
//! `--said-by <person>` (D52). Each
//! test runs in its own temporary project folder with the clock fixed by `common::Project`
//! (2026-11-20T10:00:00+01:00).

mod common;

use common::Project;
use predicates::str::contains;
use serde_json::json;

/// Writes `topics = [...]` into dossier.toml, after the `code` line, so that it is a top-level key
/// before the tables (format.md §35).
fn project_has_topics(project: &Project, names: &[&str]) {
    let list = names
        .iter()
        .map(|name| format!("\"{name}\""))
        .collect::<Vec<_>>()
        .join(", ");
    let text = project.read("DOSSIER/dossier.toml");
    let mut edited = String::new();
    for line in text.split_inclusive('\n') {
        edited.push_str(line);
        if line.starts_with("code = ") {
            edited.push_str(&format!("topics = [{list}]\n"));
        }
    }
    assert!(edited.contains("topics = ["), "{edited}");
    project.write("DOSSIER/dossier.toml", &edited);
}

/// Adds an invented finding as anna, a person, so that it is approved at once. The field topic is
/// written when `topic` is given.
fn add_finding(project: &Project, title: &str, topic: Option<&str>) {
    let section = "Observation=An invented observation for a test.";
    let field = topic.map(|topic| format!("topic={topic}"));
    let mut args = vec!["add", "finding", "--title", title, "--section", section];
    if let Some(field) = &field {
        args.extend(["--field", field.as_str()]);
    }
    project.stdout("anna", &args);
}

#[test]
fn the_list_counts_approved_entries_per_topic_and_the_ones_without_a_topic() {
    let project = Project::new();
    project_has_topics(&project, &["pumps", "rig"]);
    add_finding(&project, "The first seal leaks", Some("pumps"));
    add_finding(&project, "The rig is slow", Some("rig"));
    add_finding(&project, "The pump bearing is hot", Some("PUMPS"));
    add_finding(&project, "A note without a topic", None);
    // A draft names a topic but is left out of the counts.
    project.stdout(
        "anna",
        &[
            "found",
            "The old reader is slow",
            "--title",
            "Slow reader",
            "--later",
        ],
    );
    project.stdout("anna", &["set", "FND-005", "--field", "topic=rig"]);

    assert_eq!(
        project.stdout("anna", &["topics"]),
        "pumps  2\nrig  1\n(no topic)  1\n"
    );
}

#[test]
fn json_gives_the_same_numbers_as_the_text() {
    let project = Project::new();
    project_has_topics(&project, &["pumps", "rig"]);
    add_finding(&project, "The first seal leaks", Some("pumps"));
    add_finding(&project, "The rig is slow", Some("rig"));
    add_finding(&project, "The pump bearing is hot", Some("PUMPS"));
    add_finding(&project, "A note without a topic", None);
    project.stdout(
        "anna",
        &[
            "found",
            "The old reader is slow",
            "--title",
            "Slow reader",
            "--later",
        ],
    );
    project.stdout("anna", &["set", "FND-005", "--field", "topic=rig"]);

    let output = project.stdout("anna", &["topics", "--json"]);
    let value: serde_json::Value =
        serde_json::from_str(output.trim_end()).expect("one line of JSON");
    assert_eq!(value["schema"], "dossier.topics.v1");
    assert_eq!(value["ok"], true);
    assert_eq!(value["warnings"], json!([]));
    assert_eq!(
        value["data"],
        json!({
            "topics": [
                {"name": "pumps", "entries": 2},
                {"name": "rig", "entries": 1},
            ],
            "without": 1,
        })
    );
}

#[test]
fn a_topic_with_no_entries_is_listed_with_zero_and_no_topic_line_without_such_entries() {
    let project = Project::new();
    project_has_topics(&project, &["pumps", "rig"]);
    add_finding(&project, "The first seal leaks", Some("pumps"));

    assert_eq!(project.stdout("anna", &["topics"]), "pumps  1\nrig  0\n");
}

#[test]
fn without_topics_in_the_settings_the_list_says_how_to_add_one() {
    let project = Project::new();

    assert_eq!(
        project.stdout("anna", &["topics"]),
        "no topics: dossier topics add <name>\n"
    );
    let output = project.stdout("anna", &["topics", "--json"]);
    let value: serde_json::Value =
        serde_json::from_str(output.trim_end()).expect("one line of JSON");
    assert_eq!(value["data"]["topics"], json!([]));
}

#[test]
fn a_person_adds_a_topic_and_only_the_topics_line_of_the_settings_changes() {
    let project = Project::new();
    let before = project.read("DOSSIER/dossier.toml");

    assert_eq!(
        project.stdout("anna", &["topics", "add", "Pumps"]),
        "added topic pumps\n"
    );

    let after = project.read("DOSSIER/dossier.toml");
    assert!(
        after.lines().any(|line| line == "topics = [\"pumps\"]"),
        "{after}"
    );
    let others: Vec<&str> = after
        .lines()
        .filter(|line| !line.starts_with("topics = "))
        .collect();
    assert_eq!(others, before.lines().collect::<Vec<&str>>());
    assert_eq!(project.stdout("anna", &["topics"]), "pumps  0\n");
    assert!(
        project
            .stdout("", &["log"])
            .contains("  anna  changed  SETTINGS  topic added: pumps"),
        "the journal names the person and the change"
    );
}

#[test]
fn a_topic_already_on_the_list_changes_nothing_whatever_the_case() {
    let project = Project::new();
    project.stdout("anna", &["topics", "add", "pumps"]);
    let before = project.read("DOSSIER/dossier.toml");

    assert_eq!(
        project.stdout("anna", &["topics", "add", "PUMPS"]),
        "pumps is already a topic\n"
    );

    assert_eq!(project.read("DOSSIER/dossier.toml"), before);
    assert_eq!(
        project
            .stdout("", &["log"])
            .matches("topic added: pumps")
            .count(),
        1
    );
}

#[test]
fn an_llm_is_refused_and_the_settings_stay_as_they_are() {
    let project = Project::new();
    let before = project.read("DOSSIER/dossier.toml");

    project
        .run("anna-claude", &["topics", "add", "rig"])
        .failure()
        .code(1)
        .stderr(contains(
            "ask your person; with their yes: dossier topics add <name> --said-by <person>",
        ));

    assert_eq!(project.read("DOSSIER/dossier.toml"), before);
}

#[test]
fn an_llm_adds_a_topic_with_said_by_and_the_journal_names_the_person() {
    let project = Project::new();

    project
        .run(
            "anna-claude",
            &["topics", "add", "rig", "--said-by", "anna"],
        )
        .success()
        .stdout("added topic rig\n");

    let after = project.read("DOSSIER/dossier.toml");
    assert!(
        after.lines().any(|line| line == "topics = [\"rig\"]"),
        "{after}"
    );
    assert!(
        project
            .stdout("", &["log"])
            .contains("topic added: rig (said by anna)"),
        "the journal names the person who agreed"
    );
}

#[test]
fn an_llm_naming_a_person_who_is_not_in_the_settings_is_refused() {
    let project = Project::new();
    let before = project.read("DOSSIER/dossier.toml");

    project
        .run(
            "anna-claude",
            &["topics", "add", "rig", "--said-by", "nobody"],
        )
        .failure()
        .code(1)
        .stderr(contains("no person nobody in dossier.toml"));

    assert_eq!(project.read("DOSSIER/dossier.toml"), before);
}

#[test]
fn a_name_that_is_empty_has_a_comma_or_a_line_break_or_more_than_three_words_is_a_usage_error() {
    let project = Project::new();
    let before = project.read("DOSSIER/dossier.toml");

    for name in ["a, b", "", "   ", "pumps\nrig"] {
        project
            .run("anna", &["topics", "add", name])
            .failure()
            .code(2);
    }
    project
        .run("anna", &["topics", "add", "one two three four"])
        .failure()
        .code(2)
        .stderr(contains("a topic has at most three words"));

    assert_eq!(project.read("DOSSIER/dossier.toml"), before);
}

#[test]
fn a_dry_run_says_what_it_would_write_and_writes_nothing() {
    let project = Project::new();
    let before = project.read("DOSSIER/dossier.toml");

    project
        .run("anna", &["topics", "add", "rig", "--dry-run"])
        .success()
        .stdout("added topic rig\n")
        .stderr(contains("warning: dry run: would write"));

    assert_eq!(project.read("DOSSIER/dossier.toml"), before);
    assert!(!project.stdout("", &["log"]).contains("topic added"));
}
