//! Runs the built program for `check`: a clean dossier is ok; each kind of problem is printed on
//! its own line and the command exits with 1, with and without `--json`. Each test runs in its
//! own project folder, with the clock fixed by `common::Project`.

mod common;

use common::Project;
use serde_json::Value;
use std::fs;
use std::path::Path;

/// Invented text for a decision, a lesson and a finding.
const DECISION: &str = "Use the transient runs for the last three points.";
const REASON: &str = "Steady runs converged where the transient run already showed rotating stall.";
const STATEMENT: &str = "Steady runs with a mixing-plane interface overpredict the stall margin.";
const VALID_FOR: &str = "An invented test rig with a tip Mach number above 1.1.";
const EVIDENCE: &str = "An invented comparison of two speedlines, section 1.";
const OBSERVATION: &str = "The mass flow still oscillated with half the time step.";

/// The output of `check` when there is no problem.
const OK: &str = "DOSSIER: ok (0 problems)\n";

/// Records the decision DEC-001 by anna: approved.
fn decide(project: &Project) {
    project.stdout("anna", &["decide", DECISION, "--reason", REASON]);
}

/// Records a lesson with the statement STATEMENT as `agent`: approved for a person, a draft for
/// an LLM.
fn learn_by(project: &Project, agent: &str) {
    project.stdout(
        agent,
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
            "Steady mixing-plane runs overpredict stall margin",
        ],
    );
}

/// The text of the entry file with `id`.
fn text_of(project: &Project, id: &str) -> String {
    fs::read_to_string(project.entry_file(id)).expect("the entry file")
}

/// Writes the entry file with `id` by hand, as a person would edit it.
fn write_entry(project: &Project, id: &str, text: &str) {
    fs::write(project.entry_file(id), text).expect("the entry is changed by hand");
}

/// The path that `check` prints for `file`: relative to the project folder, with `/`.
fn shown(file: &Path) -> String {
    let folder = file
        .parent()
        .and_then(Path::file_name)
        .and_then(|name| name.to_str())
        .expect("the kind folder");
    let name = file
        .file_name()
        .and_then(|name| name.to_str())
        .expect("the file name");
    format!("DOSSIER/{folder}/{name}")
}

/// The line number (1-based) of the first line of `text` that is exactly `line`.
fn line_of(text: &str, line: &str) -> usize {
    text.lines()
        .position(|candidate| candidate == line)
        .expect("the line is in the text")
        + 1
}

/// Runs `check` as anna, where it must find problems: asserts exit code 1 and returns the
/// standard error text.
fn problems(project: &Project) -> String {
    let stderr = project
        .run("anna", &["check"])
        .code(1)
        .get_output()
        .stderr
        .clone();
    String::from_utf8(stderr).expect("the standard error is UTF-8")
}

#[test]
fn a_clean_dossier_with_two_entries_is_ok() {
    let project = Project::new();
    assert_eq!(project.stdout("anna", &["check"]), OK);

    decide(&project);
    project.stdout(
        "ben",
        &["found", OBSERVATION, "--evidence", "Runs 12 and 13."],
    );

    assert_eq!(project.stdout("anna", &["check"]), OK);
}

#[test]
fn a_clean_dossier_is_ok_in_json_too() {
    let project = Project::new();
    decide(&project);

    let stdout = project.stdout("anna", &["check", "--json"]);
    let value: Value = serde_json::from_str(stdout.trim()).expect("one line of JSON");

    assert_eq!(value["schema"], "dossier.check.v1");
    assert_eq!(value["ok"], true);
    assert!(value["data"].is_null());
}

#[test]
fn approved_lessons_and_drafts_by_an_llm_are_ok() {
    let project = Project::new();
    learn_by(&project, "anna");
    learn_by(&project, "anna-claude");

    assert_eq!(project.stdout("anna", &["check"]), OK);
}

#[test]
fn a_file_renamed_to_another_id_gives_name() {
    let project = Project::new();
    decide(&project);
    let file = project.entry_file("DEC-001");
    let renamed = file.with_file_name(
        file.file_name()
            .and_then(|name| name.to_str())
            .expect("a file name")
            .replacen("DEC-001", "DEC-009", 1),
    );
    fs::rename(&file, &renamed).expect("the file is renamed");

    assert_eq!(
        problems(&project),
        format!(
            "error: {}:1: DEC-001 NAME file name does not start with DEC-001-\nDOSSIER: 1 problems\n",
            shown(&renamed)
        )
    );
}

#[test]
fn a_related_field_naming_a_missing_entry_gives_link() {
    let project = Project::new();
    decide(&project);
    let text = text_of(&project, "DEC-001");
    let changed = text.replacen("- history:", "- related: DEC-999\n- history:", 1);
    write_entry(&project, "DEC-001", &changed);

    let line = line_of(&changed, "- related: DEC-999");
    assert_eq!(
        problems(&project),
        format!(
            "error: {}:{line}: DEC-001 LINK related names DEC-999, which does not exist\nDOSSIER: 1 problems\n",
            shown(&project.entry_file("DEC-001"))
        )
    );
}

#[test]
fn an_approved_lesson_edited_by_hand_gives_changed() {
    let project = Project::new();
    learn_by(&project, "anna");
    let text = text_of(&project, "LES-001");
    let changed = text.replace(
        "overpredict the stall margin",
        "underpredict the stall margin",
    );
    assert_ne!(changed, text);
    write_entry(&project, "LES-001", &changed);

    assert_eq!(
        problems(&project),
        format!(
            "error: {}:1: LES-001 CHANGED approved lesson changed after approval\nDOSSIER: 1 problems\n",
            shown(&project.entry_file("LES-001"))
        )
    );
}

#[test]
fn an_entry_with_the_author_line_deleted_gives_who() {
    let project = Project::new();
    decide(&project);
    let text = text_of(&project, "DEC-001");
    let without = text.replacen("- author: anna\n", "", 1);
    assert_ne!(without, text);
    write_entry(&project, "DEC-001", &without);

    assert_eq!(
        problems(&project),
        format!(
            "error: {}:1: DEC-001 WHO missing field author\nDOSSIER: 1 problems\n",
            shown(&project.entry_file("DEC-001"))
        )
    );
}

#[test]
fn a_file_that_does_not_parse_gives_parse_with_no_id() {
    let project = Project::new();
    project.write(
        "DOSSIER/findings/FND-005-broken.md",
        "<!-- dossier-format: 1 -->\nthis file has no heading\n",
    );

    assert_eq!(
        problems(&project),
        "error: DOSSIER/findings/FND-005-broken.md:2: - PARSE no heading\nDOSSIER: 1 problems\n"
    );
}

#[test]
fn an_unknown_status_gives_status() {
    let project = Project::new();
    decide(&project);
    let text = text_of(&project, "DEC-001");
    let changed = text.replacen("- status: approved", "- status: maybe", 1);
    write_entry(&project, "DEC-001", &changed);

    let line = line_of(&changed, "- status: maybe");
    assert_eq!(
        problems(&project),
        format!(
            "error: {}:{line}: DEC-001 STATUS status must be draft, approved, rejected, superseded or retired, not \"maybe\"\nDOSSIER: 1 problems\n",
            shown(&project.entry_file("DEC-001"))
        )
    );
}

#[test]
fn a_created_time_that_is_not_a_time_gives_time() {
    let project = Project::new();
    decide(&project);
    let text = text_of(&project, "DEC-001");
    let changed = text.replacen(
        "- created: 2026-11-20T10:00:00+01:00",
        "- created: yesterday",
        1,
    );
    assert_ne!(changed, text);
    write_entry(&project, "DEC-001", &changed);

    let line = line_of(&changed, "- created: yesterday");
    assert_eq!(
        problems(&project),
        format!(
            "error: {}:{line}: DEC-001 TIME created is not a time: \"yesterday\"\nDOSSIER: 1 problems\n",
            shown(&project.entry_file("DEC-001"))
        )
    );
}

#[test]
fn two_files_with_one_id_give_duplicate_on_each() {
    let project = Project::new();
    decide(&project);
    let original = project.entry_file("DEC-001");
    let copy = original.with_file_name("DEC-001-copy.md");
    fs::copy(&original, &copy).expect("the entry is copied");

    let stderr = problems(&project);
    let (original, copy) = (shown(&original), shown(&copy));
    assert!(
        stderr.contains(&format!(
            "{original}:2: DEC-001 DUPLICATE the same ID is also in {copy}\n"
        )),
        "{stderr}"
    );
    assert!(
        stderr.contains(&format!(
            "{copy}:2: DEC-001 DUPLICATE the same ID is also in {original}\n"
        )),
        "{stderr}"
    );
    assert!(stderr.ends_with("DOSSIER: 2 problems\n"), "{stderr}");
}

#[test]
fn an_approved_entry_without_a_required_section_gives_incomplete() {
    let project = Project::new();
    decide(&project);
    let text = text_of(&project, "DEC-001");
    let changed = text.replacen("## Reason\n", "## Reasons\n", 1);
    assert_ne!(changed, text);
    write_entry(&project, "DEC-001", &changed);

    assert_eq!(
        problems(&project),
        format!(
            "error: {}:1: DEC-001 INCOMPLETE missing section Reason\nDOSSIER: 1 problems\n",
            shown(&project.entry_file("DEC-001"))
        )
    );
}

#[test]
fn problems_fail_with_exit_one_and_refused_in_json() {
    let project = Project::new();
    decide(&project);
    let text = text_of(&project, "DEC-001");
    write_entry(
        &project,
        "DEC-001",
        &text.replacen("- author: anna\n", "", 1),
    );

    let stdout = project
        .run("anna", &["check", "--json"])
        .code(1)
        .get_output()
        .stdout
        .clone();
    let stdout = String::from_utf8(stdout).expect("the standard output is UTF-8");
    let value: Value = serde_json::from_str(stdout.trim()).expect("one line of JSON");

    assert_eq!(value["schema"], "dossier.check.v1");
    assert_eq!(value["ok"], false);
    assert_eq!(value["error"]["code"], "REFUSED");
    let message = value["error"]["message"]
        .as_str()
        .expect("the message is text");
    assert!(message.ends_with("DOSSIER: 1 problems"), "{message}");
    assert!(message.contains(" WHO missing field author\n"), "{message}");
}
