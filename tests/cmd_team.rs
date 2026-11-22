//! Runs the built program for `team`: the team settings are shown, and `team use` links a project
//! to its team dossier (R-PLACE-2) and refuses a folder that is not one. Each test runs in its own
//! project folder with the clock fixed by `common::Project` (2026-11-20T10:00:00+01:00). The team
//! dossier is made in a second folder whose name has a space and an umlaut.

mod common;

use common::Project;
use predicates::str::contains;
use serde_json::Value;
use std::path::{Path, PathBuf};

/// The fixed clock of `common::Project`.
const NOW: &str = "2026-11-20T10:00:00+01:00";

/// A team dossier made by anna in a folder of its own, with anna as its member. `dir` keeps the
/// folder alive; `dossier` is its DOSSIER folder.
struct Team {
    dir: tempfile::TempDir,
    dossier: PathBuf,
}

/// Makes a team dossier with `init --team-dossier --code TEAM` run as anna in a second folder,
/// then adds anna to its members (the team's settings are set by the tests, not by `team`).
fn team_dossier(project: &Project) -> Team {
    let dir = tempfile::Builder::new()
        .prefix("Team Äpfel ")
        .tempdir()
        .expect("a temporary team folder");
    project
        .command("anna", NOW)
        .current_dir(dir.path())
        .args(["init", "--team-dossier", "--code", "TEAM"])
        .assert()
        .success();
    let dossier = dir.path().join("DOSSIER");
    dossier::config::set_list(&dossier, "members", &["anna".to_string()])
        .expect("anna is a member of the team");
    Team { dir, dossier }
}

/// The path as dossier.toml stores it: with / separators.
fn slashed(path: &Path) -> String {
    path.to_str().expect("a UTF-8 path").replace('\\', "/")
}

/// The records of the project's journal, oldest first, settings events included.
fn journal(project: &Project) -> Vec<dossier::journal::Record> {
    dossier::journal::read(&project.root().join("DOSSIER").join(".state"))
        .expect("the journal reads")
}

/// The settings of the project, as text.
fn settings(project: &Project) -> String {
    project.read("DOSSIER/dossier.toml")
}

#[test]
fn team_use_links_the_project_and_team_shows_it() {
    let project = Project::new();
    let team = team_dossier(&project);
    let path = team.dossier.display().to_string();
    project
        .run("anna", &["team", "use", &path])
        .success()
        .stdout(format!("linked to the team dossier {path}\n"));

    let stored = slashed(&team.dossier);
    assert!(
        settings(&project).contains(&format!("team = \"{stored}\"\n")),
        "dossier.toml names the team: {}",
        settings(&project)
    );
    let cfg = dossier::config::load(&project.root().join("DOSSIER")).expect("settings load");
    assert_eq!(cfg.team.as_deref(), Some(stored.as_str()));

    // The team's member is now a member of the project, and the team dossier is shown.
    project
        .run("anna", &["team"])
        .success()
        .stdout(format!(
            "team dossier: {path}\nmembers: anna\nnames: none\n"
        ))
        .stderr("");
}

#[test]
fn team_use_is_recorded_in_the_journal_with_who_and_when() {
    let project = Project::new();
    let team = team_dossier(&project);
    let path = team.dossier.display().to_string();
    let before = journal(&project).len();
    project.run("anna", &["team", "use", &path]).success();

    let records = journal(&project);
    assert_eq!(records.len(), before + 1);
    let record = records.last().expect("the link is in the journal");
    assert_eq!(record.id, "SETTINGS");
    assert_eq!(record.event, "changed");
    assert_eq!(record.command, "team");
    assert_eq!(record.detail, format!("team {}", slashed(&team.dossier)));
    assert_eq!(record.name, "anna");
    assert_eq!(record.person, "anna");
    assert!(!record.llm);
    assert_eq!(record.time, NOW);
}

#[test]
fn team_use_accepts_the_folder_that_holds_the_dossier() {
    let project = Project::new();
    let team = team_dossier(&project);
    let holder = team.dir.path().display().to_string();
    project
        .run("anna", &["team", "use", &holder])
        .success()
        .stdout(format!(
            "linked to the team dossier {}\n",
            team.dossier.display()
        ));
    let cfg = dossier::config::load(&project.root().join("DOSSIER")).expect("settings load");
    assert_eq!(cfg.team, Some(slashed(&team.dossier)));
}

#[test]
fn team_use_takes_a_relative_path_from_the_project_folder() {
    let project = Project::new();
    let team = team_dossier(&project);
    // The team folder is next to the project folder: ../<its name>/DOSSIER from the project.
    let name = team
        .dir
        .path()
        .file_name()
        .and_then(|name| name.to_str())
        .expect("the team folder name is text");
    let relative = format!("../{name}/DOSSIER");
    project
        .run("anna", &["team", "use", &relative])
        .success()
        .stdout(format!(
            "linked to the team dossier {}\n",
            team.dossier.display()
        ));
    let cfg = dossier::config::load(&project.root().join("DOSSIER")).expect("settings load");
    assert_eq!(cfg.team, Some(slashed(&team.dossier)));
}

#[test]
fn a_folder_that_is_not_a_team_dossier_is_refused_and_nothing_changes() {
    let project = Project::new();
    let other = Project::new();
    let before = journal(&project).len();
    let settings_before = settings(&project);

    // Another project's DOSSIER, in its folder.
    project
        .run(
            "anna",
            &["team", "use", &other.root().display().to_string()],
        )
        .failure()
        .code(1)
        .stderr(contains("is not a team dossier"));
    // A folder with no dossier at all.
    let empty = tempfile::tempdir().expect("an empty folder");
    project
        .run(
            "anna",
            &["team", "use", &empty.path().display().to_string()],
        )
        .failure()
        .code(4)
        .stderr(contains("no team dossier here"));

    assert_eq!(settings(&project), settings_before);
    assert_eq!(journal(&project).len(), before);
}

#[test]
fn a_refused_team_use_as_json_gives_the_bad_value_name() {
    let project = Project::new();
    let other = Project::new();
    let out = project
        .command("anna", NOW)
        .args(["--json", "team", "use", &other.root().display().to_string()])
        .assert()
        .failure()
        .code(1)
        .get_output()
        .stdout
        .clone();
    let line: Value = serde_json::from_slice(&out).expect("one JSON line");
    assert_eq!(line["schema"], "dossier.team.v1");
    assert_eq!(line["ok"], false);
    assert_eq!(line["error"]["code"], "BAD_VALUE");
}

#[test]
fn team_use_with_dry_run_writes_nothing() {
    let project = Project::new();
    let team = team_dossier(&project);
    let before = journal(&project).len();
    let settings_before = settings(&project);
    project
        .command("anna", NOW)
        .args([
            "--dry-run",
            "team",
            "use",
            &team.dossier.display().to_string(),
        ])
        .assert()
        .success()
        .stdout(contains("linked to the team dossier"))
        .stderr(contains("dry run: would write"));
    assert_eq!(settings(&project), settings_before);
    assert_eq!(journal(&project).len(), before);
}

#[test]
fn team_use_inside_a_team_dossier_is_refused() {
    let project = Project::new();
    let team = team_dossier(&project);
    // Run from the team folder: the team dossier is the context, and it is not linked to a team.
    project
        .command("anna", NOW)
        .current_dir(team.dir.path())
        .args(["team", "use", &project.root().display().to_string()])
        .assert()
        .failure()
        .code(1)
        .stderr(contains("run team use in the project folder"));
    assert!(!settings_of(&team.dossier).contains("team ="));
}

/// The settings of a DOSSIER folder, as text.
fn settings_of(dossier: &Path) -> String {
    std::fs::read_to_string(dossier.join("dossier.toml")).expect("the settings read")
}

#[test]
fn team_without_a_link_says_there_is_no_team_dossier() {
    let project = Project::new();
    project
        .run("anna", &["team"])
        .success()
        .stdout("team dossier: none\nmembers: none\nnames: none\n")
        .stderr("");
}

#[test]
fn team_as_json_gives_the_team_dossier_members_and_names() {
    let project = Project::new();
    let team = team_dossier(&project);
    project
        .run(
            "anna",
            &["team", "use", &team.dossier.display().to_string()],
        )
        .success();
    let out = project.stdout("anna", &["--json", "team"]);
    let line: Value = serde_json::from_str(out.trim_end()).expect("one JSON line");
    assert_eq!(line["schema"], "dossier.team.v1");
    assert_eq!(line["ok"], true);
    assert_eq!(line["data"]["team"], slashed(&team.dossier));
    assert_eq!(line["data"]["members"], serde_json::json!(["anna"]));
    assert_eq!(line["data"]["names"], serde_json::json!([]));
    assert_eq!(line["warnings"], serde_json::json!([]));
}

#[test]
fn a_team_dossier_that_cannot_be_reached_is_a_warning_when_shown() {
    let project = Project::new();
    let team = team_dossier(&project);
    let path = team.dossier.display().to_string();
    project.run("anna", &["team", "use", &path]).success();
    std::fs::remove_dir_all(&team.dossier).expect("remove the team dossier");

    project
        .run("anna", &["team"])
        .success()
        .stdout(format!(
            "team dossier: {path}\nmembers: none\nnames: none\n"
        ))
        .stderr(contains("warning: the team dossier cannot be reached"));
}
