//! Finding tests (TEST-12): a dossier of invented entries, a fixed list of searches that must each
//! find their entry among the first three results (R-FIND-1 to 4, R-READ-3), `search --similar`
//! that lists the most related decision first, and a search that finds nothing. The entries are
//! written one minute apart with the fixed clock, in one temporary project.

mod common;

use common::Project;
use serde_json::Value;

/// The clock of the first write; each later write is one minute later.
fn at(minute: u32) -> String {
    format!("2026-11-20T10:{minute:02}:00+01:00")
}

/// The decisions, in the order they are written: the title and the reason.
const DECISIONS: [(&str, &str); 7] = [
    (
        "Each rig command opens its own valve connection",
        "Keeping a connection open needs a helper process.",
    ),
    (
        "Pump seals are replaced every 500 hours",
        "Two seals leaked after 600 hours.",
    ),
    (
        "Speedline runs use the transient mode above 90 percent speed",
        "Steady runs did not converge there.",
    ),
    (
        "Results are stored as one file per run",
        "One large file was lost twice.",
    ),
    (
        "The bearing temperature limit is 95 degrees",
        "The supplier names 100 degrees.",
    ),
    (
        "The controller firmware is not updated during a campaign",
        "An update changed the valve timing once.",
    ),
    (
        "Every report names the rig and the date",
        "Reports were mixed up.",
    ),
];

/// The findings, in the order they are written.
const FINDINGS: [&str; 4] = [
    "Reusing one valve connection fails on the old controller",
    "Cavitation starts below 2 bar inlet pressure",
    "The rig log and the controller clock differ by two seconds",
    "The spare pump has a different impeller",
];

/// The glossary term, its meaning and its other spelling.
const TERM: &str = "ControlLine";
const TERM_MEANING: &str = "The line that opens the rig valve.";
const TERM_ALSO: &str = "valve connection";

/// The fixed questions: the words of a search, and the title of the entry that must be among the
/// first three results.
const QUESTIONS: [(&str, &str); 10] = [
    (
        "connection reuse",
        "Reusing one valve connection fails on the old controller",
    ),
    (
        "reused connections",
        "Reusing one valve connection fails on the old controller",
    ),
    (
        "seal replacement",
        "Pump seals are replaced every 500 hours",
    ),
    ("storing results", "Results are stored as one file per run"),
    (
        "bearing temperature",
        "The bearing temperature limit is 95 degrees",
    ),
    (
        "cavitation pressure",
        "Cavitation starts below 2 bar inlet pressure",
    ),
    (
        "firmware updates",
        "The controller firmware is not updated during a campaign",
    ),
    (
        "ControlLine",
        "Each rig command opens its own valve connection",
    ),
    (
        "impeller of the spare pumps",
        "The spare pump has a different impeller",
    ),
    (
        "transient speedline",
        "Speedline runs use the transient mode above 90 percent speed",
    ),
];

/// A project with the seven decisions, the four findings and the glossary term, written as anna,
/// one minute apart, in that order.
fn dossier() -> Project {
    let project = Project::new();
    let mut minute = 0;
    for (title, reason) in DECISIONS {
        project
            .run_at("anna", &at(minute), &["decide", title, "--reason", reason])
            .success();
        minute += 1;
    }
    for finding in FINDINGS {
        project
            .run_at("anna", &at(minute), &["found", finding])
            .success();
        minute += 1;
    }
    project
        .run_at(
            "anna",
            &at(minute),
            &["term", "add", TERM, TERM_MEANING, "--also", TERM_ALSO],
        )
        .success();
    project
}

/// The JSON data of a `--json` run of `args`, as the parsed envelope.
fn json_of(project: &Project, args: &[&str]) -> Value {
    let json = project.stdout("", args);
    serde_json::from_str(&json).expect("one JSON line")
}

/// The titles of the first three results of `search <words>`, in the printed order.
fn first_three_titles(project: &Project, words: &str) -> Vec<String> {
    let mut args = vec!["--json", "search"];
    args.extend(words.split_whitespace());
    let value = json_of(project, &args);
    value["data"]
        .as_array()
        .expect("the results")
        .iter()
        .take(3)
        .map(|hit| hit["title"].as_str().unwrap_or_default().to_string())
        .collect()
}

#[test]
fn every_fixed_question_finds_its_entry_among_the_first_three() {
    let project = dossier();

    let mut failures = Vec::new();
    for (words, expected) in QUESTIONS {
        let titles = first_three_titles(&project, words);
        if !titles.iter().any(|title| title == expected) {
            failures.push(format!(
                "  search {words}: expected {expected:?}, got {titles:?}"
            ));
        }
    }
    assert!(
        failures.is_empty(),
        "{} of {} searches did not find their entry among the first three:\n{}",
        failures.len(),
        QUESTIONS.len(),
        failures.join("\n")
    );
}

#[test]
fn similar_to_the_valve_finding_lists_the_valve_decision_first() {
    let project = dossier();

    let value = json_of(&project, &["--json", "search", "--similar", "FND-001"]);
    let hits = value["data"].as_array().expect("the results");
    assert_eq!(hits[0]["title"], DECISIONS[0].0, "{value}");
}

#[test]
fn words_that_no_entry_holds_find_nothing() {
    let project = dossier();

    let stdout = project.stdout("", &["search", "xylophone", "marimba"]);
    assert_eq!(stdout, "0 matches\n");
    assert!(!stdout.contains("some of the words:"), "{stdout}");
}
