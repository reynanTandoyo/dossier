//! Runs the built program for `close`: each approved finding, lesson and how-to of a project is
//! asked about in turn (`p` promotes it to the team dossier, `k` keeps it in the project, anything
//! else leaves it for later), each answer is kept, and a second run asks only what is left. Also:
//! drafts, decisions and assumptions are not asked, only a decider at a terminal may close, and
//! a promotion without a team stops the run with the answers before it kept. Each test runs in its
//! own temporary project folder (common::Project), with the clock fixed at 2026-11-20T10:00:00+01:00.

mod common;

use common::Project;
use dossier::entry::{Entry, Status};
use predicates::str::contains;
use serde_json::json;
use std::path::{Path, PathBuf};

/// The fixed clock of every run (the same as common::Project's).
const NOW: &str = "2026-11-20T10:00:00+01:00";

/// The confidential term of the project; the team copies must not contain it.
const TERM: &str = "NT-999";

/// The statement of the first lesson, free of the term.
const STATEMENT: &str = "A coarse mesh hides the shock position.";

/// The statement of the second lesson, free of the term.
const OTHER: &str = "Halving the time step did not remove the oscillation.";

/// The history line that anna's `k` adds to a lesson (format.md §6).
const KEPT_LINE: &str = "2026-11-20T10:00:00+01:00 anna: changed — kept in the project";

/// Runs `close` in the project as anna with `answers` as DOSSIER_ANSWERS. Asserts success and
/// returns the standard output and the standard error as text.
fn close(project: &Project, answers: &str) -> (String, String) {
    let output = project
        .command("anna", NOW)
        .env("DOSSIER_ANSWERS", answers)
        .arg("close")
        .assert()
        .success()
        .get_output()
        .clone();
    (
        String::from_utf8(output.stdout).expect("the standard output is UTF-8"),
        String::from_utf8(output.stderr).expect("the standard error is UTF-8"),
    )
}

/// Adds an approved lesson as anna. The statements of the tests are free of the term.
fn lesson(project: &Project, statement: &str) {
    project.stdout(
        "anna",
        &[
            "learn",
            statement,
            "--valid-for",
            "transonic cases",
            "--evidence",
            "Runs 4 to 6.",
            "--confidence",
            "measured",
        ],
    );
}

/// The entry with `id` in the DOSSIER folder `dossier`.
fn entry(dossier: &Path, id: &str) -> Entry {
    dossier::store::get(dossier, id)
        .unwrap_or_else(|error| panic!("reading {id}: {error:?}"))
        .entry
}

/// The DOSSIER folder of the project.
fn project_dossier(project: &Project) -> PathBuf {
    project.root().join("DOSSIER")
}

/// A team dossier in a folder of its own whose name has non-ASCII characters, made as anna, with
/// anna as its member. The folder is removed when the returned guard is dropped.
fn team_dossier(project: &Project) -> (tempfile::TempDir, PathBuf) {
    let folder = tempfile::Builder::new()
        .prefix("Team Äpfel ")
        .tempdir()
        .expect("a temporary team folder");
    project
        .command("anna", NOW)
        .current_dir(folder.path())
        .args(["init", "--code", "TEAM", "--team-dossier"])
        .assert()
        .success();
    let dossier = folder.path().join("DOSSIER");
    dossier::config::set_list(&dossier, "members", &["anna".to_string()])
        .expect("anna is a member of the team");
    (folder, dossier)
}

/// A project linked to a team dossier (`team use`), with the confidential term TERM. Returns the
/// project, the team's folder (kept by the caller) and the team's DOSSIER.
fn with_team() -> (Project, tempfile::TempDir, PathBuf) {
    let project = Project::new();
    let (folder, team) = team_dossier(&project);
    project.stdout("anna", &["team", "use", &team.display().to_string()]);
    project.stdout("anna", &["confidential", "add", TERM]);
    (project, folder, team)
}

#[test]
fn kept_and_left_answers_are_written_and_a_second_run_asks_only_what_is_left() {
    let project = Project::new();
    lesson(&project, STATEMENT);
    lesson(&project, OTHER);

    let (stdout, stderr) = close(&project, "k|l");
    assert_eq!(stdout, "0 promoted, 1 kept, 1 left for later\n");
    assert!(stderr.contains("# LES-001 ·"), "{stderr}");
    assert!(stderr.contains("# LES-002 ·"), "{stderr}");

    let dossier = project_dossier(&project);
    let first = entry(&dossier, "LES-001");
    assert_eq!(first.field("promote").as_deref(), Some("no"));
    assert_eq!(first.field("promoted"), None);
    assert_eq!(first.status(), Some(Status::Approved));
    assert_eq!(first.history().last().map(String::as_str), Some(KEPT_LINE));
    let second = entry(&dossier, "LES-002");
    assert_eq!(second.field("promote"), None);
    assert_eq!(second.history().len(), 2, "created and approved only");

    let journal = project.entry_journal();
    assert!(
        journal.iter().any(|record| record.id == "LES-001"
            && record.event == "changed"
            && record.detail == "kept in the project"),
        "{journal:?}"
    );

    // A second run asks only about LES-002.
    let (stdout, stderr) = close(&project, "l");
    assert_eq!(stdout, "0 promoted, 0 kept, 1 left for later\n");
    assert!(stderr.contains("# LES-002 ·"), "{stderr}");
    assert!(!stderr.contains("LES-001"), "{stderr}");
}

#[test]
fn a_kept_lesson_keeps_its_approval() {
    let project = Project::new();
    lesson(&project, STATEMENT);
    close(&project, "k");
    // The new `promote: no` line is not a change of content: the approval is recorded again.
    let shown = project.stdout("anna", &["show", "LES-001"]);
    assert!(!shown.contains("changed after approval"), "{shown}");
}

#[test]
fn a_promoted_entry_goes_to_the_team_and_is_not_asked_again() {
    let (project, _folder, team) = with_team();
    lesson(&project, STATEMENT);

    let (stdout, stderr) = close(&project, "p");
    assert_eq!(stdout, "1 promoted, 0 kept, 0 left for later\n");
    assert!(
        stderr.contains("LES-001: promoted as team:LES-001"),
        "{stderr}"
    );

    let copy = entry(&team, "LES-001");
    assert_eq!(copy.status(), Some(Status::Draft));
    assert_eq!(copy.field("source").as_deref(), Some("P-001:LES-001"));
    let original = entry(&project_dossier(&project), "LES-001");
    assert_eq!(original.field("promoted").as_deref(), Some("team:LES-001"));

    let (stdout, stderr) = close(&project, "p");
    assert_eq!(stdout, "0 promoted, 0 kept, 0 left for later\n");
    assert!(!stderr.contains("# LES-001 ·"), "{stderr}");
}

#[test]
fn a_promotion_without_a_team_stops_the_run_and_keeps_the_answers_before_it() {
    let project = Project::new();
    lesson(&project, STATEMENT);
    lesson(&project, OTHER);
    project
        .command("anna", NOW)
        .env("DOSSIER_ANSWERS", "k|p")
        .arg("close")
        .assert()
        .code(4)
        .stderr(contains("no team dossier"));

    let dossier = project_dossier(&project);
    let first = entry(&dossier, "LES-001");
    assert_eq!(first.field("promote").as_deref(), Some("no"));
    let second = entry(&dossier, "LES-002");
    assert_eq!(second.field("promote"), None);
    assert_eq!(second.field("promoted"), None);
}

#[test]
fn drafts_decisions_and_assumptions_are_not_asked() {
    let project = Project::new();
    project.stdout(
        "anna",
        &["decide", "Use the fine mesh", "--reason", "Shock position."],
    );
    project.stdout(
        "anna",
        &[
            "assume",
            "The inlet is uniform.",
            "--reason",
            "No profile was delivered.",
        ],
    );
    // An LLM's lesson is a draft until a person approves it.
    project.stdout(
        "anna-claude",
        &[
            "learn",
            STATEMENT,
            "--valid-for",
            "transonic cases",
            "--evidence",
            "Runs 4 to 6.",
            "--confidence",
            "measured",
        ],
    );
    project.stdout(
        "anna",
        &[
            "found",
            "The mass flow still oscillates.",
            "--evidence",
            "Run 12.",
        ],
    );

    let (stdout, stderr) = close(&project, "k");
    assert_eq!(stdout, "0 promoted, 1 kept, 0 left for later\n");
    assert!(stderr.contains("# FND-001 ·"), "{stderr}");
    for other in ["DEC-001", "ASM-001", "LES-001"] {
        assert!(!stderr.contains(&format!("# {other} ·")), "{stderr}");
    }
}

#[test]
fn an_approved_how_to_is_asked_about_too() {
    let project = Project::new();
    project.stdout(
        "anna",
        &[
            "howto",
            "--software",
            "ExampleSolver",
            "--title",
            "Run a speedline",
            "--section",
            "Purpose=Runs the speedline points.",
            "--section",
            "Steps=Set the mass flow and run.",
        ],
    );
    let (stdout, stderr) = close(&project, "k");
    assert_eq!(stdout, "0 promoted, 1 kept, 0 left for later\n");
    assert!(stderr.contains("# HOW-001 ·"), "{stderr}");
    assert_eq!(
        entry(&project_dossier(&project), "HOW-001")
            .field("promote")
            .as_deref(),
        Some("no")
    );
}

#[test]
fn only_a_decider_at_a_terminal_closes_a_project() {
    let project = Project::new();
    lesson(&project, STATEMENT);

    // An LLM is refused before anything is asked.
    project
        .run("anna-claude", &["close"])
        .code(1)
        .stderr(contains("is an LLM"));
    // Without a terminal nothing is asked.
    project
        .run("anna", &["close"])
        .code(1)
        .stderr(contains("person at a terminal"));
    // A person who is not in deciders is refused, and nothing is written.
    dossier::config::set_list(&project_dossier(&project), "deciders", &["ben".to_string()])
        .expect("ben decides");
    project
        .command("anna", NOW)
        .env("DOSSIER_ANSWERS", "k")
        .arg("close")
        .assert()
        .code(1)
        .stderr(contains("anna is not in deciders"));
    assert_eq!(
        entry(&project_dossier(&project), "LES-001").field("promote"),
        None
    );
}

#[test]
fn the_json_output_lists_each_answer() {
    let project = Project::new();
    lesson(&project, STATEMENT);
    lesson(&project, OTHER);
    let output = project
        .command("anna", NOW)
        .env("DOSSIER_ANSWERS", "k|l")
        .args(["close", "--json"])
        .assert()
        .success()
        .get_output()
        .clone();
    let stdout = String::from_utf8(output.stdout).expect("the standard output is UTF-8");
    let value: serde_json::Value = serde_json::from_str(stdout.trim()).expect("one JSON line");
    assert_eq!(value["schema"], "dossier.close.v1");
    assert_eq!(value["ok"], true);
    assert_eq!(value["data"]["promoted"], json!([]));
    assert_eq!(value["data"]["kept"], json!(["LES-001"]));
    assert_eq!(value["data"]["left"], json!(["LES-002"]));
}
