//! Runs the built program for `promote`: a lesson, a finding and a how-to copied into the team
//! dossier as drafts linked both ways, the confidential check (a term found, no terms yet), the
//! refused kinds, an investigation that needs `--as` and must be solved first (R-INV-8), the
//! sections a team copy of an investigation gets from its attempts, pictures (confirmed at a
//! terminal, refused for an LLM) and a team that is not there. Each test runs in
//! its own temporary project folder (common::Project), with the clock fixed at
//! 2026-11-20T10:00:00+01:00.

mod common;

use common::Project;
use dossier::entry::{Entry, Kind, Status};
use predicates::str::contains;
use std::path::{Path, PathBuf};

/// The fixed clock of every run (the same as common::Project's).
const NOW: &str = "2026-11-20T10:00:00+01:00";

/// A term that is confidential in the project.
const TERM: &str = "Nordwind";

/// The statement of the lesson, free of the term.
const STATEMENT: &str = "A coarse mesh hides the shock position.";

/// A team dossier next to the project, made as anna, with anna as its member.
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

/// Links the project to the team dossier.
fn link(project: &Project, team: &Path) {
    let stored = team.display().to_string().replace('\\', "/");
    dossier::config::set_value(
        &project.root().join("DOSSIER"),
        "team",
        &format!("\"{stored}\""),
    )
    .expect("set the team path");
}

/// A project with a team, the confidential term added and a lesson (LES-001) by anna.
fn with_team() -> (Project, tempfile::TempDir, PathBuf) {
    let project = Project::new();
    let (folder, team) = team_dossier(&project);
    link(&project, &team);
    project.stdout("anna", &["confidential", "add", TERM]);
    project.stdout(
        "anna",
        &[
            "learn",
            STATEMENT,
            "--valid-for",
            "transonic cases",
            "--evidence",
            "Runs 4 to 6.",
            "--confidence",
            "measured",
            "--software",
            "ExampleSolver",
        ],
    );
    (project, folder, team)
}

/// The entry file with `id` in a dossier folder.
fn entry_in(dossier: &Path, id: &str) -> Entry {
    let prefix = format!("{id}-");
    for folder in std::fs::read_dir(dossier).expect("the dossier is read") {
        let folder = folder.expect("a folder").path();
        if !folder.is_dir() {
            continue;
        }
        for file in std::fs::read_dir(&folder).expect("the folder is read") {
            let file = file.expect("a file").path();
            let name = file.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if name.starts_with(&prefix) {
                let text = std::fs::read_to_string(&file).expect("the entry is read");
                return Entry::parse(&text).expect("the entry parses");
            }
        }
    }
    panic!("no entry {id} in {}", dossier.display());
}

/// Every file under `folder` with its bytes, sorted by path: a snapshot to compare before and after
/// a refused command.
fn files_of(folder: &Path) -> Vec<(PathBuf, Vec<u8>)> {
    let mut files = Vec::new();
    let mut pending = vec![folder.to_path_buf()];
    while let Some(next) = pending.pop() {
        for item in std::fs::read_dir(&next).expect("the folder is read") {
            let path = item.expect("a folder entry").path();
            if path.is_dir() {
                pending.push(path);
            } else {
                let bytes = std::fs::read(&path).expect("the file is read");
                files.push((path, bytes));
            }
        }
    }
    files.sort();
    files
}

/// Opens INV-001 as anna and closes it as solved (invented text).
fn solved_investigation(project: &Project) {
    project.stdout(
        "anna",
        &[
            "problem",
            "The density residual grows from iteration 40 on.",
        ],
    );
    project.stdout(
        "anna",
        &["solved", "INV-001", "The outlet flux had the wrong sign."],
    );
}

/// A PNG file's bytes (invented content).
fn png_bytes() -> Vec<u8> {
    let mut bytes = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
    bytes.extend_from_slice(b"invented picture data");
    bytes
}

/// Opens INV-001 as anna, records a failed attempt and a worked one (2), and closes it as solved
/// by the worked one (invented text).
fn investigation_with_attempts(project: &Project) {
    project.stdout(
        "anna",
        &[
            "problem",
            "The density residual grows from iteration 40 on.",
        ],
    );
    project.stdout(
        "anna",
        &[
            "tried",
            "INV-001",
            "The CFL number is too high",
            "--failed",
            "--did",
            "CFL 5 to 1",
            "--result",
            "diverges at iteration 180",
        ],
    );
    project.stdout(
        "anna",
        &[
            "tried",
            "INV-001",
            "The flux at the outlet has the wrong sign",
            "--worked",
            "--did",
            "corrected the sign in the outlet flux",
            "--result",
            "converges in 900 iterations",
        ],
    );
    project.stdout(
        "anna",
        &[
            "solved",
            "INV-001",
            "The outlet flux had the wrong sign.",
            "--attempt",
            "2",
        ],
    );
}

/// Adds a picture to LES-001 as anna.
fn add_picture(project: &Project, name: &str, caption: &str) {
    let path = project.root().join("plots").join(name);
    std::fs::create_dir_all(path.parent().expect("a folder")).expect("the folder");
    std::fs::write(&path, png_bytes()).expect("the picture");
    let relative = format!("plots/{name}");
    project.stdout(
        "anna",
        &["image", "add", "LES-001", &relative, "--caption", caption],
    );
}

#[test]
fn a_lesson_is_promoted_as_a_draft_linked_both_ways() {
    let (project, _folder, team) = with_team();
    let stdout = project.stdout("anna", &["promote", "LES-001"]);
    assert_eq!(
        stdout,
        "LES-001: promoted as team:LES-001 (draft, waiting for approval in the team dossier)\n"
    );

    let copy = entry_in(&team, "LES-001");
    assert_eq!(copy.status(), Some(Status::Draft));
    assert_eq!(copy.field("source").as_deref(), Some("P-001:LES-001"));
    assert_eq!(copy.field("confidence").as_deref(), Some("measured"));
    assert_eq!(copy.field("software").as_deref(), Some("ExampleSolver"));
    assert_eq!(copy.section("Statement").as_deref(), Some(STATEMENT));
    assert_eq!(
        copy.section("Valid for").as_deref(),
        Some("transonic cases")
    );

    let original = entry_in(&project.root().join("DOSSIER"), "LES-001");
    assert_eq!(original.field("promoted").as_deref(), Some("team:LES-001"));
    assert_eq!(original.status(), Some(Status::Approved));
    assert_eq!(
        original.history().last().map(String::as_str),
        Some("2026-11-20T10:00:00+01:00 anna: promoted — team:LES-001")
    );

    let journal = project.entry_journal();
    assert!(
        journal
            .iter()
            .any(|r| r.id == "LES-001" && r.event == "promoted")
    );
}

#[test]
fn promoting_twice_is_refused() {
    let (project, _folder, _team) = with_team();
    project.stdout("anna", &["promote", "LES-001"]);
    project
        .run("anna", &["promote", "LES-001"])
        .code(1)
        .stderr(contains("already promoted"));
}

#[test]
fn a_finding_becomes_a_lesson_with_its_observation_as_statement() {
    let (project, _folder, team) = with_team();
    project.stdout(
        "anna",
        &["found", "The inlet is uniform.", "--evidence", "Run 12."],
    );
    project.stdout("anna", &["promote", "FND-001"]);
    let copy = entry_in(&team, "LES-001");
    assert_eq!(
        copy.section("Statement").as_deref(),
        Some("The inlet is uniform.")
    );
    assert_eq!(copy.section("Evidence").as_deref(), Some("Run 12."));
    assert_eq!(copy.field("source").as_deref(), Some("P-001:FND-001"));
}

#[test]
fn a_decision_is_refused() {
    let (project, _folder, _team) = with_team();
    project.stdout(
        "anna",
        &["decide", "Use the fine mesh", "--reason", "Shock position."],
    );
    project
        .run("anna", &["promote", "DEC-001"])
        .code(1)
        .stderr(contains("only findings, lessons and how-tos are promoted"));
}

#[test]
fn a_lesson_cannot_be_promoted_as_a_howto() {
    let (project, _folder, _team) = with_team();
    project
        .run("anna", &["promote", "LES-001", "--as", "howto"])
        .code(1);
}

#[test]
fn a_lesson_with_a_confidential_term_is_refused_naming_the_line() {
    let (project, _folder, team) = with_team();
    project.stdout(
        "anna",
        &[
            "learn",
            "Nordwind needs a finer mesh.",
            "--valid-for",
            "transonic cases",
            "--evidence",
            "Runs 7.",
            "--confidence",
            "measured",
        ],
    );
    project
        .run("anna", &["promote", "LES-002"])
        .code(1)
        .stderr(contains("confidential terms found"))
        .stderr(contains("Nordwind (line"));
    assert!(
        !team
            .join("lessons")
            .read_dir()
            .is_ok_and(|mut d| d.next().is_some())
    );
    let original = entry_in(&project.root().join("DOSSIER"), "LES-002");
    assert_eq!(original.field("promoted"), None);
}

#[test]
fn a_project_with_no_terms_is_refused() {
    let project = Project::new();
    let (_folder, team) = team_dossier(&project);
    link(&project, &team);
    project.stdout(
        "anna",
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
    project
        .run("anna", &["promote", "LES-001"])
        .code(1)
        .stderr(contains("no confidential terms yet"));
}

#[test]
fn a_project_without_a_team_is_refused() {
    let project = Project::new();
    project.stdout("anna", &["confidential", "add", TERM]);
    project.stdout(
        "anna",
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
    project.run("anna", &["promote", "LES-001"]).failure();
}

#[test]
fn a_confirmed_picture_is_copied_and_a_declined_one_is_left_out() {
    let (project, _folder, team) = with_team();
    add_picture(&project, "contour.png", "Mach contours");
    add_picture(&project, "profile.png", "Inlet profile");
    project
        .command("anna", NOW)
        .env("DOSSIER_ANSWERS", "yes|no")
        .args(["promote", "LES-001"])
        .assert()
        .success();

    let copy = entry_in(&team, "LES-001");
    let pictures = copy.section("Pictures").expect("a Pictures section");
    assert_eq!(pictures, "![Mach contours](../images/LES-001/contour.png)");
    let folder = team.join("images").join("LES-001");
    assert_eq!(
        std::fs::read(folder.join("contour.png")).expect("the copied picture"),
        png_bytes()
    );
    assert!(!folder.join("profile.png").exists());
}

#[test]
fn an_investigation_without_as_is_a_usage_error_and_writes_nothing() {
    let (project, _folder, team) = with_team();
    solved_investigation(&project);
    let before = files_of(&team);
    project
        .run("anna", &["promote", "INV-001"])
        .code(2)
        .stderr(contains("add --as howto or --as lesson"));
    assert_eq!(files_of(&team), before, "the team dossier is not changed");
    let original = entry_in(&project.root().join("DOSSIER"), "INV-001");
    assert_eq!(original.field("promoted"), None);
}

#[test]
fn an_open_investigation_is_refused_and_the_team_dossier_is_not_written() {
    let (project, _folder, team) = with_team();
    project.stdout(
        "anna",
        &[
            "problem",
            "The density residual grows from iteration 40 on.",
        ],
    );
    let before = files_of(&team);
    project
        .run("anna", &["promote", "INV-001", "--as", "howto"])
        .code(1)
        .stderr(contains(
            "INV-001 is not solved yet; close it with dossier solved first",
        ));
    assert_eq!(files_of(&team), before, "the team dossier is not changed");
    let original = entry_in(&project.root().join("DOSSIER"), "INV-001");
    assert_eq!(original.field("promoted"), None);
}

#[test]
fn a_solved_investigation_is_promoted_as_a_lesson_draft_with_its_source() {
    let (project, _folder, team) = with_team();
    solved_investigation(&project);
    let stdout = project.stdout("anna", &["promote", "INV-001", "--as", "lesson"]);
    assert_eq!(
        stdout,
        "INV-001: promoted as team:LES-001 (draft, waiting for approval in the team dossier)\n"
    );

    let copy = entry_in(&team, "LES-001");
    assert_eq!(copy.kind(), Some(Kind::Lesson));
    assert_eq!(copy.status(), Some(Status::Draft));
    assert_eq!(copy.field("source").as_deref(), Some("P-001:INV-001"));

    let original = entry_in(&project.root().join("DOSSIER"), "INV-001");
    assert_eq!(original.field("promoted").as_deref(), Some("team:LES-001"));
}

#[test]
fn a_solved_investigation_is_promoted_as_a_howto_draft_with_its_source() {
    let (project, _folder, team) = with_team();
    solved_investigation(&project);
    project.stdout("anna", &["promote", "INV-001", "--as", "howto"]);

    let copy = entry_in(&team, "HOW-001");
    assert_eq!(copy.kind(), Some(Kind::Howto));
    assert_eq!(copy.status(), Some(Status::Draft));
    assert_eq!(copy.field("source").as_deref(), Some("P-001:INV-001"));
}

#[test]
fn a_solved_investigation_as_a_howto_has_its_attempts_as_steps_and_traps() {
    let (project, _folder, team) = with_team();
    investigation_with_attempts(&project);
    project.stdout("anna", &["promote", "INV-001", "--as", "howto"]);

    let copy = entry_in(&team, "HOW-001");
    assert_eq!(
        copy.section("Purpose").as_deref(),
        Some("The density residual grows from iteration 40 on.")
    );
    assert_eq!(
        copy.section("Steps").as_deref(),
        Some(
            "The outlet flux had the wrong sign.\n- The flux at the outlet has the wrong sign: corrected the sign in the outlet flux"
        )
    );
    assert_eq!(
        copy.section("Traps").as_deref(),
        Some("- The CFL number is too high: diverges at iteration 180")
    );
}

#[test]
fn a_how_to_draft_leaves_out_the_traps_when_no_attempt_failed() {
    let (project, _folder, team) = with_team();
    project.stdout(
        "anna",
        &[
            "problem",
            "The density residual grows from iteration 40 on.",
        ],
    );
    project.stdout(
        "anna",
        &["tried", "INV-001", "The outlet flux is wrong", "--worked"],
    );
    project.stdout(
        "anna",
        &["solved", "INV-001", "The outlet flux had the wrong sign."],
    );
    project.stdout("anna", &["promote", "INV-001", "--as", "howto"]);

    let copy = entry_in(&team, "HOW-001");
    assert_eq!(copy.section("Traps"), None);
    assert_eq!(
        copy.section("Steps").as_deref(),
        Some("The outlet flux had the wrong sign.\n- The outlet flux is wrong")
    );
}

#[test]
fn a_solved_investigation_as_a_lesson_has_its_attempts_as_evidence() {
    let (project, _folder, team) = with_team();
    investigation_with_attempts(&project);
    project.stdout("anna", &["promote", "INV-001", "--as", "lesson"]);

    let copy = entry_in(&team, "LES-001");
    assert_eq!(
        copy.section("Statement").as_deref(),
        Some("The outlet flux had the wrong sign.")
    );
    assert_eq!(
        copy.section("Evidence").as_deref(),
        Some(
            "The density residual grows from iteration 40 on.\n\n1. failed: The CFL number is too high — diverges at iteration 180\n2. worked: The flux at the outlet has the wrong sign — converges in 900 iterations"
        )
    );
}

#[test]
fn a_confidential_term_in_an_attempt_refuses_the_promotion_and_names_the_term() {
    let (project, _folder, team) = with_team();
    project.stdout(
        "anna",
        &[
            "problem",
            "The density residual grows from iteration 40 on.",
        ],
    );
    project.stdout(
        "anna",
        &[
            "tried",
            "INV-001",
            "The inlet of Nordwind is too coarse",
            "--failed",
        ],
    );
    project.stdout(
        "anna",
        &["solved", "INV-001", "The outlet flux had the wrong sign."],
    );
    let before = files_of(&team);
    project
        .run("anna", &["promote", "INV-001", "--as", "howto"])
        .code(1)
        .stderr(contains("confidential terms found"))
        .stderr(contains("Nordwind (line"));
    assert_eq!(files_of(&team), before, "the team dossier is not changed");
    let original = entry_in(&project.root().join("DOSSIER"), "INV-001");
    assert_eq!(original.field("promoted"), None);
}

#[test]
fn pictures_need_a_person_at_a_terminal() {
    let (project, _folder, team) = with_team();
    add_picture(&project, "contour.png", "Mach contours");
    project
        .run("anna-claude", &["promote", "LES-001"])
        .code(1)
        .stderr(contains("pictures"));
    assert!(
        !team
            .join("lessons")
            .read_dir()
            .is_ok_and(|mut d| d.next().is_some())
    );
    // A person without a terminal is refused too.
    project
        .run("anna", &["promote", "LES-001"])
        .code(1)
        .stderr(contains("pictures"));
}
