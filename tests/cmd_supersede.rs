//! Runs the built program for `supersede`: an approved entry is closed with the time and the ID
//! of the entry that replaces it, and the new entry points back to it. Refusals leave both files
//! and the journal as they were. Each test runs in its own temporary project folder
//! (common::Project), with the clock fixed at 2026-11-20T10:00:00+01:00.

mod common;

use common::Project;
use predicates::str::contains;

/// The text of the entry file with `id`.
fn entry_text(project: &Project, id: &str) -> String {
    std::fs::read_to_string(project.entry_file(id)).expect("the entry file")
}

/// The history lines of an entry file, in file order.
fn history_lines(text: &str) -> Vec<&str> {
    text.lines()
        .filter(|line| line.starts_with("- history: "))
        .collect()
}

/// Whether every line of `before` appears in `after` in the same order; lines may be added
/// between them. This is how a test checks that the other lines of a file keep their bytes.
fn keeps_lines(before: &str, after: &str) -> bool {
    let mut rest = after.lines();
    before
        .lines()
        .all(|line| rest.any(|candidate| candidate == line))
}

/// The journal lines written by `supersede`.
fn supersede_journal(project: &Project) -> Vec<String> {
    project
        .read("DOSSIER/.state/journal.jsonl")
        .lines()
        .filter(|line| line.contains("\"command\":\"supersede\""))
        .map(str::to_string)
        .collect()
}

/// Anna records an approved decision with the given text.
fn decide(project: &Project, decision: &str) {
    project.stdout(
        "anna",
        &["decide", decision, "--reason", "Because the test says so."],
    );
}

#[test]
fn anna_supersedes_a_decision_and_the_two_files_point_to_each_other() {
    let project = Project::new();
    decide(&project, "Use the transient runs.");
    decide(&project, "Use the transient runs with a finer tail.");
    let old_before = entry_text(&project, "DEC-001");
    let new_before = entry_text(&project, "DEC-002");

    let output = project.stdout("anna", &["supersede", "DEC-001", "--by", "dec-002"]);
    assert_eq!(output, "DEC-001: superseded by DEC-002\n");

    let old = entry_text(&project, "DEC-001");
    assert!(old.contains("\n- status: superseded\n"));
    assert!(!old.contains("- status: approved"));
    assert!(old.contains("\n- valid-from: 2026-11-20T10:00:00+01:00\n"));
    assert!(old.contains("\n- valid-until: 2026-11-20T10:00:00+01:00\n"));
    assert!(old.contains("\n- superseded-by: DEC-002\n"));
    assert!(
        old.contains("\n## Decision\n\nUse the transient runs.\n"),
        "the text of the old entry is kept"
    );
    assert_eq!(
        history_lines(&old),
        [
            "- history: 2026-11-20T10:00:00+01:00 anna: created",
            "- history: 2026-11-20T10:00:00+01:00 anna: approved",
            "- history: 2026-11-20T10:00:00+01:00 anna: superseded — by DEC-002",
        ]
    );
    assert!(
        keeps_lines(
            &old_before.replace("- status: approved\n", "- status: superseded\n"),
            &old
        ),
        "every other line of the old entry is kept"
    );

    let new = entry_text(&project, "DEC-002");
    assert!(new.contains("\n- status: approved\n"));
    assert!(new.contains("\n- supersedes: DEC-001\n"));
    assert!(!new.contains("valid-until"), "the new entry is still valid");
    assert_eq!(
        history_lines(&new),
        [
            "- history: 2026-11-20T10:00:00+01:00 anna: created",
            "- history: 2026-11-20T10:00:00+01:00 anna: approved",
            "- history: 2026-11-20T10:00:00+01:00 anna: changed — supersedes DEC-001",
        ]
    );
    assert!(
        keeps_lines(&new_before, &new),
        "the new entry only gains lines"
    );

    let journal = supersede_journal(&project);
    assert_eq!(journal.len(), 2, "one journal line per file: {journal:?}");
    assert!(journal[0].contains("\"time\":\"2026-11-20T10:00:00+01:00\""));
    assert!(journal[0].contains("\"person\":\"anna\""));
    assert!(journal[0].contains("\"llm\":false"));
    assert!(journal[0].contains("\"id\":\"DEC-001\""));
    assert!(journal[0].contains("\"event\":\"superseded\""));
    assert!(journal[0].contains("\"detail\":\"by DEC-002\""));
    assert!(journal[1].contains("\"id\":\"DEC-002\""));
    assert!(journal[1].contains("\"event\":\"changed\""));
    assert!(journal[1].contains("\"detail\":\"supersedes DEC-001\""));
}

#[test]
fn a_draft_can_neither_be_superseded_nor_replace_an_entry_and_nothing_is_written() {
    let project = Project::new();
    decide(&project, "Use the transient runs.");
    project.stdout(
        "anna",
        &[
            "add",
            "decision",
            "--title",
            "Finer tail",
            "--section",
            "Decision=Run a finer tail.",
            "--section",
            "Reason=Better resolution.",
            "--draft",
        ],
    );
    let old_before = entry_text(&project, "DEC-001");
    let draft_before = entry_text(&project, "DEC-002");
    assert!(draft_before.contains("\n- status: draft\n"));

    project
        .run("anna", &["supersede", "DEC-001", "--by", "DEC-002"])
        .failure()
        .code(1)
        .stderr(contains("DEC-002 is not approved (status: draft)"));
    project
        .run("anna", &["supersede", "DEC-002", "--by", "DEC-001"])
        .failure()
        .code(1)
        .stderr(contains("DEC-002 is not approved (status: draft)"));

    assert_eq!(entry_text(&project, "DEC-001"), old_before);
    assert_eq!(entry_text(&project, "DEC-002"), draft_before);
    assert!(supersede_journal(&project).is_empty());
}

#[test]
fn an_llm_cannot_supersede_and_nothing_is_written() {
    let project = Project::new();
    decide(&project, "Use the transient runs.");
    decide(&project, "Use the transient runs with a finer tail.");
    let old_before = entry_text(&project, "DEC-001");

    project
        .run("anna-claude", &["supersede", "DEC-001", "--by", "DEC-002"])
        .failure()
        .code(1)
        .stderr(contains("supersede is for a person; anna-claude is an LLM"));

    assert_eq!(entry_text(&project, "DEC-001"), old_before);
    assert!(!entry_text(&project, "DEC-002").contains("- supersedes:"));
    assert!(supersede_journal(&project).is_empty());
}

#[test]
fn an_entry_cannot_supersede_itself() {
    let project = Project::new();
    decide(&project, "Use the transient runs.");
    let before = entry_text(&project, "DEC-001");

    project
        .run("anna", &["supersede", "DEC-001", "--by", "DEC-001"])
        .failure()
        .code(1)
        .stderr(contains("DEC-001 cannot supersede itself"));

    assert_eq!(entry_text(&project, "DEC-001"), before);
    assert!(supersede_journal(&project).is_empty());
}

#[test]
fn entries_of_different_kinds_cannot_replace_each_other() {
    let project = Project::new();
    decide(&project, "Use the transient runs.");
    project.stdout(
        "anna",
        &[
            "assume",
            "The inlet profile is uniform.",
            "--reason",
            "No measured profile was delivered.",
        ],
    );
    let before = entry_text(&project, "DEC-001");

    project
        .run("anna", &["supersede", "DEC-001", "--by", "ASM-001"])
        .failure()
        .code(1)
        .stderr(contains("both entries must be of the same kind"));

    assert_eq!(entry_text(&project, "DEC-001"), before);
    assert!(supersede_journal(&project).is_empty());
}

#[test]
fn an_entry_that_is_already_superseded_cannot_be_superseded_again() {
    let project = Project::new();
    decide(&project, "Use the transient runs.");
    decide(&project, "Use the transient runs with a finer tail.");
    decide(&project, "Use the transient runs with a coarser tail.");
    project.stdout("anna", &["supersede", "DEC-001", "--by", "DEC-002"]);
    let after_first = entry_text(&project, "DEC-001");

    project
        .run("anna", &["supersede", "DEC-001", "--by", "DEC-003"])
        .failure()
        .code(1)
        .stderr(contains("DEC-001 is not approved (status: superseded)"));

    assert_eq!(entry_text(&project, "DEC-001"), after_first);
    assert!(!entry_text(&project, "DEC-003").contains("- supersedes:"));
    assert_eq!(supersede_journal(&project).len(), 2);
}

#[test]
fn an_unknown_entry_is_not_found_and_a_missing_by_is_a_usage_error() {
    let project = Project::new();
    decide(&project, "Use the transient runs.");

    project
        .run("anna", &["supersede", "DEC-001", "--by", "DEC-009"])
        .failure()
        .code(1)
        .stderr(contains("no entry DEC-009"));
    project
        .run("anna", &["supersede", "DEC-001"])
        .failure()
        .code(2);
    assert!(supersede_journal(&project).is_empty());
}

#[test]
fn the_time_of_the_run_is_the_valid_until_time() {
    let project = Project::new();
    decide(&project, "Use the transient runs.");
    decide(&project, "Use the transient runs with a finer tail.");

    project
        .run_at(
            "anna",
            "2026-12-01T08:30:00+01:00",
            &["supersede", "DEC-001", "--by", "DEC-002"],
        )
        .success();

    let old = entry_text(&project, "DEC-001");
    assert!(old.contains("\n- valid-until: 2026-12-01T08:30:00+01:00\n"));
    assert!(old.contains("- history: 2026-12-01T08:30:00+01:00 anna: superseded — by DEC-002"));
}
