//! Runs the built program for `skill install`: the Claude Code skill `.claude/skills/dossier/
//! SKILL.md` of the current folder. Each test runs in its own temporary project folder
//! (common::Project, with the clock fixed and the people anna and ben), so that the journal lines
//! of a write can be read back. The folder names with spaces and non-ASCII letters are invented.

mod common;

use common::Project;
use predicates::str::contains;

/// The fixed clock of `common::Project`, as it appears in the journal.
const WHEN: &str = "2026-11-20T10:00:00+01:00";

/// The skill file, relative to the folder the program runs in.
const SKILL: &str = ".claude/skills/dossier/SKILL.md";

/// Whether a file or folder of the project folder exists.
fn exists(project: &Project, relative: &str) -> bool {
    project.root().join(relative).exists()
}

/// The journal lines of the project's dossier about the skill, oldest first.
fn skill_records(project: &Project) -> Vec<dossier::journal::Record> {
    let state = project.root().join("DOSSIER").join(".state");
    dossier::journal::read(&state)
        .unwrap()
        .into_iter()
        .filter(|record| record.id == "SKILL")
        .collect()
}

#[test]
fn install_with_yes_writes_the_skill_file_starting_with_the_front_matter() {
    let project = Project::new();
    project
        .run("anna", &["skill", "install", "--yes"])
        .success()
        .stdout(contains(".claude/skills/dossier/SKILL.md: skill written"));
    let text = project.read(SKILL);
    assert!(text.starts_with("---"), "{text}");
    assert!(text.starts_with("---\nname: dossier\n"), "{text}");
    assert_eq!(text, dossier::agents::skill());
}

#[test]
fn install_without_yes_says_what_would_be_written_and_writes_nothing() {
    let project = Project::new();
    project
        .run("anna", &["skill", "install"])
        .success()
        .stdout(contains(
            "would write .claude/skills/dossier/SKILL.md; add --yes",
        ));
    assert!(!exists(&project, ".claude"));
    assert!(skill_records(&project).is_empty());
}

#[test]
fn install_with_yes_adds_a_journal_line_with_the_agent_and_the_time() {
    let project = Project::new();
    project
        .run("anna", &["skill", "install", "--yes"])
        .success();
    let records = skill_records(&project);
    assert_eq!(records.len(), 1);
    let record = &records[0];
    assert_eq!(record.time, WHEN);
    assert_eq!(record.name, "anna");
    assert_eq!(record.person, "anna");
    assert!(!record.llm);
    assert_eq!(record.command, "skill");
    assert_eq!(record.id, "SKILL");
    assert_eq!(record.event, "changed");
    assert_eq!(record.detail, "skill written");
}

#[test]
fn a_second_install_writes_the_same_text_and_adds_another_journal_line() {
    let project = Project::new();
    project
        .run("anna", &["skill", "install", "--yes"])
        .success();
    project
        .run("anna", &["skill", "install", "--yes"])
        .success();
    assert_eq!(project.read(SKILL), dossier::agents::skill());
    assert_eq!(skill_records(&project).len(), 2);
}

#[test]
fn a_dry_run_with_yes_writes_nothing() {
    let project = Project::new();
    project
        .run("anna", &["skill", "install", "--yes", "--dry-run"])
        .success()
        .stdout(contains(
            "dry run: would write .claude/skills/dossier/SKILL.md",
        ));
    assert!(!exists(&project, ".claude"));
    assert!(skill_records(&project).is_empty());
}

#[test]
fn json_output_without_yes_reports_the_file_and_writes_nothing() {
    let project = Project::new();
    project
        .run("anna", &["skill", "install", "--json"])
        .success()
        .stdout(contains("\"schema\":\"dossier.skill.v1\""))
        .stdout(contains("\"written\":false"));
    assert!(!exists(&project, ".claude"));
}

#[test]
fn a_folder_with_spaces_and_non_ascii_letters_gets_the_file_and_the_dossier_above_gets_the_line() {
    let project = Project::new();
    let folder = project.root().join("Projekt Übersicht");
    std::fs::create_dir_all(&folder).unwrap();
    project
        .command("anna", WHEN)
        .current_dir(&folder)
        .args(["skill", "install", "--yes"])
        .assert()
        .success();
    let text = std::fs::read_to_string(
        folder
            .join(".claude")
            .join("skills")
            .join("dossier")
            .join("SKILL.md"),
    )
    .unwrap();
    assert_eq!(text, dossier::agents::skill());
    assert_eq!(skill_records(&project).len(), 1);
}

#[test]
fn outside_a_dossier_the_file_is_written_and_no_journal_line_is_added() {
    let project = Project::new();
    let outside = tempfile::TempDir::new().unwrap();
    project
        .command("anna", WHEN)
        .current_dir(outside.path())
        .args(["skill", "install", "--yes"])
        .assert()
        .success();
    let path = outside
        .path()
        .join(".claude")
        .join("skills")
        .join("dossier")
        .join("SKILL.md");
    assert!(path.is_file());
    assert!(skill_records(&project).is_empty());
}

#[test]
fn a_dossier_option_that_is_not_a_dossier_is_refused_before_anything_is_written() {
    let project = Project::new();
    std::fs::create_dir_all(project.root().join("plain")).unwrap();
    project
        .run("anna", &["skill", "install", "--yes", "--dossier", "plain"])
        .code(4)
        .stderr(contains("is not a dossier folder"));
    assert!(!exists(&project, ".claude"));
}
