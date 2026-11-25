//! Runs the built program for `topics` (docs/design.md section 21.2, R-TOPIC-4): each topic of the
//! project's list with the number of approved entries that name it, then the approved decisions,
//! findings and lessons that name no topic. Each test runs in its own temporary project folder with
//! the clock fixed by `common::Project` (2026-11-20T10:00:00+01:00).

mod common;

use common::Project;
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
